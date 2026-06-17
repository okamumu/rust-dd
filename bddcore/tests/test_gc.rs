use bddcore::prelude::*;

#[test]
fn bdd_gc_retains_live_cache_entries() {
    let mut dd = BddManager::new();
    let hx = dd.create_header(0, "x");
    let hy = dd.create_header(1, "y");
    let hz = dd.create_header(2, "z");
    let (z0, o1) = (dd.zero(), dd.one());
    let x = dd.create_node(hx, z0, o1);
    let y = dd.create_node(hy, z0, o1);
    let zz = dd.create_node(hz, z0, o1);

    let a = dd.and(x, y); // lives (we keep it)
    let _b = dd.or(x, zz); // dies once zz is unreachable

    let cache_before = dd.size().2;
    assert!(cache_before > 0);

    let reclaimed = dd.gc(&[a, x, y]); // keep a, x, y; zz and b become garbage
    assert!(reclaimed > 0);

    let cache_after = dd.size().2;
    // Entries whose operands+result all survive are kept (a full flush would
    // leave 0); entries touching a reclaimed node are dropped.
    assert!(cache_after > 0, "live cache entries must be retained, not flushed");
    assert!(
        cache_after < cache_before,
        "entries touching reclaimed nodes must be dropped"
    );
    // The kept memoized op is still consistent (now served from the cache).
    assert_eq!(dd.and(x, y), a);
}

#[test]
fn bdd_gc_keeps_roots_and_reclaims_rest() {
    let mut dd = BddManager::new();
    let hx = dd.create_header(0, "x");
    let hy = dd.create_header(1, "y");
    let hz = dd.create_header(2, "z");
    let (z0, o1) = (dd.zero(), dd.one());
    let x = dd.create_node(hx, z0, o1);
    let y = dd.create_node(hy, z0, o1);
    let zv = dd.create_node(hz, z0, o1);

    let f = dd.and(x, y);
    let _g = dd.or(x, zv); // garbage once we drop it

    let live_before = dd.live_node_count();
    // Keep f and the inputs we will reuse below.
    let reclaimed = dd.gc(&[f, x, y]);
    assert!(reclaimed > 0, "gc should reclaim the unreferenced or() nodes");
    assert!(dd.live_node_count() < live_before);

    // Survivors keep their ids and structure: recomputing and(x, y) hash-conses
    // back to the very same node f.
    assert_eq!(dd.and(x, y), f);
}

#[test]
fn bdd_gc_empty_roots_frees_all_nonterminals() {
    let mut dd = BddManager::new();
    let hx = dd.create_header(0, "x");
    let hy = dd.create_header(1, "y");
    let (z0, o1) = (dd.zero(), dd.one());
    let x = dd.create_node(hx, z0, o1);
    let y = dd.create_node(hy, z0, o1);
    let _ = dd.and(x, y);

    let total = dd.size().1; // includes 3 terminals
    let reclaimed = dd.gc(&[]);
    assert_eq!(reclaimed, total - 3, "every non-terminal should be reclaimed");
    assert_eq!(dd.live_node_count(), 3, "only the three terminals remain live");
}

#[test]
fn bdd_gc_slots_are_reused_without_growth() {
    let mut dd = BddManager::new();
    let hx = dd.create_header(0, "x");
    let hy = dd.create_header(1, "y");
    let (z0, o1) = (dd.zero(), dd.one());
    let x = dd.create_node(hx, z0, o1);
    let y = dd.create_node(hy, z0, o1);
    let _ = dd.and(x, y);

    dd.gc(&[]); // free everything, populating the free list
    let total_slots = dd.size().1;
    assert!(total_slots - dd.live_node_count() > 0, "free list should be non-empty");

    // Allocating new nodes recycles freed slots: total slot count must not grow.
    let hw = dd.create_header(3, "w");
    let _w = dd.create_node(hw, z0, o1);
    assert_eq!(dd.size().1, total_slots, "new node should reuse a freed slot");
}

#[test]
fn zdd_gc_keeps_roots_and_reclaims_rest() {
    let mut dd = ZddManager::new();
    let ha = dd.create_header(0, "a");
    let hb = dd.create_header(1, "b");
    let (z0, o1) = (dd.zero(), dd.one());
    let a = dd.create_node(ha, z0, o1);
    let b = dd.create_node(hb, z0, o1);

    let u = dd.union(a, b);
    let _p = dd.product(a, b); // garbage

    let live_before = dd.live_node_count();
    let reclaimed = dd.gc(&[u, a, b]);
    assert!(reclaimed > 0);
    assert!(dd.live_node_count() < live_before);
    // union(a, b) hash-conses back to u.
    assert_eq!(dd.union(a, b), u);
}

#[test]
fn zdd_gc_empty_roots_frees_all_nonterminals() {
    let mut dd = ZddManager::new();
    let ha = dd.create_header(0, "a");
    let hb = dd.create_header(1, "b");
    let (z0, o1) = (dd.zero(), dd.one());
    let a = dd.create_node(ha, z0, o1);
    let b = dd.create_node(hb, z0, o1);
    let _ = dd.union(a, b);

    let total = dd.size().1;
    let reclaimed = dd.gc(&[]);
    assert_eq!(reclaimed, total - 3);
    assert_eq!(dd.live_node_count(), 3);
}
