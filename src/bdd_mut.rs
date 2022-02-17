use std::rc::Rc;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashSet,
    HashMap,
};

use crate::nodes::{
    NodeHeader,
    Terminal,
    NonTerminal,
    NonTerminalBDD,
};

use crate::dot::{
    Dot,
};

use crate::gc::{
    Gc,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    NOT,
    AND,
    OR,
    XOR,
}

type Node = BddMutNode;

#[derive(Debug,Clone)]
pub enum BddMutNode {
    NonTerminal(Rc<RefCell<NonTerminalBDD<Node>>>),
    Zero,
    One,
    None,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::NonTerminal(x), Node::NonTerminal(y)) => x.borrow().id() == y.borrow().id(),
            (Node::Zero, Node::Zero) => true,
            (Node::One, Node::One) => true,
            _ => false,
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
    pub fn new_nonterminal(id: NodeId, header: &NodeHeader, low: &Self, high: &Self) -> Self {
        let x = NonTerminalBDD::new(id, header.clone(), [low.clone(), high.clone()]);
        Self::NonTerminal(Rc::new(RefCell::new(x)))
    }

    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.borrow().id(),
            Self::Zero => 0,
            Self::One => 1,
            _ => panic!("Did not get NodeId."),
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
pub struct BddMut {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node,
    one: Node,
    utable: HashMap<(HeaderId, NodeId, NodeId), Node>,
    cache: HashMap<(Operation, NodeId, NodeId), Node>,
}

impl BddMut {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 3,
            zero: Node::Zero,
            one: Node::One,
            utable: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    pub fn size(&self) -> (HeaderId, NodeId, usize) {
        (self.num_headers, self.num_nodes, self.utable.len())
    }
    
    pub fn header(&mut self, level: Level, label: &str) -> NodeHeader {
        let h = NodeHeader::new(self.num_headers, level, label, 2);
        self.num_headers += 1;
        h
    }
    
    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node]) -> Result<Node,String> {
        if nodes.len() == h.edge_num() {
            Ok(self.create_node(h, &nodes[0], &nodes[1]))
        } else {
            Err(format!("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, low: &Node, high: &Node) -> Node {
        if low == high {
            return low.clone()
        }
        
        let key = (h.id(), low.id(), high.id());
        match self.utable.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = Node::new_nonterminal(self.num_nodes, h, low, high);
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
        let key = (Operation::NOT, f.id(), 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    Node::Zero => self.one(),
                    Node::One => self.zero(),
                    Node::NonTerminal(fx) => {
                        let fnode = fx.borrow();
                        let low = self.not(&fnode[0]);
                        let high = self.not(&fnode[1]);
                        self.create_node(fnode.header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn and(&mut self, f: &Node, g: &Node) -> Node {
        let key = (Operation::AND, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => self.zero(),
                    (Node::One, _) => g.clone(),
                    (_, Node::Zero) => self.zero(),
                    (_, Node::One) => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let low = self.and(&fnode.borrow()[0], g);
                        let high = self.and(&fnode.borrow()[1], g);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let low = self.and(f, &gnode.borrow()[0]);
                        let high = self.and(f, &gnode.borrow()[1]);
                        self.create_node(gnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let low = self.and(&fnode.borrow()[0], &gnode.borrow()[0]);
                        let high = self.and(&fnode.borrow()[1], &gnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn or(&mut self, f: &Node, g: &Node) -> Node {
        let key = (Operation::OR, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => g.clone(),
                    (Node::One, _) => self.one(),
                    (_, Node::Zero) => f.clone(),
                    (_, Node::One) => self.one(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let low = self.or(&fnode.borrow()[0], g);
                        let high = self.or(&fnode.borrow()[1], g);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let low = self.or(f, &gnode.borrow()[0]);
                        let high = self.or(f, &gnode.borrow()[1]);
                        self.create_node(gnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let low = self.or(&fnode.borrow()[0], &gnode.borrow()[0]);
                        let high = self.or(&fnode.borrow()[1], &gnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn xor(&mut self, f: &Node, g: &Node) -> Node {
        let key = (Operation::XOR, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => g.clone(),
                    (Node::One, _) => self.not(g),
                    (_, Node::Zero) => f.clone(),
                    (_, Node::One) => self.not(f),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let low = self.xor(&fnode.borrow()[0], g);
                        let high = self.xor(&fnode.borrow()[1], g);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let low = self.xor(f, &gnode.borrow()[0]);
                        let high = self.xor(f, &gnode.borrow()[1]);
                        self.create_node(gnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let low = self.xor(&fnode.borrow()[0], &gnode.borrow()[0]);
                        let high = self.xor(&fnode.borrow()[1], &gnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn imp(&mut self, f: &Node, g: &Node) -> Node {
        let tmp = self.not(f);
        self.or(&tmp, g)
    }
}

impl Gc for BddMut {
    type Node = Node;

    fn clear_cache(&mut self) {
        self.cache.clear();
    }

    fn clear_table(&mut self) {
        self.utable.clear();
    }
    
    fn gc_impl(&mut self, f: &Self::Node, visited: &mut HashSet<Self::Node>) {
        if visited.contains(f) {
            return
        }
        match f {
            Node::NonTerminal(fnode) => {
                let key = (fnode.borrow().header().id(), fnode.borrow()[0].id(), fnode.borrow()[1].id());
                self.utable.insert(key, f.clone());
                for x in fnode.borrow().iter() {
                    self.gc_impl(&x, visited);
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
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
                io.write(s.as_bytes()).unwrap();
            },
            Node::One => {
                let s = format!("\"obj{}\" [shape=square, label=\"1\"];\n", self.id());
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.borrow().id(), fnode.borrow().label());
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.borrow().iter().enumerate() {
                    x.dot_impl(io, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.borrow().id(), x.id(), i);
                    io.write(s.as_bytes()).unwrap();
                }
            },
            _ => (),
        };
        visited.insert(self.clone());
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
        let zero = Node::Zero;
        let one = Node::One;
        println!("{:?}", zero);
        println!("{:?}", one);
    }

    #[test]
    fn new_nonterminal() {
        let none = Node::None;
        let zero = Node::Zero;
        let one = Node::One;
        let h = NodeHeader::new(0, 0, "x", 2);
        let x: Node = Node::new_nonterminal(3, &h, &none, &none);
        println!("{:?}", x);
        if let Node::NonTerminal(v) = &x {
            v.borrow_mut()[0] = zero.clone();
            v.borrow_mut()[1] = one.clone();
        }
        println!("{:?}", x);
    }

    #[test]
    fn test_dot() {
        let mut dd = BddMut::new();
        let h1 = dd.header(0, "x");
        let h2 = dd.header(1, "y");
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.or(&x, &y);
        let z = dd.not(&z);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

    }
}
