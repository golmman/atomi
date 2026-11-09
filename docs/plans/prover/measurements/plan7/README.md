# plan7 measurements

Plan: [`../../plan7.md`](../../plan7.md) (item 16, wall attribution and a
deterministic auxiliary budget). Parsed output only: no raw transcripts,
provari artifacts, proof files or `__pycache__` are committed. Temporary
proofs are written under `proofs/`, checked, then deleted by the driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus/guidance sha256, oracle version, stack flags |
| `attribution.json` | phase A: 8 stacks x easy-win/core (seeds 0–2) and deep-win (seed 0), fixed-`--evals` and canonical 10 s budgets; decision rule |
| `cheaper.json` | B2 leave-one-out: simulation plies 6/4/3/2 and no-sim, easy-win/core, seeds 0–2 |
| `budget.json` | B3 grid: 10 k / 100 k / 1 M / 10 M / unlimited, easy-win/core, seeds 0–2; fixed-eval determinism repeats |
| `wall.json` | phase C: chosen configuration, 10 s / 192 MB, seeds 0–2, deep-win one attempt, D14 checks |
| `soundness.json` | guidance-v1 seed 0 with the chosen configuration: refutations and both-checker D14 proofs |
| `run_plan7.py` | stdlib-only driver (`env`, `attribution`, `cheaper`, `budget`, `wall`, `soundness`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan7/run_plan7.py --phase env
python3 docs/plans/prover/measurements/plan7/run_plan7.py --phase attribution
python3 docs/plans/prover/measurements/plan7/run_plan7.py --phase cheaper
python3 docs/plans/prover/measurements/plan7/run_plan7.py --phase budget \
  --budgets 10000,100000,1000000,10000000,0 --chosen-budget 100000
python3 docs/plans/prover/measurements/plan7/run_plan7.py --phase wall \
  --chosen-budget 100000
python3 docs/plans/prover/measurements/plan7/run_plan7.py --phase soundness \
  --chosen-budget 100000
make test-full
make lint
```

One atomi process at a time. The product point is the plan4 calibration
(`--guide hand --guide-scale 64 --guide-ef-weight 0`); `bare` is
`--guide none`. The fixed-evaluation budget is 10 M child evals, the canonical
budget 10 s / 192 MB.

## Phase A — where the wall goes

Medians over seeds 0–2 for easy-win/core (deep-win is seed 0 only). `aux` is
`sim_nodes + settle_nodes`.

| Stack | easy-win 10 s wall · exp · aux · proved | core 10 s wall · exp · aux | easy-win 10 M wall · exp |
| --- | --- | --- | --- |
| bare | **1.965 s** · 1.24 M · 0 · 3/3 | 10.003 s · 5.38 M · 0 | 0.554 s · 321 k |
| hand (product) | **5.829 s** · 115 k · 21.7 M · 3/3 | 10.037 s · 148 k · 33.7 M | 5.778 s · 115 k |
| no closures | 8.549 s · 744 k · 20.8 M · 2/3 | 10.010 s · 584 k · 25.7 M | 3.664 s · 317 k |
| no sim | **2.096 s** · 132 k · 6.7 M · 3/3 | 10.009 s · 486 k · 28.2 M | 2.105 s · 132 k |
| neither | 3.905 s · 993 k · 0 · 3/3 | 10.005 s · 2.10 M · 0 | 1.280 s · 313 k |
| aux 1/1 (per frame) | 4.587 s · 773 k · 5.5 M · 3/3 | 10.009 s · 1.48 M · 10.5 M | 1.969 s · 324 k |
| sim only 1 | 2.056 s · 117 k · 6.7 M · 3/3 | 10.014 s · 446 k · 28.5 M | 2.059 s · 117 k |
| settle only 1 | 10.004 s · 767 k · 26.2 M · 1/3 | 10.009 s · 537 k · 26.5 M | 4.318 s · 325 k |

The plan's literal decision rule on easy-win, seed 0, canonical budget:
`(hand − neither)/hand = (6.051 − 4.388)/6.051 = 0.275 < 0.3`. That one seed
is just below the threshold. Across seeds the share is 0.275 / 0.408 / 0.330;
the median is **0.330 ≥ 0.3**, so the premise survives by the median reading.
The no-sim comparison is stronger: removing simulation alone cuts the
easy-win wall from 5.829 s to 2.096 s (**64 %** of the hand wall). The
`neither` control is confounded because removing closures also grows the tree
and adds wall.

The attribution also shows why an auxiliary-only lever cannot reach the wall
gate: with all auxiliary work removed (`neither`, 0 aux), easy-win still takes
3.9 s because the tree grows to ~1 M expansions, and the bare search takes
1.97 s. The 0.5 s gate is below even the bare search's 10 s tree.

## Phase B — cheaper auxiliary paths

Canonical 10 s, medians over seeds 0–2:

| Variant | easy-win wall | expansions | aux |
| --- | ---: | ---: | ---: |
| simulation 6 plies (hand) | 5.777 s | 115,147 | 21.7 M |
| simulation 4 plies | 4.609 s | 113,080 | 16.3 M |
| simulation 3 plies | 4.327 s | 118,650 | 14.5 M |
| simulation 2 plies | 4.196 s | 121,224 | 12.3 M |
| no simulation | **2.073 s** | 131,645 | 6.7 M |

Shorter playouts are a real but insufficient gain (6 → 2 plies: 1.38× on the
median). No simulation is the best auxiliary configuration, but it still
misses the 0.5 s gate by 4.1×. `--guide-sim-plies` is opt-in and default 6,
so the product trajectory is unchanged.

## Phase B3 — global auxiliary budget grid

Medians over seeds 0–2, canonical 10 s. `aux` is the admitted work; `skipped`
is the number of optional operations the global budget declined.

| Budget | easy-win wall · exp · aux · proved | core wall · exp · aux · proved |
| --- | --- | --- |
| 10 k | 3.848 s · 915 k · 10 k · 3/3 | 10.008 s · 2.01 M · 10 k · 0/3 |
| 100 k | 3.891 s · 923 k · 100 k · 3/3 | 10.005 s · 2.00 M · 100 k · 0/3 |
| 1 M | 4.008 s · 885 k · 1 M · 3/3 | 10.005 s · 1.95 M · 1 M · 0/3 |
| 10 M | 4.386 s · 456 k · 10 M · 3/3 | 10.007 s · 1.42 M · 10 M · 0/3 |
| unlimited | **5.791 s** · 115 k · 21.7 M · 3/3 | 10.016 s · 147 k · 33.7 M · 0/3 |

The global budget lowers easy-win wall to ~3.8–3.9 s, but it starves the guide
early, so the tree grows ~8× and the wall saving is only ~1.5×. No grid point
proves `core` at 10 s. Determinism holds under a fixed eval cap: two runs at
`--evals 10000000 --guide-aux-budget 100000` are identical on both cells
(exit, length, `child_evals`, `expansions`, `aux_nodes`, `aux_skipped`); see
`budget.json` `repeat_fixed`. The canonical 10 s `core` repeats differ only
because the wall deadline censors different amounts of work.

## Phase C — wall gate

Chosen configuration: the calibrated hand stack with `--guide-aux-budget
100000` (the best easy-win median in the grid).

| Cell | seed | exit | expansions | child_evals | aux | wall | provari hand | factor | D14 |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| easy-win | 0 | proved | 922,854 | 27.4 M | 100 k | 3.841 s | 0.139 s | 27.6× | accepted |
| easy-win | 1 | proved | 992,315 | 29.1 M | 100 k | 4.162 s | 0.139 s | 29.9× | accepted |
| easy-win | 2 | proved | 887,174 | 26.0 M | 100 k | 3.711 s | 0.139 s | 26.7× | accepted |
| core | 0 | timeout | 2.04 M | 62.3 M | 100 k | 10.003 s | 0.515 s | ≥ 19.4× | — |
| core | 1 | timeout | 1.98 M | 60.9 M | 100 k | 10.007 s | 0.515 s | ≥ 19.4× | — |
| core | 2 | timeout | 2.00 M | 61.4 M | 100 k | 10.003 s | 0.515 s | ≥ 19.4× | — |
| deep-win | 0 | timeout | 2.40 M | 71.1 M | 100 k | 10.004 s | 6.468 s | ≥ 1.55× | — |

`deep-win` is classified **censored for search** (no `proof_full`, no
refutation). The provari reference walls are from `guidance-v1.tsv`; they were
not re-measured in this session, so the factors are reference-based, not
same-session ratios.

## Soundness

`guidance-v1`, seed 0, 10 s / 5 M evals / 192 MB, chosen configuration:
0 refutations. 17 cells proved inside the cap; all 17 D14 proofs are accepted
by `atomi validate` and `provari check` with equal position counts.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | pass: attribution/cheaper/budget/wall/soundness matrices complete |
| G2 determinism | pass under a fixed-eval cap (`repeat_fixed` identical); no wall clock in the search; default `aux_budget=0` reproduces the plan3/4 tree (easy-win 121,866 / 110,533 / 115,147) |
| G3 wall | **fail**: easy-win 3.7–4.2 s (≥ 26.7×), core censored at 10 s, deep-win censored |
| G4 soundness | pass: no refutation; all 17 proved proofs accepted by both checkers; `make test-full` and `make veto` green |
| G5 build/docs | pass: `make lint` green; no GPL dependency; no raw artifacts |
