//! Binary-state system (BSS) reliability analysis over Binary Decision Diagrams.
//!
//! This crate provides an ergonomic, value-style API (`BddMgr` / `BddNode`) on top of
//! the arena-based BDD engine in `relib-bdd` (`bddcore`). It computes system
//! probability, path/cut enumeration, minimal **path** vectors (`minpath`) and minimal
//! **cut** vectors (`mincut`) of a structure function, the dual structure function
//! (`dual`), k-of-n structures, and solution counting.
//!
//! It is the Rust engine behind the BSS/BDD side of the
//! [`relibmss`](https://github.com/MssReliab/relibmss) Python package. `relibmss` is the
//! interface for general users; use this crate to write reliability experiments directly
//! in Rust.
//!
//! # Example
//!
//! ```
//! use bss::prelude::*;
//! use std::collections::HashMap;
//!
//! let mut mgr = BddMgr::new();
//! let x = mgr.defvar("x");
//! let y = mgr.defvar("y");
//! let z = mgr.defvar("z");
//!
//! // Structure function of the system: (x AND y) OR z.
//! let f = x.and(&y).or(&z);
//!
//! // Component failure/working probabilities -> system probability.
//! let mut pv = HashMap::new();
//! pv.insert("x".to_string(), 0.2_f64);
//! pv.insert("y".to_string(), 0.3);
//! pv.insert("z".to_string(), 0.6);
//! let p = f.prob(&pv, &[true]);
//! println!("system probability = {p}");
//! ```
//!
//! # Extending with your own analysis
//!
//! New analysis passes are written by traversing the diagram through the
//! `common::DDForest` trait (`get_node` / `level` / …) without touching the BDD engine.
//! The `bdd_prob` / `bdd_path` / `bdd_minsol` / `bdd_count` / `bdd_kofn` modules serve as
//! reference implementations.

pub mod bdd_path;
pub mod bdd_minsol;
pub mod bdd_dual;
pub mod bdd_prob;
pub mod bdd_count;
pub mod bdd_kofn;
pub mod bss;

pub mod prelude {
    pub use bddcore::prelude::*;
    pub use crate::bdd_path::*;
    pub use crate::bdd_minsol::*;
    pub use crate::bdd_dual::*;
    pub use crate::bdd_prob::*;
    pub use crate::bdd_count::*;
    pub use crate::bss::*;
}
