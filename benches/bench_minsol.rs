use dd::common::{
    HeaderId,
    NodeId,
    Level,
    HashSet,
    HashMap,
};
// use dd::common::*;
use dd::nodes::*;
use dd::bdd::*;
use dd::dot::*;
use dd::gc::*;

use std::hash::Hash;

use std::io::BufWriter;

use std::fmt::Display;

type Node = BddNode;

fn clock<F>(s: &str, f: F) where F: FnOnce() {
    let start = std::time::Instant::now();
    f();
    let end = start.elapsed();
    println!("{}: time {}", s, end.as_secs_f64());
}

// macro_rules! ftand {
//     ($dd:ident, $($x:expr),+) => {{
//         let mut tmp = $dd.one();
//         $(
//             tmp = $dd.and(&tmp, &$x);
//         )*
//         tmp
//     }};
// }

macro_rules! ftand {
    ($dd:ident, $($x:expr),+) => {{
        let mut tmp = Vec::new();
        $(
            tmp.push($x.clone());
        )*
        ftand($dd, &tmp)
    }};
}

// macro_rules! ftor {
//     ($dd:ident, $($x:expr),+) => {{
//         let mut tmp = $dd.zero();
//         $(
//             tmp = $dd.or(&tmp, &$x);
//         )*
//         tmp
//     }};
// }

macro_rules! ftor {
    ($dd:ident, $($x:expr),+) => {{
        let mut tmp = Vec::new();
        $(
            tmp.push($x.clone());
        )*
        ftor($dd, &tmp)
    }};
}

macro_rules! ftkofn {
    ($dd:ident, $k:expr, $($x:expr),+) => {{
        let mut tmp = Vec::new();
        $(
            tmp.push($x.clone());
        )*
        ftkofn($dd, $k, &tmp)
    }};
}

fn ftand(dd: &mut Bdd, nodes: &[Node]) -> Node {
    let mut tmp = dd.one();
    for x in nodes.iter() {
        tmp = dd.and(&tmp, x);
    }
    tmp
}

fn ftor(dd: &mut Bdd, nodes: &[Node]) -> Node {
    let mut tmp = dd.zero();
    for x in nodes.iter() {
        tmp = dd.or(&tmp, x);
    }
    tmp
}

/*
function _koutofn(b::BDD.Forest, k::Int, args)
    n = length(args)
    (k == 1) && return BDD.or!(b, args...)
    (k == n) && return BDD.and!(b, args...)
    x = args[1]
    xs = args[2:end]
    BDD.ifthenelse!(b, x, _koutofn(b, k-1, xs), _koutofn(b, k, xs))
end
*/

fn ftkofn(dd: &mut Bdd, k: usize, nodes: &[Node]) -> Node {
    let n = nodes.len();
    match k {
        k if k == 1 => ftor(dd, nodes),
        k if k == n => ftand(dd, nodes),
        _ => {
            match nodes {
                [v, rest @ ..] => {
                    let x = ftkofn(dd, k-1, rest);
                    let y = ftkofn(dd, k, rest);
                    dd.ite(v, &x, &y)
                },
                [] => panic!("error"),
            }
        },
    }
}

// fn dot_print(f: &Node) {
//     let mut buf = vec![];
//     {
//         let mut io = BufWriter::new(&mut buf);
//         f.dot(&mut io);
//     }
//     let s = std::str::from_utf8(&buf).unwrap();
//     println!("{}", s);
// }

fn mcsbdd(dd: &mut Bdd, f: &Node) -> Node {
    let mut cache: HashMap<NodeId,Node>  = HashMap::default();
    let top = minsol(dd, f, &mut cache);
    top
}

fn mcsvec(dd: &Bdd, f: &Node) -> Vec<Box<[usize]>> {
    let mut path = Vec::new();
    let mut pset = Vec::new();
    extract(dd, f, &mut path, &mut pset);
    pset
}

fn minsol(dd: &mut Bdd, f: &Node, cache: &mut HashMap<NodeId,Node>) -> Node {
    let key = f.id();
    match cache.get(&key) {
        Some(x) => x.clone(),
        None => {
            let node = match f {
                Node::Zero | Node::One => f.clone(),
                Node::NonTerminal(fnode) => {
                    let tmp = minsol(dd, &fnode[1], cache);
                    let high = dd.diff(&tmp, &fnode[0]);
                    let low = minsol(dd, &fnode[0], cache);
                    dd.create_node(fnode.header(), &low, &high)
                },
            };
            cache.insert(key, node.clone());
            node
        }
    }
}

