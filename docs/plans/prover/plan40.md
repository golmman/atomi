# Plan 40 — kill review (plans 38–39) and item 48 (proposed, GUIDE-EVAL)

Read only: [`report38`](report38.md), [`report39`](report39.md), the plan33
[`bench.md`](bench.md) row, [`analysis1`](analysis1.md) §3–§4,
[`plan19`](plan19.md)/[`report19`](report19.md), D8/D14/D21/D33/D47–D49,
[`gotchas`](gotchas.md). Pins: `atomic-movegen` 2.8.0; oracle `48686bc`.

## Kill review (fixed prompt; covers plans 38–39)

### 1. Absolute distance to the goal

`T_3`, Cap-120 product row: `core` 4.923 s, `easy-win` 0.376 s, `deep-win`
**53.42 s** (5,803,257 expansions, length 36), geomean **6.606**, 3/3, D14
9/9. `T_4` is `deep-win` **5.34×** away; `T_5` is **6.61×** geomean away.

Plans 38–39 changed the question, not the rung. **Length is a real tree lever
but not a product route:** `L = 24` is length 24 at 37.5 s vs 36 at 53.3 s
(1.42×). At 9,246 ns/exp, 10 s allows **1.082 M** expansions; the `L = 24`
tree is 4.109 M, still **3.80×** too large before the per-node 4.38×.
**Deepening is closed:** every `L < 20` prefix costs more than the whole `L =
24` pass (best `L = 16` 45.500 s / 132.5 M vs 37.190 s / 109.7 M); cumulative
650.95 M / 239.98 s against a 54.86 M / 18.60 s bar.

No measured single axis reaches `T_4`. The only untested in-tree lever is the
guide's **evaluation/ordering signal** (item 48); items 44/50 remain a
`T_4`-only gamble.

### 2. If we started today, would we build this?

**Yes** to the prover, hand-guide stack, full-key proof table and two checkers:
that asset produced `T_3`. **No** to the 20-plan per-node detour and standalone
instruments. Proof length and a fixed-context tree metric would be first-class
from M1.

### 3. Which initiative or plan would we cancel?

- **Item 52 Phase B** (deepening): closed by report39; keep `--max-length`,
  no driver or new table key.
- **Items 44/50 as a goal route**: close unless the owner wants a `T_4`-only
  gamble; `M` is INCONCLUSIVE-BY-UNITS at 1.18–1.46×, not 4.38×.
- **Item 41** stays a `movegen` wishlist line, not a `prover` dependency.
- **Items 53/49**: absorb the 5 M/fixed-cap instrument here; widen the sample
  only after a tree lever clears a gate.
- **Keep 51 and 48**; fold item 47's scoping into them.
- **No plan37 micro-candidates** (`populate_state`, `Mover::effect`): ≈1.20×.

### 4. Prior-art delta (analysis1 §3; pin unchanged ⇒ changed rows only)

**R25** moved *skip → active* in plan38 and is **closed for deepening** by
report39's prefix numbers; only the opt-in flag remains. **R2 (banded EF-PN)**
is re-argued with a number: plan19 rejected it at **1.2–1.35×** on every edge,
but with the *current* eval. The band's step distribution is an eval property,
so R2 is re-testable only **after** item48, with the same 20 M gate. **R4/R5**
were rejected at **1.9× on `core`**: their counting cost more tree than the
numbers bought. R17 is not a wall row; R22 is still 45 of 153 k expansions;
R23/R24 keep. No full re-read.

### 5. Smallest next experiment

**One scratch TERMS-V1 eval at the 5 M `deep-win` probe**, after a read-only
cost pass, against the control (5,000,034 child evals / 168,910 expansions).

## Item 48 — GUIDE-EVAL (proposed)

**Question.** Does a hand-cut term set — material, explosion targets, fuses,
pressure, checkers — beat the current `hand_eval` at like cost and reduce
`child_evals`/expansions on the product `--guide hand` stack?

The current eval is `material/32 + pressure*12 + one-sided signature*20 +
mobility/4`, clamped to `[-256, 256]`. It reads one side's threats only and
spends most of its cost in two full piece loops (`attacked_squares`,
`commoner_pressure`).

**Candidate TERMS-V1** (our own code; no model, D8), fixed scales, no tuning on
the three bench cells:

- material /32 and commoner pressure ×12 (unchanged);
- **signed** explosion targets: `signature_for(us) - signature_for(them)` ×20;
- **exposure**: non-pawn pieces in the opposing commoner's blast zone,
  difference ×8, capped at 3 per side;
