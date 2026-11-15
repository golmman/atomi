# Plan 41 — item 54 (proposed, GUIDE-SCALE): the EF-PN weight is the live eval lever

Read only: [`report40`](report40.md), [`plan40`](plan40.md), [`report39`](report39.md),
[`analysis1`](analysis1.md) §3 R2, [`plan19`](plan19.md)/[`report19`](report19.md),
[`gotchas`](gotchas.md), D33/D47/D49, the plan33 row of [`bench.md`](bench.md).
Pins: `atomic-movegen` 2.8.0; oracle `48686bc`. Binary `dd3c07c6…` at `4e973d2`.

## 1. The question

Plan40 closed item 48 (adding eval **terms**) negative: TERMS-V1 moved the
5 M `deep-win` tree by 31 expansions (0.99982×). But the eval is read through
`eval::scale_weighted`: `f = clamp(16 + e·w/16, 8, 32)`, default **w = 16**.
The eval's distribution is skewed (corpus roots + one ply: median −12, 60.4 %
at `f = 8`, 0.1 % at `f = 32`), so the default mapping is a near-constant
bias, not a signal. Plan19 scanned R2's **band edge**, never `w` on the
adopted numbers algebra. **Question: does a different EF-PN weight cut the
`deep-win` wall on the product `--guide hand` stack?** This is the surviving
question of report40 §7.4 (EF-PN weight sensitivity), not a term list, not a
per-node micro-plan, not a deepening schedule.

## 2. Phase 0 — read-only probe (this session; evidence, not a claim)

`deep-win`, `--guide hand --memory 192 --seed 0 --seconds 120`, one process at
a time, same binary. `--guide-ef-weight` after `--guide hand` (D33).

| w | ms | expansions | length |
| ---: | ---: | ---: | ---: |
| 16 (control) | 53,099 | 5,803,257 | 36 |
| 8 | 60,725 | 6,248,348 | 31 |
| 6 | 47,167 | 4,945,117 | 35 |
| 4 | 36,735 | 3,888,745 | 37 |
| 3 | 33,388 | 3,499,269 | 38 |
| **2** | **29,848** | **3,165,142** | **25** |
| 1 | 33,972 | 3,771,434 | 35 |
| 0 | 61,378 | 6,902,183 | 28 |

The curve is **non-monotone** (w = 8 is worse than 16; w = 3 worse than 2 and
4), so a single point is not a tuning rule. Seeds 1–2 at w = 2:
27,014 / 2,840,088 / 47 and 38,910 / 4,035,397 / 36, against control
49,868 / 5,376,491 / 37 and 36,315 / 3,862,476 / 29. **Median `deep-win`
wall 29,848 vs 49,868 = 0.599×** (1.67×), T4 distance 5.34× → **2.98×**.

Other cells, seeds 0–2, w = 2 vs 16 (ms / expansions / length):

| Cell | w = 16 | w = 2 | median ratio |
| --- | --- | --- | ---: |
| `core` | 4,813/424,249/15 · 4,160/344,750/26 · 4,104/333,878/20 | 4,376/382,485/16 · 4,493/383,603/19 · 4,368/374,939/13 | **1.052×** |
| `easy-win` | 361/37,999/19 · 433/43,071/19 · 630/67,674/16 | 667/61,489/23 · 511/48,123/19 · 1,060/113,903/14 | **1.540×** |

Median geomean ratio `(1.052 · 1.540 · 0.599)^(1/3)` = **0.990×**: a T4 lever
with a T5 wash, not a T5 lever. `easy-win` at 0.667 s is still 15× inside the
10 s rung. **The 5 M probe is blind**: w = 2 is 170,871 expansions vs 168,910
(1.012×) while the 120 s tree is 0.545× — D33's fixed-cap trap. The gate must
be 120 s wall, never the 5 M probe.

## 3. Phase A — full matrix

Weights **w = 2** (primary) and **w = 4** (secondary; the only other
`deep-win` win, 36,735 ms). 3 cells × seeds 0–2 × 2 weights + same-session
control, 120 s, one process at a time. Record wall, `child_evals`,
`expansions`, `length`, proof entries. No per-cell weight.

