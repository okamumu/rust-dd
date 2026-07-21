//! Minimal binary-state system (BSS) reliability experiment.
//!
//! Build a binary structure function and compute its probability, Birnbaum importance,
//! and the number of minimal path sets.
//!
//! Run with: `cargo run -p relib-bss --example bss_reliability`

use bss::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut mgr = BddMgr::new();
    let x = mgr.defvar("x");
    let y = mgr.defvar("y");
    let z = mgr.defvar("z");

    // Structure function: (x AND y) OR z.
    let f = x.and(&y).or(&z);

    let mut pv = HashMap::new();
    pv.insert("x".to_string(), 0.2_f64);
    pv.insert("y".to_string(), 0.3);
    pv.insert("z".to_string(), 0.6);

    let p = f.prob(&pv, &[true]);
    println!("system probability = {p}");

    // Birnbaum importance of each component.
    let importance = f.bmeas(&pv, &[true]);
    println!("Birnbaum importance = {importance:?}");

    // Number of minimal path sets.
    // minpath returns None for a non-monotone function; (x AND y) OR z is monotone.
    let minpaths = f.minpath().expect("a coherent (monotone) function");
    println!("number of minimal path sets = {}", minpaths.bdd_count(&[true]));
}
