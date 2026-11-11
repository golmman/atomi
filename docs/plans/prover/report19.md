# Report 19 — item 26, NUMBERS-V1 (the starting-number algebra)

Plan: [`plan19.md`](plan19.md). Data:
[`measurements/plan19/`](measurements/plan19/README.md) (`env.json`, `a.json`,
`probe.json`, `gate.json`, `tree.json`, `matrix.json`, `tier2.json`).
Decisions: D12, D14, D21, D26, D30, D32, D33. Host: Ryzen 9 5950X, 4 vCPU,
one process at a time. Binary pinned in `env.json`; the tree/gate rows were
measured before the default change (§5), the matrix after it.

## 0. Verdict

The starting-number algebra is the first >2× **tree** lever since the guide
was ported. Tree-only, medians of seeds 0–2, the adopted arm cuts `core`
**4.74×**, `easy-win` 3.34× and `deep-win` 2.80× against the product hand
(§4). The declared six-mechanism stack misses the gate on `core` (1.78×), so
G3 is judged on the post-gate both-direction ablations (D21): **R1, R3 and R6
are adopted; R2 (banded EF-PN) and R4/R5 (exact class numbers) are rejected
with data** — R2 costs 1.2–1.35× at every probed edge, R4/R5 cost 1.9× on
`core`. `--guide hand` is now R1+R3+R6 (§5). The wall did not follow the
tree: the auxiliary passes cost ~30× per node (analysis1 §1.3) and now
dominate it, so the 120 s matrix is a wash on coverage and items 27–28 (aux
cost) are the next lever. No `T_3`, no rung change.

## 1. Phase 0 — the CLI order trap (D33)

`--guide hand|none` assigned a fresh `GuideConfig` where it appeared, so
every guide switch given *before* it was silently dropped. New
`cli::GuideArgs` collects the base and the switches and applies **base first,
then the switches in argv order**; `prove` and `classify` share it. A
value-less switch (`--guide-group`, `--guide-sim-exact`, …) never sets
`enabled`, so `--guide none --guide-sim-exact` stays the bare tree (the
plan-18 identity contract) while `--guide-on X` still enables the guide.
Four unit tests cover both orders, the base-then-switches order, the
`--guide none` identity and the `numbers` aggregate. Counter identity against
HEAD `e71289b` (rebuilt in `/tmp/atomi-plan19-base`): `--guide none` 500,038
/ 15,260; `--guide hand` 1,000,003 / 28,996 / sim 4,912,778 / settle
1,803,065 — identical in both binaries.

## 2. Phase A — reproduction, and a broken instrument (G2)

Plan's instrument (20 M child evals, seeds 0–2, expansions; `a.json`):
`core` bare 630,056 / 623,703 / 626,795, product 648,828 / 653,551 / 647,032,
flag stack 600,894 / 601,453 / 601,438 (all censored); `easy-win` bare
censored at 669,920 / 667,086 / 665,909, product proved at 468,301 / 631,763
/ 211,254, flag stack proved at 139,254 / 134,325 / 196,891; `deep-win` all
censored at 636 k–669 k.

**The 20 M instrument cannot see a tree cut while censored**: every censored
row costs ~`child_evals/32` expansions (630 k / 649 k / 601 k on `core`),
because expansions count expanded nodes and the branching factor, not the
guide. `G2` as written therefore "fails" (flag stack 1.05× of bare) although
the premise holds (product hand 0.97× of bare = the net loss, confirmed).

On the analysis-1 §4 instrument (auxiliary passes off, `--seconds 120`) the
reproduction is **exact** on all three cells: `core` 3,588,914 expansions
(proved, length 17, 18.16 s) = analysis1's 3,588,914; `easy-win` 670,065 =
670 k; `deep-win` 23,425,762 = 23.4 M. The premise is reproduced; the gate
instrument is replaced in §4.

## 3. Phase B — the six mechanisms (implemented, flags, default off)

All six sit behind `--guide-on/off` names; `--guide-on numbers` turns the six
on. `StartInput` carries a fresh child's inputs (class, R4 number, AND class,
evasion count, child eval, sim score, legacy-prior flag); `start_numbers`
applies, in order: base pair (R1 or the product table), `global_scale`, R6's
absolute pair, the history prior, R2's band, the legacy per-child EF-PN, the
simulation, the product `UNLIKELY` prior.

- **R1 `fresh-class`** — `(class, 1)` at an OR node, `(1, class)` at an AND
  node with the AND class ∈ {1, 2}; the class pn comes from `--guide-params`
  (quiet/threat/weak/hanging) and from R3 for evasions.
