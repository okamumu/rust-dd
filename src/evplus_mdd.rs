use std::rc::Rc;
use std::hash::{Hash, Hasher};

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
    EdgeValue,
    TerminalBinaryValue,
};

use crate::nodes::{
    NodeHeader,
    Terminal,
    NonTerminal,
    TerminalBinary,
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

type Node<E,V> = EvMddNode<E,V>;
type Edge<E,V> = EvEdge<E,Node<E,V>>;

#[derive(Debug,Clone)]
pub enum EvMddNode<E,V> {
    NonTerminal(Rc<NonTerminalMDD<EvEdge<E,Node<E,V>>>>),
    Terminal(Rc<TerminalBinary<V>>),
}

impl<E,V> PartialEq for Node<E,V> where E: EdgeValue, V: TerminalBinaryValue {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<E,V> Eq for Node<E,V> where E: EdgeValue, V: TerminalBinaryValue {}

impl<E,V> Hash for Node<E,V> where E: EdgeValue, V: TerminalBinaryValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<E,V> Node<E,V> where E: EdgeValue, V: TerminalBinaryValue {
    fn new_nonterminal(id: NodeId, header: &NodeHeader, edges: &[Edge<E,V>]) -> Self {
        let x = NonTerminalMDD::new(
            id,
            header.clone(),
            edges.iter().map(|x| x.clone()).collect::<Vec<_>>().into_boxed_slice(),
        );
        Node::NonTerminal(Rc::new(x))
    }

    fn new_terminal(id: NodeId, value: V) -> Self {
        let x = TerminalBinary::new(id, value);
        Node::Terminal(Rc::new(x))
    }
    
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Terminal(x) => x.id(),
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
pub struct EvMdd<E = i64, V = u8> where E: EdgeValue, V: TerminalBinaryValue {
    num_headers: HeaderId,
    num_nodes: NodeId,
    omega: Node<E,V>,
    infinity: Node<E,V>,
    utable: HashMap<(HeaderId, Box<[(E,NodeId)]>), Node<E,V>>,
    cache: HashMap<(Operation, NodeId, NodeId, E), Edge<E,V>>,
}

impl<E,V> EvMdd<E,V> where E: EdgeValue, V: TerminalBinaryValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            infinity: Node::new_terminal(0, V::low()),
            omega: Node::new_terminal(1, V::high()),
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
    
    pub fn node(&mut self, h: &NodeHeader, edges: &[Edge<E,V>]) -> Result<Node<E,V>,String> {
        if h.edge_num() == edges.len() {
            Ok(self.create_node(h, edges))
        } else {
            Err(format!("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, edges: &[Edge<E,V>]) -> Node<E,V> {
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
    
    pub fn omega(&self) -> Node<E,V> {
        self.omega.clone()
    }
    
    pub fn infinity(&self) -> Node<E,V> {
        self.infinity.clone()
    }

    pub fn min(&mut self, fv: E, f: &Node<E,V>, gv: E, g: &Node<E,V>) -> Edge<E,V> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::MIN, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+x.value(), x.node().clone()),
            None => {
                match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() == V::low() && gnode.value() == V::low() => Edge::new(E::zero(), self.infinity()),
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => Edge::new(gv, g.clone()),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => Edge::new(fv, f.clone()),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => Edge::new(mu, self.omega()),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => Edge::new(mu, self.omega()),
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
    
    pub fn max(&mut self, fv: E, f: &Node<E,V>, gv: E, g: &Node<E,V>) -> Edge<E,V> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::MAX, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+x.value(), x.node().clone()),
            None => {
                match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => Edge::new(E::zero(), self.infinity()),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => Edge::new(E::zero(), self.infinity()),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() == V::high() && gnode.value() == V::high() => Edge::new(std::cmp::max(fv, gv), self.omega()),
                    (Node::Terminal(fnode), Node::NonTerminal(_)) if fnode.value() == V::high() && fv <= gv => Edge::new(gv, g.clone()),
                    (Node::NonTerminal(_), Node::Terminal(gnode)) if gnode.value() == V::high() && fv >= gv => Edge::new(fv, f.clone()),
                    (Node::Terminal(fnode), Node::NonTerminal(gnode)) if fnode.value() == V::high() && fv > gv => {
                        let edges = gnode.iter()
                            .map(|gedge| self.max(fv-mu, f, gv-mu+gedge.value(), gedge.node())).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(gnode.header(), &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(gnode)) if gnode.value() == V::high() && fv < gv => {
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

    pub fn add(&mut self, fv: E, f: &Node<E,V>, gv: E, g: &Node<E,V>) -> Edge<E,V> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::ADD, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+mu+x.value(), x.node().clone()),
            None => {
                match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => Edge::new(E::zero(), self.infinity()),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => Edge::new(E::zero(), self.infinity()),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() == V::high() && gnode.value() == V::high() => Edge::new(fv+gv, self.omega()),
                    (Node::Terminal(fnode), Node::NonTerminal(_)) if fnode.value() == V::high() => Edge::new(fv+gv, g.clone()),
                    (Node::NonTerminal(_), Node::Terminal(gnode)) if gnode.value() == V::high() => Edge::new(fv+gv, f.clone()),
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
    
    pub fn rebuild(&mut self, fs: &[Node<E,V>]) {
        self.utable.clear();
        let mut visited = HashSet::new();
        for x in fs.iter() {
            self.rebuild_table(x, &mut visited);
        }
    }

    fn rebuild_table(&mut self, f: &Node<E,V>, visited: &mut HashSet<Node<E,V>>) {
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

impl<E,V> Dot for EvMdd<E,V> where E: EdgeValue, V: TerminalBinaryValue {
    type Node = Node<E,V>;
    
    fn dot_impl<T>(&self, io: &mut T, f: &Self::Node, visited: &mut HashSet<Self::Node>) where T: std::io::Write {
        if visited.contains(f) {
            return
        }
        match f {
            Node::Terminal(fnode) if fnode.value() == V::high() => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id(), fnode.value());
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id(), fnode.label());
                io.write(s.as_bytes()).unwrap();
                for (i,e) in fnode.iter().enumerate() {
                    self.dot_impl(io, e.node(), visited);
                    if e.node() != &self.infinity {
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

    pub fn table<E,V>(dd: &EvMdd<E,V>, fv: E, f: &Node<E,V>) -> Vec<(Vec<usize>,Option<E>)> where E: EdgeValue, V: TerminalBinaryValue {
        let mut tab = Vec::new();
        let p = Vec::new();
        table_(dd, f, &p, &mut tab, fv);
        tab
    }

    pub fn table_<E,V>(dd: &EvMdd<E,V>, f: &Node<E,V>, path: &[usize], tab: &mut Vec<(Vec<usize>,Option<E>)>, s: E) where E: EdgeValue, V: TerminalBinaryValue {
        match f {
            Node::Terminal(fnode) if fnode.value() == V::low() => {
                tab.push((path.to_vec(), None));
            },
            Node::Terminal(fnode) if fnode.value() == V::high() => {
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
        let zero = Node::<i32,_>::new_terminal(0, false);
        let one = Node::<i32,_>::new_terminal(1, true);
        println!("{:?}", zero);
        println!("{:?}", one);
    }

    #[test]
    fn new_nonterminal() {
        let zero: Node<i32,_> = Node::new_terminal(0, false);
        let one: Node<i32,_> = Node::new_terminal(1, true);
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
    fn new_test3() {
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
            dd.dot(&mut io, &f);
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
            dd.dot(&mut io, &g);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, 0, &g) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn new_test5() {
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
            dd.dot(&mut io, z.node());
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, z.value(), z.node()) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn new_test6() {
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
            dd.dot(&mut io, &z.node());
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, z.value(), &z.node()) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn new_test_add() {
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
            dd.dot(&mut io, &z.node());
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, z.value(), &z.node()) {
            println!("{:?}", x);
        }
    }

}