- **fuse**: attackers on the enemy commoner's square minus ours, ×16, capped;
- **checkers**: `board.checkers().count()` for the side to move, ×16 penalty;
- mobility /4, or a cheaper proxy if Phase 0 finds it over budget.

All terms are bounded bitboard loops: no movegen, allocation or board build.
The eval is read where today's is (EF-PN child scaling, banding, simulation
score); no new keyed state, proof or table change.

**Phase 0 — read-only price and freeze.** A `/tmp` harness over `corpus-v1`
plus seeded legal playouts (≈5 k positions, the `testutil` construction)
measures current vs candidate **ns/call** (median of 3) and each term's
distribution/saturation; freeze formula and weights first. **G-40a:** candidate
≤ **199 ns/call**; over budget, trim mobility first, else stop.

**Phase A — scratch 5 M probe.** Copy `HEAD` to `/tmp/atomi-plan40/`, wire the
candidate behind a temporary switch, run `deep-win --guide hand --memory 192
--seed 0 --evals 5000000`, one process at a time, same session as the control.
**G-40b:** at 5,000,034 child evals candidate `expansions` ≤ **0.90×** control
(≤ 152,019), no proof/refutation change, 5 M control byte-identical. Fail ⇒
stop, report negative, no repo change.

**Phase B — repo opt-in, only on a pass.** Add `--guide-eval rich` (default off,
D49), tests for the algebra/bounds and `--guide hand`/`--guide none` identity,
then the three cells at 20 M and 120 s, seeds 0–2. **G-40c:** `--guide hand`
tree-identical to 5,803,257 / 155,236,740 / length 36 and 5,000,034 at 5 M.
**G-40d:** `deep-win` 120 s wall ≤ **0.85×** control, `length` not increasing,
D14 9/9; `core`/`easy-win` no wall regression > 10 %. On a pass it may become
the `hand_eval` of `--guide hand` (D21/D49) with a D14 matrix and `bench.md`
row; otherwise it stays opt-in.

**G-40e:** `make test`, `make lint`, `make test-full` + veto green; no GPL
dependency, provari source, binary or proof. **G-40f:** no default before
G-40d, no `T_4`/`T_5` claim, no `bench.md` row for the scratch probe; data in
`measurements/plan40/`, `report40.md`.

**Scope/effort.** In: review, Phase 0, scratch Phase A, Phase B only on a pass;
data and report. Out: default before the gate, search-base change, model,
memory/table change, items 44/50, provari source. ≈1–2 days.
**Risks:** richer eval costs more than the tree saves (G-40a/b); weights fitted
to the bench (fixed scales, non-bench sample); no wall win (G-40d/D49); the
5 M cap cannot see a late cut (D33); ±256 saturation.

## Ideation

- **A (recommended): TERMS-V1 into the existing read points.** Gain: the only
  untested in-tree signal; a 0.90× 5 M tree is a plausible first step. Cheapest
  experiment: Phase A.
- **B (formulation change): eval as ordering only, not number scaling.** Order
  children with TERMS-V1 and leave `pn`/`dn` untouched; removes the EF-PN
  distortion plan19 measured, but loses the number signal. Cheapest: the same
  5 M probe with `--guide-ef-weight 0` plus a TERMS-V1 tie-break.
- **C: asymmetric attacker/defender eval.** Separate OR-start and AND-defence
  functions. Cheapest: Phase 0's distributions split by node kind.
- **D: length-aware eval**, preferring lines nearer the proof-length target;
  needs a context the eval lacks. Cheapest: Phase B's `length` column.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan40.md`. Record the
   kill review, then execute item 48 Phase 0: price current `hand_eval` vs
   TERMS-V1 on `/tmp` over `corpus-v1` plus seeded playouts, freeze at ≤199
   ns/call, and run the scratch 5 M `deep-win` probe against 5,000,034 /
   168,910. If G-40b passes, implement opt-in `--guide-eval rich`, prove the
   `--guide hand` identity, and run the three-cell 20 M/120 s matrix with D14.
   Data in `measurements/plan40/`, `report40.md`; no default, no `bench.md` row
   unless G-40d passes."
2. If Phase A fails: close item 48 negative; the next move is the owner's
   `T_4`/`T_5` ruling, with reports 38–40 as the tree-axis evidence.
3. If G-40d passes: adopt TERMS-V1 into `--guide hand`, append the Cap-120 row,
   then re-test R2 on the new eval's step distribution.
