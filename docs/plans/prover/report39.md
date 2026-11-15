# Report 39 — item 52 Phase B (SHORT-PASS): Phase 0 stops the deepening schedule before any driver code

Plan: [`plan39.md`](plan39.md) (item 52 Phase B). Initiative:
[`initiative.md`](initiative.md), D47/D49, [`gotchas`](gotchas.md) §Search.
Data: [`measurements/plan39/`](measurements/plan39/README.md). **No engine,
CLI, default, test or `bench.md` row changed**; Phase 1 was not implemented
because the Phase 0 stop rule fired.

## 1. Verdict

**STOP — Phase B is closed negative at Phase 0.** Every `L < 20` pass costs
more than the single `L = 24` control in both `child_evals` and wall. The
cheapest small pass (`L = 16`, 45.500 s) is already **1.22×** the control's
wall (37.190 s) and **1.21×** its `child_evals`; `L = 12` is 2.24× and `L = 18`
is 1.77×. A deepening schedule must pay at least one such prefix before it
reaches `L = 24`, so it cannot beat 37.5 s. The driver, `clear_open`, the
per-iteration cap and the S1–S4 schedules were therefore not built or run.

| Gate | Bar | Result |
| --- | --- | --- |
| G-39a Phase 0 | `L = 12/14/16/18` at 5 M and 120 s, parsed, `env.json`, README | **PASS** — `runs.json`, `env.json`, `README.md`, `run.sh` |
| G-39b control | tree-identical to G-38a | **PASS (control only)** — 5,803,257 / 155,236,740 / length 36 at 120 s; 5 M probe 5,000,034 / 168,910 |
| G-39c single iteration | `--deepen 24` reproduces Phase A | **NOT RUN** — no `--deepen` driver; a direct `--max-length 24` run reproduces 109,715,277 / length 24 / 37.190 s |
| G-39d table rule | `clear_open` keeps `Refuted`/proofs, drops `Open` | **NOT RUN** — no `clear_open` |
| G-39e D14 | every proof accepted by both checkers | **N/A** — no new proof claimed |
| G-39f measurement | a schedule beats 0.85× the control with length ≤ 24 | **FAIL** — no `L < 20` pass is even below 1.0×; best is 1.22× |
| G-39g gates | `make test` + `make test-full` + veto | **PASS** — unchanged tree; `VETO: PASS` |
| G-39h boundary | no default, no `bench.md` row, no `T_4` claim, no provari source | **PASS** |

## 2. Phase 0 results (one process at a time)

Cell `deep-win`, `--guide hand --memory 192 --seed 0`, same binary
(`f14ec300…`), same session. Full counters in
[`runs.json`](measurements/plan39/runs.json).

| Run | exit | length | `child_evals` | expansions | ms | proof entries |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| control 120 s | proved | 36 | 155,236,740 | 5,803,257 | 52,184 | 5,716,874 |
| `L = 24` 120 s | proved | 24 | 109,715,277 | 4,109,113 | **37,190** | 4,481,393 |
| `L = 12` 120 s | proved | 12 | 203,878,285 | 7,116,345 | 83,136 | 8,493,465 |
| `L = 14` 120 s | proved | 14 | 133,735,711 | 4,812,636 | 45,526 | 6,132,143 |
| `L = 16` 120 s | proved | 16 | 132,487,780 | 4,821,074 | 45,500 | 5,783,348 |
| `L = 18` 120 s | proved | 18 | 180,850,219 | 6,776,360 | 65,820 | 7,816,685 |

At the 5 M probe: control and `L = 18` are 5,000,034 / 168,910; `L = 12/14/16`
are 5,000,003 / 169,652, 5,000,021 / 168,983 and 5,000,037 / 169,031. A bound
below 18 already changes the early tree even when no proof exists yet.

**Stop arithmetic.** Cumulative `L < 20`: **650,951,995** `child_evals` and
**239,982 ms**, against the plan's bars **54,857,638** and **18,595 ms** (0.5×
the `L = 24` pass). Both bars fail by >11×. The absolute clause — any single
`L < 20` pass exceeding `L = 24` ⇒ stop — also fires for all four passes.

## 3. Mechanism reading

