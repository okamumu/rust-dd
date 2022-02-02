use dd::common::*;
use dd::bdd::*;

fn clock<F>(s: &str, f: F) where F: FnOnce() {
    let start = std::time::Instant::now();
    f();
    let end = start.elapsed();
    println!("{}: time {}", s, end.as_secs_f64());
}

pub fn table<T>(dd: &BDD<T>, f: &Node<T>) -> Vec<(Vec<usize>,usize)> where T: TerminalBin {
    let mut tab = Vec::new();
    let p = vec![0; dd.size().0];
    table_(dd, f.level().unwrap(), f, &p, &mut tab);
    tab
}

pub fn table_<T>(dd: &BDD<T>, level: Level, f: &Node<T>, path: &[usize], tab: &mut Vec<(Vec<usize>,usize)>) where T: TerminalBin {
    println!("enter {}", level);
    match f {
        Node::Terminal(_) if f == &dd.zero() => {
            println!("match terminal 1");
            let p = path.to_vec();
            tab.push((p,0));
            println!("{:?}", tab);
        },
        Node::Terminal(_) if f == &dd.one() => {
            println!("match terminal 2");
            let p = path.to_vec();
            tab.push((p,1));
            println!("{:?}", tab);
        },
        Node::NonTerminal(fnode) => {
            println!("match nonterminal");
            for (i,e) in fnode.iter().enumerate() {
                println!("loop {} level {} next node {:?}", i, level, e);
                let mut p = path.to_vec();
                match e.level() {
                    Some(l) if l == level-1 => {
                        println!("go 1");
                        p.push(i);
                        table_(dd, level-1, e, &p, tab);
                    },
                    Some(l) if l < level-1 => {
                        println!("go 2");
                        p.push(i);
                        table_(dd, level-1, e, &p, tab);
                    },
                    None if level == 0 => {
                        println!("go 3");
                        p.push(i);
                        table_(dd, level-1, e, &p, tab);
                    },
                    None if level > 0 => {
                        println!("go 4");
                        p.push(i);
                        table_(dd, level-1, e, &p, tab);
                    },
                    _ => (),
                }
            }
        },
        _ => (),
    };
}

fn bench_bdd1 () {
    let n = 1000;
    let mut f: BDD = BDD::new();
    let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
    let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();

    let mut b = f.one();
    for i in 0..n {
        b = f.and(&b, &x[i]);
    }    
    println!("bdd2 node {:?}", f.size());
}

fn bench_bdd2 () {
    let n = 1000;
    let mut f: BDD = BDD::new();
    let mut b = f.one();
    {
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();
    
        for i in (0..n).rev() {
            b = f.and(&b, &x[i]);
        }
    
        println!("bdd2 node {:?}", f.size());
    }
    {
        f.clear();
        f.rebuild(&vec![b]);
        println!("bdd2 node {:?}", f.size());
    }
}

fn bench_bdd3 () {
    let n = 3;
    let mut f: BDD = BDD::new();
    let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
    let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();

    let b = f.and(&x[0], &x[1]);
    let b = f.or(&b, &x[2]);
    println!("bdd2 node {:?}", f.size());
    println!("bdd table {:?}", table(&f, &b));
}

fn main() {
    clock("bdd1", bench_bdd1);
    clock("bdd2", bench_bdd2);
    clock("bdd3", bench_bdd3);
}