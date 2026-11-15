# Plan 38 — kill review (plans 36–37) and item 52 (proposed, SHORT-PASS): price proof length before more per-node work

Read only: [`report36`](report36.md)–[`report37`](report37.md) and their
[`measurements`](../../../measurements/plan36/README.md), the plan33 row of
[`bench.md`](bench.md), [`analysis1`](analysis1.md) §3, [`analysis2`](analysis2.md)
§5–§6, D45–D49, [`gotchas`](gotchas.md). Pins unchanged: oracle `48686bc`
(v0.1.0), idea source `bceb1ee` (v0.4.0), `atomic-movegen` 2.8.0.

## Kill review (fixed prompt)

### 1. Absolute distance to the goal

`T_3`, Cap-120 row: `core` 4.923 s, `easy-win` 0.376 s, `deep-win` **53.42 s**
(5,803,257 expansions, length 36), geomean **6.606**. `T_4` is `deep-win`
**5.34×** away; `T_5` is **6.61×** geomean away: `core` is 9.22× and
`easy-win` 2.65× the reference already.

The decomposition is unchanged at **4.38× per node** (9,246 vs 2,111
ns/body), with **no valid tree ratio** (length 36/37/29 on `deep-win` against
mate 19; `core` 15 against 17; `easy-win` 19/19/16 against 15). The arithmetic
this review adds is the binding one:

| Route to `deep-win` ≤ 10 s | Required |
| --- | ---: |
| keep 9,246 ns/exp | **≤ 1.082 M** expansions (5.37× tree cut) |
| reach the reference's 2,111 ns/exp | ≤ 4.737 M expansions (1.23× tree cut); today's tree still takes **12.25 s** |
| reach `M` = 1,723 ns/exp | exactly 10.0 s on today's tree; no margin |

So **no single measured lever reaches `T_4`**. The per-node micro-line is
closed (census 3.22× ceiling, `M` band 1,559–2,201 against 1,723, overhead
class ≈1.20×), but the deficit is still required for `T_5`: even a perfect
per-node port leaves `T_4` 1.23× short and leaves `T_5` dependent on a tree cut.
Plans 34–37 closed a **route**, not the deficit.

The only measured tree target is proof length: on `deep-win` our shortest
committed proof is 29, our seed-0 proof 36, the reference's mate 19. That is
**1.21–1.89× shorter** than every current proof, with no measured tree factor.

### 2. If we started today, would we build this?

**Yes** to the prover, the hand guide, the full-key proof table and the two
checkers: that stack produced `T_3` and is the durable asset. **No** to this
plan sequence: 18+ plans of per-node micro-work after the guide stack, with
proof length visible but unmeasured for ~20 plans. Starting today, proof length
and a fixed-context tree metric would be first-class from M1; a body rewrite
would not be bought before a tree experiment prices its multiplier. The per-node
detour produced good measurements, but no route to `T_4`.

### 3. Which initiative or plan would you cancel?

- **Close item 44 and item 50 as a goal route.** `M` is INCONCLUSIVE-BY-UNITS
  (B floor 1,679 ns, C 2,201 ns); even B reaches `T_4` only if a full port
  lands at 1,679 ns/exp, below the reference's own 2,111, and C is 12.8 s.
  Plan34's rewrite was priced at 1.18–1.46×, not 4.38×. Keep item 44 only if
  the owner wants a `T_4`-only gamble; do not fund item 50 as its prerequisite.
- **Cancel item 41 as a `prover` dependency.** Ceiling measured at ≈1.11×;
  it stays a `movegen` wishlist line. Report37 does not change this.
- **Cancel items 53 and 49 as standalone plans.** They are instruments, not
  levers. Absorb the 5 M / fixed-cap numbers into item 52's gates; widen the
  sample only after a tree lever clears its first gate.
- **Keep item 51** (tree-change gate) and **item 48** (guide quality), the
  latter after length, as the second tree axis.
- **Do not plan the plan37 follow-up micro-candidates** (`populate_state`,
  `Mover::effect`) as a route; ≈1.20× for the whole class.

### 4. Prior-art delta (analysis1 §3; pin unchanged ⇒ changed rows only)

