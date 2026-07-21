use std::collections::HashMap;

use bss::prelude::*;

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
fn test_minpath_monotone_detection() {
    let mut bss = BddMgr::new();
    let x = bss.defvar("x");
    let y = bss.defvar("y");
    let z = bss.defvar("z");

    // Monotone (coherent): built from positive literals with and/or -> Some.
    let mono = x.and(&y).or(&z);
    assert!(
        mono.minpath().is_some(),
        "and/or of positive literals must be detected monotone"
    );

    // Non-monotone functions -> None (early abort).
    assert!(x.xor(&y).minpath().is_none(), "xor is non-monotone");
    assert!(x.and(&y.not()).minpath().is_none(), "x & !y is non-monotone");
    assert!(z.not().minpath().is_none(), "!z is non-monotone");
}

#[test]
fn test_dual_and_mincut() {
    let mut bss = BddMgr::new();
    let x = bss.defvar("x");
    let y = bss.defvar("y");

    // Series system phi = x & y: min path = {x,y}; min cut = {x},{y}.
    let series = x.and(&y);
    // dual(x & y) == x | y
    assert!(series.dual().eq(&x.or(&y)), "dual(x&y) must equal x|y");
    // mincut count: series has 2 minimal cut vectors ({x}, {y}).
    let cut = series.mincut().expect("x&y is coherent");
    assert_eq!(cut.bdd_count(&[true]), 2, "series -> 2 min cut vectors");
    // min path of series: 1 minimal path vector ({x,y}).
    assert_eq!(series.minpath().unwrap().bdd_count(&[true]), 1);

    // Parallel system phi = x | y: dual is x & y; min cut = {x,y} (1 vector).
    let parallel = x.or(&y);
    assert!(parallel.dual().eq(&x.and(&y)), "dual(x|y) must equal x&y");
    assert_eq!(parallel.mincut().unwrap().bdd_count(&[true]), 1);
    assert_eq!(parallel.minpath().unwrap().bdd_count(&[true]), 2);

    // dual is an involution: dual(dual(phi)) == phi.
    assert!(series.dual().dual().eq(&series), "dual is an involution");

    // Non-monotone -> mincut None.
    assert!(x.xor(&y).mincut().is_none());
}
