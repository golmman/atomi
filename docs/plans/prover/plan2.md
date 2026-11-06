# Prover Plan 2: M1 — per-node cost

Initiative: [`prover`](initiative.md), backlog **item 2 (M1)**. Drafted
2026-11-06 in plan session 2, after [`report1.md`](report1.md). Baseline:
atomi M0 at `9aa96c9` (the committed plan1 tree).

M1 is one **declared coupled pair** (working agreement 2): the cost of
classifying a child splits into a *memory* half (table probes that miss the
cache) and a *compute* half (clone + make-move + outcome per child). The
pair is coupled because the memory half's prefetch needs every child key
before any child is classified, and the compute half produces those keys
without making the move. The halves land in separate phases (A and B), each
with its own measurement, so the report can attribute the gain.

**Gate (initiative, unchanged):** wall per 1 M child evals at or below the M0
baseline **and** the legacy front-end's, same session; a small reference win
proved within a stated cap and checker-accepted; soundness fixtures green.
Plus (report1 next step 2) a 10 M fixture sweep with both checkers.

**Stop rule (initiative §Backlog, handoff §7):** if M1 does not improve
per-node cost over the legacy front-end, the per-node diagnosis is wrong.
Stop, record, and re-diagnose before M2–M3.

Prerequisite reading (summarized below; the plan is self-contained):
[`AGENTS.md`](../../../AGENTS.md), [`initiative.md`](initiative.md) (D1–D17),
[`plan1.md`](plan1.md) §Search/§Table/§Events/§Strategy,
[`report1.md`](report1.md), [`bench.md`](bench.md) reading rules.

## Why this plan exists

The plan-session probes below show where M0's time goes. On `core`, 93% of
search time is frame entry (child classification). About half of that is
memory stall: the same search with a 1 MiB table runs at 77 ms per 1 M child
evals, against 164 ms at the 192 MB label. The rest is per-child compute,
≈ 50 ns per child for clone + `do_move` + `outcome()` + key. On this host the
legacy front-end is already **faster per child eval than M0** (147 vs 172 ms
per 1 M on `core` at 10 M). So the binding gate is the legacy number, not
M0's.

## Plan-session findings (read-only probes, 2026-11-06)

All probes ran on this repo's host (`ae54d05b36f8`, Ryzen 9 5950X container,
4 vCPU, L2 512 KiB/core, L3 2 × 32 MiB, THP `madvise`). They used scratch
builds in `/tmp/plan2-probe` and changed nothing in the repo. They are sizing
evidence, **not** ledger numbers: the execution session re-measures
everything it gates on (reading rule 1). `perf` is unavailable in the
container (`perf_event_paranoid = 2`, no capability), so the cost split comes
from in-process timers and a micro-benchmark.

1. **Same-session per-node, M0 vs legacy front-end** (3 repeats, medians,
   ms per 1 M child evals). atomi was `9aa96c9`, `--memory 192 --seconds
   600`. The legacy side was the front-end alone via the legacy plan3 probe
   (`atomic_solver@5f85133:docs/plans/winprover/measurements/plan3/probe.rs`,
   sha256 `44edddd3…a1d59f`), compiled with `rustc --edition 2024 -O -C
   panic=abort -C lto=thin` against the scratch export's rlib:

   | Cell | Cap | atomi M0 | legacy front-end | M0 evals/expansion | FE evals/node |
   | --- | ---: | ---: | ---: | ---: | ---: |
   | `core` | 1 M | 164.0 | 156.0 | 32.5 | 45.5 |
   | `easy-win` | 1 M | 155.0 | 144.0 | 31.5 | 46.3 |
   | `deep-win` | 1 M | 159.0 | 147.0 | 29.7 | 35.8 |
   | `core` | 10 M | **171.9** | **146.8** | 32.0 | 44.5 |
   | `easy-win` | 10 M | 163.3 | 139.2 | 31.1 | 44.0 |
   | `deep-win` | 10 M | 159.9 | 142.0 | 30.1 | 38.4 |

   - The front-end runs with its fixed 32 MB scratch table
     (`WIN_FRONT_END_TT_MB`), static move ordering and sibling simulation on.
     Its `child_evals` counts the simulation's evaluations too (legacy
     semantics, D11), so the per-eval comparison is the initiative's
     definition, not an exact like-for-like.
   - M0 drifted from report1's 162.0 to 171.9 on `core` at 10 M in this
     session. That is the reason the gate is same-session only.
2. **Table size dominates M0's cost.** `core`, 10 M, M0, median of 3 (ms per
   1 M): 1 MiB **77.0**, 4 MiB 83.2, 16 MiB 108.3, 64 MiB 159.3, 192 MiB 163.6,
   1024 MiB 169.9. Expansions stay within 304,599–312,551, so the trajectory
   barely moves. At the legacy front-end's 32 MB, M0 is 140.8 (noisy:
   125–157).
3. **Where search time goes** (scratch copy with `Instant` timers, `core`,
   10 M): `Frame::enter` 1.60 s of 1.72 s (**93%**). The MID loop's
   value/select takes 43 ms, descent clone + `do_move` 11 ms, and table
   stores 8 ms.
4. **Child census** (scratch counters, 10 M per cell): the table holds an
   entry for only **3.3% / 2.6% / 3.5%** of child evals (`core` / `easy-win`
   / `deep-win`). 0.42–0.47% of children are terminal. The mean line length
   at an expansion is 6.2 / 8.3 / 6.7. Over 968 sampled expanded positions
   (every 997th expansion, 30,368 children, 31.4 per node), 9.9% of children
   are captures, 1.5% give check and 11.3% are double pushes.
