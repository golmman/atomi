# Report 40 — kill review (plans 38–39) and item 48 GUIDE-EVAL

Plan: [`plan40.md`](plan40.md). Initiative: [`initiative.md`](initiative.md),
D8/D14/D21/D33/D47–D49, [`gotchas`](gotchas.md). Data:
[`measurements/plan40/`](measurements/plan40/README.md). **No engine, CLI,
default, test or `bench.md` row changed.**

## 1. Kill review (fixed prompt; plans 38–39)

### Absolute distance to the goal

`T_3` (Cap-120) is `core` 4.923 s, `easy-win` 0.376 s, `deep-win` **53.42 s**,
geomean **6.606**, 3/3, D14 9/9. `T_4` is `deep-win` **5.34×** away; `T_5` is
**6.61×** geomean away.

Plans 38–39 changed the question, not the rung. Length is a real tree lever
but not a product route: `L = 24` is length 24 at 37.5 s against 36 at 53.3 s
(1.42×). At 9,246 ns/exp, 10 s allows **1.082 M** expansions; the `L = 24`
tree is **4.109 M**, still **3.80×** too large before the per-node 4.38×.
Deepening is closed: every `L < 20` prefix costs more than the whole `L = 24`
pass (best `L = 16` 45.500 s / 132.5 M against 37.190 s / 109.7 M); the
cumulative `L < 20` cost is 650.95 M / 239.98 s against a 54.86 M / 18.60 s
bar. No measured single axis reaches `T_4`; the only untested in-tree lever
was the guide's evaluation/ordering signal, item 48 (now tested: §2–§3).

### If we started today, would we build this?

**Yes** to the prover, hand-guide stack, full-key proof table and two
checkers: that asset produced `T_3`. **No** to the 20-plan per-node detour and
standalone instruments. Proof length and a fixed-context tree metric would be
first-class from M1.

### Which initiative or plan would we cancel?

- **Item 52 Phase B** (deepening): closed by report39; keep `--max-length`,
  no driver or new table key.
- **Items 44/50 as a goal route**: close unless the owner wants a `T_4`-only
  gamble; `M` is INCONCLUSIVE-BY-UNITS at 1.18–1.46×, not 4.38×.
- **Item 41** stays a `movegen` wishlist line, not a `prover` dependency.
- **Items 53/49**: absorb the 5 M/fixed-cap instrument here; widen the sample
  only after a tree lever clears a gate.
- **Keep 51 and 48**; fold item 47's scoping into them.
- **No plan37 micro-candidates** (`populate_state`, `Mover::effect`): ≈1.20×.

### Prior-art delta (analysis1 §3; pin unchanged ⇒ changed rows only)

**R25** moved *skip → active* in plan38 and is closed for deepening by
report39's prefix numbers; only the opt-in flag remains. **R2 (banded
EF-PN)** is re-argued with a number: plan19 rejected it at **1.2–1.35×** on
every edge, but with the *current* eval. The band's step distribution is an
eval property, so R2 is re-testable only **after** item48, with the same 20 M
gate. **R4/R5** were rejected at **1.9× on `core`**: their counting cost more
tree than the numbers bought. R17 is not a wall row; R22 is still 45 of 153 k
expansions; R23/R24 keep. No full re-read.

### Smallest next experiment

**One scratch TERMS-V1 eval at the 5 M `deep-win` probe**, after a read-only
cost pass, against the control (5,000,034 child evals / 168,910 expansions).
Executed below; the answer is no.

## 2. Item 48 — Phase 0 (G-40a PASS)

Read-only `/tmp` harness over `corpus-v1` plus the `testutil` seeded legal
playouts: **8,293** positions, median of 3 timed passes, 20 passes per sample.
Own code only (D8); no model or provari source. The formula and weights were
frozen before Phase A; no term was tuned on a bench cell.

Frozen **TERMS-V1**: `material/32 + pressure*12 + (sig(us)-sig(them))*20 +
(exposure(us)-exposure(them))*8 + fuse*16 - checkers*16 + mobility/4`,
clamped to `[-256, 256]`. Exposure counts non-pawn pieces (commoner included)
in the opposing commoner's blast zone, capped at 3 per side; fuse is attackers
on the enemy commoner's square minus attackers on ours, each capped at 3;
checkers is the side to move's checker count as a penalty.

| Eval | ns/call |
| --- | ---: |
| current `hand_eval` | **149.11** |
| TERMS-V1 with mobility | **175.24** |
| TERMS-V1 trimmed (no mobility) | **139.20** |

G-40a bar **199 ns/call**: **PASS** with mobility retained, so the trim was
not needed. No sample position saturates the overall clamp (TERMS-V1 range
`[-226, 191]`); the new terms fire on 5.8–16.5 % of positions. TERMS-V1
differs from the current eval on 2,169 of 8,293 positions.

## 3. Item 48 — Phase A (G-40b FAIL, STOP)

