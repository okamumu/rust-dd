use mddcore::prelude::*;

pub fn minsol<V>(mdd: &mut MtMdd2Manager<V>, node: &Node) -> Node
where
    V: MddValue,
{
    match node {
        Node::Value(fnode) => {
            let mut cache1 = BddHashMap::default();
            let mut cache2 = BddHashMap::default();
            let result = vminsol(&mut mdd.mtmdd_mut(), *fnode, &mut cache1, &mut cache2);
            Node::Value(result)
        }
        Node::Bool(fnode) => {
            let mut cache1 = BddHashMap::default();
            let mut cache2 = BddHashMap::default();
            let result = bminsol(&mut mdd.mdd_mut(), *fnode, &mut cache1, &mut cache2);
            Node::Bool(result)
        }
    }
}

fn vminsol<V>(
    dd: &mut mtmdd::MtMddManager<V>,
    node: NodeId,
    cache1: &mut BddHashMap<NodeId, NodeId>,
    cache2: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> NodeId
where
    V: MddValue,
{
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return *x;
    }
    let result = match dd.get_node(&node).unwrap() {
        mtmdd::Node::Terminal(_fnode) => node,
        mtmdd::Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let mut result = Vec::new();
            for (i, x) in fnodeid.iter().enumerate() {
                if i == 0 {
                    let tmp = vminsol(dd, *x, cache1, cache2);
                    result.push(tmp);
                } else {
                    let tmp = vminsol(dd, *x, cache1, cache2);
                    let tmp2 = vwithout(dd, fnodeid[i - 1], tmp, cache2);
                    result.push(tmp2);
                }
            }
            dd.create_node(headerid, &result)
        }
        mtmdd::Node::Undet => dd.undet(),
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
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::Terminal(_)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let tmp: Vec<_> = fnodeid
                .into_iter()
                .map(|x| vwithout(mdd, x, g, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::Terminal(_), mtmdd::Node::NonTerminal(gnode)) => {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = gnodeid
                .into_iter()
                .map(|x| vwithout(mdd, f, x, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::NonTerminal(_gnode))
            if mdd.level(&f) > mdd.level(&g) =>
        {
            vwithout(mdd, fnode[0], g, cache)
        }
        (mtmdd::Node::NonTerminal(_fnode), mtmdd::Node::NonTerminal(gnode))
            if mdd.level(&f) < mdd.level(&g) =>
        {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = gnodeid
                .into_iter()
                .map(|x| vwithout(mdd, f, x, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mtmdd::Node::NonTerminal(fnode), mtmdd::Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
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
    cache1: &mut BddHashMap<NodeId, NodeId>,
    cache2: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> NodeId {
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return *x;
    }
    let result = match dd.get_node(&node).unwrap() {
        mdd::Node::Zero => dd.undet(),
        mdd::Node::One => node,
        mdd::Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let mut result = Vec::new();
            for (i, x) in fnodeid.iter().enumerate() {
                if i == 0 {
                    let tmp = bminsol(dd, *x, cache1, cache2);
                    result.push(tmp);
                } else {
                    let tmp = bminsol(dd, *x, cache1, cache2);
                    let tmp2 = bwithout(dd, fnodeid[i - 1], tmp, cache2);
                    result.push(tmp2);
                }
            }
            dd.create_node(headerid, &result)
        }
        mdd::Node::Undet => dd.undet(),
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
        (mdd::Node::NonTerminal(fnode), mdd::Node::One) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let tmp: Vec<_> = fnodeid
                .into_iter()
                .map(|x| bwithout(mdd, x, g, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mdd::Node::NonTerminal(fnode), mdd::Node::NonTerminal(_gnode))
            if mdd.level(&f) > mdd.level(&g) =>
        {
            bwithout(mdd, fnode[0], g, cache)
        }
        (mdd::Node::NonTerminal(_fnode), mdd::Node::NonTerminal(gnode))
            if mdd.level(&f) < mdd.level(&g) =>
        {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
            let tmp: Vec<_> = gnodeid
                .into_iter()
                .map(|x| bwithout(mdd, f, x, cache))
                .collect();
            mdd.create_node(headerid, &tmp)
        }
        (mdd::Node::NonTerminal(fnode), mdd::Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().cloned().collect();
            let gnodeid: Vec<_> = gnode.iter().cloned().collect();
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
        let result = minsol(&mut mgr, &node);
        println!("{}", mgr.dot_string(&result));
    }
}