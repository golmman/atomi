# Prover Report 7: item 16 — the wall premise survives, the wall gate does not

Plan: [`plan7.md`](plan7.md) (item 16, wall attribution and a deterministic
auxiliary budget). Executed 2026-11-09. **G3 fails**; B1/B2 landed behind
opt-in flags, defaults unchanged. The plan's stop rule applies: record the
result and take the continue / re-pin / close decision at the kill review
(D24). No ledger row and no goal claim follow.

## Headline

- **The auxiliary wall premise survives.** On `easy-win`, removing sibling
  simulation alone cuts the calibrated-hand wall from a 5.829 s median to
  2.096 s (**64 %** of the wall). The plan's `hand` vs `no sim+closures`
  decision rule reads 0.275 on seed 0 (just under 0.3) but 0.375 on the
  three-seed median (0.275 / 0.408 / 0.330), so it passes by the median
  reading. The `neither` control is confounded: removing closures grows the
  tree and adds wall.
- **A global auxiliary budget is deterministic and works, but is not a wall
  lever of the needed size.** `--guide-aux-budget` caps the global auxiliary
  work, records `aux_nodes`/`aux_skipped`, and is identical on repeated
  fixed-eval runs. Its best `easy-win` wall is ≈ 3.8–3.9 s at 10 k–100 k
  units (vs 5.8 s unlimited), because the guide starves and the tree grows
  ~8×. `core` is censored at every budget.
- **Shorter playouts help, no simulation helps more.** `--guide-sim-plies`
  6 → 2 gives 5.777 → 4.196 s; `--guide-off sim` gives **2.073 s**. Still
  4.1× above the 0.5 s gate.
- **G3 fails decisively.** Chosen configuration (`aux_budget=100000`):
  `easy-win` 3.711–4.162 s (26.7–29.9× the reference wall), `core` censored
  on all seeds, `deep-win` censored for search at 10 s. The 0.5 s gate is
  below even the bare 10 s tree's 1.97 s.
