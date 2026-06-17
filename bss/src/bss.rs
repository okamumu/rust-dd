use bddcore::prelude::*;
use crate::bdd_count;
use crate::bdd_prob;
use crate::bdd_minsol;
use crate::bdd_kofn;
use crate::bdd_path::*;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::rc::Weak;
use std::ops::{Add, Sub, Mul};

/// Minimum live-node count at which automatic gc may fire.
const GC_FLOOR: usize = 1 << 16;

/// State shared between a `BddMgr` and all of its `BddNode` handles, enabling
/// reference-counted gc roots: every live handle pins its node here, so the key
/// set is exactly the set of external roots.
#[derive(Debug)]
struct GcState {
    roots: BddHashMap<NodeId, u32>,
    /// Auto-gc fires once live occupancy reaches this; re-armed to 2x the
    /// surviving live set (but never below `floor`) after each collection.
    threshold: usize,
    /// Lower bound for `threshold` (configurable via `set_gc_threshold`).
    floor: usize,
}

/// Fire a garbage collection if live occupancy has reached the threshold.
///
/// Must be called only when no `BddManager` borrow is held (it borrows the
/// manager) — i.e. at the boundary of a wrapper op, after the result has been
/// wrapped in a (pinned) `BddNode` so it is protected as a root.
fn maybe_gc(bdd: &Rc<RefCell<BddManager>>, gc: &Rc<RefCell<GcState>>) {
    if bdd.borrow().live_node_count() < gc.borrow().threshold {
        return;
    }
    let roots: Vec<NodeId> = gc.borrow().roots.keys().copied().collect();
    let live = {
        let mut b = bdd.borrow_mut();
        b.gc(&roots);
        b.live_node_count()
    };
    let mut s = gc.borrow_mut();
    s.threshold = live.saturating_mul(2).max(s.floor);
}

pub struct BddMgr {
    bdd: Rc<RefCell<BddManager>>,
    gc: Rc<RefCell<GcState>>,
    vars: HashMap<String, BddNode>,
}

#[derive(Debug)]
pub struct BddNode {
    parent: Weak<RefCell<BddManager>>,
    gc: Weak<RefCell<GcState>>,
    node: NodeId,
}

impl BddNode {
    pub fn new(bdd: &Rc<RefCell<BddManager>>, gc: &Rc<RefCell<GcState>>, node: NodeId) -> Self {
        Self::from_weak(Rc::downgrade(bdd), Rc::downgrade(gc), node)
    }

    fn from_weak(
        parent: Weak<RefCell<BddManager>>,
        gc: Weak<RefCell<GcState>>,
        node: NodeId,
    ) -> Self {
        if let Some(g) = gc.upgrade() {
            *g.borrow_mut().roots.entry(node).or_insert(0) += 1;
        }
        BddNode { parent, gc, node }
    }

    /// Wrap a node freshly computed by an op on `self`, then let the collector
    /// run (safe here: no manager borrow is held, and the result is now pinned).
    fn rewrap(&self, bdd: &Rc<RefCell<BddManager>>, node: NodeId) -> BddNode {
        let n = BddNode::from_weak(self.parent.clone(), self.gc.clone(), node);
        if let Some(gc) = self.gc.upgrade() {
            maybe_gc(bdd, &gc);
        }
        n
    }
}

impl Clone for BddNode {
    fn clone(&self) -> Self {
        BddNode::from_weak(self.parent.clone(), self.gc.clone(), self.node)
    }
}

impl Drop for BddNode {
    fn drop(&mut self) {
        if let Some(g) = self.gc.upgrade() {
            let mut s = g.borrow_mut();
            if let Some(c) = s.roots.get_mut(&self.node) {
                *c -= 1;
                if *c == 0 {
                    s.roots.remove(&self.node);
                }
            }
        }
    }
}

impl BddMgr {
    // constructor
    pub fn new() -> Self {
        BddMgr {
            bdd: Rc::new(RefCell::new(BddManager::new())),
            gc: Rc::new(RefCell::new(GcState {
                roots: BddHashMap::default(),
                threshold: GC_FLOOR,
                floor: GC_FLOOR,
            })),
            vars: HashMap::default(),
        }
    }

