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

use crate::mtmdd::{
    MtMddNode,
    MtMdd,
};

use crate::mdd::{
    MddNode,
    Mdd,
};

use crate::dot::Dot;
use crate::count::Count;
use crate::gc::Gc;

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    EQ,
    NEQ,
    LT,
    LTE,
    GT,
    GTE,
    IF,
    ELSE,
    UNION,
}

type VNode<V> = MtMddNode<V>;
type BNode = MddNode;
type Node<V> = MtMdd2Node<V>;

#[derive(Debug,Clone)]
pub enum MtMdd2Node<V> {
    Value(VNode<V>),
    Bool(BNode),
    Undet,
}

impl<V> PartialEq for Node<V> where V: TerminalNumberValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Value(f), Node::Value(g)) => f.id() == g.id(),
            (Node::Bool(f), Node::Bool(g)) => f.id() == g.id(),
            (Node::Undet, Node::Undet) => true,
            _ => false,
        }
    }
}

impl<V> Eq for Node<V> where V: TerminalNumberValue {}

impl<V> Hash for Node<V> where V: TerminalNumberValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Node::Value(f) => f.id().hash(state),
            Node::Bool(f) => f.id().hash(state),
            Node::Undet => 0.hash(state),
        }
    }
}

#[derive(Debug)]
pub struct MtMdd2<V> {
    mdd: Mdd,
    mtmdd: MtMdd<V>,
    num_headers: HeaderId,
    bcache: HashMap<(Operation, NodeId, NodeId), BNode>,
    vcache: HashMap<(Operation, NodeId, NodeId), VNode<V>>,
}

impl<V> MtMdd2<V> where V: TerminalNumberValue {
    pub fn new() -> Self {
        Self {
            mdd: Mdd::new(),
            mtmdd: MtMdd::new(),
            num_headers: 0,
            bcache: HashMap::default(),
            vcache: HashMap::default()
        }
    }

    pub fn size(&self) -> (usize, HeaderId, NodeId, usize) {
        let (u1, _x1, y1, z1) = self.mtmdd.size();
        let (_x2, y2, z2) = self.mdd.size();
        (u1, self.num_headers, y1+y2, z1+z2)
    }
   
    pub fn one(&self) -> Node<V> {
        Node::Bool(self.mdd.one())
    }

    pub fn zero(&self) -> Node<V> {
        Node::Bool(self.mdd.zero())
    }

    pub fn value(&mut self, value: V) -> Node<V> {
        Node::Value(self.mtmdd.value(value))
    }

