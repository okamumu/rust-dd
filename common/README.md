# relib-common

Shared primitives and traits for the `relib-*` decision-diagram crates: type aliases
(`NodeId`, `HeaderId`, `Level`), the `BddHashMap`/`BddHashSet` aliases, the core traits
(`Terminal`, `NonTerminal`, `NodeHeader`, `DDForest`, `Dot`), and the shared direct-mapped
`ComputeCache` used to memoize `apply` results in the BDD/MDD engines.

This crate is the shared base of the Rust reliability-analysis engine behind the
[`relibmss`](https://github.com/MssReliab/relibmss) Python package. It is not meant to be
used directly; depend on `relib-bdd`, `relib-mdd`, `relib-bss`, or `relib-mss` instead.

Import name is `common`:

```rust
use common::prelude::*;
```

License: MIT.