5. **Per-child component cost** (micro-benchmark over those 968 positions,
   ns per child unless noted):

   | Component | ns |
   | --- | ---: |
   | `generate_legal` per node | 212–219 |
   | clone + `do_move` | 16 |
   | `rules::status(child)` | 23 (`populate_state` 9; + `has_legal_move` 19; `material_draw` 1.3) |
   | `rules::child_key` = `hash_after_with(Legal)` | 10.9 (clones on every double push) |
   | `hash_and_rule50_after` | 8 |
   | `child.hash_with(Legal)` on a made child | 2.6 |
   | `is_zeroing` / `blast_casualties` | 1.7 / 1.8 |
   | M0 classify equivalent, no table | 49.5 |
   | M0 classify equivalent, 192 MiB table (isolated loop) | 66 |

   - At a 1 MiB table the search runs at ≈ 2.4 µs per expansion, which
     matches the isolated compute (0.22 + 31.4 × 66 ns ≈ 2.3 µs).
   - So ≈ 85 of M0's ≈ 164 ms per 1 M at 192 MB is memory stall.
   - M0's entry is 32 B and its bucket is 128 B (two cache lines).
6. **Effect spike** (micro-benchmark only, never in the repo). The child
   key and counter come from `hash_and_rule50_after`. A double push without
   an enemy pawn attacking the passed square gets its key from an XOR with a
   per-file en passant key, derived once from the public hash API. Quiet,
   non-commoner, non-special moves get a check test from attack bitboards.
   Only captures, castling, en passant, promotions, commoner moves, checking
   quiet moves, and double pushes next to an enemy pawn are made on a board
   (**21.3%** of children).
   - Result: **26.4 ns** per child vs M0's 49.5 (no table).
   - 0 key, counter or check mismatches against clone + `do_move` on all
     ~23,900 unmade children.
   - 0 terminal children among the unmade ones in this sample.
7. **Eager allocation cost.** A mate in one (`6rk/8/8/8/8/8/8/6QK w - - 0 1`)
   takes **55.5 ms** of process wall at `--memory 192` (RSS 129 MiB) and
   412 ms at `--memory 1024`, with search `ms = 0`. This is the
   `vec![[None; 4]; n]` memset. The bench measures process wall, and
   provari's `easy-win` reference is 0.137 s.
8. **`atomic-movegen` 2.4.0 primitives** (read in the registry source):
   - `hash_and_rule50_after(m)` returns the `Always` key and the child
     counter from one classification.
   - `hash_after_with(m, Legal)` clones and makes every double push.
   - `do_move` always sets the en passant square on a double push. The
     `Legal` key drops that file key unless the child's `legal_ep_square()`
     is `Some`, which requires an enemy pawn attacking the passed square.
   - The Zobrist tables are **not public**, so the per-file en passant key
     must be derived as `hash_with(Always) ^ hash_with(Legal)` on a board
     whose en passant target cannot be captured.
   - `populate_state` + `generate_legal_with_state` + `outcome_with(state,
     moves_exist)` decide a node's own status for free during movegen.
   - `is_double_push`, `is_capture`, `blast_casualties`,
     `attacks::{attacks_from, rook_attacks, bishop_attacks, pawn_attacks,
     king_attacks}` and `Board::checkers()` are available. `Move::to_bits`
     is 16 bits, all of them used.
   - `_mm_prefetch` is still an `unsafe` call at rustc 1.99.1 (checked with a
     scratch compile).
9. **Legacy `src/search/tt/alloc.rs`** (MIT, sha256
   `0c49511e…f0fa5582`) builds the bucket array with `alloc_zeroed` +
   `Vec::from_raw_parts`. Its normative invariant: an all-zero entry is a
   valid value that reads as *not live*, and every read path filters on the
   one liveness signal. Its tests pin the zeroed bytes and that lazy and
   eager tables behave identically. The initiative marks it **copy**.
10. **provari ideas** (pinned scratch clone, read for ideas only; nothing is
    copied, and the text below is ours):
    - a move's effect (child key, counter, occupancy after, capture flag) is
      computed from the parent without building the child;
    - a child board is built only where a check or a capture needs it, and
      a stalemate after a quiet move is found when that child is expanded;
    - children are classified in passes: effects and prefetches first, then
      the reads;
    - a small one-hash bit filter of "positions possibly stored", kept in
      cache, turns most misses away before the table is touched. It is
      dropped once a quarter of its bits are set;
    - the table uses 64-byte clusters of four 16-byte slots with partial
      check bits and least-work replacement, and holds no proofs;
    - proofs live in a separate fixed-size open-addressing table with full
      keys. It keeps a non-dominated `(move, length, limit)` set per key,
      never evicts, and refuses new proofs near full. It is sized inside the
      same `--memory` label.
11. **Housekeeping found:** the `PROVENANCE.md` rows for
    `bench/run_bench.py` and the copied `bench.md` sections say `sha256 of
    source` instead of a digest. At the pin, legacy `run_bench.py` is
    `c702786c…068444` and legacy `bench.md` is `f11f0e73…25fbfb`.

## Decisions (new; the execution session folds them into `initiative.md`)

| # | Decision | Resolution |
| --- | --- | --- |
| D18 | Proof memory from M1 (revises D15) | A fixed-size **proof table** (full 64-bit keys, 16-byte slots, non-dominated `(mv, length, limit)` per key, never evicts, refuses inserts at 90% load) is the search's proof memory **and** the writer's source. It is always on, so collecting a proof cannot change the trajectory (D14 holds by construction). `--memory` covers the TT, the proof table and the filter (§Memory budget). **`--proof-memory` is removed.** The event sink stays the search's hook for M4's bounded worker. The CLI uses `NullSink`, and the event-fed `ProofStore` remains as a test-only differential source. A full proof table still fails the export closed (`store-full`) |
| D19 | Lazy terminal detection (new) | Child classification is exact for explosion, mate, material draw and the 50-move limit. A **stalemate reached by a quiet, non-checking move** is not detected at child initialization: such a child starts Open `(1, 1)`. Every node computes its own exact status at frame entry, from the movegen it runs anyway. `child_evals` keeps its D11 meaning (one per child record). Soundness rests on the node-entry status and the unchanged value algebra (§Soundness) |
| D20 | `unsafe` in M1 (AGENTS.md rule) | Three sites at most, each with a `// SAFETY:` comment and a measured justification in the report: (1) the copied `alloc_zeroed` allocator; (2) one prefetch wrapper (`_mm_prefetch`, x86-64 only, a no-op elsewhere); (3) optionally `madvise(MADV_HUGEPAGE)` via a 3-line `extern "C"` (Linux only, no `libc` crate). Each has a pre-registered keep rule (§Measurement M4). A site that fails its rule is removed |

