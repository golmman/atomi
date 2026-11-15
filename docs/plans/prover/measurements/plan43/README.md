# Plan 43 measurements — item 56 GUIDE-SCALE-FREE (quantile EF-PN map)

Plan: [`plan43.md`](../../plan43.md). Report: [`report43.md`](../../report43.md).
Host, binaries and pins: [`env.json`](env.json). One process at a time, 120 s
wall cap, `--guide hand --memory 192 --seed N`.

| File | Contents |
| --- | --- |
| `run.py` | Phase 0 driver: the 8-cell fit sample and the diagnostics |
| `runs.json` | every Phase 0 run, parsed counters (resumable by label) |
| `hist/*.json` | the oriented attacker eval histogram at each fresh EF-PN read point |
| `summary.json` | the fit percentiles |
| `phaseA.py` | the scratch Phase A driver (quantile map vs the w = 16 control) |
| `phaseA.json` | the Phase A rows and ratios |

## Phase 0 — fit sample (frozen before Phase A)

`corpus-v1` = [`bench/corpus.json`](../../../../bench/corpus.json), sha256
`9cd0169a…47d0e`. The fit sample is the eight fixed `sample20` **0-based**
indices `0,1,2,4,5,6,7,8`; index 3 (`easy-win`) and index 10 (`deep-win`) are
excluded, so no claim cell is in the fit. The held-out diagnostics are the
fixed `guidance-v1` 0-based indices 6–10 (the same set as plan42). Seed 0,
`--evals 2000000` per fit cell; diagnostics at 120 s.

The histogram build is a `/tmp` copy of `HEAD` (`30b4b9fe…`); it records the
oriented attacker eval at each **fresh** EF-PN read point, split by node kind
and by parent/child board. The extra `hand_eval` calls do not change the tree:
`deep-win` reproduces plan33's 5,803,257 expansions exactly (wall is slower,
91.4 s vs 53.4 s, because of the instrumentation).

| Fit cell (0-based) | id | ms | `child_evals` | expansions |
| ---: | --- | ---: | ---: | ---: |
| 0 | `2a0443d446ee` | 1,158 | 2,000,015 | 69,935 |
| 1 | `b9a7ed7eb368` | 1,194 | 2,000,015 | 77,693 |
| 2 | `2e8276dc4530` | 1,251 | 2,000,030 | 67,919 |
| 4 | `6559fbdddb47` | 1,076 | 2,000,007 | 73,576 |
| 5 | `3004d09cef79` | 1,119 | 2,000,019 | 66,675 |
| 6 | `5862b1958212` | 1,124 | 2,000,008 | 66,499 |
| 7 | `dd3f89766a40` | 1,281 | 2,000,004 | 67,065 |
| 8 | `8271708709d4` | 1,185 | 2,000,024 | 73,322 |

No fit cell proved at 2 M evals. The distribution is narrow and skewed
(median 12, p20 ≈ 0, p80 ≈ 26), not the ±128 scale plan19 probed.

| Histogram | n | p10 | p20 | p50 | p80 | p90 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OR parent | 6,848,889 | −7 | −1 | 12 | 29 | 38 |
| AND parent | 6,246,023 | −6 | 0 | 12 | 24 | 32 |
| OR child | 6,848,889 | −9 | −2 | 12 | 29 | 38 |
| AND child | 6,246,023 | −7 | 0 | 12 | 24 | 32 |
| shared parent | 13,094,912 | — | 0 | — | 26 | — |
| shared child | 13,094,912 | — | −1 | — | 26 | — |

## Frozen map (before any Phase A run)

- **Read point:** the parent node's `hand_eval`, oriented to the attacker
  (`ctx.eval` at OR, `-ctx.eval` at AND). This is R2's read point, the one the
  frozen factor table belongs to.
- **Edges:** one **shared** edge pair, the 20th/80th percentiles of the shared
  parent fit distribution: **lo = 0, hi = 26**. A per-node-kind split is
  ideation B, not this plan (plan43 §6 risk iv).
- **Step:** `0` if `oriented < 0`, `1` if `0 ≤ oriented < 26`, `2` if
  `oriented ≥ 26`.
- **Factor table (R2's, reused):** OR `pn ×{11,6,1}`, `dn ×{1,11,21}`; AND
  `pn ×{21,11,1}`, `dn ×{1,6,11}`. Max/min factor ratio **21×** (G-43a's
  ≥4×/≤21× band).
- **Replacement, not addition:** the quantile band replaces the linear
  `f = clamp(16 + e·w/16, 8, 32)` weight map (the plan's "removes the
  arbitrary weight and the 60 % saturation"). The control is the default
  `--guide hand` w = 16 stack.

G-43a: the map is a pure function of the eval (no cell, seed or position
input); the fit sample and histograms are recorded; the factor ratio is 21×;
no claim or held-out cell is in the fit. The bands separate (p20 = 0,
p80 = 26), so the map is not a constant.

## Diagnostics (not fit)

| Cell | ms | `child_evals` | expansions | length |
| --- | ---: | ---: | ---: | ---: |
| `core` | 8,021 | 12,439,376 | 424,249 | 15 |
| `easy-win` | 599 | 1,016,547 | 37,999 | 19 |
| `deep-win` | 91,437 | 155,236,740 | 5,803,257 | 36 |
| `dec01` (held-out 6) | 262 | 494,508 | 22,561 | 11 |
| `dec03` (7) | 13 | 1,979 | 68 | 4 |
| `dec04` (8) | 8 | 1,186 | 71 | 8 |
| `dec05` (9) | 1,844 | 2,526,268 | 181,740 | 35 |
| `dec07` (10) | 3 | 303 | 15 | 4 |

The diagnostic walls are from the instrumented build and are **not** Phase A
controls; Phase A uses the clean `/tmp/atomi-plan43a` binary (`cfee2b8e…`).

## Phase A — G-43b FAIL

Clean `/tmp/atomi-plan43a` binary (`cfee2b8e…`), `--guide hand --memory 192
--seed 0 --seconds 120`, one process at a time. The map is the frozen
replacement above; the control is the default w = 16 stack.

| Cell | control ms | map ms | ratio | control exp | map exp | control len | map len |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `deep-win` | 54,164 | 41,449 | **0.765×** | 5,803,257 | 4,359,526 | 36 | 34 |
| `core` | 4,965 | 7,900 | 1.591× | 424,249 | 750,081 | 15 | 19 |
| `easy-win` | 366 | 777 | 2.123× | 37,999 | 80,338 | 19 | 19 |

`deep-win` is inside the 0.80× bar and the length does not increase, but
`core` and `easy-win` are far outside the 10 % guard. **G-43b FAIL ⇒ stop**:
no Phase B, no `--guide-ef-map`, no default, no `bench.md` row. The map
repeats w = 2's `deep-win`/easy-cell split (plan risk iii).
