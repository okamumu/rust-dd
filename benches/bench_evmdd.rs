use dd::common::*;
use dd::nodes::*;
use dd::evplus_mdd::{
    EvMdd,
    EvMddNode,
};
use dd::dot::Dot;

type Node<E,V> = EvMddNode<E,V>;

pub fn table<T,U>(dd: &EvMdd<T,U>, fv: T, f: &Node<T,U>) -> Vec<(Vec<usize>,Option<T>)> where T: EdgeValue, U: TerminalBinaryValue {
    let mut tab = Vec::new();
    let p = Vec::new();
    table_(dd, f, &p, &mut tab, fv);
    tab
}

pub fn table_<T,U>(dd: &EvMdd<T,U>, f: &Node<T,U>, path: &[usize], tab: &mut Vec<(Vec<usize>,Option<T>)>, s: T) where T: EdgeValue, U: TerminalBinaryValue {
    match f {
        Node::Terminal(_) if f == &dd.infinity() => {
            tab.push((path.to_vec(), None));
        },
        Node::Terminal(_) if f == &dd.omega() => {
            tab.push((path.to_vec(), Some(s)));
        },
        Node::NonTerminal(fnode) => {
            for (i,e) in fnode.iter().enumerate() {
                let mut p = path.to_vec();
                p.push(i);
                table_(dd, &e.node(), &p, tab, s + e.value());
            }
        },
        _ => (),
    };
}

// fn bench_evmdd1() {
//     println!("hello");
// }

fn main() {
    println!("hello");
}