## Scope and non-goals

**In scope:**

- **Phase A** (memory half): TT v2 (64-byte clusters, partial keys, no
  proofs), the filter, the proof table, the lookup/store routing,
  child-key prefetch, the lazy zeroed allocation (copy) and the memory
  split; the writer reads the proof table; CLI contract updates.
- **Phase B** (compute half): `Mover`/`Effect`, the multi-pass frame,
  node-entry status, and keys passed down instead of recomputed.
- Measurement and the ledger row; docs.

**Out of scope:** move classes, EF-PN, quiet grouping, any change to
starting numbers (M2); static tactics and simulation (M3); the bounded
strategy worker, `.pvr` and dropping mates from the proof table (M4);
restarts and generation-aware replacement (D5); an own board or pseudo-legal
movegen (D2: only if M1 shows it is needed); pooling the per-frame child
`Vec`s (≈ 1% here; a follow-up option); the value algebra (unchanged,
`OPEN_MAX` rule kept); the validator (unchanged); report1's missing tests,
except where §Tests touches them.

## Design

### Layout after M1

```
src/
  mover.rs          Mover (per node) + Effect (per move); no crate:: deps but rules
  table/
    mod.rs          Tables facade: split, lookup, store, prefetch (header >10 KB note if needed)
    tt.rs           TT v2: Slot, Cluster, index, replacement
    filter.rs       one-hash bit filter
    proofs.rs       proof table (ProofTable, ProofSlot)
    alloc.rs        copied + adapted from atomic_solver@5f85133…:src/search/tt/alloc.rs
  search/
    mod.rs          driver: keys passed into mid, node-entry terminal path, Output
    frame.rs        multi-pass child initialization
    value.rs        unchanged
  rules.rs          + status_with, ep_key(file), legal_child_key
  strategy/         write.rs reads a ProofSource; store.rs kept (test-only use)
```

`src/table.rs` becomes `src/table/`. The layer rules in `lib.rs` become:
`search` depends on `events`, `mover`, `rules`, `table` and `value`;
`strategy` depends on `table::proofs` (read only) and `rules`; `validate`
stays independent.

### Memory budget (`table::split`)

