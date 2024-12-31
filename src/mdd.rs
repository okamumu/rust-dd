use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::slice::Iter;

use crate::common::{HashMap, HashSet, HeaderId, Level, NodeId};

use crate::dot::Dot;
use crate::nodes::*;

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

#[derive(Debug)]
pub enum Node {
    NonTerminal(NonTerminalMDD),
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

#[derive(Debug)]
pub struct MddManager {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node>,
    zero: NodeId,
    one: NodeId,
    undet: NodeId,
    utable: HashMap<(HeaderId, Box<[NodeId]>), NodeId>,
    cache: HashMap<(Operation, NodeId, NodeId), NodeId>,
}

impl DDForest for MddManager {
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

impl MddManager {
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

    fn new_nonterminal(&mut self, header: HeaderId, nodes: &[NodeId]) -> NodeId {
        let id = self.nodes.len();
        let tmp = Node::NonTerminal(NonTerminalMDD::new(id, header, nodes));
        self.nodes.push(tmp);
        debug_assert!(id == self.nodes[id].id());
        id
    }

    pub fn create_header(&mut self, level: Level, label: &str, edge_num: usize) -> HeaderId {
        let id = self.headers.len();
        let tmp = NodeHeader::new(id, level, label, edge_num);
        self.headers.push(tmp);
        debug_assert!(id == self.headers[id].id());
        id
    }

    pub fn create_node(&mut self, header: HeaderId, nodes: &[NodeId]) -> NodeId {
        if let Some(&first) = nodes.first() {
            if nodes.iter().all(|&x| first == x) {
                return first;
            }
        }
        let key = (header, nodes.to_vec().into_boxed_slice());
        if let Some(&nodeid) = self.utable.get(&key) {
            return nodeid;
        }
        let node = self.new_nonterminal(header, nodes);
        self.utable.insert(key, node);
        node
    }

    #[inline]
    pub fn size(&self) -> (usize, usize, usize) {
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
    Not,
    And,
    Or,
    XOr,
    Replace,
}

impl MddManager {
    pub fn not(&mut self, f: NodeId) -> NodeId {
        let key = (Operation::Not, f, 0);
        if let Some(&nodeid) = self.cache.get(&key) {
            return nodeid;
        }
        let node = match self.get_node(f).unwrap() {
            Node::Undet => self.undet(),
            Node::Zero => self.one(),
            Node::One => self.zero(),
            Node::NonTerminal(fnode) => {
                let headerid = fnode.headerid();
                let nodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = nodeid.iter().map(|&f| self.not(f)).collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn and(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::And, f, g);
        if let Some(&nodeid) = self.cache.get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => self.zero(),
            (Node::One, _) => g,
            (_, Node::Zero) => self.zero(),
            (_, Node::One) => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.and(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.and(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.and(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn or(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Or, f, g);
        if let Some(&nodeid) = self.cache.get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => g,
            (Node::One, _) => self.one(),
            (_, Node::Zero) => f,
            (_, Node::One) => self.one(),
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.or(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.or(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.or(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn xor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::XOr, f, g);
        if let Some(&nodeid) = self.cache.get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => g,
            (Node::One, _) => self.not(g),
            (_, Node::Zero) => f,
            (_, Node::One) => self.not(f),
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => {
                self.zero()
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.xor(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.xor(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.xor(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
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

    pub fn replace(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Replace, f, g);
        if let Some(&nodeid) = self.cache.get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => g,
            (_, Node::Undet) => f,
            (Node::Zero, _) => self.zero(),
            (Node::One, _) => self.one(),
            (Node::NonTerminal(fnode), Node::Zero) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::One) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.replace(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }
}

// impl Gc for Mdd {
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
//             let key = (fnode.header().id(), fnode.iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
//             self.utable.insert(key, f.clone());
//             for x in fnode.iter() {
//                 self.gc_impl(x, visited);
//             }
//         }
//         visited.insert(f.clone());
//     }
// }

impl MddManager {
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
                let mut result = (1, 0);
                let fnodeid = fnode.iter().cloned().collect::<Vec<_>>();
                for x in fnodeid {
                    let tmp = self.count_edge_impl(x, visited);
                    result.0 += tmp.0;
                    result.1 += tmp.1 + 1;
                }
                visited.insert(key);
                result
            }
            Node::One | Node::Zero | Node::Undet => {
                visited.insert(key);
                (1, 0)
            }
        }
    }
}

impl Dot for MddManager {
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
            Node::Zero => {
                let s = format!("\"obj{}\" [shape=square, label=\"0\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::One => {
                let s = format!("\"obj{}\" [shape=square, label=\"1\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::Undet => {
                let s = format!("\"obj{}\" [shape=square, label=\"?\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::NonTerminal(fnode) => {
                let s = format!(
                    "\"obj{}\" [shape=circle, label=\"{}\"];\n",
                    fnode.id(),
                    self.label(id).unwrap()
                );
                io.write_all(s.as_bytes()).unwrap();
                for (i, &xid) in fnode.iter().enumerate() {
                    if let Node::Undet | Node::Zero | Node::One | Node::NonTerminal(_) =
                        self.get_node(xid).unwrap()
                    {
                        self.dot_impl(io, xid, visited);
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
            _ => (),
        };
        visited.insert(id);
    }
}

// impl Mdd {
//     pub fn build_from_pathset<P>(&mut self, headers: &[NodeHeader], pathset: P) {
//         for x in P {
//             for i in domain[level] {
//                 x[level]
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    impl Drop for Node {
        fn drop(&mut self) {
            println!("Dropping Node{}", self.id());
        }
    }

    #[test]
    fn test_create_node() {
        let mut dd = MddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let x = dd.create_node(h1, &[dd.zero(), dd.one()]);
        println!("{:?}", dd.get_node(x));
        let y = dd.create_node(h2, &[dd.zero(), dd.one()]);
        println!("{:?}", dd.get_node(y));
    }

    #[test]
    fn test_and() {
        let mut dd = MddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
        let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
        let z = dd.and(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_or() {
        let mut dd = MddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
        let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
        let z = dd.or(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_xor() {
        let mut dd = MddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
        let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
        let z = dd.xor(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_ite() {
        let mut dd = MddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
        let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
        let z = dd.ite(x, y, dd.one());
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_count() {
        let mut dd = MddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
        let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
        let z = dd.ite(x, y, dd.one());
        println!("{:?}", dd.count(z));
    }

    #[test]
    fn test_replace() {
        let mut dd = MddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let x = dd.create_node(h1, &[dd.zero(), dd.undet(), dd.one()]);
        let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
        let z = dd.and(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
        let w = dd.replace(z, dd.one());
        println!("{:?}", dd.get_node(w));
        println!("{}", dd.dot_string(w));
    }
}
