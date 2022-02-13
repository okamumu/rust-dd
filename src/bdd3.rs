use std::rc::Rc;
use std::cell::RefCell;
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

type Node<V> = BDD3Node<V>;

#[derive(Debug,Clone)]
pub enum BDD3Node<V> {
    NonTerminal(Rc<RefCell<NonTerminalBDD<BDD3Node<V>>>>),
    Terminal(Rc<TerminalBinary<V>>),
    None,
}

impl<V> PartialEq for BDD3Node<V> where V: TerminalBinaryValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::NonTerminal(x), Node::NonTerminal(y)) => x.borrow().id() == y.borrow().id(),
            (Node::Terminal(x), Node::Terminal(y)) => x.value() == y.value(),
            _ => false,
        }
    }
}

impl<V> Eq for BDD3Node<V> where V: TerminalBinaryValue {}

impl<V> Hash for BDD3Node<V> where V: TerminalBinaryValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<V> BDD3Node<V> where V: TerminalBinaryValue {
    pub fn new_nonterminal(id: NodeId, header: &NodeHeader, low: &Self, high: &Self) -> Self {
        let x = NonTerminalBDD::new(id, header.clone(), [low.clone(), high.clone()]);
        Self::NonTerminal(Rc::new(RefCell::new(x)))
    }

    pub fn new_terminal(id: NodeId, value: V) -> Self {
        let x = TerminalBinary::new(id, value);
        Self::Terminal(Rc::new(x))
    }
    
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.borrow().id(),
            Self::Terminal(x) => x.id(),
            _ => panic!(),
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
pub struct BDD3<V=u8> {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node<V>,
    one: Node<V>,
    utable: HashMap<(HeaderId, NodeId, NodeId), Node<V>>,
    cache: HashMap<(Operation, NodeId, NodeId), Node<V>>,
}

impl<V> BDD3<V> where V: TerminalBinaryValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 3,
            zero: Node::new_terminal(1, V::low()),
            one: Node::new_terminal(2, V::high()),
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
            self.rebuild_table(x, &mut visited);
        }
    }

    fn rebuild_table(&mut self, f: &Node<V>, visited: &mut HashSet<Node<V>>) {
        if visited.contains(f) {
            return
        }
        match f {
            Node::NonTerminal(fnode) => {
                let key = (fnode.borrow().header().id(), fnode.borrow()[0].id(), fnode.borrow()[1].id());
                self.utable.insert(key, f.clone());
                for x in fnode.borrow().iter() {
                    self.rebuild_table(&x, visited);
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
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.borrow().id(), fnode.borrow().label());
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.borrow().iter().enumerate() {
                    self.dot_(io, x, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.borrow().id(), x.id(), i);
                    io.write(s.as_bytes()).unwrap();
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
        let none = Node::None;
        let zero = Node::new_terminal(1, false);
        let one = Node::new_terminal(2, true);
        let h = NodeHeader::new(0, 0, "x", 2);
        let x: BDD3Node<bool> = Node::new_nonterminal(3, &h, &none, &none);
        println!("{:?}", x);
        if let Node::NonTerminal(v) = &x {
            v.borrow_mut()[0] = zero.clone();
            v.borrow_mut()[1] = one.clone();
        }
        println!("{:?}", x);
    }

}
