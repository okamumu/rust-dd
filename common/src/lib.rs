//! Shared primitives and traits for the `relib-*` decision-diagram crates.
//!
//! This crate carries the type aliases ([`NodeId`](common::NodeId),
//! [`HeaderId`](common::HeaderId), [`Level`](common::Level),
//! [`OperationId`](common::OperationId)), the [`BddHashMap`](common::BddHashMap) /
//! [`BddHashSet`](common::BddHashSet) aliases (std hashmaps with a `wyhash` hasher), and
//! the core traits ([`Terminal`](nodes::Terminal), [`NonTerminal`](nodes::NonTerminal),
//! [`NodeHeader`](nodes::NodeHeader), [`DDForest`](nodes::DDForest), [`Dot`](dot::Dot))
//! that the DD managers implement. It also provides the shared, direct-mapped
//! [`ComputeCache`](compute_cache::ComputeCache) used to memoize `apply` results.
//!
//! **This crate is not meant to be used directly.** Depend on one of the crates built on
//! top of it instead:
//!
//! - [`relib-bdd`](https://crates.io/crates/relib-bdd) — BDD / ZDD engines
//! - [`relib-mdd`](https://crates.io/crates/relib-mdd) — MDD / MTMDD / MTMDD2 engines
//! - [`relib-bss`](https://crates.io/crates/relib-bss) — binary-state system reliability
//! - [`relib-mss`](https://crates.io/crates/relib-mss) — multi-state system reliability
//!
//! The package is named `relib-common` on crates.io but the import name is `common`:
//!
//! ```
//! use common::prelude::*;
//! ```
//!
//! Part of the Rust engine behind the
//! [`relibmss`](https://github.com/MssReliab/relibmss) Python package.

pub mod common;
pub mod compute_cache;
pub mod dot;
pub mod nodes;

pub mod prelude {
    pub use std::ops::Index;
    pub use std::slice::Iter;
    pub use crate::common::{BddHashSet, BddHashMap};
    pub use crate::common::{HeaderId, Level, NodeId, OperationId};
    pub use crate::compute_cache::ComputeCache;
    pub use crate::nodes::{NonTerminal, Terminal, NodeHeader, DDForest};
    pub use crate::dot::Dot;
}
