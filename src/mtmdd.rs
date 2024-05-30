use std::rc::Rc;
use std::hash::{Hash, Hasher};

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
    TerminalNumberValue,
};

use crate::nodes::{
    NodeHeader,
    Terminal,
    NonTerminal,
    TerminalNumber,
    NonTerminalMDD,
};

use crate::dot::Dot;
use crate::count::Count;
use crate::gc::Gc;

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    ADD,
    SUB,
    MUL,
    DIV,
    MIN,
    MAX,
    AND,
    OR,
    XOR,
    NOT,
    EQ,
    NEQ,
    LT,
    LTE,
    GT,
    GTE,
}

type Node<V> = MtMddNode<V>;

#[derive(Debug,Clone)]
pub enum MtMddNode<V> {
    NonTerminal(Rc<NonTerminalMDD<Node<V>>>),
    Terminal(Rc<TerminalNumber<V>>),
    One,
    Zero,
    Undet,
}

impl<V> PartialEq for Node<V> where V: TerminalNumberValue {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<V> Eq for Node<V> where V: TerminalNumberValue {}

impl<V> Hash for Node<V> where V: TerminalNumberValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<V> Node<V> where V: TerminalNumberValue {
    pub fn new_nonterminal(id: NodeId, header: &NodeHeader, nodes: &[Self]) -> Self {
        let x = NonTerminalMDD::new(
            id,
            header.clone(),
            nodes.iter().map(|x| x.clone()).collect::<Vec<_>>().into_boxed_slice(),
        );
        Self::NonTerminal(Rc::new(x))
    }

