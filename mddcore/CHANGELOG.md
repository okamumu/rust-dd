## relib-mdd 0.8.0

- Version bump for workspace lockstep; no functional changes.

## relib-mdd 0.7.0

- Version bump for workspace lockstep; no functional changes.

## relib-mdd 0.6.0

- Version bump for workspace lockstep; no functional changes.

## relib-mdd 0.5.1

- Store node ids/children as `u32` (like `relib-bdd`): `NonTerminalMDD` becomes
  `{ id: u32, header: u32, nodes: Box<[u32]> }` and the MDD/MTMDD unique tables
  become `(u32, Box<[u32]>) -> u32`, with `u32` freelists/vtable. Halves the id
  bytes stored per node and copied into the unique-table key. `NonTerminalMDD`
  drops the `common::NonTerminal`/`Index` impls for value-returning inherent
  accessors (`edge`, `iter`, `len`); casts confined to the node/table helpers.
  ~28% peak-RSS reduction on a 2.1M-node multi-state diagram; result is
  canonical (node counts/probabilities unchanged). No public API change
  (`NodeId` stays `usize`).

## relib-mdd 0.5.0

- **Breaking:** removed the public `get_cache` / `get_bcache` / `get_vcache`
  accessors on the MDD managers (the operation caches are now the shared
  `relib-common::ComputeCache`, an implementation detail).
- Adopt the shared direct-mapped `ComputeCache` for all four operation caches
  (`MddManager`, `MtMddManager`, and `MtMdd2Manager`'s bool/value caches).
- Commutative operand ordering before the computed-table key: boolean `and`/`or`/
  `xor` and MTMDD `add`/`mul`/`min`/`max` (`sub`/`div`/`rem` left as-is).
  ~2.2–2.6× on the MTMDD sum workload in `bdd-bench`.
- Native `ite`: the boolean MDD `ite` (was `or(and,and(not))`) becomes a k-ary
  Shannon recursion with its own cache; the value-side `MtMdd2::ite` (was
  `replace(vif(f,g), vif(!f,h))`) becomes a ternary cross-forest recursion. Up to
  ~1.9× (boolean) / ~1.3–1.6× (value) on `ite`-heavy workloads; result unchanged.

## relib-mdd 0.4.1

- Version bump for workspace lockstep; no functional changes.

## relib-mdd 0.4.0

- First release on crates.io, published as `relib-mdd` (import name stays `mddcore`).
- Add mark-and-sweep garbage collection (`gc`) across MDD / MTMDD / MTMDD2.
- `gc` now retains live operation-cache entries rather than flushing the whole cache.

## mddcore 0.3.3

- add clear_cache

## mddcore 0.3.2

- Add undet method

## mddcore 0.3.1

- Change the letters for true/false

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

