## relib-bss 0.8.0

- Added `BddNode::dual()` — the dual structure function `φ^D(x) = ¬φ(¬x)`, computed
  by an O(size) memoized recursion (swap each node's children, complement terminals).
  It preserves monotonicity.
- Added `BddNode::mincut() -> Option<BddNode>` — the minimal **cut** vectors of the
  structure function, defined as `dual().minpath()` (the minimal path vectors of the
  dual). `None` when the function is not monotone/coherent. `minpath` gives minimal
  **path** vectors, `mincut` the minimal **cut** vectors; they are dual (series
  `x&y`: path `{x,y}`, cut `{x},{y}`; parallel `x|y` is the reverse). Additive.

## relib-bss 0.7.0

- **Breaking**: `BddNode::minpath` now returns `Option<BddNode>` (`None` when the
  function is not monotone/coherent). The separate `minpath_checked` and the
  panicking behavior are removed — a single `minpath()` returns the minimal
  path/cut sets or `None`. `bdd_minsol::minsol` is unchanged (already `Option`).

## relib-bss 0.6.0

- Coherence-checked `minpath`. `BddNode::minpath_checked() -> Option<BddNode>`
  returns `None` when the function is not monotone (coherent); the Rauzy minsol
  decomposition is only valid for coherent functions. Detection is folded into
  the minsol recursion via the local invariant `and(f0, f1) == f0` (`f0 ⇒ f1`,
  an O(1) id compare on the canonical BDD) and aborts on the first violation, so
  a non-coherent input never builds a meaningless result.
- `minpath()` keeps its signature but now **panics** on a non-coherent input
  (previously it returned a silently-wrong result).
- **Breaking**: `bdd_minsol::minsol` now returns `Option<NodeId>`.

## relib-bss 0.5.1

- Version bump for workspace lockstep; no functional changes.

## relib-bss 0.5.0

- Inherit the `relib-bdd` 0.5.0 native `ite` and commutative operand ordering
  (faster `ite`/`kofn`/`@match` construction). No API change.

## relib-bss 0.4.1

- Fix exponential-time `BddMgr::kofn`: the naive Shannon recursion never memoized
  the `(k, index)` subproblems, so it made `O(2^n)` recursive calls despite the
  polynomial-size result (n=24 took ~40 ms). Memoize on `(k, start)` → `O(n·k)`
  (n=24 now ~0.15 ms). The result BDD is unchanged (canonical); also fixes a
  `usize` underflow that panicked on `kofn(0, ..)`.
- Inherit the `relib-bdd` computed-table speedup (no API change).

## relib-bss 0.4.0

- First release on crates.io, published as `relib-bss` (import name stays `bss`).
- Add garbage collection to `BddMgr`: `gc` (mark-and-sweep), `set_gc_threshold` for
  automatic threshold-triggered collection, and `live_node_count`.
- Inherit the `relib-bdd` u32 node narrowing (roughly half-size BDD nodes) and the
  `apply` hot-path short-circuit.

## bss 0.3.3

- add clear_cache

## bss 0.3.2

- add get_children
- add get_id2 in mss

## bss 0.3.1

- add and, or, kofn

## rust-dd 0.3.0

- make workspace including the following crates:
    - common v0.3.0
    - bddcore v0.3.0
    - mddcore v0.3.0
    - bss v0.3.0
    - mss v0.3.0

## rust-dd 0.2.0

- Add mtmdd2 which uses both boolean and integer values
- Change the interface of bdd, bdd_mut, zdd, zdd_mut by removing `node` method

## rust-dd 0.1.0

- first release

