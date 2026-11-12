# Bench — performance ledger vs. provari

Status: **standing document of the [`prover`](initiative.md) initiative** (the
successor of `winprover`). It fixes the benchmark set, the metric, and one
*appended* row per executed plan, so the performance leg has a single,
comparable history. It is not a plan and not a report: plans live in
`planN.md`, their evidence in `reportN.md` / `measurements/planN/`.

## Successor reading notes

- **The product hand guide was a net loss on the tree** (analysis1 §1.2,
  re-measured in plan19): `--guide hand` cost 0.97× of `--guide none` on
  `core` at 20 M child evals (648,828 vs 630,056 expansions) and 12.79 M vs
  9.27 M expansions tree-only at 120 s. Item 26 replaced its starting-number
  algebra (report19): `--guide hand` is now R1+R3+R6 and cuts the tree
  2.8–4.7× against the old product stack. Rows before plan19 measure the old
  product stack; the `Stack` column says which.

Differences from the predecessor ledger, stated once so the verbatim
sections below stay untouched:

- The legacy ledger's 'item 6 default decision' maps to milestone **M5** of
  this initiative.
- The item-2 checker is here the repo's own `atomi validate` **plus**
  `provari check` (decisions D9/D14): a win counts only when both accept.
- The candidate command is `atomi prove "<FEN>" --seconds 10 --memory 192`.
- Protocol note: atomi's `--memory` labelled only the transposition table in
  M0 (the proof store had its own `--proof-memory` budget outside the
  label). From M1 (plan2) `--memory` covers the TT, the proof table and the
  filter; `--proof-memory` is gone.

## The goal this ledger measures

The initiative's success condition, reformulated as a necessary-condition
staircase by **D30** (plan session 17, 2026-11-10), superseding the tier
clauses of D27/D28/D29, quoted from [`initiative.md`](initiative.md):

T_5 (goal): all three cells proved at 10 s, geomean wall < 1.0 vs provari --guide hand, every claim D14-accepted.

The goal is the top rung of a chain of **necessary conditions**, each a subset
of the next (`T_5 ⊆ T_4 ⊆ … ⊆ T_1`), so `goal ⇒ every rung` and a failed rung
is a hard "not yet":

| Rung | Requirement | Implied by the next |
| --- | --- | --- |
| `T_1` | `core` proved at 120 s | yes |
| `T_2` | `core` + `easy-win` at 120 s | yes (more cells) |
| `T_3` | all three at 120 s | yes (more cells) |
| `T_4` | all three at 10 s | yes (tighter budget) |
| `T_5` | all three at 10 s **and** geomean `< 1.0` | — (the goal) |

Nesting is sound because the search is deterministic (D22) and `--seconds`
only stops it, so proving at 10 s implies proving at 120 s; more cells is
monotone by definition. The wall clause lives only at `T_5` because
`geomean < 1` does not imply each cell `< 1`. A lower rung is necessary, never
sufficient; a censored cell is never converted and a longer run is never
substituted. Current position: `T_2` (plan15).

- every claimed win is accepted by this repo's own replay validator **and** by
  the pinned `provari check` (every rung);
- this repo's gates are green, and there are no outcome regressions on the
  copied fixture oracle (§Corpus and fixtures);
- the reference is re-pinned only via D17 at M5; the node/`child_evals` score
  option is not chosen.

