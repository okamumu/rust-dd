use std::rc::Rc;
use std::hash::{Hash, Hasher};

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    TerminalBinaryValue,
    HashSet,
    HashMap,
};

use crate::nodes::{
    NodeHeader,
    Terminal,
    NonTerminal,
    TerminalBinary,
    NonTerminalBDD,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    NOT,
    AND,
    OR,
    XOR,
}

pub type Node<V> = BDDNode<V>;

#[derive(Debug,Clone)]
pub enum BDDNode<V> {
    NonTerminal(Rc<NonTerminalBDD<BDDNode<V>>>),
    Terminal(Rc<TerminalBinary<V>>),
}

impl<V> PartialEq for BDDNode<V> where V: TerminalBinaryValue {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<V> Eq for BDDNode<V> where V: TerminalBinaryValue {}

impl<V> Hash for BDDNode<V> where V: TerminalBinaryValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<V> BDDNode<V> where V: TerminalBinaryValue {
    pub fn new_nonterminal(id: NodeId, header: &NodeHeader, low: &Self, high: &Self) -> Self {
        let x = NonTerminalBDD::new(id, header.clone(), [low.clone(), high.clone()]);
        Self::NonTerminal(Rc::new(x))
    }

    pub fn new_terminal(id: NodeId, value: V) -> Self {
        let x = TerminalBinary::new(id, value);
        Self::Terminal(Rc::new(x))
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
pub struct BDD<V=u8> {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node<V>,
    one: Node<V>,
    utable: HashMap<(HeaderId, NodeId, NodeId), Node<V>>,
    cache: HashMap<(Operation, NodeId, NodeId), Node<V>>,
}

impl<V> BDD<V> where V: TerminalBinaryValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            zero: Node::new_terminal(0, V::low()),
            one: Node::new_terminal(1, V::high()),
            utable: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    pub fn new_with_type(_: V) -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            zero: Node::new_terminal(0, V::low()),
            one: Node::new_terminal(1, V::high()),
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
    
    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node<V>]) -> Result<Node<V>,String> {
        if nodes.len() == h.edge_num() {
            Ok(self.create_node(h, &nodes[0], &nodes[1]))
        } else {
            Err(format!("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, low: &Node<V>, high: &Node<V>) -> Node<V> {
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
    
    pub fn zero(&self) -> Node<V> {
        self.zero.clone()
    }
    
    pub fn one(&self) -> Node<V> {
        self.one.clone()
    }

    pub fn not(&mut self, f: &Node<V>) -> Node<V> {
        let key = (Operation::NOT, f.id(), 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    Node::Terminal(fnode) if fnode.value() == V::low() => self.one(),
                    Node::Terminal(fnode) if fnode.value() == V::high() => self.zero(),
                    Node::NonTerminal(fnode) => {
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

    pub fn and(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::AND, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => self.zero(),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => g.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => self.zero(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => f.clone(),
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
    
    pub fn or(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::OR, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => self.one(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => self.one(),
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

    pub fn xor(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::XOR, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => self.not(g),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => self.not(f),
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
    
    pub fn imp(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let tmp = self.not(f);
        self.or(&tmp, g)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    pub fn rebuild(&mut self, fs: &[Node<V>]) {
        self.utable.clear();
        let mut visited = HashSet::new();
        for x in fs.iter() {
            self.make_utable_(x, &mut visited);
        }
    }

    fn make_utable_(&mut self, f: &Node<V>, visited: &mut HashSet<Node<V>>) {
        if visited.contains(f) {
            return
        }
        match f {
            Node::NonTerminal(fnode) => {
                let key = (fnode.header().id(), fnode[0].id(), fnode[1].id());
                self.utable.insert(key, f.clone());
                for x in fnode.iter() {
                    self.make_utable_(&x, visited);
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
    }

    pub fn dot<U>(&self, io: &mut U, f: &Node<V>) where U: std::io::Write {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited = HashSet::new();
        io.write(s1.as_bytes()).unwrap();
        self.dot_(io, f, &mut visited);
        io.write(s2.as_bytes()).unwrap();
    }

    pub fn dot_<U>(&self, io: &mut U, f: &Node<V>, visited: &mut HashSet<Node<V>>) where U: std::io::Write {
        if visited.contains(f) {
            return
        }
        match f {
            Node::Terminal(fnode) => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id(), fnode.value());
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id(), fnode.label());
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.iter().enumerate() {
                    self.dot_(io, x, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.id(), x.id(), i);
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
            println!("{:?}", x.header());
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
