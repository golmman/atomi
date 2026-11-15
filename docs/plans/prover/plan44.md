# Plan 44 — kill review (plans 42–43) and item 53 (proposed, TREE-METRIC): an instrument that can see a tree lever

Read only: [`report42`](report42.md), [`report43`](report43.md),
[`plan42`](plan42.md), [`plan43`](plan43.md), [`analysis1`](analysis1.md) §3,
[`analysis2`](analysis2.md) §5–§6, [`gotchas`](gotchas.md), D30/D33/D47–D50,
the plan33 row of [`bench.md`](bench.md). Pins: `atomic-movegen` 2.8.0; oracle
`48686bc`; binary at `4de2b56`. No provari source is read.

## Kill review (fixed prompt; covers plans 42–43)

### 1. Absolute distance to the goal

`T_3`, Cap-120 plan33 row: `core` 4.923 s, `easy-win` 0.376 s, `deep-win`
**53.42 s**, geomean **6.606**, 3/3, D14 9/9. `T_4` is `deep-win` **5.34×**;
`T_5` is **6.61× geomean**. **D50 (owner, 2026-11-16): `T_5` remains the
goal** — the re-scope to `T_4` is declined, so the distance is the full 6.61×.

The two axes: per-node **4.38×** (D48) with a measured **3.22× ceiling**
(plan34: 68.97 pp of op budget against ≥ 81.3 pp needed); the tree
(`deep-win` 5,803,257 expansions against 2,099,100 reference bodies; `length`
36 against `mate` 19). If the per-node gap is roughly uniform, per-node parity
alone leaves ≈1.5× (`6.61 / 4.38`), so `T_5` needs per-node parity **and** a
≈1.5× tree advantage. Plans 42–43 closed the last two tree levers as product
changes: w = 2 is 0.604× on `deep-win` but 1.6–2.1× on the easy cells and 2/5
held-out; the quantile map is 0.765× on `deep-win` but 1.59×/2.12× on
`core`/`easy-win`. **No measured lever is general.**

### 2. If we started today, would we build this?

**Yes** to the prover, hand-guide stack, full-key proof table, validator and
veto. **No** to the 20-plan per-node detour, and **no** to instruments that
were never used to judge a lever. The instrument is now the prerequisite, not
a detour: three tree levers in a row were judged by a gate that could not see
them (held-out all-censored, the 5 M probe blind to tree levers, single seed).

### 3. Which initiative or plan would we cancel?

- **Item 48** (terms): closed negative; do not re-open.
- **Item 52 Phase B** (deepening): closed negative; keep `--max-length`.
- **Items 44/50 as a `T_5` route**: keep only as the `T_4` gamble; item 50 is
  the funding test.
- **Item 41**: a `movegen` wishlist line, not a `prover` dependency.
- **Plan37 micro-candidates**: ≈1.20×; opportunistic only.

### 4. Prior-art delta (analysis1 §3; pin unchanged ⇒ changed rows only)

**R2 (banded EF-PN)** is re-argued with a number: plan41 tested the weight,
plan43 the quantile edges; both are cell-specific, so the band is not
re-opened without a new ordering hypothesis. **R25 (length-limited passes)**:
`L = 24` is 1.42× on `deep-win` but a strict regression on top of w = 2
(plan42); the axis is measured, not refuted. R22 skip; R23/R24 keep; R9
closed; R11–R13, R18/R19 adopted; R26 skip.

### 5. Smallest next experiment

Item 53's metric: reproduce the committed counters on the three cells and the
5 M probe, then rank three historical levers (NULL-PIPE, HOTPATH-V2, G1) by
the metric and check the sign against their 120 s rows. If the metric cannot
rank levers it already knows, it cannot judge a new one.

## Item 53 — TREE-METRIC (proposed)

**Question.** Can a fixed-context tree metric rank tree-changing levers the
same way the 120 s matrix does, at a fraction of the cost, so the next tree
lever is not judged by a blind gate?

**Phase 0 — define and widen the sample (read-only).** Fix the metric per
cell: at the 5 M `--evals` probe **and** at a fixed cap (120 s), record
`expansions`, `child_evals`, `length`, wall and ns/exp, with `--guide none` as
a standing control. Widen the tuning sample (item 49): N fixed `corpus-v1`
cells outside the three claim cells, seed 0, fixed 10 s, report the
factor/tree-identity distribution. No tuning cell becomes a claim cell;
nothing enters `bench.md`.

**Phase A — reproduce.** The metric reproduces the committed values
byte-for-byte on the three cells and the 5 M probe; two same-binary runs are
identical.

**Phase B — rank known levers.** Rank NULL-PIPE, HOTPATH-V2 and G1 by the
metric; the sign must agree with their 120 s rows (0.926×, 0.7223–0.7541×,
0.986×).

## Gates

- **G-44a.** The metric is a pure function of the binary, FEN, flags and seed;
  wall is reported, never ranked.
- **G-44b.** Reproduces the committed counters byte-for-byte; two runs
  identical.
- **G-44c.** Sign agreement on **3/3** historical levers.
- **G-44d.** The widened sample's direction agrees with the three claim cells
  on **≥ 2/3** historical levers, or the sample is reported as non-predictive.
- **G-44e.** `make test`, `make lint`, `make test-full` + veto green.
- **G-44f. Boundary.** No engine change, no default, no `bench.md` row, no
  `T_4`/`T_5` claim, no item 48/52 re-open, no provari source. Data in
  `measurements/plan44/`, `report44.md`; raw transcripts and proofs are not
  committed.

## Scope, effort, risks

**In:** the metric definition, the widened sample, the reproduction and
ranking, data and report. **Out:** engine changes, defaults, new eval terms, a
new search base, provari code. Effort ≈ 1–2 days.

**Risks.** (i) The 5 M probe is blind to tree levers (D33: w = 2 is 1.012× at
5 M and 0.545× at 120 s), so the metric must include a fixed cap, not only the
probe. (ii) The widened sample may be non-predictive — that is item 49's own
question and a result, not a failure. (iii) A metric that ranks known levers
can still miss a new axis: it is a gate, not a proof.

## Ideation

- **A (recommended): fixed-context metric + widened sample.** Gain: a tree
  lever costs minutes, not a 120 s × seeds session; the next lever is judged
  by a gate that can see it. Cheapest: Phase A.
- **B (changes the formulation): make the metric the primary gate for tree
  changes (item 51), with the 120 s matrix as confirmation only.** Gain:
  removes the counter-identity requirement that has blocked every tree lever.
  Cheapest: the same Phase B.
- **C: a standing "required improvement" table** — what tree/per-node
  combination reaches `T_5` — so every plan is priced against the goal, not
  against `T_4`. Gain: no plan can quietly optimise for the wrong rung.
  Cheapest: one page, no runs.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan44.md`. Execute
   Phase 0: fix the metric and widen the sample (item 49) on N fixed non-claim
   `corpus-v1` cells; then Phase A reproduction on the three cells and the 5 M
   probe, and Phase B ranking of NULL-PIPE, HOTPATH-V2 and G1. Apply
   G-44a–f. No engine change, no default, no `bench.md` row, no `T_4`/`T_5`
   claim. Data in `measurements/plan44/`, `report44.md`."
2. If the metric ranks known levers: use it to re-test the length axis (item
   47) and the w = 2 × `L` composition on the widened sample.
3. If it does not: record the negative and take item 50 (EFFECT-SHAPE) as the
   next decisive test.
