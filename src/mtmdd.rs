use std::hash::{Hash, Hasher};

use crate::common::{HashMap, HashSet, HeaderId, Level, NodeId, TerminalNumberValue};

use crate::nodes::*;

use crate::mdd::NonTerminalMDD;

use crate::dot::Dot;

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
    Value: TerminalNumberValue,
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
    V: TerminalNumberValue,
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
    vtable: HashMap<V, NodeId>,
    utable: HashMap<(HeaderId, Box<[NodeId]>), NodeId>,
    cache: HashMap<(Operation, NodeId, NodeId), NodeId>,
}

impl<V> DDForest for MtMddManager<V>
where
    V: TerminalNumberValue,
{
    type Node = Node<V>;
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
            Node::Terminal(_) | Node::Undet => None,
        })
    }

    fn label(&self, id: NodeId) -> Option<&str> {
        self.get_node(id).and_then(|node| match node {
            Node::NonTerminal(fnode) => self.get_header(fnode.headerid()).map(|x| x.label()),
            Node::Terminal(_) | Node::Undet => None,
        })
    }
}

impl<V> MtMddManager<V>
where
    V: TerminalNumberValue,
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
        let vtable = HashMap::default();
        let utable = HashMap::default();
        let cache = HashMap::default();
        Self {
            headers,
            nodes,
            undet,
            vtable,
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

    fn new_terminal(&mut self, value: V) -> NodeId {
        let id = self.nodes.len();
        let tmp = Node::Terminal(TerminalNumber::new(id, value));
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
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Replace,
}

impl<V> MtMddManager<V>
where
    V: TerminalNumberValue,
{
    pub fn add(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Add, f, g);
        if let Some(&x) = self.cache.get(&key) {
            return x;
        }
        let node = match (&self.get_node(f).unwrap(), &self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(fnode.value() + gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.add(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn sub(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Sub, f, g);
        if let Some(&x) = self.cache.get(&key) {
            return x;
        }
        let node = match (&self.get_node(f).unwrap(), &self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(fnode.value() - gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.sub(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn mul(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Mul, f, g);
        if let Some(&x) = self.cache.get(&key) {
            return x;
        }
        let node = match (&self.get_node(f).unwrap(), &self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(fnode.value() * gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.mul(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn div(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Div, f, g);
        if let Some(&x) = self.cache.get(&key) {
            return x;
        }
        let node = match (&self.get_node(f).unwrap(), &self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                if gnode.value() == V::zero() {
                    return self.undet();
                }
                self.value(fnode.value() / gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.div(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn min(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Min, f, g);
        if let Some(&x) = self.cache.get(&key) {
            return x;
        }
        let node = match (&self.get_node(f).unwrap(), &self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(std::cmp::min(fnode.value(), gnode.value()))
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.min(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn max(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Max, f, g);
        if let Some(&x) = self.cache.get(&key) {
            return x;
        }
        let node = match (&self.get_node(f).unwrap(), &self.get_node(g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(std::cmp::max(fnode.value(), gnode.value()))
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.max(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache.insert(key, node);
        node
    }

    pub fn replace(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Replace, f, g);
        if let Some(x) = self.cache.get(&key) {
            return *x;
        }
        let node = match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Undet, _) => g,
            (_, Node::Undet) => f,
            (Node::Terminal(fnode), _) => f,
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
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
                let headerid = fnode.headerid();
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

// impl<V> Gc for MtMdd<V> where V: TerminalNumberValue {
//     type Node = Node<V>;

//     fn clear_cache(&mut self) {
//         self.cache.clear();
//     }

//     fn clear_table(&mut self) {
//         self.vtable.clear();
//         self.utable.clear();
//     }

//     fn gc_impl(&mut self, f: &Self::Node, visited: &mut HashSet<Self::Node>) {
//         if visited.contains(f) {
//             return
//         }
//         match f {
//             Node::Terminal(fnode) => {
//                 self.vtable.insert(fnode.value(), f.clone());
//             },
//             Node::NonTerminal(fnode) => {
//                 let key = (fnode.header().id(), fnode.iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
//                 self.utable.insert(key, f.clone());
//                 for x in fnode.iter() {
//                     self.gc_impl(x, visited);
//                 }
//             },
//             _ => (),
//         };
//         visited.insert(f.clone());
//     }
// }

impl<V> MtMddManager<V>
where
    V: TerminalNumberValue,
{
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
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                for x in fnodeid {
                    let tmp = self.count_edge_impl(x, visited);
                    result.0 += tmp.0;
                    result.1 += tmp.1 + 1;
                }
                visited.insert(key);
                result
            }
            Node::Terminal(_) | Node::Undet => {
                visited.insert(key);
                (1, 0)
            }
        }
    }
}

impl<V> Dot for MtMddManager<V>
where
    V: TerminalNumberValue,
{
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
                let s = format!("\"obj{}\" [shape=square, label=\"Undet\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::Terminal(fnode) => {
                let s = format!(
                    "\"obj{}\" [shape=square, label=\"{}\"];\n",
                    fnode.id(),
                    fnode.value()
                );
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
                    if let Node::Terminal(_) | Node::NonTerminal(_) | Node::Undet =
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
        };
        visited.insert(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // impl<V> Drop for Node<V> where V: TerminalNumberValue<V> {
    //     fn drop(&mut self) {
    //         println!("Dropping Node{}", self.id());
    //     }
    // }

    #[test]
    fn test_create_node() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let x = dd.create_node(h1, &[v0, v1]);
        println!("{:?}", dd.get_node(x));
        let y = dd.create_node(h2, &[v0, v1]);
        println!("{:?}", dd.get_node(y));
    }

    #[test]
    fn test_add() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.add(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_sub() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.sub(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_mul() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.mul(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_div() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.div(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_min() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.min(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_max() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.max(x, y);
        println!("{:?}", dd.get_node(z));
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_count() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.add(x, y);
        let (nodes, edges) = dd.count(z);
        println!("Nodes: {}, Edges: {}", nodes, edges);
    }

    #[test]
    fn test_replace() {
        let mut dd = MtMddManager::new();
        let h1 = dd.create_header(0, "x", 3);
        let h2 = dd.create_header(1, "y", 3);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let v2 = dd.value(2);
        let x = dd.create_node(h1, &[v0, v1, v2]);
        let y = dd.create_node(h2, &[v0, v1, v2]);
        let z = dd.div(x, y);
        let v100 = dd.value(100);
        let w = dd.replace(z, v100);
        println!("{:?}", dd.get_node(w));
        println!("{}", dd.dot_string(w));
    }
}
