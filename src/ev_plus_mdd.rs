use std::rc::Rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};

// use std::collections::{HashMap, HashSet};
use hashbrown::{HashMap, HashSet};

type HeaderId = usize;
type NodeId = usize;
type OperationId = usize;
type Level = usize;
type EdgeValue = i64;

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
pub struct Edge {
    value: EdgeValue,
    node: Node,
}

impl Edge {
    pub fn new(value: EdgeValue, node: Node) -> Self {
        Self {
            value: value,
            node: node,
        }
    }
}

#[derive(Debug)]
pub struct NonTerminalNode {
    id: NodeId,
    header: NodeHeader,
    edges: Box<[Edge]>,
}

#[derive(Debug)]
pub struct TerminalNode {
    id: NodeId,
    value: bool,
}

#[derive(Debug,Clone)]
pub enum Node {
    NonTerminal(Rc<NonTerminalNode>),
    Terminal(Rc<TerminalNode>),
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().hash(state);
    }
}

impl Node {
    fn new_nonterminal(id: NodeId, header: &NodeHeader, edges: &[Edge]) -> Self {
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
    
    pub fn get_id(&self) -> NodeId {
        match self {
            Node::NonTerminal(x) => x.id,
            Node::Terminal(x) => x.id,
        }        
    }

    pub fn get_header(&self) -> Option<&NodeHeader> {
        match self {
            Node::NonTerminal(x) => Some(&x.header),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct EVMDD {
    num_headers: usize,
    num_nodes: usize,
    omega: Node,
    infinity: Node,
    utable: HashMap<(HeaderId, Box<[(EdgeValue,NodeId)]>), Node>,
    cache: HashMap<(OperationId, NodeId, NodeId, EdgeValue), Edge>,
}

const MIN: OperationId = 1;

impl EVMDD {
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

    pub fn size(&self) -> (usize, usize, usize) {
        (self.num_headers, self.num_nodes, self.utable.len())
    }
    
    pub fn header(&mut self, level: Level, label: &str, edge_num: usize) -> NodeHeader {
        let h = NodeHeader::new(self.num_headers, level, label, edge_num);
        self.num_headers += 1;
        h
    }
    
    pub fn node(&mut self, h: &NodeHeader, edges: &[Edge]) -> Result<Node,String> {
        if h.edge_num == edges.len() {
            Ok(self.create_node(h, edges))
        } else {
            Err(String::from("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, edges: &[Edge]) -> Node {
        if edges.iter().all(|x| &edges[0] == x) {
            return edges[0].node.clone()
        }
        
        let key = (h.id, edges.iter().map(|x| (x.value, x.node.get_id())).collect::<Vec<_>>().into_boxed_slice());
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
    
    pub fn get_omega(&self) -> Node {
        self.omega.clone()
    }
    
    pub fn get_infinity(&self) -> Node {
        self.infinity.clone()
    }

    pub fn min(&mut self, fv: EdgeValue, f: &Node, gv: EdgeValue, g: &Node) -> Edge {
        let mu = std::cmp::min(fv, gv);
        let key = (MIN, f.get_id(), g.get_id(), fv-gv);
        match self.cache.get(&key) {
            Some(x) => Edge::new(x.value + mu, x.node.clone()),
            None => {
                let edge = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value == false && gnode.value == false => Edge::new(0, self.get_infinity()),
                    (Node::Terminal(fnode), _) if fnode.value == false => Edge::new(gv, g.clone()),
                    (_, Node::Terminal(gnode)) if gnode.value == false => Edge::new(fv, f.clone()),
                    (Node::Terminal(fnode), _) if fnode.value == true => Edge::new(mu, self.get_omega()),
                    (_, Node::Terminal(gnode)) if gnode.value == true => Edge::new(mu, self.get_omega()),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let edges = fnode.edges.iter()
                            .map(|fedge| self.min(fv+fedge.value, &fedge.node, 0, g)).collect::<Vec<_>>();
                        Edge::new(0, self.create_node(&fnode.header, &edges))
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let edges = gnode.edges.iter()
                            .map(|gedge| self.min(0, f, gv+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        Edge::new(0, self.create_node(&gnode.header, &edges))
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let edges = fnode.edges.iter().zip(gnode.edges.iter())
                            .map(|(fedge,gedge)| self.min(fv-mu+fedge.value, &fedge.node, gv-mu+gedge.value, &gedge.node)).collect::<Vec<_>>();
                        Edge::new(mu, self.create_node(&fnode.header, &edges))
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, edge.clone());
                edge
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
    //             let key = (fnode.header.id, fnode.nodes.iter().map(|x| x.get_id()).collect::<Vec<_>>().into_boxed_slice());
    //             self.utable.insert(key, f.clone());
    //             for x in fnode.nodes.iter() {
    //                 self.make_utable_(&x, visited);
    //             }
    //         },
    //         _ => (),
    //     };
    //     visited.insert(f.clone());
    // }

    pub fn dot<T>(&self, io: &mut T, f: &Node) where T: std::io::Write {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited = HashSet::new();
        io.write(s1.as_bytes()).unwrap();
        self.dot_(io, f, &mut visited);
        io.write(s2.as_bytes()).unwrap();
    }

    pub fn dot2_<T>(&self, io: &mut T, f: &Node, visited: &mut HashSet<Node>) where T: std::io::Write {
        if visited.contains(f) {
            return
        }
        match f {
            Node::Terminal(fnode) if fnode.value == false => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id, "infinity");
                io.write(s.as_bytes()).unwrap();
            },
            Node::Terminal(fnode) if fnode.value == true => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id, "omega");
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id, fnode.header.label);
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.edges.iter().enumerate() {
                    self.dot_(io, &x.node, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}:{}\"];\n", fnode.id, x.node.get_id(), i, x.value);
                    io.write(s.as_bytes()).unwrap();
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
    }

    pub fn dot_<T>(&self, io: &mut T, f: &Node, visited: &mut HashSet<Node>) where T: std::io::Write {
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
                        let s = format!("\"obj{}:{}:{}\" [shape=diamond, label=\"{}\"];\n", fnode.id, e.node.get_id(), e.value, e.value);
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}\" -> \"obj{}:{}:{}\" [label=\"{}\", arrowhead=none];\n", fnode.id, fnode.id, e.node.get_id(), e.value, i);
                        io.write(s.as_bytes()).unwrap();
                        let s = format!("\"obj{}:{}:{}\" -> \"obj{}\";\n", fnode.id, e.node.get_id(), e.value, e.node.get_id());
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
    //         println!("Dropping Node{}", self.get_id());
    //     }
    // }

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
        let zero = Node::new_terminal(0, false);
        let one = Node::new_terminal(1, true);
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
        println!("{:?}", x.get_header());
    }

    #[test]
    fn new_test1() {
        let mut dd = EVMDD::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &vec![Edge::new(1, dd.get_omega()), Edge::new(2, dd.get_omega())]);
        println!("{:?}", x);
        let y = dd.create_node(&h, &vec![Edge::new(1, dd.get_omega()), Edge::new(2, dd.get_omega())]);
        println!("{:?}", y);
        println!("{:?}", Rc::strong_count(y.get_header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd = EVMDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &vec![Edge::new(1, dd.get_omega()), Edge::new(2, dd.get_omega())]);
        let y = dd.create_node(&h2, &vec![Edge::new(1, dd.get_omega()), Edge::new(2, dd.get_omega())]);
        let z = dd.min(0, &x, 0, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{:?}", Rc::strong_count(y.get_header().unwrap()));
    }
    
    #[test]
    fn new_test3() {
        let mut dd = EVMDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let f11 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_omega()), Edge::new(0, dd.get_infinity())]);
        let f12 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_infinity()), Edge::new(0, dd.get_omega())]);
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
    }

    #[test]
    fn new_test4() {
        let mut dd = EVMDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let g11 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_omega()), Edge::new(2, dd.get_omega())]);
        let g12 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_infinity()), Edge::new(0, dd.get_omega())]);
        let g21 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(0, dd.get_infinity())]);
        let g22 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(2, g12.clone())]);
        let g = dd.create_node(&h3, &vec![Edge::new(0, g21.clone()), Edge::new(2, g21.clone()), Edge::new(1, g22.clone())]);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            dd.dot(&mut io, &g);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }

    #[test]
    fn new_test5() {
        let mut dd = EVMDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let h3 = NodeHeader::new(2, 2, "z", 3);
        
        let f11 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_omega()), Edge::new(0, dd.get_infinity())]);
        let f12 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_infinity()), Edge::new(0, dd.get_omega())]);
        let f21 = dd.create_node(&h2, &vec![Edge::new(0, f11.clone()), Edge::new(2, f11.clone())]);
        let f22 = dd.create_node(&h2, &vec![Edge::new(1, f11.clone()), Edge::new(0, f12.clone())]);
        let f = dd.create_node(&h3, &vec![Edge::new(0, f21.clone()), Edge::new(1, f22.clone()), Edge::new(2, f22.clone())]);

        let g11 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_omega()), Edge::new(2, dd.get_omega())]);
        let g12 = dd.create_node(&h1, &vec![Edge::new(0, dd.get_infinity()), Edge::new(0, dd.get_omega())]);
        let g21 = dd.create_node(&h2, &vec![Edge::new(0, g11.clone()), Edge::new(0, dd.get_infinity())]);
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
    }

    // #[test]
    // fn new_test4() {
    //     let mut dd = MDD::new();
    //     let h1 = NodeHeader::new(0, 0, "x", 2);
    //     let h2 = NodeHeader::new(1, 1, "y", 2);
    //     let x = dd.create_node(&h1, &vec![dd.get_zero(), dd.get_one()]);
    //     let y = dd.create_node(&h2, &vec![dd.get_zero(), dd.get_one()]);
    //     let z = dd.or(&x, &y);

    //     let mut buf = vec![];
    //     {
    //         let mut io = BufWriter::new(&mut buf);
    //         dd.dot(&mut io, &z);
    //     }
    //     let s = std::str::from_utf8(&buf).unwrap();
    //     println!("{}", s);
    // }

    // #[test]
    // fn new_test5() {
    //     let mut dd = MDD::new();
    //     let h1 = NodeHeader::new(0, 0, "x", 2);
    //     let h2 = NodeHeader::new(1, 1, "y", 2);
    //     let x = dd.create_node(&h1, &vec![dd.get_zero(), dd.get_one()]);
    //     let y = dd.create_node(&h2, &vec![dd.get_zero(), dd.get_one()]);
    //     let z = dd.and(&x, &y);
    //     let z = dd.not(&z);

    //     let mut buf = vec![];
    //     {
    //         let mut io = BufWriter::new(&mut buf);
    //         dd.dot(&mut io, &z);
    //     }
    //     let s = std::str::from_utf8(&buf).unwrap();
    //     println!("{}", s);
    // }
}
