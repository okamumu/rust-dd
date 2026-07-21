# relib-rs Developer Guide

Internals reference for contributors: crate layout, the arena/forest model, the apply
algorithms, the cache strategy, garbage collection, and the public API map. For build/test
commands and the release runbook see the repo `CLAUDE.md`; for per-crate API details see the
rustdoc (`cargo doc --no-deps --workspace --open`, or docs.rs).

> Scope: this documents the **product-track** engine (`common`, `bddcore`, `mddcore`,
> `bss`, `mss`). `evmdd_core/` and `benches/` are out of the workspace build and not covered.

---

## 1. Crate layout & dependency direction

Dependencies flow strictly upward; lower crates never depend on higher ones.

```
        bss (relib-bss)          mss (relib-mss)          ← reliability layers (user-facing)
           │                        │
        bddcore (relib-bdd)     mddcore (relib-mdd)       ← decision-diagram engines
           └──────────┬─────────────┘
                   common (relib-common)                 ← shared primitives + ComputeCache
```

| dir | crates.io | lib name (`use`) | contents |
|---|---|---|---|
| `common/`  | `relib-common` | `common`  | type aliases, hashmap aliases, core traits, `ComputeCache` |
| `bddcore/` | `relib-bdd`    | `bddcore` | `BddManager` (BDD), `ZddManager` (ZDD), `_ops`/`_dot`/`_stack` |
| `mddcore/` | `relib-mdd`    | `mddcore` | `MddManager`, `MtMddManager<V>`, `MtMdd2Manager<V>` |
| `bss/`     | `relib-bss`    | `bss`     | `bdd` (`BddMgr`/`BddNode`), `bss` (`BssMgr`), `zdd` (`ZddMgr`/`ZddNode`) + `bdd_prob`/`bdd_path`/`bdd_minsol`/`bdd_dual`/`bdd_count`/`bdd_kofn` + `zdd_convert`/`zdd_count`/`zdd_path` |
| `mss/`     | `relib-mss`    | `mss`     | `mdd` (`MddMgr<V>`/`MddNode<V>`), `mss` (`MssMgr<V>`), `zmdd` (`ZmddMgr<V>`/`ZmddNode<V>`) + `mdd_prob`/`mdd_path`/`mdd_minsol`/`mdd_count` + `zmdd_convert` |

The crates.io **package** name (`relib-*`) differs from the **lib** name so `use` paths stay
stable. Every crate re-exports through a `prelude` module (`use common::prelude::*`).

---

## 2. Core architecture: the forest/arena model

Every DD manager is a **forest/arena**, not a tree of heap-allocated nodes.

- **Nodes and headers live in `Vec`s on the manager**; everything else holds `NodeId` /
  `HeaderId` **indices** (`common::common`: `type NodeId = usize`, `HeaderId = usize`,
  `Level = usize`). No `Rc`/`Box` node graphs inside the core crates.
- **Unique table (`utable`)** — maps a node's structural key (header + children ids) to its
  `NodeId`. Guarantees canonical, hash-consed nodes: identical subgraphs are never
  duplicated. Backed by `BddHashMap` (std `HashMap` + `wyhash`). Lossless.
- **Operation cache (`cache`)** — memoizes apply results so an apply runs in time
  proportional to the product of operand sizes. The direct-mapped `common::ComputeCache`
  (§4). Lossy (a hint).
- **Terminals** are special-cased, not stored as normal nodes:
  - BDD / boolean MDD: `Zero`, `One`, `Undet` (fixed ids 0/1/2).
  - MTMDD value forest: `Undet` + integer value terminals (hash-consed in a `vtable`).
- **Non-terminals** reference a shared `NodeHeader` (level, label, edge count) by `HeaderId`.
- **`DDForest` trait** (`common::nodes`) — `get_node`, `get_header`, `level`, `label` — lets
  generic algorithms (dot, counting) work across DD types.

### u32 node storage (memory)

`NodeId`/`HeaderId` are `usize` at the API boundary, but nodes and unique-table keys **store
ids as `u32`** (node/header counts fit in 32 bits). This halves the id bytes stored per node
and copied/hashed per `create_node`. Casts are confined to the node accessors and table
helpers; users never see `u32`.

- `NonTerminalBDD { id: u32, header: u32, edges: [u32;2] }`, `utable: (u32,u32,u32)→u32`.
- `NonTerminalMDD { id: u32, header: u32, nodes: Box<[u32]> }`, `utable: (u32, Box<[u32]>)→u32`.
- These node types intentionally do **not** implement the shared `NonTerminal` trait (whose
  `Index`/`iter` return `&NodeId`, incompatible with `u32` storage); they expose
  value-returning inherent accessors: `id()`, `headerid()`, `edge(i)->NodeId`,
  `iter()->impl Iterator<Item=NodeId>`, `len()`.
