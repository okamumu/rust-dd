use std::ops::Add;
use bddcore::prelude::*;

/// Count the sets in a genuine [`ZddManager`] family.
///
/// Mirrors `bdd_count::zdd_count` (which reads the fake-ZDD in a `BddManager`) but over a
/// real `ZddManager`: `One` contributes 1 when `ss` selects `true`, `Zero` contributes 1
/// when `ss` selects `false`, and a non-terminal sums both edges. With the default
/// `ss = [true]` this is the number of sets in the family.
pub fn zdd_count<T>(
    dd: &ZddManager,
    ss: &[bool],
    node: NodeId,
    cache: &mut BddHashMap<NodeId, T>,
) -> T
where
    T: Add<Output = T> + Clone + From<u32>,
{
    if let Some(x) = cache.get(&node) {
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
            let e0 = zdd_count(dd, ss, fnode.edge(0), cache);
            let e1 = zdd_count(dd, ss, fnode.edge(1), cache);
            e0 + e1
        }
        Node::Undet => T::from(0),
    };
    cache.insert(node, result.clone());
    result
}

/// Count nodes / non-terminal nodes / edges of a `ZddManager` subgraph (for `size()`).
pub fn node_count(
    dd: &ZddManager,
    node: NodeId,
    cache: &mut BddHashSet<NodeId>,
) -> (u64, u64, u64) {
    if cache.contains(&node) {
        return (0, 0, 1);
    }
    let result = match dd.get_node(&node).unwrap() {
        Node::One | Node::Zero | Node::Undet => (0, 1, 1),
        Node::NonTerminal(fnode) => {
            let (n0, v0, e0) = node_count(dd, fnode.edge(0), cache);
            let (n1, v1, e1) = node_count(dd, fnode.edge(1), cache);
            (n0 + n1 + 1, v0 + v1, e0 + e1 + 1)
        }
    };
    cache.insert(node);
    result
}
