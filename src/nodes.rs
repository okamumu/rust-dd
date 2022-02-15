use std::rc::Rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};
use std::slice::{Iter, IterMut};
use std::ops::{Index, IndexMut};

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    TerminalNumberValue,
    EdgeValue,
};

/// Trait for terminal node.
pub trait Terminal {
    /// type for value
    type Value;
    /// A method to get nodeid
    fn id(&self) -> NodeId;
    /// A method to get a value stored in terminal node
    fn value(&self) -> Self::Value;
}

/// The trait for non-terminal node.
pub trait NonTerminal : Index<usize> + IndexMut<usize> {
        type Node;
    fn id(&self) -> NodeId;
    fn header(&self) -> &NodeHeader;
    fn level(&self) -> Level;
    fn label(&self) -> &str;
    fn iter(&self) -> Iter<Self::Node>;
    fn iter_mut(&mut self) -> IterMut<Self::Node>;
}

#[derive(Debug)]
pub struct NodeHeaderData {
    id: HeaderId,
    level: Level,
    label: String,
    edge_num: usize,
}

#[derive(Debug,Clone)]
pub struct NodeHeader(Rc<NodeHeaderData>);

impl Deref for NodeHeader {
    type Target = Rc<NodeHeaderData>;
    
    fn deref(&self) -> &Rc<NodeHeaderData> {
        &self.0
    }
}

impl PartialEq for NodeHeader {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for NodeHeader {}

impl Hash for NodeHeader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl NodeHeader {
    pub fn new(id: HeaderId, level: Level, label: &str, edge_num: usize) -> Self {
        let data = NodeHeaderData {
            id: id,
            level: level,
            label: label.to_string(),
            edge_num: edge_num,
        };
        Self(Rc::new(data))
    }

    #[inline]
    pub fn id(&self) -> HeaderId {
        self.id
    }

    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    #[inline]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[inline]
    pub fn edge_num(&self) -> usize {
        self.edge_num
    }
}

#[derive(Debug)]
pub struct TerminalNumber<V> {
    id: NodeId,
    value: V
}

impl<V> TerminalNumber<V> {
    #[inline]
    pub fn new(id: NodeId, value: V) -> Self {
        Self {
            id: id,
            value: value,
        }
    }
}

impl<V> Terminal for TerminalNumber<V> where V: TerminalNumberValue {
    type Value = V;

    #[inline]
    fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    fn value(&self) -> Self::Value {
        self.value
    }
}

#[derive(Debug)]
pub struct NonTerminalBDD<N> {
    id: NodeId,
    header: NodeHeader,
    nodes: [N; 2],
}

impl<N> NonTerminalBDD<N> {
    #[inline]
    pub fn new(id: NodeId, header: NodeHeader, nodes: [N; 2]) -> Self {
        Self {
            id: id,
            header: header,
            nodes: nodes,
        }
    }
}

impl<N> NonTerminal for NonTerminalBDD<N> {
    type Node = N;

    #[inline]
    fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    fn header(&self) -> &NodeHeader {
        &self.header
    }

    #[inline]
    fn level(&self) -> Level {
        self.header.level()
    }

    #[inline]
    fn label(&self) -> &str {
        self.header.label()
    }

    #[inline]
    fn iter(&self) -> Iter<Self::Node> {
        self.nodes.iter()
    }

    #[inline]
    fn iter_mut(&mut self) -> IterMut<Self::Node> {
        self.nodes.iter_mut()
    }
}

impl<N> Index<usize> for NonTerminalBDD<N> {
    type Output = N;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<N> IndexMut<usize> for NonTerminalBDD<N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

#[derive(Debug)]
pub struct NonTerminalMDD<N> {
    id: NodeId,
    header: NodeHeader,
    nodes: Box<[N]>,
}

impl<N> NonTerminalMDD<N> {
    #[inline]
    pub fn new(id: NodeId, header: NodeHeader, nodes: Box<[N]>) -> Self {
        Self {
            id: id,
            header: header,
            nodes: nodes,
        }
    }
}

impl<N> NonTerminal for NonTerminalMDD<N> {
    type Node = N;

    #[inline]
    fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    fn header(&self) -> &NodeHeader {
        &self.header
    }

    #[inline]
    fn level(&self) -> Level {
        self.header.level()
    }

    #[inline]
    fn label(&self) -> &str {
        self.header.label()
    }

    #[inline]
    fn iter(&self) -> Iter<Self::Node> {
        self.nodes.iter()
    }

    #[inline]
    fn iter_mut(&mut self) -> IterMut<Self::Node> {
        self.nodes.iter_mut()
    }
}

impl<N> Index<usize> for NonTerminalMDD<N> {
    type Output = N;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<N> IndexMut<usize> for NonTerminalMDD<N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct EvEdge<V,N> {
    value: V,
    node: N,
}

impl<V,N> EvEdge<V,N> where V: EdgeValue {
    #[inline]
    pub fn new(value: V, node: N) -> Self {
        Self {
            value: value,
            node: node,
        }
    }

    #[inline]
    pub fn value(&self) -> V {
        self.value
    }

    #[inline]
    pub fn node(&self) -> &N {
        &self.node
    }
}

