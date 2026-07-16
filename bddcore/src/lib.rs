//! Binary Decision Diagrams (BDD) and Zero-suppressed Decision Diagrams (ZDD) in safe Rust.
//!
//! Both engines are implemented as an **arena/forest**, not a tree of heap-allocated
//! nodes: nodes live in `Vec`s on the manager and everything else holds
//! [`NodeId`](common::common::NodeId) indices into them. A unique table (hash-consing)
//! keeps nodes canonical and shared, and an operation cache memoizes `apply` results.
//!
//! - [`BddManager`](bdd::BddManager) — binary decision diagrams
//! - [`ZddManager`](zdd::ZddManager) — zero-suppressed decision diagrams
//!
//! Both support mark-and-sweep garbage collection (`gc`), reclaiming nodes that are no
//! longer reachable from the roots you keep.
//!
//! For **binary-state reliability analysis** (probability, minimal cut/path sets, k-of-n)
//! use the higher-level [`relib-bss`](https://crates.io/crates/relib-bss) crate, which
//! wraps this engine in an ergonomic value-style API. This crate is the raw engine.
//!
//! The package is named `relib-bdd` on crates.io but the import name is `bddcore`:
//!
//! ```
//! use bddcore::prelude::*;
//! ```
//!
//! Part of the Rust engine behind the
//! [`relibmss`](https://github.com/MssReliab/relibmss) Python package.

pub mod nodes;

pub mod bdd;
pub mod bdd_ops;
pub mod bdd_dot;

pub mod zdd;
pub mod zdd_ops;
pub mod zdd_dot;

pub mod prelude {
    pub use common::prelude::*;
    pub use crate::nodes::*;
    pub use crate::bdd::BddManager;
    pub use crate::zdd::ZddManager;
}
