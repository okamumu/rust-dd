//! Multi-state system (MSS) reliability analysis over MTMDD2.
//!
//! This crate provides an ergonomic, value-style API (`MddMgr<V>` / `MddNode<V>`) on top
//! of the arena-based MTMDD2 engine in `relib-mdd` (`mddcore`). It computes multi-state
//! system probability, minimal path/cut vectors, and counting, keeping the
//! decision-diagram engine and the analysis passes tightly integrated.
//!
//! It is the Rust engine behind the MSS/MDD side of the
//! [`relibmss`](https://github.com/MssReliab/relibmss) Python package. `relibmss` is the
//! interface for general users and students; use this crate to write reliability
//! experiments directly in Rust.
//!
//! # Example
//!
//! ```
//! use mss::prelude::*;
//! use std::collections::HashMap;
//!
//! let mut mgr: MddMgr<i32> = MddMgr::new();
//! mgr.defvar("x", 3); // a 3-state variable
//! mgr.defvar("y", 3);
//! mgr.defvar("z", 3);
//!
//! let mut vars = HashMap::new();
//! vars.insert("x".to_string(), 3);
//! vars.insert("y".to_string(), 3);
//! vars.insert("z".to_string(), 3);
//!
//! // Structure function given in reverse Polish notation: x * (y + z).
//! let node = mgr.rpn("x y z + *", &vars).unwrap();
//! println!("{}", node.dot());
//! ```
//!
//! # Extending with your own analysis
//!
//! New analysis passes are written by traversing the diagram through the
//! `common::DDForest` trait (`get_node` / `level` / …) without touching the MDD engine.
//! The `mdd_prob` / `mdd_path` / `mdd_minsol` / `mdd_count` modules serve as reference
//! implementations.

pub mod mdd_path;
pub mod mdd_prob;
pub mod mdd_count;
pub mod mdd_minsol;
pub mod mss;

pub mod prelude {
    pub use mddcore::prelude::*;
    pub use crate::mdd_path::*;
    pub use crate::mdd_minsol::*;
    pub use crate::mdd_prob::*;
    pub use crate::mdd_count::*;
    pub use crate::mss::*;
}
