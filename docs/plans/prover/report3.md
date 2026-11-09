# Prover Report 3: M2+M3 — the hand-guide stack

Plan: [`plan3.md`](plan3.md) (initiative [`prover`](initiative.md), backlog
item 3, M2+M3 merged by D21). Executed 2026-11-09. The **stack gate G1 did
not pass**: the implementation is complete, the measurement set is frozen and
the gate was run, but `core` is not proved at 10 s and `easy-win` misses the
145,046-expansion limit. Per the D21 stop rule the stack is recorded rather
than split back into mechanism plans.

## Headline

- **The hand-guide stack is implemented and sound, but it does not meet D21.**
  - `easy-win` (guidance set, 20 M cap, seeds 0–3) proves in 211,254–631,763
    expansions — every seed is above the 145,046 limit.
  - `core` is censored on every seed at 20 M child evals (≈ 0.64–0.65 M
    expansions) and at 10 s (≈ 0.15 M expansions).
  - The 10 s wall condition fails for both gate cells.
- **The guide does work on tactical wins.** All 18 `wins.tsv` cells in
  `guidance-v1` are proved on all four seeds, often with 2–100× fewer
  expansions than the bare skeleton (`dec03` 2,087 → 27; `dec09` 78 → 2;
  `dec18` 1,392 → 7). Coverage is 19/24 for `hand` versus 24/24 for provari
  hand (the set is frozen by *provari* wins, not atomi wins).
- **`core` is the blocker.** At the same 20 M cap the bare skeleton reaches
  623–630 k expansions and the full stack 644–654 k; the guide does not
  reduce `core`'s tree at that horizon, while the auxiliary work makes each
  expansion several times more expensive.
- **`EF-PN` is the `easy-win` cost.** With `--guide-off efpn` the same stack
  proves `easy-win` in **110,293 expansions** at 10 s, under the gate, while
  the full stack needs 211–632 k. This matches the retro's provari ablation
  (`easy-win` 0.44× without EF-PN): the mechanism helps `core` and hurts
  `easy-win`.
- **No soundness failure.** No guidance-set refutation, no never-prove row,
  `make test-full` green, veto green, one D14 proof accepted by both
  checkers with identical counters.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X container (4 vCPU), Linux 7.2.13, THP `madvise` |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154; licence gate clean |
| atomi | HEAD `3947d0b9dcd1a0f11b9bce05e3d21297213e73a8` on a dirty tree (agents do not commit) |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Corpus | `bench/corpus.json` sha256 `9cd0169a…47d0e` (verified) |
| Guidance set | `guidance-v1.tsv`, 24 cells, sha256 `2da2a65b960a65c6d4a722cc6b1b50b802541814f9443f66d077036a47ee8c7d` |
| Census | 73 candidates (`core + sample20 + wins.tsv`, deduplicated by FEN), 30 s / 60 s selection budgets |

All digests and the binary hashes are in
[`measurements/plan3/env.json`](measurements/plan3/env.json).

## Exact commands

Gates: `make test`, `make lint`, `make test-full` (includes `make veto`),
`cargo build --release`, the plan's §Validation block (Python `ast.parse` on
the drivers, corpus sha256, `git status`). The measurement commands are in
[`measurements/plan3/README.md`](measurements/plan3/README.md).

The gate was run as four cell shards (`guidance-shard0..3.json`, documented in
the README) because the single-process run was not finishing in one sitting;
the deterministic `expansions` counters are unaffected by the sharding. The
shards were merged into `guidance.json` with the same summary/gate code.

## Results

### Census and the frozen set

- 73 candidates, 30 s selection budget, one 60 s retry on a censor.
- 24 shared candidates were selected in the plan's fixed order: `core`,
  `easy-win`, `deep-win`, then the remaining shared `sample20` cells
  (`b9a7ed7eb368`, `3004d09cef79`, `dd3f89766a40`), then 18 `wins.tsv`
  cells (`dec01`–`dec20`, first occurrence by file order).
- The set was frozen before the gate. Its sha256 is pinned in `env.json`; the
  driver refuses a mismatch.

