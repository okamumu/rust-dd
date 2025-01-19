use common::prelude::*;
use crate::mdd;
use crate::mtmdd;
use crate::nodes::MddValue;
use crate::mtmdd2_ops::MtMdd2Operation;

#[derive(Debug, Clone, Copy)]
pub enum Node {
    Value(NodeId),
    Bool(NodeId),
}

#[derive(Debug)]
pub struct MtMdd2Manager<V> {
    mdd: mdd::MddManager,
    mtmdd: mtmdd::MtMddManager<V>,
    bcache: BddHashMap<(MtMdd2Operation, NodeId, NodeId), NodeId>,
    vcache: BddHashMap<(MtMdd2Operation, NodeId, NodeId), NodeId>,
}

impl<V> MtMdd2Manager<V>
where
    V: MddValue
{
    pub fn new() -> Self {
        Self {
            mdd: mdd::MddManager::new(),
            mtmdd: mtmdd::MtMddManager::new(),
            bcache: BddHashMap::default(),
            vcache: BddHashMap::default(),
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
        let (u1, x1, y1, z1) = self.mtmdd.size();
        let (x2, y2, z2) = self.mdd.size();
        (u1, x1, y1, z1 + x2 + y2 + z2)
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
    pub fn get_vcache(&self) -> &BddHashMap<(MtMdd2Operation, NodeId, NodeId), NodeId> {
        &self.vcache
    }

    #[inline]
    pub fn get_bcache(&self) -> &BddHashMap<(MtMdd2Operation, NodeId, NodeId), NodeId> {
        &self.bcache
    }

    #[inline]
    pub fn get_mut_vcache(&mut self) -> &mut BddHashMap<(MtMdd2Operation, NodeId, NodeId), NodeId> {
        &mut self.vcache
    }

    #[inline]
    pub fn get_mut_bcache(&mut self) -> &mut BddHashMap<(MtMdd2Operation, NodeId, NodeId), NodeId> {
        &mut self.bcache
    }
}
