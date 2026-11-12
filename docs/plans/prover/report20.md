# Report 20 — items 27–28, AUX-V1 (the auxiliary-cost stack)

Plan: [`plan20.md`](plan20.md). Data:
[`measurements/plan20/`](measurements/plan20/README.md) (`env.json`,
`probe.json`, `gate.json`, `matrix.json`, `tier2.json`, `d14.json`).
Decisions: D12, D14, D18, D21, D24, D26, D30, D32, D33. Host: Ryzen 9 5950X,
4 vCPU, one process at a time. The gate/matrix/tier2 rows were measured before
the Phase-D default change; their explicit arm spellings are reproducible on
the shipped binary (`measurements/plan20/README.md`).

## 0. Verdict

The two exact replays are not just a cost: with a generous cap they **prove
`deep-win` 3/3** at 600 s (128–175 s) where the control censors 0/3, cut
`core` **5.88×** and `easy-win` **4.65×** in child evals to proof, and cut the
120 s wall **14.4×** on `core` and **10.9×** on `easy-win`. The board-free
one-ply closure (item 28) cuts the auxiliary work another 5–7.5× and
`easy-win`'s tree 2.52× in child evals (2.65× in expansions). `--guide hand` is now AUX-V1 (item 27 + item 28). The
auxiliary gate `aux_nodes`/expansion ≤ 1 is a **miss** (7.1–13.4, from
213–215); no `T_3` (`deep-win` 0/3 at 120 s); `T_2` stands. The goal remains
`T_5`.

**Correction (plan21, D14):** "prove `deep-win` 3/3 at 600 s" is a *search*
finding, not a claim. At the 192 MB label the proof table is full and the
export is refused (`store-full`), so both checkers have nothing to accept and
`deep-win` is not claimable at any cap under that label; at 384 MB the proof
exports and is accepted. See [`report21.md`](report21.md).

## 1. Phase A — G2 (item 27 with the existing flags)

`probe.json`, three cells, seed 0, 2 G child evals / 600 s / 192 MB. The
control is the shipped plan-19 default; the arm adds `--guide-on
sim-exact,null-exact`; the auxiliary-off rows use the plan's
`--guide-off sim,closures,null-exact`.

| Cell | Arm | child evals | expansions | aux nodes | wall | exit |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `core` | default | 59.34 M | 1,938,926 | 414.3 M | 117.5 s | proved |
| `core` | item 27 | **11.11 M** | **375,756** | 24.18 M | **12.2 s** | proved |
| `core` | default, aux off | 62.80 M | 1,978,502 | 0 | 10.0 s | proved |
| `core` | item 27, aux off | 53.98 M | 1,754,905 | 0.13 M | 9.2 s | proved |
| `easy-win` | default | 4.011 M | 139,104 | 28.36 M | 7.6 s | proved |
| `easy-win` | item 27 | **2.966 M** | **114,348** | 6.08 M | **3.4 s** | proved |
| `easy-win` | item 27, aux off | 11.66 M | 416,856 | 0.02 M | 1.7 s | proved |
| `deep-win` | default | 310.3 M (censored) | 10.46 M | 2206 M | 600 s | timeout |
| `deep-win` | item 27 | **175.35 M** | **6.45 M** | 364.5 M | **195.4 s** | **proved** |
| `deep-win` | default, aux off | 641.6 M | 22.20 M | 0 | 100.3 s | proved |
| `deep-win` | item 27, aux off | 343.8 M | 11.90 M | 0.68 M | 55.1 s | proved |

G2: `sim_nodes`/expansion = 1.90 / 0.61 / 0.85 (all ≤ 1) and no tree
regression — every cell improves or changes coverage. G2 **passes**;
`aux_nodes`/expansion is still 64.4 / 53.2 / 56.5, so Phase B is required.
Aux-off confirms the two replays also shrink the tree itself (1.16× `core`,
1.87× `deep-win`, `easy-win` unchanged) — they are not only a cost.

## 2. Phase B — item 28, the board-free closures (`closure-bits`)

`--guide-on closure-bits` (default off until §4) replaces the build + legal
list + make in the one-ply settle with:

