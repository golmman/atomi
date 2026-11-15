# Plan 41 measurements — item 54 GUIDE-SCALE (EF-PN weight)

Plan: [`plan41.md`](../../plan41.md). Report: [`report41.md`](../../report41.md).
Host, binaries and pins: [`env.json`](env.json). One process at a time, same
binary, 120 s wall cap, `--guide hand --memory 192 --seed N`, and
`--guide-ef-weight` written **after** `--guide hand` (D33).

| File | Contents |
| --- | --- |
| `run.py` | the driver: Phase 0 scan, the 3-cell × seeds 0–2 matrix, the held-out set |
| `runs.json` | every 120 s run, parsed counters (resumable by label) |
| `summary.json` | medians, ratios, held-out rows, gate verdicts |
| `d14.py`, `d14.json` | w = 2 D14: 9/9 both checkers, timed/untimed `child_evals` 9/9 |
| `supplementary.json` | one 600 s held-out run (the 120 s check is blind, not negative) |

## Phase 0 — G-41a PASS

`deep-win`, seed 0, 120 s. The counters reproduce the plan's table exactly;
the walls are within run noise.

| w | ms | expansions | length |
| ---: | ---: | ---: | ---: |
| 16 (control) | 53,773 | 5,803,257 | 36 |
| 8 | 61,953 | 6,248,348 | 31 |
| 6 | 48,320 | 4,945,117 | 35 |
| 4 | 37,564 | 3,888,745 | 37 |
| 3 | 33,304 | 3,499,269 | 38 |
| **2** | **30,401** | **3,165,142** | **25** |
| 1 | 34,846 | 3,771,434 | 35 |
| 0 | 62,243 | 6,902,183 | 28 |

Seeds 1–2 at w = 2: 27,280 / 2,840,088 / 47 and 38,916 / 4,035,397 / 36;
controls 50,327 / 5,376,491 / 37 and 36,266 / 3,862,476 / 29. Median
`deep-win` wall **30,401 vs 50,327 = 0.604×**.

Other cells, seeds 0–2, w = 2 vs 16 (ms / expansions / length):

| Cell | w = 16 | w = 2 | ratio of medians |
| --- | --- | --- | ---: |
| `core` | 4,886/424,249/15 · 4,069/344,750/26 · 4,042/333,878/20 | 4,435/382,485/16 · 4,404/383,603/19 · 4,362/374,939/13 | **1.082×** |
| `easy-win` | 374/37,999/19 · 413/43,071/19 · 625/67,674/16 | 680/61,489/23 · 513/48,123/19 · 1,092/113,903/14 | **1.647×** |

## Phase A — w = 2 and w = 4, 3 cells × seeds 0–2

Same-session w = 16 controls. `ratio` = candidate median / control median.

| Weight | `core` | `easy-win` | `deep-win` | geomean |
| --- | ---: | ---: | ---: | ---: |
| **w = 2** | 1.082× | 1.647× | **0.604×** | **1.0249×** |
| w = 4 | 1.087× | 1.279× | 0.746× | 1.0122× |

All 9 runs at each weight proved; no product cell was censored. w = 2 has the
lower median `deep-win` wall (30,401 vs 37,564 ms), so it is the G-41b
selection. Its geomean is inside the 1.05× guard.

## Held-out direction — G-41b(ii) unmeasurable

Five `sample20` cells, seed 0, 120 s, w = 2 vs 16. **All ten runs censored**
(`exit=timeout`, wall 120.0 s both sides), so the wall direction is not
measurable. The only 120 s signal is expansions:

| Index base | Cells | w = 2 fewer expansions |
| --- | --- | ---: |
| 0-based (1,5,9,13,17) | `b9a7…`, `3004…`, `60af…`, `3a7f…`, `5d89…` | **3/5** |
| 1-based (1,5,9,13,17) | `2a04…`, `6559…`, `8271…`, `8862…`, `9da3…` | **2/5** |

The plan does not say whether the indices are 0- or 1-based. Both readings are
recorded; neither is a wall win. A supplementary 600 s run of the first
0-based cell (`b9a7…`, w = 16) **proves at 584.0 s / 41,770,467 expansions /
length 23**, so the 120 s held-out check is blind, not negative.

## D14 — w = 2 (evidence, not applied)

`d14.json`: 9/9 (cell, seed) proofs accepted by `atomi validate` and
`provari check`; the timed 120 s run's `child_evals` equals the untimed
`--seconds 600 --proof-out` run 9/9.

## Gates

| Gate | Result |
| --- | --- |
| G-41a Phase 0 reproduced | **PASS** — counters exact |
| G-41b selection | **FAIL** — w = 2 wins `deep-win` and the geomean guard, but the held-out clause is unmeasurable (all censored; 3/5 by expansions only under 0-based indexing) |
| G-41c D14 | **PASS as evidence** — 9/9 both checkers, evals match; not applied |
| G-41d default | **not applied** — `ef_weight` stays 16; no `bench.md` row |
| G-41e `make test`, `make lint`, `make test-full` + veto | **PASS** |
| G-41f boundary | **PASS** — no engine/default/bench row, no new term, no provari source, no `T_4`/`T_5` claim |

## Limits

- The held-out set is `sample20`, where only 2/20 cells are shared wins; the
  other 18 are expected to be censored at 120 s. The guard needs a set where
  atomi proves at the cap (guidance-v1 shared wins) or a longer cap.
- The 5 M probe is not used: D33's fixed-cap trap makes it blind to this lever.
- The weight is a global constant; the non-monotone curve (w = 8 worse than
  16, w = 3 worse than 2 and 4) means the point is not a tuning rule.
