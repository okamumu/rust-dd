## Unreleased

- **Docs**: `bmeas` rustdoc now documents the interval-arithmetic behavior — instantiating the
  generic `T` with an interval type gives a guaranteed but *conservative* enclosure (dependency
  problem + worst-case subtraction in the `D_j` difference; `Σ_j p = 1` not enforced). No code
  change.

## relib-mss 0.11.0

- **New: multi-state Birnbaum importance — `MddNode::bmeas`** (`mdd_prob::bmeas`). Returns,
  per variable, the adjacent-state differences
  `D_{i,j} = P(φ∈ss | x_i=j) − P(φ∈ss | x_i=j−1)` (vector length `M_i − 1`), computed in one
  backward-differentiation pass — the multi-state generalization of `relib-bss`'s `bmeas`
  (which is the binary case `P(φ∈ss|x=1) − P(φ∈ss|x=0)`). The difference form is the correct
  quantity on a reduced diagram: variables skipped on a path are irrelevant there and cancel
  out of every `D_{i,j}`. Real-valued only this round (no interval version yet).

## relib-mss 0.10.0

- **New: `ZmddMgr` / `ZmddNode`** — minimal path vector families as genuine ZMDDs, with the
  label-wise set operations `intersect` / `setdiff`, plus `count` / `extract`. (Only
  `intersect`/`setdiff` this round — see the crate TODO for the rest.)
- **New: `MssMgr`** — the multi-state manager, mirroring `bss::BssMgr`. It owns an `MddMgr`
  (structure functions over MTMDD2) and a `ZmddMgr` (families), and provides the analysis
  spanning both: **`MssMgr::minpath(&node) -> Option<ZmddNode>`** returns the minimal path
  vectors as a genuine ZMDD family. Build expressions through the delegated MDD API
  (`defvar`/`rpn`/`min`/`max`/… or the `MddNode` operators).
- **Source reorg** (mirrors the `bss` layout `bdd.rs`/`bss.rs`/`zdd.rs`): the MDD wrapper
  moved from `mss.rs` to **`mdd.rs`** (`MddMgr`/`MddNode`); new **`mss.rs`** holds `MssMgr`;
  `zmdd.rs` is the ZMDD wrapper. Prelude re-exports unchanged, so `use mss::prelude::*` still
  works.
- **Breaking / removed** (superseded by the genuine `ZmddNode`, as on the BSS side):
  - `minpath` moved off `MddNode` onto `MssMgr` and now returns a genuine `ZmddNode`
    (was an `MddNode` read with ZMDD semantics via `ZmddMgr::from_minsol`).
  - Removed the fake-ZMDD readers `MddNode::zmdd_extract` / `MddNode::zmdd_count`,
    `mdd_path::ZMddPath`, and `mdd_count::{zmdd_count, vzmdd_count, bzmdd_count}`.
    `MddNode::mdd_extract` / `mdd_count` (full assignments) are retained.
  - The ZMDD path iterator `ZmddSetPath` was renamed `ZmddPath` (parity with `bss::ZddPath`).

## relib-mss 0.9.1

- **Bug fix (correctness): `minpath` / `mincut` produced non-minimal path/cut vectors**
  for structure functions involving subsumption (e.g. `max(min(x,y), z)` gained a spurious
  `(y=1,z=2)`; the boolean `x&y|z` gained `{y,z}`). Conjunctive shapes (`min`) were
  unaffected. Root cause: `mdd_minsol::{vwithout,bwithout}` expanded every branch of the
  reference cofactor when the minsol family was a terminal, fabricating vectors with
  positive components that should be 0. Fixed to recurse into the reference's **zero
  branch** only (same principle as the existing `level(f) > level(g)` arm). Verified
  exhaustively against brute-force minimal path vectors for all monotone functions of
  n=3/K=2 and n=2/K=3.

## relib-mss 0.9.0

- Version bump for workspace lockstep; no functional changes. (`relib-mss`: the multi-state minsol/minpath is unchanged; the ZDD set-family work landed in `relib-bss` only.)

## relib-mss 0.8.0

- Version bump for workspace lockstep; no functional changes. (`dual`/`mincut`
  were added to `relib-bss` only; the multi-state dual is deferred.)

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

