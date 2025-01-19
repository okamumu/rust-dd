use crate::mdd::MddManager;
use crate::mdd::Node;
use common::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MddOperation {
    Not,
    And,
    Or,
    XOr,
    Replace,
}

impl MddManager {
    pub fn not(&mut self, f: NodeId) -> NodeId {
        let key = (MddOperation::Not, f, 0);
        if let Some(&nodeid) = self.get_cache().get(&key) {
            return nodeid;
        }
        let node = match self.get_node(&f).unwrap() {
            Node::Undet => self.undet(),
            Node::Zero => self.one(),
            Node::One => self.zero(),
            Node::NonTerminal(fnode) => {
                let headerid = fnode.headerid();
                let nodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = nodeid.iter().map(|&f| self.not(f)).collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn and(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MddOperation::And, f, g);
        if let Some(&nodeid) = self.get_cache().get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.and(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.and(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.and(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => self.zero(),
            (Node::One, _) => g,
            (_, Node::Zero) => self.zero(),
            (_, Node::One) => f,
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn or(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MddOperation::Or, f, g);
        if let Some(&nodeid) = self.get_cache().get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.or(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.or(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.or(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => g,
            (Node::One, _) => self.one(),
            (_, Node::Zero) => f,
            (_, Node::One) => self.one(),
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn xor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MddOperation::XOr, f, g);
        if let Some(&nodeid) = self.get_cache().get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => {
                self.zero()
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.xor(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.xor(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.xor(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => g,
            (Node::One, _) => self.not(g),
            (_, Node::Zero) => f,
            (_, Node::One) => self.not(f),
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn imp(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.not(f);
        self.or(tmp, g)
    }

    pub fn nand(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.and(f, g);
        self.not(tmp)
    }

    pub fn nor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.or(f, g);
        self.not(tmp)
    }

    pub fn xnor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let tmp = self.xor(f, g);
        self.not(tmp)
    }

    pub fn ite(&mut self, f: NodeId, g: NodeId, h: NodeId) -> NodeId {
        let x1 = self.and(f, g);
        let barf = self.not(f);
        let x2 = self.and(barf, h);
        self.or(x1, x2)
    }

    pub fn replace(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MddOperation::Replace, f, g);
        if let Some(&nodeid) = self.get_cache().get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => g,
            (_, Node::Undet) => f,
            (Node::Zero, _) => self.zero(),
            (Node::One, _) => self.one(),
            (Node::NonTerminal(fnode), Node::Zero) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::One) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.replace(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }
}
