use bddcore::prelude::*;

// impl Drop for Node {
//     fn drop(&mut self) {
//         println!("Dropping Node{}", self.id());
//     }
// }

#[test]
fn new_test1() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    println!("{:?}", dd.get_node(&x));
    let y = dd.create_node(h2, dd.zero(), dd.one());
    println!("{:?}", dd.get_node(&y));
}

#[test]
fn test_union() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let h3 = dd.create_header(2, "z");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.create_node(h3, dd.zero(), dd.one());
    let tmp1 = dd.union(x, y);
    let tmp2 = dd.union(tmp1, z);
    println!("{}", dd.dot_string(&tmp2));
}

#[test]
fn test_intersect() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let h3 = dd.create_header(2, "z");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.create_node(h3, dd.zero(), dd.one());
    let tmp1 = dd.union(x, y);
    let tmp2 = dd.union(y, z);
    let tmp3 = dd.intersect(tmp1, tmp2);
    println!("{}", dd.dot_string(&tmp3));
}

#[test]
fn test_intersect2() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let h3 = dd.create_header(2, "z");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.create_node(h3, dd.zero(), dd.one());
    let tmp1 = dd.intersect(x, y);
    let tmp2 = dd.intersect(tmp1, z);
    println!("{}", dd.dot_string(&tmp2));
}

#[test]
fn test_setdiff() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let h3 = dd.create_header(2, "z");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.create_node(h3, dd.zero(), dd.one());
    let tmp1 = dd.union(x, y);
    let tmp2 = dd.union(x, z);
    let tmp3 = dd.setdiff(tmp1, tmp2);
    println!("{}", dd.dot_string(&tmp3));
}

#[test]
fn test_product() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let h3 = dd.create_header(2, "z");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.create_node(h3, dd.zero(), dd.one());
    let tmp1 = dd.product(x, y);
    let tmp2 = dd.union(tmp1, z);
    println!("{}", dd.dot_string(&tmp2));
}

#[test]
fn test_product2() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let h3 = dd.create_header(2, "z");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.create_node(h3, dd.zero(), dd.one());
    let tmp1 = dd.union(x, y);
    let tmp2 = dd.union(x, z);
    let tmp3 = dd.product(tmp1, tmp2);
    println!("{}", dd.dot_string(&tmp3));
}

#[test]
fn test_divide() {
    let mut dd = ZddManager::new();
    let h1 = dd.create_header(0, "a");
    let h2 = dd.create_header(1, "b");
    let h3 = dd.create_header(2, "c");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.create_node(h3, dd.zero(), dd.one());
    let tmp = dd.product(x, y);
    let abc = dd.product(tmp, z);
    println!("abc\n{}", dd.dot_string(&abc));
    let bc = dd.product(y, z);
    println!("bc\n{}", dd.dot_string(&tmp));
    let ac = dd.product(x, z);
    println!("ac\n{}", dd.dot_string(&tmp));
    let tmp = dd.union(abc, bc);
    println!("abc+bc\n{}", dd.dot_string(&tmp));
    let s = dd.union(tmp, ac);
    println!("abc+bc+ac\n{}", dd.dot_string(&s));
    let tmp3 = dd.divide(s, bc);
    println!("(abc+bc+ac)/bc\n{}", dd.dot_string(&tmp3));
}

