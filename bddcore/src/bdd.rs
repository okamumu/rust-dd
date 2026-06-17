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

use common::prelude::*;
use crate::nodes::*;
use crate::bdd_ops::Operation;

pub struct BddManager {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node>,
    zero: NodeId,
    one: NodeId,
    undet: NodeId,
    // Keys/values are stored as u32 (node and header counts fit comfortably in
    // 32 bits): halves the memory of these two large tables and the bytes hashed
    // per lookup, which dominate apply on big diagrams. NodeId/HeaderId stay
    // usize at the public boundary; casts are confined to the helpers below.
    utable: BddHashMap<(u32, u32, u32), u32>,
    cache: BddHashMap<(Operation, u32, u32), u32>,
}

impl DDForest for BddManager {
    type Node = Node;
    type NodeHeader = NodeHeader;

    #[inline]
    fn get_node(&self, id: &NodeId) -> Option<&Self::Node> {
        self.nodes.get(*id)
    }

    #[inline]
    fn get_header(&self, id: &HeaderId) -> Option<&Self::NodeHeader> {
        self.headers.get(*id)
    }

    fn level(&self, id: &NodeId) -> Option<Level> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(&fnode.headerid()).map(|x| x.level()),
            Node::Zero | Node::One | Node::Undet => None,
        })
    }

    fn label(&self, id: &NodeId) -> Option<&str> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(&fnode.headerid()).map(|x| x.label()),
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
        let utable = BddHashMap::default();
        let cache = BddHashMap::default();
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

    /// Fast level lookup for the apply hot path.
    ///
    /// Returns the node's level (non-terminals) or a sentinel `Level::MAX` for
    /// terminals (terminals sit below all variables). Drops the `Option`
    /// wrapping of `DDForest::level` in the inner apply comparisons.
    #[inline]
    pub(crate) fn node_level(&self, id: NodeId) -> Level {
        match &self.nodes[id] {
            Node::NonTerminal(fnode) => self.headers[fnode.headerid()].level(),
            _ => Level::MAX,
        }
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
        let key = (header as u32, low as u32, high as u32);
        if let Some(&nodeid) = self.utable.get(&key) {
            return nodeid as NodeId;
        }
        let node = self.new_nonterminal(header, low, high);
        self.utable.insert(key, node as u32);
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

    /// Look up a memoized apply result. Casts the u32-stored value back to NodeId.
    #[inline]
    pub(crate) fn cache_get(&self, key: &(Operation, u32, u32)) -> Option<NodeId> {
        self.cache.get(key).map(|&v| v as NodeId)
    }

    /// Memoize an apply result, storing it narrowed to u32.
    #[inline]
    pub(crate) fn cache_put(&mut self, key: (Operation, u32, u32), val: NodeId) {
        self.cache.insert(key, val as u32);
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

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
}