**Held-out direction check (G-41b(ii)).** Five fixed `sample20` cells
(indices 1, 5, 9, 13, 17 in `corpus.json` order), 120 s, seed 0, w = 2 vs 16.
Not claim cells, not a `bench.md` row. Direction passes if w = 2 is faster on
**≥ 3/5**; this is the guard against a three-cell artefact.

## 4. Gates

- **G-41a** Phase 0 table reproduced (the runs above; `env.json`, `runs.json`,
  README under `measurements/plan41/`).
- **G-41b selection** — choose the weight with the lowest **median `deep-win`
  wall** over seeds 0–2, subject to: all 9 runs proved; median geomean factor
  ≤ **1.05×** control; no cell censored; held-out direction ≥ 3/5. No
  per-cell weight, no weight chosen on `core`/`easy-win`.
- **G-41c D14** — every proof at the selected weight accepted by
  `atomi validate` and `provari check`; the timed run's `child_evals` matches
  the untimed `--seconds 600 --proof-out` run.
- **G-41d default** — only on G-41b/c: set `GuideParams::default().ef_weight`
  to the selected weight (D49: measured wall win), pin the default and the
  `--guide-ef-weight` round trip in a unit test, append one Cap-120 `bench.md`
  row. Otherwise keep w = 16 and record negative.
- **G-41e** `make test`, `make lint`, `make test-full` + veto green.
- **G-41f boundary** — no new eval term, no model, no provari source, no
  `T_4`/`T_5` claim, no item 48 re-open, no item 49/51/53 work.

## 5. Scope, effort, risks

**In:** Phase 0 table, Phase A matrix, held-out check, D14, and only on a pass
the one-line default + test + `bench.md` row. **Out:** new terms, per-node-kind
weight (needs code; ideation C), R2 band (closed, plan19), item 48, items
49/51/53, any search-base change. ≈1 day.

**Risks:** (i) the non-monotone curve means w = 2 may be a seed-0 artefact —
medians over seeds 0–2 and the held-out check are the guard; (ii) `easy-win`
regresses 1.54× — accepted only because it is 0.667 s, and the gate is
geomean + `deep-win`, not per-cell wall; (iii) the 5 M probe cannot see the
lever (D33) — do not gate on it; (iv) a global constant is a blunt mapping
(ideation A); (v) changing the default changes every `--guide hand` run —
G-41c/d and the full gate.

## 6. Ideation

- **A (recommended): global weight scan.** Gain: 1.67× median `deep-win` wall,
  T4 distance 5.34× → 2.98×, one parameter, no code. Cheapest: Phase 0 (done).
- **B (formulation change): scale-free EF-PN.** Replace the constant `w` with
  a mapping invariant to the eval's scale — a fixed quantile map, or a running
  mean/σ normalisation — so the 60 % `f = 8` saturation disappears. Gain:
  removes the arbitrary weight and could help all three cells; cost: a new
  read point and a distribution to maintain. Cheapest: measure the eval
  distribution on the **search frontier** (not corpus roots) and test a fixed
  z-score map at the same 120 s gate.
- **C: per-node-kind weight.** `EfPnMode::Both` is not CLI-reachable; add a
  second weight for AND nodes. Gain: the defender's eval is currently
  unscaled; cost: code + flag. Cheapest: expose `Both` and run w = 2 on both
  kinds.
- **D: class-specific weight.** The class numbers already encode urgency; a
  weight per class could avoid the `easy-win` regression. Cheapest: Phase A's
  per-class expansion counts.

## 7. Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan41.md`. Execute
   Phase A: w = 2 and w = 4, 3 cells × seeds 0–2 at 120 s with a same-session
   control, plus the five-cell held-out direction check; apply G-41b. If it
   passes, run D14, set `GuideParams::default().ef_weight`, add the unit test
   and one Cap-120 `bench.md` row; otherwise record negative and keep w = 16.
   Data in `measurements/plan41/`, `report41.md`; no new term, no provari
   source, no `T_4`/`T_5` claim."
2. If w = 2 wins `deep-win` but fails the geomean guard: propose a T4-only
   default under D49 (owner ruling) or keep it opt-in.
3. If the weight is too blunt: scope ideation B (scale-free mapping) as item
   55, with the frontier distribution as its Phase 0.
