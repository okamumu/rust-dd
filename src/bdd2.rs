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

type Node<V> = BDD2Node<V>;

#[derive(Debug)]
pub enum BDD2Node<V> {
    NonTerminal(NonTerminalBDD<NodeId>),
    Terminal(TerminalBinary<V>),
    None,
}

impl<V> PartialEq for BDD2Node<V> where V: TerminalBinaryValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::NonTerminal(x), Node::NonTerminal(y)) => x.id() == y.id(),
            (Node::Terminal(x), Node::Terminal(y)) => x.value() == y.value(),
            _ => false,
        }
    }
}

impl<V> Eq for BDD2Node<V> where V: TerminalBinaryValue {}

impl<V> Hash for BDD2Node<V> where V: TerminalBinaryValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<V> BDD2Node<V> where V: TerminalBinaryValue {
    pub fn new_nonterminal(id: NodeId, header: &NodeHeader, low: NodeId, high: NodeId) -> Self {
        let x = NonTerminalBDD::new(id, header.clone(), [low, high]);
        Self::NonTerminal(x)
    }

    pub fn new_terminal(id: NodeId, value: V) -> Self {
        let x = TerminalBinary::new(id, value);
        Self::Terminal(x)
    }
    
    pub fn id(&self) -> NodeId {
        match self {
            Self::NonTerminal(x) => x.id(),
            Self::Terminal(x) => x.id(),
            _ => 0,
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
pub struct BDD2<V=u8> {
    num_headers: HeaderId,
    num_nodes: NodeId,
    nodes: Vec<Node<V>>,
    zero: NodeId,
    one: NodeId,
    utable: HashMap<(HeaderId, NodeId, NodeId), NodeId>,
    cache: HashMap<(Operation, NodeId, NodeId), NodeId>,
}

impl<V> BDD2<V> where V: TerminalBinaryValue {
    pub fn new() -> Self {
        Self {
            num_headers: 0,
            num_nodes: 3,
            nodes: vec![Node::None, Node::new_terminal(1, V::low()), Node::new_terminal(2, V::high())],
            zero: 1,
            one: 2,
            utable: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    pub fn header(&mut self, level: Level, label: &str) -> NodeHeader {
        let h = NodeHeader::new(self.num_headers, level, label, 2);
        self.num_headers += 1;
        h
    }

    pub fn get(&self, id: NodeId) -> Option<&NonTerminalBDD<NodeId>> {
        if let Node::NonTerminal(f) = &self.nodes[id] {
            Some(&f)
        } else {
            None
        }
    }
    
    // pub fn get_mut(&mut self, id: NodeId) -> &mut Node<V> {
    //     &mut self.nodes[id]
    // }

    pub fn node(&mut self, h: &NodeHeader, nodes: &[NodeId]) -> Result<NodeId,String> {
        if nodes.len() == h.edge_num() {
            Ok(self.create_node(h, nodes[0], nodes[1]))
        } else {
            Err(format!("Did not match the number of edges in header and arguments."))
        }
    }

    fn create_node(&mut self, h: &NodeHeader, low: NodeId, high: NodeId) -> NodeId {
        if low == high {
            return low
        }
        
        let key = (h.id(), low, high);
        match self.utable.get(&key) {
            Some(x) => *x,
            None => {
                let nodeid = self.num_nodes;
                self.nodes.push(Node::new_nonterminal(nodeid, h, low, high));
                self.num_nodes += 1;
                self.utable.insert(key, nodeid);
                nodeid
            }
        }
    }
    
    pub fn zero(&self) -> NodeId {
        self.zero
    }
    
    pub fn one(&self) -> NodeId {
        self.one
    }

    pub fn not(&mut self, f: NodeId) -> NodeId {
        let key = (Operation::NOT, f, 0);
        match self.cache.get(&key) {
            Some(x) => x.clone(),
            None => {
                let node = match f {
                    1 => self.one(),
                    2 => self.zero(),
                    _ => {
                        let lowid = self.get(f).unwrap()[0];
                        let highid = self.get(f).unwrap()[1];
                        let low = self.not(lowid);
                        let high = self.not(highid);
                        let h = self.get(f).unwrap().header().clone();
                        self.create_node(&h, low, high)
                    },
                };
                self.cache.insert(key, node);
                node
            }
        }
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
}