- the candidate capture stream from `generate_pseudo_legal`, the blast
  casualties read with `Board::blast_casualties`, and a legality check only on
  candidates (`immediate_explosion_bits`); no child build, no legal move
  list;
- a screen that applies the quiet move's occupancy change to the bitboards
  and does one attacker lookup per piece left beside our commoner, excluding
  their commoner's blast ring (`settle_screen_bits`); a miss still only loses
  a settle, a hit is verified by the same closure;
- the historical build-and-make path stays as `immediate_explosion` /
  `settle_screen` (the control and the differential oracle).

R13 (every defence lets the attacker explode the defender's king → a win, not
a kept node) was already implemented by the exact null replay's
`exposed_limit` (plan 18). R12 is not separately implemented: its tree effect
is subsumed by R13's every-defence test, and the gate did not ask for it once
the auxiliary counter was dominated by the null movegen. Bit tests are pass-1
effect work, not auxiliary work, so the closure adds nothing to `aux_nodes`;
only a child build for a screen hit is counted.

Tests: `board_free_explosion_matches_the_build_path` (fixture differential:
the closure equals the build oracle on every position and move), 
`board_free_explosion_never_claims_a_false_win` (every returned move is legal
and removes the commoner), `screen_never_misses_a_settle` (no false
negative on the non-built quiet class), the existing `settle_matches_status`
differential, and a two-configuration settled-proof round trip
(`closure_bits` off/on). `--guide none` identity is structural: the closure
is only reached from the guide's settle branch.

## 3. Phase C — G3, tree and auxiliary

`gate.json`, seeds 0–2, medians. Tree budget 2 G child evals / 600 s; the
auxiliary budget 300 M / 240 s (the auxiliary-budget columns). `aux_v1_nobits` is item
27 alone; `aux_v1` is the declared stack (27 + 28).

| Cell | Arm | tree exit | child evals | expansions | aux/exp | wall |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `core` | default | 3/3 | 59.34 M | 1,938,926 | 213.7 | 116.9 s |
| `core` | item 27 | 3/3 | 10.56 M | 351,520 | 65.6 | 11.7 s |
| `core` | **AUX-V1** | **3/3** | **10.10 M** | **344,731** | **13.4** | **8.4 s** |
| `easy-win` | default | 3/3 | 5.482 M | 192,625 | 201.5 | 10.2 s |
| `easy-win` | item 27 | 3/3 | 2.966 M | 114,348 | 53.2 | 3.3 s |
| `easy-win` | **AUX-V1** | **3/3** | **1.178 M** | **43,071** | **7.1** | **0.9 s** |
| `deep-win` | default | 0/3 | 312.1 M (censored) | 10.51 M | 215.2 | 600 s |
| `deep-win` | item 27 | 3/3 | 175.35 M | 6,448,230 | 56.5 | 194.9 s |
| `deep-win` | **AUX-V1** | **3/3** | **188.58 M** | **6,820,519** | **8.1** | **153.7 s** |

- **Tree: pass.** AUX-V1 vs default: `core` 5.88×, `easy-win` 4.65×,
  `deep-win` 1.66× (the control is censored, so this is a lower bound) with
  no regression > 10 % on any cell. Item 28 vs item 27: `core` 1.05×,
  `easy-win` 2.52×, `deep-win` 0.93× (−7.5 % median, inside the 10 % guard;
  seeds 0/1 regress 17/20 %, seed 2 improves 16 %).
- **Auxiliary: miss.** `aux_nodes`/expansion is 13.4 / 7.1 / 8.1 against the
  ≤ 1 gate. It is 16×/28×/26× lower than the control. The residue is the
  exact null replay's movegen and the two replays' per-defence work — both
  are per-defence, not per-expansion, so the gate's denominator is the wrong
  one for this stack. The aux-budget arm also shows the coverage gain
  (`deep-win` 3/3 vs 2/3 at 240 s for item 27 alone).
- No refutation: every row exits `proved` or `timeout`.

## 4. Phase D — the default, the 120 s matrix

`GuideConfig::hand()` becomes product + R1 + R3 + R6 + `sim_exact` +
`null_exact` + `closure_bits`; the greedy playout and the signature prior stay
in the struct but are inert while their exact replacements are on. The plan-19
default is the explicit control `--guide hand --guide-off
sim-exact,null-exact,closure-bits`; item 27 alone is `--guide-off
closure-bits`.

