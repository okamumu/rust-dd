//! Ergonomic wrapper over [`ZmddManager`]: `ZmddMgr` / `ZmddNode` for the set families of
//! minimal path vectors. Families are produced by [`MssMgr::minpath`](crate::mss::MssMgr::minpath)
//! and support the label-wise set operations `intersect` / `setdiff` (see `mddcore::zmdd_ops`),
//! plus `count` / `extract`.

use mddcore::prelude::*;
use mddcore::mtmdd::Node as ZNode;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

use crate::zmdd_convert;

const GC_FLOOR: usize = 1 << 16;

#[derive(Debug)]
struct GcState {
    roots: BddHashMap<NodeId, u32>,
    threshold: usize,
    floor: usize,
}

fn maybe_gc<V>(zmdd: &Rc<RefCell<ZmddManager<V>>>, gc: &Rc<RefCell<GcState>>)
where
    V: MddValue,
{
    if zmdd.borrow().live_node_count() < gc.borrow().threshold {
        return;
    }
    let roots: Vec<NodeId> = gc.borrow().roots.keys().copied().collect();
    let live = {
        let mut z = zmdd.borrow_mut();
        z.gc(&roots);
        z.live_node_count()
    };
    let mut s = gc.borrow_mut();
    s.threshold = live.saturating_mul(2).max(s.floor);
}

/// Manager (forest owner) for **minimal path/cut vector families** (ZMDDs). Owned by an
/// [`MssMgr`](crate::mss::MssMgr); `minpath` results live here. Two families must come from
/// the same `ZmddMgr` to be combined with the [`ZmddNode`] set operations.
pub struct ZmddMgr<V> {
    zmdd: Rc<RefCell<ZmddManager<V>>>,
    gc: Rc<RefCell<GcState>>,
}

impl<V> ZmddMgr<V>
where
    V: MddValue,
{
    pub fn new() -> Self {
        ZmddMgr {
            zmdd: Rc::new(RefCell::new(ZmddManager::new())),
            gc: Rc::new(RefCell::new(GcState {
                roots: BddHashMap::default(),
                threshold: GC_FLOOR,
                floor: GC_FLOOR,
            })),
        }
    }

    fn wrap(&self, node: NodeId) -> ZmddNode<V> {
        let n = ZmddNode::new(&self.zmdd, &self.gc, node);
        maybe_gc(&self.zmdd, &self.gc);
        n
    }

    /// Convert a minsol result (the fake-ZMDD `Node` in a `MtMdd2Manager`) into a genuine
    /// ZMDD family in this manager. Used by [`MssMgr::minpath`](crate::mss::MssMgr::minpath).
    pub(crate) fn convert(&self, src_rc: &Rc<RefCell<MtMdd2Manager<V>>>, node: &Node) -> ZmddNode<V> {
        let id = {
            let src = src_rc.borrow();
            let mut dst = self.zmdd.borrow_mut();
            zmdd_convert::to_zmdd(&src, node, &mut dst)
        };
        self.wrap(id)
    }

    pub fn size(&self) -> (usize, usize, usize, usize) {
        self.zmdd.borrow().size()
    }

    pub fn clear_cache(&mut self) {
        self.zmdd.borrow_mut().clear_cache();
    }
}

/// A handle to a set family in a [`ZmddMgr`]'s forest.
#[derive(Debug)]
pub struct ZmddNode<V> {
    parent: Weak<RefCell<ZmddManager<V>>>,
    gc: Weak<RefCell<GcState>>,
    node: NodeId,
}

