fn bench_mdd1 () {
    use dd::mdd::*;
    let n = 1000;
    let mut f = MDD::new();
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
    use dd::mdd::*;
    let n = 1000;
    let mut f = MDD::new();
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

fn bench_bdd1 () {
    use dd::bdd::*;
    let n = 1000;
    let mut f = BDD::new();
    let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
    let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();

    let mut b = f.one();

    let start = std::time::Instant::now();

    for i in 0..n {
        b = f.and(&b, &x[i]);
    }

    let end = start.elapsed();
    println!("bdd2 node {:?}", f.size());
    println!("bdd2 {} sec", end.as_secs_f64());
}

fn bench_bdd2 () {
    use dd::bdd::*;
    let n = 1000;
    let mut f = BDD::new();
    let mut b = f.one();
    {
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.zero(), f.one()]).unwrap()).collect::<Vec<_>>();
    
        let start = std::time::Instant::now();
    
        for i in (0..n).rev() {
            b = f.and(&b, &x[i]);
        }
    
        let end = start.elapsed();
        println!("bdd2 node {:?}", f.size());
        println!("bdd2 rev {} sec", end.as_secs_f64());
    }
    {
        let start = std::time::Instant::now();
        f.clear();
        f.rebuild(&vec![b]);
        let end = start.elapsed();
        println!("bdd2 node {:?}", f.size());
        println!("bdd2 clear {} sec", end.as_secs_f64());
    }
}

fn main() {
    bench_bdd1();
    bench_bdd2();
    bench_mdd1();
    bench_mdd2();
}