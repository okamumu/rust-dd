use std::collections::HashMap;

use mss::prelude::*;

/// `bmeas[x][d]` must equal `P(φ∈ss | x=d+1) − P(φ∈ss | x=d)`, where each conditional is
/// `prob` computed with `x` pinned to that state (its probability vector replaced by a unit
/// vector). Verified for every (variable, transition) on both the value forest
/// (`max(min(x,y),z)`) and the boolean forest (`[x>=1] & [y>=2]`).
#[test]
fn test_bmeas_matches_pinned_prob() {
    fn check(mgr_build: impl Fn(&mut MddMgr<i32>) -> MddNode<i32>, k: usize) {
        let mut mgr: MddMgr<i32> = MddMgr::new();
        let mut node = mgr_build(&mut mgr);
        let states = 3usize;
        // Arbitrary non-degenerate probability vectors per variable.
        let base: HashMap<String, Vec<f64>> = [
            ("x", vec![0.2, 0.3, 0.5]),
            ("y", vec![0.5, 0.1, 0.4]),
            ("z", vec![0.25, 0.25, 0.5]),
        ]
        .iter()
        .map(|(s, v)| (s.to_string(), v.clone()))
        .collect();
        let ss: Vec<i32> = (1..k as i32).collect(); // success = performance >= 1

        // P(φ∈ss | var = j): pin `var` to state j (unit vector e_j).
        let cond = |node: &mut MddNode<i32>, var: &str, j: usize| {
            let mut pinned = base.clone();
            let mut e = vec![0.0; states];
            e[j] = 1.0;
            pinned.insert(var.to_string(), e);
            node.prob(&pinned, &ss)
        };

        let bm = node.bmeas(&base, &ss);
        for (var, vec) in &bm {
            assert_eq!(vec.len(), states - 1, "one difference per state boundary");
            for (d, &g) in vec.iter().enumerate() {
                let expected = cond(&mut node, var, d + 1) - cond(&mut node, var, d);
                assert!(
                    (g - expected).abs() < 1e-9,
                    "bmeas[{var}][{d}] = {g}, expected P(.|{var}={}) - P(.|{var}={d}) = {expected}",
                    d + 1
                );
            }
        }
    }
    // value forest: max(min(x,y), z), K = 3 (values 0..2)
    check(
        |mgr| {
            let x = mgr.defvar("x", 3);
            let y = mgr.defvar("y", 3);
            let z = mgr.defvar("z", 3);
            x.min(&y).max(&z)
        },
        3,
    );
    // boolean forest: [x>=1] & [y>=2]  (a Bool-tagged structure function)
    check(
        |mgr| {
            let x = mgr.defvar("x", 3);
            let y = mgr.defvar("y", 3);
            let one = mgr.value(1);
            let two = mgr.value(2);
            x.ge(&one).and(&y.ge(&two))
        },
        2,
    );
}

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
    let mut mgr: MssMgr<i32> = MssMgr::new();
    let x = mgr.defvar("x", 3);
    let y = mgr.defvar("y", 3);

    // Coherent value functions (non-decreasing in every component) -> Some.
    let sum = x.add(&y); // x + y
    assert!(mgr.minpath(&sum).is_some(), "x + y is coherent");
    let mx = x.max(&y); // max(x, y)
    assert!(mgr.minpath(&mx).is_some(), "max(x,y) is coherent");

    // Non-coherent value: x - y decreases in y.
    let diff = x.sub(&y);
    assert!(mgr.minpath(&diff).is_none(), "x - y is not coherent");

    // Boolean structure functions via comparison.
    let one = mgr.value(1);
    let ge = x.ge(&one); // [x >= 1] : non-decreasing in x -> coherent
    assert!(mgr.minpath(&ge).is_some(), "[x>=1] is coherent");
    let lt = x.lt(&y); // [x < y] : decreasing in x -> not coherent
    assert!(mgr.minpath(&lt).is_none(), "[x<y] is not coherent");
    let eq = x.eq(&y); // [x == y] : not monotone
    assert!(mgr.minpath(&eq).is_none(), "[x==y] is not coherent");
}

/// Regression for the minsol non-minimal bug (the `without` (NonTerminal, Terminal)
/// case must recurse into f's zero branch, not expand every branch). `max(min(x,y), z)`
/// is the multi-state analogue of `x&y|z`; its minimal path vectors are exactly
/// {z=1},{z=2},{x=1,y=1},{x=2,y=2} — the old code fabricated non-minimal vectors like
/// {y=1,z=2}.
#[test]
fn test_minpath_no_spurious_vectors() {
    use std::collections::HashSet;
    let mut mgr: MssMgr<i32> = MssMgr::new();
    let x = mgr.defvar("x", 3);
    let y = mgr.defvar("y", 3);
    let z = mgr.defvar("z", 3);

    let phi = x.min(&y).max(&z); // max(min(x,y), z)
    let mp = mgr.minpath(&phi).expect("max(min(x,y),z) is coherent");

    let ss: HashSet<i32> = [1, 2].into_iter().collect();
    let mut got: Vec<Vec<(String, usize)>> = mp
        .extract(&ss)
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

#[test]
fn test_zmdd_intersect_setdiff() {
    use std::collections::HashSet;

    fn sorted_vecs(z: &ZmddNode<i32>, ss: &HashSet<i32>) -> Vec<Vec<(String, usize)>> {
        let mut v: Vec<Vec<(String, usize)>> = z
            .extract(ss)
            .map(|d| {
                let mut e: Vec<(String, usize)> = d.into_iter().collect();
                e.sort();
                e
            })
            .collect();
        v.sort();
        v
    }
    fn set(items: &[(&str, usize)]) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = items.iter().map(|(s, n)| (s.to_string(), *n)).collect();
        v.sort();
        v
    }

    let mut mgr: MssMgr<i32> = MssMgr::new();
    let x = mgr.defvar("x", 3);
    let y = mgr.defvar("y", 3);
    let z = mgr.defvar("z", 3);

    // f = max(min(x,y), z): MPV {z=1},{z=2},{x=1,y=1},{x=2,y=2}
    let f = x.min(&y).max(&z);
    // g = min(x,y): MPV {x=1,y=1},{x=2,y=2}
    let g = x.min(&y);
    let a = mgr.minpath(&f).expect("coherent");
    let b = mgr.minpath(&g).expect("coherent");

    let ss: HashSet<i32> = [1, 2].into_iter().collect();

    // intersect (label-wise): {x=1,y=1},{x=2,y=2}
    let inter = a.intersect(&b);
    assert_eq!(
        sorted_vecs(&inter, &ss),
        vec![set(&[("x", 1), ("y", 1)]), set(&[("x", 2), ("y", 2)])]
    );
    assert_eq!(inter.count(&ss), 2);

    // setdiff a - b (label-wise): {z=1},{z=2}
    let diff = a.setdiff(&b);
    assert_eq!(
        sorted_vecs(&diff, &ss),
        vec![set(&[("z", 1)]), set(&[("z", 2)])]
    );
    assert_eq!(diff.count(&ss), 2);
}
