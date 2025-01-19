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
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
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
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
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
