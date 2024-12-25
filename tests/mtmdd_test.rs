// use dd::mtmdd::*;
// use dd::dot::Dot;

// #[test]
// fn integration_test_mtmdd1 () {
//     let n = 100;
//     let mut f: MtMdd<i64> = MtMdd::new();
//     let h1 = f.header(1, "y1", 2);
//     let h2 = f.header(2, "y2", 2);
//     let h3 = f.header(3, "y3", 2);
//     let consts = (0..n).into_iter().map(|i| f.value(i)).collect::<Vec<_>>();
//     let y1 = f.node(&h1, &[consts[0].clone(), consts[1].clone()]).unwrap();
//     let y2 = f.node(&h2, &[consts[0].clone(), consts[1].clone()]).unwrap();
//     let y3 = f.node(&h3, &[consts[0].clone(), consts[1].clone()]).unwrap();
//     // let tmp2 = f.mul(&consts[2], &y2);
//     let tmp3 = f.mul(&consts[3], &y3);
//     let b = Some(f.mul(&consts[2], &y2)).and_then(|x| Some(f.add(&y1, &x))).and_then(|x| Some(f.add(&x, &tmp3))).unwrap();
//     // let b = f.add(&b, &tmp3);

//     let mut buf = vec![];
//     {
//         let mut io = std::io::BufWriter::new(&mut buf);
//         b.dot(&mut io);
//     }
//     let s = std::str::from_utf8(&buf).unwrap();
//     println!("{}", s);
// }
