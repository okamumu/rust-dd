use crate::mtmdd_ops::MtMddOperation;
use crate::nodes::*;
use common::prelude::*;

#[derive(Debug)]
pub struct TerminalNumber<Value> {
    id: NodeId,
    value: Value,
}

impl<Value> TerminalNumber<Value> {
    pub fn new(id: NodeId, value: Value) -> Self {
        Self { id, value }
    }

    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }
}

impl<Value> Terminal for TerminalNumber<Value>
where
    Value: MddValue,
{
    type Value = Value;

    #[inline]
    fn value(&self) -> Self::Value {
        self.value
    }
}

#[derive(Debug)]
pub enum Node<V> {
    NonTerminal(NonTerminalMDD),
    Terminal(TerminalNumber<V>),
    Undet,
}

impl<V> Node<V>
where
    V: MddValue,
{
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Terminal(x) => x.id,
            Self::Undet => 0,
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
pub struct MtMddManager<V> {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node<V>>,
    undet: NodeId,
    vtable: BddHashMap<V, NodeId>,
    utable: BddHashMap<(HeaderId, Box<[NodeId]>), NodeId>,
    cache: BddHashMap<(MtMddOperation, NodeId, NodeId), NodeId>,
    // Slots in `nodes` reclaimed by gc(), available for reuse.
    freelist: Vec<NodeId>,
}

impl<V> DDForest for MtMddManager<V>
where
    V: MddValue,
{
    type Node = Node<V>;
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
            Node::Terminal(_) | Node::Undet => None,
        })
    }

    fn label(&self, id: &NodeId) -> Option<&str> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(&fnode.headerid()).map(|x| x.label()),
            Node::Terminal(_) | Node::Undet => None,
        })
    }
}

impl<V> MtMddManager<V>
where
    V: MddValue,
{
    pub fn new() -> Self {
        let headers = Vec::default();
        let mut nodes = Vec::default();
        let undet = {
            let tmp = Node::Undet;
            let id = tmp.id();
            nodes.push(tmp);
            debug_assert!(id == nodes[id].id());
            id
        };
        let vtable = BddHashMap::default();
        let utable = BddHashMap::default();
        let cache = BddHashMap::default();
        Self {
            headers,
            nodes,
            undet,
            vtable,
            utable,
            cache,
            freelist: Vec::new(),
        }
    }

    fn alloc(&mut self, node: impl FnOnce(NodeId) -> Node<V>) -> NodeId {
        let id = if let Some(slot) = self.freelist.pop() {
            self.nodes[slot] = node(slot);
            slot
        } else {
            let id = self.nodes.len();
            self.nodes.push(node(id));
            id
        };
        debug_assert!(id == self.nodes[id].id());
        id
    }

    fn new_nonterminal(&mut self, header: HeaderId, nodes: &[NodeId]) -> NodeId {
        self.alloc(|id| Node::NonTerminal(NonTerminalMDD::new(id, header, nodes)))
    }

    fn new_terminal(&mut self, value: V) -> NodeId {
        self.alloc(|id| Node::Terminal(TerminalNumber::new(id, value)))
    }

    /// Mark-and-sweep garbage collection. Marks all nodes reachable from `roots`
    /// plus the `Undet` terminal; reclaims the rest (including unreferenced value
    /// terminals, which are also dropped from the value table) onto the free
    /// list, drops dead unique-table entries, and flushes the cache. Does not
    /// compact, so surviving `NodeId`s stay valid. Returns slots reclaimed.
    pub fn gc(&mut self, roots: &[NodeId]) -> usize {
        let n = self.nodes.len();
        let mut live = vec![false; n];
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
        self.vtable.retain(|_, &mut v| live[v]);
        // Keep memoized results that only reference surviving nodes (operands
        // and results may be value terminals, also covered by `live`); drop only
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

    pub fn value(&mut self, value: V) -> NodeId {
        if let Some(&x) = self.vtable.get(&value) {
            return x;
        }
        let node = self.new_terminal(value);
        self.vtable.insert(value, node);
        node
    }

    pub fn create_node(&mut self, h: HeaderId, nodes: &[NodeId]) -> NodeId {
        if let Some(&first) = nodes.first() {
            if nodes.iter().all(|&x| first == x) {
                return first;
            }
        }
        let key = (h, nodes.to_vec().into_boxed_slice());
        if let Some(&x) = self.utable.get(&key) {
            return x;
        }
        let node = self.new_nonterminal(h, nodes);
        self.utable.insert(key, node);
        node
    }

    #[inline]
    pub fn size(&self) -> (usize, usize, usize, usize) {
        (
            self.headers.len(),
            self.nodes.len(),
            self.vtable.len(),
            self.cache.len(),
        )
    }

    #[inline]
    pub fn undet(&self) -> NodeId {
        self.undet
    }

    #[inline]
    pub fn get_cache(&self) -> &BddHashMap<(MtMddOperation, NodeId, NodeId), NodeId> {
        &self.cache
    }

    #[inline]
    pub fn get_mut_cache(&mut self) -> &mut BddHashMap<(MtMddOperation, NodeId, NodeId), NodeId> {
        &mut self.cache
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
