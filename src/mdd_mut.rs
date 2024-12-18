use std::rc::Rc;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
};

use crate::nodes::{
    NodeHeader,
    NonTerminal,
    NonTerminalMDD,
};

use crate::dot::Dot;
use crate::count::Count;
use crate::gc::Gc;

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    Not,
    And,
    Or,
    XOr,
}

type Node = MddMutNode;

#[derive(Debug,Clone)]
pub enum MddMutNode {
    NonTerminal(Rc<RefCell<NonTerminalMDD<Node>>>),
    Zero,
    One,
    None,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NonTerminal(x), Self::NonTerminal(y)) => x.borrow().id() == y.borrow().id(),
            (Self::Zero, Self::Zero) => true,
            (Self::One, Self::One) => true,
            _ => false
        }
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Node {
    pub fn new_nonterminal(id: NodeId, header: &NodeHeader, nodes: &[Self]) -> Self {
        let x = NonTerminalMDD::new(
            id,
            header.clone(),
            nodes.to_vec().into_boxed_slice(),
        );
        Self::NonTerminal(Rc::new(RefCell::new(x)))
    }
    
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.borrow().id(),
            Self::Zero => 0,
            Self::One => 1,
            Self::None => 2,
        }        
    }

    pub fn header(&self) -> Option<NodeHeader> {
        match self {
            Self::NonTerminal(x) => Some(x.borrow().header().clone()),
            _ => None
        }
    }

    pub fn level(&self) -> Option<Level> {
        match self {
            Self::NonTerminal(x) => Some(x.borrow().level()),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct MddMut {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node,
    one: Node,
    utable: HashMap<(HeaderId, Box<[NodeId]>), Node>,
    cache: HashMap<(Operation, NodeId, NodeId), Node>,
}

impl Default for MddMut {
    fn default() -> Self {
        Self::new()
    }
}

impl MddMut {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 3,
            zero: Node::Zero,
            one: Node::One,
            utable: HashMap::default(),
            cache: HashMap::default(),
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
    
    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node]) -> Result<Node,String> {
        if h.edge_num() == nodes.len() {
            Ok(self.create_node(h, nodes))
        } else {
            Err(String::from("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, nodes: &[Node]) -> Node {
        if nodes.iter().all(|x| &nodes[0] == x) {
            return nodes[0].clone()
        }
        
        let key = (h.id(), nodes.iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
        match self.utable.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = Node::new_nonterminal(self.num_nodes, h, nodes);
                self.num_nodes += 1;
                self.utable.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn zero(&self) -> Node {
        self.zero.clone()
    }
    
    pub fn one(&self) -> Node {
        self.one.clone()
    }

    pub fn not(&mut self, f: &Node) -> Node {
        let key = (Operation::Not, f.id(), 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    Node::Zero => self.one(),
                    Node::One => self.zero(),
                    Node::NonTerminal(fnode) => {
                        let nodes = fnode.borrow().iter().map(|f| self.not(f)).collect::<Vec<_>>();
                        self.create_node(fnode.borrow().header(), &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn and(&mut self, f: &Node, g: &Node) -> Node {
        // if f == g {
        //     return f.clone()
        // }
        let key = (Operation::And, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => self.zero(),
                    (Node::One, _) => g.clone(),
                    (_, Node::Zero) => self.zero(),
                    (_, Node::One) => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let nodes = fnode.borrow().iter().map(|f| self.and(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.borrow().header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let nodes = gnode.borrow().iter().map(|g| self.and(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.borrow().header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let nodes = fnode.borrow().iter().zip(gnode.borrow().iter()).map(|(f,g)| self.and(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.borrow().header(), &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn or(&mut self, f: &Node, g: &Node) -> Node {
        // if f == g {
        //     return f.clone()
        // }
        let key = (Operation::Or, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => g.clone(),
                    (Node::One, _) => self.one(),
                    (_, Node::Zero) => f.clone(),
                    (_, Node::One) => self.one(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let nodes = fnode.borrow().iter().map(|f| self.or(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.borrow().header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let nodes = gnode.borrow().iter().map(|g| self.or(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.borrow().header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let nodes = fnode.borrow().iter().zip(gnode.borrow().iter()).map(|(f,g)| self.or(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.borrow().header(), &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn xor(&mut self, f: &Node, g: &Node) -> Node {
        let key = (Operation::XOr, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => g.clone(),
                    (Node::One, _) => self.not(g),
                    (_, Node::Zero) => f.clone(),
                    (_, Node::One) => self.not(f),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let nodes = fnode.borrow().iter().map(|f| self.xor(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.borrow().header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let nodes = gnode.borrow().iter().map(|g| self.xor(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.borrow().header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let nodes = fnode.borrow().iter().zip(gnode.borrow().iter()).map(|(f,g)| self.xor(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.borrow().header(), &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
}

impl Gc for MddMut {
    type Node = Node;

    fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    fn clear_table(&mut self) {
        self.utable.clear();
        self.num_nodes = 3;
    }
    
    fn gc_impl(&mut self, f: &Self::Node, visited: &mut HashSet<Self::Node>) {
        if visited.contains(f) {
            return
        }
        if let Node::NonTerminal(fnode) = f {
            fnode.borrow_mut().set_id(self.num_nodes);
            self.num_nodes += 1;
            let key = (fnode.borrow().header().id(), fnode.borrow().iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
            self.utable.insert(key, f.clone());
            for x in fnode.borrow().iter() {
                self.gc_impl(x, visited);
            }
        }
        visited.insert(f.clone());
    }
}

impl Count for Node {
    type NodeId = NodeId;
    type T = u64;

    fn count_edge_impl(&self, visited: &mut HashSet<NodeId>) -> Self::T {
        let key = self.id();
        match visited.get(&key) {
            Some(_) => 0,
            None => {
                match self {
                    Node::NonTerminal(fnode) => {
                        let mut sum = 0;
                        for x in fnode.borrow().iter() {
                            let tmp = x.count_edge_impl(visited);
                            sum += tmp + 1;
                        }
                        visited.insert(key);
                        sum
                    },
                    Node::One | Node::Zero => {
                        visited.insert(key);
                        0
                    },
                    Node::None => panic!("An edge is not connected to any node.")
                }
            }
        }
    }
}

impl Dot for Node {
    type Node = Node;

    fn dot_impl<T>(&self, io: &mut T, visited: &mut HashSet<Self::Node>) where T: std::io::Write {
        if visited.contains(self) {
            return
        }
        match self {
            Node::Zero => {
                let s = format!("\"obj{}\" [shape=square, label=\"0\"];\n", self.id());
                io.write_all(s.as_bytes()).unwrap();
            },
            Node::One => {
                let s = format!("\"obj{}\" [shape=square, label=\"1\"];\n", self.id());
                io.write_all(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.borrow().id(), fnode.borrow().label());
                io.write_all(s.as_bytes()).unwrap();
                for (i,x) in fnode.borrow().iter().enumerate() {
                    x.dot_impl(io, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.borrow().id(), x.id(), i);
                    io.write_all(s.as_bytes()).unwrap();
                }
            },
            Node::None => {
                let s = format!("\"obj{}\" [shape=square, label=\"none\"];\n", self.id());
                io.write_all(s.as_bytes()).unwrap();
            },
        };
        visited.insert(self.clone());
    }
}

// impl Mdd {
//     pub fn build_from_pathset<P>(&mut self, headers: &[NodeHeader], pathset: P) {
//         for x in P {
//             for i in domain[level] {
//                 x[level]
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;

    // impl Drop for Node {
    //     fn drop(&mut self) {
    //         println!("Dropping Node{}", self.id());
    //     }
    // }

    #[test]
    fn new_test1() {
        let mut dd: MddMut = MddMut::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &[dd.zero(), dd.one()]);
        println!("{:?}", x);
        let y = dd.create_node(&h, &[dd.zero(), dd.one()]);
        println!("{:?}", y);
        // println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd: MddMut = MddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &[dd.zero(), dd.one()]);
        let y = dd.create_node(&h2, &[dd.zero(), dd.one()]);
        let z = dd.and(&x, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        // println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }
    
    #[test]
    fn new_test3() {
        let mut dd: MddMut = MddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &[dd.zero(), dd.one()]);
        let y = dd.create_node(&h2, &[dd.zero(), dd.one()]);
        let z = dd.and(&x, &y);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }

    #[test]
    fn new_test4() {
        let mut dd: MddMut = MddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &[dd.zero(), dd.one()]);
        let y = dd.create_node(&h2, &[dd.zero(), dd.one()]);
        let z = dd.or(&x, &y);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }

    #[test]
    fn new_test5() {
        let mut dd: MddMut = MddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &[dd.zero(), dd.one()]);
        let y = dd.create_node(&h2, &[dd.zero(), dd.one()]);
        let z = dd.and(&x, &y);
        let z = dd.not(&z);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_dot() {
        let mut dd: MddMut = MddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &[dd.zero(), dd.one()]);
        let y = dd.create_node(&h2, &[dd.zero(), dd.one()]);
        let z = dd.and(&x, &y);
        let z = dd.not(&z);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }

    // #[test]
    // fn test_mdd_pset() {
    //     let mut dd: Mdd = Mdd::new();
    //     let h1 = NodeHeader::new(0, 0, "x", 2);
    //     let h2 = NodeHeader::new(1, 1, "y", 2);
    //     let pset = vec![
    //         vec![0,0],
    //         vec![1,0],
    //     ];

    //     let b = dd.from_pset(&vec![h1, h2], pset);

    //     let mut buf = vec![];
    //     {
    //         let mut io = BufWriter::new(&mut buf);
    //         dd.dot(&mut io, &b);
    //     }
    //     let s = std::str::from_utf8(&buf).unwrap();
    //     println!("{}", s);
    // }
}
