use bddcore::prelude::*;

/// Minimal solutions (minimal path/cut vectors) of a **monotone** BDD.
///
/// Returns `None` if the function is not monotone (coherent) — the Rauzy minsol
/// decomposition is only valid then. Monotonicity is detected bottom-up inside
/// this recursion via the local invariant "the cofactors ascend": a node on
/// variable `x` with cofactors `(f0, f1)` is monotone in `x` iff `f0 ⇒ f1`,
/// which on the canonical (hash-consed) BDD is the O(1) test **`and(f0, f1) == f0`**
/// (the same "meet equals the lower cofactor" invariant the MDD/MTMDD use with
/// `and`/`min`). The first violation short-circuits to `None`.
pub fn minsol(
    dd: &mut BddManager,
    node: NodeId,
    cache1: &mut BddHashMap<NodeId, Option<NodeId>>,
    cache2: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> Option<NodeId> {
    let key = node;
    if let Some(x) = cache1.get(&key) {
        return *x;
    }
    let result = match dd.get_node(&node).unwrap() {
        Node::Zero => Some(dd.zero()),
        Node::One => Some(dd.one()),
        Node::NonTerminal(fnode) => {
            let headerid = fnode.headerid();
            let f0 = fnode.edge(0);
            let f1 = fnode.edge(1);
            // Bottom-up: recurse the low child first; a non-monotone subfunction
            // short-circuits to None without touching the high child.
            match minsol(dd, f0, cache1, cache2) {
                None => None,
                Some(low) => match minsol(dd, f1, cache1, cache2) {
                    None => None,
                    Some(high_src) => {
                        // Monotone in this variable iff f0 ⇒ f1, i.e.
                        // and(f0, f1) == f0 (canonical -> O(1) id compare).
                        if dd.and(f0, f1) != f0 {
                            None
                        } else {
                            let high = without(dd, high_src, f0, cache2);
                            Some(dd.create_node(headerid, low, high))
                        }
                    }
                },
            }
        }
        Node::Undet => panic!("Undetermined node"),
    };
    cache1.insert(key, result);
    result
}

// enum BddStackValue<'a, 'b> {
//     Bdd2(&'a bdd::BddNode, &'a bdd::BddNode),
//     BddHeader((NodeId, NodeId), &'b dd::nodes::NodeHeader),
// }

// pub fn minsol_stack(
//     dd: &mut bdd::Bdd,
//     node: &bdd::BddNode,
//     cache1: &mut HashMap<NodeId, bdd::BddNode>,
//     cache2: &mut HashMap<(NodeId, NodeId), bdd::BddNode>,
// ) -> bdd::BddNode {
//     let mut next_stack = Vec::with_capacity(2048);
//     let mut result_stack = Vec::with_capacity(2048);
//     next_stack.push(node);
//     while let Some(x) = next_stack.pop() {
//         if let Some(result) = cache1.get(&x.id()) {
//             result_stack.push(result.clone());
//             continue;
//         }
//         match x {
//             bdd::BddNode::Zero => {
//                 let result = dd.zero();
//                 result_stack.push(result.clone());
//             }
//             bdd::BddNode::One => {
//                 let result = dd.one();
//                 result_stack.push(result.clone());
//             }
//             bdd::BddNode::NonTerminal(fnode) => {
//                 next_stack.push(&fnode.edge(0));
//                 next_stack.push(&fnode.edge(1));
//                 next_stack.push(fnode);
//             }
//         }
//     }
//     let key = node.id();
//     match cache1.get(&key) {
//         Some(x) => x.clone(),
//         None => {
//             let result = match node {
//                 bdd::BddNode::Zero => dd.zero(),
//                 bdd::BddNode::One => dd.one(),
//                 bdd::BddNode::NonTerminal(fnode) => {
//                     let tmp = minsol(dd, &fnode.edge(1), cache1, cache2);
//                     let high = without_stack(dd, &tmp, &fnode.edge(0), cache2);
//                     let low = minsol(dd, &fnode.edge(0), cache1, cache2);
//                     dd.create_node(fnode.header(), &low, &high)
//                 }
//             };
//             cache1.insert(key, result.clone());
//             result
//         }
//     }
// }

// pub fn without_stack(
//     dd: &mut bdd::Bdd,
//     f: &bdd::BddNode, // minsol tree
//     g: &bdd::BddNode,
//     cache: &mut HashMap<(NodeId, NodeId), bdd::BddNode>,
// ) -> bdd::BddNode {
//     let mut next_stack = Vec::new(); //with_capacity(2048);
//     let mut result_stack = Vec::new(); //with_capacity(2048);
//     next_stack.push(BddStackValue::Bdd2(f, g));
//     while let Some(stackvalue) = next_stack.pop() {
//         match stackvalue {
//             BddStackValue::BddHeader(key, header) => {
//                 let high = result_stack.pop().unwrap();
//                 let low = result_stack.pop().unwrap();
//                 let result = dd.create_node(&header, &low, &high);
//                 cache.insert(key, result.clone());
//                 result_stack.push(result.clone());
//             }
//             BddStackValue::Bdd2(f, g) => {
//                 let key = (f.id(), g.id());
//                 if let Some(x) = cache.get(&key) {
//                     result_stack.push(x.clone());
//                     continue;
//                 }
//                 match (f, g) {
//                     (bdd::BddNode::Zero, _) => {
//                         let result = dd.zero();
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (_, bdd::BddNode::Zero) => {
//                         let result = f;
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (_, bdd::BddNode::One) => {
//                         let result = dd.zero();
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (bdd::BddNode::One, bdd::BddNode::NonTerminal(gnode)) => {
//                         next_stack.push(BddStackValue::BddHeader(key, gnode.header()));
//                         next_stack.push(BddStackValue::Bdd2(f, &gnode.edge(1)));
//                         next_stack.push(BddStackValue::Bdd2(f, &gnode.edge(0)));
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode))
//                         if fnode.id() == gnode.id() =>
//                     {
//                         let result = dd.zero();
//                         cache.insert(key, result.clone());
//                         result_stack.push(result.clone());
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode))
//                         if fnode.level() > gnode.level() =>
//                     {
//                         next_stack.push(BddStackValue::BddHeader(key, fnode.header()));
//                         next_stack.push(BddStackValue::Bdd2(&fnode.edge(1), g));
//                         next_stack.push(BddStackValue::Bdd2(&fnode.edge(0), g));
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode))
//                         if fnode.level() < gnode.level() =>
//                     {
//                         next_stack.push(BddStackValue::Bdd2(f, &gnode.edge(0)));
//                     }
//                     (bdd::BddNode::NonTerminal(fnode), bdd::BddNode::NonTerminal(gnode)) => {
//                         next_stack.push(BddStackValue::BddHeader(key, fnode.header()));
//                         next_stack.push(BddStackValue::Bdd2(&fnode.edge(1), &gnode.edge(1)));
//                         next_stack.push(BddStackValue::Bdd2(&fnode.edge(0), &gnode.edge(0)));
//                     }
//                 }
//             }
//         }
//     }
//     if let Some(node) = result_stack.pop() {
//         node.clone()
//     } else {
//         panic!("result stack is empty");
//     }
// }

