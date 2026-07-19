use bddcore::prelude::*;

pub fn and(
    dd: &mut BddManager,
    node: &[NodeId]
) -> NodeId {
    let mut res = dd.one();
    for &n in node {
        res = dd.and(res, n)
    }
    res
}

pub fn or(
    dd: &mut BddManager,
    node: &[NodeId]
) -> NodeId {
    let mut res = dd.zero();
    for &n in node {
        res = dd.or(res, n)
    }
    res
}

/// BDD for "at least `k` of the given nodes are true".
///
/// Shannon expansion on `node[start]` with memoization on `(k, start)`: the
/// distinct subproblems are `(k, suffix)` and there are only O(n·k) of them, so
/// this is O(n·k) `ite` calls. (The naive un-memoized recursion recomputes each
/// subproblem and is O(2ⁿ) despite the polynomial-size result.)
pub fn kofn(
    dd: &mut BddManager,
    k: usize,
    node: &[NodeId]
) -> NodeId {
    let n = node.len();
    // Flat (k+1) x (n+1) memo table, valid only within this call.
    let mut memo: Vec<Option<NodeId>> = vec![None; (k + 1) * (n + 1)];
    kofn_rec(dd, node, k, 0, n, &mut memo)
}

fn kofn_rec(
    dd: &mut BddManager,
    node: &[NodeId],
    k: usize,
    start: usize,
    n: usize,
    memo: &mut [Option<NodeId>],
) -> NodeId {
    if k == 0 {
        return dd.one(); // "at least 0" is always true
    }
    let remaining = n - start;
    if k > remaining {
        return dd.zero(); // impossible: not enough variables left
    }
    let idx = k * (n + 1) + start;
    if let Some(v) = memo[idx] {
        return v;
    }
    let then = kofn_rec(dd, node, k - 1, start + 1, n, memo);
    let else_ = kofn_rec(dd, node, k, start + 1, n, memo);
    let res = dd.ite(node[start], then, else_);
    memo[idx] = Some(res);
    res
}
