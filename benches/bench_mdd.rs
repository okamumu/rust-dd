use std::hash::Hash;
use dd::common::{
    HashMap,
    TerminalBinaryValue,
};
use dd::nodes::{
    NodeHeader,
};
use dd::mdd::*;

fn bench_mdd1 () {
    let n = 1000;
    let mut f: MDD = MDD::new();
    let mut b = f.one();
    {
        let v = vec![f.zero(), f.zero(), f.zero(), f.zero(), f.one()];
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i), 5)).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &v).unwrap()).collect::<Vec<_>>();
    
        let start = std::time::Instant::now();
    
        for i in x.into_iter() {
            b = f.and(&b, &i);
        }
    
        let end = start.elapsed();
        println!("mdd3 node {:?}", f.size());
        println!("mdd3 {} sec", end.as_secs_f64());
    }
}

fn bench_mdd2 () {
    let n = 1000;
    let mut f: MDD = MDD::new();
    let mut b = f.one();
    {
        let v = vec![f.zero(), f.zero(), f.zero(), f.zero(), f.one()];
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i), 5)).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &v).unwrap()).collect::<Vec<_>>();
    
        let start = std::time::Instant::now();
    
        for i in (0..n).rev() {
            b = f.and(&b, &x[i]);
        }
    
        let end = start.elapsed();
        println!("mdd3 node {:?}", f.size());
        println!("mdd3 rev {} sec", end.as_secs_f64());
    }
    {
        let start = std::time::Instant::now();
        f.clear();
        f.rebuild(&vec![b]);
        let end = start.elapsed();
        println!("mdd3 node {:?}", f.size());
        println!("mdd3 clear {} sec", end.as_secs_f64());
    }
}

fn bench_mdd3 () {
    let data = [
        vec![2, 1, 0],
        vec![1, 0, 1],
    ];

    let mut f: MDD = MDD::new();
    let one = f.one();
    let zero = f.zero();
    let h1 = f.header(1, "x1", 3);
    let h2 = f.header(2, "x2", 2);
    let h3 = f.header(3, "x3", 2);

    creat_mdd_node(&mut f, &vec![h1,h2,h3], &data);
}

fn creat_mdd_node<V,T>(dd: &mut MDD<V>, headers: &[NodeHeader], data: &[Vec<T>]) where T: Clone+PartialEq+Eq+Hash+std::fmt::Debug, V: TerminalBinaryValue {
    let mut id = 0;
    let mut table: HashMap<Vec<T>,i32> = HashMap::new();
    let max_level = headers.len();
    let level = 0;
    let mut paths = data.iter().map(|x| x.to_vec()).collect::<Vec<_>>();
    let mut nodes = vec![dd.node(&headers[0], &(0..headers[0].edge_num()).map(|_| Default::default()).collect::<Vec<_>>())];
    for i in 0..max_level {
        node = nodes.pop();
        for v in paths.iter_mut() {
            x, rest = v
            match table.get(rest) {
                Some(n) => n,
                None => {
                    let n = dd.node(&headers[i], &());
                    replace(node[i], &n);
                    table.insert(rest.to_vec(), n.clone());
                    n
                }
            }
            let _ = v.pop();
        }
    }
    println!("{:?}", table);
}

fn main() {
    bench_mdd1();
    bench_mdd2();
    bench_mdd3();
}