### D21 node gate (`guidance.json`)

`--seconds 600 --evals 20000000 --memory 192`, seeds 0–3, across the 24
cells. Expansions are deterministic.

| Cell | provari nodes | bare expansions (seed 0/1/2/3) | hand expansions (seed 0/1/2/3) | hand result |
| --- | ---: | --- | --- | --- |
| `core` | 342,289 | 630,056 / 623,703 / 626,795 / 623,412 | 648,828 / 653,551 / 647,032 / 644,280 | censored (20 M) |
| `easy-win` | 72,523 | 669,920 / 667,086 / 665,909 / 662,055 | **468,301 / 631,763 / 211,254 / 556,280** | proved, all seeds |
| `deep-win` | 4,574,912 | 668,608 / 659,429 / 662,882 / 662,470 | 663,127 / 662,502 / 662,816 / 663,388 | censored |
| `b9a7ed7eb368` | 13,952,139 | 754,345 / 736,767 / 757,905 / 749,828 | 741,385 / 739,832 / 741,685 / 739,700 | censored |
| `3004d09cef79` | 18,273,275 | 667,774 / 651,330 / 657,353 / 653,302 | 644,228 / 644,544 / 645,199 / 644,211 | censored |
| `dd3f89766a40` | 17,714,846 | 681,655 / 677,156 / 679,578 / 682,453 | 737,785 / 735,399 / 731,446 / 738,848 | censored |
| 18 `wins.tsv` cells | 1–32,863 | e.g. `dec03` 2,087 / 1,965 / 1,015 / 1,812 | e.g. `dec03` 27 / 26 / 56 / 50 | proved, all seeds |

- **Coverage:** provari hand 24/24 (by selection), atomi hand **19/24**,
  atomi bare 18/24. The five misses are `core`, `deep-win`, `b9a7ed7eb368`,
  `3004d09cef79`, `dd3f89766a40`.
- **Refutations:** none on the guidance set (every `wins.tsv`-derived cell
  was proved on every seed).
- **Seed spread on `easy-win`** is large (211 k … 632 k). Seed 2 is the only
  one near the limit, and still 1.46× above it.
- The gate reference walls in this file were measured while the shards ran
  concurrently and are not used for any wall claim; only `nodes` are used.

### Wall / coverage gate (`wall.json`)

`--seconds 10 --memory 192`, seeds 0–3, 3 repeats, same-session reference.

| Cell | provari hand | atomi hand | result |
| --- | ---: | ---: | --- |
| `core` | 0.528 s · 342,289 nodes | 10.03 s · ≈ 150 k expansions | **censored** |
| `easy-win` | 0.133 s · 72,523 nodes | 10.03 s · ≈ 190 k expansions | **censored** |

No proof-check cell was won in the 10 s wall run, so the D14 loop had no
eligible cell. One separate `easy-win` proof run (seed 2, 600 s) had
identical child evals (5,694,014), wrote 3,410 positions, length 17, and was
accepted by **both** `atomi validate` and `provari check` with equal
positions.

### Per-node regression (M1 guard)

Same session, `core`, 10 M child evals, 192 MB, median of 3:

| Side | ms / 10 M | ms / 1 M |
| --- | ---: | ---: |
| m1 (`git archive HEAD` baseline) | 539 | 53.9 |
| plan3, `--guide none --seed 0` | 583 | 58.3 |

The no-guide trajectory is counter-identical to m1 (10,000,007 child evals,
312,551 expansions, 13,920 proof entries; also 1,000,032 / 30,786 at the
1 M cap), but the guide plumbing costs **+8.2 %** per node even when off.
Skipping class computation and the per-node `starts` allocation when the
guide is off reduced this from +11.3 % to +8.2 %; the remainder is the bigger
`Frame`/`Child` records and the extra context call.

The final binary (sha256 `0ae697…a060`) reproduces the gate's `core` and
`easy-win` `expansions` and `child_evals` for all four seeds exactly; the
shard run and the final tree differ only in guide-off plumbing.

