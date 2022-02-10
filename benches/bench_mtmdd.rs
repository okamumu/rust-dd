use dd::mtmdd::*;

fn bench_mtmdd1 () {
    let n = 100;
    let mut f: MTMDD<i64> = MTMDD::new();
    let h1 = f.header(1, "y1", 2);
    let h2 = f.header(2, "y2", 2);
    let h3 = f.header(3, "y3", 2);
    let consts = (0..n).into_iter().map(|i| f.value(i)).collect::<Vec<_>>();
    let y1 = f.node(&h1, &[consts[0].clone(), consts[1].clone()]).unwrap();
    let y2 = f.node(&h2, &[consts[0].clone(), consts[1].clone()]).unwrap();
    let y3 = f.node(&h3, &[consts[0].clone(), consts[1].clone()]).unwrap();
    // let tmp2 = f.mul(&consts[2], &y2);
    let tmp3 = f.mul(&consts[3], &y3);
    let b = Some(f.mul(&consts[2], &y2)).and_then(|x| Some(f.add(&y1, &x))).and_then(|x| Some(f.add(&x, &tmp3))).unwrap();
    // let b = f.add(&b, &tmp3);

    let mut buf = vec![];
    {
        let mut io = std::io::BufWriter::new(&mut buf);
        f.dot(&mut io, &b);
    }
    let s = std::str::from_utf8(&buf).unwrap();
    println!("{}", s);
}

// fn bench_mdd2 () {
//     let n = 1000;
//     let mut f: MDD = MDD::new();
//     let mut b = f.one();
//     {
//         let v = vec![f.zero(), f.zero(), f.zero(), f.zero(), f.one()];
//         let h = (0..n).into_iter().map(|i| f.header(i, &format!("x{}", i), 5)).collect::<Vec<_>>();
//         let x = (0..n).into_iter().map(|i| f.node(&h[i], &v).unwrap()).collect::<Vec<_>>();
    
//         let start = std::time::Instant::now();
    
//         for i in (0..n).rev() {
//             b = f.and(&b, &x[i]);
//         }
    
//         let end = start.elapsed();
//         println!("mdd3 node {:?}", f.size());
//         println!("mdd3 rev {} sec", end.as_secs_f64());
//     }
//     {
//         let start = std::time::Instant::now();
//         f.clear();
//         f.rebuild(&vec![b]);
//         let end = start.elapsed();
//         println!("mdd3 node {:?}", f.size());
//         println!("mdd3 clear {} sec", end.as_secs_f64());
//     }
// }

fn main() {
    bench_mtmdd1();
}