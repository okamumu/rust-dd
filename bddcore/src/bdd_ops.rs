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

impl Operation {
    /// Compact code for use as a computed-table key word.
    #[inline]
    pub(crate) fn code(&self) -> u32 {
        match self {
            Operation::And => 0,
            Operation::Or => 1,
            Operation::XOr => 2,
            Operation::Not => 3,
        }
    }
}

impl BddManager {
    pub fn not(&mut self, f: NodeId) -> NodeId {
        let key = (Operation::Not, f as u32, 0);
        if let Some(x) = self.cache_get(&key) {
            return x;
        }
        let result = match self.get_node(&f).unwrap() {
            Node::NonTerminal(fnode) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.not(f0);
                let high = self.not(f1);
                self.create_node(headerid, low, high)
            }
            Node::Zero => self.one(),
            Node::One => self.zero(),
            Node::Undet => self.undet(),
        };
        self.cache_put(key, result);
        result
    }

    pub fn and(&mut self, mut f: NodeId, mut g: NodeId) -> NodeId {
        if f == g {
            return f;
        }
        // Commutative: canonicalize operand order so and(a,b) and and(b,a)
        // share a computed-table entry (CUDD-style), improving hit rate.
        if f > g {
            std::mem::swap(&mut f, &mut g);
        }
        let key = (Operation::And, f as u32, g as u32);
        if let Some(x) = self.cache_get(&key) {
            return x;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.and(f0, g);
                let high = self.and(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = gnode.headerid();
                let low = self.and(f, g0);
                let high = self.and(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
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
        self.cache_put(key, result);
        result
    }

    pub fn or(&mut self, mut f: NodeId, mut g: NodeId) -> NodeId {
        if f == g {
            return f;
        }
        if f > g {
            std::mem::swap(&mut f, &mut g);
        }
        let key = (Operation::Or, f as u32, g as u32);
        if let Some(x) = self.cache_get(&key) {
            return x;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.or(f0, g);
                let high = self.or(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = gnode.headerid();
                let low = self.or(f, g0);
                let high = self.or(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
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
        self.cache_put(key, result);
        result
    }

    pub fn xor(&mut self, mut f: NodeId, mut g: NodeId) -> NodeId {
        if f == g {
            // f xor f = 0, except undet xor undet = undet
            return if f == self.undet() {
                self.undet()
            } else {
                self.zero()
            };
        }
        if f > g {
            std::mem::swap(&mut f, &mut g);
        }
        let key = (Operation::XOr, f as u32, g as u32);
        if let Some(x) = self.cache_get(&key) {
            return x;
        }
        let result = match (self.get_node(&f).unwrap(), self.get_node(&g).unwrap()) {
            (Node::NonTerminal(fnode), Node::NonTerminal(_gnode))
                if self.node_level(f) > self.node_level(g) =>
            {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let headerid = fnode.headerid();
                let low = self.xor(f0, g);
                let high = self.xor(f1, g);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(_fnode), Node::NonTerminal(gnode))
                if self.node_level(f) < self.node_level(g) =>
            {
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
                let headerid = gnode.headerid();
                let low = self.xor(f, g0);
                let high = self.xor(f, g1);
                self.create_node(headerid, low, high)
            }
            (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
                let (f0, f1) = (fnode.edge(0), fnode.edge(1));
                let (g0, g1) = (gnode.edge(0), gnode.edge(1));
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
        self.cache_put(key, result);
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

    /// Native `if-then-else`: `ite(f,g,h)` = "if f then g else h".
    ///
    /// A single Shannon recursion over the top variable of `f`/`g`/`h`, with its
    /// own computed table — versus the earlier `or(and(f,g), and(not f,h))`, which
    /// ran four separate apply traversals (and, not, and, or) per call. Semantics
    /// are identical, including `Undet` propagation: a terminal `f` selects one
    /// branch (so an `Undet` in the *other* branch is ignored), and `Undet` in the
    /// selected branch propagates outward.
    pub fn ite(&mut self, f: NodeId, g: NodeId, h: NodeId) -> NodeId {
        // Terminal cases on the condition.
        match self.get_node(&f).unwrap() {
            Node::One => return g,
            Node::Zero => return h,
            Node::Undet => return self.undet(),
            Node::NonTerminal(_) => {}
        }
        // f is non-terminal here.
        if g == h {
            return g;
        }
        if let Some(x) = self.ite_cache_get(f, g, h) {
            return x;
        }

        // Split on the top variable = the highest *real* level among the
        // operands (root has the largest level here; terminals report the
        // `Level::MAX` sentinel and must be excluded). `f` is non-terminal, so
        // its level seeds the max.
        let mut top = self.node_level(f);
        for x in [g, h] {
            let lx = self.node_level(x);
            if lx != Level::MAX && lx > top {
                top = lx;
            }
        }
        let (f0, f1) = self.cofactor(f, top);
        let (g0, g1) = self.cofactor(g, top);
        let (h0, h1) = self.cofactor(h, top);
        let headerid = self.top_header(f, g, h, top);

        let low = self.ite(f0, g0, h0);
        let high = self.ite(f1, g1, h1);
        let result = self.create_node(headerid, low, high);
        self.ite_cache_put(f, g, h, result);
        result
    }

    /// Split `id` on variable level `top`: if `id` is a non-terminal at that
    /// level, return its (low, high) children; otherwise it does not depend on
    /// the variable, so both cofactors are `id` itself.
    #[inline]
    fn cofactor(&self, id: NodeId, top: Level) -> (NodeId, NodeId) {
        if self.node_level(id) == top {
            if let Node::NonTerminal(n) = self.get_node(&id).unwrap() {
                return (n.edge(0), n.edge(1));
            }
        }
        (id, id)
    }

    /// Header id of whichever of `f`/`g`/`h` sits at level `top` (they share the
    /// same variable, hence the same header). At least one always matches.
    #[inline]
    fn top_header(&self, f: NodeId, g: NodeId, h: NodeId, top: Level) -> HeaderId {
        for x in [f, g, h] {
            if self.node_level(x) == top {
                if let Node::NonTerminal(n) = self.get_node(&x).unwrap() {
                    return n.headerid();
                }
            }
        }
        unreachable!("ite: top level has no matching non-terminal operand")
    }
}