### No-guide identity and determinism

- `--guide none --seed 0` on `core` at 1 M: 1,000,032 child evals, 30,786
  expansions, 1,028 proof entries — identical to the m1 binary.
- At 10 M: 10,000,007 / 312,551 / 13,920 — identical to m1.
- `--guide none --seed 0` never produces a pending settle, simulation or
  history effect (`settle_nodes = sim_nodes = 0`).
- Same binary + FEN + flags + seed repeats bit-identically; `--proof-out`
  does not change counters (tests and the D14 run).
- Different seeds change the tree but not the result on `easy-win`
  (proved on all four seeds) and not the refutation status on the fixture
  sets.

### One-ply settle differential

Over the seeded playouts, every `one_ply_settle` hit was verified by making
the move: `SideToMoveWins` leads to `status == Won`, `SideToMoveLoses` to
`status == Failed`. The unit test counts children/settles and fails on any
disagreement.

The settle is **not** redundant with `rules::status` on a built child: a
child whose side to move can explode on the next ply has `status == Open`
(the position itself is not over). The first implementation skipped built
children; the differential test caught it and the frame now settles every
fresh child (built directly, non-built after a conservative screen).

### Ablations

The full both-direction ablation over `guidance-v1` was **not run** because
G1 failed (plan C4 is conditional). A reduced measurement on `core` and
`easy-win`, seed 0, 10 s / 10 M is in `ablation.json`:

| Stack (seed 0) | `core` expansions | `easy-win` expansions | result |
| --- | ---: | ---: | --- |
| `full` | 148,480 | 185,344 | both censored at 10 s |
| `lo_efpn` | 144,384 | **110,293** | easy-win **proved**, core censored |
| `lo_classes` | 137,216 | 160,768 | both censored |
| `lo_quiet` | 160,768 | 223,232 | both censored |
| `lo_evasions` | 146,432 | 196,608 | both censored |
| `lo_priors` | 156,672 | 174,080 | both censored |
| `lo_history` | 148,480 | 199,680 | both censored |
| `lo_replay` | 157,696 | 172,032 | both censored |
| `lo_closures` | 323,559 | 316,559 | both hit the 10 M cap |
| `lo_sim` | 311,735 | 386,992 | both hit the 10 M cap |
| best add-one-in (`evasions`) | 298,900 | 313,561 | both hit the 10 M cap |
| bare skeleton | 312,551 | 312,551 | both hit the 10 M cap |

Reading (no adoption decision — the gate failed):

- `efpn` is the only leave-one-out that turns `easy-win` from censored into
  proved, at 110,293 expansions — **below** the 145,046 limit, while the
  full stack is above it.
- `closures` and `sim` are tree levers for `easy-win` (removing either
  pushes it to the 10 M cap) but cost the per-node budget; on `core` they
  make the censored count much larger because the cap fires.
- `classes` and `quiet` help `easy-win` and hurt `core` at this horizon;
  `evasions` is the reverse. No add-one-in mechanism proves either cell.
- No mechanism is adopted or removed: the plan's rule is that
  both-direction ablations decide after a passing stack gate.

## Gate table

| Gate | Result |
| --- | --- |
| **G1 D21 stack gate** | **fail**: `easy-win` 211,254–631,763 expansions (limit 145,046); `core` censored on seeds 0–3 at 20 M and at 10 s |
| **G2 soundness/coverage** | **fail on coverage**: no refutations on the guidance set, but hand 19/24 vs provari 24/24; the one D14 proof was accepted by both checkers with identical counters |
| **G3 fixture oracle** | **pass**: `make test-full` green (guide-none and hand oracle, never-prove, regressions, veto) |
| **G4 determinism/accounting** | **pass with a cost finding**: no-guide identity exact, same-seed repeatable, `--proof-out` neutral, `settle_nodes`/`sim_nodes` separate, no `Open` at `INFINITY`; per-node +8.2 % |
| **G5 tests/lint/licence** | **pass**: `make test`, `make lint`, `cargo build --release`, licence gate clean, no new dependency |
| **G6 measurement integrity** | **partial**: 24 cells, 4 seeds, hash pinned, parsed JSON/CSV under `measurements/plan3`; the census ran after development probes on the three bench cells (the set/rule were fixed in the plan and no atomi result selected a cell) |
| **G7 ablations** | **not run**: G1 failed; the reduced easy/core measurement is reported as evidence only |
| **G8 docs/provenance** | **pass**: AGENTS/bench/initiative/plans README updated, no copied code or data; `run_guidance.py` is new |

