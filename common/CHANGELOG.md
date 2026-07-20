## relib-common 0.6.0

- Version bump for workspace lockstep; no functional changes.

## relib-common 0.5.1

- Version bump for workspace lockstep; no functional changes.

## relib-common 0.5.0

- Add a shared direct-mapped, lossy computed table `ComputeCache` (CUDD-style),
  moved here from `relib-bdd` so the BDD and MDD engines share one implementation.
  Public API: `get`/`put`, `retain_live` (op-keyed caches: `k0` is an op code),
  `retain_live3` (ternary caches whose `k0` is also a node id, e.g. an `ite`
  cache), `clear`, `len`. Additive; no breaking change.

## relib-common 0.4.1

- Version bump for workspace lockstep; no functional changes.

## relib-common 0.4.0

- First release on crates.io, published as `relib-common` (import name stays `common`).
- No API change from 0.3.0.

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

