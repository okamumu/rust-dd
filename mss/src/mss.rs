//! Multi-state system manager: owns an [`MddMgr`] (structure functions over MTMDD2) and a
//! [`ZmddMgr`] (minimal-path-vector families), and provides the analysis that spans both —
//! `minpath`, which returns the family as a genuine [`ZmddNode`].
//!
//! Build expressions through the delegated MDD API ([`defvar`](MssMgr::defvar),
//! [`rpn`](MssMgr::rpn), [`min`](MssMgr::min)/[`max`](MssMgr::max), or the [`MddNode`] operators),
//! then call [`minpath`](MssMgr::minpath) and combine families with `intersect` / `setdiff`.

use mddcore::prelude::*;
use std::collections::HashMap;

use crate::mdd::{MddMgr, MddNode};
use crate::mdd_minsol;
use crate::zmdd::{ZmddMgr, ZmddNode};

/// `φ` evaluated at the extreme state vector: every component at state 0 (`top = false`) or at
/// its maximum state (`top = true`). Walks one edge per level, so it is `O(depth)`.
///
/// Used to label the **baseline member** of a family: the empty sparse vector of a path family
/// is the all-0 point, that of a cut family is the all-max point.
fn eval_extreme<V>(src_rc: &std::rc::Rc<std::cell::RefCell<MtMdd2Manager<V>>>, node: &Node, top: bool) -> V
where
    V: MddValue,
{
    let src = src_rc.borrow();
    match node {
        Node::Value(id) => {
            let mut cur = *id;
            loop {
                match src.mtmdd().get_node(&cur).unwrap() {
                    mddcore::mtmdd::Node::Terminal(t) => return t.value(),
                    mddcore::mtmdd::Node::Undet => return V::from(0),
                    mddcore::mtmdd::Node::NonTerminal(f) => {
                        let edges: Vec<_> = f.iter().collect();
                        cur = if top { edges[edges.len() - 1] } else { edges[0] };
                    }
                }
            }
        }
        Node::Bool(id) => {
            let mut cur = *id;
            loop {
                match src.mdd().get_node(&cur).unwrap() {
                    mddcore::mdd::Node::One => return V::from(1),
                    mddcore::mdd::Node::Zero | mddcore::mdd::Node::Undet => return V::from(0),
                    mddcore::mdd::Node::NonTerminal(f) => {
                        let edges: Vec<_> = f.iter().collect();
                        cur = if top { edges[edges.len() - 1] } else { edges[0] };
                    }
                }
            }
        }
    }
}

/// Owns an [`MddMgr`] (boolean/value structure functions) and a [`ZmddMgr`] (set families).
pub struct MssMgr<V> {
    mdd: MddMgr<V>,
    zmdd: ZmddMgr<V>,
}

