//! Label-wise set operations on a [`ZmddManager`]: `intersect` (`(f∩g)_r = f_r ∩ g_r`) and
//! `setdiff` (`(f−g)_r = f_r − g_r`). Both preserve the disjoint/partition form (each vector
//! stays in at most one label), so they are closed on a single multi-terminal ZMDD. (`union`
//! does NOT preserve it — a vector can land in two labels — and is deferred.)
//!
//! The recursion mirrors an apply, but respects zero-suppression: when one operand has a
//! variable `X` the other lacks, that operand's family has `X=0`, so we descend the
//! **0-edge** only (same principle as `bss::bdd_minsol::without`'s level-mismatch arm) rather
//! than every edge. Getting this wrong reproduces the non-minimal bug class.

use crate::zmdd::ZmddManager;
use crate::mtmdd::Node;
use crate::nodes::*;
use common::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ZmddOperation {
    Intersect,
    Setdiff,
}

impl ZmddOperation {
    #[inline]
    pub(crate) fn code(&self) -> u32 {
        match self {
            ZmddOperation::Intersect => 0,
            ZmddOperation::Setdiff => 1,
        }
    }
}

impl<V> ZmddManager<V>
where
    V: MddValue,
{
    /// Label-wise intersection: `(f ∩ g)_r = f_r ∩ g_r`. A vector is kept iff both families
    /// classify it under the same terminal label.
    pub fn intersect(&mut self, mut f: NodeId, mut g: NodeId) -> NodeId {
        // Commutative: canonicalize operand order for a better cache hit rate.
        if f > g {
            std::mem::swap(&mut f, &mut g);
        }
        let key = (ZmddOperation::Intersect, f, g);
        if let Some(x) = self.cache_get(&key) {
            return x;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) | (_, Node::Undet) => self.undet(),
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                let (a, b) = (fnode.value(), gnode.value());
                if a == b {
                    self.value(a)
                } else {
                    self.undet()
                }
            }
            // f has the higher variable X; g's family has X=0, so only f's 0-edge can match.
            (Node::NonTerminal(fnode), Node::Terminal(_)) => {
                let fv: Vec<NodeId> = fnode.iter().collect();
                self.intersect(fv[0], g)
            }
            (Node::Terminal(_), Node::NonTerminal(gnode)) => {
                let gv: Vec<NodeId> = gnode.iter().collect();
                self.intersect(f, gv[0])
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_)) if self.level(&f) > self.level(&g) => {
                let fv: Vec<NodeId> = fnode.iter().collect();
                self.intersect(fv[0], g)
            }
            (Node::NonTerminal(_), Node::NonTerminal(gnode)) if self.level(&f) < self.level(&g) => {
                let gv: Vec<NodeId> = gnode.iter().collect();
                self.intersect(f, gv[0])
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let h = fnode.headerid();
                let fv: Vec<NodeId> = fnode.iter().collect();
                let gv: Vec<NodeId> = gnode.iter().collect();
                let ch: Vec<NodeId> = fv
                    .iter()
                    .zip(gv.iter())
                    .map(|(&a, &b)| self.intersect(a, b))
                    .collect();
                self.create_node(h, &ch)
            }
        };
        self.cache_put(key, node);
        node
    }

    /// Label-wise difference: `(f − g)_r = f_r − g_r`. A vector of `f` is dropped iff `g`
    /// classifies it under the same label.
    pub fn setdiff(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (ZmddOperation::Setdiff, f, g);
        if let Some(x) = self.cache_get(&key) {
            return x;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => self.undet(),
            (_, Node::Undet) => f,
            (Node::Terminal(fnode), Node::Terminal(gnode)) => {
                let (a, b) = (fnode.value(), gnode.value());
                if a != b {
                    self.value(a)
                } else {
                    self.undet()
                }
            }
            // f has the higher variable X (g's family has X=0): f's non-0 edges (X=i≥1) cannot
            // be in g, so keep them; only the 0-edge is diffed against g.
            (Node::NonTerminal(fnode), Node::Terminal(_)) => {
                let h = fnode.headerid();
                let mut ch: Vec<NodeId> = fnode.iter().collect();
                ch[0] = self.setdiff(ch[0], g);
                self.create_node(h, &ch)
            }
            (Node::Terminal(_), Node::NonTerminal(gnode)) => {
                let gv: Vec<NodeId> = gnode.iter().collect();
                self.setdiff(f, gv[0])
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_)) if self.level(&f) > self.level(&g) => {
                let h = fnode.headerid();
                let mut ch: Vec<NodeId> = fnode.iter().collect();
                ch[0] = self.setdiff(ch[0], g);
                self.create_node(h, &ch)
            }
            (Node::NonTerminal(_), Node::NonTerminal(gnode)) if self.level(&f) < self.level(&g) => {
                let gv: Vec<NodeId> = gnode.iter().collect();
                self.setdiff(f, gv[0])
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let h = fnode.headerid();
                let fv: Vec<NodeId> = fnode.iter().collect();
                let gv: Vec<NodeId> = gnode.iter().collect();
                let ch: Vec<NodeId> = fv
                    .iter()
                    .zip(gv.iter())
                    .map(|(&a, &b)| self.setdiff(a, b))
                    .collect();
                self.create_node(h, &ch)
            }
        };
        self.cache_put(key, node);
        node
    }
}