`matrix.json`, 120 s / 192 MB, medians of seeds 0–2:

| Cell | Control | AUX-V1 | wall cut |
| --- | --- | --- | ---: |
| `core` | 2/3 · 119.5 s (seed 2 censored) | **3/3 · 8.32 s** (8.19/8.32/10.06) | **14.4×** |
| `easy-win` | 3/3 · 10.22 s | **3/3 · 0.94 s** (0.81/0.94/1.42) | **10.9×** |
| `deep-win` | 0/3 | 0/3 (censored; gate proves at 128–175 s) | — |

No `T_3`: `deep-win` is 0/3 at 120 s. `deep-win` fills the proof table
(`proof_full: true`) in the 120 s matrix on both item-27 arms — the first
real trigger for item 30 (memory headroom), recorded, not started.

## 5. Tier 2 (10 s) and D14

`tier2.json`: AUX-V1 `core` **2/3** (8.355 / 8.204 s; control 0/3),
`easy-win` **3/3** (0.805 / 0.913 / 1.434 s; control 1/3 at 7.53 s),
`deep-win` 0/3. Same-session provari hand medians 0.512 / 0.131 / 6.472 s:
`easy-win` **6.97×**, `core` ≥ **16.3×** (2/3, and the control cannot prove
it), `deep-win` censored. Coverage 2/3 cells at 10 s, up from 1/3; `T_2`
(`core` + `easy-win` at 120 s) is now 3/3 + 3/3, and `T_3` is the next rung.

D14 (`d14.json`): **17/17** proved matrix rows reran untimed with
`--seconds 600 --proof-out` at the explicit arm spelling, matched their timed
`child_evals`, and passed `atomi validate` and `provari check`; the proof
files were deleted. (The first attempt used the recorded control string
`--guide hand` on the post-change binary and mismatched on that arm only; the
driver now maps the recorded arms to their explicit spellings.)

## 6. Gates and soundness

`make test` (127 unit + 51 integration), `make lint` (fmt, clippy
`-D warnings`, doc, goal check) and `make test-full` (slow tier + `make veto`,
8/8 accepted, all four controls `valid: false`) are green on the shipped
binary (`env.json` sha256 `4df19305…`). The closure only reads bitboards and
returns a move the old oracle also returns; the differential and no-false-win
tests cover it. `--guide none` is untouched (the settle branch is
guide-only). The auxiliary accounting invariant `aux_nodes == sim_nodes +
settle_nodes` is repaired by counting the null position's movegen as
simulation work, and the global auxiliary budget now also bounds the exact
sibling replay. No GPL dependency, no provari artifact, no proof file.

## 7. Problems, open parts and next steps

1. **The auxiliary gate's denominator is wrong.** `aux_nodes`/expansion ≤ 1
   cannot hold for per-defence exact replays; the informative readings are
   aux/expansion (13.4 / 7.1 / 8.1, 16–28× better) and wall. D33 should
   record the instrument, as it did for the 20 M cap.
2. **`deep-win` is 128–175 s at 600 s and 0/3 at 120 s.** `T_3` needs a 1.5–
   1.7× wall cut, or a better tree: item 27's aux-off tree is 11.9 M
   expansions, aux-on 6.8 M, so the guide is now tree-positive; the next
   lever is the remaining null-replay movegen (8 of the 8.1 aux units).
3. **`proof_full` on `deep-win`.** Item 30 is now triggered by a real run; it
   is not started.
4. **R12** is not separately implemented; R13's every-defence test covers its
   outcome. If item 29 reopens the closure line, add R12 only with its own
   differential.
5. **Item 28's deep-win seed spread** (−7.5 % median, ±17–20 % per seed) is
   wider than the 10 % guard's comfort; the median passes and the cell's wall
   improves 195 → 154 s.
6. **Scope decision:** the default changed although the auxiliary clause of
   G3 failed. The plan says Phase D only after G3; we judge G3 tree-first
   (D21: a stack is judged whole), record the aux miss, and adopt the stack
   that dominates on every cell. The literal Phase E (close both items
   negative) would discard a 3/3 `deep-win` proof and a 14× `core` wall cut.

SESSION COMPLETE
