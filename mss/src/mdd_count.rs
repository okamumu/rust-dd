use std::ops::{Add, Sub, Mul};
use std::collections::HashSet;

use mddcore::prelude::*;

pub fn mddnode_count<V, T>(
    dd: &MtMdd2Manager<V>,
    node: &Node) -> (T, T, T)
where
    T: Add<Output = T> + Sub<Output = T> + Clone + From<u32>,
    V: MddValue,
{
    match node {
        Node::Value(fnode) => {
            let mut cache = BddHashSet::default();
            let (nn, nv, ne) = vmddnode_count(dd, *fnode, &mut cache);
            (nn, nv, ne-T::from(1))
        }
        Node::Bool(fnode) => {
            let mut cache = BddHashSet::default();
            let (nn, nv, ne) = bmddnode_count(dd, *fnode, &mut cache);
            (nn, nv, ne-T::from(1))
        }
    }
}

fn vmddnode_count<V, T>(
    dd: &MtMdd2Manager<V>,
    node: NodeId,
    cache: &mut BddHashSet<NodeId>) -> (T, T, T)
where
    T: Add<Output = T> + Clone + From<u32>,
    V: MddValue,
{
    let key = node;
    if cache.contains(&key) {
        return (T::from(0), T::from(0), T::from(1));
    }
    let result = match dd.mtmdd().get_node(&node).unwrap() {
        mtmdd::Node::Terminal(_) | mtmdd::Node::Undet => {
            (T::from(0), T::from(1), T::from(1))
        }
        mtmdd::Node::NonTerminal(fnode) => {
            let mut nn = T::from(1);
            let mut nv = T::from(0);
            let mut ne = T::from(1);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let (ntmp, vtmp, etmp) = vmddnode_count(dd, x, cache);
                nn = nn + ntmp;
                nv = nv + vtmp;
                ne = ne + etmp;
            }
            (nn, nv, ne)
        }
    };
    cache.insert(key);
    result
}

fn bmddnode_count<V, T>(
    dd: &MtMdd2Manager<V>,
    node: NodeId,
    cache: &mut BddHashSet<NodeId>) -> (T, T, T)
where
    T: Add<Output = T> + Clone + From<u32>,
    V: MddValue,
{
    let key = node;
    if cache.contains(&key) {
        return (T::from(0), T::from(0), T::from(1));
    }
    let result = match dd.mdd().get_node(&node).unwrap() {
        mdd::Node::Zero | mdd::Node::One | mdd::Node::Undet => {
            (T::from(0), T::from(1), T::from(1))
        }
        mdd::Node::NonTerminal(fnode) => {
            let mut nn = T::from(1);
            let mut nv = T::from(0);
            let mut ne = T::from(1);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let (ntmp, vtmp, etmp) = bmddnode_count(dd, x, cache);
                nn = nn + ntmp;
                nv = nv + vtmp;
                ne = ne + etmp;
            }
            (nn, nv, ne)
        }
    };
    cache.insert(key);
    result
}

