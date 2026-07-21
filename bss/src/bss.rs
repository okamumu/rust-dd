use bddcore::prelude::*;
use crate::bdd::{BddMgr, BddNode};
use crate::zdd::{ZddMgr, ZddNode};
use crate::bdd_minsol;
use crate::zdd_convert;

/// Binary-state system manager: owns a [`BddMgr`] (boolean structure functions) **and** a
/// [`ZddMgr`] (set families), and provides the analyses that span both — computing the
/// minimal **path** / **cut** vectors of a structure function as genuine ZDD set families.
///
/// Build boolean expressions through the delegated BDD API ([`defvar`](Self::defvar),
/// [`rpn`](Self::rpn), [`and`](Self::and)/[`or`](Self::or)/[`kofn`](Self::kofn), or the
/// [`BddNode`] operators), then call [`minpath`](Self::minpath) / [`mincut`](Self::mincut)
/// to get a [`ZddNode`] supporting the set algebra
/// (`union`/`intersect`/`setdiff`/`product`/`divide`).
///
/// ```
/// use bss::prelude::*;
///
/// let mut bss = BssMgr::new();
/// let x = bss.defvar("x");
/// let y = bss.defvar("y");
/// let paths = bss.minpath(&x.and(&y)).unwrap();   // minimal path vectors, as a ZDD
/// assert_eq!(paths.count(&[true]), 1);             // {x, y}
/// let cuts = bss.mincut(&x.and(&y)).unwrap();      // minimal cut vectors
/// assert_eq!(cuts.count(&[true]), 2);              // {x}, {y}
/// ```
pub struct BssMgr {
    bdd: BddMgr,
    zdd: ZddMgr,
}

impl BssMgr {
    pub fn new() -> Self {
        BssMgr {
            bdd: BddMgr::new(),
            zdd: ZddMgr::new(),
        }
    }

    /// The underlying BDD manager (boolean structure functions).
    pub fn bdd(&self) -> &BddMgr {
        &self.bdd
    }

    /// The underlying ZDD manager (set families).
    pub fn zdd(&self) -> &ZddMgr {
        &self.zdd
    }

    // --- BDD building, delegated to the inner BddMgr -------------------------

    pub fn defvar(&mut self, var: &str) -> BddNode {
        self.bdd.defvar(var)
    }

    pub fn rpn(&mut self, expr: &str) -> Result<BddNode, String> {
        self.bdd.rpn(expr)
    }

    pub fn zero(&self) -> BddNode {
        self.bdd.zero()
    }

    pub fn one(&self) -> BddNode {
        self.bdd.one()
    }

    pub fn create_node(&self, h: HeaderId, x0: &BddNode, x1: &BddNode) -> BddNode {
        self.bdd.create_node(h, x0, x1)
    }

    pub fn and(&self, nodes: &[BddNode]) -> BddNode {
        self.bdd.and(nodes)
    }

    pub fn or(&self, nodes: &[BddNode]) -> BddNode {
        self.bdd.or(nodes)
    }

    pub fn kofn(&self, k: usize, nodes: &[BddNode]) -> BddNode {
        self.bdd.kofn(k, nodes)
    }

    pub fn get_varorder(&self) -> Vec<String> {
        self.bdd.get_varorder()
    }

    pub fn size(&self) -> (usize, usize, usize) {
        self.bdd.size()
    }

    pub fn clear_cache(&mut self) {
        self.bdd.clear_cache();
        self.zdd.clear_cache();
    }

    // --- minpath / mincut: minsol (BDD) -> genuine ZDD set family -----------

    /// Minimal **path** vectors of the structure function `node` (its prime implicants, via
    /// the Rauzy minsol), returned as a genuine ZDD set family, or `None` if the function is
    /// not monotone (coherent).
    ///
    /// A minimal path vector is a minimal set of components whose functioning makes the
    /// system function. The minsol runs on the BDD; the result is converted (internally,
    /// once) into this manager's [`ZddMgr`], so the returned [`ZddNode`] supports the set
    /// algebra. Non-monotone input (built with `xor`/`not`/`<`/`!=`) returns `None`.
    /// See [`mincut`](Self::mincut) for the dual.
    pub fn minpath(&self, node: &BddNode) -> Option<ZddNode> {
        let bdd = node.get_mgr();
        let fake = {
            let mut mgr = bdd.borrow_mut();
            let mut cache1 = BddHashMap::default();
            let mut cache2 = BddHashMap::default();
            bdd_minsol::minsol(&mut mgr, node.get_id(), &mut cache1, &mut cache2)
        };
        fake.map(|f| {
            let zid = {
                let src = bdd.borrow();
                let mut dst = self.zdd.arena().borrow_mut();
                let mut zh = BddHashMap::default();
                let mut memo = BddHashMap::default();
                zdd_convert::to_zdd(&src, f, &mut dst, &mut zh, &mut memo)
            };
            self.zdd.wrap(zid)
        })
    }

    /// Minimal **cut** vectors of the structure function `node` as a genuine ZDD set family,
    /// or `None` if it is not monotone.
    ///
    /// A minimal cut vector is a minimal set of components whose failure makes the system
    /// fail. It is `minpath` of the dual: `mincut(φ) = minpath(φ^D)`.
    pub fn mincut(&self, node: &BddNode) -> Option<ZddNode> {
        self.minpath(&node.dual())
    }
}
