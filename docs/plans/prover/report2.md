# Prover Report 2: M1 — per-node cost

Plan: [`plan2.md`](plan2.md) (initiative [`prover`](initiative.md), backlog
item 2, M1). Executed 2026-11-06 in one session, phases A–D, with no split.

## Headline

- **G1 passes, so the stop rule does not fire.** On `core`, 10 M child
  evals, `--memory 192`, same session (median of 3):
  - m1 **53.4 ms per 1 M child evals**;
  - m0 (plan1) 166.7, so m1 is −68%;
  - legacy front-end 137.3, so m1 is −61%.

  Every other cell and cap also passes both comparisons (§Per-node).
- **Phase attribution** on that cell:

  | Step | ms per 1 M | Change |
  | --- | ---: | ---: |
  | m0 | 166.7 | |
  | m1A: memory half (TT v2, filter, proof table, prefetch) | 77.3 | −89.4 |
  | m1B: compute half (`Mover`/`Effect`, frame v2) | 63.8 | −13.5 |
  | m1: + huge pages (D20 site 3) | **53.4** | −10.4 |

- **Correctness holds.** `make test-full` is green and the veto passes. The
  10 M fixture sweep gives 41 proved, 11 censored, 0 refuted, and finds 0
  of the 31 never-prove rows. All 41 written proofs were accepted by
  `atomi validate` and `provari check`, with equal positions on every file.
  There were 0 export failures and 0 proof-table overflows.
- **Two findings changed the design:**
  1. Fully lazy stalemates (D19) made the pawn-ending win `rem01` go from
     9.4 M evals to censored at 60 M. A node-level gate (G1) now builds
     every child when the child's side to move has only its commoner and
     pawns. `rem01` now proves in 8.34 M.
  2. The planned lazy allocation was not lazy for the TT. std's
     `alloc_zeroed` memsets every layout aligned above 16 bytes. An owning
     `Zeroed<T>` fixes it: a mate in one now takes 1.3 ms and 21 MiB
     (m0: 58.5 ms, 130 MiB).
- **Bench row (no goal claim):** `easy-win` **10.897** (m0: 36.365), 1.482 s
  wall, D14 proof accepted. `core` and `deep-win` stay censored. Coverage
  is 1/3.

## Provenance

| Item | Value |
| --- | --- |
| Host | `ae54d05b36f8`, AMD Ryzen 9 5950X container (4 vCPU), Linux 7.2.13 Fedora 44, THP `enabled=madvise`, `defrag=madvise` |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), cargo 1.99.1, Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154; licence gate 13 packages, clean |
| atomi m1 | HEAD `a4c135d6ee1956588e01733bd101f6373e7d2e72` on a **dirty tree** (agents do not commit); `target/release/atomi` sha256 `2008857f…2847ab` |
| atomi m0 | `git archive 9aa96c9` → `/tmp/atomi-m0`, sha256 `81dddf7b…ae62759` |
| atomi m1A / m1B | the Phase A gate tree (`/tmp/atomi-m1a`) and the Phase B tree before the huge-page site (`/tmp/atomi-m1`) |
| Legacy front-end | `atomic_solver@5f85133…:docs/plans/winprover/measurements/plan3/probe.rs` (sha256 `44edddd3…a1d59f`), built per plan finding 1 → `/tmp/plan2-legacy-probe/probe` |
| Oracle / stretch | provari 0.1.0 `48686bc6…` / provari 0.2.0 `224ce85d…` |
| Corpus | `bench/corpus.json` sha256 `9cd0169a…47d0e` (verified) |

All digests are in [`measurements/plan2/env.json`](measurements/plan2/env.json).

## Exact commands

Gates: `make test`, `make lint`, `make test-full` (includes `make veto`),
`cargo build --release`, and the plan's §Validation block. The measurement
commands (M1 per-node, M2 wall guard, M3 sweep, M4 A/Bs, M5 bench) are
listed verbatim in
[`measurements/plan2/README.md`](measurements/plan2/README.md).

## Results

### Per-node (M1; `pernode.json`)

The table shows median ms per 1 M child evals over 3 interleaved repeats.
Every run reached its cap.