    /// Live-node count at which automatic gc fires (for tuning / tests). The
    /// collector re-arms to 2x the surviving live set after each run, but never
    /// below this value.
    pub fn set_gc_threshold(&self, threshold: usize) {
        let mut s = self.gc.borrow_mut();
        s.threshold = threshold;
        s.floor = threshold;
    }

    /// Current number of live (non-reclaimed) nodes in the underlying manager.
    pub fn live_node_count(&self) -> usize {
        self.bdd.borrow().live_node_count()
    }

    /// Wrap a freshly produced node into a pinned handle and give the collector
    /// a chance to run. Call only with no `BddManager` borrow held.
    fn wrap(&self, node: NodeId) -> BddNode {
        let n = BddNode::new(&self.bdd, &self.gc, node);
        maybe_gc(&self.bdd, &self.gc);
        n
    }

    // size
    pub fn size(&self) -> (usize, usize, usize) {
        self.bdd.borrow().size()
    }

    /// Garbage-collect the underlying BDD.
    ///
    /// Keeps every defined variable plus the supplied `keep` nodes (and
    /// everything reachable from them); reclaims the rest. Returns the number of
    /// reclaimed node slots. Because the collector does not compact, all kept
    /// nodes stay valid — but any `BddNode` not covered by `keep` (nor a
    /// variable / descendant of a kept node) must no longer be used.
    pub fn gc(&self, keep: &[&BddNode]) -> usize {
        // All live handles (including variables) are pinned roots already; the
        // explicit `keep` is accepted for API symmetry but is redundant.
        let mut roots: Vec<NodeId> = self.gc.borrow().roots.keys().copied().collect();
        roots.extend(keep.iter().map(|n| n.node));
        self.bdd.borrow_mut().gc(&roots)
    }

    // zero
    pub fn zero(&self) -> BddNode {
        let z = self.bdd.borrow().zero();
        self.wrap(z)
    }

    // one
    pub fn one(&self) -> BddNode {
        let o = self.bdd.borrow().one();
        self.wrap(o)
    }

    pub fn create_node(&self, h: HeaderId, x0: &BddNode, x1: &BddNode) -> BddNode {
        let result = self.bdd.borrow_mut().create_node(h, x0.node, x1.node);
        self.wrap(result)
    }

    // defvar
    pub fn defvar(&mut self, var: &str) -> BddNode {
        if let Some(node) = self.vars.get(var) {
            return node.clone();
        }
        let level = self.vars.len();
        let node = {
            let mut bdd = self.bdd.borrow_mut();
            let h = bdd.create_header(level, var);
            let (x0, x1) = (bdd.zero(), bdd.one());
            bdd.create_node(h, x0, x1)
        };
        // Variables stay alive for the manager's lifetime via a pinned handle.
        let bnode = BddNode::new(&self.bdd, &self.gc, node);
        self.vars.insert(var.to_string(), bnode.clone());
        bnode
    }

    pub fn get_varorder(&self) -> Vec<String> {
        let bdd = self.bdd.borrow();
        let mut result = vec!["?".to_string(); self.vars.len()];
        for (k, v) in self.vars.iter() {
            let node = bdd.get_node(&v.node).unwrap();
            let hid = node.headerid().unwrap();
            let header = bdd.get_header(&hid).unwrap();
            result[header.level()] = k.clone();
        }
        result
    }