fn extract(dd: &Bdd, f: &Node, path: &mut Vec<usize>, pset: &mut Vec<Box<[usize]>>) {
    match f {
        Node::One => {
            pset.push(path.to_vec().into_boxed_slice());
        },
        Node::NonTerminal(fnode) => {
            extract(dd, &fnode[0], path, pset);
            path.push(fnode.level());
            extract(dd, &fnode[1], path, pset);
            path.pop();
        },
        _ => (),
    }
}

fn generate_vars<T>(dd: &mut Bdd, labels: &[T]) -> HashMap<T,Node> where T: Display + Eq + Hash + Clone {
    let headers: Vec<_> = labels.iter()
        .enumerate()
        .map(|(i,x)| dd.header(i, &format!("{}", x)))
        .collect();
    let result: HashMap<T,_> = labels.iter()
        .enumerate()
        .map(|(i, x)| (x.clone(), dd.create_node(&headers[i], &dd.zero(), &dd.one())))
        .collect();
    result
}

// macro_rules! vars {
//     ($dd:ident, $labels:expr) => {{
//         generate_vars(&mut $dd, &$labels)
//     }};
// }

fn todot(dd: &mut Bdd, f: &Node) -> String {
    let mut buf = vec![];
    {
        let mut io = BufWriter::new(&mut buf);
        f.dot(&mut io);
    }
    std::str::from_utf8(&buf).unwrap().to_string()
}

fn bench_ft3 () {
    let mut dd: Bdd = Bdd::new();
    // let labels = ["A","B","C","D","E","F","G","H"];
    // let vars = generate_vars(&mut dd, &labels);

    // let f = ftkofn(&mut dd, 2, &vec![
    //     vars[labels[1]].clone(),
    //     vars[labels[2]].clone(),
    //     vars[labels[3]].clone(),
    //     vars[labels[4]].clone()
    // ]);
    let start = std::time::Instant::now();
    let f = make_benchft(&mut dd);
    let end = start.elapsed();
    println!("create time {}", end.as_secs_f64());

    println!("size {:?}", dd.size());
    println!("(nodes, edges) {:?}", dd.count(&f));

    let start = std::time::Instant::now();
    let g = mcsbdd(&mut dd, &f);
    let result = mcsvec(&dd, &g);
    let end = start.elapsed();
    println!("MCS time {}", end.as_secs_f64());

    println!("(nodes, edges) {:?}", dd.count(&g));
    println!("mcs {:?}", result.len());
}

