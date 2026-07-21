use std::collections::{HashMap, HashSet};
use std::ops::{Add, Mul, Sub};

use mddcore::prelude::*;

pub fn prob<V, T>(
    mdd: &mut MtMdd2Manager<V>,
    node: &Node,
    pv: &HashMap<String, Vec<T>>,
    ss: &HashSet<V>,
) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MddValue,
{
    match node {
        Node::Value(fnode) => {
            let mut cache = BddHashMap::default();
            vprob(&mut mdd.mtmdd_mut(), *fnode, &pv, ss, &mut cache)
        }
        Node::Bool(fnode) => {
            let mut cache = BddHashMap::default();
            bprob(&mut mdd.mdd_mut(), *fnode, &pv, ss, &mut cache)
        }
    }
}

fn vprob<V, T>(
    mdd: &mut mtmdd::MtMddManager<V>,
    node: NodeId,
    pv: &HashMap<String, Vec<T>>,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MddValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(&node).unwrap() {
        mtmdd::Node::Terminal(fnode) => {
            let value = fnode.value();
            if ss.contains(&value) {
                T::from(1.0)
            } else {
                T::from(0.0)
            }
        }
        mtmdd::Node::NonTerminal(fnode) => {
            let label = mdd.label(&node).unwrap();
            let fp = pv.get(label).unwrap();
            let mut result = T::from(0.0);
            let fnodeid: Vec<_> = fnode.iter().collect();
            for (i, x) in fnodeid.into_iter().enumerate() {
                let tmp = vprob(mdd, x, pv, ss, cache);
                result = result + fp[i] * tmp;
            }
            result
        }
        mtmdd::Node::Undet => T::from(0.0),
    };
    cache.insert(key, result.clone());
    result
}

fn bprob<V, T>(
    mdd: &mut mdd::MddManager,
    node: NodeId,
    pv: &HashMap<String, Vec<T>>,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MddValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(&node).unwrap() {
        mdd::Node::Zero => {
            if ss.contains(&V::from(0)) {
                T::from(1.0)
            } else {
                T::from(0.0)
            }
        }
        mdd::Node::One => {
            if ss.contains(&V::from(1)) {
                T::from(1.0)
            } else {
                T::from(0.0)
            }
        }
        mdd::Node::NonTerminal(fnode) => {
            let label = mdd.label(&node).unwrap();
            let fp = pv.get(label).unwrap();
            let mut result = T::from(0.0);
            let fnodeid: Vec<_> = fnode.iter().collect();
            for (i, x) in fnodeid.into_iter().enumerate() {
                let tmp = bprob(mdd, x, pv, ss, cache);
                result = result + fp[i] * tmp;
            }
            result
        }
        mdd::Node::Undet => T::from(0.0),
    };
    cache.insert(key, result.clone());
    result
}

/// Multi-state Birnbaum importance of every variable, computed by **backward differentiation**
/// (reverse-mode gradient), the multi-state generalization of `bss::bdd_prob::bmeas`.
///
/// For a structure function `φ` and success set `ss`, this returns, per variable `i`, the
/// vector of **adjacent-state differences**
/// `D_{i,j} = P(φ∈ss | x_i = j) − P(φ∈ss | x_i = j−1)` for `j = 1 .. M_i-1`
/// (so the returned vector has length `M_i − 1`; index `d` is the transition `d → d+1`).
/// For a binary variable (`M_i = 2`) this is the single value
/// `P(φ∈ss | x_i=1) − P(φ∈ss | x_i=0)` — exactly the BSS Birnbaum measure.
///
/// The **difference** form (not the raw partial `∂P/∂p_{i,j}`) is the correct quantity on a
/// *reduced* diagram: where a variable is skipped on a path it is irrelevant to `φ` there, and
/// its two conditional probabilities are equal, so that path cancels out of every `D_{i,j}` —
/// just as skipped variables cancel in the BSS `p1 − p0`. The raw partial would instead
/// silently drop those paths.
///
/// One topological pass propagates the adjoint weight `w_f` (the probability of reaching node
/// `f`) down the diagram (`w_root = 1`, `w_{edge_j} += w_f · p_{i,j}`) and accumulates
/// `D_{i,j} += w_f · (prob(edge_j) − prob(edge_{j-1}))` at each node labeled `i`.
pub fn bmeas<V, T>(
    mdd: &mut MtMdd2Manager<V>,
    node: &Node,
    pv: &HashMap<String, Vec<T>>,
    ss: &HashSet<V>,
) -> HashMap<String, Vec<T>>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MddValue,
{
    match node {
        Node::Value(fnode) => vbmeas(&mut mdd.mtmdd_mut(), *fnode, pv, ss),
        Node::Bool(fnode) => bbmeas(&mut mdd.mdd_mut(), *fnode, pv, ss),
    }
}

