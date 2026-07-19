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

