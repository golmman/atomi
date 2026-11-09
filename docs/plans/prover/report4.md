# Prover Report 4: CORE-RESET — calibration, pressure, auxiliary budget

Plan: [`plan4.md`](plan4.md) (initiative [`prover`](initiative.md), backlog
item 3, re-scoped by the D21 kill review). Executed 2026-11-10. **G1 failed:
`core` is not proved at 10 s.** Per the plan, item 3 closes; there is no
plan5 split into mechanism plans. One new positive result is recorded for
the pivot: the calibrated hand stack **with a per-frame auxiliary budget**
proves `core` in 60 s and its proof is accepted by both checkers.

## Headline

- **Calibration does not move `core` at the tested horizon.** All 25
  `global_scale × ef_weight` points are censored at 50 M child evals; the
  best `core` count is 1,607,499 expansions (scale 64, EF-PN weight 0),
  against provari hand's 342,289 nodes (4.7× tree), and the bare plan3 tree
  is 1,633,456 expansions in this session. The lever is smaller than the
  remaining gap.
- **The calibration point is an `easy-win` win.** scale 64 / weight 0
  proves `easy-win` at 10 s on seeds 0–2 in 121,866 / 110,533 / 115,147
  expansions — all under the D21 limit 145,046. The uncalibrated plan3 hand
  stack was censored on all three.
- **Auxiliary budget is the one new `core` lever.** scale 64, weight 0,
  `max_sim=1`, `max_settle=1` proves `core` at **120 s**: 278,497,676 child
  evals, 9,667,450 expansions, length 20, `proof_full=false`, proof accepted
  by `atomi validate` and `provari check` with equal positions (14,332).
  Without the budget the same flags time out at 120 s (60.4 M evals,
  1,966,080 expansions).
- **The 10 s gate still fails on `core`.** At 10 s the auxiliary-budget stack
  is censored after 1.42 M expansions (4.4 M child evals/session seed 0).
  The 120 s proof cannot be substituted for the 10 s product condition.
- **Soundness is green.** No guidance-set refutation, both D14 proof runs
  accepted by both checkers, `make test-full` and `make veto` green,
  `--proof-out` neutral.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X container (4 vCPU), Linux 7.2.13, THP `madvise` |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154; licence gate clean |
| atomi | HEAD `7ad56c9e9d13948c73cad745ad27654ee67ddaa4` + plan4 work (agents do not commit) |
| Binary | sha256 `b2915bc3…33bc8a6` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, 24 cells, sha256 `2da2a65b…8c7d` (unchanged) |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |

All commands and parsed numbers are in
[`measurements/plan4/`](measurements/plan4/README.md).

## Exact commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan4/run_sweep.py --phase probe --out .../probes.json
python3 docs/plans/prover/measurements/plan4/run_sweep.py --phase sweep --out .../sweep.json
python3 docs/plans/prover/measurements/plan3/run_guidance.py --phase wall \
  --set .../plan3/guidance-v1.tsv --atomi target/release/atomi \
  --stack 'hand=--guide hand' \
  --stack 'chosen=--guide hand --guide-scale 64 --guide-ef-weight 0' \
  --stack 'aux=--guide hand --guide-scale 64 --guide-ef-weight 0 --guide-max-sim 1 --guide-max-settle 1' \
  --seeds 0,1,2 --repeats 1 --seconds 10 --memory 192 --cells core,easy-win \
  --proof-check --out .../plan4/gate.json
