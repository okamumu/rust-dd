use core::panic;
use std::hash::Hash;

use crate::common::{HashMap, HashSet, HeaderId, Level, NodeId, TerminalNumberValue};

use crate::nodes::*;

use crate::mdd;
use crate::mtmdd;

use crate::dot::Dot;

type VNode<V> = mtmdd::Node<V>;
type BNode = mdd::Node;

#[derive(Debug, Clone, Copy)]
pub enum Node {
    Value(NodeId),
    Bool(NodeId),
}

#[derive(Debug)]
pub struct MtMdd2Manager<V> {
    mdd: mdd::MddManager,
    mtmdd: mtmdd::MtMddManager<V>,
    bcache: HashMap<(Operation, NodeId, NodeId), NodeId>,
    vcache: HashMap<(Operation, NodeId, NodeId), NodeId>,
}

impl<V> MtMdd2Manager<V>
where
    V: TerminalNumberValue,
{
    pub fn new() -> Self {
        Self {
            mdd: mdd::MddManager::new(),
            mtmdd: mtmdd::MtMddManager::new(),
            bcache: HashMap::default(),
            vcache: HashMap::default(),
        }
    }

    #[inline]
    pub fn mtmdd(&self) -> &mtmdd::MtMddManager<V> {
        &self.mtmdd
    }

    #[inline]
    pub fn mtmdd_mut(&mut self) -> &mut mtmdd::MtMddManager<V> {
        &mut self.mtmdd
    }

    #[inline]
    pub fn mdd(&self) -> &mdd::MddManager {
        &self.mdd
    }

    #[inline]
    pub fn mdd_mut(&mut self) -> &mut mdd::MddManager {
        &mut self.mdd
    }

    #[inline]
    pub fn size(&self) -> (usize, usize, usize, usize) {
        let (u1, _x1, y1, z1) = self.mtmdd.size();
        let (_x2, y2, z2) = self.mdd.size();
        self.mtmdd.size()
    }

    #[inline]
    pub fn one(&self) -> Node {
        Node::Bool(self.mdd.one())
    }

    #[inline]
    pub fn zero(&self) -> Node {
        Node::Bool(self.mdd.zero())
    }

    #[inline]
    pub fn value(&mut self, value: V) -> Node {
        Node::Value(self.mtmdd.value(value))
    }

    #[inline]
    pub fn create_header(&mut self, level: Level, label: &str, edge_num: usize) -> HeaderId {
        let h1 = self.mtmdd.create_header(level, label, edge_num);
        let h2 = self.mdd.create_header(level, label, edge_num);
        assert_eq!(h1, h2);
        h1
    }

    pub fn create_node(&mut self, h: HeaderId, nodes: &[Node]) -> Node {
        let elem: Vec<NodeId> = nodes
            .iter()
            .map(|x| match x {
                Node::Value(f) => *f,
                Node::Bool(f) => *f,
            })
            .collect();
        match nodes[0] {
            Node::Value(_) => Node::Value(self.mtmdd.create_node(h, &elem)),
            Node::Bool(_) => Node::Bool(self.mdd.create_node(h, &elem)),
        }
    }

}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Operation {
    Eq,
    Lt,
    // LtE,
    // Gt,
    // GtE,
    If,
}

impl<V> MtMdd2Manager<V>
where
    V: TerminalNumberValue,
{
    pub fn and(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.and(fnode, gnode)),
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn or(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.or(fnode, gnode)),
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn xor(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.xor(fnode, gnode)),
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn not(&mut self, f: Node) -> Node {
        match f {
            Node::Bool(fnode) => Node::Bool(self.mdd.not(fnode)),
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn add(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.add(fnode, gnode)),
            _ => Node::Value(self.mtmdd.undet()),
        }
    }

    pub fn sub(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.sub(fnode, gnode)),
            _ => Node::Value(self.mtmdd.undet()),
        }
    }

    pub fn mul(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.mul(fnode, gnode)),
            _ => Node::Value(self.mtmdd.undet()),
        }
    }

    pub fn div(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.div(fnode, gnode)),
            _ => Node::Value(self.mtmdd.undet()),
        }
    }

    pub fn max(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.max(fnode, gnode)),
            _ => Node::Value(self.mtmdd.undet()),
        }
    }

    pub fn min(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd.min(fnode, gnode)),
            _ => Node::Value(self.mtmdd.undet()),
        }
    }
}

