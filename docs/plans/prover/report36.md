# Report 36 — kill review of plans 34–35, and item 45 (M-REMAP): `M` is not the question, the decide region is

Plan: [`plan36.md`](plan36.md) (item 45 + the plans 34–35 kill review).
Initiative: [`initiative.md`](initiative.md), D30/D36/D40/D43–D47, proposed
D48. Data: [`measurements/plan36/`](measurements/plan36/README.md).
**No engine, flag, default, test or `bench.md` row changed**; no `ns` that
prices anything is the reference's (every rate is ours, report34 §4 verbatim);
nothing from the clone entered the repo — it was started as a process and its
own JSON result line was stored.

## 1. Verdicts

**Item 45 — INCONCLUSIVE-BY-UNITS.** `M` (`deep-win`, ns per expansion, against
the **1,723.17** ns `T_4` allows): **A 1,785.88** (report35 verbatim, +62.7) /
**B 1,678.60** (per-op, excess-only, −44.6) / **C 2,200.66** (region-total,
+477.5). `min` ≤ 1,723 < `max` under the plan's own rules, and the band does not
narrow under any reading of B's one judgement call (B0 1,678.60, B1 1,643.52,
B2 1,559.71). So a port of the reference's body shape is neither funded nor
refuted by this arithmetic: **only item 50's spike (the decide in `ns`/child)
can settle it**, at the owner's discretion.

**Phase 0 — the length precondition fires.** Our `deep-win` `length` is
**36 / 37 / 29** (seeds 0–2, 120 s, this session; seed 0 reproduces the
committed 5,803,257-expansion tree exactly), never ≤ 20, against the
reference's **`mate` 19**. The 2.76× "tree ratio" (5,803,257 expansions /
2,099,100 reference bodies) compares a mate-36 tree with a mate-19 one and is
**void**. Item 47 is live and **not** scoped by this plan.

**The kill review, as execution found it** (plan36's fixed prompt, five
answers): (1) the distance is `T_3` — `deep-win` **5.34×** on the wall,
**4.38×** per node — and the review's "per-node short by 3.6 %" is one
convention of three: +62.7 ns on A, −44.6 on B, +477.5 on C;
(2) no to the reshape as a shape port, yes to the two measurement items — item
45 says fund/refute is undecided, so the review's §3 stays *cancelled pending
item 50* rather than closed; (3) cancel item 44 as a shape port, deprecate
D36's tree clause and `analysis2` §6 (both now carried by proposed D48); (4)
the prior-art delta holds (pin unchanged, no full re-read); (5) the smallest
next experiment is item 50, and the probe's number is now **77.1 ns/child**
(the reference's decide loop at our region's rate is 1,403.9 ns/exp = 81 % of
the budget by itself).

| Gate | Bar | Result |
| --- | --- | --- |
| G-36a coverage | 21 buckets in exactly one class, 100 % of the op budget | **pass** — 18 priced (65.005 pp) + 3 folded (3.96 pp) = **68.965 pp** = the census's 68.97; `inside`/`unhashable` empty |
| G-36b arithmetic | convention A reproduces 1,785.9 ns ± 1 ns | **pass** — 1,785.88, Δ = 0.000 |
| G-36c no double pricing | asserted on the leaf set | **pass** — one charge keyed per bucket (18 priced + 3 folded at 0); the two counters that are two faces of one unit (effect + hash) enter one bucket's single charge; C's four region buckets carry one charge between them |
| G-36d decision | A, B, C, min, max, verdict | **pass** — unhashable 0 % of the op budget (≤ 20 %) |

## 2. Phase 0 — the length probe (read-only, 3 min)

One process at a time; 3 `atomi` seeds per cell (deep-win `--seconds 120`,
core and easy-win `--seconds 10`) + 1 reference run per cell, all
`--guide hand --memory 192`.

| Cell | our `length` | reference `mate` | our wall (median) | reference wall |
| --- | --- | ---: | ---: | ---: |
| `deep-win` | 36 / 37 / 29 | 19 | 49.045 s | 4.363 s |
| `core` | 15 / 26 / 20 | 17 | 3.987 s | 0.490 s |
| `easy-win` | 19 / 19 / 16 | 15 | 0.393 s | 0.107 s |