- **R2 `efpn-banded`** — step ∈ {0,1,2} from the parent's eval (attacker
  oriented), one edge in `hand_eval` units (`GuideParams::ef_band`), factors
  EF-PN's at a=10, b=5: OR `pn ×{11,6,1}`, `dn ×{1,11,21}`; AND the mirror
  (`pn ×{21,11,1}`, `dn ×{1,6,11}`) — the plan gave the OR row only, the AND
  row is provari's defending branch.
- **R3 `evasions-count`** — `class::surviving_evasions`: the evasion
  children still `Open` after pass 3, capped at 8 (never 0), replacing the
  parent's legal count. A post-pass, no second build, no movegen.
- **R4 `threat-count`** — the number of enemy pieces a capture would explode
  the enemy commoner from, after the move: built children hand their own
  threat signature, quiet unbuilt ones are counted from the parent with the
  move's occupancy and the moved piece's new attack set (≥2 → 1, 1 → 4, 0 → 9).
- **R5 `hanging-quiet`** — `class::blasted_cheaply`: a quiet move whose moved
  piece an enemy capture takes, blast included, without losing material in
  our own table (pawns 100 … queen 900, commoner excluded); hanging is 15.
- **R6 `unlikely-exact`** — a defence that is not a check, does not blast an
  attacker piece and brings no king contact starts as `(1, 2)` outright,
  replacing the product `pn/2, dn*2` prior (skipped when R6 is on — that fix
  alone cut the adopted arm 1.31× on `core`).

Tests: R1 both node kinds and `(1,1)` for `--guide none`; R6's absolute pair;
R2's three steps at both node kinds and edge boundaries; R3's 1-survivor and
>8-survivor cases; R4's count→number mapping plus a differential that the
board-free quiet path equals the built child's signature over the fixture
playouts; R5's material cases; R6's predicate; `ef_band` round trip; the
mechanism registry; `hand()` equals the adopted arm.

## 4. G3 — tree cut and the ablations

`plan19.md` judges the gate on `expansions` at `--evals 20000000`; §2 shows
that instrument is blind for censored trees, so the gate runs at
`--evals 300000000 --seconds 240` (`gate.json`) and the tree cut — the gate's
actual subject — with the auxiliary passes off at `--evals 2000000000
--seconds 600` (censored rows are lower bounds), three cells × seeds 0–2 ×
four arms (`tree.json`). Tree-only, medians of seeds 0–2:

| Arm | `core` | `easy-win` | `deep-win` |
| --- | ---: | ---: | ---: |
| product hand | 12,785,169 | 1,397,333 | 66,875,512 (2/3; seed 2 censored) |
| declared NUMBERS-V1 (R1–R6) | 7,195,804 | 693,035 | 23,792,249 |
| **adopted (R1+R3+R6)** | **2,699,587** | **418,896** | **23,908,211** |
| cut, declared vs product | 1.78× | 2.02× | 2.81× |
| **cut, adopted vs product** | **4.74×** | **3.34×** | **2.80×** |

G3 as declared on the six-mechanism stack: `core` **fails** (1.78× < 2×),
`easy-win` passes (2.02×, no regression vs bare), `deep-win` passes (2.81×).
G3 on the adopted arm after the D21 ablations: **all three pass**. The R2
band probe (`probe.json`, `core`, seed 0, tree only) fixes the plan's edge at
**128** — the best of {64, 128, 192} (6,706,622 / 6,072,637 / 6,559,477
expansions) — but every band is worse than no band (4,985,746), so R2 is
rejected, not tuned. R4/R5 isolated: with the band off, R1+R3+R6 = 2,598,262
vs R1–R6 = 4,985,746 (+92 %) — rejected. R1 isolated: without it the same
stack is 8,030,957 (+32 %; +92 % with the band off in the early probe) —
accepted; R3 is neutral-to-slightly negative (4,808,847 without it in the
early probe), kept for parity. Adopted vs bare: 3.43× on `core`, 2.97× on
`easy-win`.

## 5. Phase D — the default, the 120 s matrix, D14

`GuideConfig::hand()` becomes R1+R3+R6 (the adopted arm): `fresh_class`,
`evasions_count`, `unlikely_exact` on; `efpn_banded`, `threat_count`,
`hanging_quiet` off; every other product switch (classes, evasions, quiet
grouping, the legacy per-child EF-PN, closures, priors, history, sim, null
replay) unchanged. `GuideConfig::product()` keeps the pre-plan19 stack; the
explicit spelling `--guide hand --guide-on numbers --guide-off
efpn-banded,threat-count,hanging-quiet` resolves to the same config (pinned
by `hand_is_the_adopted_numbers_arm`).