Scratch clone of `HEAD` (`e39beae`) in `/tmp/atomi-plan40`, candidate behind a
temporary `ATOMI_GUIDE_EVAL=rich` switch (`phaseA/candidate.patch`; not a repo
change). `deep-win`, `--guide hand --memory 192 --seed 0 --evals 5000000`,
same session, one process at a time.

| Run | `child_evals` | expansions | vs control |
| --- | ---: | ---: | ---: |
| control-1 | 5,000,034 | 168,910 | 1.00000 |
| TERMS-V1 | 5,000,005 | **168,879** | **0.99982** |
| control-2 | 5,000,034 | 168,910 | 1.00000 |

G-40b asks for **≤ 152,019** expansions (0.90×). TERMS-V1 moves the tree by
**31** expansions, not 16,891. The control is counter-identical twice. No
proof or refutation exists at the cap, so the proof/refutation clause is
vacuous; the tree clause is decisive. **STOP**: Phase B (`--guide-eval rich`,
identity test, three-cell 20 M/120 s matrix, D14) was not built. No repo source
changed, no default changed, no `bench.md` row.

## 4. Mechanism reading

The richer terms barely change the ordering. The new signal is present in only
10–16 % of positions, and the two full `signature_for` scans plus exposure and
fuse buy a 31-expansion displacement. The existing eval already carries the
dominant material and commoner-pressure terms; the added targets/exposure/fuse
mostly reinforce directions the old eval already has, or disagree on positions
that do not sit on the 5 M frontier. A 0.90× gate needed the eval to change
which children are expanded early; it does not.

Cost was not the binding constraint: even the slower earlier sample
(186 ns/call) was under G-40a's 199. The failure is signal, not price.

## 5. Gates and boundary

| Gate | Result |
| --- | --- |
| G-40a price ≤ 199 ns/call | **PASS** — 175.24 with mobility |
| G-40b 5 M expansions ≤ 0.90× | **FAIL** — 0.99982 |
| G-40e `make test`, `make lint`, `make test-full` + veto | **PASS** — unchanged tree, `VETO: PASS` |
| G-40f no default / no `bench.md` row / no `T_4` or `T_5` claim / no provari source | **PASS** |

`make test` (149 unit + integration), `make lint` (goal check, fmt, clippy
`-D warnings`, doc) and `make test-full` (ignored tier + `make veto`) are
green on the unchanged tree.

## 6. Problems, scope and missing tests

- **The gate compares at 5,000,034 `child_evals`, but the candidate stops at
  5,000,005** (cap granularity). The ratio on expansions is used and the miss
  is 10× larger than any plausible correction; the verdict is not sensitive.
- **The plan's exposure wording is literal “non-pawn”.** The frozen candidate
  therefore includes the commoner. On the sample it changes no term value
  (own commoner is never in the enemy blast zone in these positions), so the
  choice is recorded but not load-bearing.
- **Phase A is one seed, one cell, one cap.** It cannot see a late tree cut
  after 5 M (D33); it can see that the early ordering is nearly unchanged.
- No unit test was added because no formula entered the repo. If item 48 is
  ever re-opened, the algebra/bounds and `--guide hand` identity tests from
  plan40 Phase B are the first tests.
- Ideation B (eval as ordering only, `--guide-ef-weight 0` plus tie-break) was
  not run: the measured full-eval signal is already ≈0, so an ordering-only
  variant has no evidence behind it. It remains the cheapest re-open if a new
  ordering hypothesis appears.

## 7. Next steps

1. **Close item 48 negative** ([`initiative.md`](initiative.md) row 48). Do
   not build `--guide-eval rich`, do not append a `bench.md` row.
2. **Do not re-test R2 on this eval**: the eval did not move the tree, so its
   band distribution is not a new mechanism.
3. The tree-axis evidence is now reports 38–40: length works but not enough,
   deepening is closed, and the hand eval's added terms do not change the
   early tree. The next move is the owner's `T_4`/`T_5` ruling (items 44/50 are
   `T_4`-only gambles; items 49/51/53 are instruments, not levers).
4. If the owner wants one more tree probe, the surviving question is **why**
   the eval signal is ignored (read points, EF-PN weight sensitivity, or the
   band), not which terms to add.

## 8. Follow-up options

1. **Kickoff (recommended):** “Read `docs/plans/prover/report40.md`. Close item
   48 negative in `initiative.md` with the 0.99982 5 M ratio and the 175 ns
   price; keep `--max-length` and the hand eval unchanged. Record the owner's
   `T_4`/`T_5` ruling: items 44/50 are `T_4`-only gambles, items 49/51/53 are
   instruments. Write no engine change.”
2. If the owner wants one more eval probe: make the eval an ordering-only
   tie-break (`--guide-ef-weight 0`) at the same 5 M gate; do not add terms.
3. If the owner rules for a `T_4`-only gamble: scope items 44/50 together as
   one funding test, not as a goal route.

SESSION COMPLETE — kill review recorded, G-40a PASS, G-40b FAIL/STOP, no
engine or default change, measurements in `measurements/plan40/`, report in
`report40.md`.
