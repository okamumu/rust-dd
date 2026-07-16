# relib-mss

Multi-state system (MSS) reliability analysis over MTMDD2: state probability, minimal
path/cut vectors, and counting. A Rust toolkit for multi-state reliability that keeps the
decision-diagram engine and the analysis passes tightly integrated.

This is the Rust engine behind the MSS/MDD side of the
[`relibmss`](https://github.com/MssReliab/relibmss) Python package. `relibmss` is the
interface for general users and students; use this crate to write reliability experiments
directly in Rust.

Import name is `mss`:

```rust
use mss::prelude::*;
```

New analysis passes are written against the `common::DDForest` traversal trait without
touching the DD engine; the existing `mdd_prob` / `mdd_path` / `mdd_minsol` / `mdd_count`
modules serve as reference implementations.

License: MIT.
