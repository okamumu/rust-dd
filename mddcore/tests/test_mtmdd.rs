use mddcore::prelude::*;

// impl<V> Drop for Node<V> where V: TerminalNumberValue<V> {
//     fn drop(&mut self) {
//         println!("Dropping Node{}", self.id());
//     }
// }

#[test]
fn test_create_node() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 2);
    let h2 = dd.create_header(1, "y", 2);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let x = dd.create_node(h1, &[v0, v1]);
    println!("{:?}", dd.get_node(&x));
    let y = dd.create_node(h2, &[v0, v1]);
    println!("{:?}", dd.get_node(&y));
}

#[test]
fn test_add() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let v2 = dd.value(2);
    let x = dd.create_node(h1, &[v0, v1, v2]);
    let y = dd.create_node(h2, &[v0, v1, v2]);
    let z = dd.add(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_sub() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let v2 = dd.value(2);
    let x = dd.create_node(h1, &[v0, v1, v2]);
    let y = dd.create_node(h2, &[v0, v1, v2]);
    let z = dd.sub(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_mul() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let v2 = dd.value(2);
    let x = dd.create_node(h1, &[v0, v1, v2]);
    let y = dd.create_node(h2, &[v0, v1, v2]);
    let z = dd.mul(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_div() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let v2 = dd.value(2);
    let x = dd.create_node(h1, &[v0, v1, v2]);
    let y = dd.create_node(h2, &[v0, v1, v2]);
    let z = dd.div(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_min() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let v2 = dd.value(2);
    let x = dd.create_node(h1, &[v0, v1, v2]);
    let y = dd.create_node(h2, &[v0, v1, v2]);
    let z = dd.min(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_max() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let v2 = dd.value(2);
    let x = dd.create_node(h1, &[v0, v1, v2]);
    let y = dd.create_node(h2, &[v0, v1, v2]);
    let z = dd.max(x, y);
    println!("{:?}", dd.get_node(&z));
    println!("{}", dd.dot_string(&z));
}

#[test]
fn test_replace() {
    let mut dd = MtMddManager::new();
    let h1 = dd.create_header(0, "x", 3);
    let h2 = dd.create_header(1, "y", 3);
    let v0 = dd.value(0);
    let v1 = dd.value(1);
    let v2 = dd.value(2);
    let x = dd.create_node(h1, &[v0, v1, v2]);
    let y = dd.create_node(h2, &[v0, v1, v2]);
    let z = dd.div(x, y);
    let v100 = dd.value(100);
    let w = dd.replace(z, v100);
    println!("{:?}", dd.get_node(&w));
    println!("{}", dd.dot_string(&w));
}