/// Post-order DFS reversed = topological order with the root first, so that when a node is
/// visited every ancestor has already contributed to its adjoint weight. The diagram is a
/// DAG, so a plain visited-set (no cycle detection) suffices.
fn topo_postorder<F>(f: NodeId, visited: &mut BddHashSet<NodeId>, order: &mut Vec<NodeId>, children: &F)
where
    F: Fn(NodeId) -> Vec<NodeId>,
{
    if !visited.insert(f) {
        return;
    }
    for c in children(f) {
        topo_postorder(c, visited, order, children);
    }
    order.push(f);
}

fn vbmeas<V, T>(
    mdd: &mut mtmdd::MtMddManager<V>,
    node: NodeId,
    pv: &HashMap<String, Vec<T>>,
    ss: &HashSet<V>,
) -> HashMap<String, Vec<T>>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MddValue,
{
    let mut order = Vec::new();
    let mut visited = BddHashSet::default();
    let children = |f: NodeId| match mdd.get_node(&f).unwrap() {
        mtmdd::Node::NonTerminal(fnode) => fnode.iter().collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    topo_postorder(node, &mut visited, &mut order, &children);
    order.reverse();

    let mut gradcache: HashMap<NodeId, T> = HashMap::new();
    let mut probcache: BddHashMap<NodeId, T> = BddHashMap::default();
    let mut gradevent: HashMap<String, Vec<T>> = HashMap::new();
    gradcache.insert(node, T::from(1.0));
    for f in order {
        let (label, edges): (String, Vec<NodeId>) = match mdd.get_node(&f).unwrap() {
            mtmdd::Node::NonTerminal(fnode) => {
                (mdd.label(&f).unwrap().to_string(), fnode.iter().collect())
            }
            _ => continue,
        };
        let w = *gradcache.get(&f).unwrap_or(&T::from(0.0));
        let m = edges.len();
        let pj: Vec<T> = pv.get(&label).unwrap().clone();
        let mut probs = vec![T::from(0.0); m];
        for (j, &e) in edges.iter().enumerate() {
            let g = gradcache.get(&e).copied().unwrap_or(T::from(0.0)) + w * pj[j];
            gradcache.insert(e, g);
            probs[j] = vprob(mdd, e, pv, ss, &mut probcache);
        }
        // adjacent-state Birnbaum differences (length m-1); skipped-variable paths cancel
        let slot = gradevent
            .entry(label.clone())
            .or_insert_with(|| vec![T::from(0.0); m - 1]);
        for d in 0..m.saturating_sub(1) {
            slot[d] = slot[d] + w * (probs[d + 1] - probs[d]);
        }
    }
    gradevent
}

fn bbmeas<V, T>(
    mdd: &mut mdd::MddManager,
    node: NodeId,
    pv: &HashMap<String, Vec<T>>,
    ss: &HashSet<V>,
) -> HashMap<String, Vec<T>>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
    V: MddValue,
{
    let mut order = Vec::new();
    let mut visited = BddHashSet::default();
    let children = |f: NodeId| match mdd.get_node(&f).unwrap() {
        mdd::Node::NonTerminal(fnode) => fnode.iter().collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    topo_postorder(node, &mut visited, &mut order, &children);
    order.reverse();

    let mut gradcache: HashMap<NodeId, T> = HashMap::new();
    let mut probcache: BddHashMap<NodeId, T> = BddHashMap::default();
    let mut gradevent: HashMap<String, Vec<T>> = HashMap::new();
    gradcache.insert(node, T::from(1.0));
    for f in order {
        let (label, edges): (String, Vec<NodeId>) = match mdd.get_node(&f).unwrap() {
            mdd::Node::NonTerminal(fnode) => {
                (mdd.label(&f).unwrap().to_string(), fnode.iter().collect())
            }
            _ => continue,
        };
        let w = *gradcache.get(&f).unwrap_or(&T::from(0.0));
        let m = edges.len();
        let pj: Vec<T> = pv.get(&label).unwrap().clone();
        let mut probs = vec![T::from(0.0); m];
        for (j, &e) in edges.iter().enumerate() {
            let g = gradcache.get(&e).copied().unwrap_or(T::from(0.0)) + w * pj[j];
            gradcache.insert(e, g);
            probs[j] = bprob(mdd, e, pv, ss, &mut probcache);
        }
        // adjacent-state Birnbaum differences (length m-1); skipped-variable paths cancel
        let slot = gradevent
            .entry(label.clone())
            .or_insert_with(|| vec![T::from(0.0); m - 1]);
        for d in 0..m.saturating_sub(1) {
            slot[d] = slot[d] + w * (probs[d + 1] - probs[d]);
        }
    }
    gradevent
}
