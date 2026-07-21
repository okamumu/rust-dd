use mddcore::prelude::*;

/// Minimal solutions of a **coherent (monotone)** MTMDD2 structure function, or
/// `None` if the function is not coherent.
///
/// Coherence is checked bottom-up inside the recursion via the local invariant
/// **"the cofactors of every node form a pointwise ascending chain"**, verified
/// on the canonical (hash-consed) diagram so each adjacent test is O(1):
/// - value forest (`vminsol`): `min(c_{i-1}, c_i) == c_{i-1}`  (i.e. c_{i-1} ≤ c_i),
/// - bool forest (`bminsol`):  `and(c_{i-1}, c_i) == c_{i-1}`  (i.e. c_{i-1} ⇒ c_i).
///
/// The first violation short-circuits to `None`.
pub fn minsol<V>(mdd: &mut MtMdd2Manager<V>, node: &Node) -> Option<Node>
where
    V: MddValue,
{
    match node {
        Node::Value(fnode) => {
            let mut cache1 = BddHashMap::default();
            let mut cache2 = BddHashMap::default();
            vminsol(&mut mdd.mtmdd_mut(), *fnode, &mut cache1, &mut cache2).map(Node::Value)
        }
        Node::Bool(fnode) => {
            let mut cache1 = BddHashMap::default();
            let mut cache2 = BddHashMap::default();
            bminsol(&mut mdd.mdd_mut(), *fnode, &mut cache1, &mut cache2).map(Node::Bool)
        }
    }
}

fn vminsol<V>(
    dd: &mut mtmdd::MtMddManager<V>,
    node: NodeId,
    cache1: &mut BddHashMap<NodeId, Option<NodeId>>,
    cache2: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> Option<NodeId>
where
    V: MddValue,
{
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return *x;
    }
    let result = match dd.get_node(&node).unwrap() {
        mtmdd::Node::Terminal(_fnode) => Some(node),
        mtmdd::Node::Undet => Some(dd.undet()),
        mtmdd::Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let children: Vec<NodeId> = fnode.iter().collect();
            // Coherence in this variable: cofactors ascend pointwise, i.e.
            // min(c_{i-1}, c_i) == c_{i-1} (canonical -> O(1) id compare).
            let mono = (1..children.len())
                .all(|i| dd.min(children[i - 1], children[i]) == children[i - 1]);
            if !mono {
                None
            } else {
                // Coherence in the other variables: recurse, aborting on the
                // first non-coherent child.
                let mut result = Vec::with_capacity(children.len());
                let mut ok = true;
                for (i, &c) in children.iter().enumerate() {
                    match vminsol(dd, c, cache1, cache2) {
                        None => {
                            ok = false;
                            break;
                        }
                        Some(m) => {
                            let v = if i == 0 {
                                m
                            } else {
                                vwithout(dd, children[i - 1], m, cache2)
                            };
                            result.push(v);
                        }
                    }
                }
                if ok {
                    Some(dd.create_node(headerid, &result))
                } else {
                    None
                }
            }
        }
    };
    cache1.insert(key, result);
    result
}

