use crate::mdd_ops::MddOperation;
use crate::nodes::*;
use common::prelude::*;

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
    utable: BddHashMap<(HeaderId, Box<[NodeId]>), NodeId>,
    cache: BddHashMap<(MddOperation, NodeId, NodeId), NodeId>,
    // Slots in `nodes` reclaimed by gc(), available for reuse.
    freelist: Vec<NodeId>,
}

impl DDForest for MddManager {
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

    fn new_nonterminal(&mut self, header: HeaderId, nodes: &[NodeId]) -> NodeId {
        let id = if let Some(slot) = self.freelist.pop() {
            // Recycle a slot reclaimed by a previous gc().
            self.nodes[slot] = Node::NonTerminal(NonTerminalMDD::new(slot, header, nodes));
            slot
        } else {
            let id = self.nodes.len();
            self.nodes.push(Node::NonTerminal(NonTerminalMDD::new(id, header, nodes)));
            id
        };
        debug_assert!(id == self.nodes[id].id());
        id
    }

    /// Mark-and-sweep garbage collection (see `bddcore::bdd::BddManager::gc`).
    /// Marks all nodes reachable from `roots` plus the three terminals, reclaims
    /// the rest onto the free list, drops dead unique-table entries, and flushes
    /// the cache. Does not compact, so surviving `NodeId`s stay valid. Returns
    /// the number of slots reclaimed.
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
                stack.extend(fnode.iter().copied());
            }
        }

        self.utable.retain(|_, &mut v| live[v]);
        // Keep memoized results that only reference surviving nodes; drop only
        // entries touching a reclaimed slot.
        self.cache
            .retain(|k, &mut v| live[k.1] && live[k.2] && live[v]);

        self.freelist.clear();
        for (id, &alive) in live.iter().enumerate() {
            if !alive {
                self.freelist.push(id);
            }
        }
        self.freelist.len()
    }

    /// Number of live (non-reclaimed) node slots, including terminals.
    #[inline]
    pub fn live_node_count(&self) -> usize {
        self.nodes.len() - self.freelist.len()
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

    #[inline]
    pub fn get_cache(&self) -> &BddHashMap<(MddOperation, NodeId, NodeId), NodeId> {
        &self.cache
    }

    #[inline]
    pub fn get_mut_cache(&mut self) -> &mut BddHashMap<(MddOperation, NodeId, NodeId), NodeId> {
        &mut self.cache
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
