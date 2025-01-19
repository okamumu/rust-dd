use mddcore::prelude::*;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Mul, Sub};
use std::rc::{Rc, Weak};

use crate::mdd_prob;
use crate::mdd_minsol;
use crate::mdd_count;
use crate::mdd_path::MddPath;
use crate::mdd_path::ZMddPath;

pub struct MddMgr<V> {
    mdd: Rc<RefCell<MtMdd2Manager<V>>>,
    vars: HashMap<String, MddNode<V>>,
}

#[derive(Clone, Debug)]
pub struct MddNode<V> {
    parent: Weak<RefCell<MtMdd2Manager<V>>>,
    node: Node,
}

impl<V> MddNode<V>
where
    V: MddValue,
{
    fn new(parent: &Rc<RefCell<MtMdd2Manager<V>>>, node: Node) -> Self {
        MddNode {
            parent: Rc::downgrade(&parent),
            node,
        }
    }
}

impl<V> MddMgr<V>
where
    V: MddValue,
{
    pub fn new() -> Self {
        MddMgr {
            mdd: Rc::new(RefCell::new(MtMdd2Manager::new())),
            vars: HashMap::new(),
        }
    }

    pub fn size(&self) -> (usize, usize, usize, usize) {
        self.mdd.borrow().size()
    }

    pub fn boolean(&self, other: bool) -> MddNode<V> {
        let mdd = self.mdd.borrow();
        if other {
            MddNode::new(&self.mdd, mdd.one())
        } else {
            MddNode::new(&self.mdd, mdd.zero())
        }
    }

    pub fn value(&self, value: V) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let node = mdd.value(value);
        MddNode::new(&self.mdd, node)
    }

    pub fn undet_boolean(&self) -> MddNode<V> {
        let mdd = self.mdd.borrow();
        MddNode::new(&self.mdd, mdd.undet_boolean())
    }

    pub fn undet_value(&self) -> MddNode<V> {
        let mdd = self.mdd.borrow();
        MddNode::new(&self.mdd, mdd.undet_value())
    }

    pub fn create_node(&self, h: HeaderId, nodes: &[MddNode<V>]) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let xs = nodes.iter().map(|x| x.node).collect::<Vec<_>>();
        let node = mdd.create_node(h, &xs);
        MddNode::new(&self.mdd, node)
    }

    pub fn defvar(&mut self, label: &str, range: usize) -> MddNode<V> {
        if let Some(node) = self.vars.get(label) {
            return node.clone();
        } else {
            let level = self.vars.len();
            let result = {
                let mut mdd = self.mdd.borrow_mut();
                let nodes = (0..range).map(|x| mdd.value(V::from(x as i32))).collect::<Vec<_>>();
                let h = mdd.create_header(level, label, range);
                let node = mdd.create_node(h, &nodes);
                MddNode::new(&self.mdd, node)
            };
            self.vars.insert(label.to_string(), result.clone());
            result
        }
    }

    pub fn get_varorder(&self) -> Vec<String> {
        let mut result = vec!["?".to_string(); self.vars.len()];
        for (k, v) in self.vars.iter() {
            let level = v.get_level().unwrap();
            result[level] = k.clone();
        }
        self.vars.keys().cloned().collect()
    }

    pub fn rpn(&mut self, rpn: &str, vars: &HashMap<String, usize>) -> Result<MddNode<V>, String> {
        let mut stack = Vec::new();
        let mut cache = HashMap::new();
        for token in rpn.split_whitespace() {
            match token {
                "+" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.add(a, b);
                    stack.push(tmp);
                }
                "-" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.sub(a, b);
                    stack.push(tmp);
                }
                "*" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.mul(a, b);
                    stack.push(tmp);
                }
                "/" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.div(a, b);
                    stack.push(tmp);
                }
                "min" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.min(a, b);
                    stack.push(tmp);
                }
                "max" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.max(a, b);
                    stack.push(tmp);
                }
                "==" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.eq(a, b);
                    stack.push(tmp);
                }
                "!=" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.neq(a, b);
                    stack.push(tmp);
                }
                "<" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.lt(a, b);
                    stack.push(tmp);
                }
                "<=" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.lte(a, b);
                    stack.push(tmp);
                }
                ">" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.gt(a, b);
                    stack.push(tmp);
                }
                ">=" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.gte(a, b);
                    stack.push(tmp);
                }
                "&&" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.and(a, b);
                    stack.push(tmp);
                }
                "||" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.or(a, b);
                    stack.push(tmp);
                }
                "!" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.not(a);
                    stack.push(tmp);
                }
                "?" => {
                    let mut mdd = self.mdd.borrow_mut();
                    let c = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let tmp = mdd.ite(a, b, c);
                    stack.push(tmp);
                }
                "True" => {
                    let node = {
                        let mdd = self.mdd.borrow();
                        mdd.one()
                    };
                    stack.push(node);
                }
                "False" => {
                    let node = {
                        let mdd = self.mdd.borrow();
                        mdd.zero()
                    };
                    stack.push(node);
                }
                _ if token.starts_with("save(") && token.ends_with(")") => {
                    let name = &token[5..token.len() - 1];
                    if let Some(node) = stack.last() {
                        cache.insert(name.to_string(), node.clone());
                    } else {
                        return Err("Stack is empty for save operation".to_string());
                    }
                }
                _ if token.starts_with("load(") && token.ends_with(")") => {
                    let name = &token[5..token.len() - 1];
                    if let Some(node) = cache.get(name) {
                        stack.push(node.clone());
                    } else {
                        return Err(format!("No cached value for {}", name));
                    }
                }
                _ => {
                    // parse whether it is a number or a variable
                    match token.parse::<i32>() {
                        Ok(val) => {
                            let node = {
                                let mut mdd = self.mdd.borrow_mut();
                                mdd.value(V::from(val))
                            };
                            stack.push(node);
                        }
                        Err(_) => match vars.get(token) {
                            Some(range) => {
                                let node = self.defvar(token, range.clone());
                                stack.push(node.node.clone());
                            }
                            None => panic!("Unknown variable: {}", token),
                        },
                    }
                }
            }
        }
        if stack.len() == 1 {
            Ok(MddNode::new(&self.mdd, stack.pop().unwrap()))
        } else {
            Err("Invalid expression".to_string())
        }
    }

    pub fn and(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let xs = nodes.iter().map(|x| &x.node).collect::<Vec<_>>();
        let mut result = mdd.one();
        for node in xs {
            result = mdd.and(result, *node);
        }
        MddNode::new(&self.mdd, result)
    }

    pub fn or(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let xs = nodes.iter().map(|x| &x.node).collect::<Vec<_>>();
        let mut result = mdd.zero();
        for node in xs {
            result = mdd.or(result, *node);
        }
        MddNode::new(&self.mdd, result)
    }

    pub fn min(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let xs = nodes.iter().map(|x| &x.node).collect::<Vec<_>>();
        let mut result = *xs[0];
        for &node in xs[1..].iter() {
            result = mdd.min(result, *node);
        }
        MddNode::new(&self.mdd, result)
    }

    pub fn max(&self, nodes: &[MddNode<V>]) -> MddNode<V> {
        let mut mdd = self.mdd.borrow_mut();
        let xs = nodes.iter().map(|x| &x.node).collect::<Vec<_>>();
        let mut result = *xs[0];
        for &node in xs[1..].iter() {
            result = mdd.max(result, *node);
        }
        MddNode::new(&self.mdd, result)
    }

    // pub fn not(&mut self, node: &MddNode<V>) -> MddNode<V> {
    //     let mut mdd = self.mdd.borrow_mut();
    //     let result = mdd.not(node.node);
    //     MddNode::new(&self.mdd, result)
    // }

    // pub fn ifelse(&mut self, cond: &MddNode<V>, then: &MddNode<V>, els: &MddNode<V>) -> MddNode<V> {
    //     let mut mdd = self.mdd.borrow_mut();
    //     let result = mdd.ite(cond.node, then.node, els.node);
    //     MddNode::new(&self.mdd, result)
    // }
}

