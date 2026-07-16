# relib-bdd

Binary Decision Diagrams (BDD) and Zero-suppressed Binary Decision Diagrams (ZDD) in
safe Rust, implemented as an arena/forest with a unique table (hash-consing) and an
operation cache.

Part of the Rust reliability-analysis engine behind the
[`relibmss`](https://github.com/MssReliab/relibmss) Python package. For binary-state
reliability analysis (probability, minimal cut/path sets, k-of-n) use the higher-level
[`relib-bss`](https://crates.io/crates/relib-bss) crate.

Import name is `bddcore`:

```rust
use bddcore::prelude::*;
```

License: MIT.
