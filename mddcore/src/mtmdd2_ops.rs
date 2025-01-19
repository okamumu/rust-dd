use common::prelude::*;
use crate::nodes::*;
use crate::mdd;
use crate::mtmdd;
use crate::mtmdd2::*;

type VNode<V> = mtmdd::Node<V>;
type BNode = mdd::Node;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MtMdd2Operation {
    Eq,
    Lt,
    // LtE,
    // Gt,
    // GtE,
    If,
}

impl<V> MtMdd2Manager<V>
where
    V: MddValue,
{
    pub fn and(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd_mut().and(fnode, gnode)),
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn or(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd_mut().or(fnode, gnode)),
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn xor(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd_mut().xor(fnode, gnode)),
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn not(&mut self, f: Node) -> Node {
        match f {
            Node::Bool(fnode) => Node::Bool(self.mdd_mut().not(fnode)),
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn add(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd_mut().add(fnode, gnode)),
            _ => Node::Value(self.mtmdd().undet()),
        }
    }

    pub fn sub(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd_mut().sub(fnode, gnode)),
            _ => Node::Value(self.mtmdd().undet()),
        }
    }

    pub fn mul(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd_mut().mul(fnode, gnode)),
            _ => Node::Value(self.mtmdd().undet()),
        }
    }

    pub fn div(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd_mut().div(fnode, gnode)),
            _ => Node::Value(self.mtmdd().undet()),
        }
    }

    pub fn max(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd_mut().max(fnode, gnode)),
            _ => Node::Value(self.mtmdd().undet()),
        }
    }

    pub fn min(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Value(self.mtmdd_mut().min(fnode, gnode)),
            _ => Node::Value(self.mtmdd().undet()),
        }
    }
}