The 120 s matrix (`matrix.json`, seeds 0–2; "adopted" = the shipped
`--guide hand`):

| Cell | bare | product hand | **adopted (new default)** | adopted, aux off |
| --- | --- | --- | --- | --- |
| `core` | 3/3 · 18.4 / 23.3 / 18.9 s | 0/3 (censored, 1.96 M exp) | **2/3 · 118.0 / 115.1 s**, seed 2 censored | 3/3 · 10.2 / 14.1 / 14.4 s |
| `easy-win` | 3/3 · 1.8 / 2.3 / 2.2 s | 3/3 · 22.0 / 29.8 / 10.6 s | **3/3 · 7.6 / 16.8 / 10.4 s** | 3/3 · 1.7 / 1.4 / 1.7 s |
| `deep-win` | 0/3 | 0/3 | **0/3** | 2/3 · 101.1 / 117.1 s |

The default proves `core` where the product hand cannot (2/3 at 115–118 s vs
0/3 censored) and `easy-win` 2.2× faster (median 10.4 s vs 22.0 s);
`deep-win` stays censored with the passes on, 2/3 tree-only. The tree cut is
real but the auxiliary passes eat it: `core` needs 84.8 M child evals
tree-only (14 s), ~115 s with them. The aux-on gate rows (`gate.json`) measure
auxiliary throughput, not the tree: within a fixed wall a stack's expansions
are set by its sim/settle cost (~0.5 M child evals/s against ~6 M tree-only),
and the declared stack is trajectory-sensitive there (the R6 fix moved `core`
aux-on from 62.6 M to >126 M child evals) — an auxiliary artefact.

Tier 2 (10 s, `tier2.json`): `core` 0/3, `easy-win` 1/3 (7.58 s), `deep-win`
0/3; same-session provari hand medians 0.518 / 0.133 / 6.523 s. The only
shared cell is `easy-win` at **57.0×**; `core` is censored (≥ 19.3×) and
`deep-win` censored (≥ 1.53×). Coverage 1/3, geomean 57.0 — no rung change
(`T_2` still rests on the bare tree's `core` 3/3 at 18.4–23.3 s).

D14 (`matrix.json` `d14`): **16/16 accepted** — every proved matrix row
(product, adopted, adopted-aux-off) re-ran untimed with `--seconds 600
--proof-out`, matched its timed `child_evals`, and passed `atomi validate`
and `provari check`; the files were deleted.

## 6. Gates and soundness

`make test` (126 unit + 51 integration tests), `make lint` (fmt, clippy
`-D warnings`, doc, goal check) and `make test-full` (slow tier + `make
veto`: 8/8 rows accepted, all four controls `valid: false`) are green on the
shipped binary (`env.json` sha256 `fdd42b07…`; the per-phase binaries are
listed there). Soundness: the six mechanisms
only write fresh `Open` numbers (no new keyed position, no movegen, no board
build beyond pass 1's); `--guide none` and the plan-18 flags are
counter-identical (§1); the product hand is counter-identical before and
after R1–R6 existed. No GPL dependency or provari artifact was added; the
band factors are EF-PN's published weights re-expressed, not copied code.

## 7. Problems, open parts and next steps

1. **R4/R5 as implemented cost 1.9×.** Diagnosis (not measured): our count
   sees *any* attacker of a target and ignores that a capture beside our own
   commoner explodes it, while provari counts a quiet move's threats by the
   *moved piece* only. Next: count only the moved piece's targets on the
   quiet path, exclude our own commoner's blast zone, re-run the `core` tree
   row (one 40 s run).
2. **Aux cost is now the whole wall** (items 27–28): the adopted tree proves
   `core` in 84.8 M child evals tree-only (14 s), ~115 s with the passes on.
3. **The gate instrument**: `expansions` at a fixed child-evals cap measures
   the branching factor, not the tree; pin "child evals to proof at a
   generous cap" as the tree instrument.
4. **R2 needs its own spec** before any re-try: a band on our own `hand_eval`
   (-256…256) at a decisive edge of 128 puts most positions in the middle
   step; no probed edge pays.
5. Missing tests: no integration test pins a `--guide-on numbers` search's
   counters (only unit-level pairing and the fixture-playout differentials).
6. The default change is a **scope decision** against `plan19.md` §Phase D
   (which named the six-mechanism stack): the six miss `core` (1.78×), the
   D21 ablations reject R2 and R4/R5, and the adopted arm passes all three
   cells. Reverting to the literal six is one `--guide-on numbers` away.

SESSION COMPLETE
