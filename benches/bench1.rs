fn bench_mdd1 () {
    use dd::mdd::*;
    let n = 1000;
    let mut f = MDD::new();
    let mut b = f.get_one();
    {
        let v = vec![f.get_zero(), f.get_zero(), f.get_zero(), f.get_zero(), f.get_one()];
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i), 5)).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &v).unwrap()).collect::<Vec<_>>();
    
        let start = std::time::Instant::now();
    
        for i in x.into_iter() {
            b = f.and(&b, &i);
        }
    
        let end = start.elapsed();
        println!("mdd3 node {}", f.num_nodes);
        println!("mdd3 {} sec", end.as_secs_f64());
    }
}

fn bench_mdd2 () {
    use dd::mdd::*;
    let n = 1000;
    let mut f = MDD::new();
    let mut b = f.get_one();
    {
        let v = vec![f.get_zero(), f.get_zero(), f.get_zero(), f.get_zero(), f.get_one()];
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i), 5)).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &v).unwrap()).collect::<Vec<_>>();
    
        let start = std::time::Instant::now();
    
        for i in (0..n).rev() {
            b = f.and(&b, &x[i]);
        }
    
        let end = start.elapsed();
        println!("mdd3 node {}", f.num_nodes);
        println!("mdd3 rev {} sec", end.as_secs_f64());
    }
    {
        let start = std::time::Instant::now();
        f.clear();
        f.make_utable(&b);
        let end = start.elapsed();
        println!("mdd3 node {}", f.utable.len());
        println!("mdd3 clear {} sec", end.as_secs_f64());
    }
}

fn bench_bdd1 () {
    use dd::bdd::*;
    let n = 1000;
    let mut f = BDD::new();
    let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
    let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.get_zero(), f.get_one()]).unwrap()).collect::<Vec<_>>();

    let mut b = f.get_one();

    let start = std::time::Instant::now();

    for i in 0..n {
        b = f.and(&b, &x[i]);
    }

    let end = start.elapsed();
    println!("bdd2 node {}", f.num_nodes);
    println!("bdd2 {} sec", end.as_secs_f64());
}

fn bench_bdd2 () {
    use dd::bdd::*;
    let n = 1000;
    let mut f = BDD::new();
    let mut b = f.get_one();
    {
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i))).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &vec![f.get_zero(), f.get_one()]).unwrap()).collect::<Vec<_>>();
    
        let start = std::time::Instant::now();
    
        for i in (0..n).rev() {
            b = f.and(&b, &x[i]);
        }
    
        let end = start.elapsed();
        println!("bdd2 node {}", f.num_nodes);
        println!("bdd2 rev {} sec", end.as_secs_f64());
    }
    {
        let start = std::time::Instant::now();
        f.clear();
        f.make_utable(&b);
        let end = start.elapsed();
        println!("bdd2 node {}", f.utable.len());
        println!("bdd2 clear {} sec", end.as_secs_f64());
    }
}

fn main() {
    bench_bdd1();
    bench_bdd2();
    bench_mdd1();
    bench_mdd2();
}