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

