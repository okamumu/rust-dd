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

impl MddOperation {
    /// Compact code for use as a computed-table key word.
    #[inline]
    pub(crate) fn code(&self) -> u32 {
        match self {
            MddOperation::Not => 0,
            MddOperation::And => 1,
            MddOperation::Or => 2,
            MddOperation::XOr => 3,
            MddOperation::Replace => 4,
        }
    }
}

impl MddManager {
    pub fn not(&mut self, f: NodeId) -> NodeId {
        let key = (MddOperation::Not, f, 0);
        if let Some(nodeid) = self.cache_get(&key) {
            return nodeid;
        }
        let node = match self.get_node(&f).unwrap() {
            Node::Undet => self.undet(),
            Node::Zero => self.one(),
            Node::One => self.zero(),
            Node::NonTerminal(fnode) => {
                let headerid = fnode.headerid();
                let nodeid: Vec<NodeId> = fnode.iter().collect();
                let nodes: Vec<NodeId> = nodeid.iter().map(|&f| self.not(f)).collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache_put(key, node);
        node
    }

    pub fn and(&mut self, mut f: NodeId, mut g: NodeId) -> NodeId {
        // Commutative: canonicalize operand order so and(a,b)/and(b,a) share a
        // computed-table entry (CUDD-style), improving hit rate.
        if f > g {
            std::mem::swap(&mut f, &mut g);
        }
        let key = (MddOperation::And, f, g);
        if let Some(nodeid) = self.cache_get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.and(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.and(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
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
        self.cache_put(key, node);
        node
    }

    pub fn or(&mut self, mut f: NodeId, mut g: NodeId) -> NodeId {
        if f > g {
            std::mem::swap(&mut f, &mut g);
        }
        let key = (MddOperation::Or, f, g);
        if let Some(nodeid) = self.cache_get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => f,
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.or(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.or(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
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
        self.cache_put(key, node);
        node
    }

    pub fn xor(&mut self, mut f: NodeId, mut g: NodeId) -> NodeId {
        if f > g {
            std::mem::swap(&mut f, &mut g);
        }
        let key = (MddOperation::XOr, f, g);
        if let Some(nodeid) = self.cache_get(&key) {
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
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.xor(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.xor(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
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
        self.cache_put(key, node);
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

    /// Native k-ary `if-then-else`: `ite(f,g,h)` = "if f then g else h".
    ///
    /// A single Shannon recursion over the top variable of `f`/`g`/`h` with its
    /// own computed table — versus the earlier `or(and(f,g), and(not f,h))`,
    /// which ran four separate apply traversals per call. Mirrors the k-ary
    /// `and`/`or` here (children iterated, lower operands replicated) and the
    /// binary `bddcore` native `ite`. `Undet` propagation matches the composite:
    /// a terminal `f` selects one branch (Undet in the other is ignored), and
    /// `f == Undet` yields `undet`.
    pub fn ite(&mut self, f: NodeId, g: NodeId, h: NodeId) -> NodeId {
        match self.get_node(&f).unwrap() {
            Node::One => return g,
            Node::Zero => return h,
            Node::Undet => return self.undet(),
            Node::NonTerminal(_) => {}
        }
        if g == h {
            return g;
        }
        if let Some(x) = self.ite_cache_get(f, g, h) {
            return x;
        }

        // Top variable = highest real level among the operands (`f` is
        // non-terminal, so `level(f)` seeds it; terminals report `None`).
        let mut top = self.level(&f).unwrap();
        for x in [g, h] {
            if let Some(lx) = self.level(&x) {
                if lx > top {
                    top = lx;
                }
            }
        }
        let (headerid, k) = self.ite_top_header(f, g, h, top);
        let fc = self.ite_cofactor(f, top, k);
        let gc = self.ite_cofactor(g, top, k);
        let hc = self.ite_cofactor(h, top, k);
        let nodes: Vec<NodeId> = (0..k).map(|i| self.ite(fc[i], gc[i], hc[i])).collect();
        let result = self.create_node(headerid, &nodes);
        self.ite_cache_put(f, g, h, result);
        result
    }

    /// Split `id` on variable level `top`: if `id` is a non-terminal at that
    /// level, return its `k` children; otherwise it does not depend on the
    /// variable, so it is replicated across all `k` slots (as k-ary and/or do
    /// for the lower operand).
    #[inline]
    fn ite_cofactor(&self, id: NodeId, top: Level, k: usize) -> Vec<NodeId> {
        if self.level(&id) == Some(top) {
            if let Node::NonTerminal(n) = self.get_node(&id).unwrap() {
                return n.iter().collect();
            }
        }
        vec![id; k]
    }

    /// Header id and edge count of whichever of `f`/`g`/`h` sits at level `top`
    /// (they share the same variable, hence the same header). At least one matches.
    #[inline]
    fn ite_top_header(&self, f: NodeId, g: NodeId, h: NodeId, top: Level) -> (HeaderId, usize) {
        for x in [f, g, h] {
            if self.level(&x) == Some(top) {
                if let Node::NonTerminal(n) = self.get_node(&x).unwrap() {
                    return (n.headerid(), n.iter().count());
                }
            }
        }
        unreachable!("ite: top level has no matching non-terminal operand")
    }

    pub fn replace(&mut self, f: NodeId, g: NodeId) -> NodeId {
        let key = (MddOperation::Replace, f, g);
        if let Some(nodeid) = self.cache_get(&key) {
            return nodeid;
        }
        let node = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::Undet, _) => g,
            (_, Node::Undet) => f,
            (Node::Zero, _) => self.zero(),
            (Node::One, _) => self.one(),
            (Node::NonTerminal(fnode), Node::Zero) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::One) => {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.level(&f) > self.level(&g) =>
            {
                let headerid = fnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let nodes: Vec<NodeId> = fnodeid.iter().map(|&f| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.level(&f) < self.level(&g) =>
            {
                let headerid = gnode.headerid();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
                let nodes: Vec<NodeId> = gnodeid.iter().map(|&g| self.replace(f, g)).collect();
                self.create_node(headerid, &nodes)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let headerid = gnode.headerid();
                let fnodeid: Vec<NodeId> = fnode.iter().collect();
                let gnodeid: Vec<NodeId> = gnode.iter().collect();
                let nodes: Vec<NodeId> = fnodeid
                    .iter()
                    .zip(gnodeid.iter())
                    .map(|(&f, &g)| self.replace(f, g))
                    .collect();
                self.create_node(headerid, &nodes)
            }
        };
        self.cache_put(key, node);
        node
    }
}
