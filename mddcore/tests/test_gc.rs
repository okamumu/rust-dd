use mddcore::prelude::*;

#[test]
fn mdd_gc_keeps_roots_and_reclaims_rest() {
    let mut dd = MddManager::new();
    let h0 = dd.create_header(0, "x", 2);
    let h1 = dd.create_header(1, "y", 2);
    let (z, o) = (dd.zero(), dd.one());
    let x = dd.create_node(h0, &[z, o]);
    let y = dd.create_node(h1, &[z, o]);
    let f = dd.create_node(h0, &[z, y]); // references y
    let _g = dd.create_node(h1, &[o, z]); // garbage

    let live_before = dd.live_node_count();
    let reclaimed = dd.gc(&[f, x, y]);
    assert!(reclaimed > 0);
    assert!(dd.live_node_count() < live_before);
    // survivor hash-cons identity
    assert_eq!(dd.create_node(h0, &[z, y]), f);
}

#[test]
fn mdd_gc_empty_roots_frees_all_nonterminals() {
    let mut dd = MddManager::new();
    let h0 = dd.create_header(0, "x", 2);
    let (z, o) = (dd.zero(), dd.one());
    let _ = dd.create_node(h0, &[z, o]);
    let total = dd.size().1;
    let reclaimed = dd.gc(&[]);
    assert_eq!(reclaimed, total - 3); // zero/one/undet remain
    assert_eq!(dd.live_node_count(), 3);
}

#[test]
fn mtmdd_gc_reclaims_unreferenced_value_terminals() {
    let mut dd: MtMddManager<i32> = MtMddManager::new();
    let h0 = dd.create_header(0, "x", 2);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let x = dd.create_node(h0, &[v0, v1]);

    // garbage: a value terminal and a node only reachable through it
    let v2 = dd.value(9);
    let h1 = dd.create_header(1, "y", 2);
    let _g = dd.create_node(h1, &[v1, v2]);

    let live_before = dd.live_node_count();
    let reclaimed = dd.gc(&[x]); // keep x => v0, v1 stay; v2, g freed
    assert!(reclaimed > 0);
    assert!(dd.live_node_count() < live_before);
    // kept value terminal still resolves to the same id; x re-conses to same id
    assert_eq!(dd.value(0), v0);
    assert_eq!(dd.create_node(h0, &[v0, v1]), x);
}

#[test]
fn mtmdd2_gc_collects_both_subforests() {
    fn id(n: Node) -> NodeId {
        match n {
            Node::Value(x) | Node::Bool(x) => x,
        }
    }

    let mut dd: MtMdd2Manager<i32> = MtMdd2Manager::new();
    let h0 = dd.create_header(0, "x", 2);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let x = dd.create_node(h0, &[v0, v1]); // value-forest node
    let (z, o) = (dd.zero(), dd.one());
    let b = dd.create_node(h0, &[z, o]); // bool-forest node

    // garbage in the value forest
    let v2 = dd.value(9);
    let _g = dd.create_node(h0, &[v1, v2]);

    let live_before = dd.live_node_count();
    let (rv, _rb) = dd.gc(&[x, b]);
    assert!(rv > 0, "value-forest garbage should be reclaimed");
    assert!(dd.live_node_count() < live_before);
    // survivors re-cons to the same underlying ids
    assert_eq!(id(dd.create_node(h0, &[v0, v1])), id(x));
    assert_eq!(id(dd.create_node(h0, &[z, o])), id(b));
}
