use mddcore::prelude::*;
use crate::mss::MddNode;
use std::collections::{HashMap, HashSet};

enum MddStackValue {
    Node(Option<Level>, NodeId),
    Push(String, usize),
    Pop(String),
}

pub struct MddPath<V> {
    next_stack: Vec<MddStackValue>,
    path: HashMap<String,usize>,
    node: MddNode<V>,
    labels: Vec<String>,
    edges: Vec<usize>,
    ss: HashSet<V>,
}

impl<V> MddPath<V>
where
    V: MddValue,
{
    pub fn new(node: &MddNode<V>, ss: &HashSet<V>) -> Self {
        let mut next_stack = Vec::new();
        let level = node.get_level();
        next_stack.push(MddStackValue::Node(level, node.get_id()));
        let dd = node.get_mgr();
        let (nheaders, _, _, _) = dd.borrow().size();
        let mut labels = vec![String::new(); nheaders];
        let mut edges = vec![0; nheaders];
        match node.get_node() {
            Node::Value(_fnode) => {
                for hid in 0..nheaders {
                    let i = dd.borrow().mtmdd().get_header(&hid).unwrap().level();
                    let value = dd.borrow().mtmdd().get_header(&hid).unwrap().label().to_string();
                    let e = dd.borrow().mtmdd().get_header(&hid).unwrap().edge_num();
                    labels[i] = value;
                    edges[i] = e;
                }        
            }
            Node::Bool(_fnode) => {
                for hid in 0..nheaders {
                    let i = dd.borrow().mdd().get_header(&hid).unwrap().level();
                    let value = dd.borrow().mdd().get_header(&hid).unwrap().label().to_string();
                    let e = dd.borrow().mdd().get_header(&hid).unwrap().edge_num();
                    labels[i] = value;
                    edges[i] = e;
                }        
            }
        }
        MddPath {
            next_stack,
            path: HashMap::new(),
            node: node.clone(),
            labels,
            edges,
            ss: ss.clone(),
        }
    }

    pub fn len(&self) -> u64 {
        self.node.mdd_count(&self.ss)
    }
}

impl<V> Iterator for MddPath<V>
where
    V: MddValue,
{
    type Item = HashMap<String,usize>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.node.get_node() {
            Node::Value(_fnode) => {
                self.vnext()
            }
            Node::Bool(_fnode) => {
                self.bnext()
            }
        }
    }
}

impl<V> MddPath<V>
where
    V: MddValue,
{
    fn vnext(&mut self) -> Option<HashMap<String,usize>> {
        let dd = self.node.get_mgr();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                MddStackValue::Node(level, nodeid) => {
                    let current_level = dd.borrow().mtmdd().level(&nodeid);
                    match dd.borrow().mtmdd().get_node(&nodeid).unwrap() {
                        mtmdd::Node::NonTerminal(_) | mtmdd::Node::Terminal(_) if level > current_level => {
                            let label = &self.labels[level.unwrap()];
                            let nedge = self.edges[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            for i in (0..nedge).rev() {
                                self.next_stack.push(MddStackValue::Pop(label.to_string()));
                                self.next_stack.push(MddStackValue::Node(level, nodeid));
                                self.next_stack.push(MddStackValue::Push(label.to_string(), i));
                            }
                        }
                        mtmdd::Node::NonTerminal(fnode) => {
                            let label = &self.labels[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            let fnodeid = fnode.iter().cloned().collect::<Vec<_>>();
                            for (i, x) in fnodeid.into_iter().enumerate().rev() {
                                self.next_stack.push(MddStackValue::Pop(label.to_string()));
                                self.next_stack.push(MddStackValue::Node(level, x));
                                self.next_stack.push(MddStackValue::Push(label.to_string(), i));
                            }
                        }
                        mtmdd::Node::Terminal(fnode) => {
                            let value = fnode.value();
                            if self.ss.contains(&value) {
                                let result = self.path.clone();
                                return Some(result);
                            }
                        }
                        _ => (),
                    }
                }
                MddStackValue::Push(x, i) => {
                    self.path.insert(x, i);
                }
                MddStackValue::Pop(x) => {
                    self.path.remove(&x);
                }
            }
        }
        None
    }

    fn bnext(&mut self) -> Option<HashMap<String,usize>> {
        let dd = self.node.get_mgr();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                MddStackValue::Node(level, nodeid) => {
                    let current_level = dd.borrow().mdd().level(&nodeid);
                    match dd.borrow().mdd().get_node(&nodeid).unwrap() {
                        mdd::Node::NonTerminal(_) | mdd::Node::One | mdd::Node::Zero if level > current_level => {
                            let label = &self.labels[level.unwrap()];
                            let nedge = self.edges[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            for i in (0..nedge).rev() {
                                self.next_stack.push(MddStackValue::Pop(label.to_string()));
                                self.next_stack.push(MddStackValue::Node(level, nodeid));
                                self.next_stack.push(MddStackValue::Push(label.to_string(), i));
                            }
                        }
                        mdd::Node::NonTerminal(fnode) => {
                            let label = &self.labels[level.unwrap()];
                            let level = level.and_then(|x| x.checked_sub(1));
                            let fnodeid = fnode.iter().cloned().collect::<Vec<_>>();
                            for (i, x) in fnodeid.into_iter().enumerate().rev() {
                                self.next_stack.push(MddStackValue::Pop(label.to_string()));
                                self.next_stack.push(MddStackValue::Node(level, x));
                                self.next_stack.push(MddStackValue::Push(label.to_string(), i));
                            }
                        }
                        mdd::Node::Zero => {
                            if self.ss.contains(&V::from(0)) {
                                let result = self.path.clone();
                                return Some(result);
                            }
                        }
                        mdd::Node::One => {
                            if self.ss.contains(&V::from(1)) {
                                let result = self.path.clone();
                                return Some(result);
                            }
                        }
                        _ => (),
                    }
                }
                MddStackValue::Push(x, i) => {
                    self.path.insert(x, i);
                }
                MddStackValue::Pop(x) => {
                    self.path.remove(&x);
                }
            }
        }
        None
    }
}