- Sides: m0, m1A, m1B, m1, each at `--memory` 192 / 32; the legacy
  front-end (FE) has a fixed 32 MB table.
- Cells: `easy` = `easy-win`, `deep` = `deep-win`.

| Cell, cap | m0 | m1A | m1B | m1 | FE |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` 1 M | 162.0 / 140.0 | 84.0 / 76.0 | 112.0 / 69.0 | 63.0 / 54.0 | 154.0 |
| `core` 10 M | **166.7** / 128.6 | 77.3 / 77.4 | 63.8 / 53.4 | **53.4** / 51.2 | **137.3** |
| `easy` 1 M | 151.0 / 114.0 | 79.0 / 72.0 | 109.0 / 61.0 | 57.0 / 48.0 | 142.0 |
| `easy` 10 M | 155.9 / 124.7 | 73.7 / 70.9 | 58.0 / 48.9 | 47.4 / 46.2 | 125.8 |
| `deep` 1 M | 159.0 / 144.0 | 82.0 / 78.0 | 112.0 / 66.0 | 60.0 / 51.0 | 177.0 |
| `deep` 10 M | 170.2 / 128.3 | 77.4 / 73.9 | 59.3 / 48.7 | 48.0 / 46.9 | 146.5 |

- **Gate block:** m1 53.4, m0 166.7, legacy-fe 137.3 (`pass_m0` and
  `pass_legacy` both true). m1 beats m0 and the front-end in all 12
  (cell, cap, memory) combinations.
- **Trajectory is unchanged.** Evals per expansion are identical for m0
  and every m1 stage (`core` 32.48 at 1 M, 31.99 at 10 M). The front-end
  does 35.8–46.3 evals per node because its count includes simulation
  evals.
- **G6 memory:** m1's peak RSS is ≤ 194.1 MiB at 192 MB and ≤ 34.8 MiB at
  32 MB, so it stays within label + 16 MiB. RSS now fills toward the label
  because the TT is no longer rounded down to a power of two.
- **m1B is slower than m1A at 1 M / 192 MB** (112 vs 84). This is not a
  regression in the search. m1A's TT was eagerly memset *before* the search
  clock (finding 2), while m1B faults 4 KiB pages *inside* `ms`, as the
  plan intended. At 1 M almost every first touch is a fault. Huge pages
  remove most of that cost (m1: 63.0).
- **Against the pre-registered expectation (not a gate):**
  - memory half: about 95 expected, **77.3** measured, better than
    expected;
  - compute half: about 60 or below expected, **63.8** measured, just
    above.

  The compute half saved 13.5 ms per 1 M, against about 23 predicted by the
  spike (49.5 → 26.4 ns per child). Part of the gap is the in-`ms` page
  faults that m1A did not pay: the eager A/B costs 2.6 ms per 1 M, from
  M4 round 1. The rest is not attributed, because `perf` is unavailable.
  m1 at 53.4 is far below the "diagnosis partly wrong" line of 90.

### Wall guard (M2; `wallguard.json`)

`easy-win` proved to the end, `--seconds 600 --memory 192`, 3 repeats:

| Side | Process wall | Search ms | child_evals | expansions | length |
| --- | ---: | ---: | ---: | ---: | ---: |
| m0 | 5.046 s | 4,983 | 30,245,030 | 1,046,253 | 17 |
| m1 | **1.493 s** | 1,490 | 30,335,618 (+0.30%) | 1,047,370 | 17 |

The guard holds: m1's process wall is ≤ m0's, by a factor of 3.4. The eval
count barely moved, so the speedup is not "cheaper evals, many more of
them". The +0.30% comes from a slightly different trajectory: a larger,
non-power-of-two TT and proofs that are never evicted.

### `unsafe` keep rules (M4; `ab.json`)

The A/Bs are one-line scratch patches of the final tree, built in
`/tmp/plan2-ab-*`. Every variant gave identical counters on `core` at 1 M.

| A/B | Run | Without | With | Effect | Rule | Outcome |
| --- | --- | ---: | ---: | ---: | --- | --- |
| Huge pages | round 1, 5 repeats | 65.8 | 51.6 | −21.6% | land if ≥ 3% faster | **landed** |
| Huge pages | round 2, 7 repeats | 64.1 | 52.3 | −18.4% | (confirmation) | — |
| Prefetch | round 1, no huge pages | 65.8 | 65.8 | 0.0% | keep if ≥ 3% faster | would be removed |
| Prefetch | round 2, huge-page base (the final tree) | 54.1 | 52.3 | −3.3% | keep if ≥ 3% faster | **kept** (marginal) |
| Lazy alloc, `core`/10 M | round 2, huge-page base | eager 51.9 | lazy 52.3 | +0.8% | not > 3% slower than eager | pass |
| Lazy alloc, mate-in-1 wall | 11 repeats | m0 58.5 ms (130 MiB); eager+hp 28.6 ms (194 MiB) | **1.3 ms (21 MiB)** | drops | wall must drop | **kept** |

- The huge-page site landed, so the prefetch and lazy-allocation rules
  were re-measured on the huge-page base. That base is the final tree,
  which is what the rules are written against.
- **The prefetch keep is weak evidence.** It cleared the 3% line by 0.3
  points, and it did nothing without huge pages. Revisit it if the frame
  changes (M2/M3).
- The final tree has the three D20 sites:
  1. `Zeroed<T>`, the allocator;
  2. the `_mm_prefetch` wrapper;
  3. `madvise(MADV_HUGEPAGE)`.

  Each has `// SAFETY:` comments.

