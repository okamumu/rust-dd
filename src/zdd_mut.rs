use std::hash::Hash;

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
};

use crate::dot::{
    Dot,
};

use crate::bdd_mut::{
    BddMutNode,
};

#[derive(Debug,PartialEq,Eq,Hash)]
enum Operation {
    NOT,
    INTERSECT,
    UNION,
    SETDIFF,
    PRODUCT,
}

pub type ZddMutNode<V> = BddMutNode<V>;
type Node<V> = BddMutNode<V>;

#[derive(Debug)]
pub struct ZddMut<V=u8> {
    num_headers: HeaderId,
    num_nodes: NodeId,
    zero: Node<V>,
    one: Node<V>,
    utable: HashMap<(HeaderId, NodeId, NodeId), Node<V>>,
    cache: HashMap<(Operation, NodeId, NodeId), Node<V>>,
}

impl<V> ZddMut<V> where V: TerminalBinaryValue {
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
        if high == &self.zero {
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
                        let low = self.not(&fnode.borrow()[0]);
                        let high = self.not(&fnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn intersect(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::INTERSECT, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => self.zero(),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => g.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => self.zero(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let low = self.intersect(&fnode.borrow()[0], g);
                        let high = self.intersect(&fnode.borrow()[1], &self.zero());
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let low = self.intersect(f, &gnode.borrow()[0]);
                        let high = self.intersect(&self.zero(), &gnode.borrow()[1]);
                        self.create_node(gnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let low = self.intersect(&fnode.borrow()[0], &gnode.borrow()[0]);
                        let high = self.intersect(&fnode.borrow()[1], &gnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn union(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::UNION, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => self.one(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => self.one(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let low = self.union(&fnode.borrow()[0], g);
                        let high = self.union(&fnode.borrow()[1], &self.zero());
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let low = self.union(f, &gnode.borrow()[0]);
                        let high = self.union(&self.zero(), &gnode.borrow()[1]);
                        self.create_node(gnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let low = self.union(&fnode.borrow()[0], &gnode.borrow()[0]);
                        let high = self.union(&fnode.borrow()[1], &gnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn setdiff(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::SETDIFF, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => self.not(g),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => self.not(f),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let low = self.setdiff(&fnode.borrow()[0], g);
                        let high = self.setdiff(&fnode.borrow()[1], &self.zero());
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let low = self.setdiff(f, &gnode.borrow()[0]);
                        let high = self.setdiff(&self.zero(), &gnode.borrow()[1]);
                        self.create_node(gnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let low = self.setdiff(&fnode.borrow()[0], &gnode.borrow()[0]);
                        let high = self.setdiff(&fnode.borrow()[1], &gnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn product(&mut self, f: &Node<V>, g: &Node<V>) -> Node<V> {
        let key = (Operation::PRODUCT, f.id(), g.id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value() == V::low() => self.zero(),
                    (Node::Terminal(fnode), _) if fnode.value() == V::high() => g.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::low() => self.zero(),
                    (_, Node::Terminal(gnode)) if gnode.value() == V::high() => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() > gnode.borrow().level() => {
                        let low = self.product(&fnode.borrow()[0], g);
                        let high = self.product(&fnode.borrow()[1], g);
                        self.create_node(fnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() < gnode.borrow().level() => {
                        let low = self.product(f, &gnode.borrow()[0]);
                        let high = self.product(f, &gnode.borrow()[1]);
                        self.create_node(gnode.borrow().header(), &low, &high)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.borrow().level() == gnode.borrow().level() => {
                        let low = self.product(&fnode.borrow()[0], &gnode.borrow()[0]);
                        let high = self.product(&fnode.borrow()[1], &gnode.borrow()[1]);
                        self.create_node(fnode.borrow().header(), &low, &high)
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
}

impl<V> Dot for ZddMut<V> where V: TerminalBinaryValue {
    type Node = Node<V>;

    fn dot_impl<T>(&self, io: &mut T, f: &Self::Node, visited: &mut HashSet<Self::Node>) where T: std::io::Write {
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
                    self.dot_impl(io, x, visited);
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
    use std::rc::Rc;

    // impl Drop for Node<V> {
    //     fn drop(&mut self) {
    //         println!("Dropping Node<V>{}", self.id());
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
            println!("{:?}", x.borrow().header());
        }
        println!("{:?}", x.header());
    }

    #[test]
    fn new_test1() {
        let mut dd: ZddMut = ZddMut::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &dd.zero(), &dd.one());
        println!("{:?}", x);
        let y = dd.create_node(&h, &dd.zero(), &dd.one());
        println!("{:?}", y);
        println!("{:?}", Rc::strong_count(&y.header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd: ZddMut = ZddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.one(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.intersect(&x, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{:?}", Rc::strong_count(&y.header().unwrap()));
    }
    
    #[test]
    fn new_test3() {
        let mut dd: ZddMut = ZddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.one(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.intersect(&x, &y);

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
        let mut dd: ZddMut = ZddMut::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &dd.zero(), &dd.one());
        let y = dd.create_node(&h2, &dd.zero(), &dd.one());
        let z = dd.union(&x, &y);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            dd.dot(&mut io, &z);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

    }

}
