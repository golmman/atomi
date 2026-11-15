# Plan 43 — item 56 (proposed, GUIDE-SCALE-FREE): a distribution-normalised EF-PN map

Read only: [`report40`](report40.md), [`report41`](report41.md),
[`report42`](report42.md), [`plan42`](plan42.md), [`analysis1`](analysis1.md)
§3 R2/R25, [`plan19`](plan19.md)/[`report19`](report19.md) (the fixed-edge R2
band), [`gotchas`](gotchas.md), D33/D47/D49, the plan33 row of
[`bench.md`](bench.md). Pins: `atomic-movegen` 2.8.0; oracle `48686bc`; the
plan42 binary (`fd2216fc…`, source `99f1628`). No provari source is read.

## 1. The question

`T_3` is `deep-win` **53.42 s** at Cap-120; `T_4` is **5.34×** away. The two
measured tree levers are now both exhausted as product changes: the bounded
length pass is a strict regression on top of w = 2 (plan42), and w = 2 is a
`deep-win`-only win (**0.604×**) that regresses the non-trivial easy cells
(`dec01` 2.8×, `dec05` 1.33×) and fails the held-out direction (2/5). It stays
opt-in; the default is w = 16.

The EF-PN map is

```text
f = clamp(16 + e·w/16, 8, 32);  pn × 16/f;  dn × f/16
```

a **linear function of the eval with fixed saturation**. The eval is skewed
(median −12, about 60 % of corpus roots at `f = 8`), so at w = 16 the map is a
near-constant bias and at w = 2 it is still saturated over much of the
distribution. Plan19 tested R2's **band edge** on this map; plan41 tested the
**weight**. Neither tested a map whose breakpoints come from the eval
distribution itself.

**Question.** Does a fixed, distribution-normalised EF-PN map — a quantile
band using the eval distribution at the guide's read point — cut the
`deep-win` wall on the product `--guide hand` stack without w = 2's easy-cell
regression? This is ideation B of plans 41/42, item 56: not a new eval term,
not a per-node micro-plan, not a deepening schedule.

## 2. Phase 0 — read-only distribution probe (this session)

Instrument a **temporary `/tmp` build** (or a `--stats`-only counter) to
histogram `hand_eval` at the EF-PN read point, split by node kind (OR/AND) and
by the oriented attacker eval. No repo change.

- **Fit sample:** 8 fixed `corpus-v1` cells **outside** the three claim cells,
  indices stated in the driver, seed 0, `--evals 2000000` each. The three claim
  cells and the held-out cells are **diagnostics only**, never fit.
- **Diagnostics:** `deep-win`, `core`, `easy-win`, and the fixed held-out set
  (guidance-v1 0-based 6–10, same as plan42), seed 0, 120 s.
- **Freeze before Phase A:** the two quantile edges (e.g. the 20th/80th
  percentiles of the fit distribution, OR/AND separately or one shared edge)
  and the three-step factor table, reused from EF-PN: OR `pn ×{11,6,1}`,
  `dn ×{1,11,21}`; AND `pn ×{21,11,1}`, `dn ×{1,6,11}`. Write the exact edges,
  the fit sample and the table into the plan's measurement README before any
  Phase A run.

**G-43a.** The map is a pure function of the eval (no cell, seed or position
input); the histogram and fit sample are recorded; the factor ratio is
**≥ 4× and ≤ 21×**; no claim or held-out cell is in the fit. A distribution
that does not separate the bands is a stop (the map would be a constant).

## 3. Phase A — scratch 120 s probe (no repo change)

Copy `HEAD` to `/tmp/atomi-plan43/`, wire the frozen map behind a temporary
switch, and run one process at a time, same session:

- `deep-win`, seed 0, 120 s, quantile map vs the w = 16 control;
- `core` and `easy-win`, seed 0, 120 s, same control.

**G-43b.** `deep-win` wall **≤ 0.80×** control (≤ ~42.7 s) and `length` not
increasing; `core`/`easy-win` no wall regression **> 10 %**. Fail ⇒ stop,
report negative, no repo change. The 5 M probe is blind to this lever (D33:
w = 2 is 1.012× at 5 M and 0.545× at 120 s), so it is **not** a gate.

