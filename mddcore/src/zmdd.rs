//! Zero-suppressed multi-terminal MDD (ZMDD): a decision graph denoting `f: R → 2^S`,
//! a family of **sparse vectors** stratified by terminal label.
//!
//! Semantics (see the `mss` minimal-path-vector layer for the client use):
//! `⟦N(X; c_0..c_{M-1})⟧ = ⟦c_0⟧ ∪ ⋃_{i≥1} { s∪{X=i} : s ∈ ⟦c_i⟧ }`. The 0-edge records
//! nothing (`X=0` = "not in the sparse vector"), a non-0 edge `i` records `X=i`. Terminals
//! are **labels** (`Terminal(v)`) — like a BDD's 0/1-terminals, generalized — and `Undet`
//! is the empty family. Unlike [`MtMddManager`](crate::mtmdd::MtMddManager), the reduction
//! is **zero-suppression** (`create_node` returns the 0-edge when every non-0 edge is
//! `Undet`), never the full-reduction "merge if all edges equal" (which would conflate
//! `X=0` with `X=i`).

use crate::zmdd_ops::ZmddOperation;
use crate::mtmdd::{Node, TerminalNumber};
use crate::nodes::*;
use common::prelude::*;

#[derive(Debug)]
pub struct ZmddManager<V> {
    headers: Vec<NodeHeader>,
    nodes: Vec<Node<V>>,
    undet: NodeId,
    vtable: BddHashMap<V, u32>,
    utable: BddHashMap<(u32, Box<[u32]>), u32>,
    cache: ComputeCache,
    freelist: Vec<u32>,
}

impl<V> DDForest for ZmddManager<V>
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

impl<V> ZmddManager<V>
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
        Self {
            headers,
            nodes,
            undet,
            vtable: BddHashMap::default(),
            utable: BddHashMap::default(),
            cache: ComputeCache::new(),
            freelist: Vec::new(),
        }
    }

    fn alloc(&mut self, node: impl FnOnce(NodeId) -> Node<V>) -> NodeId {
        let id = if let Some(slot) = self.freelist.pop() {
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

    fn new_nonterminal(&mut self, header: HeaderId, nodes: &[NodeId]) -> NodeId {
        self.alloc(|id| Node::NonTerminal(NonTerminalMDD::new(id, header, nodes)))
    }

    fn new_terminal(&mut self, value: V) -> NodeId {
        self.alloc(|id| Node::Terminal(TerminalNumber::new(id, value)))
    }

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
                stack.extend(fnode.iter());
            }
        }

        self.utable.retain(|_, &mut v| live[v as usize]);
        self.vtable.retain(|_, &mut v| live[v as usize]);
        self.cache.retain_live(&live);

        self.freelist.clear();
        for (id, &alive) in live.iter().enumerate() {
            if !alive {
                self.freelist.push(id as u32);
            }
        }
        self.freelist.len()
    }

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
            return x as NodeId;
        }
        let node = self.new_terminal(value);
        self.vtable.insert(value, node as u32);
        node
    }

    /// Create a node under **zero-suppression**: if every non-0 edge is `Undet` (empty), the
    /// node denotes "X = 0 for all vectors" and is elided in favour of the 0-edge. Nodes are
    /// otherwise hash-consed. The full-reduction "merge if all edges equal" rule is
    /// deliberately NOT applied (it would lose the `X=i` records).
    pub fn create_node(&mut self, h: HeaderId, nodes: &[NodeId]) -> NodeId {
        if nodes[1..].iter().all(|&x| x == self.undet) {
            return nodes[0];
        }
        let key = (h as u32, nodes.iter().map(|&x| x as u32).collect());
        if let Some(&x) = self.utable.get(&key) {
            return x as NodeId;
        }
        let node = self.new_nonterminal(h, nodes);
        self.utable.insert(key, node as u32);
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
    /// Force the **baseline member** (the empty sparse vector, i.e. every component on the
    /// 0-edge) into the family with terminal label `v`, and return the new root.
    ///
    /// The baseline vector is a member iff following 0-edges from the root reaches a terminal
    /// rather than `Undet`; this rebuilds that spine so it does. It is a no-op when the member
    /// is already there. Needed because a family converted from a **boolean** structure
    /// function loses it (the source's `Zero` leaf is indistinguishable from "not a member"),
    /// while one converted from a value forest keeps it.
    pub fn set_baseline(&mut self, node: NodeId, v: V) -> NodeId {
        match self.get_node(&node).unwrap() {
            Node::Undet => self.value(v),
            Node::Terminal(_) => node,
            Node::NonTerminal(f) => {
                let h = f.headerid();
                let mut ch: Vec<NodeId> = f.iter().collect();
                ch[0] = self.set_baseline(ch[0], v);
                self.create_node(h, &ch)
            }
        }
    }

    pub fn undet(&self) -> NodeId {
        self.undet
    }

    #[inline]
    pub(crate) fn cache_get(&self, key: &(ZmddOperation, NodeId, NodeId)) -> Option<NodeId> {
        self.cache
            .get(key.0.code(), key.1 as u32, key.2 as u32)
            .map(|v| v as NodeId)
    }

    #[inline]
    pub(crate) fn cache_put(&mut self, key: (ZmddOperation, NodeId, NodeId), val: NodeId) {
        self.cache
            .put(key.0.code(), key.1 as u32, key.2 as u32, val as u32);
    }

    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
