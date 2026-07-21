# relib-bss

Binary-state system (BSS) reliability analysis over Binary Decision Diagrams: system
probability, path/cut enumeration, minimal path vectors (`minpath`) and minimal cut
vectors (`mincut`) of a structure function, the dual structure function (`dual`), k-of-n,
and solution counting.

This is the Rust engine behind the BSS/BDD side of the
[`relibmss`](https://github.com/MssReliab/relibmss) Python package. `relibmss` is the
interface for general users; use this crate to write reliability experiments directly in
Rust.

Import name is `bss`:

```rust
use bss::prelude::*;
```

License: MIT.
