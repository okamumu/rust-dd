use std::io::BufWriter;
use std::ops::Index;
use std::slice::Iter;

use crate::common::{EdgeValue, HashMap, HashSet, HeaderId, Level, NodeId};

use crate::nodes::{NodeHeader, NonTerminal};

use crate::dot::Dot;

// use crate::gc::Gc;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct EvEdge<V> {
    value: V,
    node: NodeId,
}

impl<V> EvEdge<V>
where
    V: EdgeValue,
{
    pub fn new(value: V, node: NodeId) -> Self {
        Self { value, node }
    }

    #[inline]
    pub fn value(&self) -> V {
        self.value
    }

    #[inline]
    pub fn node(&self) -> NodeId {
        self.node
    }
}

#[derive(Debug)]
pub struct NonTerminalEvMDD<V> {
    id: NodeId,
    header: HeaderId,
    nodes: Box<[EvEdge<V>]>,
}

impl<V> NonTerminalEvMDD<V>
where
    V: EdgeValue,
{
    pub fn new(id: NodeId, header: HeaderId, nodes: Vec<EvEdge<V>>) -> Self {
        Self {
            id,
            header,
            nodes: nodes.into_boxed_slice(),
        }
    }

    #[inline]
    fn id(&self) -> NodeId {
        self.id
    }

    #[inline]
    fn headerid(&self) -> HeaderId {
        self.header
    }

    fn iter(&self) -> Iter<EvEdge<V>> {
        self.nodes.iter()
    }
}

