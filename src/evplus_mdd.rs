use std::rc::Rc;
use std::hash::{Hash, Hasher};

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
    EdgeValue,
};

use crate::nodes::{
    NodeHeader,
    Terminal,
    NonTerminal,
    NonTerminalMDD,
    EvEdge,
};

use crate::dot::{
    Dot,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    ADD,
    SUB,
    MUL,
    DIV,
    MIN,
    MAX,
}

type Node<E> = EvMddNode<E>;
type Edge<E> = EvEdge<E,Node<E>>;

#[derive(Debug,Clone)]
pub enum EvMddNode<E> {
    NonTerminal(Rc<NonTerminalMDD<EvEdge<E,Node<E>>>>),
    Omega,
    Infinity,
}

impl<E> PartialEq for Node<E> where E: EdgeValue {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<E> Eq for Node<E> where E: EdgeValue {}

impl<E> Hash for Node<E> where E: EdgeValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<E> Node<E> where E: EdgeValue {
    fn new_nonterminal(id: NodeId, header: &NodeHeader, edges: &[Edge<E>]) -> Self {
        let x = NonTerminalMDD::new(
            id,
            header.clone(),
            edges.iter().map(|x| x.clone()).collect::<Vec<_>>().into_boxed_slice(),
        );
        Node::NonTerminal(Rc::new(x))
    }

    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Infinity => 0,
            Self::Omega => 1,
        }        
    }

    pub fn header(&self) -> Option<&NodeHeader> {
        match self {
            Self::NonTerminal(x) => Some(x.header()),
            _ => None
        }
    }

    pub fn level(&self) -> Option<Level> {
        match self {
            Self::NonTerminal(x) => Some(x.level()),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct EvMdd<E = i64> where E: EdgeValue {
    num_headers: HeaderId,
    num_nodes: NodeId,
    omega: Node<E>,
    infinity: Node<E>,
    utable: HashMap<(HeaderId, Box<[(E,NodeId)]>), Node<E>>,
    cache: HashMap<(Operation, NodeId, NodeId, E), Edge<E>>,
}

impl<E> EvMdd<E> where E: EdgeValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            infinity: Node::Infinity,
            omega: Node::Omega,
            utable: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    pub fn size(&self) -> (HeaderId, NodeId, usize) {
        (self.num_headers, self.num_nodes, self.utable.len())
    }
    
    pub fn header(&mut self, level: Level, label: &str, edge_num: usize) -> NodeHeader {
        let h = NodeHeader::new(self.num_headers, level, label, edge_num);
        self.num_headers += 1;
        h
    }
    
    pub fn node(&mut self, h: &NodeHeader, edges: &[Edge<E>]) -> Result<Node<E>,String> {
        if h.edge_num() == edges.len() {
            Ok(self.create_node(h, edges))
        } else {
            Err(format!("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, edges: &[Edge<E>]) -> Node<E> {
        if edges.iter().all(|x| &edges[0] == x) {
            return edges[0].node().clone()
        }
        
        let key = (h.id(), edges.iter().map(|x| (x.value(), x.node().id())).collect::<Vec<_>>().into_boxed_slice());
        match self.utable.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = Node::new_nonterminal(self.num_nodes, h, edges);
                self.num_nodes += 1;
                self.utable.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn omega(&self) -> Node<E> {
        self.omega.clone()
    }
    
    pub fn infinity(&self) -> Node<E> {
        self.infinity.clone()
    }

    pub fn min(&mut self, fv: E, f: &Node<E>, gv: E, g: &Node<E>) -> Edge<E> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::MIN, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+x.value(), x.node().clone()),
            None => {
                match (f, g) {
                    (Node::Infinity, Node::Infinity) => Edge::new(E::zero(), self.infinity()),
                    (Node::Infinity, _) => Edge::new(gv, g.clone()),
                    (_, Node::Infinity) => Edge::new(fv, f.clone()),
                    (Node::Omega, _) => Edge::new(mu, self.omega()),
                    (_, Node::Omega) => Edge::new(mu, self.omega()),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let edges = fnode.iter()
                            .map(|fedge| self.min(fv+fedge.value(), fedge.node(), gv, g)).collect::<Vec<_>>();
                        let edge = Edge::new(E::zero(), self.create_node(fnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let edges = gnode.iter()
                            .map(|gedge| self.min(fv, f, gv+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(E::zero(), self.create_node(gnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let edges = fnode.iter().zip(gnode.iter())
                            .map(|(fedge,gedge)| self.min(fv-mu+fedge.value(), fedge.node(), gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    _ => panic!("error"),
                }
            }
        }
    }
    
    pub fn max(&mut self, fv: E, f: &Node<E>, gv: E, g: &Node<E>) -> Edge<E> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::MAX, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+x.value(), x.node().clone()),
            None => {
                match (f, g) {
                    (Node::Infinity, _) => Edge::new(E::zero(), self.infinity()),
                    (_, Node::Infinity) => Edge::new(E::zero(), self.infinity()),
                    (Node::Omega, Node::Omega) => Edge::new(std::cmp::max(fv, gv), self.omega()),
                    (Node::Omega, Node::NonTerminal(_)) if fv <= gv => Edge::new(gv, g.clone()),
                    (Node::Omega, Node::NonTerminal(gnode)) if fv > gv => {
                        let edges = gnode.iter()
                            .map(|gedge| self.max(fv-mu, f, gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(_), Node::Omega) if fv >= gv => Edge::new(fv, f.clone()),
                    (Node::NonTerminal(fnode), Node::Omega) if fv < gv => {
                        let edges = fnode.iter()
                            .map(|fedge| self.max(fv-mu+fedge.value(), fedge.node(), gv-mu, g)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let edges = fnode.iter()
                            .map(|fedge| self.max(fv-mu+fedge.value(), fedge.node(), gv-mu, g)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let edges = gnode.iter()
                            .map(|gedge| self.max(fv-mu, f, gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let edges = fnode.iter().zip(gnode.iter())
                            .map(|(fedge,gedge)| self.max(fv-mu+fedge.value(), fedge.node(), gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    _ => panic!("error"),
                }
            }
        }
    }

    pub fn add(&mut self, fv: E, f: &Node<E>, gv: E, g: &Node<E>) -> Edge<E> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::ADD, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+mu+x.value(), x.node().clone()),
            None => {
                match (f, g) {
                    (Node::Infinity, _) => Edge::new(E::zero(), self.infinity()),
                    (_, Node::Infinity) => Edge::new(E::zero(), self.infinity()),
                    (Node::Omega, Node::Omega) => Edge::new(fv+gv, self.omega()),
                    (Node::Omega, Node::NonTerminal(_)) => Edge::new(fv+gv, g.clone()),
                    (Node::NonTerminal(_), Node::Omega) => Edge::new(fv+gv, f.clone()),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let edges = fnode.iter()
                            .map(|fedge| self.add(fv-mu+fedge.value(), fedge.node(), gv-mu, g)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(fnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let edges = gnode.iter()
                            .map(|gedge| self.add(fv-mu, f, gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let edges = fnode.iter().zip(gnode.iter())
                            .map(|(fedge,gedge)| self.add(fv-mu+fedge.value(), fedge.node(), gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(mu+mu, self.create_node(fnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    _ => panic!("error"),
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    pub fn rebuild(&mut self, fs: &[Node<E>]) {
        self.utable.clear();
        let mut visited = HashSet::new();
        for x in fs.iter() {
            self.rebuild_table(x, &mut visited);
        }
    }

    fn rebuild_table(&mut self, f: &Node<E>, visited: &mut HashSet<Node<E>>) {
        if visited.contains(f) {
            return
        }
        match f {
            Node::NonTerminal(fnode) => {
                let key = (fnode.header().id(), fnode.iter().map(|x| (x.value(), x.node().id())).collect::<Vec<_>>().into_boxed_slice());
                self.utable.insert(key, f.clone());
                for x in fnode.iter() {
                    self.rebuild_table(x.node(), visited);
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
    }
}

impl<E> Dot for Node<E> where E: EdgeValue {
    type Node = Node<E>;
    
    fn dot_impl<T>(&self, io: &mut T, visited: &mut HashSet<Self::Node>) where T: std::io::Write {
        if visited.contains(self) {
            return
        }
        match self {
            Node::Omega => {
                let s = format!("\"obj{}\" [shape=square, label=\"Omega\"];\n", self.id());
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id(), fnode.label());
                io.write(s.as_bytes()).unwrap();
                for (i,e) in fnode.iter().enumerate() {
                    e.node().dot_impl(io, visited);
                    if let Node::Omega | Node::NonTerminal(_) = e.node() {
                        let s = format!("\"obj{}:{}:{}\" [shape=diamond, label=\"{}\"];\n", fnode.id(), e.node().id(), e.value(), e.value());
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}\" -> \"obj{}:{}:{}\" [label=\"{}\", arrowhead=none];\n", fnode.id(), fnode.id(), e.node().id(), e.value(), i);
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}:{}:{}\" -> \"obj{}\";\n", fnode.id(), e.node().id(), e.value(), e.node().id());
                        io.write(s.as_bytes()).unwrap();
                    }
                }
            },
            _ => (),
        };
        visited.insert(self.clone());
    }
}

impl<E> Dot for Edge<E> where E: EdgeValue {
    type Node = Node<E>;
    
    fn dot_impl<T>(&self, io: &mut T, visited: &mut HashSet<Self::Node>) where T: std::io::Write {
        let f = self.node();
        if visited.contains(f) {
            return
        }
        match f {
            Node::Omega => {
                let s = format!("\"obj{}\" [shape=square, label=\"Omega\"];\n", f.id());
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id(), fnode.label());
                io.write(s.as_bytes()).unwrap();
                for (i,e) in fnode.iter().enumerate() {
                    e.dot_impl(io, visited);
                    if let Node::Omega | Node::NonTerminal(_) = e.node() {
                        let s = format!("\"obj{}:{}:{}\" [shape=diamond, label=\"{}\"];\n", fnode.id(), e.node().id(), e.value(), e.value());
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}\" -> \"obj{}:{}:{}\" [label=\"{}\", arrowhead=none];\n", fnode.id(), fnode.id(), e.node().id(), e.value(), i);
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}:{}:{}\" -> \"obj{}\";\n", fnode.id(), e.node().id(), e.value(), e.node().id());
                        io.write(s.as_bytes()).unwrap();
                    }
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;

    // impl Drop for Node {
    //     fn drop(&mut self) {
    //         println!("Dropping Node{}", self.id());
    //     }
    // }

    pub fn table<E>(dd: &EvMdd<E>, fv: E, f: &Node<E>) -> Vec<(Vec<usize>,Option<E>)> where E: EdgeValue {
        let mut tab = Vec::new();
        let p = Vec::new();
        table_(dd, f, &p, &mut tab, fv);
        tab
    }

    pub fn table_<E>(dd: &EvMdd<E>, f: &Node<E>, path: &[usize], tab: &mut Vec<(Vec<usize>,Option<E>)>, s: E) where E: EdgeValue {
        match f {
            Node::Infinity => {
                tab.push((path.to_vec(), None));
            },
            Node::Omega => {
                tab.push((path.to_vec(), Some(s)));
            },
            Node::NonTerminal(fnode) => {
                for (i,e) in fnode.iter().enumerate() {
                    let mut p = path.to_vec();
                    p.push(i);
                    table_(dd, e.node(), &p, tab, s + e.value());
                }
            },
            _ => (),
        };
    }

    #[test]
    fn new_header() {
        let h = NodeHeader::new(0, 0, "test", 2);
        println!("{:?}", h);
        println!("{:?}", h.level());
        let x = h.clone();
        println!("{:?}", x);
        println!("{:?}", x == h);
    }

    #[test]
    fn new_terminal() {
        let zero: Node<i32> = Node::Infinity;
        let one: Node<i32> = Node::Omega;
        println!("{:?}", zero);
        println!("{:?}", one);
    }

    #[test]
    fn new_nonterminal() {
        let zero: Node<i32> = Node::Infinity;
        let one: Node<i32> = Node::Omega;
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = Node::new_nonterminal(3, &h, &vec![Edge::new(1, zero), Edge::new(2, one)]);
        println!("{:?}", x);
        if let Node::NonTerminal(x) = &x {
            println!("{:?}", x.header());
        }
        println!("{:?}", x.header());
    }

    #[test]
    fn new_test1() {
        let mut dd: EvMdd = EvMdd::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &vec![Edge::new(1, dd.omega()), Edge::new(2, dd.omega())]);
        println!("{:?}", x);
        let y = dd.create_node(&h, &vec![Edge::new(1, dd.omega()), Edge::new(2, dd.omega())]);
        println!("{:?}", y);
        println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd: EvMdd = EvMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &vec![Edge::new(1, dd.omega()), Edge::new(2, dd.omega())]);
        let y = dd.create_node(&h2, &vec![Edge::new(1, dd.omega()), Edge::new(2, dd.omega())]);
        let z = dd.min(0, &x, 0, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }
    
    #[test]
    fn test_evmdd() {
        let mut dd: EvMdd = EvMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let f11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
        let f12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let f21 = dd.create_node(&h2, &vec![Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
        let f22 = dd.create_node(&h2, &vec![Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
        let f = dd.create_node(&h3, &vec![Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            f.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, 0, &f) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn new_test4() {
        let mut dd: EvMdd = EvMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let g11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
        let g12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let g21 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
        let g22 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
        let g = dd.create_node(&h3, &vec![Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            g.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, 0, &g) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn test_evmdd_min() {
        let mut dd: EvMdd = EvMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let f11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
        let f12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let f21 = dd.create_node(&h2, &vec![Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
        let f22 = dd.create_node(&h2, &vec![Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
        let f = dd.create_node(&h3, &vec![Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

        let g11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
        let g12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let g21 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
        let g22 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
        let g = dd.create_node(&h3, &vec![Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

        let z = dd.min(0, &f, 0, &g);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        println!("f");
        for x in table(&dd, 0, &f) {
            println!("{:?}", x);
        }

        println!("g");
        for x in table(&dd, 0, &g) {
            println!("{:?}", x);
        }

        println!("min(f,g)");
        for x in table(&dd, z.value(), z.node()) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn test_evmdd_max() {
        let mut dd: EvMdd = EvMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let f11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
        let f12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let f21 = dd.create_node(&h2, &vec![Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
        let f22 = dd.create_node(&h2, &vec![Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
        let f = dd.create_node(&h3, &vec![Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

        let g11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
        let g12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let g21 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
        let g22 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
        let g = dd.create_node(&h3, &vec![Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

        let z = dd.max(0, &f, 0, &g);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        println!("f");
        for x in table(&dd, 0, &f) {
            println!("{:?}", x);
        }

        println!("g");
        for x in table(&dd, 0, &g) {
            println!("{:?}", x);
        }

        println!("max(f,g)");
        for x in table(&dd, z.value(), &z.node()) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn test_evmdd_add() {
        let mut dd: EvMdd = EvMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let f11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
        let f12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let f21 = dd.create_node(&h2, &vec![Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
        let f22 = dd.create_node(&h2, &vec![Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
        let f = dd.create_node(&h3, &vec![Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

        let g11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
        let g12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let g21 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
        let g22 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
        let g = dd.create_node(&h3, &vec![Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

        let z = dd.add(0, &f, 0, &g);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        println!("f");
        for x in table(&dd, 0, &f) {
            println!("{:?}", x);
        }

        println!("g");
        for x in table(&dd, 0, &g) {
            println!("{:?}", x);
        }

        println!("f+g");
        for x in table(&dd, z.value(), &z.node()) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn test_dot() {
        let mut dd: EvMdd = EvMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let f11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(0, dd.infinity())]);
        let f12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let f21 = dd.create_node(&h2, &vec![Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
        let f22 = dd.create_node(&h2, &vec![Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
        let f = dd.create_node(&h3, &vec![Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

        let g11 = dd.create_node(&h1, &vec![Edge::new(0, dd.omega()), Edge::new(2, dd.omega())]);
        let g12 = dd.create_node(&h1, &vec![Edge::new(0, dd.infinity()), Edge::new(0, dd.omega())]);
        let g21 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(0, dd.infinity())]);
        let g22 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
        let g = dd.create_node(&h3, &vec![Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

        let z = dd.add(0, &f, 0, &g);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }
}
