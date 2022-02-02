use dd::common::*;
use dd::ev_plus_mdd::*;

pub fn table<T>(dd: &EVMDD<T>, fv: T, f: &Node<T>) -> Vec<(Vec<usize>,Option<T>)> where T: EdgeValue {
    let mut tab = Vec::new();
    let p = Vec::new();
    table_(dd, f, &p, &mut tab, fv);
    tab
}

pub fn table_<T>(dd: &EVMDD<T>, f: &Node<T>, path: &[usize], tab: &mut Vec<(Vec<usize>,Option<T>)>, s: T) where T: EdgeValue {
    match f {
        Node::Terminal(_) if f == &dd.infinity() => {
            tab.push((path.to_vec(), None));
        },
        Node::Terminal(_) if f == &dd.omega() => {
            tab.push((path.to_vec(), Some(s)));
        },
        Node::NonTerminal(fnode) => {
            for (i,e) in fnode.edge_iter().enumerate() {
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