```

The `core` proof run is in `pressure.json` (same flags plus `--proof-out`,
120 s / 300 M cap). Gates: `make test`, `make lint`, `make test-full`
(includes `make veto`).

## Kill-review answers (D21 re-run, reduced)

- **Absolute distance?** `core` 4.7× tree at 50 M and censored at 10 s
  (provari 0.51 s); `easy-win` now proved at 10 s at 1.52–1.68× provari's
  node count but 42× its wall (auxiliary work); `deep-win` not re-run.
  Coverage on the two canonical cells: 1/2 at 10 s.
- **Build it again?** The bare DF-PN plus calibration is not enough. The
  auxiliary budget is the first configuration that proves `core` at all on
  atomi, but it is slower than 10 s.
- **Cancel?** Always-on sim/settle as currently priced; the full 24-cell ×
  seeds 0–2 D21 matrix was not re-run because the canonical `core` cell
  already fails G1. M4/M5 wait on the pivot decision.
- **Prior art.** Re-read the retro appendix; the EF-PN removal ratio
  (`easy-win` 0.44×) reproduces in atomi as scale 1 / weight 0 (110,293
  expansions, exactly the plan3 `--guide-off efpn` count).
- **Smallest next experiment.** Take the auxiliary-budget configuration as
  a new named stack and measure it at 10 s on `guidance-v1`; if it cannot
  reach the 10 s product condition, test M4 scale before any new mechanism.

## Probes and sweep

`probes.json`, 50 M child evals, seed 0, 192 MB:

| Cell | Stack | Result | child_evals | expansions | wall | sim/settle |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `core` | bare | censored | 50,000,034 | 1,633,456 | 3.0 s | 0 / 0 |
| `core` | hand | censored | 50,000,018 | 1,674,119 | 104.7 s | 256.5 M / 88.8 M |
| `easy-win` | bare | proved (17) | 30,335,618 | 1,047,370 | 1.7 s | 0 / 0 |
| `easy-win` | hand | proved (18) | 11,842,338 | 468,301 | 22.8 s | 60.0 M / 21.9 M |

Sweep `sweep.json`, 25 points, `core` 50 M / `easy-win` 10 M:

| Point | `core` expansions | `easy-win` expansions | result |
| --- | ---: | ---: | --- |
| scale 64, weight 0 (chosen) | **1,607,499** | **121,866** | core censored, easy proved |
| scale 16, weight 0 | 1,622,908 | 112,287 | core censored, easy proved |
| scale 1, weight 0 | 1,622,958 | 110,293 | core censored, easy proved |
| scale 64, weight 16 | 1,676,982 | 386,582 | core censored, easy proved |
| scale 4, weight 32 | 1,682,396 | 192,313 | core censored, easy proved |

The chosen point is the smallest `core` expansion count among points whose
`easy-win` guard is both proved and ≤ 145,046. `ef_weight=0` is the only
weight column that passes the guard at every scale; the scale itself changes
`core` by under 2%.

## Proof-table pressure and auxiliary budget

- Settled `Proved` children are no longer inserted at frame entry. A frame
  keeps them until it exits `Proved`; an OR frame installs only the child
  its value selects, an AND frame installs every settled child proof it
  needs. Settled `Refuted` children remain path-independent TT refutations.
  `settled_child_proof_round_trip` (selected child) and the new
  `selected_proved_child_matches_or_value` unit test cover the retention
  rule; the writer round trip is green.
- `--proof-share` (default 25%) is a `split_share` parameter. The byte
  invariant is tested for 1–4096 MiB × every share 1–90; the 90% refusal is
  unchanged.
- `GuideParams.max_sim` / `max_settle` are per-frame caps (0 = unlimited).
  The `core` pressure run: unlimited timeouts at 120 s; capped proves in
  59.9 s at 278.5 M child evals. The proof run reproduced `child_evals`
  exactly and was accepted by both checkers.

## G1 per cell and seed (10 s, 192 MB, seed 0–2)

| Stack | Cell | seed 0 | seed 1 | seed 2 | verdict |
| --- | --- | ---: | ---: | ---: | --- |
| `hand` | `core` | 151,552 | 145,408 | 152,576 | censored |
| `hand` | `easy-win` | 193,536 | 198,656 | 196,608 | censored |
| `chosen` (64/0) | `core` | 146,432 | 147,456 | 148,480 | censored |
| `chosen` (64/0) | `easy-win` | **121,866** | **110,533** | **115,147** | proved, all ≤ 145,046 |
| `aux` (64/0, 1/1) | `core` | 1,421,312 | 1,456,128 | 1,416,192 | censored |
| `aux` (64/0, 1/1) | `easy-win` | 918,551 | 759,177 | 773,238 | proved but > limit |

G1 requires `core` proved and `easy-win` ≤ 145,046 on every seed. `chosen`
passes the `easy-win` half on all three seeds but fails `core`; therefore
**G1 fails**. The canonical wall factors are recorded, not substituted:
provari hand 0.51 s (`core`) / 0.137 s (`easy-win`); chosen atomi 10.0 s
(censored) / 5.77 s (42×). The full 24-cell × 3-seed gate was not re-run.

## Gate table

| Gate | Result |
| --- | --- |
| **G1 D21 re-run** | **fail**: `core` censored on seeds 0,1,2 at 10 s; `easy-win` passes only with the calibrated point |
| **G2 soundness** | pass: no refutation; two D14 proofs accepted by both checkers; `--proof-out` counters identical |
| **G3 fixtures** | pass: `make test-full` green, no outcome regression |
| **G4 determinism** | pass: no-guide m1 counters unchanged; repeated seed identical; aux counters separate; no `Open` at `INFINITY` |
| **G5 build** | pass: `make test`, `make lint` (fmt/clippy/doc), release build, licence gate, no new dependency |
| **G6 docs** | pass (this report + `measurements/plan4/` + initiative/bench updates) |

## Verdict

**Item 3 is closed.** The D21 re-scope was the last attempt and G1 failed on
the canonical `core` cell. The calibrated hand stack is better than plan3 on
`easy-win` (proved at 10 s on three seeds) but not on `core`; the only
configuration that proves `core` needs about 60 s and a per-frame auxiliary
budget. No ledger row is appended.

**Pivot options**, in order:

1. **M4 first.** The auxiliary-budget configuration proves `core` with a
   691 k-entry proof table and an accepted 14,332-position proof. That is
   exactly the scale M4 is for; next measure it on `guidance-v1` at 10 s
   before adding any mechanism.
2. **New formulation (stop rule honoured).** Treat the 10 s product budget
   as the experiment: find a budget/schedule that bounds auxiliary work by
   wall time rather than by a per-frame count, then re-run G1 once.
3. **Close the hand-guide line.** If neither reaches 10 s, record the stack
   as a node-reduction result only (`easy-win` 1.5–1.7× provari nodes) and
   move the goal to per-node/parallel work.

## Not measured, problems, missing tests

- **Not measured:** the full 24-cell × seeds 0–2 D21 matrix; `deep-win`;
  both-direction ablations (conditional on G1); 3 wall repeats per cell
  (one repeat per seed); `provari` v0.2.0 stretch column.
- **Problems:** the auxiliary budget is the dominant `core` lever but its
  per-frame semantics need a wall-time wrapper for the 10 s goal. The
  calibrated point was selected on `core`/seed 0 only, as planned; the
  `easy-win` guard confirms it on seeds 0–2.
- **Missing tests:** a direct assertion that an unselected settled child's
  proof is absent from the proof table (the retention rule is covered
  structurally and by the writer round trip); a 24-cell auxiliary-budget
  gate.

## Next steps

1. Owner confirms D23 and the item-3 closure in `initiative.md`.
2. If the pivot is M4: run the auxiliary-budget stack on `guidance-v1`
   at 10 s × seeds 0–2, then start M4 with `core` as the scale case.
3. If the pivot is the wall-time budget: implement it behind a parameter,
   then one G1 re-run; no new guide mechanism.

## Deliverables

- **Engine:** calibration (`GuideParams.global_scale`, `ef_weight`), CLI
  flags, proof-table pressure, `--proof-share`, auxiliary budget, tests.
- **Measurements:** `measurements/plan4/{run_sweep.py,README.md,env.json,
  probes.json,sweep.json,gate.json,wall.json,pressure.json}`.
- **Docs:** this report, `initiative.md` (D23, status, history),
  `bench.md` (absolute distance, plan4 section), `docs/plans/README.md`.

**SESSION COMPLETE**

Deliverables and follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report4.md`,
   `initiative.md` and `measurements/plan4/`. Execute the M4-first pivot:
   measure the scale-64/weight-0 hand stack with `max_sim=1,
   max_settle=1` on `guidance-v1` at 10 s × seeds 0–2, then write the M4
   plan (no new guide mechanism)."
2. Alternative: implement a wall-time auxiliary budget behind a parameter
   and re-run G1 once on the canonical cells before M4.
3. Alternative: close the hand-guide line and plan a per-node/parallel
   milestone instead.