- Effect (bench `bigsum`, 2.1M-node MDD): peak RSS −28%. The `u32` ceiling (4.3×10⁹ nodes) is
  unreachable in practice. See `../bdd-bench/README.md`.

### The `bss`/`mss` wrappers

The core managers are arena-based and need `&mut manager` per operation. `bss`/`mss` wrap the
manager in `Rc<RefCell<..>>` and hand out `BddNode`/`MddNode<V>` handles holding a `Weak`
back-reference + a `NodeId`. This gives an ergonomic value-style API (operator overloading,
method chaining) and drives reference-counted gc (§5). **This is the layer to use/extend for
user-facing reliability computations.**

---

## 3. Algorithms

### 3.1 Apply (the hot path)

Binary operations (`and`/`or`/`xor` on BDD/MDD; `add`/`mul`/`min`/`max`/… on MTMDD) are a
**Shannon expansion** with hash-consing + memoization:

1. **Terminal / trivial cases** first (`f==g`, an operand is a terminal).
2. **Commutative operand ordering** (see 3.3) for commutative ops.
3. **Cache lookup** on `(op, f, g)`; return on hit.
4. **Split on the top variable**: pick the operand(s) at the highest level, cofactor each
   (BDD: `(low, high)`; MDD: the child vector), recurse per branch.
5. **`create_node(header, children)`** — hash-cons the result (collapses if all children
   equal); **cache-put** and return.

Level convention: **root has the largest level**. BDD uses `node_level(id)` returning
`Level::MAX` for terminals (terminals sit below all variables). MDD uses `DDForest::level ->
Option<Level>` (`None` for terminals; `None < Some(_)`), so the "top" is the max real level.
Lower operands are **replicated** across the higher variable's branches (quasi-reduced apply).

### 3.2 Native `ite`

`ite(f,g,h)` = "if f then g else h" is a **first-class** operation, not a composite, in all
three forests. Each is a single Shannon recursion over the top variable of f/g/h with its
**own dedicated computed table** keyed `(f,g,h)`:

- **BDD** (`bddcore/src/bdd_ops.rs`): was `or(and(f,g), and(not f,h))` (4 apply traversals);
  now 1. `ite`-heavy workloads (kofn) build ~5–12× faster.
- **boolean MDD** (`mddcore/src/mdd_ops.rs`): the k-ary analog (children vectors, `ite_cache`
  + `retain_live3`).
- **value-side MtMdd2** (`mddcore/src/mtmdd2_ops.rs::vite`): was `replace(vif(f,g), vif(!f,h))`
  (not + 2×vif + replace); now a single **cross-forest** ternary recursion — `f` is a boolean
  (mdd) node, `g`/`h`/result are value (mtmdd) nodes, sharing headers/levels. It is the
  3-operand generalization of the existing cross-forest binary `vif`.

Terminal rules (all three): `f==One→g`, `f==Zero→h`, `f==Undet→undet`, `g==h→g`.

### 3.3 Commutative operand ordering

For commutative ops (`and`/`or`/`xor`, and MTMDD `add`/`mul`/`min`/`max`), canonicalize the
operand pair (`if f > g { swap(f,g) }`) **before** the cache key, so `op(a,b)` and `op(b,a)`
share one entry (CUDD standard). ~2.5× on symmetric functions (n-queens); ~2.2–2.6× on the
MTMDD sum. `sub`/`div`/`rem` are non-commutative and left as-is.

### 3.4 k-of-n (`bss::kofn`, `mss` via ite)

A memoized threshold DP over `(k, start)`: `ite(v_start, kofn(k-1, start+1), kofn(k, start+1))`,
base cases `k==0 → one`, `k > remaining → zero`. `O(n·k)` — **not** the naive Shannon
recursion which is `O(2ⁿ)` (a fixed historical bug). Result is the canonical DD.

### 3.5 Reliability algorithms (`bss`/`mss` modules)

- **prob** (`bdd_prob`/`mdd_prob`) — memoized recursion computing `P(structure function ∈
  success set)` from per-variable probabilities. MDD `prob` treats the `Undet` terminal as
  contributing 0 (ZMDD-flavored).
- **count / extract** (`bdd_count`/`bdd_path`, `mdd_count`/`mdd_path`) — count satisfying
  assignments, or enumerate paths / minimal cut-path sets as a ZDD/ZMDD.