    pub fn header(&mut self, level: Level, label: &str, edge_num: usize) -> NodeHeader {
        let h = NodeHeader::new(self.num_headers, level, label, edge_num);
        self.num_headers += 1;
        h
    }

    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node<V>]) -> Result<Node<V>,String> {
        if h.edge_num() != nodes.len() {
            return Err(String::from("Did not match the number of edges in header and arguments."))
        }
        let elem_value: Result<Vec<VNode<V>>,String> = nodes.iter().map(|x| {
            match x {
                Node::Value(f) => Ok(f.clone()),
                _ => Err(String::from("nodes includes different type.")),
            }
        }).collect();
        match elem_value {
            Ok(x) => Ok(Node::Value(self.mtmdd.create_node(h, &x))),
            Err(_) => {
                let elem_bool: Result<Vec<BNode>,String> = nodes.iter().map(|x| {
                    match x {
                        Node::Bool(f) => Ok(f.clone()),
                        _ => Err(String::from("nodes includes different type.")),
                    }
                }).collect();
                match elem_bool {
                    Ok(x) => Ok(Node::Bool(self.mdd.create_node(h, &x))),
                    Err(_) => Err(String::from("nodes cannot be converted either value nor bool."))
                }
            },
        }
    }

    pub fn and(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.and(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn or(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.or(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn xor(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.xor(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn not(&mut self, f: &Node<V>) -> Node<V> {
        match f {
            Node::Bool(fnode) => Node::Bool(self.mdd.not(&fnode)),
            _ => Node::Undet,
        }
    }

    pub fn add(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.add(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn sub(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.sub(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn mul(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.mul(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn div(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.div(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn max(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.max(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn min(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.min(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn eq(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => {
                let tmp = self.mdd.xor(&fnode, &gnode);
                Node::Bool(self.mdd.not(&tmp))
            },
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.veq(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn veq(&mut self, f: &VNode<V>, g: &VNode<V>) -> BNode {
        let key = (Operation::EQ, f.id(), g.id());
        match self.bcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() == gnode.value() => self.mdd.one(),
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() != gnode.value() => self.mdd.zero(),
                    (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.veq(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.veq(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.veq(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.veq(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.veq(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mdd.undet(),
                };
                self.bcache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn neq(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.xor(&fnode, &gnode)),
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.vneq(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn vneq(&mut self, f: &VNode<V>, g: &VNode<V>) -> BNode {
        let key = (Operation::NEQ, f.id(), g.id());
        match self.bcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() != gnode.value() => self.mdd.one(),
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() == gnode.value() => self.mdd.zero(),
                    (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vneq(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vneq(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vneq(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vneq(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.vneq(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mdd.undet(),
                };
                self.bcache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn lt(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.vlt(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn vlt(&mut self, f: &VNode<V>, g: &VNode<V>) -> BNode {
        let key = (Operation::LT, f.id(), g.id());
        match self.bcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() < gnode.value() => self.mdd.one(),
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() >= gnode.value() => self.mdd.zero(),
                    (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vlt(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vlt(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vlt(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vlt(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.vlt(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mdd.undet(),
                };
                self.bcache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn lte(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.vlte(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn vlte(&mut self, f: &VNode<V>, g: &VNode<V>) -> BNode {
        let key = (Operation::LTE, f.id(), g.id());
        match self.bcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() <= gnode.value() => self.mdd.one(),
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() > gnode.value() => self.mdd.zero(),
                    (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vlte(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vlte(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vlte(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vlte(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.vlte(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mdd.undet(),
                };
                self.bcache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn gt(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.vgt(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn vgt(&mut self, f: &VNode<V>, g: &VNode<V>) -> BNode {
        let key = (Operation::GT, f.id(), g.id());
        match self.bcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() > gnode.value() => self.mdd.one(),
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() <= gnode.value() => self.mdd.zero(),
                    (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vgt(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vgt(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vgt(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vgt(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.vgt(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mdd.undet(),
                };
                self.bcache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn gte(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.vgte(&fnode, &gnode)),
            _ => Node::Undet,
        }
    }

    pub fn vgte(&mut self, f: &VNode<V>, g: &VNode<V>) -> BNode {
        let key = (Operation::GTE, f.id(), g.id());
        match self.bcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() >= gnode.value() => self.mdd.one(),
                    (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() < gnode.value() => self.mdd.zero(),
                    (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vgte(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vgte(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vgte(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vgte(f, g)).collect();
                        self.mdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.vgte(f, g)).collect();
                        self.mdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mdd.undet(),
                };
                self.bcache.insert(key, node.clone());
                node
            }
        }
    }

    fn vif(&mut self, f: &BNode, g: &VNode<V>) -> VNode<V> {
        let key = (Operation::IF, f.id(), g.id());
        match self.vcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (BNode::Zero, _) => self.mtmdd.undet(),
                    (BNode::One, _) => g.clone(),
                    (BNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vif(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    (BNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vif(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    (BNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vif(f, g)).collect();
                        self.mtmdd.create_node(gnode.header(), &nodes)
                    },
                    (BNode::NonTerminal(fnode),VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.vif(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mtmdd.undet(),
                };
                self.vcache.insert(key, node.clone());
                node
            }
        }
    }

    fn velse(&mut self, f: &BNode, g: &VNode<V>) -> VNode<V> {
        let key = (Operation::ELSE, f.id(), g.id());
        match self.vcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (BNode::Zero, _) => g.clone(),
                    (BNode::One, _) => self.mtmdd.undet(),
                    (BNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.velse(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    (BNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.velse(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    (BNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.velse(f, g)).collect();
                        self.mtmdd.create_node(gnode.header(), &nodes)
                    },
                    (BNode::NonTerminal(fnode),VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.velse(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mtmdd.undet(),
                };
                self.vcache.insert(key, node.clone());
                node
            }
        }
    }

    fn vunion(&mut self, f: &VNode<V>, g: &VNode<V>) -> VNode<V> {
        let key = (Operation::UNION, f.id(), g.id());
        match self.vcache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (VNode::Undet, _) => g.clone(),
                    (_, VNode::Undet) => f.clone(),
                    (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vunion(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                        let nodes: Vec<_> = gnode.iter().map(|f| self.vunion(f, g)).collect();
                        self.mtmdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().map(|f| self.vunion(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes: Vec<_> = gnode.iter().map(|g| self.vunion(f, g)).collect();
                        self.mtmdd.create_node(gnode.header(), &nodes)
                    },
                    (VNode::NonTerminal(fnode),VNode::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes: Vec<_> = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.vunion(f, g)).collect();
                        self.mtmdd.create_node(fnode.header(), &nodes)
                    },
                    _ => self.mtmdd.undet(),
                };
                self.vcache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn vifelse(&mut self, f: &BNode, g: &VNode<V>, h: &VNode<V>) -> VNode<V> {
        let x = self.vif(f, g);
        let y = self.velse(f, h);
        self.vunion(&x, &y)
    }

    pub fn ifelse(&mut self, f: &Node<V>, g: &Node<V>, h: &Node<V>) -> Node<V> {
        match (f, g, h) {
            (Node::Bool(fnode), Node::Value(gnode), Node::Value(hnode)) => {
                Node::Value(self.vifelse(&fnode, &gnode, &hnode))
            },
            (Node::Bool(fnode), Node::Bool(gnode), Node::Bool(hnode)) => {
                Node::Bool(self.mdd.ite(&fnode, &gnode, &hnode))
            },
            _ => Node::Undet,
        }
    }
}

impl<V> Gc for MtMdd2<V> where V: TerminalNumberValue {
    type Node = Node<V>;

    fn clear_cache(&mut self) {
        self.mdd.clear_cache();
        self.mtmdd.clear_cache();
    }
    
    fn clear_table(&mut self) {
        self.mdd.clear_table();
        self.mtmdd.clear_table();
    }
    
    fn gc_impl(&mut self, f: &Self::Node, _visited: &mut HashSet<Self::Node>) {
        match f {
            Node::Bool(bnode) => {
                let mut visited: HashSet<BNode> = HashSet::default();
                self.mdd.gc_impl(&bnode, &mut visited)
            },
            Node::Value(vnode) => {
                let mut visited: HashSet<VNode<V>> = HashSet::default();
                self.mtmdd.gc_impl(&vnode, &mut visited)
            },
            _ => ()
        }
    }
}

impl<V> Dot for Node<V> where V: TerminalNumberValue {
    type Node = Node<V>;

    fn dot_impl<T>(&self, io: &mut T, _visited: &mut HashSet<Self::Node>) where T: std::io::Write {
        match self {
            Node::Value(f) => {
                let mut visited = HashSet::<VNode<V>>::default();
                f.dot_impl(io, &mut visited)
            },
            Node::Bool(f) => {
                let mut visited = HashSet::<BNode>::default();
                f.dot_impl(io, &mut visited)
            },
            Node::Undet => {
                writeln!(io, "undet [label=\"undet\", shape=ellipse];").unwrap();
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::BufWriter, ops::RangeInclusive};

    // impl Drop for Node {
    //     fn drop(&mut self) {
    //         println!("Dropping Node{}", self.id());
    //     }
    // }

    #[test]
    fn test_ope1() {
        let n = 100;
        let mut f = MtMdd2::<i64>::new();
        let h1 = f.header(1, "y1", 2);
        let h2 = f.header(2, "y2", 2);
        let h3 = f.header(3, "y3", 2);
        let consts: Vec<_> = (0..n).into_iter().map(|i| f.value(i)).collect();
        let y1 = f.node(&h1, &[consts[0].clone(), consts[1].clone()]).unwrap();
        let y2 = f.node(&h2, &[consts[0].clone(), consts[1].clone()]).unwrap();
        let y3 = f.node(&h3, &[consts[0].clone(), consts[1].clone()]).unwrap();
        // let tmp2 = f.mul(&consts[2], &y2);
        let tmp3 = f.mul(&consts[3], &y3);
        let tmp4 = f.lt(&y3, &consts[2]);
    }

    fn gen_var<T>(f: &mut MtMdd2<T>, label: &str, level: usize, range: &[T]) -> MtMdd2Node<T>
    where
        T: TerminalNumberValue,
    {
        let count = range.len();
        let htmp = f.header(level, label, count);
        let tmp = range.iter().map(|&i| f.value(i)).collect::<Vec<_>>();
        f.node(&htmp, &tmp).unwrap()
    }

    #[test]
    fn test_ope2() {
        // x + y <= 5, 0 <= x <= 5, 0 <= y <= 5
        let mut f = MtMdd2::<i64>::new();
        let x = gen_var(&mut f, "x", 1, &vec![0,1,2,3,4,5]);
        let y = gen_var(&mut f, "y", 2, &vec![0,1,2,3,4,5]);
        let tmp1 = f.add(&x, &y);
        let tmp2 = f.value(5);
        let tmp3 = f.lte(&tmp1, &tmp2);
        println!("{}", tmp3.dot_string());
    }

    #[test]
    fn test_ope3() {
        // ifelse(x + y <= 5, x, y), 0 <= x <= 5, 0 <= y <= 5
        let mut f = MtMdd2::<i64>::new();
        let x = gen_var(&mut f, "x", 1, &vec![0,1,2,3,4,5]);
        let y = gen_var(&mut f, "y", 2, &vec![0,1,2,3,4,5]);
        let tmp1 = f.add(&x, &y);
        let tmp2 = f.value(5);
        let tmp3 = f.lte(&tmp1, &tmp2);
        let c1 = f.value(19);
        let c2 = f.value(20);
        let tmp4 = f.ifelse(&tmp3, &x, &y);
        println!("{}", tmp4.dot_string());
    }
}