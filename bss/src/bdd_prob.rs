use bddcore::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::ops::{Add, Sub, Mul};

pub fn prob<T>(
    dd: &BddManager,
    node: NodeId,
    pv: &HashMap<String, T>,
    ss: &[bool],
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(&node).unwrap() {
        Node::Zero => {
            if ss.contains(&false) {
                T::from(1.0)
            } else {
                T::from(0.0)
            }
        }
        Node::One => {
            if ss.contains(&true) {
                T::from(1.0)
            } else {
                T::from(0.0)
            }
        }
        Node::NonTerminal(fnode) => {
            let x = dd.label(&node).unwrap();
            let fp = *pv.get(x).unwrap_or(&T::from(0.0));
            let low = prob(dd, fnode[0], pv, ss, cache);
            let high = prob(dd, fnode[1], pv, ss, cache);
            (T::from(1.0) - fp) * low + fp * high
        }
        Node::Undet => panic!("Undetermined node"),
    };
    cache.insert(key, result);
    result
}

pub fn bmeas<T>(
    dd: &BddManager,
    ss: &[bool],
    node: NodeId,
    env: &HashMap<String, T>,
) -> HashMap<String, T>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Clone + Copy + PartialEq + From<f64>,
{
    let sorted_nodes = topological_sort(dd, node);
    let mut gradcache = HashMap::new();
    let mut bddcache = BddHashMap::default();
    let mut gradevent = HashMap::new();
    gradcache.insert(node, T::from(1.0));
    for f in sorted_nodes {
        match dd.get_node(&f).unwrap() {
            Node::Zero | Node::One | Node::Undet => (),
            Node::NonTerminal(fnode) => {
                let w = *gradcache.get(&fnode.id()).unwrap_or(&T::from(0.0));
                let x = dd.label(&f).unwrap();
                let p = *env.get(x).unwrap_or(&T::from(0.0));
                let barp = T::from(1.0) - p;
                let result0 = if let Some(&val) = gradcache.get(&fnode[0]) {
                    val + w * barp
                } else {
                    w * barp
                };
                gradcache.insert(fnode[0], result0);
                let result1 = if let Some(&val) = gradcache.get(&fnode[1]) {
                    val + w * p
                } else {
                    w * p
                };
                gradcache.insert(fnode[1], result1);
                let p0 = prob(dd, fnode[0], env, ss, &mut bddcache);
                let p1 = prob(dd, fnode[1], env, ss, &mut bddcache);
                let resultv = if let Some(&val) = gradevent.get(x) {
                    val + w * (p1 - p0)
                } else {
                    w * (p1 - p0)
                };
                gradevent.insert(x.to_string(), resultv);
            }
        }
    }
    gradevent
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckedState {
    Persistent,
    Temporary,
}

fn topological_sort(dd: &BddManager, f: NodeId) -> Vec<NodeId> {
    let mut result = Vec::new();
    let mut check = BddHashMap::default();
    let mut queue = VecDeque::new();
    queue.push_back(f.clone());
    while let Some(node) = queue.pop_front() {
        match check.get(&node) {
            Some(&CheckedState::Temporary) => panic!("DAG has a closed path"),
            Some(&CheckedState::Persistent) => (),
            None => {
                visit(dd, node, &mut check, &mut result, &mut queue);
            }
        }
    }
    result.reverse();
    result
}

fn visit(
    dd: &BddManager,
    x: NodeId,
    check: &mut BddHashMap<NodeId, CheckedState>,
    result: &mut Vec<NodeId>,
    queue: &mut VecDeque<NodeId>,
) {
    match check.get(&x) {
        Some(&CheckedState::Temporary) => panic!("DAG has a closed path"),
        Some(&CheckedState::Persistent) => (),
        None => {
            check.insert(x, CheckedState::Temporary);
            match dd.get_node(&x).unwrap() {
                Node::Zero | Node::One | Node::Undet => (),
                Node::NonTerminal(fnode) => {
                    for &m in fnode.iter() {
                        queue.push_back(m);
                        visit(dd, m, check, result, queue);
                    }
                }
            }
            check.insert(x, CheckedState::Persistent);
            result.push(x);
        }
    }
}