fn make_benchft(dd: &mut Bdd) -> Node {
    let n = 61;
    // let labels: Vec<usize> = (1..=n).rev().collect();

    let labels = [1,6,34,8,35,7,36,9,37,38,39,40,41,30,32,46,48,50,31,33,47,49,51,53,2,10,3,11,4,12,5,13,14,15,16,17,18,19,20,21,52,42,44,22,23,24,25,26,27,28,29,54,58,43,45,55,59,56,60,57,61];
    let c = generate_vars(dd, &labels);

    let g62 = ftand!(dd, c[&1], c[&2]);
    let g63 = ftand!(dd, c[&1], c[&3]);
    let g64 = ftand!(dd, c[&1], c[&4]);
    let g65 = ftand!(dd, c[&1], c[&5]);
    let g66 = ftand!(dd, c[&1], c[&6]);
    let g67 = ftand!(dd, c[&1], c[&7]);
    let g68 = ftand!(dd, c[&1], c[&8]);
    let g69 = ftand!(dd, c[&1], c[&9]);
    let g70 = ftor!(dd, g62, c[&10]);
    let g71 = ftor!(dd, g63, c[&11]);
    let g72 = ftor!(dd, g64, c[&12]);
    let g73 = ftor!(dd, g65, c[&13]);
    let g74 = ftor!(dd, g62, c[&14]);
    let g75 = ftor!(dd, g63, c[&15]);
    let g76 = ftor!(dd, g64, c[&16]);
    let g77 = ftor!(dd, g65, c[&17]);
    let g78 = ftor!(dd, g62, c[&18]);
    let g79 = ftor!(dd, g63, c[&19]);
    let g80 = ftor!(dd, g64, c[&20]);
    let g81 = ftor!(dd, g65, c[&21]);
    let g82 = ftor!(dd, g62, c[&22]);
    let g83 = ftor!(dd, g63, c[&23]);
    let g84 = ftor!(dd, g64, c[&24]);
    let g85 = ftor!(dd, g65, c[&25]);
    let g86 = ftor!(dd, g62, c[&26]);
    let g87 = ftor!(dd, g63, c[&27]);
    let g88 = ftor!(dd, g64, c[&28]);
    let g89 = ftor!(dd, g65, c[&29]);
    let g90 = ftor!(dd, g66, c[&30]);
    let g91 = ftor!(dd, g68, c[&31]);
    let g92 = ftor!(dd, g67, c[&32]);
    let g93 = ftor!(dd, g69, c[&33]);
    let g94 = ftor!(dd, g66, c[&34]);
    let g95 = ftor!(dd, g68, c[&35]);
    let g96 = ftor!(dd, g67, c[&36]);
    let g97 = ftor!(dd, g69, c[&37]);
    let g98 = ftor!(dd, g66, c[&38]);
    let g99 = ftor!(dd, g68, c[&39]);
    let g100 = ftor!(dd, g67, c[&40]);
    let g101 = ftor!(dd, g69, c[&41]);
    let g102 = ftor!(dd, g66, c[&42]);
    let g103 = ftor!(dd, g68, c[&43]);
    let g104 = ftor!(dd, g67, c[&44]);
    let g105 = ftor!(dd, g69, c[&45]);
    let g106 = ftkofn!(dd, 3, g70, g71, g72, g73);
    let g107 = ftkofn!(dd, 3, g74, g75, g76, g77);
    let g108 = ftkofn!(dd, 3, g78, g79, g80, g81);
    let g109 = ftkofn!(dd, 3, g82, g83, g84, g85);
    let g110 = ftkofn!(dd, 3, g86, g87, g88, g89);
    let g111 = ftkofn!(dd, 3, g94, g95, g96, g97);
    let g112 = ftkofn!(dd, 3, g98, g99, g100, g101);
    let g113 = ftand!(dd, g90, g92);
    let g114 = ftand!(dd, g91, g93);
    let g115 = ftand!(dd, g102, g104);
    let g116 = ftand!(dd, g103, g105);
    let g117 = ftor!(dd, g113, c[&46]);
    let g118 = ftor!(dd, g114, c[&47]);
    let g119 = ftor!(dd, g107, g108, c[&52]);
    let g120 = ftor!(dd, g109, g110);
    let g121 = ftor!(dd, g66, g117, c[&48]);
    let g122 = ftor!(dd, g68, g118, c[&49]);
    let g123 = ftor!(dd, g67, g117, c[&50]);
    let g124 = ftor!(dd, g69, g118, c[&51]);
    let g125 = ftkofn!(dd, 2, g121, g123, g122, g124);
    let g126 = ftor!(dd, g111, g112, g125, c[&53]);
    let g127 = ftand!(dd, g115, g120);
    let g128 = ftand!(dd, g116, g120);
    let g129 = ftor!(dd, g62, g127, c[&54]);
    let g130 = ftor!(dd, g63, g128, c[&55]);
    let g131 = ftor!(dd, g64, g127, c[&56]);
    let g132 = ftor!(dd, g65, g128, c[&57]);
    let g133 = ftor!(dd, g62, g129, c[&58]);
    let g134 = ftor!(dd, g63, g130, c[&59]);
    let g135 = ftor!(dd, g64, g131, c[&60]);
    let g136 = ftor!(dd, g65, g132, c[&61]);
    let g137 = ftkofn!(dd, 3, g133, g134, g135, g136);
    let g138 = ftor!(dd, g106, g119, g137);
    let g139 = ftor!(dd, g62, g66, g117, g129, c[&48]);
    let g140 = ftor!(dd, g63, g68, g118, g130, c[&49]);
    let g141 = ftor!(dd, g64, g67, g117, g131, c[&50]);
    let g142 = ftor!(dd, g65, g69, g118, g132, c[&51]);
    let g143 = ftand!(dd, g139, g140, g141, g142);
    let g144 = ftor!(dd, g111, g112, g143, c[&53]);
    let top = ftand!(dd, g126, g138, g144);
    top
}

fn main() {
    clock("bench ft100", bench_ft3);
}