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

use common::prelude::*;
use crate::nodes::*;
use crate::zdd_ops::ZddOperation;

pub struct ZddManager {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node>,
    zero: NodeId,
    one: NodeId,
    undet: NodeId,
    // Keys/values stored as u32 (see BddManager): halves the memory and hashed
    // bytes of these two large tables. NodeId/HeaderId stay usize at the public
    // boundary; casts are confined to create_node and the cache helpers.
    utable: BddHashMap<(u32, u32, u32), u32>,
    cache: BddHashMap<(ZddOperation, u32, u32), u32>,
    // Slots in `nodes` reclaimed by gc(), available for reuse (see BddManager).
    freelist: Vec<u32>,
}

impl DDForest for ZddManager {
    type Node = Node;
    type NodeHeader = NodeHeader;

    #[inline]
    fn get_node(&self, id: &NodeId) -> Option<&Self::Node> {
        self.nodes.get(*id)
    }

    #[inline]
    fn get_header(&self, id: &HeaderId) -> Option<&NodeHeader> {
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
            freelist: Vec::new(),
        }
    }

    fn new_nonterminal(&mut self, headerid: HeaderId, low: NodeId, high: NodeId) -> NodeId {
        let node = |id| Node::NonTerminal(NonTerminalBDD::new(id, headerid, [low, high]));
        let id = if let Some(slot) = self.freelist.pop() {
            // Recycle a slot reclaimed by a previous gc().
            let id = slot as usize;
            self.nodes[id] = node(id);
            id
        } else {
            let id = self.nodes.len();
            self.nodes.push(node(id));
            id
        };
        debug_assert!(id == self.nodes[id].id());
        id
    }

    /// Mark-and-sweep garbage collection. See `BddManager::gc` for semantics:
    /// marks all nodes reachable from `roots` plus terminals, reclaims the rest
    /// onto the free list, drops dead unique-table entries, flushes the cache,
    /// and does not compact (surviving `NodeId`s stay valid). Returns the number
    /// of slots reclaimed.
    pub fn gc(&mut self, roots: &[NodeId]) -> usize {
        let n = self.nodes.len();
        let mut live = vec![false; n];
        live[self.zero] = true;
        live[self.one] = true;
        live[self.undet] = true;

        let mut stack: Vec<NodeId> = roots.iter().copied().filter(|&r| r < n).collect();
        while let Some(id) = stack.pop() {
            if live[id] {
                continue;
            }
            live[id] = true;
            if let Node::NonTerminal(fnode) = &self.nodes[id] {
                stack.push(fnode.edge(0));
                stack.push(fnode.edge(1));
            }
        }

        self.utable.retain(|_, &mut v| live[v as usize]);
        // Keep memoized results that only reference surviving nodes; drop only
        // entries touching a reclaimed slot.
        self.cache
            .retain(|k, &mut v| live[k.1 as usize] && live[k.2 as usize] && live[v as usize]);

        self.freelist.clear();
        for (id, &alive) in live.iter().enumerate() {
            if !alive {
                self.freelist.push(id as u32);
            }
        }
        self.freelist.len()
    }

    /// Number of live (non-reclaimed) node slots, including terminals.
    #[inline]
    pub fn live_node_count(&self) -> usize {
        self.nodes.len() - self.freelist.len()
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
        let id = self.headers.len();
        let tmp = NodeHeader::new(id, level, label, 2);
        self.headers.push(tmp);
        debug_assert!(id == self.headers[id].id());
        id
    }

    pub fn create_node(&mut self, header: HeaderId, low: NodeId, high: NodeId) -> NodeId {
        if high == self.zero {
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

    /// Look up a memoized result. Casts the u32-stored value back to NodeId.
    #[inline]
    pub(crate) fn cache_get(&self, key: &(ZddOperation, u32, u32)) -> Option<NodeId> {
        self.cache.get(key).map(|&v| v as NodeId)
    }

    /// Memoize a result, storing it narrowed to u32.
    #[inline]
    pub(crate) fn cache_put(&mut self, key: (ZddOperation, u32, u32), val: NodeId) {
        self.cache.insert(key, val as u32);
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

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
    