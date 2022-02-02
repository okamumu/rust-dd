use num_traits::{NumOps, Zero, One};
use std::fmt::Display;
use std::rc::Rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};

pub type HeaderId = usize;
pub type NodeId = usize;
pub type Level = usize;

pub type HashMap<T,U> = std::collections::HashMap<T,U>;
pub type HashSet<T> = std::collections::HashSet<T>;
// pub type HashMap<T,U> = hashbrown::HashMap<T,U>;
// pub type HashSet<T> = hashbrown::HashSet<T>;

pub trait TerminalValue:
    Copy
    + Clone
    + PartialEq
    + Eq
    + Hash
    + NumOps
    + Display
    + Ord
    + Zero
    + One
    {}

impl TerminalValue for u32 {}
impl TerminalValue for u64 {}
impl TerminalValue for i32 {}
impl TerminalValue for i64 {}

pub trait EdgeValue:
    Copy
    + Clone
    + PartialEq
    + Eq
    + Hash
    + NumOps
    + Display
    + Ord
    + Zero
    {}

impl EdgeValue for i32 {}
impl EdgeValue for i64 {}

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

    pub fn id(&self) -> HeaderId {
        self.id
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn edge_num(&self) -> usize {
        self.edge_num
    }
}
