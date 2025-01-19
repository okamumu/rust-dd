use std::cmp::{max, min};

use crate::mtmdd::*;
use crate::nodes::*;
use common::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MtMddOperation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Min,
    Max,
    Replace,
}

impl<V> MtMddManager<V>
where
    V: MddValue,
{
    pub fn add(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Add, f, g);
        if let Some(&x) = self.get_cache().get(&key) {
            return x;
        }
        let node = match (&self.get_node(&f).unwrap(), &self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(fnode.value() + gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.add(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.add(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn sub(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Sub, f, g);
        if let Some(&x) = self.get_cache().get(&key) {
            return x;
        }
        let node = match (&self.get_node(&f).unwrap(), &self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(fnode.value() - gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.sub(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.sub(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn mul(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Mul, f, g);
        if let Some(&x) = self.get_cache().get(&key) {
            return x;
        }
        let node = match (&self.get_node(&f).unwrap(), &self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(fnode.value() * gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.mul(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.mul(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn div(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Div, f, g);
        if let Some(&x) = self.get_cache().get(&key) {
            return x;
        }
        let node = match (&self.get_node(&f).unwrap(), &self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                if gnode.value() == V::from(0) {
                    return self.undet();
                }
                self.value(fnode.value() / gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.div(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.div(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn rem(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Rem, f, g);
        if let Some(&x) = self.get_cache().get(&key) {
            return x;
        }
        let node = match (&self.get_node(&f).unwrap(), &self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                if gnode.value() == V::from(0) {
                    return self.undet();
                }
                self.value(fnode.value() % gnode.value())
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.rem(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.rem(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.rem(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.rem(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.rem(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn min(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Min, f, g);
        if let Some(&x) = self.get_cache().get(&key) {
            return x;
        }
        let node = match (&self.get_node(&f).unwrap(), &self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(min(fnode.value(), gnode.value()))
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.min(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.min(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn max(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Max, f, g);
        if let Some(&x) = self.get_cache().get(&key) {
            return x;
        }
        let node = match (&self.get_node(&f).unwrap(), &self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                self.value(max(fnode.value(), gnode.value()))
            }
            (Node::Terminal(_fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.max(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().cloned().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().cloned().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.max(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.get_mut_cache().insert(key, node);
        node
    }

    pub fn replace(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MtMddOperation::Replace, f, g);
        if let Some(x) = self.get_cache().get(&key) {
            return *x;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => g,
            (_, Node::Undet) => f,
            (Node::Terminal(_), _) => f,
            (Node::NonTerminal(fnode), Node::Terminal(_gnode)) => {
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
                let headerid = fnode.headerid();
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