impl<V> MssMgr<V>
where
    V: MddValue,
{
    pub fn new() -> Self {
        MssMgr {
            mdd: MddMgr::new(),
            zmdd: ZmddMgr::new(),
        }
    }

    /// The underlying MDD manager (structure functions).
    pub fn mdd(&self) -> &MddMgr<V> {
        &self.mdd
    }

    /// The underlying ZMDD manager (set families).
    pub fn zmdd(&self) -> &ZmddMgr<V> {
        &self.zmdd
    }

    // --- MDD building, delegated to the inner MddMgr -------------------------

    pub fn defvar(&mut self, label: &str, range: usize) -> MddNode<V> {
        self.mdd.defvar(label, range)
    }

    pub fn rpn(&mut self, rpn: &str, vars: &HashMap<String, usize>) -> Result<MddNode<V>, String> {
        self.mdd.rpn(rpn, vars)
    }

    pub fn value(&self, value: V) -> MddNode<V> {
        self.mdd.value(value)
    }

    pub fn boolean(&self, other: bool) -> MddNode<V> {
        self.mdd.boolean(other)
    }

    pub fn undet_boolean(&self) -> MddNode<V> {
        self.mdd.undet_boolean()
    }

    pub fn undet_value(&self) -> MddNode<V> {
        self.mdd.undet_value()
    }

    pub fn create_node(&self, h: HeaderId, nodes: &[MddNode<V>]) -> MddNode<V> {
        self.mdd.create_node(h, nodes)
    }

    pub fn and(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        self.mdd.and(nodes)
    }

    pub fn or(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        self.mdd.or(nodes)
    }

    pub fn min(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        self.mdd.min(nodes)
    }

    pub fn max(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        self.mdd.max(nodes)
    }

    pub fn get_varorder(&self) -> Vec<(String, usize)> {
        self.mdd.get_varorder()
    }

    pub fn size(&self) -> (usize, usize, usize, usize) {
        self.mdd.size()
    }

    pub fn set_gc_threshold(&self, threshold: usize) {
        self.mdd.set_gc_threshold(threshold);
    }

    pub fn live_node_count(&self) -> usize {
        self.mdd.live_node_count()
    }

    pub fn gc(&self, keep: &[&MddNode<V>]) -> (usize, usize) {
        self.mdd.gc(keep)
    }

    pub fn clear_cache(&mut self) {
        self.mdd.clear_cache();
        self.zmdd.clear_cache();
    }

    // --- minpath: minsol (MTMDD2) -> genuine ZMDD family --------------------

    /// Minimal path vectors of the structure function `node` as a genuine ZMDD family
    /// ([`ZmddNode`]), or `None` if the function is not coherent (monotone).
    ///
    /// The minsol runs on the MTMDD2 forest; the fake-ZMDD result is converted (internally,
    /// once) into this manager's [`ZmddMgr`], so the returned [`ZmddNode`] supports the
    /// label-wise set operations `intersect` / `setdiff`.
    ///
    /// **Reading the result.** Each vector is filed under the terminal label equal to **its own
    /// `φ(x)`**, so [`extract([v])`](ZmddNode::extract) returns `minimal{x : φ(x) == v}`, not
    /// the classical `minimal{x : φ(x) >= v}` — use [`extract_level`](ZmddNode::extract_level)
    /// for that. The two agree at the extreme labels and can differ in between. Vectors are
    /// reported dense (every variable present, unrecorded components at state 0). The family
    /// always contains the **baseline member**, the all-0 vector, at label `φ(0, .., 0)`; it is
    /// a correct but trivial minimal path vector, so callers usually skip it.
    pub fn minpath(&self, node: &MddNode<V>) -> Option<ZmddNode<V>> {
        let src_rc = node.get_mgr();
        let tag = node.get_node();
        let fake = {
            let mut m = src_rc.borrow_mut();
            mdd_minsol::minsol(&mut m, &tag)
        };
        let vars = std::rc::Rc::new(self.mdd.get_varorder());
        let baseline = eval_extreme(&src_rc, &tag, false);
        fake.map(|f| self.zmdd.convert(&src_rc, &f, vars.clone(), baseline))
    }

    /// Minimal **cut** vectors of the structure function `node` as a genuine ZMDD family
    /// ([`ZmddNode`]), or `None` if the function is not coherent (monotone).
    ///
    /// Computed directly by `maxsol` (the top-baseline mirror of `minsol`) — the dual `φ^D`
    /// is never materialized, avoiding the expensive multi-state edge/value reversal. The
    /// resulting family is a **cut** ZMDD: the terminal label is the resulting performance
    /// value in `φ`'s own scale, and a component the vector does not push down sits at its max
    /// state (so a boolean fault-tree failure is read with `extract([0])`).
    ///
    /// **Reading the result.** Each vector is filed under the terminal label equal to **its own
    /// `φ(x)`**, so [`extract([v])`](ZmddNode::extract) returns `maximal{x : φ(x) == v}`, not
    /// the classical `maximal{x : φ(x) <= v}` — use [`extract_level`](ZmddNode::extract_level)
    /// for that. A vector with `φ(x) < v` can still be maximal within `{x : φ(x) <= v}` and it
    /// lives in a lower stratum, so the two readings differ at intermediate levels (they agree
    /// at the extreme labels). Vectors are reported dense (every variable present, components
    /// that are not pushed down at their max state). The family always contains the **baseline
    /// member**, the all-max vector, at label `φ(max, .., max)`; it is a correct but trivial
    /// cut vector, so callers usually skip it.
    pub fn mincut(&self, node: &MddNode<V>) -> Option<ZmddNode<V>> {
        let src_rc = node.get_mgr();
        let tag = node.get_node();
        let fake = {
            let mut m = src_rc.borrow_mut();
            mdd_minsol::maxsol(&mut m, &tag)
        };
        let vars = std::rc::Rc::new(self.mdd.get_varorder());
        let baseline = eval_extreme(&src_rc, &tag, true);
        fake.map(|f| self.zmdd.convert_rev(&src_rc, &f, vars.clone(), baseline))
    }
}