## 4. Phase B — repo opt-in (only on a G-43b pass)

Add `--guide-ef-map quantile` (default **off**, D49) carrying the frozen edges
and table; unit-test the map's bounds, monotonicity in eval, `--guide hand` /
`--guide none` identity and the CLI round trip. No default change in this
phase.

**Matrix.** Three cells × seeds 0–2 at 120 s, same-session w = 16 control, plus
the fixed held-out 6–10 at 120 s seed 0. Record wall, `child_evals`,
`expansions`, `length`, proof entries.

**G-43c.** `deep-win` median wall ≤ **0.80×** control; median geomean
≤ **1.05×**; held-out direction **≥ 3/5**; no cell censored; D14 **9/9** on
both `atomi validate` and `provari check`, timed/untimed `child_evals` equal.

**G-43d.** Default only on G-43c **and** an owner ruling under D49. If adopted:
pin the default and the flag round trip in a unit test, append one Cap-120
`bench.md` row, re-run D14 on the default. Otherwise the map stays opt-in and
the plan records the negative.

## 5. Gates

- **G-43e.** `make test`, `make lint`, `make test-full` + veto green.
- **G-43f. Boundary.** No new eval term, no model, no provari source, no
  `T_4`/`T_5` claim, no item 48 re-open, no per-node micro-work, no items
  44/50, no default before G-43c/d. Data in `measurements/plan43/`,
  `report43.md`; raw transcripts and proofs are not committed.

## 6. Scope, effort, risks

**In:** distribution probe and freeze, the scratch Phase A, Phase B only on a
pass, data and report. **Out:** new eval terms, changing w = 16, a new search
base, table/memory changes, item 47/52 re-open, provari code or proofs.
Effort ≈ 1–2 days.

**Risks.** (i) A quantile map is a monotone reparametrisation of the same
eval and may not beat the best constant weight — the fit sample is a proxy for
the search frontier. (ii) The fit distribution may differ from the deep-win
frontier; Phase A's `deep-win` histogram is the check. (iii) The map may repeat
w = 2's deep-win/easy-cell split. (iv) Quantile edges are a global mapping; a
per-node-kind split is ideation B, not this plan. (v) A default is an owner
decision, not a measured pass.

## 7. Ideation

- **A (recommended): quantile band.** Gain: removes the arbitrary weight and
  the 60 % saturation; if it keeps the deep-win gain without w = 2's easy-cell
  regression, `T_4` distance falls 5.34× → ≤ 2.7×. Cheapest: Phase 0.
- **B (changes the formulation): per-node-kind normalisation.** Fit and apply
  separate maps at OR and AND nodes (the current default EF-PN is OR-only; R2
  applies both kinds with one edge). Gain: the defender's eval is not forced
  through the attacker's scale. Cheapest: Phase 0's OR/AND histograms.
- **C: z-score map.** Two parameters (mean, σ) instead of two quantiles;
  simpler to freeze and test. Cheapest: the same histogram.
- **D: quantile band on OR only.** A minimal-change ablation if Phase A shows
  the AND side is the regression source. Cheapest: Phase B ablation.

## 8. Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan43.md`. Execute
   Phase 0: histogram `hand_eval` at the EF-PN read point on 8 fixed
   non-claim `corpus-v1` cells at 2 M evals, plus the three claim cells and
   held-out 6–10 as diagnostics; freeze the quantile edges and the three-step
   table in `measurements/plan43/README.md`. Then Phase A on a `/tmp` copy:
   `deep-win` seed 0 at 120 s against the w = 16 control; apply G-43b. On a
   pass, implement `--guide-ef-map quantile` (default off) and run the Phase B
   matrix, held-out and D14; apply G-43c. No default without the owner ruling;
   otherwise record negative. Data in `measurements/plan43/`, `report43.md`;
   no provari source, no `T_4`/`T_5` claim."
2. If the map fails: close item 56 and take the instrument items **49
   (BENCH-POWER)** and **53 (TREE-METRIC)** together, so the next tree lever
   is not judged by a blind gate.
3. If it helps `deep-win` but regresses easy cells: propose a `T_4`-only
   opt-in under D49, with the held-out direction reported, not a default.