## Verdict

**M2+M3 is not accepted under D21.** The stack implements the nine named
mechanisms, the guide changes only `Open` numbers and tie-breaks, the
settle is exact and its proofs round-trip, and the no-guide path is
counter-identical to m0/m1. But the absolute target is not met:

- `easy-win` proves, but 1.5–4.4× above the node limit; `EF-PN` alone is the
  difference and removing it lands at 110 k on seed 0.
- `core` does not prove at 10 s and its 20 M expansion count is not better
  than the bare skeleton. The stack's tree gain is concentrated in the
  tactical `wins.tsv` cells.
- The auxiliary work (simulation and settle) is a large per-node cost
  (50.6 M sim + 17.7 M settle nodes for 10 M `core` child evals).

This is the D21 stop condition: record the stack, the frozen set and the
measurements; do not split the stack into one-mechanism plans.

## Not measured

- The full both-direction ablation over all 24 cells and seeds 0–2 at
  10 M (G7): conditional on G1.
- The canonical 3-cell bench row (`bench.json`): conditional on G1–G7; no
  row is appended to `bench.md`.
- `deep-win` at a budget large enough to prove with the stack.
- Wall factors on the guidance set (the gate reference walls are contended
  and not used).
- Memory/laziness and cache counters for the larger `Frame`/`Child`
  records.

## Problems and findings

1. **`EF-PN` is the easy-win cost, not a gain (measured).** The full OrOnly
   stack needs 211–632 k expansions; `--guide-off efpn` proves in 110 k at
   10 s. This reproduces the retro's 0.44× provari figure. The mechanism is
   kept in the stack because D21 names it and the core case may need it, but
   it is a removal candidate at a kill review.
2. **The guide does not reduce `core`'s tree at the 20 M horizon.** Bare
   623–630 k vs hand 644–654 k expansions. The extra mechanisms cost
   per-node time without changing the core tree; `core` remains censored
   at 20 M and at 10 s.
3. **One-ply settle must run on built children too.** A child whose side to
   move can explode on the next ply has `rules::status == Open`; the first
   implementation only settled non-built children and the differential
   test caught the missing closure. After the fix the settle is exact and
   its proof entry is stored/emitted at the child key.
4. **A `select` equal-metric bug changed the no-guide trajectory.** The
   equal-metric branch initially updated the best child but not `second`,
   which changes the 1+ε child thresholds. Fixing it restored
   `--guide none --seed 0` to the m1 counters; a regression test now pins
   `core` at 1 M and 10 M.
5. **Settle/simulation accounting.** `settle_nodes` and `sim_nodes` are
   separate and excluded from `child_evals`/`expansions`; a 10 M `core`
   run spends 17.7 M settle and 50.6 M simulation nodes. The gate therefore
   compares like with like but the wall is dominated by this work.
6. **Per-node regression of the guide plumbing.** Even with the guide off
   the new `Frame`/`Child` records cost +8.2 % on `core`/10 M. This is
   below the M1 gain and is recorded rather than hidden; it is a follow-up
   candidate (smaller `Child`, no context call when off).
7. **Census ordering deviation.** The plan requires the census before any
   atomi stack run. Development probes on `core`/`easy-win` happened first;
   the candidate order, 30/60 s budgets and selection rule were fixed in
   the plan and the probes did not affect the set. Recorded for G6.
8. **Gate sharding.** The single-process gate was too long for the session;
   four deterministic cell shards were run and merged. `expansions` are
   seed-deterministic, so the merge is exact; reference walls from the
   contended shard period are not used.

