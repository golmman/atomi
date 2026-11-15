# Report 41 — item 54 GUIDE-SCALE (EF-PN weight)

Plan: [`plan41.md`](plan41.md). Initiative: [`initiative.md`](initiative.md),
D33/D47/D49, [`gotchas`](gotchas.md). Data:
[`measurements/plan41/`](measurements/plan41/README.md). **No engine, CLI,
default, test or `bench.md` row changed.**

## 1. What was run

Same binary, one process at a time, 120 s wall cap, `--guide hand --memory 192
--seed N`, `--guide-ef-weight` after `--guide hand` (D33). The measurement
binary is `dd3c07c6…` (the plan's binary, source `4e973d2`; `HEAD` is
`6282c2a`, docs only). A post-gate `cargo build --release` produced
`749d77c2…`; the `deep-win` w = 2 seed-0 check reproduces 3,165,142
expansions / length 25, so the rebuild is behaviourally identical.

- **Phase 0:** `deep-win` seed 0, w = 16, 8, 6, 4, 3, 2, 1, 0; seeds 1–2 at
  w = 2 and 16; `core`/`easy-win` seeds 0–2 at w = 2 and 16.
- **Phase A:** w = 4 on all three cells × seeds 0–2 (the w = 2 and control
  rows are the Phase 0 rows, same session).
- **Held-out:** five `sample20` cells, seed 0, w = 2 vs 16, 120 s. The plan
  does not say whether the indices are 0- or 1-based, so both readings were
  run.
- **D14:** w = 2, all nine (cell, seed) proofs exported and claimed; the timed
  run's `child_evals` compared with an untimed `--seconds 600 --proof-out` run.
- **Supplementary:** one 600 s run of the first 0-based held-out cell at
  w = 16, to test whether the 120 s held-out check is blind.

## 2. Phase 0 — G-41a PASS

The counters reproduce the plan's table exactly; walls are within noise. The
`deep-win` curve is non-monotone: w = 8 is worse than 16, w = 3 is worse than
2 and 4. The minimum is **w = 2: 30,401 ms / 3,165,142 expansions / length
25** against the control **50,327 ms / 5,803,257 / 36** (median over seeds
0–2, **0.604×**). The other cells at w = 2 are `core` 1.082× and `easy-win`
1.647× (ratio of medians), geomean **1.0249×**. The plan's Phase 0 small-cell
ratios were 1.052/1.540/0.990; this session's are 1.082/1.647/1.025, a
small-cell wall-noise difference (the counters are exact) that still leaves
the geomean inside the 1.05× guard.

## 3. Phase A — w = 2 vs w = 4

| Weight | `core` | `easy-win` | `deep-win` | geomean |
| --- | ---: | ---: | ---: | ---: |
| **w = 2** | 1.082× | 1.647× | **0.604×** | **1.0249×** |
| w = 4 | 1.087× | 1.279× | 0.746× | 1.0122× |

All 9 runs at each weight proved; no product cell was censored. w = 2 has the
lower median `deep-win` wall (30,401 vs 37,564 ms), so it is the G-41b
selection. Its geomean is inside the 1.05× guard. The `easy-win` regression is
real (1.65×) but the cell is 0.68 s, 15× inside the 10 s rung; the gate is
geomean + `deep-win`, not per-cell wall.

## 4. Held-out — G-41b(ii) unmeasurable, so G-41b FAIL

All ten 120 s held-out runs are censored (`exit=timeout`, wall 120.0 s on both
sides). The wall direction is therefore not measurable. The only 120 s signal
is expansions: w = 2 has fewer on **3/5** cells under 0-based indices
(`b9a7…`, `3004…`, `60af…`, `3a7f…`, `5d89…`) and **2/5** under 1-based
indices (`2a04…`, `6559…`, `8271…`, `8862…`, `9da3…`). Neither is a wall win.

The plan's held-out set is `sample20`, where only 2/20 cells are shared wins
(`easy-win`, `deep-win`); the other 18 are expected to be censored at 120 s.
A supplementary 600 s run of `b9a7…` at w = 16 **proves at 584.0 s /
41,770,467 expansions / length 23**, so the 120 s check is blind, not
negative. The guard as written cannot decide the question.

