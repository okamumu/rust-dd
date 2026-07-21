use bddcore::prelude::*;

/// Dual of a BDD: `φ^D(x) = ¬φ(¬x)`.
///
/// By Shannon expansion, `dual(φ)` is "swap the two children and complement the
/// terminals", applied recursively — an O(size) operation (memoized). It
/// preserves monotonicity (`φ` increasing ⇒ `φ^D` increasing).
///
/// The minimal solutions (prime implicants) of the dual are the **minimal cut
/// vectors** of `φ`, whereas those of `φ` itself are the minimal path vectors —
/// so `mincut(φ) = minsol(dual(φ))`.
pub fn dual(
    dd: &mut BddManager,
    node: NodeId,
    cache: &mut BddHashMap<NodeId, NodeId>,
) -> NodeId {
    if let Some(&x) = cache.get(&node) {
        return x;
    }
    let result = match dd.get_node(&node).unwrap() {
        Node::Zero => dd.one(),
        Node::One => dd.zero(),
        Node::Undet => dd.undet(),
        Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let lo = fnode.edge(0);
            let hi = fnode.edge(1);
            // D(φ) = x·D(low) + ¬x·D(high): new low = dual(old high),
            // new high = dual(old low) — i.e. swap children and recurse.
            let new_low = dual(dd, hi, cache);
            let new_high = dual(dd, lo, cache);
            dd.create_node(headerid, new_low, new_high)
        }
    };
    cache.insert(node, result);
    result
}
