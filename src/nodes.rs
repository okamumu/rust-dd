use std::ops::{Index, IndexMut};
use std::slice::{Iter, IterMut};

use crate::common::{EdgeValue, HeaderId, Level, NodeId, TerminalNumberValue};

/// Trait for a terminal node, which extends `Node`.
pub trait Terminal {
    /// The type of value stored in the terminal node.
    type Value;

    /// Returns the value stored in the terminal node.
    ///
    /// This value is specific to the implementation of the terminal node.
    fn value(&self) -> Self::Value;
}

/// The trait for non-terminal node.
pub trait NonTerminal: Index<usize> {
    /// Returns the unique identifier of the node.
    ///
    /// This ID is guaranteed to be unique across all nodes.
    fn id(&self) -> NodeId;

    /// Returns the header ID associated with the node.
    ///
    /// The header ID provides additional context or metadata.
    fn headerid(&self) -> HeaderId;

    /// Returns an iterator over the children of the non-terminal node.
    ///
    /// The iterator should return the children in the order they were added.
    fn iter(&self) -> Iter<NodeId>;
}

#[derive(Debug)]
pub struct NodeHeader {
    id: HeaderId,
    level: Level,
    label: String,
    edge_num: usize,
}

impl NodeHeader {
    pub fn new(id: HeaderId, level: Level, label: &str, edge_num: usize) -> Self {
        Self {
            id,
            level,
            label: label.to_string(),
            edge_num,
        }
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

/// The trait for a decision diagram forest.
pub trait DDForest {
    /// The type of terminal node.
    type Node;
    type NodeHeader;

    /// Returns the terminal node associated with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id`: The unique identifier of the terminal node.
    ///
    /// # Returns
    ///
    /// The terminal node associated with the given ID, or `None` if the ID is invalid.
    ///
    fn get_node(&self, id: NodeId) -> Option<&Self::Node>;

    /// Returns the header associated with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id`: The unique identifier of the header.
    ///
    /// # Returns
    ///
    /// The header associated with the given ID, or `None` if the ID is invalid.
    ///
    fn get_header(&self, id: HeaderId) -> Option<&Self::NodeHeader>;

    /// Returns the label associated with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id`: The unique identifier of the header.
    ///
    /// # Returns
    ///
    /// The label associated with the given ID, or `None` if the ID is invalid.
    ///
    fn label(&self, id: NodeId) -> Option<&str>;

    /// Returns the level associated with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id`: The unique identifier of the header.
    ///
    /// # Returns
    ///
    /// The level associated with the given ID, or `None` if the ID is invalid.
    ///
    fn level(&self, id: NodeId) -> Option<Level>;
}

// #[derive(Debug)]
// pub struct TerminalNumber<Value> {
//     id: NodeId,
//     value: Value,
// }

// impl<Value> TerminalNumber<Value> {
//     pub fn new(id: NodeId, value: Value) -> Self {
//         Self { id, value }
//     }
// }

// impl<Value> Terminal for TerminalNumber<Value>
// where
//     Value: TerminalNumberValue,
// {
//     type Value = Value;

//     #[inline]
//     fn value(&self) -> Self::Value {
//         self.value
//     }
// }

// #[derive(Debug)]
// pub struct NonTerminalBDD<N> {
//     id: NodeId,
//     header: NodeHeader,
//     nodes: [N; 2],
// }

// impl<N> NonTerminalBDD<N> {
//     pub fn new(id: NodeId, header: NodeHeader, nodes: [N; 2]) -> Self {
//         Self {
//             id,
//             header,
//             nodes,
//         }
//     }
// }

// impl<N> NonTerminal for NonTerminalBDD<N> {
//     type Node = N;

//     #[inline]
//     fn id(&self) -> NodeId {
//         self.id
//     }

//     #[inline]
//     fn set_id(&mut self, id: NodeId) {
//         self.id = id;
//     }

//     #[inline]
//     fn header(&self) -> &NodeHeader {
//         &self.header
//     }

//     #[inline]
//     fn level(&self) -> Level {
//         self.header.level()
//     }

//     #[inline]
//     fn label(&self) -> &str {
//         self.header.label()
//     }

//     #[inline]
//     fn iter(&self) -> Iter<Self::Node> {
//         self.nodes.iter()
//     }

//     #[inline]
//     fn iter_mut(&mut self) -> IterMut<Self::Node> {
//         self.nodes.iter_mut()
//     }
// }

// impl<N> Index<usize> for NonTerminalBDD<N> {
//     type Output = N;

//     #[inline]
//     fn index(&self, index: usize) -> &Self::Output {
//         &self.nodes[index]
//     }
// }

// impl<N> IndexMut<usize> for NonTerminalBDD<N> {
//     #[inline]
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         &mut self.nodes[index]
//     }
// }

// #[derive(Debug)]
// pub struct NonTerminalMDD<N> {
//     id: NodeId,
//     header: NodeHeader,
//     nodes: Box<[N]>,
// }

// impl<N> NonTerminalMDD<N> {
//     pub fn new(id: NodeId, header: NodeHeader, nodes: Box<[N]>) -> Self {
//         Self {
//             id,
//             header,
//             nodes,
//         }
//     }
// }

// impl<N> NonTerminal for NonTerminalMDD<N> {
//     type Node = N;

//     #[inline]
//     fn id(&self) -> NodeId {
//         self.id
//     }

//     #[inline]
//     fn set_id(&mut self, id: NodeId) {
//         self.id = id;
//     }

//     #[inline]
//     fn header(&self) -> &NodeHeader {
//         &self.header
//     }

//     #[inline]
//     fn level(&self) -> Level {
//         self.header.level()
//     }

//     #[inline]
//     fn label(&self) -> &str {
//         self.header.label()
//     }

//     #[inline]
//     fn iter(&self) -> Iter<Self::Node> {
//         self.nodes.iter()
//     }

//     #[inline]
//     fn iter_mut(&mut self) -> IterMut<Self::Node> {
//         self.nodes.iter_mut()
//     }
// }

// impl<N> Index<usize> for NonTerminalMDD<N> {
//     type Output = N;

//     #[inline]
//     fn index(&self, index: usize) -> &Self::Output {
//         &self.nodes[index]
//     }
// }

// impl<N> IndexMut<usize> for NonTerminalMDD<N> {
//     #[inline]
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         &mut self.nodes[index]
//     }
// }

// #[derive(Debug,Clone,PartialEq,Eq,Hash)]
// pub struct EvEdge<V,N> {
//     value: V,
//     node: N,
// }

// impl<V,N> EvEdge<V,N> where V: EdgeValue {
//     pub fn new(value: V, node: N) -> Self {
//         Self {
//             value,
//             node,
//         }
//     }

//     #[inline]
//     pub fn value(&self) -> V {
//         self.value
//     }

//     #[inline]
//     pub fn node(&self) -> &N {
//         &self.node
//     }
// }

// #[macro_export]
// macro_rules! nodes {
//     ($($elem:expr),*) => {
//         vec![$($elem.clone()),*]
//     };
// }
