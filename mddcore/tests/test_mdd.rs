use mddcore::prelude::*;

// impl Drop for Node {
//     fn drop(&mut self) {
//         println!("Dropping Node{}", self.id());
//     }
// }

#[test]
fn test_create_node() {
    let mut dd = MddManager::new();
    let h1 = dd.create_header(0, "x", 2);
    let h2 = dd.create_header(1, "y", 2);
    let x = dd.create_node(h1, &[dd.zero(), dd.one()]);
    println!("{:?}", dd.get_node(&x));
    let y = dd.create_node(h2, &[dd.zero(), dd.one()]);
    println!("{:?}", dd.get_node(&y));
}

#[test]
fn test_and() {
    let mut dd = MddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
    let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
    let z = dd.and(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_or() {
    let mut dd = MddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
    let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
    let z = dd.or(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_xor() {
    let mut dd = MddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
    let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
    let z = dd.xor(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_ite() {
    let mut dd = MddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let x = dd.create_node(h1, &[dd.zero(), dd.zero(), dd.one()]);
    let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
    let z = dd.ite(x, y, dd.one());
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_replace() {
    let mut dd = MddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let x = dd.create_node(h1, &[dd.zero(), dd.undet(), dd.one()]);
    let y = dd.create_node(h2, &[dd.zero(), dd.one(), dd.one()]);
    let z = dd.and(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
    let w = dd.replace(z, dd.one());
    println!("{:?}", dd.get_node(&w));
    println!("{}", dd.dot_string(&w));
}