The `L`-curve is strongly non-monotone in both directions: `L = 14/16` are the
cheapest small passes at ≈45.5 s, `L = 12` is worse than the unbounded control
per eval count (203.9 M), and `L = 18` (180.9 M) is worse than `L = 14/16`
despite a looser bound. The bounded search pays for re-searching the same
nodes under a different allowance; that cost is already above the `L = 24`
pass at every prefix. The plan's target was 37.5 s; no prefix is close.

This is the same phenomenon Phase A recorded (`L = 20/22` slower than
`L = 24`), now with the small-`L` arm filled in. It also prices ideation B
(bound-keyed refutation table): the cold-pass overlap would have to remove
most of the 650 M prefix work to reach 37.5 s, which the measured curve gives
no reason to expect. Option C (refine after `L = 24`) is likewise cold-slower
at `L = 20/22`; only a warm refinement experiment could reopen it.

## 4. Problems and scope changes

- **Plan39 §2 contains a contradictory stop rule.** It says “proceed if the
  cumulative bars hold **or one proves**”, then says “if any single `L < 20`
  pass exceeds the `L = 24` pass, stop”. All four passes prove, so the first
  clause literally fires; all four exceed, so the absolute clause fires too.
  This session followed the stronger, last-stated clause, because a schedule
  cannot reach `L = 24` without first paying at least one prefix that is
  already slower than the whole control. Recorded, not silently resolved.
- **No Phase 1 code was written.** The plan makes the driver conditional on
  Phase 0 passing; taking the “or one proves” reading would have forced a
  driver whose own Phase 0 pricing says it cannot win. That is not a scope
  change, but it is the point an owner override would have to address.
- **No proof was claimed, so no D14 run.** The three small-`L` proofs are not
  needed: none is a product route and the wall result is negative.
- **No `test-full`-requiring source change.** The gates still ran clean on the
  unchanged tree (`make test-full` includes `make veto`, `VETO: PASS`).

## 5. Missing tests / open parts

- No test exercises an iterative-deepening driver or `clear_open`; neither
  exists. If Phase B is ever reopened, G-39d (keep `Refuted` and proofs, drop
  bound-dependent `Open`) is the first unit test.
- No warm-pass experiment exists. The cold-pass curve cannot distinguish
  “re-search is expensive” from “state retention would make a later `L`
  cheap”; only a driver can. The measured gap (every prefix alone ≥1.22× the
  control's wall) makes that experiment low-value.
- No `--deepen` CLI contract, no per-iteration stats, no `bench.md` row.

## 6. Next steps

1. **Close item 52 Phase B negative**; do not fund the driver or the
   bound-keyed refutation table on this curve. `--max-length` stays the opt-in
   measurement flag from Phase A.
2. **Owner decision on the axis:** item 48 (guide quality) is the remaining
   tree lever not priced by this plan; item 47/51 need the owner's gate
   ruling. Do not reopen per-node micro-plans (plans 34–37 closed them).
3. If the owner overrides the stop rule, the cheapest decisive experiment is
   **not** S1–S4. It is one warm `L = 20` pass after a completed `L = 24`
   (ideation C), because the cold `L = 20` is 58.4 s and only a warm pass can
   test state retention. Gate: warm ≤ 0.5× cold, else close.

## 7. Follow-up options

1. **Kickoff (recommended):** “Read `docs/plans/prover/report39.md`. Close item
   52 Phase B negative: the Phase 0 small-`L` passes all exceed the `L = 24`
   control. Update `initiative.md` item 52 and write no driver. Then run a
   plan session for item 48 (guide quality) under the D47/D49 rules, ≤ 8 KB,
   read-only probes first.”
2. If the owner wants the other reading of §2: implement only the driver and
   measure S3 `12,24` at 120 s; do not build `clear_open` until the driver’s
   own Phase 0 rows show a schedule can beat 37.5 s.
3. If the goal question returns to the owner: use report39's negative Phase B
   as the tree-axis evidence; the next scoped lever is item 48, not another
   per-node micro-plan.

SESSION COMPLETE — Phase 0 executed, stop rule applied, Phase 1 not built;
measurements in `measurements/plan39/`, report in `report39.md`, G-39a and
G-39g passed, no default and no proof claim.