impl<V> MtMdd2Manager<V>
where
    V: TerminalNumberValue,
{
    pub fn eq(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => {
                let tmp = self.mdd.xor(fnode, gnode);
                Node::Bool(self.mdd.not(tmp))
            }
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.veq(fnode, gnode)),
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn veq(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Eq, f, g);
        if let Some(x) = self.bcache.get(&key) {
            return *x;
        }
        let node = match (
            self.mtmdd.get_node(f).unwrap(),
            self.mtmdd.get_node(g).unwrap(),
        ) {
            (VNode::Undet, _) => self.mdd.zero(),
            (_, VNode::Undet) => self.mdd.zero(),
            (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() == gnode.value() => {
                self.mdd.one()
            }
            (VNode::Terminal(_), VNode::Terminal(_)) => self.mdd.zero(),
            (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.veq(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.veq(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::NonTerminal(_gnode))
                if self.mtmdd.level(f) > self.mtmdd.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.veq(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(_fnode), VNode::NonTerminal(gnode))
                if self.mtmdd.level(f) < self.mtmdd.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.veq(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid
                    .into_iter()
                    .zip(gnodeid.into_iter())
                    .map(|(f, g)| self.veq(f, g))
                    .collect();
                self.mdd.create_node(headerid, &nodes)
            }
        };
        self.bcache.insert(key, node);
        node
    }

    pub fn neq(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd.xor(fnode, gnode)),
            (Node::Value(fnode), Node::Value(gnode)) => {
                let tmp = self.veq(fnode, gnode);
                Node::Bool(self.mdd.not(tmp))
            }
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn lt(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.vlt(fnode, gnode)),
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn vlt(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Lt, f, g);
        if let Some(x) = self.bcache.get(&key) {
            return *x;
        }
        let node = match (
            self.mtmdd.get_node(f).unwrap(),
            self.mtmdd.get_node(g).unwrap(),
        ) {
            (VNode::Undet, _) => self.mdd.zero(),
            (_, VNode::Undet) => self.mdd.zero(),
            (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() < gnode.value() => {
                self.mdd.one()
            }
            (VNode::Terminal(_fnode), VNode::Terminal(_gnode)) => self.mdd.zero(),
            (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.vlt(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vlt(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::NonTerminal(_gnode))
                if self.mtmdd.level(f) > self.mtmdd.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vlt(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(_fnode), VNode::NonTerminal(gnode))
                if self.mtmdd.level(f) < self.mtmdd.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.vlt(f, g)).collect();
                self.mdd.create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid
                    .into_iter()
                    .zip(gnodeid.into_iter())
                    .map(|(f, g)| self.vlt(f, g))
                    .collect();
                self.mdd.create_node(headerid, &nodes)
            }
        };
        self.bcache.insert(key, node);
        node
    }

    pub fn lte(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => {
                let resulteq = self.veq(fnode, gnode);
                if resulteq == self.mdd.one() {
                    return Node::Bool(self.mdd.one());
                }
                let resultlt = self.vlt(fnode, gnode);
                Node::Bool(resultlt)
            }
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn gt(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => {
                let tmp = self.vlt(gnode, fnode);
                Node::Bool(tmp)
            }
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn gte(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => {
                let resultlt = self.vlt(fnode, gnode);
                let result = self.mdd.not(resultlt);
                Node::Bool(result)
            }
            _ => Node::Bool(self.mdd.undet()),
        }
    }

    pub fn ite(&mut self, f: Node, g: Node, h: Node) -> Node {
        match (f, g, h) {
            (Node::Bool(fnode), Node::Value(gnode), Node::Value(hnode)) => {
                let barf = self.mdd.not(fnode);
                let vif = self.vif(fnode, gnode);
                let barvif = self.vif(barf, hnode);
                let result = self.mtmdd.replace(vif, barvif);
                Node::Value(result)
            }
            (Node::Bool(fnode), Node::Bool(gnode), Node::Bool(hnode)) => {
                let result = self.mdd.ite(fnode, gnode, hnode);
                Node::Bool(result)
            }
            (_, Node::Value(_gnode), Node::Value(_hnode)) => {
                let result = self.mtmdd.undet();
                Node::Value(result)
            }
            (_, Node::Bool(_gnode), Node::Bool(_hnode)) => {
                let result = self.mdd.undet();
                Node::Bool(result)
            }
            _ => panic!("ite: unexpected pattern."),
        }
    }

    pub fn vif(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::If, f, g);
        if let Some(x) = self.vcache.get(&key) {
            return *x;
        }
        let node = match (
            self.mdd.get_node(f).unwrap(),
            self.mtmdd.get_node(g).unwrap(),
        ) {
            (BNode::Undet, _) => self.mtmdd.undet(),
            (_, VNode::Undet) => self.mtmdd.undet(),
            (BNode::Zero, _) => self.mtmdd.undet(),
            (BNode::One, _) => g,
            (BNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vif(f, g)).collect();
                self.mtmdd.create_node(headerid, &nodes)
            }
            (BNode::NonTerminal(fnode), VNode::NonTerminal(_gnode))
                if self.mdd.level(f) > self.mtmdd.level(g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vif(f, g)).collect();
                self.mtmdd.create_node(headerid, &nodes)
            }
            (BNode::NonTerminal(_fnode), VNode::NonTerminal(gnode))
                if self.mdd.level(f) < self.mtmdd.level(g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.vif(f, g)).collect();
                self.mtmdd.create_node(headerid, &nodes)
            }
            (BNode::NonTerminal(fnode), VNode::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().map(|&f| f).collect();
                let gnodeid: Vec<_> = gnode.iter().map(|&g| g).collect();
                let nodes: Vec<_> = fnodeid
                    .into_iter()
                    .zip(gnodeid.into_iter())
                    .map(|(f, g)| self.vif(f, g))
                    .collect();
                self.mtmdd.create_node(headerid, &nodes)
            }
        };
        self.vcache.insert(key, node);
        node
    }
}

