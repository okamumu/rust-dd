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

pub fn kofn(
    dd: &mut BddManager,
    k: usize,
    node: &[NodeId]
) -> NodeId {
    match k {
        _ if k == 1 => or(dd, node),
        _ if k == node.len() => and(dd, node),
        _ => {
            let cond = node[0];
            let then = kofn(dd, k - 1, &node[1..]);
            let else_ = kofn(dd, k, &node[1..]);
            dd.ite(cond, then, else_)
        }
    }
}