impl<V> MddNode<V>
where
    V: MddValue,
{
    pub fn get_mgr(&self) -> Rc<RefCell<MtMdd2Manager<V>>> {
        self.parent.upgrade().unwrap()
    }

    pub fn get_node(&self) -> Node {
        self.node.clone()
    }

    pub fn get_id(&self) -> NodeId {
        match &self.node {
            Node::Value(x) => *x,
            Node::Bool(x) => *x,
        }
    }

    pub fn get_header(&self) -> Option<HeaderId> {
        match &self.node {
            Node::Value(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mtmdd().get_node(x)?;
                node.headerid()
            }
            Node::Bool(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mdd().get_node(x)?;
                node.headerid()
            }
        }
    }

    pub fn get_level(&self) -> Option<Level> {
        match &self.node {
            Node::Value(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mtmdd().get_node(x)?;
                let hid = node.headerid()?;
                let header = mdd.mtmdd().get_header(&hid)?;
                Some(header.level())
            }
            Node::Bool(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mdd().get_node(x)?;
                let hid = node.headerid()?;
                let header = mdd.mdd().get_header(&hid)?;
                Some(header.level())
            }
        }
    }

    pub fn get_label(&self) -> Option<String> {
        match &self.node {
            Node::Value(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mtmdd().get_node(x)?;
                let hid = node.headerid()?;
                let header = mdd.mtmdd().get_header(&hid)?;
                Some(header.label().to_string())
            }
            Node::Bool(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mdd().get_node(x)?;
                let hid = node.headerid()?;
                let header = mdd.mdd().get_header(&hid)?;
                Some(header.label().to_string())
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        match &self.node {
            Node::Value(x) => false,
            Node::Bool(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mdd().get_node(x).unwrap();
                match node {
                    mdd::Node::One => false,
                    mdd::Node::Zero => true,
                    mdd::Node::Undet => false,
                    _ => false,
                }
            }
        }
    }

    pub fn is_one(&self) -> bool {
        match &self.node {
            Node::Value(x) => false,
            Node::Bool(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mdd().get_node(x).unwrap();
                match node {
                    mdd::Node::One => true,
                    mdd::Node::Zero => false,
                    mdd::Node::Undet => false,
                    _ => false,
                }
            }
        }
    }

    pub fn is_undet(&self) -> bool {
        match &self.node {
            Node::Value(x) => false,
            Node::Bool(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mdd().get_node(x).unwrap();
                match node {
                    mdd::Node::One => false,
                    mdd::Node::Zero => false,
                    mdd::Node::Undet => true,
                    _ => false,
                }
            }
        }
    }

    pub fn value(&self) -> Option<V> {
        match &self.node {
            Node::Value(x) => {
                let mddmgr = self.parent.upgrade().unwrap();
                let mdd = mddmgr.borrow();
                let node = mdd.mtmdd().get_node(x).unwrap();
                match node {
                    mtmdd::Node::Terminal(fnode) => Some(fnode.value()),
                    _ => None,
                }
            }
            Node::Bool(x) => None,
        }
    }

    pub fn dot(&self) -> String {
        let mddmgr = self.parent.upgrade().unwrap();
        let mdd = mddmgr.borrow();
        mdd.dot_string(&self.node)
    }

    pub fn add(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.add(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn sub(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.sub(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn mul(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.mul(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn div(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.div(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn min(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.min(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn max(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.max(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn eq(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.eq(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn ne(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.neq(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn lt(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.lt(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn le(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.lte(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn gt(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.gt(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn ge(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.gte(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn and(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.and(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn or(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.or(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn xor(&self, other: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.xor(self.node, other.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn not(&self) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.not(self.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn ite(&self, then: &MddNode<V>, els: &MddNode<V>) -> MddNode<V> {
        let mddmgr = self.parent.upgrade().unwrap();
        let mut mdd = mddmgr.borrow_mut();
        let node = mdd.ite(self.node, then.node, els.node);
        MddNode::new(&mddmgr, node)
    }

    pub fn prob<T>(&mut self, pv: &HashMap<String, Vec<T>>, ss: &[V]) -> T
    where
        T: Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Clone
            + Copy
            + PartialEq
            + From<f64>,
    {
        let mgr = self.parent.upgrade().unwrap();
        let mut mdd = mgr.borrow_mut();
        let hashset: HashSet<V> = ss.iter().cloned().collect();
        mdd_prob::prob(&mut mdd, &self.node, pv, &hashset)
    }

    pub fn minpath(&mut self) -> MddNode<V> {
        let mgr = self.parent.upgrade().unwrap();
        let mut mdd = mgr.borrow_mut();
        let node = mdd_minsol::minsol(&mut mdd, &self.node);
        MddNode::new(&mgr, node)
    }

    pub fn mdd_count(&self, ss: &HashSet<V>) -> u64 {
        let mgr = self.parent.upgrade().unwrap();
        let mdd = mgr.borrow();
        mdd_count::mdd_count(&mdd, &self.node, ss)
    }

    pub fn zmdd_count(&self, ss: &HashSet<V>) -> u64 {
        let mgr = self.parent.upgrade().unwrap();
        let mdd = mgr.borrow();
        mdd_count::zmdd_count(&mdd, &self.node, ss)
    }

    pub fn mdd_extract(&self, ss: &HashSet<V>) -> MddPath<V> {
        MddPath::new(self, ss)
    }

    pub fn zmdd_extract(&self, ss: &HashSet<V>) -> ZMddPath<V> {
        ZMddPath::new(self, ss)
    }

    pub fn size(&self) -> (u64, u64, u64) {
        let mgr = self.parent.upgrade().unwrap();
        let mdd = mgr.borrow();
        mdd_count::mddnode_count(&mdd, &self.node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdd_mgr() {
        let mut mgr: MddMgr<i32> = MddMgr::new();
        let x = mgr.defvar("x", 3);
        let y = mgr.defvar("y", 3);
        let z = mgr.defvar("z", 3);
        // let zero = mgr.zero();
        // let one = mgr.one();
        // let two = mgr.val(2);
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3);
        vars.insert("y".to_string(), 3);
        vars.insert("z".to_string(), 3);
        let rpn = "x y z + *";
        if let Ok(node) = mgr.rpn(rpn, &vars) {
            println!("{}", node.dot());
        }
    }
}
