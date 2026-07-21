use bddcore::prelude::*;

/// Convert a **minsol/dual result** (a set family stored in a [`BddManager`] and read with
/// ZDD semantics — a "fake ZDD") into a genuine [`ZddManager`] node.
///
/// This is an internal helper: it is only correct for the zero-suppression-shaped families
/// produced by `bdd_minsol::minsol` (via [`BssMgr::minpath`](crate::bss::BssMgr::minpath) /
/// `mincut`), **not** for an arbitrary boolean BDD. It is therefore `pub(crate)` and never
/// exposed as a public "BDD → ZDD" converter.
///
/// The walk is a memoized structural copy: map each source `HeaderId` to a destination
/// header preserving `(level, label)`, then rebuild every node with `ZddManager::create_node`
/// (which applies genuine zero-suppression). Because the source is built under BDD reduction
/// (merge only on `lo == hi`, never zero-suppressed), a source non-terminal always has
/// `e0 != e1`; a node with `e1 == Zero` is kept in the source but its family equals `low`,
/// which is exactly what the destination's `high == zero` suppression yields — so the
/// converted ZDD, read with ZDD semantics, enumerates the same family.
pub(crate) fn to_zdd(
    src: &BddManager,
    root: NodeId,
    dst: &mut ZddManager,
    zh: &mut BddHashMap<HeaderId, HeaderId>,
    memo: &mut BddHashMap<NodeId, NodeId>,
) -> NodeId {
    if let Some(&x) = memo.get(&root) {
        return x;
    }
    let result = match src.get_node(&root).unwrap() {
        Node::Zero => dst.zero(),
        Node::One => dst.one(),
        Node::Undet => dst.undet(),
        Node::NonTerminal(fnode) => {
            let shid = fnode.headerid();
            let e0 = fnode.edge(0);
            let e1 = fnode.edge(1);
            let low = to_zdd(src, e0, dst, zh, memo);
            let high = to_zdd(src, e1, dst, zh, memo);
            let dhid = match zh.get(&shid) {
                Some(&h) => h,
                None => {
                    let header = src.get_header(&shid).unwrap();
                    let h = dst.create_header(header.level(), header.label());
                    zh.insert(shid, h);
                    h
                }
            };
            dst.create_node(dhid, low, high)
        }
    };
    memo.insert(root, result);
    result
}
