/// ZDD (Zero-suppressed Binary Decision Diagram)
///
/// Description:
///
/// A ZDD is a rooted directed acyclic graph (DAG) with two terminal nodes, 0 and 1.
/// Each non-terminal node has a level and two edges, low and high.
/// The level is an integer that represents the variable of the node.
/// The low and high edges are the child nodes of the node.
///
/// The ZDD has a unique table that stores the non-terminal nodes.
/// The table is a hash table that maps a tuple of (level, low, high) to a non-terminal node.
///
/// The ZDD has a cache that stores the result of the operations.
/// The cache is a hash table that maps a tuple of (operation, f, g) to a node.
///
/// The ZDD has the following methods:
/// - create_header(level, label): create a new header
/// - create_node(header, low, high): create a new non-terminal node
/// - zero(): return the terminal node 0
/// - one(): return the terminal node 1
/// - size(): return the number of headers, nodes, and the size of the unique table
///

use std::hash::Hash;
use std::ops::Index;
use std::slice::Iter;

use crate::common::HashMap;
use crate::common::HashSet;
use crate::common::HeaderId;
use crate::common::Level;
use crate::common::NodeId;
use crate::common::OperationId;
use crate::dot::Dot;

use crate::nodes::DDForest;
use crate::nodes::NodeHeader;
use crate::nodes::NonTerminal;

#[derive(Debug)]
pub struct NonTerminalBDD {
    id: NodeId,
    header: HeaderId,
    edges: [NodeId; 2],
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
pub struct ZddManager {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node>,
    zero: NodeId,
    one: NodeId,
    undet: NodeId,
    utable: HashMap<(HeaderId, NodeId, NodeId), NodeId>,
    cache: HashMap<(OperationId, NodeId, NodeId), NodeId>,
    next_stack: Vec<Operation>,
    result_stack: Vec<NodeId>,
}

impl DDForest for ZddManager {
    type Node = Node;
    type NodeHeader = NodeHeader;

    #[inline]
    fn get_node(&self, id: NodeId) -> Option<&Self::Node> {
        self.nodes.get(id)
    }

    #[inline]
    fn get_header(&self, id: HeaderId) -> Option<&NodeHeader> {
        self.headers.get(id)
    }

    fn level(&self, id: NodeId) -> Option<Level> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(fnode.header).map(|x| x.level()),
            Node::Zero | Node::One | Node::Undet => None,
        })
    }

    fn label(&self, id: NodeId) -> Option<&str> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(fnode.header).map(|x| x.label()),
            Node::Zero | Node::One | Node::Undet => None,
        })
    }
}

impl ZddManager {
    pub fn new() -> Self {
        let headers = Vec::default();
        let mut nodes = Vec::default();
        nodes.push(Node::Zero);
        nodes.push(Node::One);
        nodes.push(Node::Undet);
        let zero = nodes[0].id();
        let one = nodes[1].id();
        let undet = nodes[2].id();
        let utable = HashMap::default();
        let cache = HashMap::default();
        let next_stack = Vec::default();
        let result_stack = Vec::default();
        Self {
            headers,
            nodes,
            zero,
            one,
            undet,
            utable,
            cache,
            next_stack,
            result_stack,
        }
    }

    fn new_nonterminal(&mut self, headerid: HeaderId, low: NodeId, high: NodeId) -> NodeId {
        let id = self.nodes.len();
        let node = Node::NonTerminal(NonTerminalBDD {
            id,
            header: headerid,
            edges: [low, high],
        });
        self.nodes.push(node);
        id
    }

    pub fn create_header(&mut self, level: Level, label: &str) -> HeaderId {
        let headerid = self.headers.len();
        let header = NodeHeader::new(headerid, level, label, 2);
        self.headers.push(header);
        headerid
    }

    pub fn create_node(&mut self, header: HeaderId, low: NodeId, high: NodeId) -> NodeId {
        if high == self.zero {
            return low;
        }
        let key = (header, low, high);
        if let Some(nodeid) = self.utable.get(&key) {
            return *nodeid;
        }
        let node = self.new_nonterminal(header, low, high);
        self.utable.insert(key, node);
        node
    }

