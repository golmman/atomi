# Plan 42 — kill review (plans 40–41) and item 55 (proposed, SCALE-LENGTH): compose the two measured tree levers

Read only: [`report40`](report40.md), [`report41`](report41.md),
[`report38`](report38.md), [`report39`](report39.md),
[`analysis1`](analysis1.md) §3, [`gotchas`](gotchas.md), D33/D47/D49, the
plan33 row of [`bench.md`](bench.md). Pins: `atomic-movegen` 2.8.0; oracle
`48686bc`; binary `dd3c07c6…` at `4e973d2`.

## Kill review (fixed prompt; covers plans 40–41)

### 1. Absolute distance to the goal

`T_3`, Cap-120 product row: `core` 4.923 s, `easy-win` 0.376 s, `deep-win`
**53.42 s** (5,803,257 expansions, length 36), geomean **6.606**, 3/3, D14
9/9. `T_4` is `deep-win` **5.34×** away; `T_5` is **6.61×** geomean away.

**w = 2 is a real `deep-win` lever:** median 30,401 ms against 50,327 ms
(**0.604×**), geomean 1.0249×, D14 9/9 both checkers. If adopted, `T_4`
distance falls 5.34× → **3.04×**. The default stays 16 only because the
held-out guard was unmeasurable: all ten 120 s `sample20` runs censored, and a
600 s run proves one cell at 584 s, so the check was blind, not negative.

Plan38's `L = 24` is the other measured tree lever: 37.5 s against 53.3 s
(1.42×), length 24. But w = 2 already gives length **25** on seed 0, so the
two levers **overlap on the length axis**; their composition is not
multiplicative by assumption. No measured single axis reaches `T_4`; the live
axes are the two tree levers and their composition.

### 2. If we started today, would we build this?

**Yes** to the prover, hand-guide stack, full-key proof table and two checkers:
that asset produced `T_3`. **No** to the 20-plan per-node detour and standalone
instruments. w = 2 is a one-parameter guide change; the `L` pass is an in-base
search-order change (D47). Both would be first-class from M1.

### 3. Which initiative or plan would we cancel?

- **Item 48** (terms): closed negative; do not re-open.
- **Item 52 Phase B** (deepening): closed negative; keep `--max-length`.
- **Items 44/50 as a goal route**: close unless the owner wants a `T_4`-only
  gamble; `M` is INCONCLUSIVE-BY-UNITS at 1.18–1.46×, not 4.38×.
- **Item 41** stays a `movegen` wishlist line, not a `prover` dependency.
- **Items 53/49**: absorb the held-out fix here; widen the sample only after a
  tree lever clears a gate. **Keep 51**; fold item 47's scoping into it.
- **No plan37 micro-candidates** (`populate_state`, `Mover::effect`): ≈1.20×.

### 4. Prior-art delta (analysis1 §3; pin unchanged ⇒ changed rows only)

**R2 (banded EF-PN)** is re-argued with a number: plan41 tested the **weight**,
not the band. The band's step distribution is an eval property, and with w = 2
the eval is no longer a near-constant bias, so R2 is re-testable **after** the
w = 2 default, at the same 20 M gate. **R25 (length-limited passes)** is
active: plan38/39 closed *deepening*, but the `L = 24` pass is a real 1.42×,
and its composition with w = 2 is untested. **R17** is not a wall row; **R22**
skip; **R23/R24** keep; **R9** closed; **R11–R13** adopted; **R18/R19**
adopted; **R26** skip.

### 5. Smallest next experiment

**One `deep-win` seed-0 scan at w = 2 over `L ∈ {∞, 24, 22, 20}`** (4 runs,
120 s), against the w = 2 control (30,401 ms / 3,165,142 / length 25). If no
`L < ∞` beats `L = ∞` at w = 2, the composition is closed and the next move is
the held-out fix plus the owner's w = 2 ruling. If one does, the matrix
follows.

## Item 55 — SCALE-LENGTH (proposed)

**Question.** Does the w = 2 EF-PN weight compose with a bounded-length pass,
and does the composition cut `deep-win` wall beyond either lever alone? This is
the surviving tree question of report41 §8.2 and report38/39, not a new eval
term, not a per-node micro-plan, not a deepening schedule.

**Phase 0 — read-only scan (execution session; evidence, not a claim).**
`deep-win`, `--guide hand --memory 192 --seed 0 --seconds 120
--guide-ef-weight 2 --max-length L`, `L ∈ {∞, 24, 22, 20}`, one process at a
time, same binary. Record wall, `expansions`, `length`. Pre-registered: if no
`L < ∞` beats `L = ∞` at w = 2, **stop**; the composition is closed negative
and no matrix is run.

