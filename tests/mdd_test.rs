use dd::mdd::*;
// use dd::dot::Dot;
use dd::gc::Gc;

#[test]
fn bench_mdd1 () {
    let n = 10;
    let mut f: Mdd = Mdd::new();
    let mut b = f.one();
    {
        let v = vec![f.zero(), f.zero(), f.zero(), f.zero(), f.one()];
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i), 5)).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &v).unwrap()).collect::<Vec<_>>();
    
        for i in x.into_iter() {
            b = f.and(&b, &i);
        }
        println!("-mdd3 node {:?}", f.size());
    }
}

#[test]
fn bench_mdd2 () {
    let n = 10;
    let mut f: Mdd = Mdd::new();
    let mut b = f.one();
    {
        let v = vec![f.zero(), f.zero(), f.zero(), f.zero(), f.one()];
        let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i), 5)).collect::<Vec<_>>();
        let x = (0..n).into_iter().map(|i| f.node(&h[i], &v).unwrap()).collect::<Vec<_>>();
    
        for i in (0..n).rev() {
            b = f.and(&b, &x[i]);
        }
        println!("-mdd3 node {:?}", f.size());
    }
    {
        f.clear_cache();
        f.gc(&vec![b.clone()]);
        println!("-mdd3 node {:?}", f.size());
    }
}