- **minpath / minsol** (`bdd_minsol`/`mdd_minsol`) — minimal **path** vectors of a structure
  function `φ` (its prime implicants, Rauzy-style). Returns `Option` (`None` when `φ` is not
  monotone/coherent). Note the MDD minsol representation (`Undet` terminal, skip-level =
  value 0, ZMDD-flavored) differs from MEDDLY's full reduction.
- **dual / mincut** (`bss` only, `bdd_dual`) — `dual` is the dual structure function
  `φ^D(x) = ¬φ(¬x)` (swap each node's children, complement terminals; O(size), memoized,
  monotonicity-preserving). `mincut = minpath ∘ dual` gives the minimal **cut** vectors.
  The multi-state (MDD) dual (state + value reversal) is not yet implemented.
- **ZDD set families** (`bss` only, `BssMgr` + `zdd`/`zdd_convert`) — `BssMgr` owns a `BddMgr`
  and a `ZddMgr`; `minpath`/`mincut` compute the minsol in the BDD forest, then convert
  (private `zdd_convert::to_zdd`) into a genuine `ZddManager` and return a `ZddNode`. Set
  algebra (`union`/`intersect`/`setdiff`/`product`/`divide`) comes from `bddcore::zdd_ops`.
  `ZddMgr` also builds families standalone (`empty`/`base`/`singleton`/`from_sets`, tracking
  element→header like `BddMgr::defvar`); enumeration is `zdd_path::ZddPath`.
  The `bdd_minsol::without` `(One, NonTerminal)` case must recurse the reference's zero branch
  (`without(f, g.edge(0))`) — recursing every branch fabricates non-minimal sets (fixed 0.9.1;
  the analogous `mss::mdd_minsol::{vwithout,bwithout}` fix uses the same zero-branch recursion).
- **ZMDD set families** (`mss` + `mddcore::zmdd`) — the multi-state analogue: a
  `mddcore::ZmddManager` is a zero-suppressed **multi-terminal** MDD denoting `f: R → 2^S`
  (sparse-vector families stratified by terminal label; `create_node` zero-suppresses on the
  0-edge, NOT the full-reduction "all edges equal"). `mss::MssMgr::minpath` returns the
  minimal path vectors directly as a genuine `ZmddNode` (converting the fake-ZMDD in the
  `MtMdd2Manager` via the private `mss::zmdd_convert`);
  `mddcore::zmdd_ops` provides `intersect`/`setdiff` (label-wise, partition-preserving — its
  level-mismatch arm descends the 0-edge, same principle as `bdd_minsol::without`). `union` /
  arithmetic apply / dominance / threshold / relabel are future work.
- **bmeas** (BSS) — per-variable importance measures.

### 3.6 RPN bridge (`BddMgr::rpn` / `MddMgr::rpn`)

A whitespace-separated Reverse-Polish DSL is parsed into a DD (the string interface the
`relibmss` Python package historically used). Grammar + doctests live in the rustdoc. The
parser treats an unknown token as a variable name — a known footgun (a var named `&`/`min`
etc. misparses); prefer the node API for new code.

---

## 4. Cache strategy: `common::ComputeCache`

The operation cache is only a **hint** — a miss costs a recomputation, never a wrong answer.
So instead of a growing `HashMap` (probe + periodic rehash per `put`, which dominates apply on
many-tiny-op workloads), it is a **direct-mapped, lossy** table (CUDD-style):

- A fixed power-of-two `Vec<[u32;4]>` of slots `[k0, k1, k2, val]`, indexed by
  `hash(key) & mask`, **overwritten on collision**. `get` = one array load + key compare;
  `put` = one array store.
- `val == u32::MAX` (`EMPTY`) marks an empty slot; node ids never reach `u32::MAX`.
- Grows (doubling + rehash) while the load factor stays under 3/4, up to a `2^24` ceiling;
  beyond that it stays lossy.
- **Keys are u32-narrowed by the manager**: `k0` is usually an op code (`Operation::code()`),
  `k1`/`k2` are operand node ids. The ternary `ite` caches instead use `(f,g,h)` as
  `(k0,k1,k2)` — all three are node ids.

### GC integration: `retain_live` vs `retain_live3`

On gc, entries touching a reclaimed slot must be invalidated (an id may be reused for a
different node after gc):

- `retain_live(&live)` — for op-keyed caches: checks `k1`, `k2`, `val` against one `live[]`
  array (`k0` is an op code, not a node id).
- `retain_live3(&live)` — for ternary `ite`/`vite` caches: checks **all** of `k0,k1,k2,val`.