    pub fn size(&self) -> (HeaderId, NodeId, usize) {
        (self.headers.len(), self.nodes.len(), self.utable.len())
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
}

#[derive(Debug)]
enum Operation {
    Intersect(NodeId, NodeId),
    Union(NodeId, NodeId),
    Setdiff(NodeId, NodeId),
    Product(NodeId, NodeId),
    CreateNode((OperationId, NodeId, NodeId), HeaderId),
    Result(NodeId),
    UnionStack,
    Division(NodeId, NodeId),
    IntersectStack,
}

impl Operation {
    fn id(&self) -> OperationId {
        match self {
            Self::Intersect(_, _) => 0,
            Self::Union(_, _) => 1,
            Self::Setdiff(_, _) => 2,
            Self::Product(_, _) => 3,
            Self::CreateNode(_, _) => 4,
            Self::Result(_) => 5,
            Self::UnionStack => 6,
            Self::Division(_, _) => 7,
            Self::IntersectStack => 8,
        }
    }
}

macro_rules! stack_operations {
    ($self:ident, $($op:expr),*) => {{
        let ops = [$($op),*];
        for op in ops.into_iter().rev() {
            $self.next_stack.push(op);
        }
    }};
}

type OpKey = (OperationId, NodeId, NodeId);

impl ZddManager {
    fn apply(&mut self, op: Operation) -> NodeId {
        self.next_stack.clear();
        self.result_stack.clear();
        self.next_stack.push(op);
        while let Some(op) = self.next_stack.pop() {
            match op {
                Operation::Intersect(f, g) => {
                    let key = (op.id(), f, g);
                    if let Some(id) = self.cache.get(&key) {
                        self.result_stack.push(*id);
                        continue;
                    }
                    self.apply_intersect(key, f, g);
                }
                Operation::Union(f, g) => {
                    let key = (op.id(), f, g);
                    if let Some(id) = self.cache.get(&key) {
                        self.result_stack.push(*id);
                        continue;
                    }
                    self.apply_union(key, f, g);
                }
                Operation::Setdiff(f, g) => {
                    let key = (op.id(), f, g);
                    if let Some(id) = self.cache.get(&key) {
                        self.result_stack.push(*id);
                        continue;
                    }
                    self.apply_setdiff(key, f, g);
                }
                Operation::Product(f, g) => {
                    let key = (op.id(), f, g);
                    if let Some(id) = self.cache.get(&key) {
                        self.result_stack.push(*id);
                        continue;
                    }
                    self.apply_product(key, f, g);
                }
                Operation::Division(f, g) => {
                    let key = (op.id(), f, g);
                    if let Some(id) = self.cache.get(&key) {
                        self.result_stack.push(*id);
                        continue;
                    }
                    self.apply_division(key, f, g);
                }
                Operation::CreateNode(key, h) => {
                    let high = self.result_stack.pop().unwrap();
                    let low = self.result_stack.pop().unwrap();
                    let result = self.create_node(h, low, high);
                    self.cache.insert(key, result);
                    self.result_stack.push(result);
                }
                Operation::Result(id) => {
                    self.result_stack.push(id);
                }
                Operation::UnionStack => {
                    let low = self.result_stack.pop().unwrap();
                    let high = self.result_stack.pop().unwrap();
                    let op = Operation::Union(low, high);
                    self.next_stack.push(op);
                }
            }
        }
        debug_assert!(self.result_stack.len() == 1);
        self.result_stack.pop().unwrap()
    }

