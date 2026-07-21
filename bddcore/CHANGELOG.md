## relib-bdd 0.7.0

- Version bump for workspace lockstep; no functional changes.

## relib-bdd 0.6.0

- Version bump for workspace lockstep; no functional changes.

## relib-bdd 0.5.1

- Version bump for workspace lockstep; no functional changes.

## relib-bdd 0.5.0

- Native `ite(f,g,h)`: replace the composite `or(and(f,g), and(not f,h))` (four
  apply traversals per call) with a single Shannon recursion over the top
  variable, backed by a dedicated computed table. `ite`-heavy workloads (e.g.
  k-of-n) build ~5–12× faster in `bdd-bench`; result is unchanged (canonical).
- Commutative operand ordering: canonicalize the operand pair in `and`/`or`/`xor`
  before the computed-table key, so `op(a,b)` and `op(b,a)` share an entry
  (~2.5× on symmetric functions like n-queens).
- Use the shared `relib-common::ComputeCache` (the direct-mapped table moved out
  of this crate). No public API change (operation signatures are unchanged).

## relib-bdd 0.4.1

- Replace the growing `HashMap` operation cache with a fixed, direct-mapped,
  lossy computed table (CUDD-style): one array load/store per `apply` instead of
  probing plus periodic rehash. The cache is a memoization hint, so a lossy array
  is safe; `gc` invalidates entries that reference reclaimed slots. Near-parity
  with CUDD on large-DD workloads in the `bdd-bench` comparison. No API change.

## relib-bdd 0.4.0

- First release on crates.io, published as `relib-bdd` (import name stays `bddcore`).
- Add mark-and-sweep garbage collection (`gc`) with a free list, so nodes unreachable
  from the roots are reclaimed instead of growing the arena forever.
- `gc` now retains live operation-cache entries rather than flushing the whole cache.
- Narrow `NonTerminalBDD` fields and the unique/cache tables to `u32`, halving node size.
- Add a hot-path short-circuit in `apply`.

## bddcore 0.3.1

- add clear_cache

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

