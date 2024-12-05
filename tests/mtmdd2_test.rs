#[macro_use]
extern crate dd;

use dd::common::{
    HashMap, Level, NodeId, TerminalNumberValue
};

use dd::mdd::MddNode;
use dd::nodes::{
    NodeHeader,
    Terminal,
    NonTerminal,
    TerminalNumber,
    NonTerminalMDD,
};

use dd::dot::*;

use dd::mtmdd:: {
    MtMdd, MtMddNode,
};

use dd::mtmdd2:: {
    self, build_from_rpn, gen_var, MtMdd2, MtMdd2Node, Token
};

fn minsol<V>(dd: &mut MtMdd2<V>, f: &MtMddNode<V>, minv: V, cache: &mut HashMap<NodeId,MtMddNode<V>>) -> MtMddNode<V>
    where V: TerminalNumberValue
{
    let key = f.id();
    match cache.get(&key) {
        Some(x) => x.clone(),
        None => {
            let node = match f {
                MtMddNode::Terminal(_) => f.clone(),
                MtMddNode::NonTerminal(fnode) => {
                    let mut nodes = Vec::new();
                    let n = fnode.header().edge_num();
                    for i in (0..n).rev() {
                        let mut tmp = minsol(dd, &fnode[i], minv, cache);
                        if i != 0 {
                            let x1 = dd.veq(&tmp, &fnode[i-1]);
                            tmp = dd.velse(&x1, &tmp);
                        } else {
                            let zero = dd.mtmdd_mut().value(minv);
                            let x1 = dd.veq(&tmp, &zero);
                            tmp = dd.velse(&x1, &tmp);
                        }
                        nodes.push(tmp);
                    }
                    nodes.reverse();
                    dd.mtmdd_mut().create_node(fnode.header(), &nodes)
                },
                MtMddNode::Undet => dd.mtmdd().undet()
            };
            cache.insert(key, node.clone());
            node
        }
    }
}

fn is_monotone<V>(dd: &mut MtMdd2<V>, f: &MtMddNode<V>, level: Level, cache: &mut HashMap<NodeId,MddNode>) -> MddNode
    where V: TerminalNumberValue
{
    let key = f.id();
    match cache.get(&key) {
        Some(x) => x.clone(),
        None => {
            let node: MddNode = match f {
                MtMddNode::Terminal(_) => dd.mdd().one(),
                MtMddNode::NonTerminal(fnode) => {
                    if fnode.level() <= level {
                        let n = fnode.header().edge_num();
                        let mut tmp = dd.mdd().one();
                        for i in 0..n-1 {
                            let x1 = dd.vlte(&fnode[i], &fnode[i+1]);
                            tmp = dd.mdd_mut().and(&tmp, &x1);
                        }
                        tmp
                    } else {
                        let nodes = fnode.iter().map(|x| is_monotone(dd, x, level, cache)).collect::<Vec<_>>();
                        dd.mdd_mut().create_node(fnode.header(), &nodes)
                    }
                },
                MtMddNode::Undet => dd.mdd().undet()
            };
            cache.insert(key, node.clone());
            node
        }
    }
}

#[test]
fn integration_test_mtmdd2 () {
    let mut dd = MtMdd2::<i32>::new();
    let c = gen_var(&mut dd, "C", 0, &[0,1,2]);
    let b = gen_var(&mut dd, "B", 1, &[0,1,2]);
    let a = gen_var(&mut dd, "A", 2, &[0,1]);

    let sx = build_from_rpn!{dd, b 0 == c 0 == && 0 b 0 == c 0 == || 1 b 2 == c 2 == || 3 2 ? ? ?}.unwrap();
    println!("{}", sx.dot_string());

    let ss = build_from_rpn!{dd, a 0 == 0 sx ?}.unwrap();
    println!("{}", ss.dot_string());
}

#[test]
fn integration_test_mtmdd2_minsol () {
    let mut dd = MtMdd2::<i32>::new();
    let c = gen_var(&mut dd, "C", 1, &[0,1,2]);
    let b = gen_var(&mut dd, "B", 2, &[0,1,2]);
    let a = gen_var(&mut dd, "A",3, &[0,1]);

    let sx = build_from_rpn!{dd, b 0 == c 0 == && 0 b 0 == c 0 == || 1 b 2 == c 2 == || 3 2 ? ? ?}.unwrap();
    println!("{}", sx.dot_string());

    let ss = build_from_rpn!{dd, a 0 == 0 sx ?}.unwrap();
    println!("{}", ss.dot_string());

    print!("Hello");
    let mut cache: HashMap<NodeId,MtMddNode<i32>>  = HashMap::default();
    if let MtMdd2Node::Value(f) = ss {
        let result = minsol(&mut dd, &f, 0, &mut cache);
        println!("{}", result.dot_string());
    }
}

#[test]
fn integration_test_mtmdd2_monotone1() {
    let mut dd = MtMdd2::<i32>::new();
    let c = gen_var(&mut dd, "C", 1, &[0,1,2]);
    let b = gen_var(&mut dd, "B", 2, &[0,1,2]);
    let a = gen_var(&mut dd, "A",3, &[0,1]);

    let sx = build_from_rpn!{dd, b 0 == c 0 == && 0 b 0 == c 0 == || 1 b 2 == c 2 == || 3 2 ? ? ?}.unwrap();
    println!("{}", sx.dot_string());

    let ss = build_from_rpn!{dd, a 0 == 0 sx ?}.unwrap();
    println!("{}", ss.dot_string());

    println!("Monotone");
    let mut cache: HashMap<NodeId,MddNode>  = HashMap::default();
    if let MtMdd2Node::Value(f) = ss {
        let result = is_monotone(&mut dd, &f, 2, &mut cache);
        println!("{}", result.dot_string());
    }
}