enum ZMddStackValue {
    Node(NodeId),
    Push(String, usize),
    Pop(String),
}

pub struct ZMddPath<V> {
    next_stack: Vec<ZMddStackValue>,
    path: HashMap<String,usize>,
    node: MddNode<V>,
    ss: HashSet<V>,
}

impl<V> ZMddPath<V>
where
    V: MddValue,
{
    pub fn new(node: &MddNode<V>, ss: &HashSet<V>) -> Self {
        let mut next_stack = Vec::new();
        next_stack.push(ZMddStackValue::Node(node.get_id()));
        ZMddPath {
            next_stack,
            path: HashMap::new(),
            node: node.clone(),
            ss: ss.clone(),
        }
    }

    pub fn len(&self) -> u64 {
        self.node.zmdd_count(&self.ss)
    }
}

impl<V> Iterator for ZMddPath<V>
where
    V: MddValue,
{
    type Item = HashMap<String,usize>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.node.get_node() {
            Node::Value(_fnode) => {
                self.vnext()
            }
            Node::Bool(_fnode) => {
                self.bnext()
            }
        }
    }
}

impl<V> ZMddPath<V>
where
    V: MddValue,
{
    fn vnext(&mut self) -> Option<HashMap<String,usize>> {
        let dd = self.node.get_mgr();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                ZMddStackValue::Node(node) => match dd.borrow().mtmdd().get_node(&node).unwrap() {
                    mtmdd::Node::Terminal(fnode) => {
                        let value = fnode.value();
                        if self.ss.contains(&value) {
                            let result = self.path.clone();
                            return Some(result);
                        }
                    }
                    mtmdd::Node::NonTerminal(fnode) => {
                        let label = dd.borrow().mtmdd().label(&node).unwrap().to_string();
                        let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                        for (i, x) in fnodeid.into_iter().enumerate().rev() {
                            self.next_stack.push(ZMddStackValue::Pop(label.to_string()));
                            self.next_stack.push(ZMddStackValue::Node(x));
                            self.next_stack.push(ZMddStackValue::Push(label.to_string(), i))
                        }
                    }
                    mtmdd::Node::Undet => (),
                },
                ZMddStackValue::Push(x, i) => {
                    self.path.insert(x, i);
                }
                ZMddStackValue::Pop(x) => {
                    self.path.remove(&x);
                }
            }
        }
        None
    }

    fn bnext(&mut self) -> Option<HashMap<String,usize>> {
        let dd = self.node.get_mgr();
        while let Some(stackvalue) = self.next_stack.pop() {
            match stackvalue {
                ZMddStackValue::Node(node) => match dd.borrow().mdd().get_node(&node).unwrap() {
                    mdd::Node::Zero => {
                        if self.ss.contains(&V::from(0)) {
                            let result = self.path.clone();
                            return Some(result);
                        }
                    }
                    mdd::Node::One => {
                        if self.ss.contains(&V::from(1)) {
                            let result = self.path.clone();
                            return Some(result);
                        }
                    }
                    mdd::Node::NonTerminal(fnode) => {
                        let label = dd.borrow().mdd().label(&node).unwrap().to_string();
                        let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                        for (i, x) in fnodeid.into_iter().enumerate().rev() {
                            self.next_stack.push(ZMddStackValue::Pop(label.to_string()));
                            self.next_stack.push(ZMddStackValue::Node(x));
                            self.next_stack.push(ZMddStackValue::Push(label.to_string(), i))
                        }
                    }
                    mdd::Node::Undet => (),
                },
                ZMddStackValue::Push(x, i) => {
                    self.path.insert(x, i);
                }
                ZMddStackValue::Pop(x) => {
                    self.path.remove(&x);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mss::MddMgr;
    use crate::mss::MddNode;

    fn create_mdd() -> (MddNode<i32>, MddMgr<i32>) {
        let mut mgr = MddMgr::new();
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3);
        vars.insert("y".to_string(), 3);
        vars.insert("z".to_string(), 3);
        let tmp = mgr.rpn("x y + z *", &vars).unwrap();
        (tmp, mgr)
    }

    #[test]
    fn test_zmdd_count() {
        let (node, mgr) = create_mdd();
        println!("{}", node.dot());
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        let path = ZMddPath::new(&node, &ss);
        for p in path {
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_zmdd_count2() {
        let (node, mgr) = create_mdd();
        let v = mgr.value(1);
        let node = node.eq(&v);
        println!("{}", node.dot());
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        let path = ZMddPath::new(&node, &ss);
        for p in path {
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_mdd_count() {
        let (node, mgr) = create_mdd();
        println!("{}", node.dot());
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        let path = MddPath::new(&node, &ss);
        for p in path {
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_mdd_count2() {
        let (node, mgr) = create_mdd();
        let v = mgr.value(1);
        let node = node.eq(&v);
        println!("{}", node.dot());
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        let path = MddPath::new(&node, &ss);
        for p in path {
            println!("{:?}", p);
        }
    }
}