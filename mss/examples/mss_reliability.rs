//! Minimal multi-state system (MSS) reliability experiment.
//!
//! Build a multi-state structure function and compute its state probability and a count
//! of the states mapped into the success set.
//!
//! Run with: `cargo run -p relib-mss --example mss_reliability`

use mss::prelude::*;
use std::collections::{HashMap, HashSet};

fn main() {
    let mut mgr: MddMgr<i32> = MddMgr::new();
    mgr.defvar("x", 3); // 3-state component: states 0, 1, 2
    mgr.defvar("y", 3);
    mgr.defvar("z", 3);

    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 3);
    vars.insert("y".to_string(), 3);
    vars.insert("z".to_string(), 3);

    // Structure function in reverse Polish notation: x * (y + z).
    let mut node = mgr.rpn("x y z + *", &vars).expect("valid rpn");

    // Per-state probabilities for each component (index = state 0, 1, 2).
    let mut pv: HashMap<String, Vec<f64>> = HashMap::new();
    pv.insert("x".to_string(), vec![0.1, 0.3, 0.6]);
    pv.insert("y".to_string(), vec![0.2, 0.3, 0.5]);
    pv.insert("z".to_string(), vec![0.2, 0.2, 0.6]);

    // System is "up" when its output value is in this success set.
    let success: Vec<i32> = (2..=8).collect();

    let p: f64 = node.prob(&pv, &success);
    println!("system probability (output in {{2..=8}}) = {p}");

    let ss: HashSet<i32> = success.into_iter().collect();
    println!("number of state combinations in the success set = {}", node.mdd_count(&ss));
}