### Where the caches live

- BDD: `cache` (and/or/xor/not) + `ite_cache`. Both **retained** on gc.
- boolean MDD (`MddManager`): `cache` + `ite_cache`. Retained.
- value MTMDD (`MtMddManager`): `cache`. Retained.
- **`MtMdd2Manager`** composes the two sub-managers and adds **three cross-forest caches**:
  `bcache`/`vcache` (comparisons `veq`/`vlt`) and `vite_cache` (value ite). These are
  **flushed (`clear`) on gc**, not retained — their key words mix the bool and value arenas
  (e.g. `vite_cache`: `k0`=bool id, `k1/k2/val`=value ids), which a single `live[]` array
  can't validate. This is the one asymmetry vs BSS gc (§5).

Same direct-mapped table is used by both engines (moved into `common` so BDD and MDD share it).

---

## 5. Garbage collection

Two layers.

### 5.1 Wrapper strategy (identical in `bss` and `mss`)

- **Reference-counted roots**: every live `BddNode`/`MddNode` pins its node in a shared
  `GcState.roots` map (+1 on clone/new, −1 on drop, removed at 0). The key set is exactly the
  external roots. (`bss` keys by `NodeId`; `mss` keys by the tagged `Node`, since the two
  sub-forests have independent id spaces.)
- **Adaptive threshold** (`maybe_gc`): fires when `live_node_count() >= threshold`, runs
  `manager.gc(roots)`, then re-arms `threshold = 2 × survivors` (floor `GC_FLOOR = 1<<16 =
  65536`). Appel-style doubling amortizes gc cost; small builds never collect. Tunable via
  `set_gc_threshold`. Fired only at wrapper-op boundaries where no manager borrow is held.

### 5.2 Core mark-and-sweep (per manager)

- Mark everything reachable from `roots` + terminals; reclaim the rest onto a `freelist`
  (**non-compacting** — surviving `NodeId`s stay valid, so callers keep their handles).
- Drop dead unique-table entries; **retain** live cache entries (`retain_live` /
  `retain_live3`).
- `MtMddManager::gc` additionally reclaims **unreferenced value terminals** (dropped from
  `vtable`).
- `MtMdd2Manager::gc` **partitions roots** into value/bool by tag, **clears its three
  cross-forest caches**, then runs `mtmdd.gc(vroots)` + `mdd.gc(broots)` independently.

Rule for callers: gc collects everything unreachable from `roots`, so pass every node you
still intend to use (the wrapper does this automatically via pinned handles). This is CUDD's
"reference what you keep" contract.

---

## 6. Public API map

### BSS — `bss::{BddMgr, BddNode}`

| kind | methods |
|---|---|
| manager lifecycle | `new`, `defvar`, `get_varorder`, `set_gc_threshold`, `live_node_count`, `size`, `gc`, `clear_cache` |
| build | `zero`, `one`, `create_node`, `rpn`, `and(&[..])`, `or(&[..])`, `kofn(k, &[..])` |
| node ops | `and`, `or`, `xor`, `not`, `ite`, `eq` |
| analysis | `prob`, `bmeas`, `dual` (BddNode); `minpath`/`mincut` on `BssMgr` (→ `ZddNode`); `bdd_count`/`bdd_extract`, `size` |
| introspection | `get_id`, `get_header`, `get_level`, `get_label`, `get_children`, `is_zero/one/undet`, `dot` |
| ZDD set family (`BssMgr` owns `BddMgr`+`ZddMgr`; `ZddNode`) | `minpath`/`mincut` (`BssMgr`); `union`, `intersect`, `setdiff`, `product`, `divide`, `count`, `extract`, `dot`, `size` (`ZddNode`) |

### MSS — `mss::{MddMgr<V>, MddNode<V>}` (`V: MddValue`, e.g. `i64`)

| kind | methods |
|---|---|
| manager lifecycle | `new`, `defvar(label, range)`, `get_varorder`, `set_gc_threshold`, `live_node_count`, `size`, `gc`, `clear_cache` |
| build | `boolean`, `value`, `undet_boolean`, `undet_value`, `create_node`, `rpn`, `and`/`or`/`min`/`max` (n-ary) |
| arithmetic (value) | `add`, `sub`, `mul`, `div`, `min`, `max` |
| comparison (value→bool) | `eq`, `ne`, `lt`, `le`, `gt`, `ge` |
| logic (bool) | `and`, `or`, `xor`, `not`, `ite` |
| analysis | `prob`, `mdd_count`/`mdd_extract`, `size` |
| introspection | `get_id`, `get_id2`, `get_node`, `get_header`, `get_level`, `get_label`, `get_children`, `is_boolean/value/zero/one/undet`, `value`, `dot` |
| ZMDD set family (`MssMgr` owns `MddMgr`+`ZmddMgr`; `ZmddNode`) | `minpath` (`MssMgr`); `intersect`, `setdiff`, `count`, `extract`, `size` (`ZmddNode`) |

