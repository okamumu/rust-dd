use bss::prelude::*;

/// A sizable, non-read-once BDD: OR_j (a_j AND b_j) where the a-half variables
/// all precede the b-half in the order, forcing the diagram to remember the
/// a-bits (~2^(n/2) nodes). The per-iteration `mask` negates a subset of the
/// a-side inputs, making each build a structurally distinct function.
fn build_big(mgr: &BddMgr, vars: &[BddNode], mask: u64) -> BddNode {
    let n = vars.len() / 2;
    let mut f = mgr.zero();
    for j in 0..n {
        let a = if (mask >> j) & 1 == 1 {
            vars[j].not()
        } else {
            vars[j].clone()
        };
        let term = a.and(&vars[j + n]);
        f = f.or(&term);
    }
    f
}

/// Build many distinct, sizable transient BDDs over a fixed variable set,
/// discarding each, and report the high-water arena size (total node slots).
fn churn_total(threshold: usize) -> usize {
    let mut mgr = BddMgr::new();
    mgr.set_gc_threshold(threshold);
    let vars: Vec<BddNode> = (0..16).map(|i| mgr.defvar(&format!("x{i:02}"))).collect();
    for i in 0..80u64 {
        let _f = build_big(&mgr, &vars, i); // dropped here -> becomes garbage
    }
    mgr.size().1
}

#[test]
fn auto_gc_bounds_arena_vs_no_gc() {
    let bounded = churn_total(500);
    let unbounded = churn_total(usize::MAX); // effectively never collects
    assert!(
        bounded * 2 < unbounded,
        "auto-gc arena ({bounded}) should be far smaller than no-gc ({unbounded})"
    );
}

#[test]
fn auto_gc_preserves_kept_results() {
    let mut mgr = BddMgr::new();
    mgr.set_gc_threshold(400); // collect frequently during the churn below

    let vars: Vec<BddNode> = (0..10).map(|i| mgr.defvar(&format!("x{i}"))).collect();

    // A result we hold across the whole churn; its DAG must survive every gc.
    let kept = vars[0].and(&vars[1]).or(&vars[2]);
    let size_before = kept.size();

    for i in 0..400u64 {
        let mut f = mgr.one();
        for (j, v) in vars.iter().enumerate() {
            f = if (i >> j) & 1 == 1 { f.and(v) } else { f.or(v) };
        }
        let _ = f;
    }

    // The kept handle is unchanged structurally and still canonical.
    assert_eq!(kept.size(), size_before, "kept result corrupted by gc");
    let recomputed = vars[0].and(&vars[1]).or(&vars[2]);
    assert!(kept.eq(&recomputed), "kept result lost canonical identity");
    assert!(!kept.is_zero());
}
