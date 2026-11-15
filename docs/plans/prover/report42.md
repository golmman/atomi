# Report 42 — kill review (plans 40–41) and item 55 SCALE-LENGTH

Plan: [`plan42.md`](plan42.md). Initiative: [`initiative.md`](initiative.md),
D33/D47/D49, [`gotchas`](gotchas.md). Data:
[`measurements/plan42/`](measurements/plan42/README.md). **No engine, CLI,
default, test or `bench.md` row changed.**

## 1. Kill review (fixed prompt; plans 40–41)

**Absolute distance to the goal.** `T_3` (Cap-120) is `core` 4.923 s,
`easy-win` 0.376 s, `deep-win` **53.42 s**, geomean **6.606**, 3/3, D14 9/9.
`T_4` is `deep-win` **5.34×** away; `T_5` is **6.61×** geomean away. w = 2 is
the measured `deep-win` lever (median 30,401 ms vs 50,327 ms = **0.604×**,
geomean 1.0249×, D14 9/9); the default stays 16 only because the held-out
guard was blind on `sample20`. Plan38's `L = 24` is the other tree lever
(1.42×), but w = 2 already gives length 25, so the two overlap on the length
axis. This session tests their composition (item 55).

**If we started today, would we build this?** Yes to the prover, hand-guide
stack, full-key proof table and two checkers (they produced `T_3`). No to the
20-plan per-node detour and standalone instruments. w = 2 is a one-parameter
guide change and the `L` pass is an in-base search-order change (D47); both
would be first-class from M1.

**Which initiative or plan would we cancel?** Item 48 (terms) closed
negative; item 52 Phase B (deepening) closed negative, keep `--max-length`;
items 44/50 as a goal route closed unless the owner wants a `T_4`-only
gamble; item 41 stays a `movegen` wishlist line; items 53/49 take the held-out
fix here; keep 51, fold item 47's scoping into it. No plan37 micro-candidates
(≈1.20×).

**Prior-art delta (analysis1 §3; pin unchanged ⇒ changed rows only).** R2
(banded EF-PN) is re-testable **after** a w = 2 default, at the same 20 M
gate; the band's step distribution is an eval property, not a weight. R25
(length-limited passes) is active: plan38/39 closed deepening, but `L = 24` is
a real 1.42× and its composition with w = 2 was untested until now. R17 is not
a wall row; R22 skip; R23/R24 keep; R9 closed; R11–R13, R18/R19 adopted; R26
skip.

**Smallest next experiment.** One `deep-win` seed-0 scan at w = 2 over
`L ∈ {∞, 24, 22, 20}` against the w = 2 control (30,401 ms / 3,165,142 /
length 25). Executed below; the answer is no.

## 2. What was run

Same binary (`fd2216fc…`, source `99f1628`), one process at a time, 120 s
wall cap, `--guide hand --memory 192 --seed N`, `--guide-ef-weight` after
`--guide hand` (D33).

- **Phase 0:** `deep-win`, seed 0, w = 2, `L ∈ {∞, 24, 22, 20}`, 120 s.
- **Held-out (G-42a):** five fixed `guidance-v1` cells, **0-based indices
  6–10** = the first five `wins.tsv` `dec*` cells (`dec01`, `dec03`, `dec04`,
  `dec05`, `dec07`; not `sample20`), seed 0, w = 2 vs 16, 120 s. The index
  base is fixed in `run.py::HELDOUT_INDICES`.
- **Gates:** `make test`, `make lint`, `make test-full` + veto.

## 3. Phase 0 — stop rule fired (G-42a Phase 0 reproduced)

The `L = ∞` row reproduces plan41's w = 2 control exactly (3,165,142
expansions / length 25; wall 30,469 vs 30,401 ms, noise). Full counters in
[`runs.json`](measurements/plan42/runs.json).

| `L` | ms | expansions | length | vs `L = ∞` |
| ---: | ---: | ---: | ---: | ---: |
| **∞** | **30,469** | **3,165,142** | **25** | **1.000×** |
| 24 | 66,564 | 7,327,572 | 24 | 2.184× |
| 22 | 34,598 | 3,578,565 | 22 | 1.135× |
| 20 | 37,538 | 3,871,051 | 20 | 1.232× |

No `L < ∞` beats `L = ∞`. **The pre-registered stop rule fires: the
composition is closed negative and no Phase A matrix is run.** The result
matches the overlap diagnosis: w = 2 already shortens the proof to 25, so a
bound at 24 or below buys one to five moves but pays a larger tree (1.14×–
2.18×). The bounded pass is not a composition lever on top of w = 2; it is a
strict regression.

## 4. Held-out direction — G-42a FAIL

All five cells prove at 120 s on both weights.