impl<V> MtMdd2Manager<V>
where
    V: MddValue,
{
    pub fn eq(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => {
                let tmp = self.mdd_mut().xor(fnode, gnode);
                Node::Bool(self.mdd_mut().not(tmp))
            }
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.veq(fnode, gnode)),
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn veq(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMdd2Operation::Eq, f, g);
        if let Some(x) = self.get_bcache().get(&key) {
            return *x;
        }
        let node = match (
            self.mtmdd().get_node(&f).unwrap(),
            self.mtmdd().get_node(&g).unwrap(),
        ) {
            (VNode::Undet, _) => self.mdd().zero(),
            (_, VNode::Undet) => self.mdd().zero(),
            (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() == gnode.value() => {
                self.mdd().one()
            }
            (VNode::Terminal(_), VNode::Terminal(_)) => self.mdd().zero(),
            (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.veq(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.veq(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::NonTerminal(_gnode))
                if self.mtmdd().level(&f) > self.mtmdd().level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.veq(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(_fnode), VNode::NonTerminal(gnode))
                if self.mtmdd().level(&f) < self.mtmdd().level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.veq(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
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
                self.mdd_mut().create_node(headerid, &nodes)
            }
        };
        self.get_mut_bcache().insert(key, node);
        node
    }

    pub fn neq(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Bool(fnode), Node::Bool(gnode)) => Node::Bool(self.mdd_mut().xor(fnode, gnode)),
            (Node::Value(fnode), Node::Value(gnode)) => {
                let tmp = self.veq(fnode, gnode);
                Node::Bool(self.mdd_mut().not(tmp))
            }
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn lt(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => Node::Bool(self.vlt(fnode, gnode)),
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn vlt(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMdd2Operation::Lt, f, g);
        if let Some(x) = self.get_bcache().get(&key) {
            return *x;
        }
        let node = match (
            self.mtmdd().get_node(&f).unwrap(),
            self.mtmdd().get_node(&g).unwrap(),
        ) {
            (VNode::Undet, _) => self.mdd().zero(),
            (_, VNode::Undet) => self.mdd().zero(),
            (VNode::Terminal(fnode), VNode::Terminal(gnode)) if fnode.value() < gnode.value() => {
                self.mdd().one()
            }
            (VNode::Terminal(_fnode), VNode::Terminal(_gnode)) => self.mdd().zero(),
            (VNode::Terminal(_fnode), VNode::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.vlt(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vlt(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(fnode), VNode::NonTerminal(_gnode))
                if self.mtmdd().level(&f) > self.mtmdd().level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vlt(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
            }
            (VNode::NonTerminal(_fnode), VNode::NonTerminal(gnode))
                if self.mtmdd().level(&f) < self.mtmdd().level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.vlt(f, g)).collect();
                self.mdd_mut().create_node(headerid, &nodes)
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
                self.mdd_mut().create_node(headerid, &nodes)
            }
        };
        self.get_mut_bcache().insert(key, node);
        node
    }

    pub fn lte(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => {
                let resulteq = self.veq(fnode, gnode);
                if resulteq == self.mdd().one() {
                    return Node::Bool(self.mdd().one());
                }
                let resultlt = self.vlt(fnode, gnode);
                Node::Bool(resultlt)
            }
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn gt(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => {
                let tmp = self.vlt(gnode, fnode);
                Node::Bool(tmp)
            }
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn gte(&mut self, f: Node, g: Node) -> Node {
        match (f, g) {
            (Node::Value(fnode), Node::Value(gnode)) => {
                let resultlt = self.vlt(fnode, gnode);
                let result = self.mdd_mut().not(resultlt);
                Node::Bool(result)
            }
            _ => Node::Bool(self.mdd().undet()),
        }
    }

    pub fn ite(&mut self, f: Node, g: Node, h: Node) -> Node {
        match (f, g, h) {
            (Node::Bool(fnode), Node::Value(gnode), Node::Value(hnode)) => {
                let barf = self.mdd_mut().not(fnode);
                let vif = self.vif(fnode, gnode);
                let barvif = self.vif(barf, hnode);
                let result = self.mtmdd_mut().replace(vif, barvif);
                Node::Value(result)
            }
            (Node::Bool(fnode), Node::Bool(gnode), Node::Bool(hnode)) => {
                let result = self.mdd_mut().ite(fnode, gnode, hnode);
                Node::Bool(result)
            }
            (_, Node::Value(_gnode), Node::Value(_hnode)) => {
                let result = self.mtmdd().undet();
                Node::Value(result)
            }
            (_, Node::Bool(_gnode), Node::Bool(_hnode)) => {
                let result = self.mdd().undet();
                Node::Bool(result)
            }
            _ => panic!("ite: unexpected pattern."),
        }
    }

    pub fn vif(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMdd2Operation::If, f, g);
        if let Some(x) = self.get_vcache().get(&key) {
            return *x;
        }
        let node = match (
            self.mdd().get_node(&f).unwrap(),
            self.mtmdd().get_node(&g).unwrap(),
        ) {
            (BNode::Undet, _) => self.mtmdd().undet(),
            (_, VNode::Undet) => self.mtmdd().undet(),
            (BNode::Zero, _) => self.mtmdd().undet(),
            (BNode::One, _) => g,
            (BNode::NonTerminal(fnode), VNode::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vif(f, g)).collect();
                self.mtmdd_mut().create_node(headerid, &nodes)
            }
            (BNode::NonTerminal(fnode), VNode::NonTerminal(_gnode))
                if self.mdd().level(&f) > self.mtmdd().level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<_> = fnode.iter().cloned().collect();
                let nodes: Vec<_> = fnodeid.into_iter().map(|f| self.vif(f, g)).collect();
                self.mtmdd_mut().create_node(headerid, &nodes)
            }
            (BNode::NonTerminal(_fnode), VNode::NonTerminal(gnode))
                if self.mdd().level(&f) < self.mtmdd().level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<_> = gnode.iter().cloned().collect();
                let nodes: Vec<_> = gnodeid.into_iter().map(|g| self.vif(f, g)).collect();
                self.mtmdd_mut().create_node(headerid, &nodes)
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
                self.mtmdd_mut().create_node(headerid, &nodes)
            }
        };
        self.get_mut_vcache().insert(key, node);
        node
    }
}