### Fixture sweep (M3; `sweep.json`)

| Side | `wins.tsv` proved / censored / refuted | never-prove found | proofs written / accepted (both checkers, equal positions) | export failures | `proof_full` runs |
| --- | ---: | ---: | ---: | ---: | ---: |
| m1 | 41 / 11 / 0 | 0 of 31 | 41 / 41 | 0 | 0 |
| m0 | 41 / 11 / 0 | 0 of 31 | 41 / 41 | 0 | 0 |

- The censored set is the same as plan1's: `rem07`–`rem13` and
  `m20`–`m23_white`. There are no regressions (G4 list empty).
- Every proved row has identical `child_evals` on m0 and m1 except
  `rem01`: 9,429,006 → 8,337,456 evals, proof length 65 → 45, 749 → 1,279
  positions, accepted. Censored rows differ only by cap overshoot.
- The largest live proof-table count in the sweep is 29,645 (`rem13`).

### Lazy stalemates (classifier test)

- **0 lazy stalemates** among 148,128 classified children over 8,272
  playout positions. The final gate is on; the earlier ungated run also
  had 0 among 83,337 moves.
- So the mechanism is exercised only by the constructed end-to-end test
  (`1K5n/5p2/5Pp1/6P1/8/3N4/N7/kN6 w - - 0 1`, `d3b2`). Its root outcome
  matches m0: proved, length 14, 646 evals.
- The random-playout corpus rarely reaches the pawn endings where lazy
  stalemates mattered (finding 1).

### Phase A counters (A4) and Phase B counters

- **A4, m0 vs m1A:** all 12 rows identical in exit, length, child_evals
  and expansions. These are the 8 veto rows, m99, r97, the pawn ending,
  and the `evals_cap` FEN at 5,000. No cause needed tracing.
- **m1A vs final m1:** the same 12 rows plus the 3 bench cells at 1 M are
  all identical.
- The only divergence anywhere is the two `rem01` runs above (fully lazy
  vs G1). The constructed stalemate FEN takes +9 expansions, which is the
  lazy mechanism itself.

### Bench (M5; `bench.json`)

| Side | `core` | `easy-win` | `deep-win` | Geomean | Coverage |
| --- | ---: | ---: | ---: | ---: | ---: |
| provari v0.1.0 hand (reference) | 0.529 s · 342,289 nodes · mate 17 | 0.136 s · 72,523 · mate 15 | 6.767 s · 4,574,912 · mate 22 | 1.000 | 3/3 |
| atomi **m1** | censored (≥ 18.9) 10.003 s / 181.8 M | **10.897** (1.482 s / 30.3 M) | censored (≥ 1.48) 10.003 s / 186.6 M | **10.897** (1 cell) | 1/3 |
| stretch: v0.2.0 / v0.1.0 hand | 1.040 | 0.985 | 1.001 | 1.008 | 3/3 |

