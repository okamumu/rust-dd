/// BDD (Binary Decision Diagram) implementation.
/// 
/// Description:
/// 
/// A BDD is a rooted directed acyclic graph (DAG) with two terminal nodes, 0 and 1.
/// Each non-terminal node has a level and two edges, low and high.
/// The level is an integer that represents the variable of the node.
/// The low and high edges are the child nodes of the node.
/// 
/// The BDD has a unique table that stores the non-terminal nodes.
/// The table is a hash table that maps a tuple of (level, low, high) to a non-terminal node.
/// 
/// The BDD has a cache that stores the result of the operations.
/// The cache is a hash table that maps a tuple of (operation, f, g) to a node.
/// 
/// The BDD has the following operations:
/// - not(f): negation of f
/// - and(f, g): conjunction of f and g
/// - or(f, g): disjunction of f and g
/// - xor(f, g): exclusive or of f and g
/// - setdiff(f, g): set difference of f and g
/// - imp(f, g): implication of f and g
/// - nand(f, g): nand of f and g
/// - nor(f, g): nor of f and g
/// - xnor(f, g): exclusive nor of f and g
/// - ite(f, g, h): if-then-else of f, g, and h
/// 
/// The BDD has the following methods:
/// - header(level, label): create a new header
/// - node(header, nodes): create a new non-terminal node
/// - create_node(header, low, high): create a new non-terminal node
/// - zero(): return the terminal node 0
/// - one(): return the terminal node 1
/// - size(): return the number of headers, nodes, and the size of the unique table
/// 
/// The BDD has the following traits:
/// - Gc: garbage collection
/// - Count: count the number of edges
/// - Dot: output the graph in DOT format
/// 

use std::rc::Rc;
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
    NonTerminal,
    NonTerminalBDD,
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
    Setdiff,
}

type Node = BddNode;

#[derive(Debug,Clone)]
pub enum BddNode {
    NonTerminal(Rc<NonTerminalBDD<Node>>),
    Zero,
    One,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::NonTerminal(x), Node::NonTerminal(y)) => x.id() == y.id(),
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
        Self::NonTerminal(Rc::new(x))
    }

    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Zero => 0,
            Self::One => 1,
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
pub struct Bdd {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node,
    one: Node,
    utable: HashMap<(HeaderId, NodeId, NodeId), Node>,
    cache: HashMap<(Operation, NodeId, NodeId), Node>,
}

impl Default for Bdd {
    fn default() -> Self {
        Self::new()
    }
}

impl Bdd {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            zero: Node::Zero,
            one: Node::One,
            utable: HashMap::default(),
            cache: HashMap::default(),
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
    
    // pub fn node(&mut self, h: &NodeHeader, nodes: &[Node]) -> Result<Node,String> {
    //     if nodes.len() == h.edge_num() {
    //         Ok(self.create_node(h, &nodes[0], &nodes[1]))
    //     } else {
    //         Err("Did not match the number of edges in header and arguments.".to_string())
    //     }
    // }

