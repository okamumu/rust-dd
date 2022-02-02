use std::rc::Rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};
use core::slice::Iter;

use crate::common::{
    HeaderId,
    NodeId,
    Level,
    HashMap,
    HashSet,
    TerminalValue,
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

#[derive(Debug)]
pub struct NonTerminalNode<T> {
    id: NodeId,
    header: NodeHeader,
    nodes: Box<[Node<T>]>,
}

impl<T> NonTerminalNode<T> where T: TerminalValue {
    pub fn node_iter(&self) -> Iter<Node<T>> {
        self.nodes.iter()
    }
}

#[derive(Debug)]
pub struct TerminalNode<T> {
    id: NodeId,
    value: T,
}

#[derive(Debug,Clone)]
pub enum Node<T> {
    NonTerminal(Rc<NonTerminalNode<T>>),
    Terminal(Rc<TerminalNode<T>>),
}

impl<T> PartialEq for Node<T> where T: TerminalValue {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T> Eq for Node<T> where T: TerminalValue {}

impl<T> Hash for Node<T> where T: TerminalValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<T> Node<T> where T: TerminalValue {
    fn new_nonterminal(id: NodeId, header: &NodeHeader, nodes: &[Node<T>]) -> Self {
        let x = NonTerminalNode {
            id: id,
            header: header.clone(),
            nodes: nodes.iter().map(|x| x.clone()).collect::<Vec<_>>().into_boxed_slice(),
        };
        Node::NonTerminal(Rc::new(x))
    }

    fn new_terminal(id: NodeId, value: T) -> Self {
        let x = TerminalNode {
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
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct MtMDD<T> {
    num_headers: HeaderId,
    num_nodes: NodeId,
    vtable: HashMap<T,Node<T>>,
    utable: HashMap<(HeaderId, Box<[NodeId]>), Node<T>>,
    cache: HashMap<(Operation, NodeId, NodeId), Node<T>>,
}

impl<T> MtMDD<T> where T: TerminalValue {
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
   
    pub fn value(&mut self, value: T) -> Node<T> {
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
    
    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node<T>]) -> Result<Node<T>,String> {
        if h.edge_num == nodes.len() {
            Ok(self.create_node(h, nodes))
        } else {
            Err(String::from("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, nodes: &[Node<T>]) -> Node<T> {
        if nodes.iter().all(|x| &nodes[0] == x) {
            return nodes[0].clone()
        }
        
        let key = (h.id, nodes.iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
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
    
    pub fn add(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::ADD, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value + gnode.value),
                    (Node::Terminal(fnode), Node::NonTerminal(_gnode)) if fnode.value == T::zero() => g.clone(),
                    (Node::NonTerminal(_fnode), Node::Terminal(gnode)) if gnode.value == T::zero() => f.clone(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.nodes.iter().map(|g| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.nodes.iter().map(|f| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.add(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn sub(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::SUB, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value - gnode.value),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.nodes.iter().map(|g| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.nodes.iter().map(|f| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.sub(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn mul(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::MUL, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value * gnode.value),
                    (Node::Terminal(fnode), Node::NonTerminal(_gnode)) if fnode.value == T::zero() => self.value(T::zero()),
                    (Node::NonTerminal(_fnode), Node::Terminal(gnode)) if gnode.value == T::zero() => self.value(T::zero()),
                    (Node::Terminal(fnode), Node::NonTerminal(_gnode)) if fnode.value == T::one() => g.clone(),
                    (Node::NonTerminal(_fnode), Node::Terminal(gnode)) if gnode.value == T::one() => f.clone(),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.nodes.iter().map(|g| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.nodes.iter().map(|f| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.mul(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn div(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::DIV, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(fnode.value / gnode.value),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.nodes.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.nodes.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn min(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::MIN, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(std::cmp::min(fnode.value, gnode.value)),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.nodes.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.nodes.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn max(&mut self, f: &Node<T>, g: &Node<T>) -> Node<T> {
        let key = (Operation::MAX, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), Node::Terminal(gnode)) => self.value(std::cmp::max(fnode.value, gnode.value)),
                    (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                        let nodes = gnode.nodes.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                        let nodes = fnode.nodes.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.div(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
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
    
    pub fn rebuild(&mut self, fs: &[Node<T>]) {
        self.vtable.clear();
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
            Node::Terminal(fnode) => {
                self.vtable.insert(fnode.value, f.clone());
            },
            Node::NonTerminal(fnode) => {
                let key = (fnode.header.id, fnode.nodes.iter().map(|x| x.id()).collect::<Vec<_>>().into_boxed_slice());
                self.utable.insert(key, f.clone());
                for x in fnode.nodes.iter() {
                    self.make_utable_(&x, visited);
                }
            },
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
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id, fnode.header.label);
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
        println!("{:?}", h.level);
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
            println!("{:?}", x.header);
        }
        println!("{:?}", x.header());
    }

    #[test]
    fn new_test1() {
        let mut dd = MtMDD::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h, &v);
        println!("{:?}", x);
        let y = dd.create_node(&h, &v);
        println!("{:?}", y);
        println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd = MtMDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h1, &v);
        let y = dd.create_node(&h2, &v);
        let z = dd.add(&x, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{:?}", Rc::strong_count(y.header().unwrap()));
    }
    
    #[test]
    fn new_test3() {
        let mut dd = MtMDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h1, &v);
        let y = dd.create_node(&h2, &v);
        let z = dd.add(&x, &y);

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
        let mut dd = MtMDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let v = vec![dd.value(0), dd.value(1)];
        let x = dd.create_node(&h1, &v);
        let y = dd.create_node(&h2, &v);
        let z = dd.sub(&x, &y);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            dd.dot(&mut io, &z);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }
}
