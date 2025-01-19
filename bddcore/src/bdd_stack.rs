/// BDD (Binary Decision Diagram) implementation.
///
/// Description:
///
/// A BDD is a rooted directed acyclic graph (DAG) with two terminal nodes, 0 and 1.
/// Each non-terminal node has a level and two edges, low and high.
/// The level is an integer that represents the variable of the node.
/// The low and high edges are the child nodes of the node.
///
/// The BDD has a unique table that stores the non-terminal nodes.
/// The table is a hash table that maps a tuple of (level, low, high) to a non-terminal node.
///
/// The BDD has a cache that stores the result of the operations.
/// The cache is a hash table that maps a tuple of (operation, f, g) to a node.
///
/// The BDD has the following operations:
/// - not(f): negation of f
/// - and(f, g): conjunction of f and g
/// - or(f, g): disjunction of f and g
/// - xor(f, g): exclusive or of f and g
/// - imp(f, g): implication of f and g
/// - nand(f, g): nand of f and g
/// - nor(f, g): nor of f and g
/// - xnor(f, g): exclusive nor of f and g
/// - ite(f, g, h): if-then-else of f, g, and h
///
/// The BDD has the following methods:
/// - create_header(level, label): create a new header
/// - create_node(header, low, high): create a new non-terminal node
/// - zero(): return the terminal node 0
/// - one(): return the terminal node 1
/// - size(): return the number of headers, nodes, and the size of the unique table
///
/// The BDD has the following traits:
/// - Gc: garbage collection
/// - Count: count the number of edges
/// - Dot: output the graph in DOT format
///
use std::ops::Index;
use std::slice::Iter;

use crate::common::HashMap;
use crate::common::HashSet;
use crate::common::HeaderId;
use crate::common::Level;
use crate::common::NodeId;
use crate::common::OperationId;

use crate::nodes::*;

use crate::dot::Dot;

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
}

impl Node {
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Zero => 0,
            Self::One => 1,
        }
    }

    pub fn headerid(&self) -> Option<HeaderId> {
        match self {
            Self::NonTerminal(x) => Some(x.headerid()),
            _ => None,
        }
    }
}

pub struct BddManager {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node>,
    zero: NodeId,
    one: NodeId,
    utable: HashMap<(HeaderId, NodeId, NodeId), NodeId>,
    cache: HashMap<(Operation, NodeId, NodeId), NodeId>,
    // next_stack: Vec<Operation>,
    // result_stack: Vec<NodeId>,
}

impl DDForest for BddManager {
    type Node = Node;
    type NodeHeader = NodeHeader;

    #[inline]
    fn get_node(&self, id: NodeId) -> Option<&Self::Node> {
        self.nodes.get(id)
    }

    #[inline]
    fn get_header(&self, id: HeaderId) -> Option<&Self::NodeHeader> {
        self.headers.get(id)
    }

    fn level(&self, id: NodeId) -> Option<Level> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(fnode.header).map(|x| x.level()),
            Node::Zero | Node::One => None,
        })
    }

    fn label(&self, id: NodeId) -> Option<&str> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(fnode.header).map(|x| x.label()),
            Node::Zero | Node::One => None,
        })
    }
}