| Index | Cell | w = 16 ms | w = 2 ms | ratio | w = 2 faster |
| ---: | --- | ---: | ---: | ---: | --- |
| 6 | `dec01` | 171 | 480 | 2.807× | no |
| 7 | `dec03` | 13 | 12 | 0.923× | yes |
| 8 | `dec04` | 7 | 6 | 0.857× | yes |
| 9 | `dec05` | 1,197 | 1,588 | 1.327× | no |
| 10 | `dec07` | 3 | 3 | 1.000× | tie |

**2/5 faster, 1 tie, 2 slower** — the gate needs ≥ 3/5, so **G-42a FAIL**.
The two real regressions (`dec01` 2.8×, `dec05` 1.33×) repeat plan41's
`easy-win` regression (1.647×). The three near-ties are 3–13 ms, where wall is
noise; their expansions also move by only 1–3. w = 2 is a `deep-win`-specific
lever, not a general default. This is the held-out evidence plan41 lacked.

## 5. Gates

| Gate | Result |
| --- | --- |
| G-42a Phase 0 reproduced + held-out ≥ 3/5 | **FAIL** — Phase 0 exact, held-out 2/5 |
| G-42b composition ≤ 0.50× | **FAIL** — `L = ∞` is best, so the gate is w = 2 alone: **0.604×** (plan41) > 0.50×; composition recorded as a no-op |
| G-42c D14 | **not applied** — G-42b fail; plan41 `d14.json` already covers w = 2, `L = ∞` 9/9 both checkers |
| G-42d default | **not applied** — `GuideParams::default().ef_weight` stays **16**, `max_length` stays off |
| G-42e `make test`, `make lint`, `make test-full` + veto | **PASS** — all tests, `VETO: PASS` |
| G-42f boundary | **PASS** — no new eval term, no model, no provari source, no `T_4`/`T_5` claim, no item 48 re-open |

## 6. Problems, scope and missing tests

- **The held-out set is partly noise.** The first five `dec*` cells are easy;
  three run in 3–13 ms. The plan's fixed 0-based selection was pre-registered
  in the driver before the runs, so it was not changed after seeing the
  result. A harder five-cell set (largest `ref_nodes` in `guidance-v1`) would
  be more informative but would be a new, post-hoc selection.
- **The composition question is closed, not the w = 2 default question.**
  Phase 0 is decisive that `L < ∞` does not compose with w = 2. It says
  nothing new about w = 2 alone, which plan41 measured at 0.604× and this
  held-out check now shows is easy-cell-negative.
- **No D14 run.** No new proof was claimed; the selected stack (w = 2,
  `L = ∞`) is the plan41 stack, already D14-accepted 9/9. A default change
  would still need a fresh D14 under G-42d, but G-42b failed.
- **No unit test.** No source or default changed, so there is nothing to pin.
  If the owner ever adopts w = 2, pin `ef_weight` and the `--guide-ef-weight`
  round trip then.
- **The 5 M probe was not used** (D33): it is blind to this lever.

## 7. Next steps

1. **Record the composition negative.** Do not fund a w = 2 + bounded-length
   stack; `L < ∞` is a strict regression at w = 2. `--max-length` stays the
   opt-in measurement flag from plan38.
2. **Owner ruling on w = 2 remains open**, now with a fixed held-out set: the
   lever is 0.604× on `deep-win` and D14 9/9, but it regresses the non-trivial
   easy cells (2.8×, 1.33×) and does not clear the 0.50× gate. Under D49 the
   default stays 16 unless the owner rules otherwise.
3. **Do not re-open item 48** (terms) or the R2 band on this evidence. If the
   owner wants a scale-free EF-PN, scope ideation B as a new item with the
   search-frontier eval distribution as its Phase 0.

## 8. Follow-up options

1. **Kickoff (recommended):** “Read `docs/plans/prover/report42.md`. Record the
   composition negative in `initiative.md` item 55 and the w = 2 held-out
   evidence (2/5 on `guidance-v1` dec cells, `dec01` 2.8×, `dec05` 1.33×).
   Rule on the w = 2 default: keep 16, or adopt w = 2 (then pin the default,
   the `--guide-ef-weight` round trip, a Cap-120 `bench.md` row, fresh D14 and
   `make test-full`). Write no engine change without the ruling.”
2. If the owner rules for a general guide lever, scope ideation B (scale-free
   EF-PN) as item 56 with the search-frontier distribution as its Phase 0.
3. If the owner wants a `T_4`-only gamble, scope items 44/50 together; do not
   re-run this composition.

SESSION COMPLETE — kill review recorded; Phase 0 stop fired (`L = ∞` best at
w = 2), composition closed negative; held-out direction 2/5 so G-42a fails;
G-42b fails at 0.604× vs the 0.50× bar; no default change, w = 16 and
`L = ∞` stay; gates green. Data in `measurements/plan42/`, report in
`report42.md`.