fn without(
    dd: &mut BddManager,
    f: NodeId,
    g: NodeId,
    cache: &mut BddHashMap<(NodeId, NodeId), NodeId>,
) -> NodeId {
    let key = (f, g);
    if let Some(x) = cache.get(&key) {
        return x.clone();
    }
    let node = match (dd.get_node(&f).unwrap(), dd.get_node(&g).unwrap()) {
        (Node::Zero, _) => dd.zero(),
        (_, Node::Zero) => f,
        (_, Node::One) => dd.zero(),
        (Node::One, Node::NonTerminal(gnode)) => {
            let headerid = gnode.headerid();
            let gnodeid: Vec<_> = gnode.iter().collect();
            let low = without(dd, f, gnodeid[0], cache);
            let high = without(dd, f, gnodeid[1], cache);
            dd.create_node(headerid, low, high)
        }
        (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) if fnode.id() == gnode.id() => {
            dd.zero()
        }
        (Node::NonTerminal(fnode), Node::NonTerminal(_gnode)) if dd.level(&f) > dd.level(&g) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().collect();
            let low = without(dd, fnodeid[0], g, cache);
            let high = without(dd, fnodeid[1], g, cache);
            dd.create_node(headerid, low, high)
        }
        (Node::NonTerminal(_fnode), Node::NonTerminal(gnode)) if dd.level(&f) < dd.level(&g) => {
            without(dd, f, gnode.edge(0), cache)
        }
        (Node::NonTerminal(fnode), Node::NonTerminal(gnode)) => {
            let headerid = fnode.headerid();
            let fnodeid: Vec<_> = fnode.iter().collect();
            let gnodeid: Vec<_> = gnode.iter().collect();
            let low = without(dd, fnodeid[0], gnodeid[0], cache);
            let high = without(dd, fnodeid[1], gnodeid[1], cache);
            dd.create_node(headerid, low, high)
        }
        (Node::Undet, _) | (_, Node::Undet) => panic!("Undetermined node"),
    };
    cache.insert(key, node);
    node
}