Two API styles coexist (see `README.md`): an older `Context`-centric style and the current
node-centric style (`mgr.getbdd(top).prob(...)` at the Python layer; `node.method()` here).
Prefer node-centric for new code.

---

## 7. MTMDD2: the two-forest design

`MtMdd2Manager<V>` **composes** a boolean `MddManager` (Zero/One/Undet terminals) and a
value `MtMddManager<V>` (integer terminals), tagging each node `Node::Bool(NodeId)` or
`Node::Value(NodeId)`.

- **Shared variable order**: `create_header` creates the header in **both** sub-forests and
  `assert_eq!`s their `HeaderId`s, so a given variable has the same level/edge-count/id in
  both arenas — the invariant that makes cross-forest cofactoring well-defined.
- **Operation routing**: arithmetic → value forest; boolean logic → bool forest; comparisons
  (`veq`/`vlt`) and conditionals (`vif`/`vite`) are **cross-forest** (bool + value operands →
  result in the appropriate forest).
- **Why two forests**: each range gets its correct canonical form (booleans reduce as boolean
  MDDs, values as MT-MDDs; comparison results land naturally in the bool forest). This mirrors
  MEDDLY's forest-per-range layout — see `../bdd-bench/README.md` for the comparison.
- **Maintainability note**: the two id spaces are both plain `usize`/`u32`, distinguished only
  by the `Node` tag and convention (`f` is a bool id, `g/h` are value ids in the cross-forest
  helpers). This is the design's main soft spot; the `Node` enum tags at the boundary but the
  raw `vif`/`vite`/`veq`/`vlt` helpers rely on convention. A `BoolId`/`ValueId` newtype could
  harden it but is invasive for the small risk surface (~4 functions); not currently done.

---

## 8. Conventions for contributors

- **New operations** go in that DD type's `_ops.rs`, dispatched through its operation enum +
  cache; mirror the existing `and`/`or`/`apply` (Shannon recursion + hash-cons + memoize)
  rather than recursing over nodes directly, so canonicity and caching are preserved. For a
  commutative op, add the operand-ordering swap. For a ternary op, use a dedicated
  `ComputeCache` + `retain_live3` in gc.
- **After editing a `*/src/*.rs`** used by `relibmss`, remember that package consumes the
  **published** crates, not this checkout (repoint deps to a local path only to experiment;
  don't commit that).
- **Graphviz** stays in `_dot.rs` behind the `Dot` trait. **New public items** get a
  `prelude` re-export.
- **Node storage stays u32**, API stays `NodeId=usize`, casts confined to node/table helpers
  (mirror `bddcore/src/nodes.rs`).
- **Release**: bump `[workspace.package] version` (all five lockstep) + path-dep versions, add
  a per-crate CHANGELOG entry, `cargo package --list` to check shipped files, `cargo doc` +
  `cargo test --doc`, publish in dependency order (`relib-common` → `-bdd`/`-mdd` →
  `-bss`/`-mss`). Full runbook in `CLAUDE.md`.

---

## 9. Testing & benchmarks

- **Tests**: `cargo test` (21 suites: inline `#[cfg(test)]` + `tests/` integration). Traps:
  `-p` takes the crates.io name (`relib-bdd`, not `bddcore`); `--test <file>` is required to
  run one integration file (without it the arg is a name filter → silent no-op).
- **Doctests**: `cargo test --doc` (the rpn grammar examples).
- **Benchmarks**: `../bdd-bench/` (separate, git-untracked) — native harnesses comparing the
  BDD engine vs CUDD and the MDD engine vs MEDDLY, plus the perf-change measurements
  (native-ite, commutative ordering, shared cache, u32 storage). `benches/` inside this repo
  is legacy and does not compile.

---

## 10. See also

- `CLAUDE.md` (repo root, **git-untracked**) — commands, publishing runbook, session log.
- Per-crate rustdoc (`cargo doc`) / docs.rs — API reference + rpn grammar.
- Root & per-crate `README.md`, per-crate `CHANGELOG.md`, root `TODO.md`.
- `../bdd-bench/README.md` — performance comparisons and the empirical basis for the perf
  design decisions referenced throughout this guide.