The original handoff §2 wall-only condition (geomean `< 1.0`; coverage at
least provari hand's) is the `T_5` wall clause, not deleted and not silently
changed.

## Absolute distance to the goal (read this first)

State on 2026-11-09, after plan4 CORE-RESET (item 3 closed):

- **`easy-win` is now a 10 s win, with a 42× wall.** Calibration
  (`global_scale=64, ef_weight=0`) proves it on seeds 0–2 in
  121,866 / 110,533 / 115,147 expansions (limit 145,046) at 5.8 s median
  versus provari hand 0.137 s / 72,523 nodes. Tree ratio 1.52–1.68×; wall
  ratio ≈ 42× because simulation/settle work dominates.
- **`core` still censors at 10 s.** Best calibrated 10 s count is ≈ 146 k
  expansions (provari 0.51 s / 342,289 nodes); at 50 M child evals the tree
  is 1.61 M expansions (4.7× provari). A per-frame auxiliary budget proves
  `core` at **120 s** (278.5 M evals, length 20, proof accepted by both
  checkers), not at the 10 s product budget.
- **Per node atomi is still at or better than provari** (M1: 53.4 ms per
  1 M child evals on `core`/10 M/192 MB). The remaining `core` gap is tree
  size plus guide-auxiliary cost, not raw movegen.
- Plan3's uncalibrated hand stack is in the row history below:
  `easy-win` 211,254–631,763 expansions, `core` censored. Plan4's EF-PN-off
  point reproduces the plan3 `--guide-off efpn` count exactly (110,293 at
  scale 1 / weight 0).

Implication: the next step is the M4-first pivot recorded in `report4.md`
(scale the accepted `core` proof), or a wall-time auxiliary budget. No new
per-node work is indicated. This section is the absolute number; the rows
below are relative history.

**Update (plan session 7, D24).** M4 is parked as [`planX`](planX.md): proof
export does not move the wall geomean. The next plan is [`plan7`](plan7.md)
(item 16): wall attribution, then a deterministic global auxiliary budget
under a wall gate. No ledger row follows from this note.

**Update (plan7 executed, 2026-11-09).** Item 16's wall gate **failed**, so
no ledger row is appended. Measured same-host medians at 10 s: calibrated
hand `easy-win` 5.829 s (42×), best auxiliary-budget point 3.891 s
(28×), no-sim 2.096 s, bare 1.965 s; `core` and `deep-win` censored. The
auxiliary wall premise survives (no-sim removes 64 % of easy-win wall), but
no auxiliary lever reaches the 0.5 s gate. The absolute distance is now
easy-win ≈ 27×, core ≥ 19× censored, deep-win censored, coverage 1/3. See
[`report7.md`](report7.md) and [`measurements/plan7/`](measurements/plan7/README.md).

**Update (plan session 8, 2026-11-09).** Item 16 is closed negative; the
hand-guide/auxiliary line is not continued. The next plan is
[`plan8`](plan8.md) (item 17): a same-session per-expansion re-diagnosis
against provari's per-node wall plus a 60 s coverage bracket. It either opens
item 18 (restarts + core throughput) or recommends the item-19 owner re-pin.
No ledger row.

**Update (plan session 9, D27, 2026-11-09).** The owner accepts the re-pin.
The absolute distance is unchanged: `easy-win` 58.4× at 60 s (6.013 s vs
0.103 s), `core` and `deep-win` censored at 60 s (≥ 121.5× and ≥ 9.36×),
coverage 1/3; the guide-free `core` floor is 17.1 s (34.6×), and `neither`
costs 2.83×/3.44× provari's ns per node. From [`plan9`](plan9.md), Tier 1 is
**3/3 at 120 s / 192 MB** with D14 per claim; Tier 2 is the original 10 s
geomean on the same cells. No performance row is appended by the re-pin; the
next row must state coverage before wall.

**Update (plan9 executed, 2026-11-10).** Tier 1 coverage at 120 s / 192 MB is
**1/3**, so Tier 1 **fails** and no ledger row is appended. `easy-win` is
3/3 at 5.765 s median (54.9× the same-session provari 105 ms; D14 accepted on
seeds 0–2). `core` is 0/3 (≥ 240.0×, 1.96 M expansions at the 120 s censor)
and `deep-win` is 0/3 (≥ 18.2×, 2.03 M expansions). Same-session provari hand:
core 500 ms / 342,289 nodes, deep-win 6,590 ms / 4,574,912 nodes. Tier 2 is
not claimed; `REPLAY-BOUND` was not started. See
[`report9.md`](report9.md) and [`measurements/plan9/`](measurements/plan9/README.md).

**Update (plan10 executed, 2026-11-10).** Item 18's Phase A stop rule fired.
At fixed 20 M child evals the product wall is 41.781 s (`core`) and 38.560 s
(`deep-win`); with simulation and the one-ply settle disabled the same search
is 3.160 s and 2.533 s. Auxiliary work is **92.44% / 93.43%** of the product
wall on both cells and all seeds, so the only material lever is the closed
auxiliary line. `RESTART-GEN` was not implemented, the Phase C work item and
the Phase D checkpoints were not run, and no coverage or wall claim is made.
Tier 1 remains 1/3 from plan9; no ledger row. See
[`report10.md`](report10.md) and
[`measurements/plan10/`](measurements/plan10/README.md).

**Update (plan11 executed, 2026-11-10).** The bare tree (`--guide none`) is
**2/3 cells** at 120 s / 192 MB: `core` 3/3 at 17.390 s (35.3× the
same-session provari 492 ms) and `easy-win` 3/3 at 2.008 s (19.1×), both far
better than the hand product (0/3 and 5.765 s / 54.9×). `deep-win` is **1/3**:
seed 1 proves at 111.218 s (1.399 G child evals, length 25), seeds 0 and 2
censor at 120 s (≥ 18.6×). All seven D14 proofs are accepted by both
checkers. The bare tree dominates hand on every measured cell, so the
portfolio is rejected as dominated; Tier 1 is **2/3, fail**, the 120 s budget
is not raised, and no ledger row is appended. The `guidance-v1` seed-0
diagnostic is 20/24 (hand was 19/24 at 10 s). The next lever is a `deep-win`
tree-size mechanism. See [`report11.md`](report11.md) and
[`measurements/plan11/`](measurements/plan11/README.md).

**Update (plan14 executed, 2026-11-10).** The kill review found no non-hand
mechanism with a credible 2× case: the 120 s `deep-win` tree is 50,016,256
expansions / 40,708,534 unique keys (18.6 % re-expansions, 2,661,718 proof
entries, `proof_full: false`), ~8.9× provari's 4.57 M nodes, so the gap is
unique positions, not memory, re-expansion, proof pressure or per-node cost.
The owner accepted **option A**: **D28** amends D27 so Tier 1 is **2/3**
(`core`, `easy-win`) at 120 s / 192 MB; `deep-win` is recorded as 1/3 and is
not a pass condition. The 120 s budget is not raised, the hand-guide/auxiliary
line stays closed, no portfolio is built, and the bare tree is the product
candidate. No ledger row, no engine change. See [`plan14`](plan14.md).

**Update (plan15 executed, 2026-11-10).** The final head-to-head confirms
Tier 1 **2/3** at 120 s (`core` 3/3 17.471 s / 35.8×, `easy-win` 3/3
2.023 s / 18.9×, `deep-win` 1/3 seed 1 113.458 s; 7/7 D14 accepted) and
records a **Tier 2 failure** at 10 s: coverage **1/3**, shared-cell geomean
**15.757** (`easy-win` 1.686 s vs 0.107 s), `core` ≥ 20.49× and `deep-win`
≥ 1.549× censored. The absolute distance is unchanged from plan11. The
initiative goal is not met; the recommendation is to close with the negative
Tier 2 result or take an owner decision for a new speed mechanism. See
[`report15.md`](report15.md).

**Update (plan session 18, 2026-11-10).** Item 24 (conformance) is **closed
negative**: Phase B found no first move-divergent position is a defender
node, and exact replay cuts `core` only 1.02× at 20 M. The absolute distance
is unchanged (`T_2`; `T_3` needs ~2× on `deep-win`; `T_5` needs 20–36× wall
plus 3/3). The proof-set gap is 0.10–0.20 % (proof positions vs expansions),
so the search is tree-directedness-limited. **D31** authorises **item 25**, a
DF-PN exactness stack (exact simulation + exact null-threat replay) gated on a
≥ 2× `deep-win` unique-expansion cut at 20 M / 192 MB; PDS-PN/PN² are rejected
for now because they replace the DF-PN base. No ledger row follows from the
decision. See [`plan18`](plan18.md).

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §Names and ids -->

## Names and ids (read this first)

Three kinds of identifier appear here and in the measurement files; **only one
of them is a git commit**.

| Kind | Example | Meaning |
| --- | --- | --- |
| Cell short name | `core`, `easy-win`, `deep-win` | Use this in prose, plans, reports and ledger rows. Defined in [The bench set](#the-bench-set). |
| Cell id | `1eb80b92f92c` | **Not a git hash** — provari's record id for a position in its corpus file (detail below). Used by tooling and by the corpus file. |
| Commit pin | `provari-v0.1.0`, `solver@96edb83` | A real git commit: the oracle build, and the solver state a row was measured at. Written in full (40 hex) in the pins list below. |

**Commit pins.**

- `provari-v0.1.0` = `48686bc6c786d6ea70568f0b2ad4002cf0520206` — the pinned
  provari oracle (`v0.1.0`), rebuilt from source for every row, never
  committed. This is the 1.000 side of every factor.
- `solver@96edb83` = `96edb8334527291225a9db3b4750f9aebcb88f96` — the
  `atomic_solver` commit the `bench-v0` row was measured at. Later rows name
  their own `solver@<sha>`.

**Cell ids in detail.** A cell id (`1eb80b92f92c`, `db6a741c723f`, …) is
provari's own record id in its corpus file `bench/high200.jsonl` at the pinned
commit: the first 12 hex characters of `sha256(FEN)`, i.e. a fingerprint of the
position, unique inside that file. The frozen `sample20`
([`corpus.json`](../provari/measurements/plan1/corpus.json)) copies those
`{id, fen}` pairs verbatim, and the bench driver keys cells by id because that
is what the corpus file holds. `core` is the one hand-named cell (it is not in
`high200.jsonl`); its fingerprint would be `8d0ff8bca7f0`, so a measurement
file showing that hash means the same position.

**Rule of thumb:** prose uses short names, tooling uses cell ids, rows record
the commit pins. A cell id is stable forever; a short name is stable as long as
the cell is in the set.

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §The metric, including §Canonical measurement protocol -->

## The metric

For one position (a *cell*) and one candidate configuration:

```
factor(cell) = median candidate wall [s] / median provari --guide hand wall [s]
```

- **provari `--guide hand` is 1.000 by definition.** It is the fair
  hand-evaluation-for-hand-evaluation column. The bundled NNUE/model column
  (`--guide model`) is *recorded separately and never mixed in* — it is a
  stretch column that needs an out-of-scope external artifact.
- **`< 1.0` means we are faster than provari on that cell.**
- **Ledger score = geometric mean of `factor(cell)` over the *canonical
  shared cells*** — cells where both sides prove a win inside the canonical
  budget. Censored cells are not part of the geometric mean (they have no
  wall time), they are counted in **coverage** instead.
- **Censoring convention.** A cell where we do not win inside the budget is
  reported as `censored`, and where a bound is meaningful it is printed as
  `≥ budget / reference_wall` (a *lower* bound on the factor, never a factor).
  A censored cell is never dropped and never silently converted into a win.
- **Rung order (D30).** Report the highest rung reached, then the next rung's
  gap. Coverage rungs (`T_1`–`T_4`) are necessary conditions; wall factors at
  10 s are claimed only at `T_5` on shared cells. Existing rows keep their own
  session and are not rewritten.
- **Deterministic accompaniment.** Wall time is the product metric but noisy;
  every row also records `child_evals` (ours, deterministic per stack) and
  provari's `nodes` for the same cell. They are **not** comparable
  one-to-one — our evals per expansion vs. provari's nodes is a diagnostic,
  not a ratio to optimise directly.

### Canonical measurement protocol

| Item | Value |
| --- | --- |
| Budget, both sides | 10 s, 192 MB label (`--seconds 10 --memory 192` vs `--timeout 10 --tt-size 192`) |
| Reference command | `provari prove "<FEN>" --seconds 10 --memory 192 --guide hand` |
| Candidate command | `atomic_solver --fen "<FEN>" --timeout 10 --tt-size 192 --first-outcome --outcome-only <stack flags>` |
| Repeats | 3 per cell per side (medians reported); one process at a time |
| Ordering | all reference runs and candidate runs of a row are measured in the same session, on the same host |
| Host | recorded per row; factors are same-session ratios, so a row is valid on its own host even when another row ran elsewhere |
| Oracle | `provari-v0.1.0`, rebuilt from source per row, never committed |
| Corpus | `corpus-v1` = [`../provari/measurements/plan1/corpus.json`](../provari/measurements/plan1/corpus.json), sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` |
| Tablebases | none |

The driver is [`measurements/bench/run_bench.py`](measurements/bench/run_bench.py)
(it writes [`measurements/bench/results.json`](measurements/bench/results.json)
with the per-run rows and the derived ledger). `--cases` takes **cell ids**;
`--stack` names a configuration:

```sh
python3 docs/plans/winprover/measurements/bench/run_bench.py \
  --provari /tmp/provari-bench/target/release/provari \
  --solver ./target/release/atomic_solver \
  --cases core,1eb80b92f92c,db6a741c723f --repeats 3 \
  --stack 'default=' \
  --stack 'opt=--move-class-pn --quiet-grouping --eval-pn or' \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --solver-commit "$(git rev-parse HEAD)"
```

The oracle is **rebuilt, never committed** (`provari-v0.1.0`, tag `v0.1.0`);
the hand-guide column needs only `cargo build --release` in a scratch clone,
the model column additionally needs provari's `data/fetch.sh` artifacts.

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §The bench set -->

## The bench set

Three cells, drawn from `corpus-v1` (`core` + the pinned `sample20`).

**Selection rule (fixed):** the bench set is every cell of `core + sample20`
where the pinned provari oracle proves a win **with the hand guide** inside
the canonical budget. Cells where provari itself is censored cannot carry a
factor and are therefore not bench cells, however interesting they are.

| Short name | Cell id | FEN | provari hand (1.000) | Why this cell |
| --- | --- | --- | --- | --- |
| `core` | `core` (hand-named; fingerprint `8d0ff8bca7f0`) | `r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1` | 0.491 s · 342,289 nodes · mate 17 | the initiative's motivating FEN; provari's hand guide is ~2× *better* here than `--guide none` |
| `easy-win` | `1eb80b92f92c` | `rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1` | 0.111 s · 72,523 nodes · mate 15 | the one easy shared win; provari's hand guide is >2× *worse* here than `--guide none` (the lever is case-dependent) |
| `deep-win` | `db6a741c723f` | `rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1` | 6.200 s · 4,574,912 nodes · mate 22 | the deepest hand-guide win in the corpus; the coverage cell (we do not solve it at 10 s) |

Cells measured and deliberately **out** of the set (recorded so the exclusion
is not silently revisited):

| Cell id | Why not a bench cell |
| --- | --- |
| `b9a7ed7eb368`, `dd3f89766a40` | provari **hand** is censored (10.1 M / 9.3 M nodes); only the out-of-scope bundled model wins them (provari plan1: 4.46 s and 1.86 s) |
| the remaining 16 `sample20` ids | provari's bundled model is censored on all of them, so the hand guide is expected to be censored too; item 1 confirms the hand-guide coverage of the whole corpus |

If item 1 finds an additional hand-guide win, that cell joins the set by the
rule above — it gets a short name, its reference values are measured, and the
ledger gains a new reference row. Earlier rows are not rewritten.

**2026-11-02 — item 1 hand census (re-derived, no set change).** The whole
`core + sample20` corpus was re-measured with a freshly rebuilt
`provari-v0.1.0` (`--features bundled`; the hand guide is byte-for-byte the
same search) at the canonical 10 s / 192 MB budget, both guides side by side.
Result: provari `--guide hand` wins **exactly the three cells above and no
others** (`core` plus 2/20 `sample20`), so the selection rule confirms the
set — no additions, no removals. Re-derived reference medians (3 repeats,
host `67db7a1089e2`, aarch64 8 CPU): `core` 0.490 s · 342,289 nodes ·
mate 17; `easy-win` 0.111 s · 72,523 nodes · mate 15; `deep-win` 6.240 s ·
4,574,912 nodes · mate 22. Coverage: hand 3/21 (2/20 on `sample20`), the
stretch `--guide model` column 5/21 (4/20 — it adds `b9a7ed7eb368` 4.530 s
and `dd3f89766a40` 1.853 s, the model-only wins already listed above), our
`default` 1/21, our `opt` stack 2/21. Shared-cell factors in the same
session: `hand×default` 51.739 (1 cell), `hand×opt` 6.848 (2 cells). No
ledger row is appended: plan1 does not touch the win path. Details in
[`measurements/plan1/`](measurements/plan1/README.md) and
[`report1.md`](report1.md).

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §Reading rules -->

## Reading rules (do not violate these when appending a row)

1. **Same session, same host.** A factor compares two walls measured back to
   back on one host. Cross-host precision is ~±15–20 %: the reference wall for
   `core` has measured 0.403 s (plan-session probe, x86 32-vCPU host), 0.518 s
   (provari plan1 driver, same host) and 0.491 s (this host), and our own
   per-eval throughput differed by ~40 % between the x86 host and this one.
   Do not compare a new factor against an old row and call the difference a
   lever effect — re-measure, or state the drift.
2. **Wall is the product metric; `child_evals` is the mechanism metric.** A
   child-eval reduction that does not move the wall is a mechanism finding,
   not a speed win (and vice versa: a wall win with identical evals is a
   per-node cost effect, `performance`'s territory).
3. **Censored is a result.** `≥ budget/reference` bounds and coverage counts
   are the honest form; a censored cell may never be replaced by the wall of
   a longer run without stating the budget used (as done for `deep-win`
   above).
4. **Opt-in ≠ shipped.** The ledger's `Stack` column is part of the result.
   Only a default-flag decision (item 6) turns a stack row into a product
   row; until then both stack rows stay visible.
5. **Correctness gates precede the number.** A row is only written after the
   plan's gate passed (`make test`, plus quick suite / repetition soundness /
   drift protocol for behaviour-changing levers) and, for new proof shapes,
   after the item-2 checker accepted the exported proof.
6. **Coverage is a first-class target.** A geomean below 1.0 on two cells
   while the third stays censored does not meet the goal. State coverage in
   every row. From D30, report the highest rung reached; wall factors are
   claimed only at `T_5`.

## Ledger

**Convention (D30, amended 2026-11-11).** A factor carries **no budget**: it
is `candidate wall / provari hand wall`, and provari's side is fixed — the
bench set is selected so it proves with the hand guide inside 10 s (`core`
0.49 s, `easy-win` 0.11 s, `deep-win` 6.5 s), so a longer run cannot change
the reference. A time budget has exactly two roles: (a) the **economic cap**
at which an unfinished run is reported `censored (≥ cap/ref)` — 120 s today;
(b) the **`T_5` goal threshold**, 10 s. The ledger is therefore a single
**Factors** table (no per-cell budget); the 10 s goal and the rung status
live in the goal section above and the plan notes, not in a ledger table.
Existing rows are not rewritten; rows measured under the old 10 s canonical
protocol keep their session numbers, and the `Cap` column names the censoring
cap only. Decisions append no row (history D27–D30 in
[`decisions.md`](decisions.md)).

### Factors

| Plan | Date | Host · pins | Stack | `core` | `easy-win` | `deep-win` | Geomean | Cap |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `plan1` | 2026-11-06 | `ae54d05b36f8` · `atomi@0c5a882+d` | atomi **m0** | censored (≥ 18.76) 59.5 M ev | **36.365** (4.982 s / 30.2 M) | censored (≥ 1.52) 61.8 M ev | **36.365** (1 cell) | 10 s |
| `plan2` | 2026-11-06 | `ae54d05b36f8` · `atomi@a4c135d+d` | atomi **m1** | censored (≥ 18.9) 181.8 M ev | **10.897** (1.482 s / 30.3 M) | censored (≥ 1.48) 186.6 M ev | **10.897** (1 cell) | 10 s |
| `plan11` | 2026-11-10 | `e10de197da58` · `atomi@b52e186+d` | atomi **bare** | **35.3** (17.390 s) | **19.1** (2.008 s) | censored (1/3; seed 1 111.218 s) | **25.9** (2 cells) | 120 s |
| `plan15` | 2026-11-10 | `e10de197da58` · `atomi@8953596+d` | atomi **bare** | **35.8** (17.471 s) | **18.9** (2.023 s) | censored (1/3; seed 1 113.458 s) | **26.0** (2 cells) | 120 s |
| `plan19` | 2026-11-11 | `e10de197da58` · `atomi@e71289b+d` | atomi **default** (R1+R3+R6) | censored (2/3; 115.1 s) | **78** (10.4 s) | censored (0/3; ≥ 18.4) | **78** (1 cell) | 120 s |
| `plan20` | 2026-11-12 | `e10de197da58` · `atomi@772b3cb+d` | atomi **default** (AUX-V1) | **16.3** (8.36 s) | **7.0** (0.91 s) | censored (0/3; ≥ 18.5) | **10.8** (2 cells) | 120 s |

Cell syntax: `factor (candidate wall / candidate evals)`; atomi's evals are
its `child_evals` counter. Reference (`provari-v0.1.0 --guide hand`) is
**1.000** everywhere. `Cap` is the censoring cap only — the factor itself is
budget-free. A cell proved only on some seeds is reported as
`censored (n/3; …)`, never averaged as if it were 3/3.

**At the same 120 s cap the plan19 default is slower than the bare tree on
every cell** (`easy-win` 78 vs 18.9; `core` 2/3 at 115–118 s vs 3/3 at
17.5 s): the tree cut does not pay for the auxiliary passes (report19 §5).
Whether `--guide hand` should stay the default is an open owner decision, not
a ledger fact.

**Rung (D30).** Highest rung is **`T_2`**: the bare tree proves `core` and
`easy-win` at 120 s (plan11/plan15). `T_5` is not met; its 10 s coverage and
geomeans are in the plan notes (plan15/plan19 Tier 2).

**One engine per table (D10 amendment, 2026-11-11).** Only `atomi@*` rows
form a series. The legacy `solver@5f85133` controls measured in plan1
(`default` 45.358, 1/3; `opt` 6.322, 2/3) are a frozen one-session baseline:
a different engine, host and session, recorded in the [plan1 notes](#plan1)
and [`measurements/plan1/`](measurements/plan1/README.md), never ordered as
if they were `atomi` rows (reading rule 1). They were removed from the table
on 2026-11-11 because a mixed series invites exactly that comparison; the
measurement record is unchanged.

Pins: `atomi@0c5a882+d` = HEAD `0c5a882ef9ab93c0d48d6a3f5176cfb64a0a78bb`
(plan1, dirty tree) · `atomi@a4c135d+d` = HEAD
`a4c135d6ee1956588e01733bd101f6373e7d2e72` (plan2, dirty tree) ·
`atomi@b52e186+d` = HEAD `b52e186…` (plan11, dirty tree; binary sha256
`e416f075…12c811`) · `atomi@e71289b+d` = HEAD
`e71289b630f87137774810403492eedba10d22be`
(plan19, dirty tree; binary sha256 `bf98c6df…`) · `atomi@8953596+d` = HEAD
`89535961ebdd73e78d7b2267177da1eb87bc3216` (plan15, dirty tree; binary
sha256 `b28bdb1e…62ba71f`) · `solver@5f85133` =
`5f85133ea9d751368a682e53e354bd3fc0016d00` (the frozen legacy control of the
[plan1 notes](#plan1), not a ledger row) · `provari-v0.1.0` =
`48686bc6c786d6ea70568f0b2ad4002cf0520206` (the 1.000 side of every factor).

## Plan notes

### plan1

Numbers: [`measurements/plan1/bench.json`](measurements/plan1/bench.json);
per-node baseline: [`measurements/plan1/pernode.json`](measurements/plan1/pernode.json).

- **Re-priced controls (post-`atomic-movegen` 2.4.0, this host, same
  session).** Legacy `default`: **45.358** on one shared cell (`easy-win`),
  coverage 1/3. Legacy `opt`: **6.322** on two shared cells (`core` 7.43,
  `easy-win` 5.38), coverage 2/3. Both are censored on `deep-win`. This is
  the initiative's re-measured baseline gap, kept here as the frozen record
  (the legacy controls are not ledger rows; D10 amendment): a different
  engine, host and session, never ordered against `atomi` rows (reading
  rule 1).
- **atomi M0: no performance claim.** Heuristic-free DF-PN (uniform starting
  numbers, movegen order, no guide). It wins `easy-win` at **36.365**
  (4.982 s, 30,245,030 child evals; the D14 proof run was accepted by
  `atomi validate` and `provari check`, 2,083 positions, length 17) and is
  censored on `core` (≥ 18.76) and `deep-win` (≥ 1.52). It is the
  correctness floor and the M1 per-node baseline, not a product row.
- **Moving target (stretch, D17).** provari v0.2.0 hand / v0.1.0 hand:
  0.987 / 1.007 / 1.010, geomean **1.001** on 3 cells. At its default single
  thread, v0.2.0 reproduces v0.1.0's node counts exactly on all three cells
  (342,289 / 72,523 / 4,574,912), so the ratio is wall noise around an
  unchanged search trajectory. Its `--threads N` option was not measured
  (not part of the protocol). Kept out of the goal geomean.
- **Per-node baseline (D11).** Median **162.0 ms per 1 M child evals** on
  `core` at `--evals 10000000` (≈ 5.18 µs per expansion, ≈ 32 child evals
  per expansion, peak RSS ≈ 131 MiB under the 192 MB label: the M0 table
  rounds its bucket count down to a power of two). This is M1's reference
  number.

### plan2

Numbers: [`measurements/plan2/bench.json`](measurements/plan2/bench.json);
per-node: [`measurements/plan2/pernode.json`](measurements/plan2/pernode.json).
Pin: `atomi@a4c135d+d` = HEAD `a4c135d6ee1956588e01733bd101f6373e7d2e72` on
a dirty tree (plan2 work uncommitted), binary sha256 `2008857f…2847ab`.

- **Session reference line** (v0.1.0 hand medians, 3 repeats): `core`
  0.529 s · 342,289 nodes · mate 17; `easy-win` 0.136 s · 72,523 · mate 15;
  `deep-win` 6.767 s · 4,574,912 · mate 22.
- **atomi m1: no goal claim.** Same heuristic-free search as m0 (uniform
  numbers, movegen order); M1 changed only the per-node cost. `easy-win`
  **10.897** (1.482 s, 30,335,618 child evals vs m0's 30,245,030); the D14
  proof run had identical `child_evals`, wrote 2,048 positions (length 17)
  and was accepted by `atomi validate` and `provari check`. `core` and
  `deep-win` stay censored (≥ 18.9 / ≥ 1.48), now at ≈ 18 M child evals
  per second instead of ≈ 6 M.
- **Per-node (M1 gate, same session).** `core` at 10 M, 192 MB: m1
  **53.4 ms per 1 M child evals**, m0 166.7, legacy front-end 137.3
  (fixed 32 MB table). This is M2's per-node reference.
- **Stretch.** v0.2.0 / v0.1.0 hand = 1.040 / 0.985 / 1.001, geomean
  1.008: wall noise around unchanged single-thread node counts.
- Legacy controls were not re-run (plan1's rows stand; their factors are
  ratios of their own session).

### plan3 (no row: gate failed)

Plan3's hand-guide stack did **not** meet the D21 gate, so by the plan's rule
no ledger row is appended (`measurements/plan3/guidance.json`). For the
record: on the frozen 24-cell `guidance-v1` set at `--evals 20000000`,
`easy-win` proves on all four seeds in 211,254–631,763 expansions (limit
145,046) and `core` is censored on all seeds; at the 10 s canonical budget
both are censored. `hand` coverage is 19/24 versus provari hand 24/24; the
18 `wins.tsv`-derived cells are proved on every seed. Full details in
[`report3.md`](report3.md).

### plan4 (no row: G1 failed)

Plan4 CORE-RESET did not meet G1 (`core` proved at 10 s), so no ledger row
is appended (`measurements/plan4/gate.json`). The calibrated point
(scale 64, EF-PN weight 0) proves `easy-win` at 10 s on seeds 0–2
(121,866 / 110,533 / 115,147 expansions) and is censored on `core`
(146,432 / 147,456 / 148,480 expansions at 10 s). A per-frame auxiliary
budget proves `core` at 120 s (278.5 M child evals, length 20) with an
accepted proof; that is the M4-first pivot. Item 3 is closed. Details in
[`report4.md`](report4.md).

### plan15

Tier 1 (120 s, not a wall row): `core` 3/3 at 17.471 s (35.8×), `easy-win`
3/3 at 2.023 s (18.9×), `deep-win` 1/3 (seed 1 113.458 s, seeds 0/2 censor
at 120 s); 7/7 D14 proofs accepted. Tier 1 is **2/3, pass** under D28. The
ledger row is the Tier 2 10 s head-to-head; coverage 1/3 and geomean 15.757,
so Tier 2 **fails**. Legacy `default`/`opt` and provari v0.2.0 controls were
**unavailable** and not substituted.

Session reference line (v0.1.0 hand medians, 3 repeats): `core` 0.533 s ·
342,289 nodes · mate 17; `easy-win` (`1eb80b92f92c`) 0.137 s · 72,523 nodes ·
mate 15; `deep-win` (`db6a741c723f`) 6.567 s · 4,574,912 nodes · mate 22.

Proof-check (D14): the `easy-win` atomi cell re-ran at `--seconds 600` with
identical `child_evals` (30,245,030), `proof` written (2,083 positions,
length 17), `atomi validate` valid, `provari check` valid and positions ==
won_positions: `proof_verdict = accepted`. Censored cells needed no proof run.

### plan19

The default's 120 s coverage (item 26) is `core` **2/3** (118.0 / 115.1 s,
seed 2 censored), `easy-win` **3/3** (7.6 / 16.8 / 10.4 s) and `deep-win`
**0/3**; the product hand was `core` 0/3, `easy-win` 3/3 (22.0 / 29.8 /
10.6 s), `deep-win` 0/3. Tree-only the default cuts `core` 4.74×, `easy-win`
3.34× and `deep-win` 2.80× against the product hand (medians of seeds 0–2,
2 G child evals). The wall is now auxiliary-bound: `core` needs 84.8 M child
evals tree-only (14 s) and ~115 s with the passes on. D14 16/16 accepted. No
rung change; `T_2` still rests on the bare tree's `core` 3/3. Numbers:
[`measurements/plan19/`](measurements/plan19/README.md).

## History

- **2026-11-10** — plan15 (items 5 and 6) executed. Item 5 (M4) closed as
  superseded. Item 6 (M5) appends the first ledger row since plan2: Tier 1
  **2/3 pass** at 120 s (`core` 3/3 17.471 s / 35.8×, `easy-win` 3/3 2.023 s
  / 18.9×, `deep-win` 1/3 seed 1 113.458 s; 7/7 D14 accepted); Tier 2
  **fail** at 10 s (coverage 1/3, shared-cell geomean 15.757; `core`
  ≥ 20.49×, `deep-win` ≥ 1.549× censored). Default stays `--guide none`;
  legacy/v0.2.0 controls unavailable. The stale Tier 2 coverage clause is
  recorded as an open goal-definition issue. Details in
  [`report15.md`](report15.md).
- **2026-11-06** — `bench.md` opened for the `prover` initiative: the
  verbatim sections above are inherited from the closed `winprover`
  ledger (provenance comments inline), the successor notes translate its
  conventions, and plan1 appends the first row: m0 as the no-claim
  baseline, the legacy default/opt controls re-priced under
  atomic-movegen 2.4.0, and the v0.2.0 stretch block.
- **2026-11-06** — plan2 (M1, per-node cost) appends the atomi **m1** row
  (`easy-win` 10.897, coverage 1/3, no goal claim) and its stretch block;
  per-node `core`/10 M/192 MB is 53.4 ms per 1 M child evals (m0 166.7,
  legacy front-end 137.3, same session).
- **2026-11-09** — plan3 (M2+M3 hand-guide stack) executed and closed with a
  **failed D21 gate**: `easy-win` 211,254–631,763 expansions (limit
  145,046) and `core` censored at 20 M child evals and at 10 s; coverage
  19/24. No ledger row. The stack is recorded for the D21 kill review;
  `--guide none --seed 0` remains counter-identical to m1.
- **2026-11-09** — plan4 CORE-RESET executed; **G1 failed on `core` at
  10 s**, item 3 closed. Calibration proves `easy-win` at 10 s on seeds
  0–2 (121,866 / 110,533 / 115,147, under 145,046) but `core` stays
  censored. The auxiliary-budget configuration proves `core` at 120 s
  (278.5 M child evals, both checkers accept the 14,332-position proof);
  no ledger row.
- **2026-11-09** — plan7 (item 16) executed; **G3 wall gate failed**, so no
  ledger row. Attribution: simulation is 64 % of the calibrated easy-win
  wall. The deterministic `--guide-aux-budget` (best easy-win 3.891 s) and
  opt-in `--guide-sim-plies` (2 plies 4.196 s) do not close the gap; no-sim
  is 2.096 s and bare 1.965 s. `core`/`deep-win` censored at 10 s.
  Coverage 1/3. Details in [`report7.md`](report7.md).
- **2026-11-09** — plan session 9 / D27: goal re-pinned coverage-first
  (Tier 1 3/3 at 120 s with D14; Tier 2 original 10 s geomean on shared
  cells). Plan8's measured distance (easy-win 58.4×, core ≥ 121.5×, deep-win
  ≥ 9.36×, coverage 1/3) stands; no ledger row, no factor published until
  Tier 1 passes. Plan: [`plan9.md`](plan9.md).
- **2026-11-10** — plan9 (item 19) executed; **Tier 1 failed at 1/3** at
  120 s / 192 MB. `easy-win` 3/3 (5.765 s, 54.9×, D14 accepted on seeds
  0–2); `core` 0/3 (≥ 240.0×) and `deep-win` 0/3 (≥ 18.2×) censored. Tier 2
  not claimed; `REPLAY-BOUND` not started. No ledger row. Details in
  [`report9.md`](report9.md).
- **2026-11-10** — plan10 (item 18) executed; **Phase A stop rule fired**.
  Fixed 20 M evals, product point: `core` 41.781 s, `deep-win` 38.560 s;
  both-off (sim+closures) 3.160 s / 2.533 s; auxiliary share **92.44% /
  93.43%**. The only material lever is the closed auxiliary line, so
  `RESTART-GEN` was not implemented and Phase C/D were not run. No coverage
  claim, no ledger row. Details in [`report10.md`](report10.md).
- **2026-11-10** — plan12 (item 21) executed; **Phase A stop rule fired**.
  Bare tree, `deep-win` seed 0: at 20 M evals 668,608 expansions / 620,162
  distinct keys (7.25 % re-expansions) and only **53 stored values
  (0.0079 %)** crossing the incoming node thresholds; at 120 s 51.7 M
  expansions / 2.75 M proof entries, 15,849 crossing values (0.0319 % of the
  `--stats` run). ENTRY-REUSE was not implemented and the 120 s matrix/D14
  were not run; Tier 1 stays 2/3 (plan11) and no ledger row is appended.
  The `--stats` counters are default-off and observation-only. Details in
  [`report12.md`](report12.md).
- **2026-11-10** — plan13 (item 22) executed; **all three cheap probes
  fail**. Phase A proof-share 25/40/50/60 at 120 s: no share proves
  `deep-win` 3/3 (1/3 at 25 and 50, 0/3 at 40/60); `core`/`easy-win` stay
  3/3 and the 10 % wall guard passes, but the 3/3 clause does not. Phase B
  `--max-depth` 16/20/24/28/32 at 20 M: 20–32 are inert, cap 16 cuts
  `deep-win` unique keys 2.18× but censors at depth with no proof. Phase C
  conformance stack: best `core` cut **1.22×** (`lo_fresh` 480,962 vs bare
  586,075), full stack 1.11×; the full stack cuts `easy-win` 3.2× (146,276
  vs hand 468,301) but `core` stays censored. The hand-guide line is closed
  for good; Tier 1 stays 2/3 and no ledger row is appended. Details in
  [`report13.md`](report13.md).
- **2026-11-10** — plan14 (item 23) executed; **owner accepts option A**.
  The kill review found no non-hand mechanism with a credible 2× case (120 s
  `deep-win` tree: 50.0 M expansions / 40.7 M unique keys, 18.6 %
  re-expansions, `proof_full: false`, ~8.9× provari's 4.57 M nodes). **D28**
  amends D27: Tier 1 is **2/3** (`core`, `easy-win`) at 120 s / 192 MB;
  `deep-win` is recorded as 1/3 and is not a pass condition. The 120 s budget
  is not raised, the hand-guide/auxiliary line stays closed, no portfolio is
  built, and the bare tree is the product candidate. No ledger row, no engine
  change. Details in [`plan14.md`](plan14.md).
- **2026-11-11** — ledger cleanup, no measurement: the two legacy
  `solver@5f85133` rows (`default` 45.358, `opt` 6.322) are removed from the
  ledger table. They are a frozen one-session plan1 baseline of a different
  engine, host and session and were never comparable to `atomi` rows
  (reading rule 1); they remain in the [plan1 notes](#plan1), the pins and
  [`measurements/plan1/`](measurements/plan1/README.md). The table now holds
  `atomi` rows only. No factor, pin or measurement changes; **D10 amended**
  (see [`decisions.md`](decisions.md)).
- **2026-11-11** — ledger split, no measurement: the single table becomes
  **Factors** (budget-free). A factor now carries no budget; a time budget
  survives only as the 120 s censoring cap (a `Cap` column) and as the 10 s
  `T_5` threshold stated in the goal section. The plan15/plan19 10 s results
  were moved to a short `T_5` gate table, which was then removed on the same
  day (the goal section and plan notes carry the 10 s status; the Factors
  table is sufficient). The 120 s factors join the Factors table
  (plan11/plan15 bare 35.3/19.1 and 35.8/18.9; plan19 default 78 on
  `easy-win`). The split makes the plan19 default's 120 s regression vs the
  bare tree explicit (78 vs 18.9). No engine change.

- **2026-11-12** — plan20 (items 27–28) executed; **the default is now
  AUX-V1**: `sim-exact` + `null-exact` + the board-free `closure-bits`
  closures on top of the plan-19 numbers. Tree-only medians vs the plan-19
  default: `core` 5.88×, `easy-win` 4.65×, `deep-win` 1.66× (censored control)
  in child evals; 120 s wall 14.4× `core` (2/3 → 3/3 at 8.3 s) and 10.9×
  `easy-win` (3/3 at 0.94 s). The `aux_nodes`/expansion ≤ 1 gate is a miss
  (7–13, from 213–215); exact replays are per-defence work. 10 s: `core` 2/3,
  `easy-win` 3/3, `deep-win` 0/3 → factors 16.3 / 7.0 / censored. `deep-win`
  proves at 128–175 s with a generous cap but is 0/3 at 120 s: no `T_3`, `T_2`
  stands 3/3 + 3/3. `proof_full` appeared on `deep-win` (item 30 trigger).
  The control is `--guide hand --guide-off sim-exact,null-exact,closure-bits`.
  Details in [`report20.md`](report20.md) and
  [`measurements/plan20/`](measurements/plan20/README.md).
