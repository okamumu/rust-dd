use std::rc::Rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};
use core::slice::Iter;

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
    EdgeValue,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    MIN,
    MAX,
    PLUS,
}

#[derive(Debug)]
pub struct NodeHeaderData {
    id: HeaderId,
    level: Level,
    label: String,
    edge_num: usize,
}

#[derive(Debug,Clone)]
pub struct NodeHeader(Rc<NodeHeaderData>);

impl Deref for NodeHeader {
    type Target = Rc<NodeHeaderData>;
    
    fn deref(&self) -> &Rc<NodeHeaderData> {
        &self.0
    }
}

impl PartialEq for NodeHeader {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for NodeHeader {}

impl Hash for NodeHeader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl NodeHeader {
    fn new(id: HeaderId, level: Level, label: &str, edge_num: usize) -> Self {
        let data = NodeHeaderData{
            id: id,
            level: level,
            label: label.to_string(),
            edge_num: edge_num,
        };
        Self(Rc::new(data))
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct Edge<T> where T: EdgeValue {
    value: T,
    node: Node<T>,
}

impl<T> Edge<T> where T: EdgeValue {
    pub fn new(value: T, node: Node<T>) -> Self {
        Self {
            value: value,
            node: node,
        }
    }

    pub fn value(&self) -> T {
        self.value
    }

    pub fn node(&self) -> Node<T> {
        self.node.clone()
    }
}

#[derive(Debug)]
pub struct NonTerminalNode<T> where T: EdgeValue {
    id: NodeId,
    header: NodeHeader,
    edges: Box<[Edge<T>]>,
}

impl<T> NonTerminalNode<T> where T: EdgeValue {
    pub fn edge_iter(&self) -> Iter<Edge<T>> {
        self.edges.iter()
    }
}

#[derive(Debug)]
pub struct TerminalNode {
    id: NodeId,
    value: bool,
}

#[derive(Debug,Clone)]
pub enum Node<T> where T: EdgeValue {
    NonTerminal(Rc<NonTerminalNode<T>>),
    Terminal(Rc<TerminalNode>),
}

impl<T> PartialEq for Node<T> where T: EdgeValue {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> Eq for Node<T> where T: EdgeValue {}

impl<T> Hash for Node<T> where T: EdgeValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<T> Node<T> where T: EdgeValue {
    fn new_nonterminal(id: NodeId, header: &NodeHeader, edges: &[Edge<T>]) -> Self {
        let x = NonTerminalNode {
            id: id,
            header: header.clone(),
            edges: edges.iter().map(|x| x.clone()).collect::<Vec<_>>().into_boxed_slice(),
        };
        Node::NonTerminal(Rc::new(x))
    }

    fn new_terminal(id: NodeId, value: bool) -> Self {
        let x = TerminalNode {
            id: id,
            value: value,
        };
        Node::Terminal(Rc::new(x))
    }
    
    pub fn id(&self) -> NodeId {
        match self {
            Node::NonTerminal(x) => x.id,
            Node::Terminal(x) => x.id,
        }        
    }

    pub fn header(&self) -> Option<&NodeHeader> {
        match self {
            Node::NonTerminal(x) => Some(&x.header),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct EVMDD<T> where T: EdgeValue {
    num_headers: HeaderId,
    num_nodes: NodeId,
    omega: Node<T>,
    infinity: Node<T>,
    utable: HashMap<(HeaderId, Box<[(T,NodeId)]>), Node<T>>,
    cache: HashMap<(Operation, NodeId, NodeId, T), Edge<T>>,
}

impl<T> EVMDD<T> where T: EdgeValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            infinity: Node::new_terminal(0, false),
            omega: Node::new_terminal(1, true),
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
    
    pub fn node(&mut self, h: &NodeHeader, edges: &[Edge<T>]) -> Result<Node<T>,String> {
        if h.edge_num == edges.len() {
            Ok(self.create_node(h, edges))
        } else {
            Err(format!("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, edges: &[Edge<T>]) -> Node<T> {
        if edges.iter().all(|x| &edges[0] == x) {
            return edges[0].node.clone()
        }
        
        let key = (h.id, edges.iter().map(|x| (x.value, x.node.id())).collect::<Vec<_>>().into_boxed_slice());
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
    
    pub fn omega(&self) -> Node<T> {
        self.omega.clone()
    }
    
    pub fn infinity(&self) -> Node<T> {
        self.infinity.clone()
    }

    pub fn min(&mut self, fv: T, f: &Node<T>, gv: T, g: &Node<T>) -> Edge<T> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::MIN, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+x.value, x.node.clone()),
            None => {
                match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value == false && gnode.value == false => Edge::new(T::zero(), self.infinity()),
                    (Node::Terminal(fnode), _) if fnode.value == false => Edge::new(gv, g.clone()),
                    (_, Node::Terminal(gnode)) if gnode.value == false => Edge::new(fv, f.clone()),
                    (Node::Terminal(fnode), _) if fnode.value == true => Edge::new(mu, self.omega()),
                    (_, Node::Terminal(gnode)) if gnode.value == true => Edge::new(mu, self.omega()),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let edges = fnode.edges.iter()
                            .map(|fedge| self.min(fv+fedge.value, &fedge.node, gv, g)).collect::<Vec<_>>();
                        let edge = Edge::new(T::zero(), self.create_node(&fnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let edges = gnode.edges.iter()
                            .map(|gedge| self.min(fv, f, gv+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        let edge = Edge::new(T::zero(), self.create_node(&gnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let edges = fnode.edges.iter().zip(gnode.edges.iter())
                            .map(|(fedge,gedge)| self.min(fv-mu+fedge.value, &fedge.node, gv-mu+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&fnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    _ => panic!("error"),
                }
            }
        }
    }
    
    pub fn max(&mut self, fv: T, f: &Node<T>, gv: T, g: &Node<T>) -> Edge<T> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::MAX, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+x.value, x.node.clone()),
            None => {
                match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == false => Edge::new(T::zero(), self.infinity()),
                    (_, Node::Terminal(gnode)) if gnode.value == false => Edge::new(T::zero(), self.infinity()),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value == true && gnode.value == true => Edge::new(std::cmp::max(fv, gv), self.omega()),
                    (Node::Terminal(fnode), Node::NonTerminal(_)) if fnode.value == true && fv <= gv => Edge::new(gv, g.clone()),
                    (Node::NonTerminal(_), Node::Terminal(gnode)) if gnode.value == true && fv >= gv => Edge::new(fv, f.clone()),
                    (Node::Terminal(fnode), Node::NonTerminal(gnode)) if fnode.value == true && fv > gv => {
                        let edges = gnode.edges.iter()
                            .map(|gedge| self.max(fv-mu, f, gv-mu+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&gnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(gnode)) if gnode.value == true && fv < gv => {
                        let edges = fnode.edges.iter()
                            .map(|fedge| self.max(fv-mu+fedge.value, &fedge.node, gv-mu, g)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&fnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let edges = fnode.edges.iter()
                            .map(|fedge| self.max(fv-mu+fedge.value, &fedge.node, gv-mu, g)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&fnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let edges = gnode.edges.iter()
                            .map(|gedge| self.max(fv-mu, f, gv-mu+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&gnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let edges = fnode.edges.iter().zip(gnode.edges.iter())
                            .map(|(fedge,gedge)| self.max(fv-mu+fedge.value, &fedge.node, gv-mu+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&fnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    _ => panic!("error"),
                }
            }
        }
    }

    pub fn plus(&mut self, fv: T, f: &Node<T>, gv: T, g: &Node<T>) -> Edge<T> {
        let mu = std::cmp::min(fv, gv);
        let key = (Operation::PLUS, f.id(), g.id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(mu+mu+x.value, x.node.clone()),
            None => {
                match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == false => Edge::new(T::zero(), self.infinity()),
                    (_, Node::Terminal(gnode)) if gnode.value == false => Edge::new(T::zero(), self.infinity()),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value == true && gnode.value == true => Edge::new(fv+gv, self.omega()),
                    (Node::Terminal(fnode), Node::NonTerminal(_)) if fnode.value == true => Edge::new(fv+gv, g.clone()),
                    (Node::NonTerminal(_), Node::Terminal(gnode)) if gnode.value == true => Edge::new(fv+gv, f.clone()),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let edges = fnode.edges.iter()
                            .map(|fedge| self.plus(fv-mu+fedge.value, &fedge.node, gv-mu, g)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&fnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let edges = gnode.edges.iter()
                            .map(|gedge| self.plus(fv-mu, f, gv-mu+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        let edge = Edge::new(mu, self.create_node(&gnode.header, &edges));
                        self.cache.insert(key, edge.clone());
                        edge
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let edges = fnode.edges.iter().zip(gnode.edges.iter())
                            .map(|(fedge,gedge)| self.plus(fv-mu+fedge.value, &fedge.node, gv-mu+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        let edge = Edge::new(mu+mu, self.create_node(&fnode.header, &edges));
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
    
    // pub fn make_utable(&mut self, f: &Node) {
    // self.utable.clear();
    //     let mut visited = HashSet::new();
    //     self.make_utable_(f, &mut visited);
    // }

    // fn make_utable_(&mut self, f: &Node, visited: &mut HashSet<Node>) {
    //     if visited.contains(f) {
    //         return
    //     }
    //     match f {
    //         Node::NonTerminal(fnode) => {
    //             let key = (fnode.header.id, fnode.nodes.iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
    //             self.utable.insert(key, f.clone());
    //             for x in fnode.nodes.iter() {
    //                 self.make_utable_(&x, visited);
    //             }
    //         },
    //         _ => (),
    //     };
    //     visited.insert(f.clone());
    // }

    pub fn dot<U>(&self, io: &mut U, f: &Node<T>) where U: std::io::Write {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited = HashSet::new();
        io.write(s1.as_bytes()).unwrap();
        self.dot_(io, f, &mut visited);
        io.write(s2.as_bytes()).unwrap();
    }

    pub fn dot_<U>(&self, io: &mut U, f: &Node<T>, visited: &mut HashSet<Node<T>>) where U: std::io::Write {
        if visited.contains(f) {
            return
        }
        match f {
            // Node::Terminal(fnode) if fnode.value == false => {
            //     let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id, "infinity");
            //     io.write(s.as_bytes()).unwrap();
            // },
            Node::Terminal(fnode) if fnode.value == true => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id, "omega");
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id, fnode.header.label);
                io.write(s.as_bytes()).unwrap();
                for (i,e) in fnode.edges.iter().enumerate() {
                    self.dot_(io, &e.node, visited);
                    if &e.node != &self.infinity {
                        let s = format!("\"obj{}:{}:{}\" [shape=diamond, label=\"{}\"];\n", fnode.id, e.node.id(), e.value, e.value);
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}\" -> \"obj{}:{}:{}\" [label=\"{}\", arrowhead=none];\n", fnode.id, fnode.id, e.node.id(), e.value, i);
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}:{}:{}\" -> \"obj{}\";\n", fnode.id, e.node.id(), e.value, e.node.id());
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

    pub fn table<T>(dd: &EVMDD<T>, fv: T, f: &Node<T>) -> Vec<(Vec<usize>,Option<T>)> where T: EdgeValue {
        let mut tab = Vec::new();
        let p = Vec::new();
        table_(dd, f, &p, &mut tab, fv);
        tab
    }

    pub fn table_<T>(dd: &EVMDD<T>, f: &Node<T>, path: &[usize], tab: &mut Vec<(Vec<usize>,Option<T>)>, s: T) where T: EdgeValue {
        match f {
            Node::Terminal(fnode) if fnode.value == false => {
                tab.push((path.to_vec(), None));
            },
            Node::Terminal(fnode) if fnode.value == true => {
                tab.push((path.to_vec(), Some(s)));
            },
            Node::NonTerminal(fnode) => {
                for (i,e) in fnode.edges.iter().enumerate() {
                    let mut p = path.to_vec();
                    p.push(i);
                    table_(dd, &e.node, &p, tab, s + e.value);
                }
            },
            _ => (),
        };
    }

    #[test]
    fn new_header() {
        let h = NodeHeader::new(0, 0, "test", 2);
        println!("{:?}", h);
        println!("{:?}", h.level);
        let x = h.clone();
        println!("{:?}", x);
        println!("{:?}", x == h);
    }

    #[test]
    fn new_terminal() {
        let zero = Node::<i32>::new_terminal(0, false);
        let one = Node::<i32>::new_terminal(1, true);
        println!("{:?}", zero);
        println!("{:?}", one);
    }

    #[test]
    fn new_nonterminal() {
        let zero = Node::new_terminal(0, false);
        let one = Node::new_terminal(1, true);
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = Node::new_nonterminal(3, &h, &vec![Edge::new(1, zero), Edge::new(2, one)]);
        println!("{:?}", x);
        if let Node::NonTerminal(x) = &x {
            println!("{:?}", x.header);
        }
        println!("{:?}", x.header());
    }

    #[test]
    fn new_test1() {
        let mut dd = EVMDD::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &vec![Edge::new(1, dd.omega()), Edge::new(2, dd.omega())]);
        println!("{:?}", x);
        let y = dd.create_node(&h, &vec![Edge::new(1, dd.omega()), Edge::new(2, dd.omega())]);
        println!("{:?}", y);
        println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd = EVMDD::new();
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
        let mut dd = EVMDD::new();
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
        let mut dd = EVMDD::new();
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
        let mut dd = EVMDD::new();
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
            dd.dot(&mut io, &z.node);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, z.value, &z.node) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn new_test6() {
        let mut dd = EVMDD::new();
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
            dd.dot(&mut io, &z.node);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, z.value, &z.node) {
            println!("{:?}", x);
        }
    }

    #[test]
    fn new_test_plus() {
        let mut dd = EVMDD::new();
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

        let z = dd.plus(0, &f, 0, &g);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            dd.dot(&mut io, &z.node);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        for x in table(&dd, z.value, &z.node) {
            println!("{:?}", x);
        }
    }

}
