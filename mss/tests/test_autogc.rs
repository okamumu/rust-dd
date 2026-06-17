use mss::prelude::*;
use std::collections::HashSet;

/// A sizable, distinct boolean MDD: OR_j (a_j ==/!= b_j) where all a-half vars
/// precede the b-half in the order, forcing the diagram to remember the a-side
/// values (~range^(n/2) nodes). The `mask` flips each term between == and !=,
/// making every build a structurally distinct function.
fn build_big(mgr: &MddMgr<i32>, vars: &[MddNode<i32>], mask: u64) -> MddNode<i32> {
    let n = vars.len() / 2;
    let mut f = mgr.boolean(false);
    for j in 0..n {
        let (a, b) = (&vars[j], &vars[j + n]);
        let term = if (mask >> j) & 1 == 1 {
            a.eq(b)
        } else {
            a.ne(b)
        };
        f = f.or(&term);
    }
    f
}

/// Build many distinct transient MDDs over a fixed variable set, discarding
/// each, and report the high-water arena size (total node slots).
fn churn_total(threshold: usize) -> usize {
    let mut mgr: MddMgr<i32> = MddMgr::new();
    mgr.set_gc_threshold(threshold);
    let vars: Vec<MddNode<i32>> = (0..12).map(|i| mgr.defvar(&format!("x{i:02}"), 3)).collect();
    for i in 0..120u64 {
        let _f = build_big(&mgr, &vars, i); // dropped here -> becomes garbage
    }
    mgr.size().1
}

#[test]
fn auto_gc_bounds_arena_vs_no_gc() {
    let bounded = churn_total(1000);
    let unbounded = churn_total(usize::MAX);
    assert!(
        bounded * 2 < unbounded,
        "auto-gc arena ({bounded}) should be far smaller than no-gc ({unbounded})"
    );
}

#[test]
fn auto_gc_preserves_kept_results() {
    let mut mgr: MddMgr<i32> = MddMgr::new();
    mgr.set_gc_threshold(800);
    let vars: Vec<MddNode<i32>> = (0..12).map(|i| mgr.defvar(&format!("x{i:02}"), 3)).collect();

    // A result held across the whole churn; its DAG must survive every gc.
    let kept = vars[0].add(&vars[1]).mul(&vars[2]);
    let size_before = kept.size();
    let ss: HashSet<i32> = (0..32).collect();
    let count_before = kept.mdd_count(&ss);

    for i in 0..120u64 {
        let _f = build_big(&mgr, &vars, i);
    }

    assert_eq!(kept.size(), size_before, "kept result corrupted by gc");
    assert_eq!(kept.mdd_count(&ss), count_before, "kept result value changed");
}
