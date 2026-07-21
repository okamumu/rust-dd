use std::collections::HashMap;

use mss::prelude::*;

#[test]
fn test_mdd_mgr() {
    let mut mgr: MddMgr<i32> = MddMgr::new();
    let x = mgr.defvar("x", 3);
    let y = mgr.defvar("y", 3);
    let z = mgr.defvar("z", 3);
    // let zero = mgr.zero();
    // let one = mgr.one();
    // let two = mgr.val(2);
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 3);
    vars.insert("y".to_string(), 3);
    vars.insert("z".to_string(), 3);
    let rpn = "x y z + *";
    if let Ok(node) = mgr.rpn(rpn, &vars) {
        println!("{}", node.dot());
    }
}

#[test]
fn test_minpath_coherence_detection() {
    let mut mgr: MddMgr<i32> = MddMgr::new();
    let x = mgr.defvar("x", 3);
    let y = mgr.defvar("y", 3);

    // Coherent value functions (non-decreasing in every component) -> Some.
    let mut sum = x.add(&y); // x + y
    assert!(sum.minpath().is_some(), "x + y is coherent");
    let mut mx = x.max(&y); // max(x, y)
    assert!(mx.minpath().is_some(), "max(x,y) is coherent");

    // Non-coherent value: x - y decreases in y.
    let mut diff = x.sub(&y);
    assert!(diff.minpath().is_none(), "x - y is not coherent");

    // Boolean structure functions via comparison.
    let one = mgr.value(1);
    let mut ge = x.ge(&one); // [x >= 1] : non-decreasing in x -> coherent
    assert!(ge.minpath().is_some(), "[x>=1] is coherent");
    let mut lt = x.lt(&y); // [x < y] : decreasing in x -> not coherent
    assert!(lt.minpath().is_none(), "[x<y] is not coherent");
    let mut eq = x.eq(&y); // [x == y] : not monotone
    assert!(eq.minpath().is_none(), "[x==y] is not coherent");
}

/// Regression for the minsol non-minimal bug (the `without` (NonTerminal, Terminal)
/// case must recurse into f's zero branch, not expand every branch). `max(min(x,y), z)`
/// is the multi-state analogue of `x&y|z`; its minimal path vectors are exactly
/// {z=1},{z=2},{x=1,y=1},{x=2,y=2} — the old code fabricated non-minimal vectors like
/// {y=1,z=2}.
#[test]
fn test_minpath_no_spurious_vectors() {
    use std::collections::HashSet;
    let mut mgr: MddMgr<i32> = MddMgr::new();
    let x = mgr.defvar("x", 3);
    let y = mgr.defvar("y", 3);
    let z = mgr.defvar("z", 3);

    let mut phi = x.min(&y).max(&z); // max(min(x,y), z)
    let mp = phi.minpath().expect("max(min(x,y),z) is coherent");

    let ss: HashSet<i32> = [1, 2].into_iter().collect();
    let mut got: Vec<Vec<(String, usize)>> = mp
        .zmdd_extract(&ss)
        .map(|d| {
            let mut v: Vec<(String, usize)> =
                d.into_iter().filter(|(_, val)| *val != 0).collect();
            v.sort();
            v
        })
        .collect();
    got.sort();

    let mk = |items: &[(&str, usize)]| {
        let mut v: Vec<(String, usize)> =
            items.iter().map(|(s, n)| (s.to_string(), *n)).collect();
        v.sort();
        v
    };
    let mut expected = vec![
        mk(&[("z", 1)]),
        mk(&[("z", 2)]),
        mk(&[("x", 1), ("y", 1)]),
        mk(&[("x", 2), ("y", 2)]),
    ];
    expected.sort();

    assert_eq!(got, expected, "minpath must be exactly the 4 minimal path vectors");
}
