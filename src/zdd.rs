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
use std::ops::Index;
use std::slice::Iter;

use crate::common::*;
use crate::dot::Dot;
use crate::nodes::*;

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
    cache: HashMap<(Operation, NodeId, NodeId), NodeId>,
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
        let zero = {
            let tmp = Node::Zero;
            let id = tmp.id();
            nodes.push(tmp);
            debug_assert!(id == nodes[id].id());
            id
        };
        let one = {
            let tmp = Node::One;
            let id = tmp.id();
            nodes.push(tmp);
            debug_assert!(id == nodes[id].id());
            id
        };
        let undet = {
            let tmp = Node::Undet;
            let id = tmp.id();
            nodes.push(tmp);
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
        let node = Node::NonTerminal(NonTerminalBDD {
            id,
            header: headerid,
            edges: [low, high],
        });
        self.nodes.push(node);
        debug_assert!(id == self.nodes[id].id());
        id
    }

    pub fn create_header(&mut self, level: Level, label: &str) -> HeaderId {
        let id = self.headers.len();
        let tmp= NodeHeader::new(id, level, label, 2);
        self.headers.push(tmp);
        debug_assert!(id == self.headers[id].id());
        id
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
        (self.headers.len(), self.nodes.len(), self.cache.len())
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

#[derive(Debug, PartialEq, Eq, Hash)]
enum Operation {
    Intersect,
    Union,
    Setdiff,
    Product,
    Division,
}

impl ZddManager {
    pub fn intersect(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Intersect, f, g);
        if let Some(id) = self.cache.get(&key) {
            return *id;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => g,
            (_, Node::Undet) => f,
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => self.zero(),
            (Node::One, _) => g,
            (_, Node::One) => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let f0 = fnode[0];
                self.intersect(f0, g)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let g0 = gnode[0];
                self.intersect(f, g0)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.intersect(f0, g0);
                let high = self.intersect(f1, g1);
                self.create_node(headerid, low, high)
            }
        };
        self.cache.insert(key, result);
        result
    }

    pub fn union(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Union, f, g);
        if let Some(id) = self.cache.get(&key) {
            return *id;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => f,
            (_, Node::Undet) => g,
            (Node::Zero, _) => g,
            (_, Node::Zero) => f,
            (Node::One, Node::One) => self.one(),
            (Node::NonTerminal(fnode), Node::One) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.union(f0, self.one());
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::One, Node::NonTerminal(gnode)) => {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.union(self.one(), g0);
                let high = g1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.union(f0, g);
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.union(f, g0);
                let high = g1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.union(f0, g0);
                let high = self.union(f1, g1);
                self.create_node(headerid, low, high)
            }
        };
        self.cache.insert(key, result);
        result
    }

    pub fn setdiff(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Setdiff, f, g);
        if let Some(id) = self.cache.get(&key) {
            return *id;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => f,
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => f,
            (Node::One, Node::One) => self.zero(),
            (Node::NonTerminal(fnode), Node::One) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.setdiff(f0, self.one());
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::One, Node::NonTerminal(gnode)) => {
                let g0 = gnode[0];
                self.setdiff(self.one(), g0)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => {
                self.zero()
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.setdiff(f0, g);
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let g0 = gnode[0];
                self.setdiff(f, g0)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.setdiff(f0, g0);
                let high = self.setdiff(f1, g1);
                self.create_node(headerid, low, high)
            }
        };
        self.cache.insert(key, result);
        result
    }

    pub fn product(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Product, f, g);
        if let Some(id) = self.cache.get(&key) {
            return *id;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => self.zero(),
            (_, Node::One) => f,
            (Node::One, _) => g,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.product(f0, g);
                let high = self.product(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.product(f, g0);
                let high = self.product(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.product(f0, g0);
                let high = self.product(f1, g1);
                let tmp = self.product(f1, g0);
                let high = self.union(high, tmp);
                let tmp = self.product(f0, g1);
                let high = self.union(high, tmp);
                self.create_node(headerid, low, high)
            }
        };
        self.cache.insert(key, result);
        result
    }

    pub fn divide(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Division, f, g);
        if let Some(id) = self.cache.get(&key) {
            return *id;
        }
        let result = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (_, Node::Zero) => self.undet(),
            (_, Node::One) => f,
            (Node::Zero, _) => self.zero(),
            (Node::One, _) => g,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(f) > self.level(g) =>
            {
                let f0 = fnode[0];
                self.divide(f0, g)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(_gnode))
                if self.level(f) < self.level(g) =>
            {
                self.undet()
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let x = self.divide(f0, g0);
                let y = self.divide(f1, g1);
                self.intersect(x, y)
            }
        };
        self.cache.insert(key, result);
        result
    }
}

impl Dot for ZddManager {
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
                let s = format!("\"obj{}\" [shape=square, label=\"*\"];\n", id);
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
                    fnode.id(),
                    self.label(id).unwrap()
                );
                io.write_all(s.as_bytes()).unwrap();
                for (i, xid) in fnode.iter().enumerate() {
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
    fn test_union() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let h3 = dd.create_header(2, "z");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.create_node(h3, dd.zero(), dd.one());
        let tmp1 = dd.union(x, y);
        let tmp2 = dd.union(tmp1, z);
        println!("{}", dd.dot_string(tmp2));
    }

    #[test]
    fn test_intersect() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let h3 = dd.create_header(2, "z");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.create_node(h3, dd.zero(), dd.one());
        let tmp1 = dd.union(x, y);
        let tmp2 = dd.union(y, z);
        let tmp3 = dd.intersect(tmp1, tmp2);
        println!("{}", dd.dot_string(tmp3));
    }

    #[test]
    fn test_intersect2() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let h3 = dd.create_header(2, "z");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.create_node(h3, dd.zero(), dd.one());
        let tmp1 = dd.intersect(x, y);
        let tmp2 = dd.intersect(tmp1, z);
        println!("{}", dd.dot_string(tmp2));
    }

    #[test]
    fn test_setdiff() {
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
    fn test_product() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "x");
        let h2 = dd.create_header(1, "y");
        let h3 = dd.create_header(2, "z");
        let x = dd.create_node(h1, dd.zero(), dd.one());
        let y = dd.create_node(h2, dd.zero(), dd.one());
        let z = dd.create_node(h3, dd.zero(), dd.one());
        let tmp1 = dd.product(x, y);
        let tmp2 = dd.union(tmp1, z);
        println!("{}", dd.dot_string(tmp2));
    }

    #[test]
    fn test_product2() {
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
    fn test_divide() {
        let mut dd = ZddManager::new();
        let h1 = dd.create_header(0, "a");
        let h2 = dd.create_header(1, "b");
        let h3 = dd.create_header(2, "c");
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
        println!("abc+bc+ac\n{}", dd.dot_string(s));
        let tmp3 = dd.divide(s, bc);
        println!("(abc+bc+ac)/bc\n{}", dd.dot_string(tmp3));
    }
}