    fn apply_intersect(&mut self, key: OpKey, f: NodeId, g: NodeId) {
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => {
                stack_operations!(
                    self,
                    Operation::Result(g)
                );
            }
            (_, Node::Undet) => {
                stack_operations!(
                    self,
                    Operation::Result(f)
                );
            }
            (Node::Zero, _) => {
                stack_operations!(self, Operation::Result(self.zero()));
            }
            (_, Node::Zero) => {
                stack_operations!(self, Operation::Result(self.zero()));
            }
            (Node::One, _) => {
                stack_operations!(self, Operation::Result(g));
            }
            (_, Node::One) => {
                stack_operations!(self, Operation::Result(f));
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if fnode.id() == gnode.id() =>
            {
                stack_operations!(self, Operation::Result(f));
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Intersect(fnode[0], g)
                );
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Intersect(f, gnode[0])
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                stack_operations!(
                    self,
                    Operation::Intersect(fnode[0], gnode[0]),
                    Operation::Intersect(fnode[1], gnode[1]),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
        }
    }

    fn apply_union(&mut self, key: OpKey, f: NodeId, g: NodeId) {
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => {
                stack_operations!(
                    self,
                    Operation::Result(g)
                );
            }
            (_, Node::Undet) => {
                stack_operations!(
                    self,
                    Operation::Result(f)
                );
            }
            (Node::Zero, _) => {
                stack_operations!(self, Operation::Result(g));
            }
            (_, Node::Zero) => {
                stack_operations!(self, Operation::Result(f));
            }
            (Node::One, Node::One) => {
                stack_operations!(self, Operation::Result(self.one()));
            }
            (Node::NonTerminal(fnode), Node::One) => {
                stack_operations!(
                    self,
                    Operation::Union(fnode[0], self.one()),
                    Operation::Result(fnode[1]),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
            (Node::One, Node::NonTerminal(gnode)) => {
                stack_operations!(
                    self,
                    Operation::Union(self.one(), gnode[0]),
                    Operation::Result(gnode[1]),
                    Operation::CreateNode(key, gnode.headerid())
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if fnode.id() == gnode.id() =>
            {
                stack_operations!(self, Operation::Result(f));
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Union(fnode[0], g),
                    Operation::Result(fnode[1]),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Union(f, gnode[0]),
                    Operation::Result(gnode[1]),
                    Operation::CreateNode(key, gnode.headerid())
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                stack_operations!(
                    self,
                    Operation::Union(fnode[0], gnode[0]),
                    Operation::Union(fnode[1], gnode[1]),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
        }
    }

    fn apply_setdiff(&mut self, key: OpKey, f: NodeId, g: NodeId) {
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => {
                stack_operations!(
                    self,
                    Operation::Result(g)
                );
            }
            (_, Node::Undet) => {
                stack_operations!(
                    self,
                    Operation::Result(f)
                );
            }
            (_, Node::Zero) => {
                stack_operations!(
                    self,
                    Operation::Result(f)
                );
            }
            (_, Node::One) => {
                stack_operations!(
                    self,
                    Operation::Result(self.zero())
                );
            }
            (Node::Zero, _) => {
                stack_operations!(
                    self,
                    Operation::Result(self.zero())
                );
            }
            (Node::One, Node::NonTerminal(gnode)) => {
                stack_operations!(
                    self,
                    Operation::Setdiff(self.one(), gnode[0])
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if fnode.id() == gnode.id() =>
            {
                stack_operations!(
                    self,
                    Operation::Result(self.zero())
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Setdiff(fnode[0], g),
                    Operation::Result(fnode[1]),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Setdiff(f, gnode[0])
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                stack_operations!(
                    self,
                    Operation::Setdiff(fnode[0], gnode[0]),
                    Operation::Setdiff(fnode[1], gnode[1]),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
        }
    }

    fn apply_product(&mut self, key: OpKey, f: NodeId, g: NodeId) {
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => {
                stack_operations!(
                    self,
                    Operation::Result(g)
                );
            }
            (_, Node::Undet) => {
                stack_operations!(
                    self,
                    Operation::Result(f)
                );
            }
            (_, Node::Zero) => {
                stack_operations!(
                    self,
                    Operation::Result(self.zero())
                );
            }
            (Node::Zero, _) => {
                stack_operations!(
                    self,
                    Operation::Result(self.zero())
                );
            }
            (_, Node::One) => {
                stack_operations!(
                    self,
                    Operation::Result(f)
                );
            }
            (Node::One, _) => {
                stack_operations!(
                    self,
                    Operation::Result(g)
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Product(fnode[0], g),
                    Operation::Product(fnode[1], g),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Product(f, gnode[0]),
                    Operation::Product(f, gnode[1]),
                    Operation::CreateNode(key, gnode.headerid())
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                stack_operations!(
                    self,
                    Operation::Product(fnode[0], gnode[0]),
                    Operation::Product(fnode[0], gnode[1]),
                    Operation::Product(fnode[1], gnode[0]),
                    Operation::Product(fnode[1], gnode[1]),
                    Operation::UnionStack,
                    Operation::UnionStack,
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
        }
    }

    fn apply_division(&mut self, key: OpKey, f: NodeId, g: NodeId) {
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => {
                stack_operations!(
                    self,
                    Operation::Result(g)
                );
            }
            (_, Node::Undet) => {
                stack_operations!(
                    self,
                    Operation::Result(f)
                );
            }
            (_, Node::Zero) => {
                stack_operations!(
                    self,
                    Operation::Result(self.undet())
                );
            }
            (Node::Zero, _) => {
                stack_operations!(
                    self,
                    Operation::Result(self.zero())
                );
            }
            (Node::One, _) => {
                stack_operations!(
                    self,
                    Operation::Result(self.zero())
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Division(fnode[0], g),
                    Operation::Division(fnode[1], g),
                    Operation::CreateNode(key, fnode.headerid())
                );
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                stack_operations!(
                    self,
                    Operation::Division(f, gnode[0])
                );
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                stack_operations!(
                    self,
                    Operation::Division(fnode[0], gnode[0]),
                    Operation::Division(fnode[1], gnode[1]),
                    Operation::IntersectStack
                );
            }
        }
    }
}

impl ZddManager {
    // pub fn not(&mut self, f: NodeId) -> NodeId {
    //     #[derive(Debug)]
    //     enum StackValue {
    //         Node(NodeId),
    //         Header((Operation, NodeId, NodeId), HeaderId),
    //     }
    //     let mut result_stack = Vec::new();
    //     let mut next_stack = Vec::new();
    //     next_stack.push(StackValue::Node(f));
    //     while let Some(token) = next_stack.pop() {
    //         match token {
    //             StackValue::Node(f) => {
    //                 let key = (Operation::Not, f, 0);
    //                 if let Some(id) = self.cache.get(&key) {
    //                     result_stack.push(*id);
    //                     continue;
    //                 }
    //                 match self.get_node(f) {
    //                     Node::Zero => {
    //                         result_stack.push(self.one());
    //                     }
    //                     Node::One => {
    //                         result_stack.push(self.zero());
    //                     }
    //                     Node::NonTerminal(fnode) => {
    //                         next_stack.push(StackValue::Header(key, fnode.header()));
    //                         next_stack.push(StackValue::Node(fnode[1]));
    //                         next_stack.push(StackValue::Node(fnode[0]));
    //                     }
    //                 }
    //             }
    //             StackValue::Header(key, h) => {
    //                 let high = result_stack.pop().unwrap();
    //                 let low = result_stack.pop().unwrap();
    //                 let result = self.create_node(h, low, high);
    //                 self.cache.insert(key, result);
    //                 result_stack.push(result);
    //             }
    //         }
    //     }
    //     debug_assert!(result_stack.len() == 1);
    //     if let Some(resultid) = result_stack.pop() {
    //         resultid
    //     } else {
    //         panic!("Error: not")
    //     }
    // }

    // pub fn intersect(&mut self, f: NodeId, g: NodeId) -> NodeId {
    //     #[derive(Debug)]
    //     enum StackValue {
    //         Intersect(NodeId, NodeId),
    //         CreateNode((Operation, NodeId, NodeId), HeaderId),
    //     }
    //     let mut result_stack = Vec::new();
    //     let mut next_stack = Vec::new();
    //     next_stack.push(StackValue::Intersect(f, g));
    //     while let Some(token) = next_stack.pop() {
    //         match token {
    //             StackValue::Intersect(f, g) => {
    //                 let key = (Operation::Intersect, f, g);
    //                 if let Some(id) = self.cache.get(&key) {
    //                     result_stack.push(*id);
    //                     continue;
    //                 }
    //                 match (self.get_node(f), self.get_node(g)) {
    //                     (Node::Zero, _) => {
    //                         result_stack.push(self.zero());
    //                     }
    //                     (_, Node::Zero) => {
    //                         result_stack.push(self.zero());
    //                     }
    //                     (Node::One, _) => {
    //                         result_stack.push(g);
    //                     }
    //                     (_, Node::One) => {
    //                         result_stack.push(f);
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
    //                         if fnode.id() == gnode.id() =>
    //                     {
    //                         result_stack.push(f);
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
    //                         if self.level(f) > self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::Intersect(fnode[0], g));
    //                     }
    //                     (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
    //                         if self.level(f) < self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::Intersect(f, gnode[0]));
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Intersect(fnode[1], gnode[1]));
    //                         next_stack.push(StackValue::Intersect(fnode[0], gnode[0]));
    //                     }
    //                 }
    //             }
    //             StackValue::CreateNode(key, h) => {
    //                 let high = result_stack.pop().unwrap();
    //                 let low = result_stack.pop().unwrap();
    //                 let result = self.create_node(h, low, high);
    //                 self.cache.insert(key, result);
    //                 result_stack.push(result);
    //             }
    //         }
    //     }
    //     debug_assert!(result_stack.len() == 1);
    //     if let Some(resultid) = result_stack.pop() {
    //         resultid
    //     } else {
    //         panic!("Error: intersect")
    //     }
    // }

    // pub fn union(&mut self, f: NodeId, g: NodeId) -> NodeId {
    //     #[derive(Debug)]
    //     enum StackValue {
    //         Node(NodeId),
    //         Union(NodeId, NodeId),
    //         CreateNode((Operation, NodeId, NodeId), HeaderId),
    //     }
    //     let mut result_stack = Vec::new();
    //     let mut next_stack = Vec::new();
    //     next_stack.push(StackValue::Union(f, g));
    //     while let Some(token) = next_stack.pop() {
    //         match token {
    //             StackValue::Union(f, g) => {
    //                 let key = (Operation::Union, f, g);
    //                 if let Some(id) = self.cache.get(&key) {
    //                     result_stack.push(*id);
    //                     continue;
    //                 }
    //                 match (self.get_node(f), self.get_node(g)) {
    //                     (Node::Zero, _) => {
    //                         result_stack.push(g);
    //                     }
    //                     (_, Node::Zero) => {
    //                         result_stack.push(f);
    //                     }
    //                     (Node::One, Node::One) => {
    //                         result_stack.push(self.one());
    //                     }
    //                     (Node::NonTerminal(fnode), Node::One) => {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Node(fnode[1]));
    //                         next_stack.push(StackValue::Union(fnode[0], self.one()));
    //                     }
    //                     (Node::One, Node::NonTerminal(gnode)) => {
    //                         next_stack.push(StackValue::CreateNode(key, gnode.header()));
    //                         next_stack.push(StackValue::Node(gnode[1]));
    //                         next_stack.push(StackValue::Union(self.one(), gnode[0]));
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
    //                         if fnode.id() == gnode.id() =>
    //                     {
    //                         result_stack.push(f);
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
    //                         if self.level(f) > self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Node(fnode[1]));
    //                         next_stack.push(StackValue::Union(fnode[0], g));
    //                     }
    //                     (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
    //                         if self.level(f) < self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::CreateNode(key, gnode.header()));
    //                         next_stack.push(StackValue::Node(gnode[1]));
    //                         next_stack.push(StackValue::Union(f, gnode[0]));
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Union(fnode[1], gnode[1]));
    //                         next_stack.push(StackValue::Union(fnode[0], gnode[0]));
    //                     }
    //                 }
    //             }
    //             StackValue::CreateNode(key, h) => {
    //                 let high = result_stack.pop().unwrap();
    //                 let low = result_stack.pop().unwrap();
    //                 let result = self.create_node(h, low, high);
    //                 self.cache.insert(key, result);
    //                 result_stack.push(result);
    //             }
    //             StackValue::Node(f) => {
    //                 result_stack.push(f);
    //             }
    //         }
    //     }
    //     debug_assert!(result_stack.len() == 1);
    //     if let Some(resultid) = result_stack.pop() {
    //         resultid
    //     } else {
    //         panic!("Error: union")
    //     }
    // }

    // pub fn setdiff(&mut self, f: NodeId, g: NodeId) -> NodeId {
    //     #[derive(Debug)]
    //     enum StackValue {
    //         Node(NodeId),
    //         Setdiff(NodeId, NodeId),
    //         CreateNode((Operation, NodeId, NodeId), HeaderId),
    //     }
    //     let mut result_stack = Vec::new();
    //     let mut next_stack = Vec::new();
    //     next_stack.push(StackValue::Setdiff(f, g));
    //     while let Some(token) = next_stack.pop() {
    //         match token {
    //             StackValue::Setdiff(f, g) => {
    //                 let key = (Operation::Setdiff, f, g);
    //                 if let Some(id) = self.cache.get(&key) {
    //                     result_stack.push(*id);
    //                     continue;
    //                 }
    //                 match (self.get_node(f), self.get_node(g)) {
    //                     (Node::Zero, _) => {
    //                         result_stack.push(self.zero());
    //                     }
    //                     (_, Node::Zero) => {
    //                         result_stack.push(f);
    //                     }
    //                     (Node::One, _) => {
    //                         result_stack.push(self.not(g));
    //                     }
    //                     (_, Node::One) => {
    //                         result_stack.push(self.zero());
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
    //                         if fnode.id() == gnode.id() =>
    //                     {
    //                         result_stack.push(self.zero());
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
    //                         if self.level(f) > self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Node(fnode[1]));
    //                         next_stack.push(StackValue::Setdiff(fnode[0], g));
    //                     }
    //                     (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
    //                         if self.level(f) < self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::Setdiff(f, gnode[0]));
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Setdiff(fnode[1], gnode[1]));
    //                         next_stack.push(StackValue::Setdiff(fnode[0], gnode[0]));
    //                     }
    //                 }
    //             }
    //             StackValue::CreateNode(key, h) => {
    //                 let high = result_stack.pop().unwrap();
    //                 let low = result_stack.pop().unwrap();
    //                 let result = self.create_node(h, low, high);
    //                 self.cache.insert(key, result);
    //                 result_stack.push(result);
    //             }
    //             StackValue::Node(f) => {
    //                 result_stack.push(f);
    //             }
    //         }
    //     }
    //     debug_assert!(result_stack.len() == 1);
    //     if let Some(resultid) = result_stack.pop() {
    //         resultid
    //     } else {
    //         panic!("Error: setdiff")
    //     }
    // }

    // pub fn product(&mut self, f: NodeId, g: NodeId) -> NodeId {
    //     #[derive(Debug)]
    //     enum StackValue {
    //         Product(NodeId, NodeId),
    //         Union,
    //         CreateNode((Operation, NodeId, NodeId), HeaderId),
    //     }
    //     let mut result_stack = Vec::new();
    //     let mut next_stack = Vec::new();
    //     next_stack.push(StackValue::Product(f, g));
    //     while let Some(token) = next_stack.pop() {
    //         match token {
    //             StackValue::Product(f, g) => {
    //                 let key = (Operation::Product, f, g);
    //                 if let Some(id) = self.cache.get(&key) {
    //                     result_stack.push(*id);
    //                     continue;
    //                 }
    //                 match (self.get_node(f), self.get_node(g)) {
    //                     (_, Node::Zero) => {
    //                         result_stack.push(self.zero());
    //                     }
    //                     (_, Node::One) => {
    //                         result_stack.push(f);
    //                     }
    //                     (Node::Zero, _) => {
    //                         result_stack.push(self.zero());
    //                     }
    //                     (Node::One, _) => {
    //                         result_stack.push(g);
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
    //                         if self.level(f) > self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Product(fnode[1], g));
    //                         next_stack.push(StackValue::Product(fnode[0], g));
    //                     }
    //                     (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
    //                         if self.level(f) < self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue::CreateNode(key, gnode.header()));
    //                         next_stack.push(StackValue::Product(f, gnode[1]));
    //                         next_stack.push(StackValue::Product(f, gnode[0]));
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
    //                         next_stack.push(StackValue::CreateNode(key, fnode.header()));
    //                         next_stack.push(StackValue::Union);
    //                         next_stack.push(StackValue::Union);
    //                         next_stack.push(StackValue::Product(fnode[1], gnode[1]));
    //                         next_stack.push(StackValue::Product(fnode[1], gnode[0]));
    //                         next_stack.push(StackValue::Product(fnode[0], gnode[1]));
    //                         next_stack.push(StackValue::Product(fnode[0], gnode[0]));
    //                     }
    //                 }
    //             }
    //             StackValue::Union => {
    //                 let g = result_stack.pop().unwrap();
    //                 let f = result_stack.pop().unwrap();
    //                 let result = self.union(f, g);
    //                 result_stack.push(result);
    //             }
    //             StackValue::CreateNode(key, h) => {
    //                 let high = result_stack.pop().unwrap();
    //                 let low = result_stack.pop().unwrap();
    //                 let result = self.create_node(h, low, high);
    //                 self.cache.insert(key, result);
    //                 result_stack.push(result);
    //             }
    //         }
    //     }
    //     debug_assert!(result_stack.len() == 1);
    //     if let Some(resultid) = result_stack.pop() {
    //         resultid
    //     } else {
    //         panic!("Error: product")
    //     }
    // }

    // pub fn divide(&mut self, f: NodeId, g: NodeId) -> Option<NodeId> {
    //     enum StackValue2 {
    //         Divide(NodeId, NodeId),
    //         Intersection,
    //     }
    //     let mut result_stack = Vec::new();
    //     let mut next_stack = Vec::new();
    //     next_stack.push(StackValue2::Divide(f, g));
    //     while let Some(token) = next_stack.pop() {
    //         match token {
    //             StackValue2::Divide(f, g) => {
    //                 let key = (Operation::Divide, f, g);
    //                 if let Some(id) = self.cache.get(&key) {
    //                     result_stack.push(Some(*id));
    //                     continue;
    //                 }
    //                 match (self.get_node(f), self.get_node(g)) {
    //                     (_, Node::Zero) => {
    //                         result_stack.push(None);
    //                     }
    //                     (_, Node::One) => {
    //                         result_stack.push(Some(f));
    //                     }
    //                     (Node::Zero, _) => {
    //                         result_stack.push(Some(self.zero()));
    //                     }
    //                     (Node::One, _) => {
    //                         result_stack.push(Some(self.zero()));
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
    //                         if self.level(f) > self.level(g) =>
    //                     {
    //                         next_stack.push(StackValue2::Divide(fnode[0], g));
    //                     }
    //                     (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
    //                         if self.level(f) < self.level(g) =>
    //                     {
    //                         result_stack.push(None);
    //                     }
    //                     (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
    //                         // next_stack.push(StackValue2::Header(key, fnode.header()));
    //                         next_stack.push(StackValue2::Intersection);
    //                         next_stack.push(StackValue2::Divide(fnode[1], gnode[1]));
    //                         next_stack.push(StackValue2::Divide(fnode[0], gnode[0]));
    //                     }
    //                 }
    //             }
    //             StackValue2::Intersection => {
    //                 let g = result_stack.pop().unwrap();
    //                 let f = result_stack.pop().unwrap();
    //                 match (f, g) {
    //                     (Some(f), Some(g)) => {
    //                         let result = self.intersect(f, g);
    //                         result_stack.push(Some(result));
    //                     }
    //                     (Some(f), None) => {
    //                         result_stack.push(Some(f));
    //                     }
    //                     (None, Some(g)) => {
    //                         result_stack.push(Some(g));
    //                     }
    //                     _ => result_stack.push(None),
    //                 }
    //             }
    //         }
    //     }
    //     debug_assert!(result_stack.len() == 1);
    //     if let Some(resultid) = result_stack.pop() {
    //         resultid
    //     } else {
    //         panic!("Error: product")
    //     }
    // }
}

impl Dot for ZddManager {
    fn dot_impl<T>(&self, io: &mut T, id: NodeId, visited: &mut HashSet<NodeId>)
    where
        T: std::io::Write,
    {
        if visited.contains(&id) {
            return;
        }
        let node = self.get_node(id);
        match node {
            Node::Zero => {
                let s = format!("\"obj{}\" [shape=square, label=\"0\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::One => {
                let s = format!("\"obj{}\" [shape=square, label=\"1\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::NonTerminal(fnode) => {
                let s = format!(
                    "\"obj{}\" [shape=circle, label=\"{}\"];\n",
                    fnode.id(),
                    self.label(id)
                );
                io.write_all(s.as_bytes()).unwrap();
                for (i, xid) in fnode.iter().enumerate() {
                    let x = self.get_node(*xid);
                    if let Node::Zero | Node::One | Node::NonTerminal(_) = x {
                        self.dot_impl(io, *xid, visited);
                        let s = format!(
                            "\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n",
                            fnode.id(),
                            xid,
                            i
                        );
                        io.write_all(s.as_bytes()).unwrap();
                    }
                }
            }
        };
        visited.insert(id);
    }
}

// impl Gc for Bdd {
//     type Node = Node;

//     fn clear_cache(&mut self) {
//         self.cache.clear();
//     }

//     fn clear_table(&mut self) {
//         self.utable.clear();
//     }

//     fn gc_impl(&mut self, f: &Self::Node, visited: &mut HashSet<Self::Node>) {
//         if visited.contains(f) {
//             return
//         }
//         if let Node::NonTerminal(fnode) = f {
//             let key = (fnode.header().id(), fnode[0].id(), fnode[1].id());
//             self.utable.insert(key, f.clone());
//             for x in fnode.iter() {
//                 self.gc_impl(x, visited);
//             }
//         }
//         visited.insert(f.clone());
//     }
// }

impl ZddManager {
    pub fn count(&self, node: NodeId) -> (u64, u64) {
        let mut visited = HashSet::default();
        let edges = self.count_edge_impl(node, &mut visited);
        edges
    }

    fn count_edge_impl(&self, node: NodeId, visited: &mut HashSet<NodeId>) -> (u64, u64) {
        let key = node;
        if let Some(_) = visited.get(&key) {
            return (0, 0);
        }
        match self.get_node(node) {
            Node::NonTerminal(fnode) => {
                let tmp0 = self.count_edge_impl(fnode[0], visited);
                let tmp1 = self.count_edge_impl(fnode[1], visited);
                visited.insert(key);
                (tmp0.0 + tmp1.0 + 1, tmp0.1 + tmp1.1 + 2)
            }
            Node::Zero | Node::One => {
                visited.insert(key);
                (1, 0)
            }
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;

    impl Drop for Node {
        fn drop(&mut self) {
            println!("Dropping Node{}", self.id());
        }
    }

    #[test]
    fn new_header() {
        let h = NodeHeader::new(0, 0, "test", 2);
        println!("{:?}", h);
        println!("{:?}", h.level());
    }

    #[test]
    fn new_terminal() {
        let zero = Node::Zero;
        let one = Node::One;
        println!("{:?}", zero);
        println!("{:?}", one);
    }

    #[test]
    fn new_test1() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        println!("{:?}", dd.get_node(x));
        let y = dd.create_node(h2, dd.zero(), dd.one());
        println!("{:?}", dd.get_node(y));
    }

    #[test]
    fn new_test2() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let h3 = dd.create_header(2, "z");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.create_node(h3, dd.zero(), dd.one());
        let tmp1 = dd.union(x, y);
        let tmp2 = dd.union(x, z);
        let tmp3 = dd.setdiff(tmp1, tmp2);
        println!("{}", dd.dot_string(tmp3));
    }

    #[test]
    fn new_test3() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let h3 = dd.create_header(2, "z");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.create_node(h3, dd.zero(), dd.one());
        let tmp1 = dd.union(x, y);
        let tmp2 = dd.union(x, z);
        let tmp3 = dd.product(tmp1, tmp2);
        println!("{}", dd.dot_string(tmp3));
    }

    #[test]
    fn new_test4() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let h3 = dd.create_header(2, "z");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.create_node(h3, dd.zero(), dd.one());
        let tmp = dd.product(x, y);
        let abc = dd.product(tmp, z);
        println!("abc\n{}", dd.dot_string(abc));
        let bc = dd.product(y, z);
        println!("bc\n{}", dd.dot_string(tmp));
        let ac = dd.product(x, z);
        println!("ac\n{}", dd.dot_string(tmp));
        let tmp = dd.union(abc, bc);
        println!("abc+bc\n{}", dd.dot_string(tmp));
        let s = dd.union(tmp, ac);
        println!("{}", dd.dot_string(s));
        let tmp3 = dd.divide(s, bc);
        if let Some(tmp3) = tmp3 {
            println!("{}", dd.dot_string(tmp3));
        } else {
            println!("None");
        }
    }
}