- **Soundness holds.** `guidance-v1` seed 0: 0 refutations; 17 D14 proofs,
  all accepted by `atomi validate` and `provari check`. `make test-full`
  (incl. veto) and `make lint` are green.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `dbfae46` on a dirty tree (plan7 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `e9587a6f…c46ec` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, 24 cells, sha256 `2da2a65b…8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Product point | `--guide hand --guide-scale 64 --guide-ef-weight 0` (plan4) |

The attribution/cheaper/budget rows predate the B1 output change, so their
binary sha is not recorded separately; B1 leaves the default trajectory
unchanged (`aux_budget=0`, `sim_plies=6`, easy-win expansions 121,866 /
110,533 / 115,147, as report4). Wall, soundness and env rows use the final
binary. Commands: the six driver phases in
[`measurements/plan7/README.md`](measurements/plan7/README.md), then
`make test-full` and `make lint`, one process at a time.

## Phase A — attribution

Medians over seeds 0–2 (deep-win seed 0 only). `aux = sim_nodes + settle_nodes`.

| Stack | easy-win 10 s wall · exp · aux | core 10 s wall · exp · aux |
| --- | --- | --- |
| bare | 1.965 s · 1.24 M · 0 | 10.003 s · 5.38 M · 0 |
| hand | **5.829 s** · 115 k · 21.7 M | 10.037 s · 148 k · 33.7 M |
| no closures | 8.549 s · 744 k · 20.8 M | 10.010 s · 584 k · 25.7 M |
| no sim | **2.096 s** · 132 k · 6.7 M | 10.009 s · 486 k · 28.2 M |
| neither | 3.905 s · 993 k · 0 | 10.005 s · 2.10 M · 0 |
| aux 1/1 | 4.587 s · 773 k · 5.5 M | 10.009 s · 1.48 M · 10.5 M |
| sim only 1 | 2.056 s · 117 k · 6.7 M | 10.014 s · 446 k · 28.5 M |
| settle only 1 | 10.004 s · 767 k · 26.2 M | 10.009 s · 537 k · 26.5 M |

Fixed-10 M easy-win walls: bare 0.554 s (321 k exp), hand 5.778 s (115 k),
no-sim 2.105 s (132 k), neither 1.280 s (313 k). Core censors at 10 s
everywhere; fixed-10 M core runs are in `attribution.json`.

**Decision rule.** Seed 0: (6.051 − 4.388)/6.051 = **0.275**. Seeds:
0.275 / 0.408 / 0.330; median **0.330 ≥ 0.3**. The no-sim-only share is
0.640. The premise is not false; the literal seed-0 proxy is borderline
because `neither` grows the tree. This is recorded rather than hidden.

## Phase B — deterministic budget and cheaper paths

Engine changes (all opt-in, defaults unchanged):

- `GuideParams.aux_budget: u64` (`--guide-aux-budget`, `0` = unlimited). The
  search-global counter admits optional settle/simulation work until the
  budget is exhausted, then skips it; the last admitted operation may
  overshoot by its own work. `aux_nodes` and `aux_skipped` are reported.
  No wall clock is read. The output contract gained the two diagnostic keys.
- `GuideParams.sim_plies` (`--guide-sim-plies`, default 6). `playout_with`
  bounds the playout; `simulate::playout` keeps the plan3 default.
- The existing "skip playout for a statically settled child" behaviour was
  already true (`sim` runs only on `Value::Open`); no code change was needed.
  The "second legal movegen in the settle" path was not changed because the
  attribution prices simulation, not the settle, as the dominant optional
  cost, and the settle is the tree lever (removing it regresses easy-win).

Cheaper-aux leave-one-out (canonical 10 s, medians over seeds 0–2):

| Variant | easy-win wall | expansions | aux |
| --- | ---: | ---: | ---: |
| sim 6 (default) | 5.777 s | 115,147 | 21.7 M |
| sim 4 | 4.609 s | 113,080 | 16.3 M |
| sim 3 | 4.327 s | 118,650 | 14.5 M |
| sim 2 | 4.196 s | 121,224 | 12.3 M |
| no sim | **2.073 s** | 131,645 | 6.7 M |

Budget grid (canonical 10 s, medians over seeds 0–2):

| Budget | easy-win wall · exp · proved | core wall · exp · proved |
| --- | --- | --- |
| 10 k | 3.848 s · 915 k · 3/3 | 10.008 s · 2.01 M · 0/3 |
| 100 k | 3.891 s · 923 k · 3/3 | 10.005 s · 2.00 M · 0/3 |
| 1 M | 4.008 s · 885 k · 3/3 | 10.005 s · 1.95 M · 0/3 |
| 10 M | 4.386 s · 456 k · 3/3 | 10.007 s · 1.42 M · 0/3 |
| unlimited | 5.791 s · 115 k · 3/3 | 10.016 s · 147 k · 0/3 |

The budget starves the guide early, so the tree grows and the wall gain is
only ~1.5×. A fixed-eval determinism check at
`--evals 10000000 --guide-aux-budget 100000` is identical on repeat for both
cells (exit, length, evals, expansions, aux_nodes, aux_skipped); the
canonical 10 s `core` repeats differ only because the wall censors different
amounts of completed work, not because of nondeterminism.

## Phase C — wall gate and classification

Chosen point: calibrated hand + `--guide-aux-budget 100000` (best easy-win
median). Canonical 10 s / 192 MB:

| Cell | seed | exit | exp | evals | wall | factor vs reference | D14 |
| --- | ---: | --- | ---: | ---: | ---: | ---: | --- |
| easy-win | 0 | proved | 922,854 | 27.4 M | 3.841 s | 27.6× | accepted |
| easy-win | 1 | proved | 992,315 | 29.1 M | 4.162 s | 29.9× | accepted |
| easy-win | 2 | proved | 887,174 | 26.0 M | 3.711 s | 26.7× | accepted |
| core | 0–2 | timeout | 1.98–2.04 M | 60.9–62.3 M | 10.00 s | ≥ 19.4× | — |
| deep-win | 0 | timeout | 2.40 M | 71.1 M | 10.004 s | ≥ 1.55× | — |

Reference walls are `guidance-v1.tsv` (provari hand 0.139 / 0.515 / 6.468 s);
they were not re-measured this session. `deep-win` is **censored for search**
(no `proof_full`, no refutation). G3's easy-win ≤ 0.5 s and core-proved
clauses both fail.

## Soundness

`guidance-v1`, seed 0, `--seconds 10 --evals 5000000 --memory 192`, chosen
configuration: 0 refutations; 17 proved cells; every D14 proof accepted by
both checkers with equal position counts. `make test`, `make test-full`
(slow fixture oracle + `make veto`), and `make lint` are green.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: attribution, cheaper, budget, wall and soundness matrices complete |
| G2 determinism | **pass**: fixed-eval budgeted repeats identical; no wall clock in the search; default flags reproduce the plan4 tree; `aux_nodes = sim_nodes + settle_nodes` and the budget only skips optional work |
| G3 wall | **fail**: easy-win 3.7–4.2 s (≥ 26.7×), core censored at 10 s on all seeds, deep-win censored |
| G4 soundness | **pass**: no refutation; all 17 claimed proofs accepted by both checkers; `make test-full`/veto green |
| G5 build/docs | **pass**: `make lint` green; no GPL dependency; parsed data only |

## Surprises and findings

1. **The auxiliary share is real but the global-cap design trades wall for
   tree.** Simulation is 64 % of the easy-win wall, yet capping it globally
   starves the guide and grows the tree ~8×. A per-frame cap (plan4) was
   already weaker; neither reaches the wall gate. The next lever must keep
   the small tree while making the per-unit auxiliary work cheaper, not just
   remove it.
2. **The 0.5 s gate is below the bare tree.** Bare easy-win proves in 1.97 s
   at 34 M child evals. Even a zero-cost guide could not meet 0.5 s without
   a tree or per-node change. This is the absolute distance, not an auxiliary
   budget miss.
3. **Shorter playouts are a cheap 1.4× and no-sim a 2.8×**, both real but
   opt-in. The seed-0 decision proxy (0.275) was borderline; the median
   (0.330) and component ablations make the stop auditable.

## Problems, open parts, missing tests

- **G3 failure is not a budget bug.** No auxiliary configuration measured
  here reaches 0.5 s easy-win or 10 s core. The plan explicitly says not to
  start a third hand-guide plan after a G3 fail.
- **Reference walls are not same-session**: phase A had no provari rerun, so
  factors use the frozen `guidance-v1` reference. A same-session ratio would
  not change the ≥ 26× conclusion.
- **The "avoid a second legal movegen" B2 item is unmeasured**: the
  attribution prices simulation, and the settle is the tree lever. A future
  tree/per-node plan may revisit it. `sim_plies` is opt-in, default 6.
- **No full 24 × seeds guided sweep with the budget**; the seed-0 sweep found
  no refutation. A budget-default plan would need the wider sweep.

## Next steps

1. **Do not start another hand-guide or auxiliary-budget plan.** Take the
   D24 kill-review decision with the absolute distance: easy-win 26.7×,
   core ≥ 19.4× censored, deep-win censored, coverage 1/3.
2. **Re-pin option (owner):** a slower-provari corpus or a node-based goal
   would change what "wall" means; plan7 does not decide this.
3. **If the wall goal stays:** the next plan must be a tree or core per-node
   lever (e.g. D5 restarts, cheaper guide start evaluation/replay, proof-table
   pressure), not another auxiliary cap. The measured target is easy-win
   ≤ 0.5 s and core proved at 10 s.

## Deliverables

- Engine: `--guide-aux-budget`, `--guide-sim-plies`, global `aux_nodes`/
  `aux_skipped` accounting (`src/guide/{mod,simulate}.rs`,
  `src/search/{mod,frame}.rs`, `src/{cli,main}.rs`), unit/integration tests.
- Data: [`measurements/plan7/`](measurements/plan7/README.md) —
  `run_plan7.py`, `env.json`, `attribution.json`, `cheaper.json`,
  `budget.json`, `wall.json`, `soundness.json`, `README.md`.
- Docs: this report; `initiative.md` item-16 status/history; `bench.md` note.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report7.md`,
   `plan7.md`, `measurements/plan7/` and `initiative.md`. Run the D24 kill
   review with the measured distance: easy-win 26.7×, core censored at
   10 s (≥ 19.4×), deep-win censored, coverage 1/3. Decide continue /
   re-pin / close; do not start another hand-guide or auxiliary-budget plan."
2. Draft a tree/per-node plan (D5 restarts or cheaper guide start numbers)
   under the same wall gate, with the easy-win ≤ 0.5 s target treated as an
   absolute distance.
3. Re-measure the provari hand reference in the same session as a final
   wall-gate attempt before any re-pin decision.
