use std::ops::Index;
use std::slice::Iter;

use crate::common::{HeaderId, Level, NodeId};

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
    fn get_node(&self, id: &NodeId) -> Option<&Self::Node>;

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
    fn get_header(&self, id: &HeaderId) -> Option<&Self::NodeHeader>;

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
    fn label(&self, id: &NodeId) -> Option<&str>;

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
    fn level(&self, id: &NodeId) -> Option<Level>;
}

