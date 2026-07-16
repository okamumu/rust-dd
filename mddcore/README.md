# relib-mdd

Multi-valued Decision Diagrams in safe Rust: `MddManager` (boolean MDD),
`MtMddManager<V>` (multi-terminal, value-carrying), and `MtMdd2Manager<V>` which composes
an MDD (boolean part) and an MTMDD (value part). Implemented as an arena/forest with a
unique table (hash-consing) and an operation cache.

Part of the Rust reliability-analysis engine behind the
[`relibmss`](https://github.com/MssReliab/relibmss) Python package. For multi-state system
reliability analysis (state probability, minimal path/cut vectors) use the higher-level
[`relib-mss`](https://crates.io/crates/relib-mss) crate.

Import name is `mddcore`:

```rust
use mddcore::prelude::*;
```

License: MIT.