// impl<V> Gc for MtMdd2<V> where V: TerminalNumberValue {
//     type Node = Node<V>;

//     fn clear_cache(&mut self) {
//         self.mdd.clear_cache();
//         self.mtmdd.clear_cache();
//     }

//     fn clear_table(&mut self) {
//         self.mdd.clear_table();
//         self.mtmdd.clear_table();
//     }

//     fn gc_impl(&mut self, f: &Self::Node, _visited: &mut HashSet<Self::Node>) {
//         match f {
//             Node::Bool(bnode) => {
//                 let mut visited: HashSet<BNode> = HashSet::default();
//                 self.mdd.gc_impl(bnode, &mut visited)
//             },
//             Node::Value(vnode) => {
//                 let mut visited: HashSet<VNode<V>> = HashSet::default();
//                 self.mtmdd.gc_impl(vnode, &mut visited)
//             },
//             _ => ()
//         }
//     }
// }

impl<V> Dot for MtMdd2Manager<V>
where
    V: TerminalNumberValue,
{
    type Node = Node;

    fn dot_impl<T>(&self, io: &mut T, node: Node, visited: &mut HashSet<NodeId>)
    where
        T: std::io::Write,
    {
        match node {
            Node::Value(f) => self.mtmdd.dot_impl(io, f, visited),
            Node::Bool(f) => self.mdd.dot_impl(io, f, visited),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
    Not,
    Xor,
    IfElse,
    Value(Node),
}

pub fn build_from_rpn<V>(dd: &mut MtMdd2Manager<V>, tokens: &[Token]) -> Result<Node, String>
    where V: TerminalNumberValue + std::str::FromStr
{
    let mut stack: Vec<Node> = Vec::new();

    for token in tokens {
        match token {
            Token::Add => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.add(a, b);
                stack.push(tmp);
            }
            Token::Sub => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.sub(a, b);
                stack.push(tmp);
            }
            Token::Mul => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.mul(a, b);
                stack.push(tmp);
            }
            Token::Div => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.div(a, b);
                stack.push(tmp);
            }
            Token::Eq => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.eq(a, b);
                stack.push(tmp);
            }
            Token::Neq => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.neq(a, b);
                stack.push(tmp);
            }
            Token::Lt => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.lt(a, b);
                stack.push(tmp);
            }
            Token::Lte => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.lte(a, b);
                stack.push(tmp);
            }
            Token::Gt => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.gt(a, b);
                stack.push(tmp);
            }
            Token::Gte => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.gte(a, b);
                stack.push(tmp);
            }
            Token::And => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.and(a, b);
                stack.push(tmp);
            }
            Token::Or => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.or(a, b);
                stack.push(tmp);
            }
            Token::Xor => {
                let b = stack.pop().ok_or("Stack underflow")?;
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.xor(a, b);
                stack.push(tmp);
            }
            Token::Not => {
                let a = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.not(a);
                stack.push(tmp);
            }
            Token::IfElse => {
                let else_branch = stack.pop().ok_or("Stack underflow")?;
                let then_branch = stack.pop().ok_or("Stack underflow")?;
                let condition = stack.pop().ok_or("Stack underflow")?;
                let tmp = dd.ite(condition, then_branch, else_branch);
                stack.push(tmp);
            }
            Token::Value(node) => stack.push(node.clone()),
        }
    }
    if let Some(result) = stack.pop() {
        Ok(result)
    } else {
        Err("The expression is invalid.".to_string())
    }
}

pub fn gen_var<T>(dd: &mut MtMdd2Manager<T>, label: &str, level: usize, range: &[T]) -> Node
where
    T: TerminalNumberValue,
{
    let count = range.len();
    let htmp = dd.create_header(level, label, count);
    let tmp = range.iter().map(|&i| dd.value(i)).collect::<Vec<_>>();
    dd.create_node(htmp, &tmp)
}

