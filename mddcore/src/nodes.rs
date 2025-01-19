use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Rem, Sub};

use common::prelude::*;

#[derive(Debug)]
pub struct NonTerminalMDD {
    id: NodeId,
    header: HeaderId,
    nodes: Box<[NodeId]>,
}

impl NonTerminalMDD {
    pub fn new(id: NodeId, header: HeaderId, nodes: &[NodeId]) -> Self {
        Self {
            id,
            header,
            nodes: nodes.to_vec().into_boxed_slice(),
        }
    }
}

impl NonTerminal for NonTerminalMDD {
    #[inline]
    fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    fn headerid(&self) -> HeaderId {
        self.header
    }

    fn iter(&self) -> Iter<NodeId> {
        self.nodes.iter()
    }
}

impl Index<usize> for NonTerminalMDD {
    type Output = NodeId;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
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
