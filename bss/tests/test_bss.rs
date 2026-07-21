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
    let mut bss = BssMgr::new();
    let x = bss.defvar("x");
    let y = bss.defvar("y");
    let z = bss.defvar("z");

    // Monotone (coherent): built from positive literals with and/or -> Some.
    let mono = x.and(&y).or(&z);
    assert!(
        bss.minpath(&mono).is_some(),
        "and/or of positive literals must be detected monotone"
    );

    // Non-monotone functions -> None (early abort).
    assert!(bss.minpath(&x.xor(&y)).is_none(), "xor is non-monotone");
    assert!(bss.minpath(&x.and(&y.not())).is_none(), "x & !y is non-monotone");
    assert!(bss.minpath(&z.not()).is_none(), "!z is non-monotone");
}

/// Collect a ZDD family as a sorted `Vec<Vec<String>>` for structural comparison.
fn sorted_sets(z: &ZddNode) -> Vec<Vec<String>> {
    let mut v: Vec<Vec<String>> = z
        .extract(&[true])
        .map(|mut s| {
            s.sort();
            s
        })
        .collect();
    v.sort();
    v
}

fn sets(items: &[&[&str]]) -> Vec<Vec<String>> {
    let mut v: Vec<Vec<String>> = items
        .iter()
        .map(|s| {
            let mut inner: Vec<String> = s.iter().map(|x| x.to_string()).collect();
            inner.sort();
            inner
        })
        .collect();
    v.sort();
    v
}

#[test]
fn test_dual_and_mincut() {
    let mut bss = BssMgr::new();
    let x = bss.defvar("x");
    let y = bss.defvar("y");

    // Series system phi = x & y: min path = {x,y}; min cut = {x},{y}.
    let series = x.and(&y);
    // dual(x & y) == x | y  (dual stays a BddNode op)
    assert!(series.dual().eq(&x.or(&y)), "dual(x&y) must equal x|y");
    // mincut: series has 2 minimal cut vectors ({x}, {y}); minpath: 1 ({x,y}).
    let cut = bss.mincut(&series).expect("x&y is coherent");
    assert_eq!(cut.count(&[true]), 2, "series -> 2 min cut vectors");
    assert_eq!(sorted_sets(&cut), sets(&[&["x"], &["y"]]));
    let path = bss.minpath(&series).unwrap();
    assert_eq!(path.count(&[true]), 1);
    assert_eq!(sorted_sets(&path), sets(&[&["x", "y"]]));

    // Parallel system phi = x | y: dual is x & y; min cut = {x,y} (1 vector).
    let parallel = x.or(&y);
    assert!(parallel.dual().eq(&x.and(&y)), "dual(x|y) must equal x&y");
    assert_eq!(sorted_sets(&bss.mincut(&parallel).unwrap()), sets(&[&["x", "y"]]));
    assert_eq!(sorted_sets(&bss.minpath(&parallel).unwrap()), sets(&[&["x"], &["y"]]));

    // dual is an involution: dual(dual(phi)) == phi.
    assert!(series.dual().dual().eq(&series), "dual is an involution");

    // Non-monotone -> mincut None.
    assert!(bss.mincut(&x.xor(&y)).is_none());
}

#[test]
fn test_zdd_setops() {
    let mut bss = BssMgr::new();
    let x = bss.defvar("x");
    let y = bss.defvar("y");
    let z = bss.defvar("z");

    // A = minpath(x&y | z) = { {z}, {x,y} }; B = minpath(x | z) = { {x}, {z} }.
    let a = bss.minpath(&x.and(&y).or(&z)).unwrap();
    let b = bss.minpath(&x.or(&z)).unwrap();
    assert_eq!(sorted_sets(&a), sets(&[&["z"], &["x", "y"]]));
    assert_eq!(sorted_sets(&b), sets(&[&["x"], &["z"]]));

    // union = { {x,y}, {x}, {z} }; intersect = { {z} }; setdiff a\b = { {x,y} }.
    assert_eq!(sorted_sets(&a.union(&b)), sets(&[&["x", "y"], &["x"], &["z"]]));
    assert_eq!(sorted_sets(&a.intersect(&b)), sets(&[&["z"]]));
    assert_eq!(sorted_sets(&a.setdiff(&b)), sets(&[&["x", "y"]]));

    // product {x} * {y} = {x,y}; divide {x,y} / {y} = {x}.
    let fx = bss.minpath(&x).unwrap();
    let fy = bss.minpath(&y).unwrap();
    let prod = fx.product(&fy);
    assert_eq!(sorted_sets(&prod), sets(&[&["x", "y"]]));
    assert_eq!(sorted_sets(&prod.divide(&fy)), sets(&[&["x"]]));
}

