use bddcore::prelude::*;
use crate::bss::BddNode;

enum BddStackValue {
    Node(Option<Level>, NodeId),
    Push(String),
    Pop,
}

pub struct BddPath {
    next_stack: Vec<BddStackValue>,
    path: Vec<String>,
    node: BddNode,
    labels: Vec<String>,
    ss: Vec<bool>,
}

impl BddPath {
    pub fn new(node: BddNode, ss: &[bool]) -> Self {
        let ss = ss.iter().cloned().collect::<Vec<_>>();
        let mut next_stack = Vec::new();
        let level = node.get_level();
        next_stack.push(BddStackValue::Node(level, node.get_id()));

        let dd = node.get_mgr();
        let (nheaders, _, _) = dd.borrow().size();
        let mut labels = vec![String::new(); nheaders];
        for hid in 0..nheaders {
            let i = dd.borrow().get_header(&hid).unwrap().level();
            let value = dd.borrow().get_header(&hid).unwrap().label().to_string();
            labels[i] = value;
        }

        BddPath {
            next_stack,
            path: Vec::new(),
            node,
            labels,
            ss,
        }
    }

    pub fn len(&self) -> u64 {
        self.node.bdd_count(&self.ss)
    }
}

impl Iterator for BddPath {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let dd = self.node.get_mgr();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                BddStackValue::Node(level, nodeid) => {
                    let current_level = dd.borrow().level(&nodeid);
                    match dd.borrow().get_node(&nodeid).unwrap() {
                        Node::NonTerminal(fnode) if level == current_level => {
                            let x = &self.labels[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, fnode[1]));
                            self.next_stack.push(BddStackValue::Push(x.to_string()));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, fnode[0]));
                            self.next_stack.push(BddStackValue::Push(format!("~{}", x)));
                        }
                        Node::Zero if level == None => {
                            if self.ss.contains(&false) {
                                let mut result = self.path.clone();
                                result.reverse();
                                return Some(result);
                            }
                        }
                        Node::One if level == None => {
                            if self.ss.contains(&true) {
                                let mut result = self.path.clone();
                                result.reverse();
                                return Some(result);
                            }
                        }
                        Node::NonTerminal(_fnode) => {
                            let x = &self.labels[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, nodeid));
                            self.next_stack.push(BddStackValue::Push(x.to_string()));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, nodeid));
                            self.next_stack.push(BddStackValue::Push(format!("~{}", x)));
                        }
                        Node::Zero => {
                            let x = &self.labels[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, nodeid));
                            self.next_stack.push(BddStackValue::Push(x.to_string()));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, nodeid));
                            self.next_stack.push(BddStackValue::Push(format!("~{}", x)));
                        }
                        Node::One => {
                            let x = &self.labels[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, nodeid));
                            self.next_stack.push(BddStackValue::Push(x.to_string()));
                            self.next_stack.push(BddStackValue::Pop);
                            self.next_stack.push(BddStackValue::Node(level, nodeid));
                            self.next_stack.push(BddStackValue::Push(format!("~{}", x)));
                        }
                        Node::Undet => (),
                    }
                }
                BddStackValue::Push(x) => {
                    self.path.push(x);
                }
                BddStackValue::Pop => {
                    self.path.pop();
                }
            }
        }
        None
    }
}

enum ZddStackValue {
    Node(NodeId),
    Push(String),
    Pop,
}

pub struct ZddPath {
    next_stack: Vec<ZddStackValue>,
    path: Vec<String>,
    node: BddNode,
    ss: Vec<bool>,
}

impl ZddPath {
    pub fn new(node: BddNode, ss: &[bool]) -> Self {
        let ss = ss.iter().cloned().collect::<Vec<_>>();
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
        self.node.zdd_count(&self.ss)
    }
}

impl Iterator for ZddPath {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        // let dd = self.node.parent.upgrade().unwrap();
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
                        self.next_stack.push(ZddStackValue::Node(fnode[1]));
                        self.next_stack.push(ZddStackValue::Push(x));
                        self.next_stack.push(ZddStackValue::Node(fnode[0]));
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
