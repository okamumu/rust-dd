use common::prelude::*;
use crate::nodes::*;
use crate::bdd::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    And,
    Or,
    XOr,
    Not,
}

impl BddManager {
    pub fn not(&mut self, f: NodeId) -> NodeId {
        let key = (Operation::Not, f, 0);
        if let Some(x) = self.get_cache().get(&key) {
            return *x;
        }
        let result = match self.get_node(&f).unwrap() {
            Node::NonTerminal(fnode) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.not(f0);
                let high = self.not(f1);
                self.create_node(headerid, low, high)
            }
            Node::Zero => self.one(),
            Node::One => self.zero(),
            Node::Undet => self.undet(),
        };
        self.get_mut_cache().insert(key, result);
        result
    }

    pub fn and(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::And, f, g);
        if let Some(x) = self.get_cache().get(&key) {
            return *x;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.and(f0, g);
                let high = self.and(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.and(f, g0);
                let high = self.and(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.and(f0, g0);
                let high = self.and(f1, g1);
                self.create_node(headerid, low, high)
            }
            (Node::One, _) => g,
            (_, Node::One) => f,
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => self.zero(),
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
        };
        self.get_mut_cache().insert(key, result);
        result
    }

    pub fn or(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::Or, f, g);
        if let Some(x) = self.get_cache().get(&key) {
            return *x;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.or(f0, g);
                let high = self.or(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.or(f, g0);
                let high = self.or(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.or(f0, g0);
                let high = self.or(f1, g1);
                self.create_node(headerid, low, high)
            }
            (Node::Zero, _) => g,
            (_, Node::Zero) => f,
            (Node::One, _) => self.one(),
            (_, Node::One) => self.one(),
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
        };
        self.get_mut_cache().insert(key, result);
        result
    }

    pub fn xor(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (Operation::XOr, f, g);
        if let Some(x) = self.get_cache().get(&key) {
            return *x;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => {
                self.zero()
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let (f0, f1) = (fnode[0], fnode[1]);
                let headerid = fnode.headerid();
                let low = self.xor(f0, g);
                let high = self.xor(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = gnode.headerid();
                let low = self.xor(f, g0);
                let high = self.xor(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode[0], fnode[1]);
                let (g0, g1) = (gnode[0], gnode[1]);
                let headerid = fnode.headerid();
                let low = self.xor(f0, g0);
                let high = self.xor(f1, g1);
                self.create_node(headerid, low, high)
            }
            (Node::Zero, _) => g,
            (_, Node::Zero) => f,
            (Node::One, _) => self.not(g),
            (_, Node::One) => self.not(f),
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
        };
        self.get_mut_cache().insert(key, result);
        result
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
}
