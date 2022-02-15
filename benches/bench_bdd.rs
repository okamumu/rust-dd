use dd::common::*;
use dd::nodes::*;
use dd::bdd::*;
// use dd::dot::*;

use dd::bdd_mut::*;

type Node = BddNode;

fn clock<F>(s: &str, f: F) where F: FnOnce() {
    let start = std::time::Instant::now();
    f();
    let end = start.elapsed();
    println!("{}: time {}", s, end.as_secs_f64());
}

#[derive(Debug,Clone,Copy)]
enum Binary {
    Zero,
    One,
    Undet,
}

trait Print {
    fn print(&self);
}

impl Print for Vec<(Vec<Binary>,Binary)> {
    fn print(&self) {
        for (x,y) in self {
            for v in x.iter().rev() {
                match v {
                    Binary::Zero => print!("0 "),
                    Binary::One => print!("1 "),
                    Binary::Undet => print!("U "),
                }
            }
            match y {
                Binary::Zero => println!("| 0"),
                Binary::One => println!("| 1"),
                _ => (),
            }
        }
    }
}

fn table(dd: &Bdd, f: &Node) -> Vec<(Vec<Binary>,Binary)> {
    let mut tab = Vec::new();
    let p = Vec::new();
    table_impl(dd, f.level().unwrap()+1, f, &p, &mut tab);
    tab
}

fn table_impl(dd: &Bdd, level: Level, f: &Node,
        path: &[Binary], tab: &mut Vec<(Vec<Binary>,Binary)>) {
    match f {
        Node::Zero => {
            let mut p = path.to_vec();
            for _ in 0..level {
                p.push(Binary::Undet);
            }
            tab.push((p,Binary::Zero));
        },
        Node::One => {
            let mut p = path.to_vec();
            for _ in 0..level {
                p.push(Binary::Undet);
            }
            tab.push((p,Binary::One));
        },
        Node::NonTerminal(fnode) => {
            let current_level = fnode.level();
            for (i,e) in fnode.iter().enumerate() {
                let mut p = path.to_vec();
                for _ in current_level..level-1 {
                    p.push(Binary::Undet);
                }
                match i {
                    0 => p.push(Binary::Zero),
                    1 => p.push(Binary::One),
                    _ => (),
                }
                table_impl(dd, current_level, e, &p, tab);
            }
        },
        _ => (),
    };
}

fn bench_bdd1 () {
    let n = 1000;
    let mut f: Bdd = Bdd::new();
    let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
    let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();

    let mut b = f.one();
    clock("-bench bdd1-1", ||{
        for i in 0..n {
            b = f.and(&b, &x[i]);
        }
    });
    println!("-bdd2 node {:?}", f.size());
}

fn bench_bdd2 () {
    let n = 1000;
    let mut f: Bdd = Bdd::new();
    let mut b = f.one();
    clock("-bench bdd2-1", ||{
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();
    
        for i in (0..n).rev() {
            b = f.and(&b, &x[i]);
        }
    });
    println!("-bdd2 node {:?}", f.size());
    clock("-bench bdd2-2", ||{
        f.clear();
        f.rebuild(&vec![b]);
    });
    println!("-bdd2 node {:?}", f.size());
}

fn bench_bdd3 () {
    let n = 3;
    let mut f: Bdd = Bdd::new();
    let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
    let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();

    let b = f.and(&x[0], &x[1]);
    let b = f.or(&b, &x[2]);
    println!("   bdd2 node {:?}", f.size());
    let result = table(&f, &b);
    result.print();
}

fn main() {
    clock("bench bdd1", bench_bdd1);
    clock("bench bdd2", bench_bdd2);
    clock("bench bdd3", bench_bdd3);
}