- **D14:** the `easy-win` proof run had identical `child_evals`
  (30,335,618). It wrote 2,048 positions, length 17, and both checkers
  accepted it (`proof_verdict = accepted`).
- The legacy controls were not re-run; plan1's rows stand.
- **No goal claim.** M1 changed per-node cost, not the search.
- A 10 s `core` run leaves 367,993 live proof-table entries, `proof_full`
  false. That is 12% of 3,145,728 slots (see finding 6).

## Gate table

| Gate | Result |
| --- | --- |
| **G1** per-node | **pass**: m1 53.4 ≤ m0 166.7 and ≤ legacy-fe 137.3 (`core`, 10 M, 192); every other cell and cap passes too |
| Stop rule | not triggered |
| **G2** reference wins | **pass**: `control` proved in 150,775 evals (≤ 1 M), 308 / 308 positions, both checkers valid; `easy-win` proved in 30,335,618 (≤ 60 M), D14 accepted, 2,048 positions; wall guard m1 1.493 s ≤ m0 5.046 s |
| **G3** soundness | **pass**: `make test-full` green, veto PASS (8 rows accepted, controls `false`/`false`, reverse control agrees), never-prove and soundness rows never found |
| **G4** sweep | **pass**: 0 refuted, 0 never-prove found, 41 / 41 accepted with equal positions, 0 export failures (no `store-full`), no regressions |
| **G5** | **pass**: `make test` 92 tests (58 lib + 2 + 1 + 5 + 19 + 7), 0 failures, ≈ 11–13 s; `make lint` clean; `cargo build --release` clean; licence gate clean; no new dependency |
| **G6** memory | **pass**: peak RSS ≤ 194.1 MiB at 192 MB, ≤ 34.8 MiB at 32 MB |
| **G7** determinism | **pass**: tests for repeat runs, with vs without `--proof-out`, and filter on vs off; bench and wall-guard repeats bit-identical in `child_evals`; all A/B variants counter-identical |
| **G8** provenance | **pass**: `src/table/alloc.rs` header line + `PROVENANCE.md` row (sha256 `0c49511e…f0fa5582`, adapted); the two finding-11 digests filled in (`c702786c…068444`, `f11f0e73…25fbfb`) |
| No goal claim | the bench row is reported as measured |

## Verdict

**M1 acceptance is met.** Per-node cost fell by 68% against m0 and 61%
against the legacy front-end, in the same session, on every cell. The
reference wins are proved and checker-accepted. Soundness, sweep, memory,
determinism and provenance gates are green. The per-node diagnosis was
right in direction and magnitude:

- the memory half gave the larger part;
- the compute half gave less than the spike predicted;
- huge pages, the optional D20 site, were the second-largest lever.

## Not measured

- A hardware-counter split (cache misses, TLB). `perf` is unavailable in
  the container.
- The board-child fraction on the bench cells. It is measured only on the
  test playouts: 39.6% before the gate, 48.9% after. The playouts include
  special-move and endgame fixtures, so they are not representative.
- Prefetch on the filter-off path, and the filter's own effect on speed.
  The filter is result-neutral by test, but not timed.
- Proof-table fill on long runs (600 s `core`/`deep-win`).
- The legacy `default`/`opt` controls and provari v0.2.0 with threads (not
  in the protocol).

## Problems and findings

1. **Fully lazy stalemates cost a fixture win (scope change to D19).**
   - **What happened.** `false_refutation_regressions` failed in the slow
     tier: `rem01` hit the 20 M cap and still had not proved at 60 M. m0
     proves it in 9.43 M and m1A in 8.34 M.
   - **Bisection.**
     - With every child built (`needs_board` always true) or every quiet
       child built, the counters are bit-identical to m1A. So `Effect` is
       exact, and the cost comes from the lazy stalemate semantics alone.
     - The likely mechanism: at an AND node, a defender's quiet move
       stalemates the attacker. M0 refutes that node at once. Fully lazy,
       the frame starts the child at `(1, 1)`. The search then works on
       expensive siblings first, and the loss compounds through the tree.
       This mechanism is a hypothesis; it was not traced in the search.
   - **Fix (G1).** `Mover` builds every child when the children's side to
     move has only its commoner and pawns. It is decided once per node, so
     it costs nothing per child. Building more children is always sound.
     The pre-registered decision rule (prove within 2 × m0's evals,
     18.86 M) passed with 8,337,456 evals, identical to m1A. The wider gate
     G2 (at most one non-pawn piece) was not needed.
   - Recorded as the narrowed D19 in `initiative.md`.