**The plan's unit check is wrong and is recorded, not fixed.** Plan36 said
"`core` 17 = our `length` 17 ⇒ unit check passes". Our `core` proof is **15**
attacker moves — plan33's committed Cap-120 row has the same 15, so this is not
session drift — and `easy-win` is 19 against 15. The length relation is
**cell-dependent**: shorter on `core`, longer on the other two. That matters
for item 47's scoping: the `deep-win` length gap is not "the guide writes long
proofs everywhere", it is one cell — the deepest one — where our proof is 1.9×
the reference's.

`deep-win`'s wall ratio this probe is **11.24×** (49.045 / 4.363); the committed
plan33 row is 11.81× (53.42 / 4.523). Both are *wall* ratios on trees of
different depth, so neither decomposes into a tree factor and a per-node factor —
which is precisely what D48 proposes to record.

## 3. Phase A — what the audit changed in report35's `M`

Three findings, in order of size.

1. **report35's 93× status/terminality row is a unit artifact** (the plan's
   premise, confirmed and quantified). Their per-reply legality is not the
   0.49-query terminality counter; it is `heavy_legality_tests`, **33.07 per
   body** against our own per-reply legality volume **34.08 per expansion**
   (`status@null_p1` 20.20 + `legality@null_p2` 13.88) — a **0.97×**, not a
   93×. In like units the row's ratio is **1.37×** and its addressable share is
   **2.55 pp**, not 9.39 pp. Total addressable pp: **51.01 → 44.17 (−6.84 pp =
   −632 ns)**, i.e. further from the 81.3 pp `T_4` needs, not closer.
2. **The per-child test families do not live where report35 put them.** It
   listed quiet (32.58), exposing (21.69), check (15.82), blast (19.07), threat
   (4.42), evasion (0.71) and heavy legality (33.07) as `inside` the 59.2
   per-body screen counter. They sum to **127.4 per body = 7.00 per child
   decided**, so no single 59.2 screen counter can contain them. Their true
   home is `child_decisions` (18.2 per body): they are the work of deciding a
   child. That reclassification costs `M` nothing (they were never priced) but
   it is what makes convention C well-posed.
3. **The one family whose ratio runs the other way is real**: their per-move
   screens are 59.23/body against our 26.75/exp (0.45×), so `M` prices more
   screens than we run and the leaf-wise addressable there is 0, not negative.

`M` under the three conventions, and what moves between them:

| Row(s) | A (ns) | B (ns) | C (ns) |
| --- | ---: | ---: | ---: |
| per-move screens (59.23/body) | 216.74 | **118.89** (excess 32.48 × 3.66) | inside the region |
| status/terminality (0.49/body) | 9.43 | **0.00** (excess 0 both families) | inside the region |
| child decide (18.20/body) | 686.41 | 686.41 | **1,403.89** (18.20 × 77.13 ns) |
| the other 17 rows (incl. key 76.53) | 873.3 | 873.3 | 796.8 (key inside the region) |
| **`M`** | **1,785.88** | **1,678.60** | **2,200.66** |

Addressable pp: leaf-wise **A 51.01 / B 51.11 / C 48.88**; wall-minus-`M`
**A 80.68 / B 81.84 / C 76.20** against the 81.3 needed. The wall-minus-`M`
reading clears the bar only under B, and it is the reading report35 §4 already
called weaker (it assumes our whole per-node scaffolding disappears with the
body). The leaf-wise reading — the one report35's verdict used — is short under
all three conventions.

## 4. Why the verdict is INCONCLUSIVE and not funded or refuted

* A is report35's own inclusive price; it fails by 3.6 %.
* B is deliberately generous: it charges only the excess over work **we already
  do**, i.e. it does not price a port's own per-child tests at all — a port
  living in our engine would pay our closure and our screens, which are inside
  the wall being priced, not inside `M`. B is a floor, not an estimate.
* C is the honest region reading: whatever their loop folds together, our
  decide region charges it once at **77.1 ns/child**.
* The distance between B and C is the whole question: **522 ns**, one region,
  one rate. Item 50 measures exactly that rate.

