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

use crate::common::*;
use crate::nodes::*;

use crate::dot::Dot;

// #[derive(Debug)]
// pub struct NonTerminalBDD {
//     id: NodeId,
//     header: HeaderId,
//     edges: [NodeId; 2],
// }

// impl NonTerminal for NonTerminalBDD {
//     #[inline]
//     fn id(&self) -> NodeId {
//         self.id
//     }

//     #[inline]
//     fn headerid(&self) -> HeaderId {
//         self.header
//     }

//     #[inline]
//     fn iter(&self) -> Iter<NodeId> {
//         self.edges.iter()
//     }
// }

// impl Index<usize> for NonTerminalBDD {
//     type Output = NodeId;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.edges[index]
//     }
// }

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

pub struct BddManager {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node>,
    zero: NodeId,
    one: NodeId,
    undet: NodeId,
    utable: HashMap<(HeaderId, NodeId, NodeId), NodeId>,
    cache: HashMap<(Operation, NodeId, NodeId), NodeId>,
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
            Node::NonTerminal(fnode) => self.get_header(fnode.headerid()).map(|x| x.level()),
            Node::Zero | Node::One | Node::Undet => None,
        })
    }

    fn label(&self, id: NodeId) -> Option<&str> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(fnode.headerid()).map(|x| x.label()),
            Node::Zero | Node::One | Node::Undet => None,
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
            debug_assert!(id == nodes[id].id());
            id
        };
        let one = {
            let onenode = Node::One;
            let id = onenode.id();
            nodes.push(onenode);
            debug_assert!(id == nodes[id].id());
            id
        };
        let undet = {
            let undetnode = Node::Undet;
            let id = undetnode.id();
            nodes.push(undetnode);
            debug_assert!(id == nodes[id].id());
            id
        };
        let utable = HashMap::default();
        let cache = HashMap::default();
        Self {
            headers,
            nodes,
            zero,
            one,
            undet,
            utable,
            cache,
        }
    }

    fn new_nonterminal(&mut self, headerid: HeaderId, low: NodeId, high: NodeId) -> NodeId {
        let id = self.nodes.len();
        let node = Node::NonTerminal(NonTerminalBDD::new(id, headerid, [low, high]));
        self.nodes.push(node);
        debug_assert!(id == self.nodes[id].id());
        id
    }

    pub fn create_header(&mut self, level: Level, label: &str) -> HeaderId {
        let headerid = self.headers.len();
        let header = NodeHeader::new(headerid, level, label, 2);
        self.headers.push(header);
        debug_assert!(headerid == self.headers[headerid].id());
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
    Not,
}

impl BddManager {
    pub fn not(&mut self, f: NodeId) -> NodeId {
        let key = (Operation::Not, f, 0);
        if let Some(x) = self.cache.get(&key) {
            return *x;
        }
        let result = match self.get_node(f).unwrap() {
            Node::Zero => self.one(),
            Node::One => self.zero(),
            Node::NonTerminal(fnode) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.not(f0);
                let high = self.not(f1);
                self.create_node(headerid, low, high)
            },
            Node::Undet => self.undet,
        };
        self.cache.insert(key, result);
        result
    }

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
            (Node::Undet, _) => self.undet,
            (_, Node::Undet) => self.undet,
        };
        self.cache.insert(key, result);
        result
    }

    pub fn or(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Or, f, g);
        if let Some(x) = self.cache.get(&key) {
            return *x;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Zero, _) => g,
            (_, Node::Zero) => f,
            (Node::One, _) => self.one(),
            (_, Node::One) => self.one(),
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.or(f0, g);
                let high = self.or(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.or(f, g0);
                let high = self.or(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.or(f0, g0);
                let high = self.or(f1, g1);
                self.create_node(headerid, low, high)
            }
            (Node::Undet, _) => self.undet,
            (_, Node::Undet) => self.undet,
        };
        self.cache.insert(key, result);
        result
    }

    pub fn xor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::XOr, f, g);
        if let Some(x) = self.cache.get(&key) {
            return *x;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Zero, _) => g,
            (_, Node::Zero) => f,
            (Node::One, _) => self.not(g),
            (_, Node::One) => self.not(f),
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => {
                self.zero()
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.xor(f0, g);
                let high = self.xor(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.xor(f, g0);
                let high = self.xor(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.xor(f0, g0);
                let high = self.xor(f1, g1);
                self.create_node(headerid, low, high)
            }
            (Node::Undet, _) => self.undet,
            (_, Node::Undet) => self.undet,
        };
        self.cache.insert(key, result);
        result
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
    type Node = NodeId;

    fn dot_impl<T>(&self, io: &mut T, id: NodeId, visited: &mut HashSet<NodeId>)
    where
        T: std::io::Write,
    {
        if visited.contains(&id) {
            return;
        }
        let node = self.get_node(id).unwrap();
        match node {
            Node::Undet => {
                let s = format!("\"obj{}\" [shape=square, label=\"?\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
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
                    if let Node::One | Node::Zero | Node::NonTerminal(_) = self.get_node(*xid).unwrap() {
                        self.dot_impl(io, *xid, visited);
                        let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", id, *xid, i);
                        io.write_all(s.as_bytes()).unwrap();
                    }
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
            Node::Zero | Node::One | Node::Undet => {
                visited.insert(key);
                (1, 0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // impl Drop for Node {
    //     fn drop(&mut self) {
    //         println!("Dropping Node{}", self.id());
    //     }
    // }

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