    pub fn create_node(&mut self, h: &NodeHeader, low: &Node, high: &Node) -> Node {
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
        let key = (Operation::Not, f.id(), 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    Node::Zero => self.one(),
                    Node::One => self.zero(),
                    Node::NonTerminal(fnode) => {
                        let low = self.not(&fnode[0]);
                        let high = self.not(&fnode[1]);
                        self.create_node(fnode.header(), &low, &high)
                    },
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn and(&mut self, f: &Node, g: &Node) -> Node {
        let key = (Operation::And, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => self.zero(),
                    (Node::One, _) => g.clone(),
                    (_, Node::Zero) => self.zero(),
                    (_, Node::One) => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let low = self.and(&fnode[0], g);
                        let high = self.and(&fnode[1], g);
                        self.create_node(fnode.header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let low = self.and(f, &gnode[0]);
                        let high = self.and(f, &gnode[1]);
                        self.create_node(gnode.header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let low = self.and(&fnode[0], &gnode[0]);
                        let high = self.and(&fnode[1], &gnode[1]);
                        self.create_node(fnode.header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn and2(&mut self, f: &Node, g: &Node) {
        let mut tokens = vec![];
        let mut stack = vec![];
        let x = (f.clone(), g.clone());
        stack.push(x);
        while let Some((f, g)) = stack.pop() {
            match (&f, &g) {
                (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                    let x = (fnode[0].clone(), g.clone());
                    stack.push(x);
                    let x = (fnode[1].clone(), g.clone());
                    stack.push(x);
                },
                (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                    let key = (f.clone(), gnode[0].clone());
                    stack.push(key);
                    let key = (f.clone(), gnode[1].clone());
                    stack.push(key);
                },
                (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                    let key = (fnode[0].clone(), gnode[0].clone());
                    stack.push(key);
                    let key = (fnode[1].clone(), gnode[1].clone());
                    stack.push(key);
                },
                _ => (),
            };
            tokens.push((f, g));
        }
    }

    pub fn or(&mut self, f: &Node, g: &Node) -> Node {
        let key = (Operation::Or, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => g.clone(),
                    (Node::One, _) => self.one(),
                    (_, Node::Zero) => f.clone(),
                    (_, Node::One) => self.one(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let low = self.or(&fnode[0], g);
                        let high = self.or(&fnode[1], g);
                        self.create_node(fnode.header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let low = self.or(f, &gnode[0]);
                        let high = self.or(f, &gnode[1]);
                        self.create_node(gnode.header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let low = self.or(&fnode[0], &gnode[0]);
                        let high = self.or(&fnode[1], &gnode[1]);
                        self.create_node(fnode.header(), &low, &high)
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
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => self.zero(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let low = self.xor(&fnode[0], g);
                        let high = self.xor(&fnode[1], g);
                        self.create_node(fnode.header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let low = self.xor(f, &gnode[0]);
                        let high = self.xor(f, &gnode[1]);
                        self.create_node(gnode.header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let low = self.xor(&fnode[0], &gnode[0]);
                        let high = self.xor(&fnode[1], &gnode[1]);
                        self.create_node(fnode.header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn setdiff(&mut self, f: &Node, g: &Node) -> Node {
        let key = (Operation::Setdiff, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, _) => self.zero(),
                    (_, Node::Zero) => f.clone(),
                    (_, Node::One) => self.zero(),
                    (Node::One, _) => self.one(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => self.zero(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let low = self.setdiff(&fnode[0], g);
                        let high = self.setdiff(&fnode[1], g);
                        self.create_node(fnode.header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        self.setdiff(f, &gnode[0])
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let low = self.setdiff(&fnode[0], &gnode[0]);
                        let high = self.setdiff(&fnode[1], &gnode[1]);
                        self.create_node(fnode.header(), &low, &high)
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

    pub fn nand(&mut self, f: &Node, g: &Node) -> Node {
        let tmp = self.and(f, g);
        self.not(&tmp)
    }

    pub fn nor(&mut self, f: &Node, g: &Node) -> Node {
        let tmp = self.or(f, g);
        self.not(&tmp)
    }

    pub fn xnor(&mut self, f: &Node, g: &Node) -> Node {
        let tmp = self.xor(f, g);
        self.not(&tmp)
    }

    pub fn ite(&mut self, f: &Node, g: &Node, h: &Node) -> Node {
        let x1 = self.and(f, g);
        let barf = self.not(f);
        let x2 = self.and(&barf, h);
        self.or(&x1, &x2)
    }
}

impl Gc for Bdd {
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
        if let Node::NonTerminal(fnode) = f {
            let key = (fnode.header().id(), fnode[0].id(), fnode[1].id());
            self.utable.insert(key, f.clone());
            for x in fnode.iter() {
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
                        let tmp0 = fnode[0].count_edge_impl(visited);
                        let tmp1 = fnode[1].count_edge_impl(visited);
                        visited.insert(key);
                        tmp0 + tmp1 + 2
                    },
                    Node::One | Node::Zero => {
                        visited.insert(key);
                        0
                    },
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
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id(), fnode.label());
                io.write_all(s.as_bytes()).unwrap();
                for (i,x) in fnode.iter().enumerate() {
                    x.dot_impl(io, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.id(), x.id(), i);
                    io.write_all(s.as_bytes()).unwrap();
                }
            },
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
        let zero = Node::Zero;
        let one = Node::One;
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = Node::new_nonterminal(3, &h, &zero, &one);
        println!("{:?}", x);
        if let Node::NonTerminal(x) = &x {
            println!("{:?}", x.header());
        }
        // println!("{:?}", x.header());
    }

    #[test]
    fn new_test1() {
        let mut dd: Bdd = Bdd::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &dd.zero(), &dd.one());
        println!("{:?}", x);
        let y = dd.create_node(&h, &dd.zero(), &dd.one());
        println!("{:?}", y);
        // println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd: Bdd = Bdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.and(&x, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        // println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }
    
    #[test]
    fn new_test3() {
        let mut dd: Bdd = Bdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
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
        let mut dd: Bdd = Bdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
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
        let mut dd: Bdd = Bdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.or(&x, &y);
        let z = dd.not(&z);
        println!("{:?}", z.count());
        println!("{}", z.dot_string());
    }

    #[test]
    fn test_dot() {
        let mut dd: Bdd = Bdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
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

    #[test]
    fn test_setdiff() {
        let mut dd: Bdd = Bdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let _ = dd.setdiff(&x, &y);
    }
}
