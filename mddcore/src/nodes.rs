use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Rem, Sub};

use common::prelude::*;

/// Non-terminal MDD node.
///
/// Ids/header/children are stored as `u32` (node and header counts fit in 32
/// bits) to keep the node — and therefore the `nodes` arena and the unique-table
/// keys, which copy the child slice — compact; this halves the id bytes stored
/// and hashed per `create_node` on large diagrams. The public API stays in terms
/// of `NodeId`/`HeaderId` (usize); casts are confined to the accessors here. Like
/// `bddcore::NonTerminalBDD`, this type intentionally does NOT implement the
/// shared `common::NonTerminal` trait (whose `Index`/`iter` return references to
/// `NodeId`, which u32 storage cannot provide); it exposes value-returning
/// inherent accessors instead.
#[derive(Debug)]
pub struct NonTerminalMDD {
    id: u32,
    header: u32,
    nodes: Box<[u32]>,
}

impl NonTerminalMDD {
    pub fn new(id: NodeId, header: HeaderId, nodes: &[NodeId]) -> Self {
        Self {
            id: id as u32,
            header: header as u32,
            nodes: nodes.iter().map(|&x| x as u32).collect(),
        }
    }

    #[inline]
    pub fn id(&self) -> NodeId {
        self.id as NodeId
    }

    #[inline]
    pub fn headerid(&self) -> HeaderId {
        self.header as HeaderId
    }

    /// Returns the `i`-th child as a `NodeId`.
    #[inline]
    pub fn edge(&self, i: usize) -> NodeId {
        self.nodes[i] as NodeId
    }

    /// Number of children (edge count).
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterates the children as `NodeId` values.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().map(|&x| x as NodeId)
    }
}

pub trait MddValue:
    Copy
    + Clone
    + PartialEq
    + Eq
    + Hash
    + Display
    + Ord
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + From<i32>
{
}

impl MddValue for i32 {}
impl MddValue for i64 {}