`--memory MB` is the total budget for the TT, the proof table and the filter
(`bench.md`'s "from M1 every in-search store lives inside the label").
`split(mb) -> Budget { tt_clusters, proof_slots, filter_bits }`, with
`total = mb × 2^20` bytes:

- `proof_slots = (total / 4) / 16`;
- `filter_bits = prev_power_of_two(8 × tt_slot_estimate)` clamped to
  `[2^20, 2^25]`, where `tt_slot_estimate = (total − proof bytes) / 16`.
  The filter takes 128 KiB to 4 MiB;
- `tt_clusters = (total − proof bytes − filter bytes) / 64`, at least 1, at
  most `2^32`.

Invariant (unit-tested for every `mb` in `1..=4096`):
`64·tt_clusters + 16·proof_slots + filter_bits/8 ≤ total`. At 192 MB this
gives 48 MiB of proofs (3,145,728 slots), a 4 MiB filter and 140 MiB of TT
(2,293,760 clusters, 9,175,040 slots). The TT is no longer a power of two
(see index). RSS is bounded by the label, because pages are touched lazily.

### TT v2 (`table/tt.rs`)

- `#[repr(C)] Slot { check: u32, pn: u32, dn: u32, meta: u32 }` (16 B), with
  `#[repr(C, align(64))] Cluster([Slot; 4])` (64 B, one line).
  `const` asserts pin both sizes and the alignment.
- `check = max(1, (key >> 32) as u32)`, and `check == 0` marks an empty slot,
  so an all-zero cluster is empty (the alloc invariant). The cluster index
  is `((key as u32 as u64) × n_clusters) >> 32`.
- The TT holds **only Open values and path-independent refutations**:
  - Open: `pn ≥ 1`, `dn ≥ 1`, both `≤ OPEN_MAX`;
  - Refuted: `pn = INFINITY`, `dn = 0`;
  - `meta = min(work, 2^24 − 1) << 8 | from`.

  `debug_assert!` that an Open value never has a zero number.
- Store rules (M0's minus proofs):
  - a path-dependent refutation (`cycle != ANY`) is a no-op, and an existing
    entry stays;
  - otherwise the slot with the same check is overwritten; else an empty
    slot; else the lowest-`work` slot (the first on ties).
- `usable` keeps M0's rule for refutations (`counter ≥ from`). Open entries
  are always usable.
- **Partial keys are sound here.** A check collision can only hand a node a
  wrong Open pair or a wrong path-independent refutation. In attacker-only
  semantics both affect only completeness (a lost win), never soundness,
  because proofs need proved children and proofs never live in the TT.
  Write this argument into the module doc.

### Filter (`table/filter.rs`)

- A bit array of `filter_bits` (a power of two). The bit index is
  `check & (bits − 1)`.
- Every TT store and every proof-table insert sets the key's bit.
- `may_hold(key)` is `true` once the filter is dropped. It is dropped
  permanently when more than a quarter of its bits are set (count bits as
  they flip). There is no rebuild in M1.
- Exact in the negative direction (no false negatives, because evicted keys
  leave their bits set). So **the filter changes no result**. A test pins
  identical counters with the filter on and off.

### Proof table (`table/proofs.rs`, D18)

- `#[repr(C)] ProofSlot { key: u64, mv: u16, length: u16, limit: u8,
  state: u8, _pad: [u8; 2] }` (16 B, `const` assert).
  - `state`: 0 = free (the all-zero slot), 1 = live, 2 = replaced
    (tombstone). Liveness is `state == 1` only. The key is never a
    liveness signal, so key 0 is a valid position.
  - `mv` is `Move::to_bits()`, with 0 (= `Move::NONE`) at defender
    proofs;
  - `limit` is `0..=99`.
- Open addressing with linear probing from `((key as u128 × n) >> 64)`,
  wrapping at the end. A **run** is the slots from the start up to the first
  free one.
- `insert(key, entry)` walks the key's run:
  - a live entry with `length ≤ new.length && limit ≥ new.limit` makes the
    insert a no-op;
  - live entries the new one dominates (`length ≥`, `limit ≤`) become
    tombstones;
  - the first tombstone in the run is reused; otherwise the run-ending free
    slot is taken while `used < 9/10 · slots`;
  - otherwise the insert is refused and `refused = true` is set.

  Exactly the D15 domination rule, so the set per key equals the event
  store's for the same insert sequence.
- `get(key, counter)`: the shortest live entry for `key` with
  `limit ≥ counter`. Ties go to the higher limit, then the lower `mv` bits
  (order-independent).
- It holds attacker proofs (`mv` set) **and** defender proofs (`mv` none),
  because the TT no longer can. Terminal wins (`length 0`) are never
  inserted; they are recomputed at classification. `debug_assert!` that
  `length ≥ 1` on insert.
- Diagnostics: `live()`, `used()`, `refused()`, `slots()`.

### Lookup and store routing (`table/mod.rs`, the `Tables` facade)

```rust
pub struct Tables { tt: Tt, proofs: ProofTable, filter: Filter }
impl Tables {
    pub fn new(memory_mb: usize, filter: bool) -> Tables;
    pub fn filtering(&self) -> bool;
    pub fn prefetch_filter(&self, key: u64);
    pub fn may_hold(&self, key: u64) -> bool;
    pub fn prefetch(&self, key: u64);              // TT cluster + proof run start
    pub fn lookup(&self, key: u64, counter: u16) -> Option<Value>;
    pub fn store(&mut self, key: u64, value: &Value, work: u32);
    pub fn proofs(&self) -> &ProofTable;
    pub fn into_proofs(self) -> ProofTable;
}
```

- `lookup` first tries `proofs.get(key, counter)` → `Value::Proved { length,
  limit, mv }`. Next a TT entry, if usable. `None` means fresh `(1, 1)`.
  A usable proof wins over a usable refutation (a proof is exact; a
  refutation may be conservative).
- `store` routes by value:
  - `Proved` goes to `proofs.insert`, **and** the key's TT slot is cleared
    if present. So after a proof the TT only holds a value from a later
    re-search, which mirrors M0, where the proof replaced the entry in
    place;
  - an Open value or a path-independent `Refuted` goes to `tt.store`;
  - both routes set the filter bit;
  - a path-dependent refutation does nothing.
- `filter = false` exists for the filter-neutrality test only. The CLI
  always passes `true`. Expose it as `Limits.filter: bool`, documented as a
  test hook.

### Lazy zeroed allocation (`table/alloc.rs`, copy)

- Copy legacy `src/search/tt/alloc.rs` (header line
  `from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:src/search/tt/alloc.rs`,
  plus a `PROVENANCE.md` row with sha256
  `0c49511e8806121e90565960982973679b6f7fdb6e0ac08c20f1b9efd0fa5582`,
  `adapted`).
- Adaptation:
  - generic `pub(crate) fn zeroed<T: ZeroValid>(n: usize) -> Vec<T>` over a
    crate-private `unsafe trait ZeroValid: Copy`, implemented for
    `Cluster`, `ProofSlot` and `u64`;
  - the invariant text is rewritten for our liveness signals (`check == 0`,
    `state != 1`, bit 0);
  - the legacy `TtEntry`/generation tests are replaced by our own: zeroed
    values read as empty or free for each type, `n = 0`, lengths, and a
    lazy-vs-eager equivalence over a store/lookup sequence.
- The `--memory` allocation no longer counts as search time in a meaningful
  way. Page faults now happen inside `ms`, which is honest, and process wall
  drops by the memset.

### Prefetch and huge pages (D20)

- `table::prefetch<T>(r: &T)`: x86-64 `_mm_prefetch::<_MM_HINT_T0>` in one
  `unsafe` block (SAFETY: a hint, cannot fault, reads nothing observable).
  It is a no-op on other targets. Used for filter words, TT clusters and the
  proof run's first slot.
- Huge pages are **optional** (Phase C A/B):
  - after `alloc_zeroed`, advise the 2 MiB-aligned interior of each table
    with `madvise(…, MADV_HUGEPAGE = 14)` (Linux x86-64/aarch64 `cfg`;
    the return value is ignored);
  - kept only by its §Measurement M4 rule; otherwise not landed.

### `Mover` / `Effect` (`src/mover.rs`, Phase B)

`Mover::new(board)` is per expanded node. It holds `us`, `them`, the enemy
commoner square (`Option`), whether the two commoners touch, `occupied`, our
rook-or-queen and bishop-or-queen sets, and enemy pawns.
`Mover::effect(m) -> Effect` is per legal move:

```rust
pub struct Effect {
    pub key: u64,        // Legal-identity child key (valid unless needs_board)
    pub counter: u16,    // child's 50-move counter (rule50_after)
    pub zeroing: bool,   // counter == 0
    pub needs_board: bool,
}
```

- `(h, counter) = board.hash_and_rule50_after(m)`.
- `needs_board` is set when any of these holds:
  - `m` is a promotion, en passant or castling;
  - `board.is_capture(m)`;
  - the moving piece is a commoner;
  - the move gives check;
  - it is a double push and an enemy pawn attacks the passed square.
- The check test for a quiet, non-commoner, non-special move:
  - no check when the enemy commoner is missing or the commoners touch
    (the move cannot change either fact);
  - else `occ' = occupied ^ from | to`;
  - **direct**: `attacks_from(pt, us, to, occ')` hits the enemy commoner;
  - **discovered**: `rook_attacks(ksq, occ') & rq & !from` or
    `bishop_attacks(ksq, occ') & bq & !from` is non-empty.

  This is exact for that move class (spike: 0 mismatches), and the tests pin
  it against `child.checkers()`.
- Key for a non-board double push: `h ^ rules::ep_key(file_of(passed))`.
  Any other non-board move: `h`. A board child's key is
  `child.hash_with(Legal)` after the make.
- `rules::ep_key(file)`: eight `u64`s in a `std::sync::OnceLock`. Each is
  derived as `hash_with(Always) ^ hash_with(Legal)` on a constructed board
  with an uncapturable en passant target on that file. Black is to move,
  with a white pawn on rank 4, the target on rank 3, and only commoners
  otherwise, e.g. `4k3/8/8/8/P7/8/8/4K3 b - a3 0 1` with the pawn shifted
  per file. Tests: all eight are non-zero and distinct, and they equal
  `hash_after_with(m, Legal) ^ hash_after(m)` on every uncapturable double
  push in the playout set.
- **Upstream feedback** (D2 pattern, recorded in the report, not blocking):
  `hash_after_with(m, Legal)` could skip its clone when no enemy pawn
  attacks the passed square, or expose the per-file en passant key. Either
  would retire `ep_key`.

### Frame v2 (`search/frame.rs`)

`Frame::enter(board, key, attacker, line, tables, expansions) -> Entered`,
with `enum Entered { Terminal(Value), Open(Frame, u64 /* child evals */) }`.
`Frame` keeps `children: Vec<Child>` (the `value.rs` type, unchanged) and
adds a parallel `keys: Vec<u64>`.

1. **Node status.** `populate_state` → `generate_legal_with_state` →
   `rules::status_with(board, attacker, &state, !moves.is_empty())` (new; =
   `outcome_with` + `material_draw`, the same precedence as `status`). A
   non-`Open` status returns `Entered::Terminal`:
   - `Failed { from }` → `Refuted { cycle: ANY, from }`;
   - `Won` → `Proved { length: 0, limit: 99, mv: None }`. Unreachable
     under D19, because a mate needs a check and every checking move is
     built at the parent. `debug_assert!` it, and handle it anyway.
2. **Pass 1 (effects).** For each legal move: `Effect`, then
   `child counter = effect.counter`, `zeroing = effect.zeroing`.
   - If `needs_board`: build the child (clone + `do_move`) and take its key
     from `hash_with(Legal)`. Keep the classification result from
     `rules::status(&child, attacker)` for pass 3; at ~21% of children it is
     cheaper than building twice. A per-frame scratch `Vec<Status>` is fine.
   - Record the key. If filtering, prefetch the filter word; otherwise
     prefetch the TT cluster and the proof run.
3. **Pass 2 (filter).** Only while filtering: for each child, `seen =
   may_hold(key)`; if `seen`, prefetch the TT cluster and the proof run.
4. **Pass 3 (classify)**, M0's order:
   - **terminal**:
     - a board child's status `Won` → proved terminal; `Failed { from }`
       → `Refuted { ANY, from }`;
     - for a non-board child, `counter ≥ 100` → `Refuted { ANY, from: 100 }`.
       It gives no check, so the clock decides, the same as `status`;
     - otherwise Open;
   - **repetition**: the key on `line` at index `i` → `Refuted { cycle: i,
     from: 0 }`;
   - **lookup**: if `seen` (or not filtering), `tables.lookup(key,
     counter)`;
   - **fresh**: `Open { 1, 1 }`.
5. `child_evals += moves.len()`, as in M0.

The descent passes `frame.keys[best]` into the child's `mid`, so `rules::key`
runs only at the root. `debug_assert_eq!(key, rules::key(board))` stays in
`mid`.

### Search driver (`search/mod.rs`)

- `mid(board, key, ply, tpn, tdn)`. On `Entered::Terminal(v)`:
  `expansions += 1` (it ran movegen), store `v` through `Tables::store`
  (a refutation is TT-stored; a terminal proof is not inserted), pop, and
  return `v`. The `--evals` and deadline checks are unchanged.
- Frame exit: `tables.store(key, &v, work)` and the attacker-proof event,
  as in M0.
- `prove` returns `Output<S> { result: SearchResult, proofs: ProofTable,
  sink: S }`. Tests and `main.rs` are updated. `SearchResult` gains
  `proof_entries: u64` (live slots) and `proof_full: bool` (refused ≥ 1).
- Determinism: no randomness, no hash-map iteration. Pass order is movegen
  order.

### Writer, events, CLI

- `strategy::ProofSource` trait:
  `fn entries(&self, key: u64, out: &mut Vec<StoreEntry>)` (attacker
  entries, any order) and `fn is_full(&self) -> bool`. It is implemented for
  `ProofTable` (skips `mv == 0`) and for the existing `ProofStore`.
  `write_proof(root, &impl ProofSource, path)` is otherwise unchanged:
  `choose()` is already order-independent (it tie-breaks on move bits).
- The CLI always runs with `NullSink`, and the writer reads
  `Output.proofs`. Remove `--proof-memory` from `cli.rs`, `main.rs`'s help
  and `AGENTS.md`.
- The output contract appends two fields, `"proof_entries":N` and
  `"proof_full":bool`; the order is documented in `cli.rs`. `run_bench.py`,
  `run_pernode.py` and `veto.py` parse with `json.loads` and ignore unknown
  fields (verify).
- `ProofStore` and `events` stay. A fast test feeds `ProofStore` from the
  event stream of the same search and checks that both sources write
  **byte-identical** files on the 8 veto rows.

### Soundness argument (write it into `search/mod.rs` docs)

- A proof needs only proved children, exact terminal wins (explosion via a
  made child's `status`, or a mate via a made child, since every checking
  move is made) and usable proof-table entries (full-key match, `counter ≤
  limit`). None of these changed in meaning.
- What D19 and partial TT keys can cause is a missed refutation, a wrong
  refutation or a different visiting order. Those affect completeness and
  speed only.
- The node-entry status makes every expanded node's terminality exact, so
  no frame is ever built at a position that is over.
- The validator and `provari check` remain the veto on every claimed win.

### Measurement drivers

- `bench/run_pernode.py` (extend; keep the old flags working):
  - `--atomi NAME=PATH`, repeatable (a bare `PATH` means `atomi`);
  - `--legacy-probe PATH` + `--legacy-commit SHA` (runs `probe FEN
    budget` and parses `{"evals","nodes","proved","depth","wall_s"}`);
  - `--memory` takes a comma list (atomi sides only).
  - Per (case, cap) it interleaves sides inside each repeat (repeat 1: all
    sides; repeat 2: …), one process at a time.
  - Output: rows, medians per (side, case, cap, memory) of ms per 1 M child
    evals (atomi: `ms / child_evals × 1e6`; probe: `wall_s / evals × 1e9`),
    peak RSS (atomi), evals per expansion, and a `gate` block (`core`, the
    largest cap, 192 MB: `m1`, `m0`, `legacy-fe`, `pass_m0`, `pass_legacy`).
- `docs/plans/prover/measurements/plan2/run_sweep.py` (new, stdlib):
  - runs every `wins.tsv` and `never_prove.tsv` row with `atomi prove FEN
    --evals 10000000 --memory 192 --seconds 600 --proof-out <tmp>` for
    `--atomi` and, optionally, `--baseline-atomi`;
  - runs `atomi validate` and `provari check` on every proof written;
  - writes `sweep.json`: per row and side, the exit, `child_evals`, length,
    `proof_length`, `proof`, both verdicts, `positions`/`won_positions`
    and `proof_entries`;
  - proof files go to a temporary directory outside the repo and are never
    committed.

## Tests

Fast tier (`make test`; keep the total ≈ 15 s; playouts are seeded and
bounded):

- **`mover` differential.** An own `xorshift64*` with fixed seeds drives
  random legal playouts (≤ 120 plies) from the 3 bench FENs, the 8 veto FENs,
  and fixtures that exercise castling (both colours, both wings, rights lost
  by a blast), en passant (capturable and not, pinned, blast-adjacent) and
  promotions. Aim for ≥ 5,000 positions. For every legal move:
  - `Effect.counter`/`zeroing` equal the made child's;
  - for non-board moves, `key == child.hash_with(Legal)` and the predicted
    check equals `!child.checkers().is_empty()`;
  - for board moves, the made key equals `rules::child_key`.
- **`ep_key`**: 8 non-zero, distinct keys, plus the double-push identity
  above.
- **Classifier vs M0.** Over the same playouts, the frame's pass-3 value
  (with an empty `Tables`) equals the M0 rule (make + `status` + repetition
  + fresh). The **only** allowed difference is a child that `status` calls
  `Failed` with no legal move and empty `checkers()` (a stalemate) after a
  non-board move; the frame gives that child `Open(1,1)`. Count those and
  print the count.
- **`status_with` ≡ `status`** on every playout position.
- **Lazy stalemate end to end.** Construct a FEN in which one attacker
  quiet, non-checking move stalemates the defender. Verify it with `atomi
  moves` and `rules::status`, and record the construction in the test. The
  frame gives that child `(1,1)`; a `mid` descent returns `Refuted { ANY,
  0 }`; the root outcome equals M0's (`9aa96c9`) on that FEN.
- **TT v2:**
  - sizes and alignment;
  - an all-zero cluster is empty;
  - check is never 0;
  - store/lookup round trip;
  - path-dependent refutation no-op;
  - least-work replacement;
  - `usable` rules;
  - the index stays in range for non-power-of-two counts (property over
    random keys).
- **Filter:**
  - no false negatives over 100 k random inserted keys;
  - dropping at a quarter fill;
  - `may_hold` is `true` after the drop.
- **Proof table:**
  - port `store.rs`'s domination tests;
  - `get` returns the shortest usable entry;
  - tombstone reuse;
  - wrap-around runs;
  - refusal at 90% sets `refused`;
  - a zero slot is free;
  - `key == 0` works;
  - 16-byte slot.
- **`split`**: the invariant for `mb` in `1..=4096`, and the 192 MB numbers
  above.
- **`alloc`**: the zeroed value of each `ZeroValid` type reads as empty;
  `n = 0`; lazy vs eager tables give identical lookups after the same store
  sequence.
- **Search:**
  - determinism (repeat runs identical);
  - CLI with vs without `--proof-out` identical counters (replaces the
    sink-independence test; the `VecSink` event test stays);
  - **filter on vs off identical** `child_evals`/`expansions`/exit/length
    on the mate-in-2, `control` and the pawn-ending regression FEN at
    `--evals 2000000`;
  - **writer differential** (`ProofTable` vs `ProofStore`, byte-identical
    files) on the 8 veto rows;
  - all existing search, rules, validate and corpus tests updated to the
    new API.
- **CLI:** `--proof-memory` is now `unknown option` (exit 2); the two new
  output fields come in the documented order.

Slow tier (`make test-full`): the fixture oracle at 2 M (new API), `dec10`/
`rem01` regressions, and `make veto`. The `make test-full` gate is mandatory
for both phases (search, table, proof-output changes).

## Measurement

One process at a time, same session, same host (reading rules). Scratch
builds live under `/tmp`, never in the repo.

- **M0. Binaries.**
  - atomi **m0** from `git archive 9aa96c9` into `/tmp/atomi-m0`
    (`cargo build --release`);
  - atomi **m1A**: a copy of the working tree at the Phase A gate, built in
    `/tmp/atomi-m1a`;
  - atomi **m1** (the final tree);
  - the legacy front-end probe, built from
    `/tmp/atomic_solver-5f85133/docs/plans/winprover/measurements/plan3/probe.rs`
    with the rustc command in finding 1;
  - provari v0.1.0 and v0.2.0 (`make oracle`, `make oracle-stretch`).
- **M1. Per-node (the gate).**

  ```sh
  python3 bench/run_pernode.py --atomi m0=/tmp/atomi-m0/target/release/atomi \
    --atomi m1a=/tmp/atomi-m1a/target/release/atomi --atomi m1=target/release/atomi \
    --legacy-probe /tmp/plan2-legacy-probe/probe \
    --legacy-commit 5f85133ea9d751368a682e53e354bd3fc0016d00 \
    --cases core,1eb80b92f92c,db6a741c723f --evals 1000000,10000000 \
    --memory 192,32 --repeats 3 --out docs/plans/prover/measurements/plan2/pernode.json
  ```

- **M2. Wall guard.** `easy-win` is proved to the end by m0 and m1 with
  `--seconds 600 --memory 192`, 3 repeats each: record the medians of
  process wall and search `ms`, plus `child_evals`. The guard is **m1
  process wall ≤ m0 process wall**. It catches "cheaper evals, many more of
  them".
- **M3. Fixture sweep.** `run_sweep.py --atomi target/release/atomi
  --baseline-atomi /tmp/atomi-m0/target/release/atomi --provari …` →
  `sweep.json`.
- **M4. `unsafe` keep rules (D20)**, as scratch patches of the final tree.
  Each A/B is one revert line, built in `/tmp`; the results go in `ab.json`.
  There are no feature flags in the final tree.
  - **Prefetch:** keep if the m1 median on `core`/10 M/192 is ≥ 3% faster
    than with prefetch disabled.
  - **Huge pages:** land only if ≥ 3% faster than without, on the same
    cell.
  - **Lazy allocation:** keep if the mate-in-1 process wall at `--memory
    192` drops (m0 eager: 55.5 ms in the plan session) **and** `core`/10 M
    is not > 3% slower than an eager-memset variant.
- **M5. Bench row.**

  ```sh
  python3 bench/run_bench.py \
    --provari /tmp/atomi-oracle/provari/target/release/provari \
    --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
    --provari-stretch /tmp/atomi-oracle/provari-v0.2.0/target/release/provari \
    --provari-stretch-commit 224ce85dad083b1bc8fe429a9e4349de956f5305 \
    --atomi target/release/atomi --atomi-stack 'm1=' \
    --cases core,1eb80b92f92c,db6a741c723f --repeats 3 \
    --proof-check --provari-check /tmp/atomi-oracle/provari/target/release/provari \
    --out docs/plans/prover/measurements/plan2/bench.json
  ```

  The legacy controls are not re-run (plan1's rows stand, and their factors
  are same-session ratios of their own session).
- **M6.** `measurements/plan2/README.md` (provenance table, commands) and
  `env.json`.

**Pre-registered expectation (not a gate):** the memory half brings `core`
at 10 M / 192 MB to about 95 ms per 1 M (toward the 77 ms of finding 2), and
the compute half to about 60 ms or below (finding 6). If m1 passes the gate
but lands above 90 ms, record "diagnosis partly wrong" as a finding with the
phase split.

## Gates (acceptance)

- **G1 per-node:** on `core`, 10 M, `--memory 192`, the median ms per 1 M
  child evals is `m1 ≤ m0` **and** `m1 ≤ legacy-fe`, same session (M1). All
  other cells and caps are reported. A cell that fails either comparison is
  a finding.
- **Stop rule:** `m1 > legacy-fe` on that cell → stop. Record it, write the
  report with the evidence, and propose a re-diagnosis instead of M2.
- **G2 reference wins:**
  - `control` (`dec11`) proved within `--evals 1000000` (M0: 150,775);
  - `easy-win` proved within `--evals 60000000` (M0: 30,245,030);
  - both proofs accepted by `atomi validate` **and** `provari check`, with
    `positions == won_positions`;
  - the D14 proof run for every bench cell m1 wins shows identical
    `child_evals` and `accepted`;
  - M2's wall guard holds.
- **G3 soundness:** `make test-full` green (veto: every row `accepted`,
  every control `valid: false`, the reverse control agreeing; never-prove
  and soundness fixtures never found).
- **G4 sweep (10 M, both checkers):**
  - 0 `wins.tsv` refuted;
  - 0 `never_prove.tsv` found;
  - every written proof accepted by both checkers with equal positions;
  - 0 export failures (including `store-full`).
  - An m0-proved row that m1 censors is listed with both evals. It fails
    the gate only if m1 also fails it at `2 ×` m0's `child_evals` for that
    row (re-run recorded).
- **G5:** `make test`, `make lint`, and `cargo build --release` clean. The
  licence gate passes (no new dependency).
- **G6 memory:** peak RSS ≤ label + 16 MiB at 192 and 32 (`pernode.json`).
- **G7 determinism:** repeat runs identical; with/without `--proof-out`
  identical; filter on/off identical.
- **G8 provenance:** the `alloc.rs` header line and `PROVENANCE.md` row;
  the two placeholder digests (finding 11) filled in.
- **No goal claim:** the ledger row is reported as measured. The geomean vs
  provari is not M1's gate.

## Tasks

The tasks run in phases, and each phase ends with its gate green. **Split
rule:** stop only at a phase boundary. Phase A alone is a valid landing:
Phase B then moves to plan3, and the report records A's per-node number
against m0 and legacy-fe.

**Phase A — memory half**

- A1. `table/alloc.rs` (copy + adapt, provenance), `table/tt.rs`,
  `table/filter.rs`, `table/proofs.rs`, `table/mod.rs` (`split`,
  `Tables`, prefetch), with unit tests.
- A2. Search on `Tables`: lookup/store routing, the M0 single-pass frame
  with a key-and-prefetch pre-pass (it uses `rules::child_key`; Effect
  comes in B), `Output`, `Limits.filter`, `proof_entries`/`proof_full`.
- A3. `ProofSource`; the writer over `ProofTable`; CLI (`--proof-memory`
  removed, new fields); `main.rs`; the writer-differential and filter-
  neutrality tests; all tests on the new API.
- A4. Record m0 vs m1A counters on the 8 veto rows and the fast-tier FENs.
  They are expected equal on these small rows. Every difference must be
  traced to a named cause (a proof kept that M0's TT overwrote, a cluster
  eviction) with a temporary, reverted debug counter, before Phase B starts.
- A5. Copy the tree to `/tmp/atomi-m1a` and build it (M0's m1A binary).
- Gate: `make test`, `make test-full`, `make lint`.

**Phase B — compute half**

- B1. `rules::{status_with, ep_key}`, `src/mover.rs`, with the differential
  tests.
- B2. Frame v2 (node status, passes 1–3, `Entered`), keys passed into `mid`,
  the node-entry terminal path; the classifier-vs-M0 and lazy-stalemate
  tests.
- B3. Docs in code: the soundness argument, D19 and partial-key notes.
- Gate: `make test`, `make test-full`, `make lint`.

**Phase C — measurement**

- C1. Extend `run_pernode.py`; write `run_sweep.py`; parse-check both
  (`ast.parse`).
- C2. M0 (binaries) → M4 (A/Bs, then land or remove each `unsafe` site
  and re-run `make test-full` if anything changed) → M1 → M2 → M3 → M5 →
  M6.
- Gate: G1–G7 evaluated and written down. A G1 miss follows the stop rule.

**Phase D — close-out**

- D1. `PROVENANCE.md` (the `alloc.rs` row; finding 11 digests).
- D2. `AGENTS.md`:
  - architecture (`table/`, `mover.rs`, proof table, `--memory` scope);
  - commands (no `--proof-memory`);
  - gotchas: lazy stalemate; proofs never in the TT; the filter changes no
    result; the per-file en passant key is derived.
- D3. `bench.md`: the plan2 row and summary; the successor protocol note
  now reads "`--memory` covers the TT, the proof table and the filter".
- D4. `initiative.md`: item 2 status, D18–D20, history, M2's reference
  numbers (m1 per-node, `easy-win` evals/wall); `docs/plans/README.md`
  index row.
- D5. `report2.md` (final task).

## Validation

```sh
make test
make lint
make oracle && make oracle-stretch     # once per host
make test-full                          # includes make veto
cargo build --release
python3 - <<'PY'
import ast, pathlib
for p in [*pathlib.Path("bench").glob("*.py"), *pathlib.Path("scripts").glob("*.py"),
          *pathlib.Path("docs/plans/prover/measurements/plan2").glob("*.py")]:
    ast.parse(p.read_text())
PY
sha256sum bench/corpus.json             # 9cd0169a…47d0e
git status --porcelain && git add --dry-run .
```

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| The `Effect` check test misses a check and a mate goes undetected at the parent | Only completeness and speed are at stake (node-entry status catches it, and `debug_assert!` flags an entry-`Won`). The differential test pins the prediction against `checkers()` over ≥ 5,000 positions, including castling, en passant and promotion fixtures |
| An `Effect` key differs from the made child's | Fails the differential test (exact equality). In search, a wrong key breaks transposition reuse, and it can hide a repetition. Then the search proves a line that repeats a position, and the checker rejects that proof (cycle). That is a veto, not a claimed win, but it would make the gate red |
| The per-file en passant key derivation breaks on an `atomic-movegen` update | `ep_key` tests compare against `hash_after_with(Legal)`; the lockfile pins 2.4.0; upstream feedback recorded |
| The proof table fills on long runs (no eviction) | 1/4 of the label; `proof_full` and `proof_entries` reported in every run and in the sweep; export fails closed; scale is M4's job |
| The trajectory changes (cluster layout, kept proofs, lazy stalemates) move fixture outcomes | G4's regression list with the `2 ×` re-run rule; A4 traces Phase A differences on small rows |
| Cheaper evals but more of them (the metric games itself) | The M2 wall guard; the bench row reports evals and wall |
| Prefetch or huge pages do nothing in this container | Pre-registered keep rules; an `unsafe` site without a measured win is removed (D20) |
| A partial-key collision produces a wrong refutation of a real win | ≈ 53 effective key bits at 192 MB; the completeness-only argument is documented; the sweep's 0-refuted gate would surface it |
| The legacy front-end comparison is not like-for-like (32 MB table, simulation, its eval semantics) | The initiative's definition is used as is; m1 is also reported at `--memory 32`; the caveat goes in the report |
| Measurement noise on a 4-vCPU container (finding 2: 125–157 ms spread at 32 MB) | Interleaved sides per repeat; 3 repeats; medians; one process at a time |
| GPL leakage from reading provari | Ideas only (finding 10 is in our words); no code, comments or constants copied; the module docs are written fresh |

## Report requirements (`report2.md`)

The legacy report shape: headline, provenance, exact commands, results,
gate table (G1–G8), verdict, not measured, problems and findings, missing
tests, next steps, delivered files. Specifically:

- the per-node table (m0 / m1A / m1 / legacy-fe × 3 cells × 2 caps × 192/32),
  the **phase attribution** (A vs B), and the comparison with the
  pre-registered expectation;
- the M2 wall guard and the `easy-win` evals (m0 vs m1);
- the A/B results and what happened to each `unsafe` site;
- the sweep counts (proved / censored / refuted, never-prove found, export
  failures, `proof_full`), the regression list if any, and the lazy-
  stalemate count from the classifier test;
- the Phase A counter comparison (A4) with every traced cause;
- the bench row with D14 results; **no goal claim**;
- the upstream feedback item (`ep_key`, finding 8);
- a closure block with the kickoff prompt for plan3 (M2, search guidance),
  or for Phase B if split.

## Deliverables

- **Engine:** `src/table/{mod, tt, filter, proofs, alloc}.rs`
  (`src/table.rs` removed), `src/mover.rs`, updated
  `src/search/{mod, frame}.rs`, `src/rules.rs`, `src/strategy/{mod,
  write}.rs`, `src/cli.rs`, `src/main.rs`, `src/lib.rs`.
- **Tests:** updated `tests/*.rs`, new fast tests as above.
- **Drivers:** extended `bench/run_pernode.py`, new
  `docs/plans/prover/measurements/plan2/run_sweep.py`, `bench/README.md`.
- **Measurements:** `docs/plans/prover/measurements/plan2/{README.md,
  env.json, pernode.json, pernode.csv, bench.json, sweep.json, ab.json}`.
- **Docs:** `PROVENANCE.md`, `AGENTS.md`, `bench.md`, `initiative.md`,
  `docs/plans/README.md`, `report2.md`.
- **Scratch, never committed:** `/tmp/atomi-m0`, `/tmp/atomi-m1a`,
  `/tmp/plan2-legacy-probe`, the A/B builds, the sweep's proof files.