**R25 (length-limited passes) leaves “skip for now” and becomes active.** The
number against the plateau: every current `deep-win` proof is longer than a
bound of 24 (29/36/37 against the reference's 19); a real bounded pass at
`L = 24` would either find a proof at least 1.21× shorter than our best seed or
fail. The cheap flag test is exhausted: this review's probe of the
existing `--max-depth 24` on `deep-win` exits `depth` after **3,167,606 child
evals / 1.024 s** with no proof, because it censors at the first depth hit
rather than re-searching within a bound. R25 therefore needs item 52, not
another flag run.

R17 (memory) remains not a wall row: the proof table is not full (215,267
entries at the 5 M probe), and D35 already resolved `T_3`. R22 remains small
(45 of 153 k expansions). R23/R24 keep. R9 remains closed negative; R1–R6,
R11–R13, R18/R19 adopted. Literature delta: length-limited / iterative-deepening
proof-number search is already known (PNS 1994, PN* 2001, deep df-pn 2017); no
new mechanism beyond R25. No full re-read (pins unchanged); nothing from the
clone enters the repo.

### 5. Smallest next experiment

**One real bounded pass at `L = 24`**, not a harness and not another
`--max-depth` run: at `L = ∞` it must reproduce the 5 M and 120 s trees; at
`L = 24` it either returns a proof of length ≤ 24 on `deep-win` or says "no win
within 24". That single run prices R25. If it fails at 5 M and 120 s, the
length axis is not a quick win and the owner takes the goal question; if it
proves, item 52's deepening driver has a measured target.

## Rulings on plans 34–37

- **Plan37’s LEVER verdict is accepted as qualified**: on the measured regions
  the untimed symbols are 102 % of `other` and retired instructions are 74.3 %;
  that is a real local lever.
- **G-46a is PARTIAL and G-46d fails only on `outside`**: the census `outside`
  is a plug (`wall − measured regions`), not a region. Literal G-46d would force
  INCONCLUSIVE; the review accepts LEVER for the measured partition and
  excludes the plug. This is a census defect, not a reason to re-run the census.
- **The 28.45 pp class is ≈16.5 pp / ≈1.20×**, so it does not reopen the
  per-node route. Plan37’s Phase 0 arithmetic defect is recorded, not fixed.

## Item 52 (proposed) — bounded-length pass, Phase A

**Mechanism.** Carry a remaining-proof-length bound per frame. At an attacker
node a bound of 0 means “no win within `L`”; at a defender node every reply
must fit. Proofs are stored only in the full-key table; **bound-dependent
`Refuted` values are never stored**. The unbounded `L = ∞` control is the
current search and must be tree-identical.

**Phases.** A: bound + opt-in flag (`--max-length L`, default off/`∞`), unit
tests for attacker/defender/repetition/50-move boundaries, `make test-full` and
`make veto`. B: only after A, iterative deepening `L` = 12, 14, … behind an
opt-in flag. This plan does not adopt a default (D49).

**Gates.** G-38a `L = ∞` reproduces 5,803,257 / 155,236,740 / 36 at 120 s and
5,000,034 at the 5 M probe; G-38b every bounded proof is D14-accepted by both
checkers; G-38c no bound-dependent refutation can enter the table (test);
G-38d report `L` = 20/22/24 on `deep-win` at 5 M and 120 s, with length, wall,
expansions, `child_evals`; G-38e `make test` + `make test-full`.

**Scope / effort / risks.** In: search/frame bound, CLI flag, tests,
`measurements/plan38/`, `report38.md`. Out: default change, `T_4` claim,
formulation change, guide tuning, `bench.md` row. ≈1–2 days. Risks: the bound
adds per-node cost; a long proof at `L = 24` may simply not exist; bounded
refutations can be mistaken for proofs if the table rule is wrong (G-38c).

## Follow-up options

1. **Kickoff (recommended):** “Read `docs/plans/prover/plan38.md`. Execute item
   52 Phase A as written: add an opt-in remaining-length bound with
   bound-independent table storage, prove the `L = ∞` control tree-identical
   (5,803,257 / 155,236,740 / 36 at 120 s; 5,000,034 at 5 M), then measure
   `L = 20/22/24` on `deep-win` at 5 M and 120 s with D14 for every proof.
   Data in `measurements/plan38/`, report `report38.md`; no default, no
   `bench.md` row, no provari source.”
2. If `L = 24` proves: scope Phase B deepening and item 51's gate.
3. If it fails: owner rules between item 48 (guide quality) and the `T_4`/`T_5`
   goal question; do not reopen per-node micro-plans.
