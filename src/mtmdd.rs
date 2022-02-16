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

use crate::dot::{
    Dot,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    ADD,
    SUB,
    MUL,
    DIV,
    MIN,
    MAX,
}

type Node<V> = MtMddNode<V>;

#[derive(Debug,Clone)]
pub enum MtMddNode<V> {
    NonTerminal(Rc<NonTerminalMDD<Node<V>>>),
    Terminal(Rc<TerminalNumber<V>>),
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
    vtable: HashMap<V,Node<V>>,
    utable: HashMap<(HeaderId, Box<[NodeId]>), Node<V>>,
    cache: HashMap<(Operation, NodeId, NodeId), Node<V>>,
}

impl<V> MtMdd<V> where V: TerminalNumberValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 0,
            vtable: HashMap::new(),
            utable: HashMap::new(),
            cache: HashMap::new(),
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
    
    pub fn add(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::ADD, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value() + gnode.value()),
                    (Node::Terminal(fnode), Node::NonTerminal(_gnode)) if fnode.value() == V::zero() => g.clone(),
                    (Node::NonTerminal(_fnode), Node::Terminal(gnode)) if gnode.value() == V::zero() => f.clone(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.iter().map(|g| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.iter().map(|f| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes = fnode.iter().map(|f| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes = gnode.iter().map(|g| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => panic!("error"),
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
                        let nodes = gnode.iter().map(|g| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.iter().map(|f| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes = fnode.iter().map(|f| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes = gnode.iter().map(|g| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => panic!("error"),
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
                    (Node::Terminal(fnode), Node::NonTerminal(_gnode)) if fnode.value() == V::zero() => self.value(V::zero()),
                    (Node::NonTerminal(_fnode), Node::Terminal(gnode)) if gnode.value() == V::zero() => self.value(V::zero()),
                    (Node::Terminal(fnode), Node::NonTerminal(_gnode)) if fnode.value() == V::one() => g.clone(),
                    (Node::NonTerminal(_fnode), Node::Terminal(gnode)) if gnode.value() == V::one() => f.clone(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.iter().map(|g| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.iter().map(|f| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes = fnode.iter().map(|f| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes = gnode.iter().map(|g| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => panic!("error"),
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
                        let nodes = gnode.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes = fnode.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes = gnode.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => panic!("error"),
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
                        let nodes = gnode.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes = fnode.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes = gnode.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => panic!("error"),
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
                        let nodes = gnode.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() > gnode.level() => {
                        let nodes = fnode.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() < gnode.level() => {
                        let nodes = gnode.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(gnode.header(), &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.level() == gnode.level() => {
                        let nodes = fnode.iter().zip(gnode.iter()).map(|(f,g)| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(fnode.header(), &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    pub fn rebuild(&mut self, fs: &[Node<V>]) {
        self.vtable.clear();
        self.utable.clear();
        let mut visited = HashSet::new();
        for x in fs.iter() {
            self.rebuild_tables(x, &mut visited);
        }
    }

    fn rebuild_tables(&mut self, f: &Node<V>, visited: &mut HashSet<Node<V>>) {
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
                    self.rebuild_tables(&x, visited);
                }
            },
        };
        visited.insert(f.clone());
    }
}

impl<V> Dot for Node<V> where V: TerminalNumberValue {
    type Node = Node<V>;

    fn dot_impl<T>(&self, io: &mut T, visited: &mut HashSet<Self::Node>) where T: std::io::Write {
        if visited.contains(self) {
            return
        }
        match self {
            Node::Terminal(fnode) => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id(), fnode.value());
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id(), fnode.label());
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.iter().enumerate() {
                    x.dot_impl(io, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.id(), x.id(), i);
                    io.write(s.as_bytes()).unwrap();
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
}