#[macro_export]
macro_rules! build_from_rpn {
    ($dd:ident, $($token:tt)*) => {{
        let tokens = vec![
            $(rpn_token!($dd, $token)),*
        ];
        build_from_rpn(&mut $dd, &tokens)
    }};
}

#[macro_export]
macro_rules! rpn_token {
    ($dd:ident, +) => {
        Token::Add
    };
    ($dd:ident, -) => {
        Token::Sub
    };
    ($dd:ident, *) => {
        Token::Mul
    };
    ($dd:ident, /) => {
        Token::Div
    };
    ($dd:ident, ==) => {
        Token::Eq
    };
    ($dd:ident, !=) => {
        Token::Neq
    };
    ($dd:ident, <=) => {
        Token::Lte
    };
    ($dd:ident, >=) => {
        Token::Gte
    };
    ($dd:ident, &&) => {
        Token::And
    };
    ($dd:ident, ||) => {
        Token::Or
    };
    ($dd:ident, ?) => {
        Token::IfElse
    };
    ($dd:ident, $value:literal) => {
        Token::Value($dd.value($value))
    };
    ($dd:ident, $ident:expr) => {
        Token::Value($ident.clone())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // impl Drop for Node {
    //     fn drop(&mut self) {
    //         println!("Dropping Node{}", self.id());
    //     }
    // }

    #[test]
    fn test_create_node() {
        let mut dd = MtMdd2Manager::new();
        let h1 = dd.create_header(0, "x", 2);
        let h2 = dd.create_header(1, "y", 2);
        let v0 = dd.value(0);
        let v1 = dd.value(1);
        let x = dd.create_node(h1, &[v0, v1]);
        let y = dd.create_node(h2, &[x, v1]);
        println!("{}", dd.dot_string(y));
    }

    #[test]
    fn test_create_node2() {
        let mut dd = MtMdd2Manager::new();
        let x = gen_var(&mut dd, "x", 0, &[0,1,2,3,4,5]);
        let y = gen_var(&mut dd, "y", 1, &[0,1,2,3,4,5]);
        let z = dd.add(x, y);
        println!("{}", dd.dot_string(z));
    }

    #[test]
    fn test_eq() {
        let mut dd = MtMdd2Manager::new();
        let x = gen_var(&mut dd, "x", 0, &[0,1,2]);
        let y = gen_var(&mut dd, "y", 1, &[0,1,2]);
        let z = gen_var(&mut dd, "z", 2, &[0,1,2]);
        let f = dd.add(x, y);
        let g = dd.sub(z, x);
        let h = dd.eq(f, g);
        println!("{}", dd.dot_string(h));
    }

    #[test]
    fn test_ite() {
        let mut dd = MtMdd2Manager::new();
        let x = gen_var(&mut dd, "x", 0, &[0,1,2]);
        let y = gen_var(&mut dd, "y", 1, &[0,1,2]);
        let z = gen_var(&mut dd, "z", 2, &[0,1,2]);
        let f = dd.add(x, y);
        let g = dd.eq(f, z);    
        let g = dd.ite(g, x, z);
        println!("{}", dd.dot_string(g));
    }

    #[test]
    fn test_build_rpn() {
        // case(x + y <= 5 => x, x + y >= 3 => y, _ => x), 0 <= x <= 5, 0 <= y <= 5
        let mut dd = MtMdd2Manager::new();
        let x = gen_var(&mut dd, "x", 0, &[0,1,2,3,4,5]);
        let y = gen_var(&mut dd, "y", 1, &[0,1,2,3,4,5]);
        // x y + 5 <= x x y + 3 >= y x ? ?
        let tokens = vec![
            Token::Value(x),
            Token::Value(y),
            Token::Add,
            Token::Value(dd.value(5)),
            Token::Lte,
            Token::Value(x),
            Token::Value(x),
            Token::Value(y),
            Token::Add,
            Token::Value(dd.value(3)),
            Token::Gte,
            Token::Value(y),
            Token::Value(x),
            Token::IfElse,
            Token::IfElse,
        ];
        let res = build_from_rpn(&mut dd, &tokens);
        match res {
            Ok(res) => {
                println!("{}", dd.dot_string(res))
            },
            Err(e) => {
                println!("{}", e)
            }
        }
    }

    #[test]
    fn test_ope6() {
        // case(x + y <= 5 => x, x + y >= 3 => y, _ => x), 0 <= x <= 5, 0 <= y <= 5
        let mut dd = MtMdd2Manager::new();
        let x = gen_var(&mut dd, "x", 1, &[0,1,2,3,4,5]);
        let y = gen_var(&mut dd, "y", 2, &[0,1,2,3,4,5]);
        let res = build_from_rpn!{dd, x y + 5 <= x x y + 3 >= y x ? ?};
        match res {
            Ok(res) => {
                println!("{}", dd.dot_string(res))
            },
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}
