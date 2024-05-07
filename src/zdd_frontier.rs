use std::hash::Hash;
use std::collections::VecDeque;

use crate::common::{
    NodeId,
    Level,
    HashMap,
};

use crate::nodes::{
    NodeHeader,
    NonTerminal,
};

use crate::zdd_mut::{
    ZddMut,
    ZddMutNode,
};

type Node = ZddMutNode;

#[derive(Debug)]
pub enum CheckResult<N> {
    Terminal(N),
    NonTerminal,
}

pub trait FrontierState: Clone + PartialEq + Eq + Hash {
    type Node;
    type EdgeIndex;

    fn check(&self) -> CheckResult<Self::Node>;
    fn level(&self) -> Level;
    fn next(&self, i: Self::EdgeIndex) -> Self;
}

pub struct ZddFrontierBuilder<F> {
    num_nodes: NodeId,
    headers: Vec<NodeHeader>,
    nodes: HashMap<F,Node>,
    queue: VecDeque<(Node,F)>,
}

impl<F> ZddFrontierBuilder<F> where F: FrontierState<Node=Node, EdgeIndex=usize> {
    pub fn new(headers: &[NodeHeader]) -> Self {
        Self {
            num_nodes: 3,
            headers: headers.to_vec(),
            nodes: HashMap::default(),
            queue: VecDeque::new(),
        }
    }

    pub fn build(&mut self, s: &F) -> Node {
        let root = self.get_node(s);
        while let Some((Node::NonTerminal(fnode), state)) = self.queue.pop_front() {
            for (i,x) in fnode.borrow_mut().iter_mut().enumerate() {
                let s_next = state.next(i);
                let node = self.get_node(&s_next);
                *x = node;
            }
        }
        root
    }

    fn get_node(&mut self, s: &F) -> Node {
        match self.nodes.get(s) {
            Some(n) => n.clone(),
            None => {
                let node = match s.check() {
                    CheckResult::Terminal(n) => n,
                    CheckResult::NonTerminal => {
                        let level = s.level();
                        let node = Node::new_nonterminal(self.num_nodes, &self.headers[level], &Node::None, &Node::None);
                        self.num_nodes += 1;
                        self.queue.push_back((node.clone(), s.clone()));
                        node
                    },
                };
                self.nodes.insert(s.clone(), node.clone());
                node
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;
    use crate::dot::Dot;
        
    // impl Drop for Node {
    //     fn drop(&mut self) {
    //         println!("Dropping Node{}", self.id());
    //     }
    // }

    #[derive(Debug,Clone,PartialEq,Eq,Hash)]
    struct Combi(usize,usize);
    
    impl FrontierState for Combi {
        type Node = Node;
        type EdgeIndex = usize;
    
        fn check(&self) -> CheckResult<Node> {
            let value = self.1;
            if self.0 < value {
                CheckResult::Terminal(Node::Zero)
            } else {
                if self.0 == value {
                    CheckResult::Terminal(Node::One)
                } else {
                    CheckResult::NonTerminal
                }
            }
        }
    
        fn next(&self, i: Self::EdgeIndex) -> Self {
            Self(self.0-1, self.1 + i)
        }
    
        fn level(&self) -> Level {
            self.0 - 1
        }
    }
    
    #[test]
    fn frontier1() {
        let mut dd = ZddMut::new();
        let s = Combi(5,0);
        let headers = (0..10).into_iter().map(|i| dd.header(i, &format!("x{}", i))).collect::<Vec<_>>();
        let mut f = ZddFrontierBuilder::new(&headers);
        let root = f.build(&s);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            root.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);

        let root = dd.reduce(&root);

        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            root.dot(&mut io);
        }
        let s = std::str::from_utf8(&buf).unwrap();
        println!("{}", s);
    }
}