/// Exhaustive check: for every boolean function of n≤3 variables, `minpath` must equal
/// the brute-force minimal true present-sets (monotone case) or be `None` (non-monotone).
/// (Also verified up to n=4 — 65536 functions — during development; kept at n=3 for speed.)
#[test]
fn test_minpath_exhaustive_brute_force() {
    for n in 1..=3usize {
        let npoints = 1usize << n;
        let nfuncs: u64 = 1u64 << npoints;
        for t in 0..nfuncs {
            let mut bss = BssMgr::new();
            let vars: Vec<_> = (0..n).map(|i| bss.defvar(&format!("x{}", i))).collect();
            // Build f from truth table t (bit a set => f true on assignment a).
            let mut f = bss.zero();
            for a in 0..npoints {
                if (t >> a) & 1 == 1 {
                    let mut m = bss.one();
                    for i in 0..n {
                        let lit = if (a >> i) & 1 == 1 { vars[i].clone() } else { vars[i].not() };
                        m = m.and(&lit);
                    }
                    f = f.or(&m);
                }
            }
            // Monotone-increasing?  a ⊆ b  =>  f(a) ≤ f(b).
            let is_mono = (0..npoints).all(|a| {
                (0..npoints).all(|b| (a & b) != a || ((t >> a) & 1) <= ((t >> b) & 1))
            });
            // Brute-force minimal true present-sets.
            let true_sets: Vec<usize> = (0..npoints).filter(|&a| (t >> a) & 1 == 1).collect();
            let mut expected: Vec<Vec<String>> = true_sets
                .iter()
                .filter(|&&a| !true_sets.iter().any(|&b| b != a && (b & a) == b))
                .map(|&a| {
                    let mut s: Vec<String> =
                        (0..n).filter(|&i| (a >> i) & 1 == 1).map(|i| format!("x{}", i)).collect();
                    s.sort();
                    s
                })
                .collect();
            expected.sort();

            let mp = bss.minpath(&f);
            if is_mono {
                let got = sorted_sets(&mp.expect("monotone must give Some"));
                assert_eq!(got, expected, "minpath mismatch n={} t={:#b}", n, t);
            } else {
                assert!(mp.is_none(), "non-monotone must give None n={} t={:#b}", n, t);
            }
        }
    }
}

#[test]
fn test_zdd_standalone() {
    let mut z = ZddMgr::new();
    // empty / base
    assert_eq!(z.empty().count(&[true]), 0);
    assert_eq!(z.base().count(&[true]), 1);          // {∅}
    assert_eq!(sorted_sets(&z.base()), sets(&[&[]])); // one set: the empty set

    // from_sets and set algebra
    let a = z.from_sets(&[vec!["x".into(), "y".into()], vec!["z".into()]]); // { {x,y}, {z} }
    let b = z.singleton("x").union(&z.singleton("z"));                       // { {x}, {z} }
    assert_eq!(sorted_sets(&a), sets(&[&["x", "y"], &["z"]]));
    assert_eq!(sorted_sets(&b), sets(&[&["x"], &["z"]]));

    assert_eq!(sorted_sets(&a.union(&b)), sets(&[&["x", "y"], &["x"], &["z"]]));
    assert_eq!(sorted_sets(&a.intersect(&b)), sets(&[&["z"]]));
    assert_eq!(sorted_sets(&a.setdiff(&b)), sets(&[&["x", "y"]]));

    // product {x} * {y} = {x,y}; divide back
    let prod = z.singleton("x").product(&z.singleton("y"));
    assert_eq!(sorted_sets(&prod), sets(&[&["x", "y"]]));
    assert_eq!(sorted_sets(&prod.divide(&z.singleton("y"))), sets(&[&["x"]]));
}
