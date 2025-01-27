use crate::mdd_ops::MddOperation;
use crate::nodes::*;
use common::prelude::*;

#[derive(Debug)]
pub enum Node {
    NonTerminal(NonTerminalMDD),
    Zero,
    One,
    Undet,
}

impl Node {
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Zero => 0,
            Self::One => 1,
            Self::Undet => 2,
        }
    }

    pub fn headerid(&self) -> Option<HeaderId> {
        match self {
            Self::NonTerminal(x) => Some(x.headerid()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct MddManager {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node>,
    zero: NodeId,
    one: NodeId,
    undet: NodeId,
    utable: BddHashMap<(HeaderId, Box<[NodeId]>), NodeId>,
    cache: BddHashMap<(MddOperation, NodeId, NodeId), NodeId>,
}

impl DDForest for MddManager {
    type Node = Node;
    type NodeHeader = NodeHeader;

    #[inline]
    fn get_node(&self, id: &NodeId) -> Option<&Self::Node> {
        self.nodes.get(*id)
    }

    #[inline]
    fn get_header(&self, id: &HeaderId) -> Option<&Self::NodeHeader> {
        self.headers.get(*id)
    }

    fn level(&self, id: &NodeId) -> Option<Level> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(&fnode.headerid()).map(|x| x.level()),
            Node::Zero | Node::One | Node::Undet => None,
        })
    }

    fn label(&self, id: &NodeId) -> Option<&str> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(&fnode.headerid()).map(|x| x.label()),
            Node::Zero | Node::One | Node::Undet => None,
        })
    }
}

impl MddManager {
    pub fn new() -> Self {
        let headers = Vec::default();
        let mut nodes = Vec::default();
        let zero = {
            let tmp = Node::Zero;
            let id = tmp.id();
            nodes.push(tmp);
            debug_assert!(id == nodes[id].id());
            id
        };
        let one = {
            let tmp = Node::One;
            let id = tmp.id();
            nodes.push(tmp);
            debug_assert!(id == nodes[id].id());
            id
        };
        let undet = {
            let tmp = Node::Undet;
            let id = tmp.id();
            nodes.push(tmp);
            debug_assert!(id == nodes[id].id());
            id
        };
        let utable = BddHashMap::default();
        let cache = BddHashMap::default();
        Self {
            headers,
            nodes,
            zero,
            one,
            undet,
            utable,
            cache,
        }
    }

    fn new_nonterminal(&mut self, header: HeaderId, nodes: &[NodeId]) -> NodeId {
        let id = self.nodes.len();
        let tmp = Node::NonTerminal(NonTerminalMDD::new(id, header, nodes));
        self.nodes.push(tmp);
        debug_assert!(id == self.nodes[id].id());
        id
    }

    pub fn create_header(&mut self, level: Level, label: &str, edge_num: usize) -> HeaderId {
        let id = self.headers.len();
        let tmp = NodeHeader::new(id, level, label, edge_num);
        self.headers.push(tmp);
        debug_assert!(id == self.headers[id].id());
        id
    }

    pub fn create_node(&mut self, header: HeaderId, nodes: &[NodeId]) -> NodeId {
        if let Some(&first) = nodes.first() {
            if nodes.iter().all(|&x| first == x) {
                return first;
            }
        }
        let key = (header, nodes.to_vec().into_boxed_slice());
        if let Some(&nodeid) = self.utable.get(&key) {
            return nodeid;
        }
        let node = self.new_nonterminal(header, nodes);
        self.utable.insert(key, node);
        node
    }

    #[inline]
    pub fn size(&self) -> (usize, usize, usize) {
        (self.headers.len(), self.nodes.len(), self.cache.len())
    }

    #[inline]
    pub fn zero(&self) -> NodeId {
        self.zero
    }

    #[inline]
    pub fn one(&self) -> NodeId {
        self.one
    }

    #[inline]
    pub fn undet(&self) -> NodeId {
        self.undet
    }

    #[inline]
    pub fn get_cache(&self) -> &BddHashMap<(MddOperation, NodeId, NodeId), NodeId> {
        &self.cache
    }

    #[inline]
    pub fn get_mut_cache(&mut self) -> &mut BddHashMap<(MddOperation, NodeId, NodeId), NodeId> {
        &mut self.cache
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