**Phase A — full matrix.** Best `L` from Phase 0 (or `L = ∞` if none), 3 cells
× seeds 0–2, 120 s, w = 2, same-session w = 16 control. Record wall,
`child_evals`, `expansions`, `length`, proof entries. No per-cell weight or
`L`.

**Held-out direction check (G-42a).** Five fixed cells from `guidance-v1`
shared wins (not `sample20`, censored by construction), 120 s, seed 0, w = 2
vs 16, **0-based indices stated in the driver**. Direction passes if w = 2 is
faster on **≥ 3/5** cells where both sides prove at 120 s; if fewer than three
prove, use a 600 s cap and state it.

## Gates

- **G-42a** Phase 0 table reproduced; held-out direction ≥ 3/5 on a proving
  set with the index base fixed.
- **G-42b composition** — `deep-win` median wall ≤ **0.50×** the w = 16
  control (≤ 26.7 s), all 9 runs proved, no cell censored, median geomean
  ≤ **1.05×** control. If `L = ∞` is the best, the gate is w = 2 alone
  (already measured: 0.604×, geomean 1.0249×) and the plan records the
  composition as a no-op.
- **G-42c D14** — every proof at the selected stack accepted by
  `atomi validate` and `provari check`; the timed run's `child_evals` matches
  the untimed `--seconds 600 --proof-out` run 9/9.
- **G-42d default** — only on G-42b/c **and** an owner ruling under D49. If
  adopted, set `GuideParams::default().ef_weight = 2` and (if `L < ∞`) the
  default `max_length`, pin both and the CLI round trip in unit tests, append
  one Cap-120 `bench.md` row. Otherwise keep w = 16, `L = ∞` and record
  negative.
- **G-42e** `make test`, `make lint`, `make test-full` + veto green.
- **G-42f boundary** — no new eval term, no model, no provari source, no
  `T_4`/`T_5` claim, no item 48 re-open, no item 49/51/53 work beyond the
  held-out fix.

## Scope, effort, risks

**Risks:** (i) w = 2 already shortens the proof to 25, so `L = 24` may be a
no-op or a regression — Phase 0 decides; (ii) the held-out set must be a
proving set, `sample20` is blind; (iii) the non-monotone w curve means w = 2 is
a measured point, not a tuning rule; (iv) a default change touches every
`--guide hand` run — G-42c/d and the full gate; (v) a default `L` needs the
owner's D49 ruling. Effort ≈1–2 days.

## Ideation

- **A (recommended): compose the two measured tree levers.** Gain: if
  multiplicative, `deep-win` ≈ 22.6 s (0.42×), `T_4` distance 5.34× → 2.26×;
  if not, a clean negative. Cheapest: Phase 0 scan.
- **B (formulation change): scale-free EF-PN.** Replace the constant `w` with
  a fixed quantile or running z-score map, removing the 60 % `f = 8`
  saturation. Gain: no arbitrary weight, possibly all three cells; cost: a new
  read point and a distribution. Cheapest: measure the eval distribution on
  the **search frontier**, then a fixed z-score map at the same 120 s gate.
- **C: per-node-kind weight.** Expose `EfPnMode::Both` and run w = 2 on both
  kinds. Cheapest: Phase A's per-kind expansion counts.
- **D: length-aware eval.** Prefer lines nearer the target length. Cheapest:
  Phase A's `length` column.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan42.md`. Record the
   kill review, then execute item 55 Phase 0: `deep-win` seed 0 at w = 2 over
   `L ∈ {∞, 24, 22, 20}`, 120 s, one process at a time. If an `L` beats
   `L = ∞`, run Phase A (3 cells × seeds 0–2 at the best `L`, same-session
   w = 16 control) plus the held-out check on `guidance-v1` shared wins with
   the index base fixed. Apply G-42b; on a pass run D14 and propose the
   default to the owner, otherwise record negative and keep w = 16, `L = ∞`.
   Data in `measurements/plan42/`, `report42.md`; no new term, no provari
   source, no `T_4`/`T_5` claim."
2. If the composition is a no-op: the next move is the owner's w = 2 default
   ruling with the fixed held-out evidence.
3. If w = 2 is too blunt: scope ideation B (scale-free EF-PN) as item 56, with
   the search-frontier distribution as its Phase 0.