impl BddManager {
    pub fn new() -> Self {
        let headers = Vec::default();
        let mut nodes = Vec::default();
        let zero = {
            let zeronode = Node::Zero;
            let id = zeronode.id();
            nodes.push(zeronode);
            id
        };
        let one = {
            let onenode = Node::One;
            let id = onenode.id();
            nodes.push(onenode);
            id
        };
        let utable = HashMap::default();
        let cache = HashMap::default();
        // let next_stack = Vec::default();
        // let result_stack = Vec::default();
        Self {
            headers,
            nodes,
            zero,
            one,
            utable,
            cache,
            // next_stack,
            // result_stack,
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
        if low == high {
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

    #[inline]
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
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Operation {
    And,
    Or,
    XOr,
}

// #[derive(Debug)]
// enum Operation {
//     And(NodeId, NodeId),
//     Or(NodeId, NodeId),
//     XOr(NodeId, NodeId),
//     CreateNode((OperationId, NodeId, NodeId), HeaderId),
//     Result(NodeId),
// }

// impl Operation {
//     fn id(&self) -> OperationId {
//         match self {
//             Self::And(_, _) => 0,
//             Self::Or(_, _) => 1,
//             Self::XOr(_, _) => 2,
//             Self::CreateNode(_, _) => 3,
//             Self::Result(_) => 4,
//         }
//     }
// }

// macro_rules! stack_operations {
//     ($self:ident, $($op:expr),*) => {{
//         let ops = [$($op),*];
//         for op in ops.into_iter().rev() {
//             $self.next_stack.push(op);
//         }
//     }};
// }

// type OpKey = (OperationId, NodeId, NodeId);

// impl BddManager {
//     fn apply(&mut self, op: Operation) -> NodeId {
//         self.next_stack.clear();
//         self.result_stack.clear();
//         self.next_stack.push(op);
//         while let Some(op) = self.next_stack.pop() {
//             match op {
//                 Operation::And(f, g) => {
//                     let key = (op.id(), f, g);
//                     if let Some(id) = self.cache.get(&key) {
//                         self.result_stack.push(*id);
//                         continue;
//                     }
//                     self.apply_and(key, f, g);
//                 }
//                 Operation::Or(f, g) => {
//                     let key = (op.id(), f, g);
//                     if let Some(id) = self.cache.get(&key) {
//                         self.result_stack.push(*id);
//                         continue;
//                     }
//                     self.apply_or(key, f, g);
//                 }
//                 Operation::XOr(f, g) => {
//                     let key = (op.id(), f, g);
//                     if let Some(id) = self.cache.get(&key) {
//                         self.result_stack.push(*id);
//                         continue;
//                     }
//                     self.apply_xor(key, f, g);
//                 }
//                 Operation::CreateNode(key, h) => {
//                     let high = self.result_stack.pop().unwrap();
//                     let low = self.result_stack.pop().unwrap();
//                     let result = self.create_node(h, low, high);
//                     self.cache.insert(key, result);
//                     self.result_stack.push(result);
//                 }
//                 Operation::Result(id) => {
//                     self.result_stack.push(id);
//                 }
//             }
//         }
//         debug_assert!(self.result_stack.len() == 1);
//         self.result_stack.pop().unwrap()
//     }

//     fn apply_and(&mut self, key: OpKey, f: NodeId, g: NodeId) {
//         if let (Some(fnode), Some(gnode)) = (self.get_node(f), self.get_node(g)) {
//             match (fnode, gnode) {
//                 (Node::Zero, _) => {
//                     stack_operations!(self, Operation::Result(self.zero()))
//                 }
//                 (_, Node::Zero) => {
//                     stack_operations!(self, Operation::Result(self.zero()))
//                 }
//                 (Node::One, _) => {
//                     stack_operations!(self, Operation::Result(g))
//                 }
//                 (_, Node::One) => {
//                     stack_operations!(self, Operation::Result(f))
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
//                     if fnode.id() == gnode.id() =>
//                 {
//                     stack_operations!(self, Operation::Result(f))
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
//                     if self.level(f) > self.level(g) =>
//                 {
//                     stack_operations!(
//                         self,
//                         Operation::And(fnode[0], g),
//                         Operation::And(fnode[1], g),
//                         Operation::CreateNode(key, fnode.headerid())
//                     )
//                 }
//                 (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
//                     if self.level(f) < self.level(g) =>
//                 {
//                     stack_operations!(
//                         self,
//                         Operation::And(f, gnode[0]),
//                         Operation::And(f, gnode[1]),
//                         Operation::CreateNode(key, gnode.headerid())
//                     )
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
//                     stack_operations!(
//                         self,
//                         Operation::And(fnode[0], gnode[0]),
//                         Operation::And(fnode[1], gnode[1]),
//                         Operation::CreateNode(key, fnode.headerid())
//                     )
//                 }
//             }
//         }
//     }

//     fn apply_or(&mut self, key: OpKey, f: NodeId, g: NodeId) {
//         if let (Some(fnode), Some(gnode)) = (self.get_node(f), self.get_node(g)) {
//             match (fnode, gnode) {
//                 (Node::Zero, _) => {
//                     stack_operations!(self, Operation::Result(g))
//                 }
//                 (_, Node::Zero) => {
//                     stack_operations!(self, Operation::Result(f))
//                 }
//                 (Node::One, _) => {
//                     stack_operations!(self, Operation::Result(self.one()))
//                 }
//                 (_, Node::One) => {
//                     stack_operations!(self, Operation::Result(self.one()))
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
//                     if fnode.id() == gnode.id() =>
//                 {
//                     stack_operations!(self, Operation::Result(f))
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
//                     if self.level(f) > self.level(g) =>
//                 {
//                     stack_operations!(
//                         self,
//                         Operation::Or(fnode[0], g),
//                         Operation::Or(fnode[1], g),
//                         Operation::CreateNode(key, fnode.headerid())
//                     )
//                 }
//                 (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
//                     if self.level(f) < self.level(g) =>
//                 {
//                     stack_operations!(
//                         self,
//                         Operation::Or(f, gnode[0]),
//                         Operation::Or(f, gnode[1]),
//                         Operation::CreateNode(key, gnode.headerid())
//                     )
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
//                     stack_operations!(
//                         self,
//                         Operation::Or(fnode[0], gnode[0]),
//                         Operation::Or(fnode[1], gnode[1]),
//                         Operation::CreateNode(key, fnode.headerid())
//                     )
//                 }
//             }
//         }
//     }

//     fn apply_xor(&mut self, key: OpKey, f: NodeId, g: NodeId) {
//         if let (Some(fnode), Some(gnode)) = (self.get_node(f), self.get_node(g)) {
//             match (fnode, gnode) {
//                 (Node::Zero, _) => {
//                     stack_operations!(self, Operation::Result(g))
//                 }
//                 (_, Node::Zero) => {
//                     stack_operations!(self, Operation::Result(f))
//                 }
//                 (Node::One, Node::One) => {
//                     stack_operations!(self, Operation::Result(self.zero()))
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
//                     if fnode.id() == gnode.id() =>
//                 {
//                     stack_operations!(self, Operation::Result(self.zero()))
//                 }
//                 (Node::NonTerminal(fnode), Node::One) => {
//                     stack_operations!(
//                         self,
//                         Operation::XOr(fnode[0], self.one()),
//                         Operation::XOr(fnode[1], self.one()),
//                         Operation::CreateNode(key, fnode.headerid())
//                     )
//                 }
//                 (Node::One, Node::NonTerminal(gnode)) => {
//                     stack_operations!(
//                         self,
//                         Operation::XOr(gnode[0], self.one()),
//                         Operation::XOr(gnode[1], self.one()),
//                         Operation::CreateNode(key, gnode.headerid())
//                     )
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
//                     if self.level(f) > self.level(g) =>
//                 {
//                     stack_operations!(
//                         self,
//                         Operation::XOr(fnode[0], g),
//                         Operation::XOr(fnode[1], g),
//                         Operation::CreateNode(key, fnode.headerid())
//                     )
//                 }
//                 (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
//                     if self.level(f) < self.level(g) =>
//                 {
//                     stack_operations!(
//                         self,
//                         Operation::XOr(f, gnode[0]),
//                         Operation::XOr(f, gnode[1]),
//                         Operation::CreateNode(key, gnode.headerid())
//                     )
//                 }
//                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
//                     stack_operations!(
//                         self,
//                         Operation::XOr(fnode[0], gnode[0]),
//                         Operation::XOr(fnode[1], gnode[1]),
//                         Operation::CreateNode(key, fnode.headerid())
//                     )
//                 }
//             }
//         }
//     }
// }

impl BddManager {
    pub fn and(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::And, f, g);
        if let Some(x) = self.cache.get(&key) {
            return *x;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => self.zero(),
            (Node::One, _) => g,
            (_, Node::One) => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.and(f0, g);
                let high = self.and(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.and(f, g0);
                let high = self.and(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.and(f0, g0);
                let high = self.and(f1, g1);
                self.create_node(headerid, low, high)
            }
        };
        self.cache.insert(key, result);
        result
    }

    pub fn or(&mut self, f: NodeId, g: NodeId) -> NodeId {
        self.apply(Operation::Or(f, g))
    }

    pub fn xor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        self.apply(Operation::XOr(f, g))
    }

    pub fn not(&mut self, f: NodeId) -> NodeId {
        self.xor(f, self.one())
    }

    pub fn imp(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.not(f);
        self.or(tmp, g)
    }

    pub fn nand(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.and(f, g);
        self.not(tmp)
    }

    pub fn nor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.or(f, g);
        self.not(tmp)
    }

    pub fn xnor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.xor(f, g);
        self.not(tmp)
    }

    pub fn ite(&mut self, f: NodeId, g: NodeId, h: NodeId) -> NodeId {
        let x1 = self.and(f, g);
        let barf = self.not(f);
        let x2 = self.and(barf, h);
        self.or(x1, x2)
    }
}

impl Dot for BddManager {
    fn dot_impl<T>(&self, io: &mut T, id: NodeId, visited: &mut HashSet<NodeId>)
    where
        T: std::io::Write,
    {
        if visited.contains(&id) {
            return;
        }
        let node = self.get_node(id).unwrap();
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
                    id,
                    self.label(id).unwrap()
                );
                io.write_all(s.as_bytes()).unwrap();
                for (i, xid) in fnode.iter().enumerate() {
                    self.dot_impl(io, *xid, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", id, *xid, i);
                    io.write_all(s.as_bytes()).unwrap();
                }
            }
        };
        visited.insert(id);
    }
}

// impl BddManager {
//     fn gc(&mut self) {
//         self.cache.clear();
//         self.utable.clear();
//         self.clear_cache();
//         self.clear_table();
//         let mut visited = HashSet::default();
//         for x in fs.iter() {
//             self.gc_impl(x, &mut visited);
//         }
//     }

//     fn gc_impl(&mut self, f: &Self::Node, visited: &mut HashSet<Self::Node>);

// }

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

impl BddManager {
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
        match self.get_node(node).unwrap() {
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
        let mut dd = BddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        println!("{:?}", dd.get_node(x));
        let y = dd.create_node(h2, dd.zero(), dd.one());
        println!("{:?}", dd.get_node(y));
    }

    #[test]
    fn test_and() {
        let mut dd = BddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.and(x, y);
        println!("{:?}", dd.get_node(x));
        println!("{:?}", dd.get_node(y));
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_or() {
        let mut dd = BddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.or(x, y);
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_xor() {
        let mut dd = BddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.xor(x, y);
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_not() {
        let mut dd = BddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.or(x, y);
        let z = dd.not(z);
        println!("{}", dd.dot_string(z));
    }
}