    pub fn new_terminal(id: NodeId, value: V) -> Self {
        let x = TerminalNumber::new(id, value);
        Self::Terminal(Rc::new(x))
    }
    
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Terminal(x) => x.id(),
            Self::One => 1,
            Self::Zero => 0,
            Self::Undet => 2,
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
pub struct MtMdd<V> {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node<V>,
    one: Node<V>,
    undet: Node<V>,
    vtable: HashMap<V,Node<V>>,
    utable: HashMap<(HeaderId, Box<[NodeId]>), Node<V>>,
    cache: HashMap<(Operation, NodeId, NodeId), Node<V>>,
}

impl<V> MtMdd<V> where V: TerminalNumberValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 3,
            zero: Node::Zero,
            one: Node::One,
            undet: Node::Undet,
            vtable: HashMap::default(),
            utable: HashMap::default(),
            cache: HashMap::default(),
        }
    }

    pub fn size(&self) -> (usize, HeaderId, NodeId, usize) {
        (self.vtable.len(), self.num_headers, self.num_nodes, self.utable.len())
    }
   
    pub fn value(&mut self, value: V) -> Node<V> {
        match self.vtable.get(&value) {
            Some(x) => x.clone(),
            None => {
                let node = Node::new_terminal(self.num_nodes, value);
                self.num_nodes += 1;
                self.vtable.insert(value, node.clone());
                node
            }
        }
    }

    pub fn header(&mut self, level: Level, label: &str, edge_num: usize) -> NodeHeader {
        let h = NodeHeader::new(self.num_headers, level, label, edge_num);
        self.num_headers += 1;
        h
    }
    
    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node<V>]) -> Result<Node<V>,String> {
        if h.edge_num() == nodes.len() {
            Ok(self.create_node(h, nodes))
        } else {
            Err(String::from("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, nodes: &[Node<V>]) -> Node<V> {
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
    
    pub fn zero(&self) -> Node<V> {
        self.zero.clone()
    }
    
    pub fn one(&self) -> Node<V> {
        self.one.clone()
    }

    pub fn undet(&self) -> Node<V> {
        self.undet.clone()
    }

    pub fn and(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::AND, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, Node::Zero) => self.zero(),
                    (Node::One, Node::Zero) => self.zero(),
                    (Node::Zero, Node::One) => self.zero(),
                    (Node::One, Node::One) => self.one(),
                    (Node::NonTerminal(fnode), Node::Zero|Node::One) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::Zero|Node::One, Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.and(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.and(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
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
                    (Node::Zero, Node::Zero) => self.zero(),
                    (Node::One, Node::Zero) => self.one(),
                    (Node::Zero, Node::One) => self.one(),
                    (Node::One, Node::One) => self.one(),
                    (Node::NonTerminal(fnode), Node::Zero|Node::One) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::Zero|Node::One, Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.and(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.and(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
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
                    (Node::Zero, Node::Zero) => self.zero(),
                    (Node::One, Node::Zero) => self.one(),
                    (Node::Zero, Node::One) => self.one(),
                    (Node::One, Node::One) => self.zero(),
                    (Node::NonTerminal(fnode), Node::Zero|Node::One) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::Zero|Node::One, Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.and(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.and(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.and(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn not(&mut self, f: &Node<V>) -> Node<V> {
        let key = (Operation::NOT, f.id(), 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    Node::Zero => self.one(),
                    Node::One => self.zero(),
                    Node::NonTerminal(fnode) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.not(f)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn add(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::ADD, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value() + gnode.value()),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.add(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.add(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.add(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.add(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.add(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn sub(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::SUB, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value() - gnode.value()),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.sub(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.sub(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.sub(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.sub(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.sub(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn mul(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::MUL, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value() * gnode.value()),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.mul(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.mul(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.mul(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.mul(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.mul(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn div(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::DIV, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value() / gnode.value()),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.div(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.div(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.div(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.div(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.div(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn min(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::MIN, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(std::cmp::min(fnode.value(), gnode.value())),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.min(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.min(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.min(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.min(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.min(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn max(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::MAX, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(std::cmp::max(fnode.value(), gnode.value())),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.max(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.max(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.max(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.max(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.max(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn eq(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::EQ, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, Node::Zero) => self.one(),
                    (Node::Zero, Node::One) => self.zero(),
                    (Node::One, Node::Zero) => self.zero(),
                    (Node::One, Node::One) => self.one(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() == gnode.value() => self.one(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() != gnode.value() => self.zero(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.eq(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.eq(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.eq(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.eq(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.eq(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn neq(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::NEQ, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Zero, Node::Zero) => self.zero(),
                    (Node::Zero, Node::One) => self.one(),
                    (Node::One, Node::Zero) => self.one(),
                    (Node::One, Node::One) => self.zero(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() != gnode.value() => self.one(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() == gnode.value() => self.zero(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.neq(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.neq(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.neq(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.neq(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.neq(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn lt(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::LT, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() < gnode.value() => self.one(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() >= gnode.value() => self.zero(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.lt(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.lt(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.lt(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.lt(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.lt(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn lte(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::LTE, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() <= gnode.value() => self.one(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() > gnode.value() => self.zero(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.lte(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.lte(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.lte(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.lte(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.lte(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn gt(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::GT, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() > gnode.value() => self.one(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() <= gnode.value() => self.zero(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.gt(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.gt(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.gt(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.gt(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.gt(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn gte(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::GTE, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() >= gnode.value() => self.one(),
                    (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() < gnode.value() => self.zero(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.gte(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.gte(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.gte(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.gte(f, g)).collect();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.gte(f, g)).collect();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => self.undet(),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    // pub fn iffunc(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
    //     let key = (Operation::NEQ, f.id(), g.id());
    //     match self.cache.get(&key) {
    //         Some(x) => x.clone(),
    //         None => {
    //             let node = match (f, g) {
    //                 (Node::Zero, Node::Zero) => self.zero(),
    //                 (Node::Zero, Node::One) => self.one(),
    //                 (Node::One, Node::Zero) => self.one(),
    //                 (Node::One, Node::One) => self.zero(),
    //                 (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() != gnode.value() => self.one(),
    //                 (Node::Terminal(fnode), Node::Terminal(gnode)) if fnode.value() == gnode.value() => self.zero(),
    //                 (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
    //                     let nodes: Vec<_> = gnode.iter().map(|g| self.iffunc(f, g)).collect();
    //                     self.create_node(gnode.header(), &nodes)
    //                 },
    //                 (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
    //                     let nodes: Vec<_> = fnode.iter().map(|f| self.iffunc(f, g)).collect();
    //                     self.create_node(fnode.header(), &nodes)
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
    //                     let nodes: Vec<_> = fnode.iter().map(|f| self.iffunc(f, g)).collect();
    //                     self.create_node(fnode.header(), &nodes)
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
    //                     let nodes: Vec<_> = gnode.iter().map(|g| self.iffunc(f, g)).collect();
    //                     self.create_node(gnode.header(), &nodes)
    //                 },
    //                 (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
    //                     let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.iffunc(f, g)).collect();
    //                     self.create_node(fnode.header(), &nodes)
    //                 },
    //                 _ => self.undet(),
    //             };
    //             self.cache.insert(key, node.clone());
    //             node
    //         }
    //     }
    // }
}

impl<V> Gc for MtMdd<V> where V: TerminalNumberValue {
    type Node = Node<V>;

    fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    fn clear_table(&mut self) {
        self.vtable.clear();
        self.utable.clear();
    }
    
    fn gc_impl(&mut self, f: &Self::Node, visited: &mut HashSet<Self::Node>) {
        if visited.contains(f) {
            return
        }
        match f {
            Node::Terminal(fnode) => {
                self.vtable.insert(fnode.value(), f.clone());
            },
            Node::NonTerminal(fnode) => {
                let key = (fnode.header().id(), fnode.iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
                self.utable.insert(key, f.clone());
                for x in fnode.iter() {
                    self.gc_impl(&x, visited);
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
    }
}

impl<V> Count for Node<V> where V: TerminalNumberValue {
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
                        for x in fnode.iter() {
                            let tmp = x.count_edge_impl(visited);
                            sum += tmp + 1;
                        }
                        visited.insert(key);
                        sum
                    },
                    Node::Terminal(_) | Node::Zero | Node::One | Node::Undet => {
                        visited.insert(key);
                        0
                    },
                }
            }
        }
    }
}

impl<V> Dot for Node<V> where V: TerminalNumberValue {
    type Node = Node<V>;

    fn dot_impl<T>(&self, io: &mut T, visited: &mut HashSet<Self::Node>) where T: std::io::Write {
        if visited.contains(self) {
            return
        }
        match self {
            Node::Zero => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", self.id(), false);
                io.write(s.as_bytes()).unwrap();
            },
            Node::One => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", self.id(), true);
                io.write(s.as_bytes()).unwrap();
            },
            Node::Terminal(fnode) => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id(), fnode.value());
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id(), fnode.label());
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.iter().enumerate() {
                    if let Node::Zero | Node::One | Node::Terminal(_) | Node::NonTerminal(_) = x {
                        x.dot_impl(io, visited);
                        let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.id(), x.id(), i);
                        io.write(s.as_bytes()).unwrap();
                    }
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
        let zero = Node::new_terminal(0, 0);
        let one = Node::new_terminal(1, 1);
        println!("{:?}", zero);
        println!("{:?}", one);
    }

    #[test]
    fn new_nonterminal() {
        let zero = Node::new_terminal(0, 0);
        let one = Node::new_terminal(1, 1);
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = Node::new_nonterminal(3, &h, &vec![zero, one]);
        println!("{:?}", x);
        if let Node::NonTerminal(x) = &x {
            println!("{:?}", x.header());
        }
        println!("{:?}", x.header());
    }

    #[test]
    fn new_test1() {
        let mut dd = MtMdd::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h, &v);
        println!("{:?}", x);
        let y = dd.create_node(&h, &v);
        println!("{:?}", y);
        // println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd = MtMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h1, &v);
        let y = dd.create_node(&h2, &v);
        let z = dd.add(&x, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        // println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }
    
    #[test]
    fn new_test3() {
        let mut dd = MtMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h1, &v);
        let y = dd.create_node(&h2, &v);
        let z = dd.add(&x, &y);

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
        let mut dd = MtMdd::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h1, &v);
        let y = dd.create_node(&h2, &v);
        let z = dd.sub(&x, &y);

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
        let mut dd = MtMdd::<i64>::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h1, &v);
        let y = dd.create_node(&h2, &v);
        let z = dd.sub(&x, &y);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            z.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }
}