The plan's two verdict clauses (`FUNDED ⇔ min ≤ 1,723` and
`INCONCLUSIVE ⇔ min ≤ 1,723 < max`) overlap by construction. The third is the
refinement of the first and is applied whenever `max > 1,723`; that is recorded
in `remap.json` (`verdict.rule`) rather than silently chosen.

## 5. Phase B — the record

* **`initiative.md`**: the staleness note is now the corrected record; the
  status paragraph's "per-expansion cost (11.5–15.5×)" and "tree at parity, ≈8×
  per expansion to close" carry the correction (**4.38× per node**, 9,246
  ns/exp vs 2,111 ns/body; no tree term until item 47); D45's row keeps its
  text and gains a bracket that voids only its tree-at-parity premise. Items 45
  and 47 carry their new status. A new **proposed D48** row records the
  supersession.
* **`decisions.md`**: **proposed D48** — *wall decomposition corrected: per
  node, not tree × per-node*. It supersedes **D36's tree clause and its
  `tree × per-expansion` decomposition** (`analysis2` §6's `1.18 × 13.5`):
  their reported `nodes` is 1.44× their bodies, and the two engines prove
  different lengths on `deep-win`. D36's surviving content: item 29 closed
  negative, the wall is a first-order finding, the next work is inside the
  hand-guide stack.
* **`gotchas.md` §Scope**: "measure against ≈8×" → **4.38× per node**, with
  "quote no tree ratio in a plan".
* **`history.md`**: one line (above).

Nothing in the goal, the rungs, `check_goal.py` or `bench.md`'s ledger moved.

## 6. Problems met

1. **The plan's `core` unit check is wrong** (§2). Found because the probe runs
   it rather than inheriting it.
2. **`report35`'s "per child" labels are per body.** The seven test families
   read 21.69 / 15.82 / 19.07 / 4.42 (exposing / check / blast / threat) *per
   body* — 5.18 light tests + 1.82 heavy *per child decided*. Not fixed in
   report35 (reports are not rewritten); recorded here and in `remap.json`.
3. **B needs one judgement call** — which our op is "their screen". Handled as
   three pre-registered readings (B0/B1/B2); all three are inside the bar, so
   the verdict does not depend on the choice — but the *distance to the bar*
   moves by 119 ns across it.
4. **The status row's own rate (19.13 ns) is an average of five leaves** whose
   individual rates span 0.50 ns (`legality@null_p2`, clamped against the 7.6 ns
   tax) to 45.55 (`status@replay`). The like-unit charge is reproducible but its
   *meaning* is an average; a per-leaf re-pricing needs a per-leaf census of our
   legality sites, which the plan34 instrument does not have.

## 7. Missing tests / open parts

* **No census of our `core`**: the audit is `deep-win`-only, so `core`'s `M`
  (2,117 ns, report35) is un-audited and its like-unit correction is unknown.
* **The decide region in `ns`/child is priced, not measured.** 77.1 ns/child is
  a sum of four census buckets over our closure-call count; item 50 measures the
  real per-child cost of a mask-only decide.
* **Item 47 has no plan and no gate**; the length axis (R25 in our words) is
  still the only unported row on it, and this session only established that the
  precondition fires.
* **Item 46 (plan37) is untouched** — the 28.45 pp / 2,869 ns/exp no counter
  owns is the other open measurement, and it shares this audit's region-vs-leaf
  question.

## 8. Next steps

1. **Owner: rule on D48** (and on D47, still pending) — D36's tree clause and
   decomposition cannot stay quoted in plans either way.
2. **Item 50** (EFFECT-SHAPE) is now the only path to funding or refuting item
   44: `M` ≤ 1,723 ⇔ the decide ≤ **34.3 ns/child** against today's 77.1
   (region) / 37.7 (our closure leaf). Cheapest decisive experiment: the
   mask-only decide timed in `ns`/child, no wall, no reference.
3. **[`plan37`](plan37.md)** (item 46) — run before or after; it is the other
   half of the same question (what the regions outside our named calls cost).
4. **Item 47**: scope SHORT-WIN (R25's idea in our own words) knowing the gap is
   one deep cell (36 vs 19) and that we are already shorter on `core`.
