use std::rc::Rc;
use std::hash::{Hash, Hasher};
use core::slice::Iter;

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
    NodeHeader,
    TerminalBin,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    NOT,
    AND,
    OR,
    XOR,
}

#[derive(Debug)]
pub struct NonTerminal<T> {
    id: NodeId,
    header: NodeHeader,
    nodes: [Node<T>; 2],
}

impl<T> NonTerminal<T> {
    pub fn node_iter(&self) -> Iter<Node<T>> {
        self.nodes.iter()
    }
}

#[derive(Debug)]
pub struct Terminal<T> {
    id: NodeId,
    value: T
}

#[derive(Debug,Clone)]
pub enum Node<T> {
    NonTerminal(Rc<NonTerminal<T>>),
    Terminal(Rc<Terminal<T>>),
}

impl<T> PartialEq for Node<T> where T: TerminalBin {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> Eq for Node<T> where T: TerminalBin {}

impl<T> Hash for Node<T> where T: TerminalBin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<T> Node<T> where T: TerminalBin {
    fn new_nonterminal(id: NodeId, header: &NodeHeader, low: &Node<T>, high: &Node<T>) -> Self {
        let x = NonTerminal {
            id: id,
            header: header.clone(),
            nodes: [low.clone(), high.clone()],
        };
        Node::NonTerminal(Rc::new(x))
    }

    fn new_terminal(id: NodeId, value: T) -> Self {
        let x = Terminal {
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
            _ => None
        }
    }

    pub fn level(&self) -> Option<Level> {
        self.header()
            .and_then(|x| Some(x.level()))
    }
}

#[derive(Debug)]
pub struct BDD<T=u8> {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node<T>,
    one: Node<T>,
    utable: HashMap<(HeaderId, NodeId, NodeId), Node<T>>,
    cache: HashMap<(Operation, NodeId, NodeId), Node<T>>,
}

impl<T> BDD<T> where T: TerminalBin {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            zero: Node::new_terminal(0, T::low()),
            one: Node::new_terminal(1, T::high()),
            utable: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    pub fn new_with_type(_: T) -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            zero: Node::new_terminal(0, T::low()),
            one: Node::new_terminal(1, T::high()),
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
    
    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node<T>]) -> Result<Node<T>,String> {
        if nodes.len() == h.edge_num() {
            Ok(self.create_node(h, &nodes[0], &nodes[1]))
        } else {
            Err(format!("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, low: &Node<T>, high: &Node<T>) -> Node<T> {
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
    
    pub fn zero(&self) -> Node<T> {
        self.zero.clone()
    }
    
    pub fn one(&self) -> Node<T> {
        self.one.clone()
    }

    pub fn not(&mut self, f: &Node<T>) -> Node<T> {
        let key = (Operation::NOT, f.id(), 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    Node::Terminal(fnode) if fnode.value == T::low() => self.one(),
                    Node::Terminal(fnode) if fnode.value == T::high() => self.zero(),
                    Node::NonTerminal(fnode) => {
                        let low = self.not(&fnode.nodes[0]);
                        let high = self.not(&fnode.nodes[1]);
                        self.create_node(&fnode.header, &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn and(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::AND, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == T::low() => self.zero(),
                    (Node::Terminal(fnode), _) if fnode.value == T::high() => g.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value == T::low() => self.zero(),
                    (_, Node::Terminal(gnode)) if gnode.value == T::high() => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() > gnode.header.level() => {
                        let low = self.and(&fnode.nodes[0], g);
                        let high = self.and(&fnode.nodes[1], g);
                        self.create_node(&fnode.header, &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() < gnode.header.level() => {
                        let low = self.and(f, &gnode.nodes[0]);
                        let high = self.and(f, &gnode.nodes[1]);
                        self.create_node(&gnode.header, &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() == gnode.header.level() => {
                        let low = self.and(&fnode.nodes[0], &gnode.nodes[0]);
                        let high = self.and(&fnode.nodes[1], &gnode.nodes[1]);
                        self.create_node(&fnode.header, &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn or(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::OR, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == T::low() => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value == T::high() => self.one(),
                    (_, Node::Terminal(gnode)) if gnode.value == T::low() => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value == T::high() => self.one(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() > gnode.header.level() => {
                        let low = self.or(&fnode.nodes[0], g);
                        let high = self.or(&fnode.nodes[1], g);
                        self.create_node(&fnode.header, &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() < gnode.header.level() => {
                        let low = self.or(f, &gnode.nodes[0]);
                        let high = self.or(f, &gnode.nodes[1]);
                        self.create_node(&gnode.header, &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() == gnode.header.level() => {
                        let low = self.or(&fnode.nodes[0], &gnode.nodes[0]);
                        let high = self.or(&fnode.nodes[1], &gnode.nodes[1]);
                        self.create_node(&fnode.header, &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn xor(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::XOR, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == T::low() => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value == T::high() => self.not(g),
                    (_, Node::Terminal(gnode)) if gnode.value == T::low() => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value == T::high() => self.not(f),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() > gnode.header.level() => {
                        let low = self.xor(&fnode.nodes[0], g);
                        let high = self.xor(&fnode.nodes[1], g);
                        self.create_node(&fnode.header, &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() < gnode.header.level() => {
                        let low = self.xor(f, &gnode.nodes[0]);
                        let high = self.xor(f, &gnode.nodes[1]);
                        self.create_node(&gnode.header, &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level() == gnode.header.level() => {
                        let low = self.xor(&fnode.nodes[0], &gnode.nodes[0]);
                        let high = self.xor(&fnode.nodes[1], &gnode.nodes[1]);
                        self.create_node(&fnode.header, &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn imp(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let tmp = self.not(f);
        self.or(&tmp, g)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    pub fn rebuild(&mut self, fs: &[Node<T>]) {
        self.utable.clear();
        let mut visited = HashSet::new();
        for x in fs.iter() {
            self.make_utable_(x, &mut visited);
        }
    }

    fn make_utable_(&mut self, f: &Node<T>, visited: &mut HashSet<Node<T>>) {
        if visited.contains(f) {
            return
        }
        match f {
            Node::NonTerminal(fnode) => {
                let key = (fnode.header.id(), fnode.nodes[0].id(), fnode.nodes[1].id());
                self.utable.insert(key, f.clone());
                for x in fnode.nodes.iter() {
                    self.make_utable_(&x, visited);
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
    }

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
            Node::Terminal(fnode) => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id, fnode.value);
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id, fnode.header.label());
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.nodes.iter().enumerate() {
                    self.dot_(io, x, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.id, x.id(), i);
                    io.write(s.as_bytes()).unwrap();
                }
            },
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
        let x = Node::new_nonterminal(3, &h, &zero, &one);
        println!("{:?}", x);
        if let Node::NonTerminal(x) = &x {
            println!("{:?}", x.header);
        }
        // println!("{:?}", x.header());
    }

    #[test]
    fn new_test1() {
        let mut dd: BDD = BDD::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &dd.zero(), &dd.one());
        println!("{:?}", x);
        let y = dd.create_node(&h, &dd.zero(), &dd.one());
        println!("{:?}", y);
        // println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd: BDD = BDD::new();
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
        let mut dd: BDD = BDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.and(&x, &y);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            dd.dot(&mut io, &z);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

    }

    #[test]
    fn new_test4() {
        let mut dd: BDD = BDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.or(&x, &y);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            dd.dot(&mut io, &z);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

    }

    #[test]
    fn new_test5() {
        let mut dd = BDD::new_with_type(true);
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.or(&x, &y);
        let z = dd.not(&z);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            dd.dot(&mut io, &z);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

    }
}
