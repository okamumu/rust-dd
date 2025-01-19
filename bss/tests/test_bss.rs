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