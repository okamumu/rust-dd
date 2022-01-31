use std::rc::Rc;
use std::ops::Deref;
use std::hash::{Hash, Hasher};

// use std::collections::{HashMap, HashSet};
use hashbrown::{HashMap, HashSet};

type HeaderId = usize;
type NodeId = usize;
type OperationId = usize;
type Level = usize;

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
pub struct NonTerminalNode {
    id: NodeId,
    header: NodeHeader,
    nodes: Box<[Node]>,
}

#[derive(Debug)]
pub struct TerminalNode {
    id: NodeId,
    value: bool,
}

#[derive(Debug,Clone)]
pub enum Node {
    NonTerminal(Rc<NonTerminalNode>),
    Terminal(Rc<TerminalNode>),
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().hash(state);
    }
}

impl Node {
    fn new_nonterminal(id: NodeId, header: &NodeHeader, nodes: &[Node]) -> Self {
        let x = NonTerminalNode {
            id: id,
            header: header.clone(),
            nodes: nodes.iter().map(|x| x.clone()).collect::<Vec<_>>().into_boxed_slice(),
        };
        Node::NonTerminal(Rc::new(x))
    }

    fn new_terminal(id: NodeId, value: bool) -> Self {
        let x = TerminalNode {
            id: id,
            value: value,
        };
        Node::Terminal(Rc::new(x))
    }
    
    pub fn get_id(&self) -> NodeId {
        match self {
            Node::NonTerminal(x) => x.id,
            Node::Terminal(x) => x.id,
        }        
    }

