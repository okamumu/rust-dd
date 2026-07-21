## relib-mss 0.7.0

- **Breaking**: `MddNode::minpath` now returns `Option<MddNode<V>>` (`None` when
  the structure function is not coherent). `minpath_checked` and the panicking
  behavior are removed — a single `minpath()` returns the minimal path/cut
  vectors or `None`.

## relib-mss 0.6.0

- Coherence-checked `minpath`. `MddNode::minpath_checked() -> Option<MddNode<V>>`
  returns `None` when the structure function is not coherent (monotone). Detection
  is folded into the minsol recursion via the local invariant "cofactors ascend
  pointwise", an O(1) id compare on the canonical diagram after one meet apply:
  `and(c[i-1], c[i]) == c[i-1]` on the boolean forest, `min(c[i-1], c[i]) == c[i-1]`
  on the value (MTMDD) forest. Aborts on the first violation.
- `minpath()` keeps its signature but now **panics** on a non-coherent input
  (previously silently-wrong).
- **Breaking**: `mdd_minsol::minsol` now returns `Option<Node>`.

## relib-mss 0.5.1

- Inherit the `relib-mdd` 0.5.1 u32 node storage (lower memory on large
  multi-state diagrams). No API change.

## relib-mss 0.5.0

- Inherit the `relib-mdd` 0.5.0 native `ite` (boolean and value-side) and
  commutative operand ordering: faster `ifelse`/`switch`/`@match` and arithmetic
  construction. No API change.

## relib-mss 0.4.1

- Version bump for workspace lockstep; no functional changes.

## relib-mss 0.4.0

- First release on crates.io, published as `relib-mss` (import name stays `mss`).
- Add garbage collection to `MddMgr`: `gc` (mark-and-sweep), `set_gc_threshold` for
  automatic threshold-triggered collection, and `live_node_count`.
- Inherit the `relib-mdd` mark-and-sweep gc across MDD / MTMDD / MTMDD2.

## mss 0.3.6

- add clear_cache

## mss 0.3.5

- add get_children
- add get_id2 in mss

## mss 0.3.3

- add get_varorder

## mss 0.3.2

- add is_boolean, is_value

## mss 0.3.1

- add undet
- add ver_order

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