**Verdict: G-41b FAIL** (the held-out clause is not met as written). Under the
plan's stop rule, G-41c/d are not applied: `GuideParams::default().ef_weight`
stays **16**, no unit test, no `bench.md` row. w = 2 remains available as
`--guide-ef-weight 2`.

## 5. D14 — w = 2 (evidence for an owner ruling)

Although G-41b failed, the candidate was checked so a ruling can be made
without another session. `d14.json`: **9/9** (cell, seed) proofs accepted by
`atomi validate` and `provari check`; the timed 120 s run's `child_evals`
equals the untimed `--seconds 600 --proof-out` run **9/9**. The w = 2 proofs
are sound; the open question is the default policy, not correctness.

## 6. Gates and boundary

| Gate | Result |
| --- | --- |
| G-41a Phase 0 reproduced | **PASS** — counters exact |
| G-41b selection | **FAIL** — w = 2 wins `deep-win` and the geomean guard, but the held-out clause is unmeasurable |
| G-41c D14 | **PASS as evidence** — 9/9 both checkers, evals match; not applied |
| G-41d default | **not applied** — `ef_weight` stays 16; no `bench.md` row |
| G-41e `make test`, `make lint`, `make test-full` + veto | **PASS** — 149 unit + integration, `VETO: PASS` |
| G-41f boundary | **PASS** — no engine/default/bench row, no new term, no provari source, no `T_4`/`T_5` claim |

## 7. Problems, scope and missing tests

- **The held-out set is the plan's defect.** `sample20` is mostly censored by
  construction; a direction check on it at 120 s cannot produce a wall win.
  The 0-/1-based ambiguity makes the expansion fallback 3/5 or 2/5. The next
  held-out check must use cells where atomi proves at the cap (guidance-v1
  shared wins) or a cap long enough to prove (the 600 s run shows one cell
  needs 584 s).
- **The weight is a global constant.** The non-monotone curve means w = 2 is
  a measured point, not a tuning rule. Ideation B (scale-free mapping) and C
  (per-node-kind weight) remain untested; neither is in this plan's scope.
- **No source change, so no new unit test.** If the default is changed later,
  pin `GuideParams::default().ef_weight` and the `--guide-ef-weight` round trip
  in a unit test, and append one Cap-120 `bench.md` row.
- **The 5 M probe was not used** (D33): it is blind to this lever. The gate is
  120 s wall.

## 8. Next steps

1. **Owner ruling (recommended).** w = 2 is a measured `deep-win` wall win
   (0.604×) with a geomean wash (1.0249×) and D14 9/9. Under D49 it is a
   candidate default; the held-out guard did not refute it, it failed to
   measure. Options: (a) adopt w = 2 as the default and add the unit test +
   one Cap-120 `bench.md` row; (b) keep w = 16 and w = 2 opt-in; (c) re-run the
   held-out check on a proving set before deciding.
2. **Fix the held-out instrument** (item 49/51/53 territory): use
   guidance-v1 shared wins or a 600 s cap, and state the index base.
3. **Do not re-open item 48** (terms) or the R2 band on this evidence; the
   lever is the weight, not the term list.

## 9. Follow-up options

1. **Kickoff (recommended):** “Read `docs/plans/prover/report41.md`. Rule on
   the w = 2 default: it is 0.604× on `deep-win`, geomean 1.0249×, D14 9/9,
   but the plan's held-out set was all-censored at 120 s. If adopted, set
   `GuideParams::default().ef_weight = 2`, pin the default and the
   `--guide-ef-weight` round trip in a unit test, append one Cap-120
   `bench.md` row, and run `make test-full` + veto. If not, keep w = 16 and
   record the ruling.”
2. Re-run the held-out direction on a set where atomi proves at 120 s
   (guidance-v1 shared wins), with the index base fixed.
3. If the weight is too blunt, scope ideation B (scale-free EF-PN) as item 55,
   with the search-frontier eval distribution as its Phase 0.

SESSION COMPLETE — Phase 0 reproduced, w = 2 is a 0.604× `deep-win` lever with
a 1.0249× geomean, D14 9/9; the held-out guard is unmeasurable on `sample20`,
so G-41b fails and the default stays 16. No engine or default change;
measurements in `measurements/plan41/`, report in `report41.md`.
