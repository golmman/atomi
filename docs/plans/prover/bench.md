# Bench — performance ledger vs. provari

Status: **standing document of the [`prover`](initiative.md) initiative** (the
successor of `winprover`). It fixes the benchmark set, the metric, and one
*appended* row per executed plan, so the performance leg has a single,
comparable history. It is not a plan and not a report: plans live in
`planN.md`, their evidence in `reportN.md` / `measurements/planN/`.

## Successor reading notes

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

The initiative's success condition, quoted verbatim from
[`initiative.md`](initiative.md):

- on the shared-win set, the geometric mean of `atomi / provari --guide hand`
  wall time is `< 1.0`;
- coverage is at least provari hand's (currently 3/3 on the bench cells);
- every claimed win is accepted by this repo's own replay validator **and** by
  the pinned `provari check`;
- this repo's gates are green, and there are no outcome regressions on the
  copied fixture oracle (§Corpus and fixtures). The legacy solver's gates are
  not this repo's gates.

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
[`report8.md`](report8.md) and [`measurements/plan7/`](measurements/plan7/README.md).

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
   every row.

## Ledger

**Pins.** `atomi@0c5a882+d` = atomi built from HEAD `0c5a882ef9ab93c0d48d6a3f5176cfb64a0a78bb`
on a dirty tree (uncommitted work-in-progress) · `atomi@a4c135d+d` = atomi
built from HEAD `a4c135d6ee1956588e01733bd101f6373e7d2e72` on a dirty tree
(plan2 work uncommitted) · `solver@5f85133` =
`5f85133ea9d751368a682e53e354bd3fc0016d00` · `provari-v0.1.0` =
`48686bc6c786d6ea70568f0b2ad4002cf0520206` (the 1.000 side of every factor).

| Plan | Date | Host · pins | Stack | `core` | `easy-win` | `deep-win` | Geomean | Coverage |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `plan1` | 2026-11-06 | `ae54d05b36f8` · `atomi@0c5a882+d` | atomi **m0** | censored (≥ 18.76) 10.061 s / 59.5 M | **36.365** (4.982 s / 30.2 M) | censored (≥ 1.52) 10.061 s / 61.8 M | **36.365** (1 cell) | 1/3 |
| `plan1` | 2026-11-06 | `ae54d05b36f8` · `solver@5f85133` | legacy **default** | censored (≥ 18.76) 10.020 s / 66.2 M | **45.358** (6.214 s / 43.3 M) | censored (≥ 1.52) 10.019 s / 67.6 M | **45.358** (1 cell) | 1/3 |
| `plan1` | 2026-11-06 | `ae54d05b36f8` · `solver@5f85133` | legacy **opt** | **7.43** (3.960 s / 17.2 M) | **5.38** (0.737 s / 3.23 M) | censored (≥ 1.52) 10.019 s / 46.2 M | **6.322** (2 cells) | 2/3 |
| `plan1` (stretch) | 2026-11-06 | `ae54d05b36f8` · `provari-v0.2.0@224ce85` | v0.2.0 **hand** | 0.987× | 1.007× | 1.010× | **1.001** (3 cells, stretch block only) | 3/3 |
| `plan2` | 2026-11-06 | `ae54d05b36f8` · `atomi@a4c135d+d` | atomi **m1** | censored (≥ 18.9) 10.003 s / 181.8 M | **10.897** (1.482 s / 30.3 M) | censored (≥ 1.48) 10.003 s / 186.6 M | **10.897** (1 cell) | 1/3 |
| `plan2` (stretch) | 2026-11-06 | `ae54d05b36f8` · `provari-v0.2.0@224ce85` | v0.2.0 **hand** | 1.040× | 0.985× | 1.001× | **1.008** (3 cells, stretch block only) | 3/3 |

Cell syntax: `factor (candidate wall / candidate evals)`; legacy evals are
its `evals:` counter, atomi's are `child_evals`. Reference
(`provari-v0.1.0 --guide hand`) is **1.000** everywhere.

Session reference line (v0.1.0 hand medians, 3 repeats): `core` 0.533 s ·
342,289 nodes · mate 17; `easy-win` (`1eb80b92f92c`) 0.137 s · 72,523
nodes · mate 15; `deep-win` (`db6a741c723f`) 6.567 s · 4,574,912 nodes ·
mate 22.

Proof-check (D14): the `easy-win` atomi cell re-ran at `--seconds 600`
with identical `child_evals` (30,245,030), `proof` written (2,083
positions, length 17), `atomi validate` valid, `provari check` valid and
positions == won_positions: `proof_verdict = accepted`. Censored cells
needed no proof run.

### plan1 summary

Numbers: [`measurements/plan1/bench.json`](measurements/plan1/bench.json);
per-node baseline: [`measurements/plan1/pernode.json`](measurements/plan1/pernode.json).

- **Re-priced controls (post-`atomic-movegen` 2.4.0, this host, same
  session).** Legacy `default`: **45.358** on one shared cell (`easy-win`),
  coverage 1/3. Legacy `opt`: **6.322** on two shared cells (`core` 7.43,
  `easy-win` 5.38), coverage 2/3. Both are censored on `deep-win`. This is
  the initiative's re-measured baseline gap. It is not compared with the
  frozen legacy rows (reading rule 1).
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

### plan2 summary

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

### plan3 (no ledger row — gate failed)

Plan3's hand-guide stack did **not** meet the D21 gate, so by the plan's rule
no ledger row is appended (`measurements/plan3/guidance.json`). For the
record: on the frozen 24-cell `guidance-v1` set at `--evals 20000000`,
`easy-win` proves on all four seeds in 211,254–631,763 expansions (limit
145,046) and `core` is censored on all seeds; at the 10 s canonical budget
both are censored. `hand` coverage is 19/24 versus provari hand 24/24; the
18 `wins.tsv`-derived cells are proved on every seed. Full details in
[`report3.md`](report3.md).

### plan4 (no ledger row — G1 failed)

Plan4 CORE-RESET did not meet G1 (`core` proved at 10 s), so no ledger row
is appended (`measurements/plan4/gate.json`). The calibrated point
(scale 64, EF-PN weight 0) proves `easy-win` at 10 s on seeds 0–2
(121,866 / 110,533 / 115,147 expansions) and is censored on `core`
(146,432 / 147,456 / 148,480 expansions at 10 s). A per-frame auxiliary
budget proves `core` at 120 s (278.5 M child evals, length 20) with an
accepted proof; that is the M4-first pivot. Item 3 is closed. Details in
[`report4.md`](report4.md).

## History

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
  Coverage 1/3. Details in [`report8.md`](report8.md).
