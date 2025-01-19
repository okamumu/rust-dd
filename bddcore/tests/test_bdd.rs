use bddcore::prelude::*;

// impl Drop for Node {
//     fn drop(&mut self) {
//         println!("Dropping Node{}", self.id());
//     }
// }

#[test]
fn new_test1() {
    let mut dd = BddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    println!("{:?}", dd.get_node(&x));
    let y = dd.create_node(h2, dd.zero(), dd.one());
    println!("{:?}", dd.get_node(&y));
}

#[test]
fn test_and() {
    let mut dd = BddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.and(x, y);
    println!("{:?}", dd.get_node(&x));
    println!("{:?}", dd.get_node(&y));
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_or() {
    let mut dd = BddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.or(x, y);
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_xor() {
    let mut dd = BddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.xor(x, y);
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_not() {
    let mut dd = BddManager::new();
    let h1 = dd.create_header(0, "x");
    let h2 = dd.create_header(1, "y");
    let x = dd.create_node(h1, dd.zero(), dd.one());
    let y = dd.create_node(h2, dd.zero(), dd.one());
    let z = dd.or(x, y);
    let z = dd.not(z);
    println!("{}", dd.dot_string(&z));
}
