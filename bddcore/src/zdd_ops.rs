use common::prelude::*;
use crate::nodes::*;
use crate::zdd::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ZddOperation {
    Intersect,
    Union,
    Setdiff,
    Product,
    Division,
}

impl ZddManager {
    pub fn intersect(&mut self, f: NodeId, g: NodeId) -> NodeId {
        if f == g {
            return f;
        }
        let key = (ZddOperation::Intersect, f as u32, g as u32);
        if let Some(id) = self.cache_get(&key) {
            return id;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => g,
            (_, Node::Undet) => f,
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => self.zero(),
            (Node::One, _) => g,
            (_, Node::One) => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let f0 = fnode.edge(0);
                self.intersect(f0, g)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                let g0 = gnode.edge(0);
                self.intersect(f, g0)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.intersect(f0, g0);
                let high = self.intersect(f1, g1);
                self.create_node(headerid, low, high)
            }
        };
        self.cache_put(key, result);
        result
    }

    pub fn union(&mut self, f: NodeId, g: NodeId) -> NodeId {
        if f == g {
            return f;
        }
        let key = (ZddOperation::Union, f as u32, g as u32);
        if let Some(id) = self.cache_get(&key) {
            return id;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => f,
            (_, Node::Undet) => g,
            (Node::Zero, _) => g,
            (_, Node::Zero) => f,
            (Node::One, Node::One) => self.one(),
            (Node::NonTerminal(fnode), Node::One) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.union(f0, self.one());
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::One, Node::NonTerminal(gnode)) => {
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = gnode.headerid();
                let low = self.union(self.one(), g0);
                let high = g1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.union(f0, g);
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = gnode.headerid();
                let low = self.union(f, g0);
                let high = g1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.union(f0, g0);
                let high = self.union(f1, g1);
                self.create_node(headerid, low, high)
            }
        };
        self.cache_put(key, result);
        result
    }

    pub fn setdiff(&mut self, f: NodeId, g: NodeId) -> NodeId {
        if f == g {
            // f \ f = empty, except undet \ undet = undet
            return if f == self.undet() {
                self.undet()
            } else {
                self.zero()
            };
        }
        let key = (ZddOperation::Setdiff, f as u32, g as u32);
        if let Some(id) = self.cache_get(&key) {
            return id;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => f,
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => f,
            (Node::One, Node::One) => self.zero(),
            (Node::NonTerminal(fnode), Node::One) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.setdiff(f0, self.one());
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::One, Node::NonTerminal(gnode)) => {
                let g0 = gnode.edge(0);
                self.setdiff(self.one(), g0)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.setdiff(f0, g);
                let high = f1;
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                let g0 = gnode.edge(0);
                self.setdiff(f, g0)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.setdiff(f0, g0);
                let high = self.setdiff(f1, g1);
                self.create_node(headerid, low, high)
            }
        };
        self.cache_put(key, result);
        result
    }

    pub fn product(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (ZddOperation::Product, f as u32, g as u32);
        if let Some(id) = self.cache_get(&key) {
            return id;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Zero, _) => self.zero(),
            (_, Node::Zero) => self.zero(),
            (_, Node::One) => f,
            (Node::One, _) => g,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.product(f0, g);
                let high = self.product(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = gnode.headerid();
                let low = self.product(f, g0);
                let high = self.product(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.product(f0, g0);
                let high = self.product(f1, g1);
                let tmp = self.product(f1, g0);
                let high = self.union(high, tmp);
                let tmp = self.product(f0, g1);
                let high = self.union(high, tmp);
                self.create_node(headerid, low, high)
            }
        };
        self.cache_put(key, result);
        result
    }

    pub fn divide(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (ZddOperation::Division, f as u32, g as u32);
        if let Some(id) = self.cache_get(&key) {
            return id;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (_, Node::Zero) => self.undet(),
            (_, Node::One) => f,
            (Node::Zero, _) => self.zero(),
            (Node::One, _) => g,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let f0 = fnode.edge(0);
                self.divide(f0, g)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                self.undet()
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let x = self.divide(f0, g0);
                let y = self.divide(f1, g1);
                self.intersect(x, y)
            }
        };
        self.cache_put(key, result);
        result
    }
}
