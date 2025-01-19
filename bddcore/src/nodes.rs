use common::prelude::*;

#[derive(Debug)]
pub struct NonTerminalBDD {
    id: NodeId,
    header: HeaderId,
    edges: [NodeId; 2],
}

impl NonTerminalBDD {
    pub fn new(id: NodeId, header: HeaderId, edges: [NodeId; 2]) -> Self {
        Self { id, header, edges }
    }
}

impl NonTerminal for NonTerminalBDD {
    #[inline]
    fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    fn headerid(&self) -> HeaderId {
        self.header
    }

    #[inline]
    fn iter(&self) -> Iter<NodeId> {
        self.edges.iter()
    }
}

impl Index<usize> for NonTerminalBDD {
    type Output = NodeId;

    fn index(&self, index: usize) -> &Self::Output {
        &self.edges[index]
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