2. **The TT was not lazily allocated.**
   - On unix, std's `System::alloc_zeroed` calls `calloc` only when the
     alignment is ≤ 16. For the `align(64)` `Cluster` it calls
     `posix_memalign` plus a full memset. The legacy `alloc.rs` never hit
     this, because its entries were less aligned.
   - The m1A binary, built at the Phase A gate, has this bug: the mate in
     one took 63 ms and 145 MiB.
   - Fix: `Zeroed<T>` allocates with alignment 16 plus `align` bytes of
     slack, rounds the pointer up, and stores the base and layout for
     `Drop`.
   - Recorded in `PROVENANCE.md` (`adapted:`) and as an `AGENTS.md`
     gotcha.
3. **Huge pages are a large lever on this host:** −18% to −22% on
   `core`/10 M. The likely reason is fewer TLB misses on a table far larger
   than 4 KiB-page TLB reach. That is inferred, not counter-measured. With
   huge pages, the label hardly matters any more: m1 at 32 MB (51.2) is
   close to m1 at 192 MB (53.4), against 53.4 vs 63.8 for m1B.
4. **Prefetch is marginal:** 0% without huge pages, 3.3% with them. It was
   kept by its pre-registered rule; see §M4.
5. **The legacy front-end comparison is not like-for-like.** The front-end
   has a fixed 32 MB table, counts simulation evals and does 36–46 evals
   per node (m0/m1: 30–32). m1 also wins at the same 32 MB (51.2 vs 137.3
   on `core`/10 M).
6. **The proof table never evicts (D18).** At about 37 k new entries per
   second on `core` (367,993 in 10 s), the 90% bound at 192 MB (2.83 M
   slots) would be hit after about 77 s. That is not an issue at the 10 s
   bench budget, but long `--seconds 600` runs would fail their export
   with `store-full`. This is
   M4's scaling job (bounded worker, dropping mates from the table).
7. **Upstream feedback (D2 pattern, not blocking).** `atomic-movegen`'s
   `hash_after_with(m, Legal)` clones and makes every double push. It
   could skip the clone when no enemy pawn attacks the passed square, or
   expose the per-file en passant key; either would retire our derived
   `rules::ep_key`.
8. **Deviations from plan2:**
   - No `rules::legal_child_key`: `Mover` owns the key.
   - `write_proof` takes `S: ProofSource + Sync`, because the scoped
     writer thread needs `Sync`.
   - A proof-table insert also reuses a tombstone it created in the same
     pass, so `used` does not grow on replacement.
   - Frame pass 1 writes `Child` records directly; there are no
     per-frame effect or status vectors.
   - A shared `src/testutil.rs` holds the seeded playouts.
   - The lazy-stalemate end-to-end FEN had to keep immobile non-pawn
     material on the stalemated side, so that G1 leaves the move lazy.
   - Measurement additions:
     - `docs/plans/prover/measurements/plan2/run_ab.py` and
       `wallguard.json`, which the plan's deliverables list omitted;
     - an extra per-node side `m1b`, for the huge-page attribution;
     - two M4 rounds: round 2 on the huge-page base, because huge pages
       landed.
   - Files over 10 KB carry header justifications: `src/mover.rs` 13.3 KB,
     `src/search/frame.rs` 12.5 KB, `src/table/alloc.rs` 14.0 KB,
     `src/table/proofs.rs`, `src/table/tt.rs`, `src/rules.rs` 11.2 KB,
     `src/search/mod.rs`, `tests/search.rs`.

## Missing tests

- **A lazy stalemate inside a real search trajectory**, beyond the
  constructed FEN. The playout corpus produced 0; `rem01` covers only the
  gated case, in the slow tier.
- **An automated RSS or laziness guard for `Zeroed<T>`.** Laziness is
  measured only (M4). An in-process RSS test would be flaky under parallel
  tests; a separate-process CLI test is an option.