    pub fn get_header(&self) -> Option<&NodeHeader> {
        match self {
            Node::NonTerminal(x) => Some(&x.header),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct MDD {
    pub num_headers: usize,
    pub num_nodes: usize,
    zero: Node,
    one: Node,
    pub utable: HashMap<(HeaderId, Box<[NodeId]>), Node>,
    cache: HashMap<(OperationId, NodeId, NodeId), Node>,
}

const NOT: OperationId = 0;
const AND: OperationId = 1;
const OR: OperationId = 2;
const XOR: OperationId = 3;

impl MDD {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 2,
            zero: Node::new_terminal(0, false),
            one: Node::new_terminal(1, true),
            utable: HashMap::new(),
            cache: HashMap::new(),
        }
    }
    
    pub fn header(&mut self, level: Level, label: &str, edge_num: usize) -> NodeHeader {
        let h = NodeHeader::new(self.num_headers, level, label, edge_num);
        self.num_headers += 1;
        h
    }
    
    pub fn node(&mut self, h: &NodeHeader, nodes: &[Node]) -> Result<Node,String> {
        if h.edge_num == nodes.len() {
            Ok(self.create_node(h, nodes))
        } else {
            Err(String::from("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, nodes: &[Node]) -> Node {
        if nodes.iter().all(|x| &nodes[0] == x) {
            return nodes[0].clone()
        }
        
        let key = (h.id, nodes.iter().map(|x| x.get_id()).collect::<Vec<_>>().into_boxed_slice());
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
    
    pub fn get_zero(&self) -> Node {
        self.zero.clone()
    }
    
    pub fn get_one(&self) -> Node {
        self.one.clone()
    }

    pub fn not(&mut self, f: &Node) -> Node {
        let key = (NOT, f.get_id(), 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    Node::Terminal(fnode) if fnode.value == false => self.get_one(),
                    Node::Terminal(fnode) if fnode.value == true => self.get_zero(),
                    Node::NonTerminal(fnode) => {
                        let nodes = fnode.nodes.iter().map(|f| self.not(f)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn and(&mut self, f: &Node, g: &Node) -> Node {
        // if f == g {
        //     return f.clone()
        // }
        let key = (AND, f.get_id(), g.get_id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == false => self.get_zero(),
                    (Node::Terminal(fnode), _) if fnode.value == true => g.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value == false => self.get_zero(),
                    (_, Node::Terminal(gnode)) if gnode.value == true => f.clone(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.and(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.and(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.and(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }
    
    pub fn or(&mut self, f: &Node, g: &Node) -> Node {
        // if f == g {
        //     return f.clone()
        // }
        let key = (OR, f.get_id(), g.get_id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == false => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value == true => self.get_one(),
                    (_, Node::Terminal(gnode)) if gnode.value == false => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value == true => self.get_one(),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.or(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.or(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.or(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    _ => panic!("error"),
                };
                self.cache.insert(key, node.clone());
                node
            }
        }
    }

    pub fn xor(&mut self, f: &Node, g: &Node) -> Node {
        let key = (XOR, f.get_id(), g.get_id());
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match (f, g) {
                    (Node::Terminal(fnode), _) if fnode.value == false => g.clone(),
                    (Node::Terminal(fnode), _) if fnode.value == true => self.not(g),
                    (_, Node::Terminal(gnode)) if gnode.value == false => f.clone(),
                    (_, Node::Terminal(gnode)) if gnode.value == true => self.not(f),
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level > gnode.header.level => {
                        let nodes = fnode.nodes.iter().map(|f| self.xor(f, g)).collect::<Vec<_>>();
                        self.create_node(&fnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level < gnode.header.level => {
                        let nodes = gnode.nodes.iter().map(|g| self.xor(f, g)).collect::<Vec<_>>();
                        self.create_node(&gnode.header, &nodes)
                    },
                    (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.header.level == gnode.header.level => {
                        let nodes = fnode.nodes.iter().zip(gnode.nodes.iter()).map(|(f,g)| self.xor(f, g)).collect::<Vec<_>>();
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
        self.utable.clear();
    }
    
    pub fn make_utable(&mut self, f: &Node) {
        let mut visited = HashSet::new();
        self.make_utable_(f, &mut visited);
    }

    fn make_utable_(&mut self, f: &Node, visited: &mut HashSet<Node>) {
        if visited.contains(f) {
            return
        }
        match f {
            Node::NonTerminal(fnode) => {
                let key = (fnode.header.id, fnode.nodes.iter().map(|x| x.get_id()).collect::<Vec<_>>().into_boxed_slice());
                self.utable.insert(key, f.clone());
                for x in fnode.nodes.iter() {
                    self.make_utable_(&x, visited);
                }
            },
            _ => (),
        };
        visited.insert(f.clone());
    }

    pub fn dot<T>(&self, io: &mut T, f: &Node) where T: std::io::Write {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited = HashSet::new();
        io.write(s1.as_bytes()).unwrap();
        self.dot_(io, f, &mut visited);
        io.write(s2.as_bytes()).unwrap();
    }

    pub fn dot_<T>(&self, io: &mut T, f: &Node, visited: &mut HashSet<Node>) where T: std::io::Write {
        if visited.contains(f) {
            return
        }
        match f {
            Node::Terminal(fnode) if fnode.value == false => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id, 0);
                io.write(s.as_bytes()).unwrap();
            },
            Node::Terminal(fnode) if fnode.value == true => {
                let s = format!("\"obj{}\" [shape=square, label=\"{}\"];\n", fnode.id, 1);
                io.write(s.as_bytes()).unwrap();
            },
            Node::NonTerminal(fnode) => {
                let s = format!("\"obj{}\" [shape=circle, label=\"{}\"];\n", fnode.id, fnode.header.label);
                io.write(s.as_bytes()).unwrap();
                for (i,x) in fnode.nodes.iter().enumerate() {
                    self.dot_(io, x, visited);
                    let s = format!("\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n", fnode.id, x.get_id(), i);
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
    //         println!("Dropping Node{}", self.get_id());
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
        let x = Node::new_nonterminal(3, &h, &vec![zero, one]);
        println!("{:?}", x);
        if let Node::NonTerminal(x) = &x {
            println!("{:?}", x.header);
        }
        println!("{:?}", x.get_header());
    }

    #[test]
    fn new_test1() {
        let mut dd = MDD::new();
        let h = NodeHeader::new(0, 0, "x", 2);
        let x = dd.create_node(&h, &vec![dd.get_zero(), dd.get_one()]);
        println!("{:?}", x);
        let y = dd.create_node(&h, &vec![dd.get_zero(), dd.get_one()]);
        println!("{:?}", y);
        println!("{:?}", Rc::strong_count(y.get_header().unwrap()));
    }

    #[test]
    fn new_test2() {
        let mut dd = MDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &vec![dd.get_zero(), dd.get_one()]);
        let y = dd.create_node(&h2, &vec![dd.get_zero(), dd.get_one()]);
        let z = dd.and(&x, &y);
        println!("{:?}", x);
        println!("{:?}", y);
        println!("{:?}", z);
        println!("{:?}", Rc::strong_count(y.get_header().unwrap()));
    }
    
    #[test]
    fn new_test3() {
        let mut dd = MDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &vec![dd.get_zero(), dd.get_one()]);
        let y = dd.create_node(&h2, &vec![dd.get_zero(), dd.get_one()]);
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
        let mut dd = MDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &vec![dd.get_zero(), dd.get_one()]);
        let y = dd.create_node(&h2, &vec![dd.get_zero(), dd.get_one()]);
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
        let mut dd = MDD::new();
        let h1 = NodeHeader::new(0, 0, "x", 2);
        let h2 = NodeHeader::new(1, 1, "y", 2);
        let x = dd.create_node(&h1, &vec![dd.get_zero(), dd.get_one()]);
        let y = dd.create_node(&h2, &vec![dd.get_zero(), dd.get_one()]);
        let z = dd.and(&x, &y);
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
