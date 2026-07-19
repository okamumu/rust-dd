use common::prelude::*;
use crate::mdd;
use crate::mtmdd;
use crate::nodes::MddValue;
use crate::mtmdd2_ops::MtMdd2Operation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node {
    Value(NodeId),
    Bool(NodeId),
}

#[derive(Debug)]
pub struct MtMdd2Manager<V> {
    mdd: mdd::MddManager,
    mtmdd: mtmdd::MtMddManager<V>,
    // Direct-mapped, lossy computed tables (CUDD-style); see common::ComputeCache.
    bcache: ComputeCache,
    vcache: ComputeCache,
    // Ternary computed table for the native value-side `ite(f,g,h)`: f is a
    // boolean (mdd) node, g/h/result are value (mtmdd) nodes. Keyed (f,g,h);
    // flushed (not retained) by gc like the other two caches, so the
    // cross-forest key mix needs no per-arena liveness check.
    vite_cache: ComputeCache,
}

impl<V> MtMdd2Manager<V>
where
    V: MddValue
{
    pub fn new() -> Self {
        Self {
            mdd: mdd::MddManager::new(),
            mtmdd: mtmdd::MtMddManager::new(),
            bcache: ComputeCache::new(),
            vcache: ComputeCache::new(),
            vite_cache: ComputeCache::new(),
        }
    }

    #[inline]
    pub fn mtmdd(&self) -> &mtmdd::MtMddManager<V> {
        &self.mtmdd
    }

    #[inline]
    pub fn mtmdd_mut(&mut self) -> &mut mtmdd::MtMddManager<V> {
        &mut self.mtmdd
    }

    #[inline]
    pub fn mdd(&self) -> &mdd::MddManager {
        &self.mdd
    }

    #[inline]
    pub fn mdd_mut(&mut self) -> &mut mdd::MddManager {
        &mut self.mdd
    }

    #[inline]
    pub fn size(&self) -> (usize, usize, usize, usize) {
        let (vheader_size, vnode_size, vnode_size_val, vcache_size) = self.mtmdd.size();
        let (_bheader_size, bnode_size, bcache_size) = self.mdd.size();
        (
            vheader_size,
            vnode_size + bnode_size,
            vnode_size_val,
            vcache_size + bcache_size + self.vcache.len() + self.bcache.len() + self.vite_cache.len(),
        )
    }

    #[inline]
    pub fn one(&self) -> Node {
        Node::Bool(self.mdd.one())
    }

    #[inline]
    pub fn zero(&self) -> Node {
        Node::Bool(self.mdd.zero())
    }

    #[inline]
    pub fn value(&mut self, value: V) -> Node {
        Node::Value(self.mtmdd.value(value))
    }

    #[inline]
    pub fn undet_boolean(&self) -> Node {
        Node::Bool(self.mtmdd.undet())
    }

    #[inline]
    pub fn undet_value(&self) -> Node {
        Node::Value(self.mtmdd.undet())
    }

    #[inline]
    pub fn create_header(&mut self, level: Level, label: &str, edge_num: usize) -> HeaderId {
        let h1 = self.mtmdd.create_header(level, label, edge_num);
        let h2 = self.mdd.create_header(level, label, edge_num);
        assert_eq!(h1, h2);
        h1
    }

    pub fn create_node(&mut self, h: HeaderId, nodes: &[Node]) -> Node {
        let elem: Vec<NodeId> = nodes
            .iter()
            .map(|x| match x {
                Node::Value(f) => *f,
                Node::Bool(f) => *f,
            })
            .collect();
        match nodes[0] {
            Node::Value(_) => Node::Value(self.mtmdd.create_node(h, &elem)),
            Node::Bool(_) => Node::Bool(self.mdd.create_node(h, &elem)),
        }
    }

    #[inline]
    pub(crate) fn vcache_get(&self, key: &(MtMdd2Operation, NodeId, NodeId)) -> Option<NodeId> {
        self.vcache
            .get(key.0.code(), key.1 as u32, key.2 as u32)
            .map(|v| v as NodeId)
    }

    #[inline]
    pub(crate) fn vcache_put(&mut self, key: (MtMdd2Operation, NodeId, NodeId), val: NodeId) {
        self.vcache
            .put(key.0.code(), key.1 as u32, key.2 as u32, val as u32);
    }

    #[inline]
    pub(crate) fn bcache_get(&self, key: &(MtMdd2Operation, NodeId, NodeId)) -> Option<NodeId> {
        self.bcache
            .get(key.0.code(), key.1 as u32, key.2 as u32)
            .map(|v| v as NodeId)
    }

    #[inline]
    pub(crate) fn bcache_put(&mut self, key: (MtMdd2Operation, NodeId, NodeId), val: NodeId) {
        self.bcache
            .put(key.0.code(), key.1 as u32, key.2 as u32, val as u32);
    }

    /// Look up a memoized value-side `ite(f,g,h)` result (f is a bool node,
    /// g/h/result are value nodes).
    #[inline]
    pub(crate) fn vite_cache_get(&self, f: NodeId, g: NodeId, h: NodeId) -> Option<NodeId> {
        self.vite_cache
            .get(f as u32, g as u32, h as u32)
            .map(|v| v as NodeId)
    }

    /// Memoize a value-side `ite(f,g,h)` result.
    #[inline]
    pub(crate) fn vite_cache_put(&mut self, f: NodeId, g: NodeId, h: NodeId, val: NodeId) {
        self.vite_cache
            .put(f as u32, g as u32, h as u32, val as u32);
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.vcache.clear();
        self.bcache.clear();
        self.vite_cache.clear();
        self.mtmdd.clear_cache();
        self.mdd.clear_cache();
    }

    /// Mark-and-sweep garbage collection over the composite forest.
    ///
    /// The value (MTMDD) and boolean (MDD) sub-forests are independent — a
    /// `Value` node lives entirely in the MTMDD, a `Bool` node entirely in the
    /// MDD — so roots are partitioned by tag and each sub-manager is collected
    /// with its own roots. The two cross-manager caches are flushed as well.
    /// Returns the slots reclaimed as `(value_forest, bool_forest)`.
    pub fn gc(&mut self, roots: &[Node]) -> (usize, usize) {
        let mut vroots = Vec::new();
        let mut broots = Vec::new();
        for r in roots {
            match r {
                Node::Value(f) => vroots.push(*f),
                Node::Bool(f) => broots.push(*f),
            }
        }
        self.vcache.clear();
        self.bcache.clear();
        self.vite_cache.clear();
        let v = self.mtmdd.gc(&vroots);
        let b = self.mdd.gc(&broots);
        (v, b)
    }

    /// Total number of live (non-reclaimed) node slots across both sub-forests.
    #[inline]
    pub fn live_node_count(&self) -> usize {
        self.mtmdd.live_node_count() + self.mdd.live_node_count()
    }
}