## Missing tests

- A test that finds a real searched line where a settled `Proved` child is
  on the writer's path (the constructed `MATE_IN_2` case covers the round
  trip; a playout-generated case would be stronger).
- A test that the null-side signature never observes a null move in the
  full search path (the unit test pins the pure function; there is no spy
  sink for table/line access).
- A hand-guide never-prove check beyond the existing fast/slow fixture
  tests.
- A per-node benchmark harness for the guide-on stack (the driver reports
  `settle_nodes`/`sim_nodes`, but no kept per-node JSON).

## Next steps

1. **Kill review (D21 stop rule).** Decide continue/re-scope/close with:
   - `easy-win` at 110 k without EF-PN;
   - `core` unchanged by the stack at 20 M;
   - the tactical `wins.tsv` cells proving well.
2. If continuing: **make EF-PN an easy/defender switch** (or default off in
   the easy stack) and attack `core` separately; the current all-mechanisms
   `hand` is dominated by simulation/settle cost on that cell.
3. Report the per-node plumbing regression (+8.2 %) as an M1 follow-up.
4. Do not append a bench row; the goal claim remains M5's job.

## Delivered files

- **Engine:** `src/guide/{mod,class,eval,threat,history,simulate}.rs`;
  `src/search/{mod,frame,value}.rs` (`Child.class`, pending settles,
  `Limits.guide/seed/guide_params`, `SearchResult.settle_nodes/sim_nodes`);
  `src/cli.rs`, `src/main.rs`, `src/lib.rs`.
- **Drivers/measurements:** `docs/plans/prover/measurements/plan3/
  {run_guidance.py, README.md, env.json, census.json, guidance-v1.tsv,
  guidance.json, wall.json, ablation.json}`.
- **Tests:** guide unit tests; frame/classifier and settle differential;
  `tests/search.rs` (seed/guide contract, no-guide identity, same-seed
  repeatability, settled-child proof round trip);
  `tests/fixture_oracle.rs` hand oracle.
- **Docs:** `AGENTS.md`, `docs/plans/README.md`, `initiative.md`,
  `bench.md` (gate result note, no ledger row), this report.
- **Scratch, never committed:** `/tmp/atomi-oracle`, `/tmp/atomi-m1check`,
  `/tmp/plan3-easy-s2.jsonl`, `/tmp/atomi-plan3-proofs`.

## SESSION COMPLETE

**Deliverables.** The hand-guide stack is implemented, tested, measured and
documented. All nine mechanisms exist behind runtime switches, the no-guide
path is m1-identical, the settle is exact and proof-round-trippable, and
`make test-full`/`make lint`/the licence gate are green. The D21 stack gate
**failed**: `easy-win` misses 145,046 expansions on every seed and `core`
is censored at 10 s and 20 M. The measurement set is frozen at 24 cells and
both the node gate and the 10 s wall run are committed as evidence. Nothing
is committed by the agent; the owner commits the working tree.

**Follow-up options.**

1. **Kill review for item 3 (D21 stop rule).** Kickoff prompt:

   > Kill review for the atomi `prover` initiative after plan3. Read
   > `docs/plans/prover/report3.md`, `plan3.md`, `initiative.md` and
   > `measurements/plan3/guidance.json`. Answer: what is the absolute
   > distance to the goal now; would we build this stack again; which
   > mechanisms would you cancel; and what is the smallest next experiment
   > that could make `core` provable at 10 s? Do not implement.

2. **A focused core plan** (new item, if the review says continue):
   measure the core tree with EF-PN off/`both`, simulation off and settle
   off at 20 M, then decide whether the null-move replay/priors are the
   missing core lever. Gate: `core` proved at 10 s.
3. **M1 follow-up:** recover the +8.2 % guide-plumbing regression with a
   smaller `Frame`/`Child` and no guide context call when off; re-run the
   per-node driver.
4. **M4 (scaled proof output)** can proceed independently: the settle proof
   path is exact and the writer/validator/veto gates are green at the
   current scale.
