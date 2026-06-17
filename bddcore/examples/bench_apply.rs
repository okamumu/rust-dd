//! Apply-heavy benchmark for BddManager / ZddManager.
//!
//! Builds the BDD for an n-bit array multiplier (the canonical apply stressor:
//! the middle output bits have large BDDs regardless of variable order) and a
//! ZDD product workload, timing total construction. Result-affecting quantities
//! (output node ids, total node count) are printed so before/after refactors can
//! be checked for identical behavior.
//!
//! Run: cargo run --release --example bench_apply

use bddcore::prelude::*;
use std::time::Instant;

/// Create a fresh BDD variable at the given level.
fn var(dd: &mut BddManager, level: Level, label: &str) -> NodeId {
    let h = dd.create_header(level, label);
    let (z, o) = (dd.zero(), dd.one());
    dd.create_node(h, z, o)
}

/// One full adder: returns (sum, carry_out).
fn full_adder(dd: &mut BddManager, a: NodeId, b: NodeId, cin: NodeId) -> (NodeId, NodeId) {
    let axb = dd.xor(a, b);
    let sum = dd.xor(axb, cin);
    let ab = dd.and(a, b);
    let acin = dd.and(axb, cin);
    let carry = dd.or(ab, acin);
    (sum, carry)
}

/// Build the BDD for every output bit of an n-bit array multiplier.
/// Returns the output node ids.
fn multiplier(dd: &mut BddManager, n: usize) -> Vec<NodeId> {
    // Interleaved variable order a0,b0,a1,b1,... (forces large intermediates).
    let mut a = Vec::with_capacity(n);
    let mut b = Vec::with_capacity(n);
    for i in 0..n {
        a.push(var(dd, 2 * i, &format!("a{i}")));
        b.push(var(dd, 2 * i + 1, &format!("b{i}")));
    }

    // partial[i][j] = a[i] & b[j]
    let zero = dd.zero();
    let mut out = vec![zero; 2 * n];

    // Shift-add accumulation: out += (a & b[j]) << j
    for j in 0..n {
        let mut carry = dd.zero();
        for i in 0..n {
            let pp = dd.and(a[i], b[j]);
            let pos = i + j;
            let (s, c) = full_adder(dd, out[pos], pp, carry);
            out[pos] = s;
            carry = c;
        }
        // propagate final carry upward
        let mut pos = n + j;
        while carry != dd.zero() && pos < 2 * n {
            let (s, c) = full_adder(dd, out[pos], dd.zero(), carry);
            out[pos] = s;
            carry = c;
            pos += 1;
        }
    }
    out
}

fn bench_bdd(n: usize) {
    let start = Instant::now();
    let mut dd = BddManager::new();
    let out = multiplier(&mut dd, n);
    let elapsed = start.elapsed();
    let (headers, nodes, cache) = dd.size();
    let checksum: usize = out.iter().sum();
    println!(
        "BDD mult n={n}: {:.4}s  headers={headers} nodes={nodes} cache={cache} checksum={checksum}",
        elapsed.as_secs_f64()
    );
}

/// One apply-heavy ZDD build on a fresh manager. Returns (nodes, checksum).
fn zdd_build(n: usize) -> (usize, usize) {
    let mut dd = ZddManager::new();
    let base = dd.one();
    let mut vars = Vec::with_capacity(n);
    for i in 0..n {
        let h = dd.create_header(i, &format!("e{i}"));
        let z = dd.zero();
        vars.push(dd.create_node(h, z, base));
    }
    // A = family of all 2-element subsets {e_i, e_j}  (irregular -> many nodes)
    let mut a = dd.zero();
    for i in 0..n {
        for j in (i + 1)..n {
            let m = dd.product(vars[i], vars[j]);
            a = dd.union(a, m);
        }
    }
    // heavy: square it (subsets up to size 4) and combine with all set ops
    let g = dd.product(a, a);
    let u = dd.union(a, g);
    let isect = dd.intersect(a, g);
    let d = dd.setdiff(g, a);
    let (_, nodes, _) = dd.size();
    (nodes, u + isect + d + g)
}

fn bench_zdd(n: usize) {
    const REPEAT: usize = 60;
    let start = Instant::now();
    let mut last = (0, 0);
    for _ in 0..REPEAT {
        last = zdd_build(n);
    }
    let elapsed = start.elapsed();
    let (nodes, checksum) = last;
    println!(
        "ZDD heavy n={n} x{REPEAT}: {:.4}s  nodes={nodes} checksum={checksum}",
        elapsed.as_secs_f64()
    );
}

/// Identical-operand / cold-cache workload: exercises the `f == g` short-circuit.
/// Builds a sizable BDD, then repeatedly clears the op-cache and computes
/// and(f,f)/or(f,f). With the short-circuit these are O(1); otherwise each is a
/// full O(|f|) recursion. Mirrors the post-GC world where the cache is flushed.
fn bench_reuse(n: usize) {
    let mut dd = BddManager::new();
    // f = OR of all multiplier output bits -> a non-trivial shared DAG.
    let out = multiplier(&mut dd, n);
    let mut f = dd.zero();
    for &o in &out {
        f = dd.or(f, o);
    }
    let (_, nodes, _) = dd.size();

    const REPEAT: usize = 2000;
    let start = Instant::now();
    let mut acc = 0usize;
    for _ in 0..REPEAT {
        dd.clear_cache();
        let a = dd.and(f, f);
        let o = dd.or(f, f);
        acc += a + o;
    }
    let elapsed = start.elapsed();
    println!(
        "reuse and/or(f,f) |f|={nodes} x{REPEAT}: {:.4}s  acc_ok={}",
        elapsed.as_secs_f64(),
        acc == 2 * REPEAT * f
    );
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9);
    bench_bdd(n);
    bench_zdd(18);
    bench_reuse(7);
}