fn vwithout<V>(
    mdd: &mut mtmdd::MtMddManager<V>,
    f: NodeId,
    g: NodeId, // minsol tree
    cache: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> NodeId
where
    V: MddValue,
{
    let key = (f, g);
    if let Some(x) = cache.get(&key) {
        return *x;
    }
    let result = match (mdd.get_node(&f).unwrap(), mdd.get_node(&g).unwrap()) {
        (mtmdd::Node::Undet, _) => g,
        (_, mtmdd::Node::Undet) => mdd.undet(),
        (mtmdd::Node::Terminal(fnode), mtmdd::Node::Terminal(gnode)) => {
            if fnode.value() == gnode.value() {
                mdd.undet()
            } else {
                g
            }
        }
        // g (the minsol family) is a terminal, so the candidate vector has all
        // remaining variables at 0 — only f's zero branch is relevant. Expanding
        // every branch of f (the previous behavior) fabricated non-minimal vectors
        // with positive components (e.g. minpath(Max(X,Y)) gained (X=1,Y=2)). Same
        // principle as the `level(f) > level(g)` arm below.
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::Terminal(_)) => {
            vwithout(mdd, fnode.edge(0), g, cache)
        }
        (mtmdd::Node::Terminal(_), mtmdd::Node::NonTerminal(gnode)) => {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().collect();
            let tmp: Vec<_> = gnodeid
                .into_iter()
                .map(|x| vwithout(mdd, f, x, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::NonTerminal(_gnode))
            if mdd.level(&f) > mdd.level(&g) =>
        {
            vwithout(mdd, fnode.edge(0), g, cache)
        }
        (mtmdd::Node::NonTerminal(_fnode), mtmdd::Node::NonTerminal(gnode))
            if mdd.level(&f) < mdd.level(&g) =>
        {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().collect();
            let tmp: Vec<_> = gnodeid
                .into_iter()
                .map(|x| vwithout(mdd, f, x, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().collect();
            let gnodeid: Vec<_> = gnode.iter().collect();
            let tmp: Vec<_> = fnodeid
                .into_iter()
                .zip(gnodeid.into_iter())
                .map(|(f, g)| vwithout(mdd, f, g, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
    };
    cache.insert(key, result);
    result
}

fn bminsol(
    dd: &mut mdd::MddManager,
    node: NodeId,
    cache1: &mut BddHashMap<NodeId, Option<NodeId>>,
    cache2: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> Option<NodeId> {
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return *x;
    }
    let result = match dd.get_node(&node).unwrap() {
        mdd::Node::Zero => Some(dd.undet()),
        mdd::Node::One => Some(node),
        mdd::Node::Undet => Some(dd.undet()),
        mdd::Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let children: Vec<NodeId> = fnode.iter().collect();
            // Coherence in this variable: cofactors ascend, i.e. c_{i-1} => c_i,
            // and(c_{i-1}, c_i) == c_{i-1} (canonical -> O(1) id compare).
            let mono = (1..children.len())
                .all(|i| dd.and(children[i - 1], children[i]) == children[i - 1]);
            if !mono {
                None
            } else {
                let mut result = Vec::with_capacity(children.len());
                let mut ok = true;
                for (i, &c) in children.iter().enumerate() {
                    match bminsol(dd, c, cache1, cache2) {
                        None => {
                            ok = false;
                            break;
                        }
                        Some(m) => {
                            let v = if i == 0 {
                                m
                            } else {
                                bwithout(dd, children[i - 1], m, cache2)
                            };
                            result.push(v);
                        }
                    }
                }
                if ok {
                    Some(dd.create_node(headerid, &result))
                } else {
                    None
                }
            }
        }
    };
    cache1.insert(key, result);
    result
}

fn bwithout(
    mdd: &mut mdd::MddManager,
    f: NodeId,
    g: NodeId, // minsol tree
    cache: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> NodeId {
    let key = (f, g);
    if let Some(x) = cache.get(&key) {
        return *x;
    }
    let result = match (mdd.get_node(&f).unwrap(), mdd.get_node(&g).unwrap()) {
        (mdd::Node::Undet, _) => g,
        (_, mdd::Node::Undet) => mdd.undet(),
        (mdd::Node::Zero, mdd::Node::One) => mdd.one(),
        (mdd::Node::Zero, _) => g,
        (_, mdd::Node::Zero) => mdd.undet(), // probably this case is inpossible
        (mdd::Node::One, _) => mdd.undet(),
        // g (the minsol family) is the {∅} terminal, so the candidate vector has
        // all remaining variables at 0 — only f's zero branch matters. Expanding
        // every branch of f fabricated non-minimal vectors (e.g. minpath(x&y|z)
        // gained {y,z}). Same principle as the `level(f) > level(g)` arm below.
        (mdd::Node::NonTerminal(fnode), mdd::Node::One) => {
            bwithout(mdd, fnode.edge(0), g, cache)
        }
        (mdd::Node::NonTerminal(fnode), mdd::Node::NonTerminal(_gnode))
            if mdd.level(&f) > mdd.level(&g) =>
        {
            bwithout(mdd, fnode.edge(0), g, cache)
        }
        (mdd::Node::NonTerminal(_fnode), mdd::Node::NonTerminal(gnode))
            if mdd.level(&f) < mdd.level(&g) =>
        {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().collect();
            let tmp: Vec<_> = gnodeid
                .into_iter()
                .map(|x| bwithout(mdd, f, x, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mdd::Node::NonTerminal(fnode), mdd::Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().collect();
            let gnodeid: Vec<_> = gnode.iter().collect();
            let tmp: Vec<_> = fnodeid
                .into_iter()
                .zip(gnodeid.into_iter())
                .map(|(f, g)| bwithout(mdd, f, g, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
    };
    cache.insert(key, result);
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
    fn test_minsol() {
        let (node, mut mgr) = create_mdd();
        println!("{}", mgr.dot_string(&node));
        // phi = (x+y)*z is coherent (non-decreasing in each component) -> Some.
        let result = minsol(&mut mgr, &node).expect("(x+y)*z is coherent");
        println!("{}", mgr.dot_string(&result));
    }
}