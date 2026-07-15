# relib-bss

Binary-state system (BSS) reliability analysis over Binary Decision Diagrams: system
probability, path/cut enumeration, minimal solutions (minimal cut/path sets), k-of-n, and
solution counting.

This is the Rust engine behind the BSS/BDD side of the
[`relibmss`](https://github.com/okamumu/relibmss) Python package. `relibmss` is the
interface for general users; use this crate to write reliability experiments directly in
Rust.

Import name is `bss`:

```rust
use bss::prelude::*;
```

License: MIT.
