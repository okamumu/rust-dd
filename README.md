# relib-rs

Decision-diagram libraries (BDD, ZDD, MDD, MTMDD, MTMDD2) and reliability-analysis
layers built on top of them, in safe Rust. This workspace is the Rust engine behind the
[`relibmss`](https://github.com/MssReliab/relibmss) Python package: `relibmss` is the
interface for general users and students, while the crates here are meant for writing
reliability experiments directly in Rust.

## Crates

Published on crates.io under the `relib-` prefix. The internal library name (used in
`use` statements) is kept short:

| crates.io name | `use` name | contents |
|---|---|---|
| `relib-common` | `common`  | shared primitives, traits (`DDForest`, `Dot`, …) |
| `relib-bdd`    | `bddcore` | Binary Decision Diagrams (BDD) + Zero-suppressed BDDs (ZDD) |
| `relib-mdd`    | `mddcore` | Multi-valued DDs: MDD, MTMDD, MTMDD2 |
| `relib-bss`    | `bss`     | Binary-state system reliability over BDDs |
| `relib-mss`    | `mss`     | Multi-state system reliability over MTMDD2 |

Dependencies flow strictly upward: `common → {bddcore, mddcore} → {bss, mss}`.

The reliability layers (`bss`, `mss`) are the intended entry points for analysis; new
analysis passes are written against the `common::DDForest` traversal trait without
touching the DD engines. See `bss` / `mss` crate docs for usage.

Future decision-diagram families (SDD, MxD) are planned to be added under the same
`relib-` naming scheme.

## Build & test

```bash
cargo build
cargo test
```

## License

MIT
