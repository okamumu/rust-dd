use std::ops::{Add, Mul};
use bddcore::prelude::*;

fn power<T>(x: T, n: usize) -> T
where
    T: Mul<Output = T> + From<u32> + Clone,
{
    let mut result = T::from(1);
    for _ in 0..n {
        result = result * x.clone();
    }
    result
}

pub fn node_count<T>(
    dd: &BddManager,
    node: NodeId,
    cache: &mut BddHashSet<NodeId>,
) -> (T, T, T)
where
    T: Add<Output = T> + Clone + From<u32>,
{
    let key = node;
    if cache.contains(&key) {
        return (T::from(0), T::from(0), T::from(1));
    }
    let result = match dd.get_node(&node).unwrap() {
        Node::One | Node::Zero | Node::Undet => {
            (T::from(0), T::from(1), T::from(1))
        }
        Node::NonTerminal(fnode) => {
            let (n0, v0, e0): (T, T, T) = node_count(dd, fnode[0], cache);
            let (n1, v1, e1): (T, T, T) = node_count(dd, fnode[1], cache);
            (n0 + n1 + T::from(1), v0 + v1, e0 + e1 + T::from(1))
        }
    };
    cache.insert(key);
    result
}

pub fn bdd_count<T>(
    dd: &BddManager,
    ss: &[bool],
    node: NodeId,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(&node).unwrap() {
        Node::One => {
            if ss.contains(&true) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::Zero => {
            if ss.contains(&false) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let current_level = dd.level(&node).unwrap();
            if let Some(next_level) = dd.level(&fnode[0]) {
                result = result
                    + power(T::from(2), current_level - next_level - 1)
                        * bdd_count(dd, ss, fnode[0], cache);
            } else {
                result =
                    result + power(T::from(2), current_level) * bdd_count(dd, ss, fnode[0], cache);
            }
            if let Some(next_level) = dd.level(&fnode[1]) {
                result = result
                    + power(T::from(2), current_level - next_level - 1)
                        * bdd_count(dd, ss, fnode[1], cache);
            } else {
                result =
                    result + power(T::from(2), current_level) * bdd_count(dd, ss, fnode[1], cache);
            }
            result
        }
        Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

pub fn zdd_count<T>(
    dd: &BddManager,
    ss: &[bool],
    node: NodeId,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32>,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match dd.get_node(&node).unwrap() {
        Node::One => {
            if ss.contains(&true) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::Zero => {
            if ss.contains(&false) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        Node::NonTerminal(fnode) => {
            zdd_count(dd, ss, fnode[0], cache) + zdd_count(dd, ss, fnode[1], cache)
        }
        Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}
