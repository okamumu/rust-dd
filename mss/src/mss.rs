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
    pub fn minpath(&self, node: &MddNode<V>) -> Option<ZmddNode<V>> {
        let src_rc = node.get_mgr();
        let tag = node.get_node();
        let fake = {
            let mut m = src_rc.borrow_mut();
            mdd_minsol::minsol(&mut m, &tag)
        };
        fake.map(|f| self.zmdd.convert(&src_rc, &f))
    }
}
