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
    assert!(sum.minpath_checked().is_some(), "x + y is coherent");
    let mut mx = x.max(&y); // max(x, y)
    assert!(mx.minpath_checked().is_some(), "max(x,y) is coherent");
    // minpath() must not panic and must agree with the checked variant.
    assert_eq!(sum.minpath().size(), sum.minpath_checked().unwrap().size());

    // Non-coherent value: x - y decreases in y.
    let mut diff = x.sub(&y);
    assert!(diff.minpath_checked().is_none(), "x - y is not coherent");

    // Boolean structure functions via comparison.
    let one = mgr.value(1);
    let mut ge = x.ge(&one); // [x >= 1] : non-decreasing in x -> coherent
    assert!(ge.minpath_checked().is_some(), "[x>=1] is coherent");
    let mut lt = x.lt(&y); // [x < y] : decreasing in x -> not coherent
    assert!(lt.minpath_checked().is_none(), "[x<y] is not coherent");
    let mut eq = x.eq(&y); // [x == y] : not monotone
    assert!(eq.minpath_checked().is_none(), "[x==y] is not coherent");
}
