# relib-bss

Binary-state system (BSS) reliability analysis over Binary Decision Diagrams: system
probability, path/cut enumeration, the dual structure function (`dual`), k-of-n, and
solution counting. The minimal path vectors (`minpath`) and minimal cut vectors (`mincut`)
are returned as genuine ZDD set families (via `BssMgr`) supporting set algebra
(`union`/`intersect`/`setdiff`/`product`/`divide`). A `ZddMgr` can also build set families
standalone (`empty`/`base`/`singleton`/`from_sets`).

This is the Rust engine behind the BSS/BDD side of the
[`relibmss`](https://github.com/MssReliab/relibmss) Python package. `relibmss` is the
interface for general users; use this crate to write reliability experiments directly in
Rust.

Import name is `bss`:

```rust
use bss::prelude::*;
```

License: MIT.