impl<V> Index<usize> for NonTerminalEvMDD<V>
where
    V: EdgeValue,
{
    type Output = EvEdge<V>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

// #[macro_export]
// macro_rules! nodes {
//     ($($elem:expr),*) => {
//         vec![$($elem.clone()),*]
//     };
// }

#[derive(Debug, PartialEq, Eq, Hash)]
enum Operation {
    Add,
    Sub,
    // MUL,
    // DIV,
    Min,
    Max,
}

#[derive(Debug)]
pub enum Node<V> {
    NonTerminal(NonTerminalEvMDD<V>),
    Omega,
    Infinity,
}

impl<V> Node<V>
where
    V: EdgeValue,
{
    fn new_nonterminal(id: NodeId, header: HeaderId, edges: Vec<EvEdge<V>>) -> Self {
        Node::NonTerminal(NonTerminalEvMDD::new(id, header, edges))
    }

    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Omega => 0,
            Self::Infinity => 1,
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
pub struct EvMddManager<V>
where
    V: EdgeValue,
{
    headers: Vec<NodeHeader>,
    nodes: Vec<Node<V>>,
    omega: NodeId,
    infinity: NodeId,
    utable: HashMap<(HeaderId, Box<[EvEdge<V>]>), NodeId>,
    cache: HashMap<(Operation, NodeId, NodeId, V), EvEdge<V>>,
}

impl<V> EvMddManager<V>
where
    V: EdgeValue,
{
    pub fn new() -> Self {
        let headers = Vec::new();
        let mut nodes = Vec::new();
        let omega = {
            let x = Node::Omega;
            let id = x.id();
            nodes.push(x);
            debug_assert!(id == nodes[id].id());
            id
        };
        let infinity = {
            let x = Node::Infinity;
            let id = x.id();
            nodes.push(x);
            debug_assert!(id == nodes[id].id());
            id
        };
        let utable = HashMap::default();
        let cache = HashMap::default();
        Self {
            headers,
            nodes,
            omega,
            infinity,
            utable,
            cache,
        }
    }

    fn new_nonterminal(&mut self, header: HeaderId, edges: Vec<EvEdge<V>>) -> NodeId {
        let x = Node::new_nonterminal(self.nodes.len(), header, edges);
        let id = x.id();
        self.nodes.push(x);
        debug_assert!(id == self.nodes[id].id());
        id
    }

    pub fn size(&self) -> (usize, usize, usize) {
        (self.headers.len(), self.nodes.len(), self.utable.len())
    }

    pub fn create_header(&mut self, level: Level, label: &str, edge_num: usize) -> HeaderId {
        let id = self.headers.len();
        let h = NodeHeader::new(id, level, label, edge_num);
        self.headers.push(h);
        debug_assert!(id == self.headers[id].id());
        id
    }

    pub fn create_node(&mut self, h: HeaderId, edges: &[EvEdge<V>]) -> (V, NodeId) {
        if let Some(first) = edges.first() {
            if edges.iter().all(|x| first == x) {
                return (first.value(), first.node());
            }
        }

        let mut edges = edges.to_vec();
        let mu = edges.iter().map(|x| x.value()).min().unwrap();
        edges.iter_mut().for_each(|x| x.value = x.value() - mu);

        let key = (h, edges.to_vec().into_boxed_slice());
        if let Some(&x) = self.utable.get(&key) {
            return (mu, x);
        }
        let node = self.new_nonterminal(h, edges);
        self.utable.insert(key, node);
        (mu, node)
    }

    pub fn create_edge(&mut self, h: HeaderId, edges: &[EvEdge<V>]) -> EvEdge<V> {
        let (value, node) = self.create_node(h, edges);
        EvEdge::new(value, node)
    }

    pub fn omega(&self) -> NodeId {
        self.omega
    }

    pub fn infinity(&self) -> NodeId {
        self.infinity
    }

    pub fn get_header(&self, id: HeaderId) -> Option<&NodeHeader> {
        self.headers.get(id)
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node<V>> {
        self.nodes.get(id)
    }

    pub fn level(&self, id: NodeId) -> Option<Level> {
        match self.get_node(id).unwrap() {
            Node::NonTerminal(x) => self.headers.get(x.headerid()).map(|x| x.level()),
            _ => None,
        }
    }

    pub fn label(&self, id: NodeId) -> Option<&str> {
        match self.get_node(id).unwrap() {
            Node::NonTerminal(x) => self.headers.get(x.headerid()).map(|x| x.label()),
            _ => None,
        }
    }
}

impl<V> EvMddManager<V>
where
    V: EdgeValue,
{
    pub fn min(&mut self, fv: V, f: NodeId, gv: V, g: NodeId) -> EvEdge<V> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::Min, f, g, fv - gv);
        if let Some(x) = self.cache.get(&key) {
            return EvEdge::new(mu + x.value(), x.node());
        }
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Infinity, Node::Infinity) => EvEdge::new(V::zero(), self.infinity()),
            (Node::Infinity, _) => EvEdge::new(gv, g),
            (_, Node::Infinity) => EvEdge::new(fv, f),
            (Node::Omega, Node::Omega) => EvEdge::new(mu, self.omega()),
            (Node::Omega, Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.min(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::Omega) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.min(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.min(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.min(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .zip(gnodeid.into_iter())
                    .map(|(fedge, gedge)| {
                        self.min(
                            fv - mu + fedge.value(),
                            fedge.node(),
                            gv - mu + gedge.value(),
                            gedge.node(),
                        )
                    })
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
        }
    }

    pub fn max(&mut self, fv: V, f: NodeId, gv: V, g: NodeId) -> EvEdge<V> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::Max, f, g, fv - gv);
        if let Some(x) = self.cache.get(&key) {
            return EvEdge::new(mu + x.value(), x.node());
        }
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Infinity, _) => EvEdge::new(V::zero(), self.infinity()),
            (_, Node::Infinity) => EvEdge::new(V::zero(), self.infinity()),
            (Node::Omega, Node::Omega) => EvEdge::new(std::cmp::max(fv, gv), self.omega()),
            (Node::Omega, Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.max(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::Omega) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.max(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.max(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.max(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .zip(gnodeid.into_iter())
                    .map(|(fedge, gedge)| {
                        self.max(
                            fv - mu + fedge.value(),
                            fedge.node(),
                            gv - mu + gedge.value(),
                            gedge.node(),
                        )
                    })
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + val, node)
            }
        }
    }

    pub fn add(&mut self, fv: V, f: NodeId, gv: V, g: NodeId) -> EvEdge<V> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::Add, f, g, fv - gv);
        if let Some(x) = self.cache.get(&key) {
            return EvEdge::new(mu + mu + x.value(), x.node());
        }
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Infinity, _) => EvEdge::new(V::zero(), self.infinity()),
            (_, Node::Infinity) => EvEdge::new(V::zero(), self.infinity()),
            (Node::Omega, Node::Omega) => EvEdge::new(fv + gv, self.omega()),
            (Node::Omega, Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.add(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::Omega) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.add(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.add(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.add(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + mu + val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .zip(gnodeid.into_iter())
                    .map(|(fedge, gedge)| {
                        self.add(
                            fv - mu + fedge.value(),
                            fedge.node(),
                            gv - mu + gedge.value(),
                            gedge.node(),
                        )
                    })
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(mu + mu + val, node)
            }
        }
    }

    pub fn sub(&mut self, fv: V, f: NodeId, gv: V, g: NodeId) -> EvEdge<V> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::Sub, f, g, fv - gv);
        if let Some(x) = self.cache.get(&key) {
            return *x; //EvEdge::new(x.value(), x.node());
        }
        match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
            (Node::Infinity, _) => EvEdge::new(V::zero(), self.infinity()),
            (_, Node::Infinity) => EvEdge::new(V::zero(), self.infinity()),
            (Node::Omega, Node::Omega) => EvEdge::new(fv - gv, self.omega()),
            (Node::Omega, Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.sub(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(val, node)
            }
            (Node::NonTerminal(fnode), Node::Omega) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.sub(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) > self.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .map(|fedge| self.sub(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
                if self.level(f) < self.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = gnodeid
                    .into_iter()
                    .map(|gedge| self.sub(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(val, node)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let edges: Vec<_> = fnodeid
                    .into_iter()
                    .zip(gnodeid.into_iter())
                    .map(|(fedge, gedge)| {
                        self.sub(
                            fv - mu + fedge.value(),
                            fedge.node(),
                            gv - mu + gedge.value(),
                            gedge.node(),
                        )
                    })
                    .collect();
                let (val, node) = self.create_node(headerid, &edges);
                self.cache.insert(key, EvEdge::new(val, node));
                EvEdge::new(val, node)
            }
        }
    }

    // pub fn mul(&mut self, fv: V, f: NodeId, gv: V, g: NodeId) -> EvEdge<V> {
    //     let mu = std::cmp::min(fv, gv);
    //     let key = (Operation::Sub, f, g, fv - gv);
    //     if let Some(x) = self.cache.get(&key) {
    //         return *x; //EvEdge::new(x.value(), x.node());
    //     }
    //     match (self.get_node(f).unwrap(), self.get_node(g).unwrap()) {
    //         (Node::Infinity, _) => EvEdge::new(V::zero(), self.infinity()),
    //         (_, Node::Infinity) => EvEdge::new(V::zero(), self.infinity()),
    //         (Node::Omega, Node::Omega) => EvEdge::new(fv * gv, self.omega()),
    //         (Node::Omega, Node::NonTerminal(gnode)) => {
    //             let headerid = gnode.headerid();
    //             let gnodeid: Vec<_> = gnode.iter().cloned().collect();
    //             let edges: Vec<_> = gnodeid
    //                 .into_iter()
    //                 .map(|gedge| self.mul(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
    //                 .collect();
    //             let (val, node) = self.create_node(headerid, &edges);
    //             self.cache.insert(key, EvEdge::new(val, node));
    //             EvEdge::new(val, node)
    //         }
    //         (Node::NonTerminal(fnode), Node::Omega) => {
    //             let headerid = fnode.headerid();
    //             let fnodeid: Vec<_> = fnode.iter().cloned().collect();
    //             let edges: Vec<_> = fnodeid
    //                 .into_iter()
    //                 .map(|fedge| self.mul(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
    //                 .collect();
    //             let (val, node) = self.create_node(headerid, &edges);
    //             self.cache.insert(key, EvEdge::new(val, node));
    //             EvEdge::new(val, node)
    //         }
    //         (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
    //             if self.level(f) > self.level(g) =>
    //         {
    //             let headerid = fnode.headerid();
    //             let fnodeid: Vec<_> = fnode.iter().cloned().collect();
    //             let edges: Vec<_> = fnodeid
    //                 .into_iter()
    //                 .map(|fedge| self.mul(fv - mu + fedge.value(), fedge.node(), gv - mu, g))
    //                 .collect();
    //             let (val, node) = self.create_node(headerid, &edges);
    //             self.cache.insert(key, EvEdge::new(val, node));
    //             EvEdge::new(val, node)
    //         }
    //         (Node::NonTerminal(fnode), Node::NonTerminal(gnode))
    //             if self.level(f) < self.level(g) =>
    //         {
    //             let headerid = gnode.headerid();
    //             let gnodeid: Vec<_> = gnode.iter().cloned().collect();
    //             let edges: Vec<_> = gnodeid
    //                 .into_iter()
    //                 .map(|gedge| self.mul(fv - mu, f, gv - mu + gedge.value(), gedge.node()))
    //                 .collect();
    //             let (val, node) = self.create_node(headerid, &edges);
    //             self.cache.insert(key, EvEdge::new(val, node));
    //             EvEdge::new(val, node)
    //         }
    //         (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
    //             let headerid = fnode.headerid();
    //             let fnodeid: Vec<_> = fnode.iter().cloned().collect();
    //             let gnodeid: Vec<_> = gnode.iter().cloned().collect();
    //             let edges: Vec<_> = fnodeid
    //                 .into_iter()
    //                 .zip(gnodeid.into_iter())
    //                 .map(|(fedge, gedge)| {
    //                     self.mul(
    //                         fv - mu + fedge.value(),
    //                         fedge.node(),
    //                         gv - mu + gedge.value(),
    //                         gedge.node(),
    //                     )
    //                 })
    //                 .collect();
    //             let (val, node) = self.create_node(headerid, &edges);
    //             self.cache.insert(key, EvEdge::new(val, node));
    //             EvEdge::new(val, node)
    //         }
    //     }
    // }

    // pub fn max(&mut self, fv: E, f: &Node<E>, gv: E, g: &Node<E>) -> Edge<E> {
    //     let mu = std::cmp::min(fv, gv);
    //     let key = (Operation::Max, f.id(), g.id(), fv-gv);
    //     match self.cache.get(&key) {
    //         Some(x) => Edge::new(mu+x.value(), x.node().clone()),
    //         None => {
    //             match (f, g) {
    //                 (Node::Infinity, _) => Edge::new(E::zero(), self.infinity()),
    //                 (_, Node::Infinity) => Edge::new(E::zero(), self.infinity()),
    //                 (Node::Omega, Node::Omega) => Edge::new(std::cmp::max(fv, gv), self.omega()),
    //                 (Node::Omega, Node::NonTerminal(_)) if fv <= gv => Edge::new(gv, g.clone()),
    //                 (Node::Omega, Node::NonTerminal(gnode)) if fv > gv => {
    //                     let edges = gnode.iter()
    //                         .map(|gedge| self.max(fv-mu, f, gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(_), Node::Omega) if fv >= gv => Edge::new(fv, f.clone()),
    //                 (Node::NonTerminal(fnode), Node::Omega) if fv < gv => {
    //                     let edges = fnode.iter()
    //                         .map(|fedge| self.max(fv-mu+fedge.value(), fedge.node(), gv-mu, g)).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
    //                     let edges = fnode.iter()
    //                         .map(|fedge| self.max(fv-mu+fedge.value(), fedge.node(), gv-mu, g)).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
    //                     let edges = gnode.iter()
    //                         .map(|gedge| self.max(fv-mu, f, gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
    //                     let edges = fnode.iter().zip(gnode.iter())
    //                         .map(|(fedge,gedge)| self.max(fv-mu+fedge.value(), fedge.node(), gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 _ => panic!("error"),
    //             }
    //         }
    //     }
    // }

    // pub fn add(&mut self, fv: E, f: &Node<E>, gv: E, g: &Node<E>) -> Edge<E> {
    //     let mu = std::cmp::min(fv, gv);
    //     let key = (Operation::Add, f.id(), g.id(), fv-gv);
    //     match self.cache.get(&key) {
    //         Some(x) => Edge::new(mu+mu+x.value(), x.node().clone()),
    //         None => {
    //             match (f, g) {
    //                 (Node::Infinity, _) => Edge::new(E::zero(), self.infinity()),
    //                 (_, Node::Infinity) => Edge::new(E::zero(), self.infinity()),
    //                 (Node::Omega, Node::Omega) => Edge::new(fv+gv, self.omega()),
    //                 (Node::Omega, Node::NonTerminal(_)) => Edge::new(fv+gv, g.clone()),
    //                 (Node::NonTerminal(_), Node::Omega) => Edge::new(fv+gv, f.clone()),
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
    //                     let edges = fnode.iter()
    //                         .map(|fedge| self.add(fv-mu+fedge.value(), fedge.node(), gv-mu, g)).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
    //                     let edges = gnode.iter()
    //                         .map(|gedge| self.add(fv-mu, f, gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
    //                     let edges = fnode.iter().zip(gnode.iter())
    //                         .map(|(fedge,gedge)| self.add(fv-mu+fedge.value(), fedge.node(), gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu+mu, self.create_node(fnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 _ => panic!("error"),
    //             }
    //         }
    //     }
    // }

    // // not yet: the algorithm is wrong. it should be fixed.
    // pub fn sub(&mut self, fv: E, f: &Node<E>, gv: E, g: &Node<E>) -> Edge<E> {
    //     let mu = std::cmp::min(fv, gv);
    //     let key = (Operation::Sub, f.id(), g.id(), fv-gv);
    //     match self.cache.get(&key) {
    //         Some(x) => Edge::new(x.value(), x.node().clone()),
    //         None => {
    //             match (f, g) {
    //                 (Node::Infinity, _) => Edge::new(E::zero(), self.infinity()),
    //                 (_, Node::Infinity) => Edge::new(E::zero(), self.infinity()),
    //                 (Node::Omega, Node::Omega) => Edge::new(fv-gv, self.omega()),
    //                 (Node::Omega, Node::NonTerminal(_)) => Edge::new(fv-gv, g.clone()),
    //                 (Node::NonTerminal(_), Node::Omega) => Edge::new(fv-gv, f.clone()),
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
    //                     let edges = fnode.iter()
    //                         .map(|fedge| self.add(fv-mu-fedge.value(), fedge.node(), gv-mu, g)).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
    //                     let edges = gnode.iter()
    //                         .map(|gedge| self.add(fv-mu, f, gv-mu-gedge.value(), gedge.node())).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
    //                     let edges = fnode.iter().zip(gnode.iter())
    //                         .map(|(fedge,gedge)| self.add(fv-mu-fedge.value(), fedge.node(), gv-mu-gedge.value(), gedge.node())).collect::<Vec<_>>();
    //                     let edge = Edge::new(mu+mu, self.create_node(fnode.header(), &edges));
    //                     self.cache.insert(key, edge.clone());
    //                     edge
    //                 },
    //                 _ => panic!("error"),
    //             }
    //         }
    //     }
    // }
}

// impl<E> Gc for EvMdd<E> where E: EdgeValue {
//     type Node = Node<E>;

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
//             let key = (fnode.header().id(), fnode.iter().map(|x| (x.value(), x.node().id())).collect::<Vec<_>>().into_boxed_slice());
//             // let key = (fnode.header().id(),
//             //     fnode.iter().map(|x| x.value()).collect::<Vec<_>>().into_boxed_slice(),
//             //     fnode.iter().map(|x| x.node().id()).collect::<Vec<_>>().into_boxed_slice());
//             self.utable.insert(key, f.clone());
//             for x in fnode.iter() {
//                 self.gc_impl(x.node(), visited);
//             }
//         }
//         visited.insert(f.clone());
//     }
// }

impl<E> EvMddManager<E> where E: EdgeValue,
{
    pub fn dot<T>(&self, io: &mut T, node: EvEdge<E>)
    where
        T: std::io::Write,
    {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited: HashSet<NodeId> = HashSet::default();
        io.write_all(s1.as_bytes()).unwrap();

        let e = node.clone();
        let id = -1;
        let s = format!(
            "\"obj{}\" [shape=point, label=\"\"];\n",
            id,
        );
        io.write_all(s.as_bytes()).unwrap();
        if let Node::Omega | Node::NonTerminal(_) = self.get_node(e.node()).unwrap() {
            let s = format!(
                "\"obj{}:{}:{}\" [shape=square, label=\"{}\"];\n",
                id,
                e.node(),
                e.value(),
                e.value()
            );
            io.write_all(s.as_bytes()).unwrap();
            let s = format!(
                "\"obj{}\" -> \"obj{}:{}:{}\" [arrowhead=none];\n",
                id,
                id,
                e.node(),
                e.value()
            );
            io.write_all(s.as_bytes()).unwrap();
            let s = format!(
                "\"obj{}:{}:{}\" -> \"obj{}\";\n",
                id,
                e.node(),
                e.value(),
                e.node()
            );
            io.write_all(s.as_bytes()).unwrap();
        }

        self.dot_impl(io, node, &mut visited);
        io.write_all(s2.as_bytes()).unwrap();
    }

    pub fn dot_string(&self, node: EvEdge<E>) -> String {
        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            self.dot(&mut io, node);
        }
        std::str::from_utf8(&buf).unwrap().to_string()
    }

    fn dot_impl<T>(&self, io: &mut T, edge: EvEdge<E>, visited: &mut HashSet<NodeId>)
    where
        T: std::io::Write,
    {
        let (_vavlue, id) = (edge.value(), edge.node());
        if visited.contains(&id) {
            return;
        }
        match self.get_node(id).unwrap() {
            Node::Omega => {
                let s = format!(
                    "\"obj{}\" [shape=rectangle, height=0.1, width=2, label=\"Omega\"];\n",
                    id
                );
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::NonTerminal(fnode) => {
                let s = format!(
                    "\"obj{}\" [shape=circle, label=\"{}\"];\n",
                    id,
                    self.label(id).unwrap()
                );
                io.write_all(s.as_bytes()).unwrap();
                for (i, e) in fnode.iter().enumerate() {
                    self.dot_impl(io, e.clone(), visited);
                    if let Node::Omega | Node::NonTerminal(_) = self.get_node(e.node()).unwrap() {
                        let s = format!(
                            "\"obj{}:{}:{}\" [shape=square, label=\"{}\"];\n",
                            id,
                            e.node(),
                            e.value(),
                            e.value()
                        );
                        io.write_all(s.as_bytes()).unwrap();
                        let s = format!(
                            "\"obj{}\" -> \"obj{}:{}:{}\" [label=\"{}\", arrowhead=none];\n",
                            id,
                            id,
                            e.node(),
                            e.value(),
                            i
                        );
                        io.write_all(s.as_bytes()).unwrap();
                        let s = format!(
                            "\"obj{}:{}:{}\" -> \"obj{}\";\n",
                            id,
                            e.node(),
                            e.value(),
                            e.node()
                        );
                        io.write_all(s.as_bytes()).unwrap();
                    }
                }
            }
            _ => (),
        };
        visited.insert(id.clone());
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

    struct Table<E> {
        labels: Vec<String>,
        inputs: Vec<Vec<Option<usize>>>,
        outputs: Vec<Option<E>>,
    }

    impl<E> Table<E>
    where
        E: EdgeValue,
    {
        fn new(dd: &EvMddManager<E>) -> Self {
            let mut labels = Vec::new();
            for h in dd.headers.iter() {
                labels.push(h.label().to_string());
            }
            Self {
                labels: labels,
                inputs: Vec::new(),
                outputs: Vec::new(),
            }
        }

        fn size(&self) -> usize {
            self.labels.len()
        }

        fn create_table(dd: &EvMddManager<E>, v: E, f: NodeId) -> Self {
            let mut tab = Self::new(dd);
            let mut p = vec![None; tab.size()];
            tab.create_table_impl(dd, v, f, &mut p);
            tab
        }

        fn create_table_impl(
            &mut self,
            dd: &EvMddManager<E>,
            v: E,
            f: NodeId,
            path: &mut Vec<Option<usize>>,
        ) {
            match dd.get_node(f).unwrap() {
                Node::Infinity => {
                    self.inputs.push(path.clone());
                    self.outputs.push(None);
                }
                Node::Omega => {
                    self.inputs.push(path.clone());
                    self.outputs.push(Some(v));
                }
                Node::NonTerminal(fnode) => {
                    for (i, e) in fnode.iter().enumerate() {
                        path[dd.level(f).unwrap()] = Some(i);
                        self.create_table_impl(dd, v + e.value(), e.node(), path);
                        path[dd.level(f).unwrap()] = None;
                    }
                }
            };
        }

        fn print_table(&self) {
            for l in self.labels.iter() {
                print!("{} ", l);
            }
            println!("|");
            for (p, x) in self.inputs.iter().zip(self.outputs.iter()) {
                for i in p.iter() {
                    match i {
                        Some(x) => print!("{} ", x),
                        None => print!("U "),
                    }
                }
                match x {
                    Some(x) => println!("| {}", x),
                    None => println!("| Inf"),
                }
            }
        }
    }

    #[test]
    fn test_create_node1() {
        let mut dd = EvMddManager::new();
        let h = dd.create_header(0, "x", 2);
        let (v1, x) = dd.create_node(h, &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())]);
        println!("{:?}", x);
        let (v2, y) = dd.create_node(h, &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())]);
        println!("{:?}", y);
        println!("{}", dd.dot_string(EvEdge::new(v1, x)));
    }

    #[test]
    fn test_min1() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let (vx, x) = dd.create_node(
            h1,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let (vy, y) = dd.create_node(
            h2,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let z = dd.min(vx, x, vy, y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{} {}", z.value(), dd.dot_string(z));
        let tabx = Table::create_table(&dd, vx, x);
        tabx.print_table();
        let taby = Table::create_table(&dd, vy, y);
        taby.print_table();
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
    }

    #[test]
    fn test_min2() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let h3 = dd.create_header(2, "z", 3);

        let (vf11, f11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(0, dd.infinity())],
        );
        debug_assert!(vf11 == 0);
        let (vf12, f12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vf12 == 0);
        let (vf21, f21) = dd.create_node(h2, &[EvEdge::new(0, f11), EvEdge::new(2, f11)]);
        debug_assert!(vf21 == 0);
        let (vf22, f22) = dd.create_node(h2, &[EvEdge::new(1, f11), EvEdge::new(0, f12)]);
        debug_assert!(vf22 == 0);
        let (vf, f) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, f21),
                EvEdge::new(1, f22),
                EvEdge::new(2, f22),
            ],
        );
        debug_assert!(vf == 0);

        let (vg11, g11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        debug_assert!(vg11 == 0);
        let (vg12, g12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vg12 == 0);
        let (vg21, g21) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(0, dd.infinity())]);
        debug_assert!(vg21 == 0);
        let (vg22, g22) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(2, g12)]);
        debug_assert!(vg22 == 0);
        let (vg, g) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, g21),
                EvEdge::new(2, g21),
                EvEdge::new(1, g22),
            ],
        );
        debug_assert!(vg == 0);

        let z = dd.min(0, f, 0, g);
        println!("{}", dd.dot_string(z));

        println!("f");
        let tabf = Table::create_table(&dd, vf, f);
        tabf.print_table();

        println!("g");
        let tabg = Table::create_table(&dd, vg, g);
        tabg.print_table();

        println!("min(f,g)");
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
    }

    #[test]
    fn test_max1() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let (vx, x) = dd.create_node(
            h1,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let (vy, y) = dd.create_node(
            h2,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let z = dd.max(vx, x, vy, y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{} {}", z.value(), dd.dot_string(z));
        let tabx = Table::create_table(&dd, vx, x);
        tabx.print_table();
        let taby = Table::create_table(&dd, vy, y);
        taby.print_table();
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
    }

    #[test]
    fn test_max2() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let h3 = dd.create_header(2, "z", 3);

        let (vf11, f11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(0, dd.infinity())],
        );
        debug_assert!(vf11 == 0);
        let (vf12, f12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vf12 == 0);
        let (vf21, f21) = dd.create_node(h2, &[EvEdge::new(0, f11), EvEdge::new(2, f11)]);
        debug_assert!(vf21 == 0);
        let (vf22, f22) = dd.create_node(h2, &[EvEdge::new(1, f11), EvEdge::new(0, f12)]);
        debug_assert!(vf22 == 0);
        let (vf, f) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, f21),
                EvEdge::new(1, f22),
                EvEdge::new(2, f22),
            ],
        );
        debug_assert!(vf == 0);

        let (vg11, g11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        debug_assert!(vg11 == 0);
        let (vg12, g12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vg12 == 0);
        let (vg21, g21) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(0, dd.infinity())]);
        debug_assert!(vg21 == 0);
        let (vg22, g22) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(2, g12)]);
        debug_assert!(vg22 == 0);
        let (vg, g) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, g21),
                EvEdge::new(2, g21),
                EvEdge::new(1, g22),
            ],
        );
        debug_assert!(vg == 0);

        let z = dd.max(0, f, 0, g);
        println!("{}", dd.dot_string(z));

        println!("f");
        let tabf = Table::create_table(&dd, vf, f);
        tabf.print_table();

        println!("g");
        let tabg = Table::create_table(&dd, vg, g);
        tabg.print_table();

        println!("{}", dd.dot_string(z));

        println!("max(f,g)");
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
    }

    #[test]
    fn test_add1() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let (vx, x) = dd.create_node(
            h1,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let (vy, y) = dd.create_node(
            h2,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let z = dd.add(vx, x, vy, y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{} {}", z.value(), dd.dot_string(z));
        let tabx = Table::create_table(&dd, vx, x);
        tabx.print_table();
        let taby = Table::create_table(&dd, vy, y);
        taby.print_table();
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
    }

    #[test]
    fn test_add2() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let h3 = dd.create_header(2, "z", 3);

        let (vf11, f11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(0, dd.infinity())],
        );
        debug_assert!(vf11 == 0);
        let (vf12, f12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vf12 == 0);
        let (vf21, f21) = dd.create_node(h2, &[EvEdge::new(0, f11), EvEdge::new(2, f11)]);
        debug_assert!(vf21 == 0);
        let (vf22, f22) = dd.create_node(h2, &[EvEdge::new(1, f11), EvEdge::new(0, f12)]);
        debug_assert!(vf22 == 0);
        let (vf, f) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, f21),
                EvEdge::new(1, f22),
                EvEdge::new(2, f22),
            ],
        );
        debug_assert!(vf == 0);

        let (vg11, g11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        debug_assert!(vg11 == 0);
        let (vg12, g12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vg12 == 0);
        let (vg21, g21) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(0, dd.infinity())]);
        debug_assert!(vg21 == 0);
        let (vg22, g22) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(2, g12)]);
        debug_assert!(vg22 == 0);
        let (vg, g) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, g21),
                EvEdge::new(2, g21),
                EvEdge::new(1, g22),
            ],
        );
        debug_assert!(vg == 0);

        let tmp = dd.min(0, f, 0, g);
        let z = dd.add(0, f, tmp.value(), tmp.node());
        println!("{}", dd.dot_string(z));

        println!("f");
        let tabf = Table::create_table(&dd, vf, f);
        tabf.print_table();

        println!("g");
        let tabg = Table::create_table(&dd, vg, g);
        tabg.print_table();

        println!("{}", dd.dot_string(z));

        println!("max(f,g)");
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
    }

    #[test]
    fn test_sub1() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let (vx, x) = dd.create_node(
            h1,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let (vy, y) = dd.create_node(
            h2,
            &[EvEdge::new(1, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        let z = dd.sub(vx, x, vy, y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{} {}", z.value(), dd.dot_string(z));
        let tabx = Table::create_table(&dd, vx, x);
        tabx.print_table();
        let taby = Table::create_table(&dd, vy, y);
        taby.print_table();
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_sub2() {
        let mut dd = EvMddManager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let h3 = dd.create_header(2, "z", 3);

        let (vf11, f11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(0, dd.infinity())],
        );
        debug_assert!(vf11 == 0);
        let (vf12, f12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vf12 == 0);
        let (vf21, f21) = dd.create_node(h2, &[EvEdge::new(0, f11), EvEdge::new(2, f11)]);
        debug_assert!(vf21 == 0);
        let (vf22, f22) = dd.create_node(h2, &[EvEdge::new(1, f11), EvEdge::new(0, f12)]);
        debug_assert!(vf22 == 0);
        let (vf, f) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, f21),
                EvEdge::new(1, f22),
                EvEdge::new(2, f22),
            ],
        );
        debug_assert!(vf == 0);

        let (vg11, g11) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.omega()), EvEdge::new(2, dd.omega())],
        );
        debug_assert!(vg11 == 0);
        let (vg12, g12) = dd.create_node(
            h1,
            &[EvEdge::new(0, dd.infinity()), EvEdge::new(0, dd.omega())],
        );
        debug_assert!(vg12 == 0);
        let (vg21, g21) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(0, dd.infinity())]);
        debug_assert!(vg21 == 0);
        let (vg22, g22) = dd.create_node(h2, &[EvEdge::new(0, g11), EvEdge::new(2, g12)]);
        debug_assert!(vg22 == 0);
        let (vg, g) = dd.create_node(
            h3,
            &[
                EvEdge::new(0, g21),
                EvEdge::new(2, g21),
                EvEdge::new(1, g22),
            ],
        );
        debug_assert!(vg == 0);

        let tmp = dd.min(0, f, 0, g);
        let z = dd.sub(0, f, tmp.value(), tmp.node());
        println!("{}", dd.dot_string(z));

        println!("f");
        let tabf = Table::create_table(&dd, vf, f);
        tabf.print_table();

        println!("g");
        let tabg = Table::create_table(&dd, vg, g);
        tabg.print_table();

        println!("{}", dd.dot_string(z));

        println!("max(f,g)");
        let tabz = Table::create_table(&dd, z.value(), z.node());
        tabz.print_table();
    }
    
    //     #[test]
    //     fn test_evmdd_max() {
    //         let mut dd: EvMdd = EvMdd::new();
    //         let h1 = NodeHeader::new(0, 0, "x", 2);
    //         let h2 = NodeHeader::new(1, 1, "y", 2);
    //         let h3 = NodeHeader::new(2, 2, "z", 3);

    //         let f11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
    //         let f12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let f21 = dd.create_node(&h2, &[Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
    //         let f22 = dd.create_node(&h2, &[Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
    //         let f = dd.create_node(&h3, &[Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

    //         let g11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
    //         let g12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let g21 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
    //         let g22 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
    //         let g = dd.create_node(&h3, &[Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

    //         let z = dd.max(0, &f, 0, &g);

    //         let mut buf = vec![];
    //         {
    //             let mut io = BufWriter::new(&mut buf);
    //             z.dot(&mut io);
    //         }
    //         let s = std::str::from_utf8(&buf).unwrap();
    //         println!("{}", s);

    //         println!("f");
    //         for x in table(&dd, 0, &f) {
    //             println!("{:?}", x);
    //         }

    //         println!("g");
    //         for x in table(&dd, 0, &g) {
    //             println!("{:?}", x);
    //         }

    //         println!("max(f,g)");
    //         for x in table(&dd, z.value(), z.node()) {
    //             println!("{:?}", x);
    //         }
    //     }

    //     #[test]
    //     fn test_evmdd_add() {
    //         let mut dd: EvMdd = EvMdd::new();
    //         let h1 = NodeHeader::new(0, 0, "x", 2);
    //         let h2 = NodeHeader::new(1, 1, "y", 2);
    //         let h3 = NodeHeader::new(2, 2, "z", 3);

    //         let f11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
    //         let f12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let f21 = dd.create_node(&h2, &[Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
    //         let f22 = dd.create_node(&h2, &[Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
    //         let f = dd.create_node(&h3, &[Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

    //         let g11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
    //         let g12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let g21 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
    //         let g22 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
    //         let g = dd.create_node(&h3, &[Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

    //         let z = dd.add(0, &f, 0, &g);

    //         let mut buf = vec![];
    //         {
    //             let mut io = BufWriter::new(&mut buf);
    //             z.dot(&mut io);
    //         }
    //         let s = std::str::from_utf8(&buf).unwrap();
    //         println!("{}", s);

    //         println!("f");
    //         for x in table(&dd, 0, &f) {
    //             println!("{:?}", x);
    //         }

    //         println!("g");
    //         for x in table(&dd, 0, &g) {
    //             println!("{:?}", x);
    //         }

    //         println!("f+g");
    //         for x in table(&dd, z.value(), z.node()) {
    //             println!("{:?}", x);
    //         }
    //     }

    //     #[test]
    //     fn test_evmdd_sub() {
    //         let mut dd: EvMdd = EvMdd::new();
    //         let h1 = NodeHeader::new(0, 0, "x", 2);
    //         let h2 = NodeHeader::new(1, 1, "y", 2);
    //         let h3 = NodeHeader::new(2, 2, "z", 3);

    //         let f11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
    //         let f12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let f21 = dd.create_node(&h2, &[Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
    //         let f22 = dd.create_node(&h2, &[Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
    //         let f = dd.create_node(&h3, &[Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

    //         let g11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
    //         let g12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let g21 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
    //         let g22 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
    //         let g = dd.create_node(&h3, &[Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

    //         let z = dd.sub(0, &f, 0, &g);

    //         let mut buf = vec![];
    //         {
    //             let mut io = BufWriter::new(&mut buf);
    //             z.dot(&mut io);
    //         }
    //         let s = std::str::from_utf8(&buf).unwrap();
    //         println!("{}", s);

    //         println!("f");
    //         for x in table(&dd, 0, &f) {
    //             println!("{:?}", x);
    //         }

    //         println!("g");
    //         for x in table(&dd, 0, &g) {
    //             println!("{:?}", x);
    //         }

    //         println!("f-g");
    //         for x in table(&dd, z.value(), z.node()) {
    //             println!("{:?}", x);
    //         }
    //     }

    //     #[test]
    //     fn test_dot() {
    //         let mut dd: EvMdd = EvMdd::new();
    //         let h1 = NodeHeader::new(0, 0, "x", 2);
    //         let h2 = NodeHeader::new(1, 1, "y", 2);
    //         let h3 = NodeHeader::new(2, 2, "z", 3);

    //         let f11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
    //         let f12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let f21 = dd.create_node(&h2, &[Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
    //         let f22 = dd.create_node(&h2, &[Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
    //         let f = dd.create_node(&h3, &[Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

    //         let g11 = dd.create_node(&h1, &[Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
    //         let g12 = dd.create_node(&h1, &[Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
    //         let g21 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
    //         let g22 = dd.create_node(&h2, &[Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
    //         let g = dd.create_node(&h3, &[Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

    //         let z = dd.add(0, &f, 0, &g);

    //         let mut buf = vec![];
    //         {
    //             let mut io = BufWriter::new(&mut buf);
    //             z.dot(&mut io);
    //         }
    //         let s = std::str::from_utf8(&buf).unwrap();
    //         println!("{}", s);
    //     }
}
