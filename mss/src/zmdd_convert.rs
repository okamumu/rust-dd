//! Convert a minsol result (the fake-ZMDD living in the `MtMdd2Manager`, built under full
//! reduction) into a genuine [`ZmddManager`] (zero-suppression). Internal helper for
//! [`MssMgr::minpath`](crate::mss::MssMgr::minpath) (via `ZmddMgr::convert`).
//!
//! Correctness: rebuilding each source node with the ZMDD `create_node` applies
//! zero-suppression (`non-0 edges all Undet → 0-edge`), which is family-preserving; and the
//! source's full-reduction only fired where every edge was equal (= the variable is
//! "don't-care" = `X=0` for a minsol result), which the genuine ZMDD represents by the
//! absence of that node too. So the two agree on the enumerated sparse vectors. The boolean
//! sub-forest maps `One → value(1)` (label 1 = "present"), `Zero`/`Undet → Undet` (empty).

use mddcore::prelude::*;
use mddcore::{mdd, mtmdd};

pub(crate) fn to_zmdd<V>(src: &MtMdd2Manager<V>, node: &Node, dst: &mut ZmddManager<V>) -> NodeId
where
    V: MddValue,
{
    let mut hmap: BddHashMap<HeaderId, HeaderId> = BddHashMap::default();
    let mut memo: BddHashMap<NodeId, NodeId> = BddHashMap::default();
    match node {
        Node::Value(id) => to_zmdd_value(src.mtmdd(), *id, dst, &mut hmap, &mut memo),
        Node::Bool(id) => to_zmdd_bool(src.mdd(), *id, dst, &mut hmap, &mut memo),
    }
}

fn dst_header<V>(
    dst: &mut ZmddManager<V>,
    hmap: &mut BddHashMap<HeaderId, HeaderId>,
    shid: HeaderId,
    level: Level,
    label: &str,
    edge_num: usize,
) -> HeaderId
where
    V: MddValue,
{
    match hmap.get(&shid) {
        Some(&h) => h,
        None => {
            let h = dst.create_header(level, label, edge_num);
            hmap.insert(shid, h);
            h
        }
    }
}

fn to_zmdd_value<V>(
    src: &mtmdd::MtMddManager<V>,
    root: NodeId,
    dst: &mut ZmddManager<V>,
    hmap: &mut BddHashMap<HeaderId, HeaderId>,
    memo: &mut BddHashMap<NodeId, NodeId>,
) -> NodeId
where
    V: MddValue,
{
    if let Some(&x) = memo.get(&root) {
        return x;
    }
    let result = match src.get_node(&root).unwrap() {
        mtmdd::Node::Undet => dst.undet(),
        mtmdd::Node::Terminal(t) => {
            let v = t.value();
            dst.value(v)
        }
        mtmdd::Node::NonTerminal(f) => {
            let shid = f.headerid();
            let edges: Vec<NodeId> = f.iter().collect();
            let (level, label) = {
                let hdr = src.get_header(&shid).unwrap();
                (hdr.level(), hdr.label().to_string())
            };
            let ch: Vec<NodeId> = edges
                .iter()
                .map(|&e| to_zmdd_value(src, e, dst, hmap, memo))
                .collect();
            let dhid = dst_header(dst, hmap, shid, level, &label, edges.len());
            dst.create_node(dhid, &ch)
        }
    };
    memo.insert(root, result);
    result
}

fn to_zmdd_bool<V>(
    src: &mdd::MddManager,
    root: NodeId,
    dst: &mut ZmddManager<V>,
    hmap: &mut BddHashMap<HeaderId, HeaderId>,
    memo: &mut BddHashMap<NodeId, NodeId>,
) -> NodeId
where
    V: MddValue,
{
    if let Some(&x) = memo.get(&root) {
        return x;
    }
    let result = match src.get_node(&root).unwrap() {
        mdd::Node::Undet | mdd::Node::Zero => dst.undet(),
        mdd::Node::One => dst.value(V::from(1)),
        mdd::Node::NonTerminal(f) => {
            let shid = f.headerid();
            let edges: Vec<NodeId> = f.iter().collect();
            let (level, label) = {
                let hdr = src.get_header(&shid).unwrap();
                (hdr.level(), hdr.label().to_string())
            };
            let ch: Vec<NodeId> = edges
                .iter()
                .map(|&e| to_zmdd_bool(src, e, dst, hmap, memo))
                .collect();
            let dhid = dst_header(dst, hmap, shid, level, &label, edges.len());
            dst.create_node(dhid, &ch)
        }
    };
    memo.insert(root, result);
    result
}