impl<V> ZmddNode<V>
where
    V: MddValue,
{
    fn from_weak(
        parent: Weak<RefCell<ZmddManager<V>>>,
        gc: Weak<RefCell<GcState>>,
        node: NodeId,
    ) -> Self {
        if let Some(g) = gc.upgrade() {
            *g.borrow_mut().roots.entry(node).or_insert(0) += 1;
        }
        ZmddNode { parent, gc, node }
    }

    fn new(zmdd: &Rc<RefCell<ZmddManager<V>>>, gc: &Rc<RefCell<GcState>>, node: NodeId) -> Self {
        Self::from_weak(Rc::downgrade(zmdd), Rc::downgrade(gc), node)
    }

    fn rewrap(&self, zmdd: &Rc<RefCell<ZmddManager<V>>>, node: NodeId) -> Self {
        let n = ZmddNode::from_weak(self.parent.clone(), self.gc.clone(), node);
        if let Some(gc) = self.gc.upgrade() {
            maybe_gc(zmdd, &gc);
        }
        n
    }

    pub fn get_mgr(&self) -> Rc<RefCell<ZmddManager<V>>> {
        self.parent.upgrade().unwrap()
    }

    pub fn get_id(&self) -> NodeId {
        self.node
    }

    /// Label-wise intersection with another family from the same manager.
    pub fn intersect(&self, other: &ZmddNode<V>) -> ZmddNode<V> {
        let zmdd = self.parent.upgrade().unwrap();
        let result = zmdd.borrow_mut().intersect(self.node, other.node);
        self.rewrap(&zmdd, result)
    }

    /// Label-wise difference (`self − other`).
    pub fn setdiff(&self, other: &ZmddNode<V>) -> ZmddNode<V> {
        let zmdd = self.parent.upgrade().unwrap();
        let result = zmdd.borrow_mut().setdiff(self.node, other.node);
        self.rewrap(&zmdd, result)
    }

    /// Number of sparse vectors whose terminal label is in `ss`.
    pub fn count(&self, ss: &HashSet<V>) -> u64 {
        let mgr = self.parent.upgrade().unwrap();
        let dd = mgr.borrow();
        let mut cache = BddHashMap::default();
        zmdd_count(&dd, self.node, ss, &mut cache)
    }

    /// Enumerate the sparse vectors (as `{var: value}`, non-zero components only) whose
    /// terminal label is in `ss`.
    pub fn extract(&self, ss: &HashSet<V>) -> ZmddPath<V> {
        ZmddPath::new(self.clone(), ss)
    }
}

impl<V> Clone for ZmddNode<V>
where
    V: MddValue,
{
    fn clone(&self) -> Self {
        ZmddNode::from_weak(self.parent.clone(), self.gc.clone(), self.node)
    }
}

impl<V> Drop for ZmddNode<V> {
    fn drop(&mut self) {
        if let Some(g) = self.gc.upgrade() {
            let mut s = g.borrow_mut();
            if let Some(c) = s.roots.get_mut(&self.node) {
                *c -= 1;
                if *c == 0 {
                    s.roots.remove(&self.node);
                }
            }
        }
    }
}

fn zmdd_count<V>(
    dd: &ZmddManager<V>,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, u64>,
) -> u64
where
    V: MddValue,
{
    if let Some(&c) = cache.get(&node) {
        return c;
    }
    let r = match dd.get_node(&node).unwrap() {
        ZNode::Undet => 0,
        ZNode::Terminal(t) => {
            if ss.contains(&t.value()) {
                1
            } else {
                0
            }
        }
        ZNode::NonTerminal(f) => {
            let edges: Vec<NodeId> = f.iter().collect();
            edges.into_iter().map(|e| zmdd_count(dd, e, ss, cache)).sum()
        }
    };
    cache.insert(node, r);
    r
}

enum SV {
    Node(NodeId),
    Push(String, usize),
    Pop(String),
}

/// Enumerates the sparse vectors of a [`ZmddNode`] family (non-zero components only; the
/// 0-edge records nothing).
pub struct ZmddPath<V> {
    stack: Vec<SV>,
    path: HashMap<String, usize>,
    node: ZmddNode<V>,
    ss: HashSet<V>,
}

impl<V> ZmddPath<V>
where
    V: MddValue,
{
    fn new(node: ZmddNode<V>, ss: &HashSet<V>) -> Self {
        let mut stack = Vec::new();
        stack.push(SV::Node(node.get_id()));
        ZmddPath {
            stack,
            path: HashMap::new(),
            node,
            ss: ss.clone(),
        }
    }
}

impl<V> Iterator for ZmddPath<V>
where
    V: MddValue,
{
    type Item = HashMap<String, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let dd = self.node.get_mgr();
        while let Some(sv) = self.stack.pop() {
            match sv {
                SV::Node(id) => match dd.borrow().get_node(&id).unwrap() {
                    ZNode::Undet => (),
                    ZNode::Terminal(t) => {
                        if self.ss.contains(&t.value()) {
                            return Some(self.path.clone());
                        }
                    }
                    ZNode::NonTerminal(f) => {
                        let label = dd.borrow().label(&id).unwrap().to_string();
                        let edges: Vec<NodeId> = f.iter().collect();
                        for (i, e) in edges.into_iter().enumerate().rev() {
                            if i == 0 {
                                self.stack.push(SV::Node(e)); // 0-edge records nothing
                            } else {
                                self.stack.push(SV::Pop(label.clone()));
                                self.stack.push(SV::Node(e));
                                self.stack.push(SV::Push(label.clone(), i));
                            }
                        }
                    }
                },
                SV::Push(x, i) => {
                    self.path.insert(x, i);
                }
                SV::Pop(x) => {
                    self.path.remove(&x);
                }
            }
        }
        None
    }
}