pub fn mdd_count<V, T>(
    mdd: &MtMdd2Manager<V>,
    node: &Node,
    ss: &HashSet<V>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MddValue,
{
    match node {
        Node::Value(fnode) => {
            let mut cache = BddHashMap::default();
            if let Some(level) = mdd.mtmdd().level(fnode) {
                let (nheaders, _, _, _) = mdd.mtmdd().size();
                let mut level2headers = vec![0; nheaders];
                for hid in 0..nheaders {
                    if let Some(h) = mdd.mtmdd().get_header(&hid) {
                        level2headers[h.level()] = hid;
                    }
                }
                vmdd_count(mdd.mtmdd(), *fnode, ss, &mut cache, Some(level), &level2headers)
            } else {
                T::from(1)
            }
        }
        Node::Bool(fnode) => {
            let mut cache = BddHashMap::default();
            if let Some(level) = mdd.mdd().level(fnode) {
                let (nheaders, _, _) = mdd.mdd().size();
                let mut level2headers = vec![0; nheaders];
                for hid in 0..nheaders {
                    if let Some(h) = mdd.mdd().get_header(&hid) {
                        level2headers[h.level()] = hid;
                    }
                }
                bmdd_count(mdd.mdd(), *fnode, ss, &mut cache, Some(level), &level2headers)
            } else {
                T::from(1)
            }
        }
    }
}

fn vmdd_count<V, T>(
    mdd: &mtmdd::MtMddManager<V>,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<(NodeId, Option<usize>), T>,
    level: Option<usize>,
    level2headers: &[usize],
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MddValue,
{
    let key = (node, level);
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(&node).unwrap() {
        mtmdd::Node::NonTerminal(_) | mtmdd::Node::Terminal(_) if level > mdd.level(&node) => {
            let nedges = mdd.get_header(&level2headers[level.unwrap()]).unwrap().edge_num() as u32;
            T::from(nedges) * vmdd_count(mdd, node, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers)
        }
        mtmdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = vmdd_count(mdd, x, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers);
                result = result + tmp;
            }
            result
        }
        mtmdd::Node::Terminal(fnode) => {
            let value = fnode.value();
            if ss.contains(&value) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mtmdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

fn bmdd_count<V, T>(
    mdd: &mdd::MddManager,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
    level: Option<usize>,
    level2headers: &[usize],
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MddValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(&node).unwrap() {
        mdd::Node::NonTerminal(_) | mdd::Node::Zero | mdd::Node::One if level > mdd.level(&node) => {
            let nedges = mdd.get_header(&level2headers[level.unwrap()]).unwrap().edge_num() as u32;
            T::from(nedges) * bmdd_count(mdd, node, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers)
        }
        mdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = bmdd_count(mdd, x, ss, cache, level.and_then(|x| x.checked_sub(1)), level2headers);
                result = result + tmp;
            }
            result
        }
        mdd::Node::Zero => {
            if ss.contains(&V::from(0)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::One => {
            if ss.contains(&V::from(1)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

pub fn zmdd_count<V, T>(
    mdd: &MtMdd2Manager<V>,
    node: &Node,
    ss: &HashSet<V>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MddValue,
{
    match node {
        Node::Value(fnode) => {
            let mut cache = BddHashMap::default();
            vzmdd_count(mdd.mtmdd(), *fnode, ss, &mut cache)
        }
        Node::Bool(fnode) => {
            let mut cache = BddHashMap::default();
            bzmdd_count(mdd.mdd(), *fnode, ss, &mut cache)
        }
    }
}

fn vzmdd_count<V, T>(
    mdd: &mtmdd::MtMddManager<V>,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
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
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mtmdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = vzmdd_count(mdd, x, ss, cache);
                result = result + tmp;
            }
            result
        }
        mtmdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

fn bzmdd_count<V, T>(
    mdd: &mdd::MddManager,
    node: NodeId,
    ss: &HashSet<V>,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32> + Mul<Output = T>,
    V: MddValue,
{
    let key = node;
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let result = match mdd.get_node(&node).unwrap() {
        mdd::Node::Zero => {
            if ss.contains(&V::from(0)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::One => {
            if ss.contains(&V::from(1)) {
                T::from(1)
            } else {
                T::from(0)
            }
        }
        mdd::Node::NonTerminal(fnode) => {
            let mut result = T::from(0);
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            for x in fnodeid.into_iter() {
                let tmp = bzmdd_count(mdd, x, ss, cache);
                result = result + tmp;
            }
            result
        }
        mdd::Node::Undet => T::from(0),
    };
    cache.insert(key, result.clone());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mdd() -> (Node, MtMdd2Manager<i32>) {
        let mut mgr = MtMdd2Manager::<i32>::new(); 
        let h = mgr.create_header(0, "x", 3);
        let zero = mgr.value(0);
        let one = mgr.value(1);
        let two = mgr.value(2);
        let x = mgr.create_node(h, &vec![zero, one, two]);
        let h = mgr.create_header(1, "y", 3);
        let y = mgr.create_node(h, &vec![zero, one, two]);
        let h = mgr.create_header(2, "z", 3);
        let z = mgr.create_node(h, &vec![zero, one, two]);
        let tmp = mgr.add(x, y);
        (mgr.mul(tmp, z), mgr)
    }

    #[test]
    fn test_zmdd_count() {
        let (node, mut mgr) = create_mdd();
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        println!("{}", mgr.dot_string(&node));
        let result: u64 = zmdd_count(&mut mgr, &node, &ss);
        println!("{}", result);
        assert!(result == 3);
    }

    #[test]
    fn test_mdd_count() {
        let (node, mut mgr) = create_mdd();
        let ss = vec![0].into_iter().collect::<HashSet<_>>();
        println!("{}", mgr.dot_string(&node));
        let result: u64 = mdd_count(&mut mgr, &node, &ss);
        println!("{}", result);
        assert!(result == 11);
    }

    #[test]
    fn test_mddnode_count() {
        let (node, mut mgr) = create_mdd();
        println!("{}", mgr.dot_string(&node));
        let result: (u64, u64, u64) = mddnode_count(&mut mgr, &node);
        println!("{:?}", result);
        assert!(result == (9, 7, 27));
    }

    #[test]
    fn test_mddnode_count2() {
        let (node, mut mgr) = create_mdd();
        let zero = mgr.value(0);
        let node = mgr.eq(node, zero);
        println!("{}", mgr.dot_string(&node));
        let result: (u64, u64, u64) = mddnode_count(&mut mgr, &node);
        println!("{:?}", result);
        assert!(result == (3, 2, 9));
    }
}