    pub fn rpn(&mut self, expr: &str) -> Result<BddNode, String> {
        let mut stack = Vec::new();
        let mut cache = HashMap::new();
        for token in expr.split_whitespace() {
            match token {
                "0" | "False" => {
                    let bdd = self.bdd.borrow_mut();
                    stack.push(bdd.zero());
                }
                "1" | "True" => {
                    let bdd = self.bdd.borrow_mut();
                    stack.push(bdd.one());
                }
                "&" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(bdd.and(left, right));
                }
                "|" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(bdd.or(left, right));
                }
                "^" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(bdd.xor(left, right));
                }
                "~" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let node = stack.pop().unwrap();
                    stack.push(bdd.not(node));
                }
                "?" => {
                    let mut bdd = self.bdd.borrow_mut();
                    let else_ = stack.pop().unwrap();
                    let then = stack.pop().unwrap();
                    let cond = stack.pop().unwrap();
                    stack.push(bdd.ite(cond, then, else_));
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
                    let node = self.defvar(token);
                    stack.push(node.node);
                }
            }
        }
        if stack.len() == 1 {
            return Ok(self.wrap(stack.pop().unwrap()));
        } else {
            return Err("Invalid expression".to_string());
        }
    }

    pub fn and(&self, nodes: &[BddNode]) -> BddNode {
        let ids = nodes.iter().map(|x| x.node).collect::<Vec<NodeId>>();
        let result = bdd_kofn::and(&mut self.bdd.borrow_mut(), &ids);
        self.wrap(result)
    }

    pub fn or(&self, nodes: &[BddNode]) -> BddNode {
        let ids = nodes.iter().map(|x| x.node).collect::<Vec<NodeId>>();
        let result = bdd_kofn::or(&mut self.bdd.borrow_mut(), &ids);
        self.wrap(result)
    }

    pub fn kofn(&self, k: usize, nodes: &[BddNode]) -> BddNode {
        let ids = nodes.iter().map(|x| x.node).collect::<Vec<NodeId>>();
        let result = bdd_kofn::kofn(&mut self.bdd.borrow_mut(), k, &ids);
        self.wrap(result)
    }

    pub fn clear_cache(&mut self) {
        self.bdd.borrow_mut().clear_cache();
    }
}

impl BddNode {
    pub fn get_mgr(&self) -> Rc<RefCell<BddManager>> {
        self.parent.upgrade().unwrap()
    }

    pub fn get_id(&self) -> NodeId {
        self.node
    }

