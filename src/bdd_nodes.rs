use std::rc::Rc;
use std::hash::{Hash, Hasher};
use core::slice::Iter;
use std::ops::Index;

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
    NodeHeader,
    TerminalBin,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    NOT,
    AND,
    OR,
    XOR,
}

#[derive(Debug)]
pub struct Terminal<T> {
    id: NodeId,
    value: T
}

impl<T> Terminal<T> where T: TerminalBin {
    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    pub fn value(&self) -> T {
        self.value
    }
}

#[derive(Debug)]
pub struct NonTerminal<T> {
    id: NodeId,
    header: NodeHeader,
    nodes: [Node<T>; 2],
}

impl<T> NonTerminal<T> {
    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    pub fn header(&self) -> &NodeHeader {
        &self.header
    }

    #[inline]
    pub fn iter(&self) -> Iter<Node<T>> {
        self.nodes.iter()
    }

    #[inline]
    pub fn level(&self) -> Level {
        self.header.level()
    }

    #[inline]
    pub fn label(&self) -> &str {
        self.header.label()
    }
}

impl<T> Index<usize> for NonTerminal<T> where T: TerminalBin {
    type Output = Node<T>;

    fn index(&self, i: usize) -> &Self::Output {
        &self.nodes[i]
    }
}

#[derive(Debug,Clone)]
pub enum Node<T> {
    NonTerminal(Rc<NonTerminal<T>>),
    Terminal(Rc<Terminal<T>>),
}

impl<T> PartialEq for Node<T> where T: TerminalBin {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> Eq for Node<T> where T: TerminalBin {}

impl<T> Hash for Node<T> where T: TerminalBin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<T> Node<T> where T: TerminalBin {
    pub fn new_nonterminal(id: NodeId, header: &NodeHeader, low: &Node<T>, high: &Node<T>) -> Self {
        let x = NonTerminal {
            id: id,
            header: header.clone(),
            nodes: [low.clone(), high.clone()],
        };
        Node::NonTerminal(Rc::new(x))
    }

    pub fn new_terminal(id: NodeId, value: T) -> Self {
        let x = Terminal {
            id: id,
            value: value,
        };
        Node::Terminal(Rc::new(x))
    }
    
    pub fn id(&self) -> NodeId {
        match self {
            Node::NonTerminal(x) => x.id(),
            Node::Terminal(x) => x.id(),
        }        
    }

    pub fn header(&self) -> Option<&NodeHeader> {
        match self {
            Node::NonTerminal(x) => Some(x.header()),
            _ => None
        }
    }

    pub fn level(&self) -> Option<Level> {
        self.header()
            .and_then(|x| Some(x.level()))
    }
}
