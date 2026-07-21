use bddcore::prelude::*;
use crate::zdd::ZddNode;

enum ZddStackValue {
    Node(NodeId),
    Push(String),
    Pop,
}

/// Enumerates the sets of a genuine [`ZddManager`] family as `Vec<String>` of labels
/// (the ZDD counterpart of [`bdd_path::BddPath`](crate::bdd_path::BddPath)).
///
/// At a non-terminal it descends the `edge(1)` branch with the variable's label pushed
/// (element present) and the `edge(0)` branch without it (element absent); the `One`
/// terminal emits the accumulated set.
pub struct ZddPath {
    next_stack: Vec<ZddStackValue>,
    path: Vec<String>,
    node: ZddNode,
    ss: Vec<bool>,
}

impl ZddPath {
    pub fn new(node: ZddNode, ss: &[bool]) -> Self {
        let ss = ss.to_vec();
        let mut next_stack = Vec::new();
        next_stack.push(ZddStackValue::Node(node.get_id()));
        ZddPath {
            next_stack,
            path: Vec::new(),
            node,
            ss,
        }
    }

    pub fn len(&self) -> u64 {
        self.node.count(&self.ss)
    }
}

impl Iterator for ZddPath {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let dd = self.node.get_mgr();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                ZddStackValue::Node(node) => match dd.borrow().get_node(&node).unwrap() {
                    Node::Zero => {
                        if self.ss.contains(&false) {
                            let mut result = self.path.clone();
                            result.reverse();
                            return Some(result);
                        }
                    }
                    Node::One => {
                        if self.ss.contains(&true) {
                            let mut result = self.path.clone();
                            result.reverse();
                            return Some(result);
                        }
                    }
                    Node::NonTerminal(fnode) => {
                        let x = dd.borrow().label(&node).unwrap().to_string();
                        self.next_stack.push(ZddStackValue::Pop);
                        self.next_stack.push(ZddStackValue::Node(fnode.edge(1)));
                        self.next_stack.push(ZddStackValue::Push(x));
                        self.next_stack.push(ZddStackValue::Node(fnode.edge(0)));
                    }
                    Node::Undet => (),
                },
                ZddStackValue::Push(x) => self.path.push(x),
                ZddStackValue::Pop => {
                    self.path.pop();
                }
            }
        }
        None
    }
}
