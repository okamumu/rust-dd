use common::prelude::*;

/// Non-terminal BDD/ZDD node.
///
/// Ids/edges are stored as `u32` (node and header counts fit in 32 bits) to
/// keep the node — and therefore the `nodes` arena — compact. The public API
/// stays in terms of `NodeId`/`HeaderId` (usize); casts are confined to the
/// accessors here. This type intentionally does NOT implement the shared
/// `common::NonTerminal` trait (whose `Index`/`iter` return references to
/// `NodeId`, which u32 storage cannot provide); it exposes value-returning
/// inherent accessors instead.
#[derive(Debug)]
pub struct NonTerminalBDD {
    id: u32,
    header: u32,
    edges: [u32; 2],
}

impl NonTerminalBDD {
    pub fn new(id: NodeId, header: HeaderId, edges: [NodeId; 2]) -> Self {
        Self {
            id: id as u32,
            header: header as u32,
            edges: [edges[0] as u32, edges[1] as u32],
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

    /// Returns the `i`-th edge (0 = low, 1 = high) as a `NodeId`.
    #[inline]
    pub fn edge(&self, i: usize) -> NodeId {
        self.edges[i] as NodeId
    }

    /// Iterates the edges as `NodeId` values (low then high).
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.edges.iter().map(|&e| e as NodeId)
    }
}

#[derive(Debug)]
pub enum Node {
    NonTerminal(NonTerminalBDD),
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