- **An effect test for the huge-page advice.** Only contents and alignment
  are pinned.
- Still open from report1: castling and en passant inside a
  checker-accepted proof; a real repetition-free detour win; the
  two-counter writer case end to end; a writer-produced cross-line cycle.

## Next steps

1. **plan3 = M2 (item 3, search guidance):**
   - move-class starting numbers, EF-PN hand-eval scaling and quiet
     grouping, with constants exposed for an external optimizer;
   - the gate is `core` and `easy-win` proved within a stated cap,
     measured against M1. M1's reference numbers are in `initiative.md`.
   - Keep G1. Re-check prefetch if the frame changes.
2. Options before or with M4:
   - proof-table scale for long runs (finding 6);
   - pooling the per-frame `Vec`s;
   - the board-child fraction on the bench cells, as a cheap counter.
3. File the `ep_key` upstream feedback with `atomic-movegen`.

## Delivered files

- **Engine:**
  - `src/table/{mod, tt, filter, proofs, alloc}.rs`; `src/table.rs`
    removed;
  - `src/mover.rs`, `src/testutil.rs`;
  - updated `src/search/{mod, frame}.rs`, `src/rules.rs`,
    `src/strategy/{mod, store, write}.rs`, `src/cli.rs`, `src/main.rs`,
    `src/lib.rs`.
- **Tests:** updated `tests/search.rs` and `tests/fixture_oracle.rs`, plus
  new unit tests in every new module.
- **Drivers:** extended `bench/run_pernode.py`, `bench/README.md`; new
  `docs/plans/prover/measurements/plan2/{run_sweep.py, run_ab.py}`.
- **Measurements:**
  `docs/plans/prover/measurements/plan2/{README.md, env.json, pernode.json,
  pernode.csv, ab.json, wallguard.json, sweep.json, bench.json}`.
- **Docs:** `PROVENANCE.md`, `AGENTS.md`, `bench.md`, `initiative.md`,
  `docs/plans/README.md`, this report.
- **Scratch, never committed:**
  - binaries: `/tmp/atomi-m0`, `/tmp/atomi-m1a`, `/tmp/atomi-m1`,
    `/tmp/plan2-ab-{nopf, hp, eager, hp-nopf, hp-eager}`,
    `/tmp/plan2-legacy-probe`;
  - the sweep's proof files (deleted by the driver).

## SESSION COMPLETE

**Deliverables.** M1 is implemented, measured and documented. All gates
G1–G8 are green, and the stop rule did not fire. Per-node cost on `core` at
10 M is 53.4 ms per 1 M child evals (m0 166.7, legacy front-end 137.3).
Nothing is committed. The owner commits the working tree: about 20 modified
files, plus the new `src/table/`, `src/mover.rs`, `src/testutil.rs` and
`docs/plans/prover/measurements/plan2/`.

**Follow-up options.**

1. Plan M2. Kickoff prompt:

   > Plan session for `docs/plans/prover/plan3.md` (initiative `prover`,
   > backlog item 3, M2 search guidance). Read `AGENTS.md`,
   > `docs/plans/prover/initiative.md` (D1–D20), `plan2.md`, `report2.md`
   > and `bench.md`. Baseline: the plan2 tree (atomi m1: 53.4 ms per 1 M
   > child evals on `core` at 10 M; `easy-win` proved in 30,335,618 child
   > evals, 1.482 s; `core`/`deep-win` censored at 10 s). Scope: move-class
   > starting numbers, EF-PN hand-eval scaling (attacking branch vs both),
   > quiet grouping, with the constants exposed per the legacy optimizer
   > contract (spec only). provari is ideas only (GPL boundary). Gate:
   > `core` and `easy-win` proved within a stated cap and measured against
   > m1 in the same session, with the 10 M fixture sweep and both checkers.
   > Keep the D19/G1 stalemate gate and the `OPEN_MAX` rule. Run read-only
   > probes only and write a self-contained `plan3.md`.

2. A short M1.5 plan for proof-table scale on long runs (finding 6), before
   M4.
3. File the `ep_key` upstream feedback (finding 7) with `atomic-movegen`.