    pub fn get_header(&self) -> Option<HeaderId> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(&self.node)?;
        node.headerid()
    }

    pub fn get_level(&self) -> Option<Level> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(&self.node)?;
        let hid = node.headerid()?;
        let header = bdd.get_header(&hid)?;
        Some(header.level())
    }

    pub fn get_label(&self) -> Option<String> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(&self.node)?;
        let hid = node.headerid()?;
        let header = bdd.get_header(&hid)?;
        Some(header.label().to_string())
    }

    pub fn get_children(&self) -> Option<(BddNode, BddNode)> {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(&self.node)?;
        match node {
            Node::Zero | Node::One | Node::Undet => None,
            Node::NonTerminal(fnode) => {
                // A `bdd` borrow is held here, so pin only (no maybe_gc).
                let f0 = BddNode::from_weak(self.parent.clone(), self.gc.clone(), fnode.edge(0));
                let f1 = BddNode::from_weak(self.parent.clone(), self.gc.clone(), fnode.edge(1));
                Some((f0, f1))
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(&self.node).unwrap();
        match node {
            Node::Zero => true,
            _ => false,
        }
    }

    pub fn is_one(&self) -> bool {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(&self.node).unwrap();
        match node {
            Node::One => true,
            _ => false,
        }
    }

    pub fn is_undet(&self) -> bool {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let node = bdd.get_node(&self.node).unwrap();
        match node {
            Node::Undet => true,
            _ => false,
        }
    }

    pub fn dot(&self) -> String {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow().dot_string(&self.node);
        result
    }

    pub fn and(&self, other: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().and(self.node, other.node);
        self.rewrap(&bdd, result)
    }

    pub fn or(&self, other: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().or(self.node, other.node);
        self.rewrap(&bdd, result)
    }

    pub fn xor(&self, other: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().xor(self.node, other.node);
        self.rewrap(&bdd, result)
    }

    pub fn not(&self) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().not(self.node);
        self.rewrap(&bdd, result)
    }

    pub fn ite(&self, then: &BddNode, else_: &BddNode) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let result = bdd.borrow_mut().ite(self.node, then.node, else_.node);
        self.rewrap(&bdd, result)
    }

    pub fn eq(&self, other: &BddNode) -> bool {
        self.node == other.node
    }

    pub fn prob<T>(&self, pv: &HashMap<String, T>, ss: &[bool]) -> T
    where
        T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = BddHashMap::default();
        bdd_prob::prob(
            &mut bdd.clone().borrow_mut(),
            self.node,
            &pv,
            ss,
            &mut cache,
        )
    }

    pub fn bmeas<T>(&self, pv: &HashMap<String, T>, ss: &[bool]) -> HashMap<String, T>
    where
        T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    {
        let bdd = self.parent.upgrade().unwrap();
        bdd_prob::bmeas(&mut bdd.clone().borrow_mut(), ss, self.node, &pv)
    }

    // obtain minimal path vectors (mpvs) of monotone BDD
    pub fn minpath(&self) -> BddNode {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache1 = BddHashMap::default();
        let mut cache2 = BddHashMap::default();
        let result = bdd_minsol::minsol(&mut bdd.borrow_mut(), self.node, &mut cache1, &mut cache2);
        self.rewrap(&bdd, result)
    }

    pub fn bdd_count(&self, ss: &[bool]) -> u64 {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = BddHashMap::default();
        bdd_count::bdd_count(&mut bdd.clone().borrow_mut(), ss, self.node, &mut cache)
    }

    pub fn bdd_extract(&self, ss: &[bool]) -> BddPath {
        BddPath::new(self.clone(), ss)
    }

    pub fn zdd_count(&self, ss: &[bool]) -> u64 {
        let bdd = self.parent.upgrade().unwrap();
        let mut cache = BddHashMap::default();
        bdd_count::zdd_count(&mut bdd.clone().borrow_mut(), ss, self.node, &mut cache)
    }

    pub fn zdd_extract(&self, ss: &[bool]) -> ZddPath {
        ZddPath::new(self.clone(), ss)
    }

    pub fn size(&self) -> (u64, u64, u64) {
        let bddmgr = self.parent.upgrade().unwrap();
        let bdd = bddmgr.borrow();
        let mut cache = BddHashSet::default();
        let (nn, nv, ne) = bdd_count::node_count(&bdd, self.node, &mut cache);
        (nn, nv, ne-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bss_mgr() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let f = x.and(&y).or(&z);
        let g = x.and(&y).xor(&z);
        let h = x.and(&y).ite(&z, &x);
        let i = x.and(&y).ite(&z, &y);
        let j = x.and(&y).ite(&z, &x.and(&y));
        let k = x.and(&y).ite(&z, &x.and(&y).ite(&z, &x));
        let l = x.and(&y).ite(&z, &x.and(&y).ite(&z, &x.and(&y)));
        let m = x
            .and(&y)
            .ite(&z, &x.and(&y).ite(&z, &x.and(&y).ite(&z, &x)));
        let n = x
            .and(&y)
            .ite(&z, &x.and(&y).ite(&z, &x.and(&y).ite(&z, &x.and(&y))));
    }

    #[test]
    fn test_bss_mgr_prob() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let f = x.and(&y).or(&z);
        let mut pv = HashMap::new();
        pv.insert("x".to_string(), 0.2);
        pv.insert("y".to_string(), 0.3);
        pv.insert("z".to_string(), 0.6);
        let result = f.prob(&pv, &[true]);
        println!("{:?}", result);
    }

    #[test]
    fn test_bss_mgr_rpn() {
        let mut bss = BddMgr::new();
        let x = bss.rpn("x").unwrap();
        let y = bss.rpn("y").unwrap();
        let z = bss.rpn("z").unwrap();
        let f = bss.rpn("x y & z |").unwrap();
    }

    #[test]
    fn test_bdd_path() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        let path = z.bdd_extract(&[true]);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_bdd_path2() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        let path = z.bdd_extract(&[false]);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_bdd_path3() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        println!("{}", z.bdd_count(&[true, false]));
        let path = z.bdd_extract(&[false, true]);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_zdd_path() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        let path = z.zdd_extract(&[true]);
        let mut count = 0;
        for p in path {
            count += 1;
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_node_count() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let z = bss.rpn("x y & z |").unwrap();
        println!("{}", z.dot());
        println!("{:?}", z.size());
    }

    #[test]
    fn test_kofn1() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let f = bss.kofn(2, &vec![x, y, z]);
        println!("{}", f.dot());
    }

    #[test]
    fn test_and1() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let f = bss.and(&vec![x, y, z]);
        println!("{}", f.dot());
    }

    #[test]
    fn test_or1() {
        let mut bss = BddMgr::new();
        let x = bss.defvar("x");
        let y = bss.defvar("y");
        let z = bss.defvar("z");
        let f = bss.or(&vec![x, y, z]);
        println!("{}", f.dot());
    }
}

