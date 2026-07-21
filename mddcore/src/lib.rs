//! Multi-valued Decision Diagrams (MDD, MTMDD, MTMDD2) in safe Rust.
//!
//! Where a BDD variable is binary, an MDD variable has `n` states. This crate provides
//! three engines:
//!
//! - [`MddManager`](mdd::MddManager) — boolean MDD (multi-valued variables, boolean terminals)
//! - [`MtMddManager<V>`](mtmdd::MtMddManager) — multi-terminal MDD, terminals carry a value of type `V`
//! - [`MtMdd2Manager<V>`](mtmdd2::MtMdd2Manager) — **composes** the two above into one
//!   structure; its `Node` enum tags a node as `Bool(NodeId)` or `Value(NodeId)`, so
//!   boolean conditions and value expressions can be mixed
//!
//! All are implemented as an **arena/forest**: nodes live in `Vec`s on the manager and
//! everything else holds [`NodeId`](common::common::NodeId) indices into them. A unique
//! table (hash-consing) keeps nodes canonical and shared, an operation cache memoizes
//! results, and mark-and-sweep garbage collection (`gc`) reclaims unreachable nodes.
//!
//! For **multi-state system reliability analysis** (state probability, minimal path/cut
//! vectors) use the higher-level [`relib-mss`](https://crates.io/crates/relib-mss) crate,
//! which wraps `MtMdd2Manager` in an ergonomic value-style API. This crate is the raw
//! engine.
//!
//! The package is named `relib-mdd` on crates.io but the import name is `mddcore`:
//!
//! ```
//! use mddcore::prelude::*;
//! ```
//!
//! Part of the Rust engine behind the
//! [`relibmss`](https://github.com/MssReliab/relibmss) Python package.

pub mod nodes;

pub mod mdd;
pub mod mdd_dot;
pub mod mdd_ops;

pub mod mtmdd;
pub mod mtmdd_dot;
pub mod mtmdd_ops;

pub mod mtmdd2;
pub mod mtmdd2_ops;
pub mod mtmdd2_dot;

pub mod zmdd;
pub mod zmdd_dot;
pub mod zmdd_ops;

pub mod prelude {
    pub use common::prelude::*;
    pub use crate::mdd::MddManager;
    pub use crate::mtmdd::MtMddManager;
    pub use crate::mtmdd2::MtMdd2Manager;
    pub use crate::zmdd::ZmddManager;
    pub use crate::mdd;
    pub use crate::mtmdd;
    pub use crate::mtmdd2::*;
    pub use crate::nodes::*;
}
