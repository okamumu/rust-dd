use mddcore::prelude::*;

// impl Drop for Node {
//     fn drop(&mut self) {
//         println!("Dropping Node{}", self.id());
//     }
// }

#[test]
fn test_create_node() {
    let mut dd = MtMdd2Manager::new();
    let h1 = dd.create_header(0, "x", 2);
    let h2 = dd.create_header(1, "y", 2);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let x = dd.create_node(h1, &[v0, v1]);
    let y = dd.create_node(h2, &[x, v1]);
    println!("{}", dd.dot_string(&y));
}

// #[test]
// fn test_create_node2() {
//     let mut dd = MtMdd2Manager::new();
//     let x = gen_var(&mut dd, "x", 0, &[0, 1, 2, 3, 4, 5]);
//     let y = gen_var(&mut dd, "y", 1, &[0, 1, 2, 3, 4, 5]);
//     let z = dd.add(x, y);
//     println!("{}", dd.dot_string(z));
// }

// #[test]
// fn test_eq() {
//     let mut dd = MtMdd2Manager::new();
//     let x = gen_var(&mut dd, "x", 0, &[0, 1, 2]);
//     let y = gen_var(&mut dd, "y", 1, &[0, 1, 2]);
//     let z = gen_var(&mut dd, "z", 2, &[0, 1, 2]);
//     let f = dd.add(x, y);
//     let g = dd.sub(z, x);
//     let h = dd.eq(f, g);
//     println!("{}", dd.dot_string(h));
// }

// #[test]
// fn test_ite() {
//     let mut dd = MtMdd2Manager::new();
//     let x = gen_var(&mut dd, "x", 0, &[0, 1, 2]);
//     let y = gen_var(&mut dd, "y", 1, &[0, 1, 2]);
//     let z = gen_var(&mut dd, "z", 2, &[0, 1, 2]);
//     let f = dd.add(x, y);
//     let g = dd.eq(f, z);
//     let g = dd.ite(g, x, z);
//     println!("{}", dd.dot_string(g));
// }

// #[test]
// fn test_build_rpn() {
//     // case(x + y <= 5 => x, x + y >= 3 => y, _ => x), 0 <= x <= 5, 0 <= y <= 5
//     let mut dd = MtMdd2Manager::new();
//     let x = gen_var(&mut dd, "x", 0, &[0, 1, 2, 3, 4, 5]);
//     let y = gen_var(&mut dd, "y", 1, &[0, 1, 2, 3, 4, 5]);
//     // x y + 5 <= x x y + 3 >= y x ? ?
//     let tokens = vec![
//         Token::Value(x),
//         Token::Value(y),
//         Token::Add,
//         Token::Value(dd.value(5)),
//         Token::Lte,
//         Token::Value(x),
//         Token::Value(x),
//         Token::Value(y),
//         Token::Add,
//         Token::Value(dd.value(3)),
//         Token::Gte,
//         Token::Value(y),
//         Token::Value(x),
//         Token::IfElse,
//         Token::IfElse,
//     ];
//     let res = build_from_rpn(&mut dd, &tokens);
//     match res {
//         Ok(res) => {
//             println!("{}", dd.dot_string(res))
//         }
//         Err(e) => {
//             println!("{}", e)
//         }
//     }
// }

// #[test]
// fn test_ope6() {
//     // case(x + y <= 5 => x, x + y >= 3 => y, _ => x), 0 <= x <= 5, 0 <= y <= 5
//     let mut dd = MtMdd2Manager::new();
//     let x = gen_var(&mut dd, "x", 1, &[0, 1, 2, 3, 4, 5]);
//     let y = gen_var(&mut dd, "y", 2, &[0, 1, 2, 3, 4, 5]);
//     let res = build_from_rpn! {dd, x y + 5 <= x x y + 3 >= y x ? ?};
//     match res {
//         Ok(res) => {
//             println!("{}", dd.dot_string(res))
//         }
//         Err(e) => {
//             println!("{}", e)
//         }
//     }
// }
