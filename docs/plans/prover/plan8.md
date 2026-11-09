# Prover Plan 8: item 17 — comparable per-node re-diagnosis and the wall-goal decision

Initiative: [`prover`](initiative.md), backlog **item 17**, proposed decision
**D26**. Drafted 2026-11-09 after the plan7 kill review. This is a
**measurement and decision plan**: no engine change, no default change, no
proof-format work.

**Premise to test.** "Per-node cost is competitive" (M1: 53.4 ms per 1 M
`child_evals`) has never been measured in the unit that decides the goal:
wall per **expansion** against provari's wall per **node**. Plan7's fixed-10 M
runs give `easy-win` no-sim 132 k expansions / 2.105 s (≈ 15.9 µs per
expansion) against provari 72,523 nodes / 0.139 s (≈ 1.9 µs per node). If that
gap survives a same-session, same-unit comparison, the **core** is the
first-order gap, not the guide. If it does not, the gap is tree/aux, and the
wall goal is not near-term.

## Absolute distance (report7, 2026-11-09)

Goal: geomean wall `< 1.0`, coverage 3/3.

| Cell | atomi best product | provari hand | Factor |
| --- | --- | --- | ---: |
| `easy-win` | 5.83 s product; 3.89 s budgeted; 2.07 s no-sim | 0.139 s / 72,523 nodes | 27–42× |
| `core` | censored at 10 s (≈ 2.0 M exp) | 0.515 s / 342,289 nodes | ≥ 19× |
| `deep-win` | censored for search at 120 s | 6.468 s / 4,574,912 nodes | ≥ 1.55× |

Coverage is **1/3**. Item 3 and item 15 are closed; item 16's wall gate
failed. No further hand-guide or auxiliary-budget plan is in scope.

## Scope

**In:** one stdlib measurement driver; same-session provari controls; a
focused per-expansion decomposition on `easy-win` and `core`; a long-budget
coverage bracket on `core`/`deep-win`; `report8.md`; the item 17–19 backlog
and priority update.

**Out:** any search/guide/table change; restarts implementation (item 18);
the re-pin itself (owner, item 19); M4/`planX`; proofs beyond D14.

## Proposed D26 (owner to confirm)

**Per-node cost is judged per expansion and per node, never `child_evals`
against provari nodes.** The diagnostic is `ns/expansion` (main search plus
auxiliary) and `children/expansion`; `child_evals/expansion` is reported, not
compared across engines. The decision rule is pre-registered in §Gate below.
If it recommends a re-pin, that is item 19 and an owner decision; it is never
a silent change to the success condition.

## Design

### Phase A — comparable per-node (measurement only)

Cells `easy-win`, `core`; seeds 0–2; one process at a time. Re-run provari
`prove --guide hand --seconds 10 --memory 192` in the **same session** (three
repeats; record `nodes`, wall, version). Stacks:

| Stack | Flags |
| --- | --- |
| bare | `--guide none` |
| neither | `--guide hand --guide-off sim,closures` (0 aux; main-search isolation) |
| no-sim | `--guide hand --guide-off sim` (product calibration) |
| hand | `--guide hand --guide-scale 64 --guide-ef-weight 0` |

Budgets: fixed `--evals 10000000` runs (uncensored tree) and canonical
`--seconds 10 --memory 192`. Record wall, `child_evals`, `expansions`,
`sim_nodes`, `settle_nodes`, `aux_nodes`, length/exit. Derive:

- `ns/exp = wall·1e9 / expansions`, `exp/s`, `children/exp = child_evals/exp`;
- `R = (ns/exp atomi) / (wall_p / nodes_p)` per cell and stack;
- auxiliary share `(sim_nodes + settle_nodes) / (child_evals + aux)`.

**Decision A.** If `R > 2` on `easy-win` or `core` for the `neither` stack,
the core is a first-order gap: item 18 includes **core throughput**. If
`R ≤ 2`, item 18 is **tree/restarts** only. If the bare tree still proves
`easy-win` at ≥ 10×, record the zero-guide floor; no guide variant can go
below it.

### Phase B — long-budget coverage/factor bracket

Product configuration, `--seconds 60 --memory 192`, seeds 0–2: `core` and
`deep-win` (one additional 60 s `easy-win` reference). Record proved/censored,
wall, expansions, and the factor against the same-session provari hand. Also
run `--guide none` at 60 s on `core` (floor for a guide-free tree).

**Decision B.** If `core` **and** `deep-win` prove at 60 s with factors ≤ 3×,
the wall goal is near-term and item 18 proceeds. If either is censored, or a
factor is > 3×, the wall goal is not near-term: recommend **item 19 (re-pin)**
to the owner. Classification only; a censored cell is never converted.

### Phase C — decision, report, next plan

Write `report8.md` with the measured `R`, the coverage bracket, and exactly
one recommendation: **continue** (item 18: core throughput and/or restarts
under a wall gate) or **re-pin** (item 19, with two concrete options: a
coverage-first tier, or a node/`child_evals` score). Do not implement either
in this plan. If continue, draft `plan9.md` in the next plan session.

## Gates

- **G1 data.** Phase A and B matrices complete (cell × stack × seed ×
  budget; row counts recorded in the README), parsed JSON under
  `measurements/plan8/`, `env.json`, one process at a time.
- **G2 comparability.** provari and atomi measured in the same session and
  host; three repeats; `nodes`/`expansions`/`child_evals` defined in the
  README; no cross-row factor claimed as same-session.
- **G3 decision.** `R`, the 60 s coverage and the chosen recommendation are
  recorded with the pre-registered thresholds. G3 is a **decision gate**, not
  a wall gate; it cannot pass by dropping a cell or changing the metric.
- **G4 soundness.** No refutation or never-prove; any claimed D14 proof is
  accepted by `atomi validate` **and** `provari check`; `make test` green.
  No search/guide change, so `make test-full` is required only if code
  changes (it should not).
- **G5 docs.** `make lint`; measurement README; no GPL dependency, raw
  transcripts, provari artifacts or `__pycache__`.

## Tasks and stop points

- **A** driver + per-expansion matrix; stop if provari controls are missing
  (INCONCLUSIVE, never a frozen reference).
- **B** 60 s bracket; stop if a cell exceeds 3 × 60 s of wall in total.
- **C** decision and `report8.md`; stop before any item-18 implementation.

## Risks

| Risk | Mitigation |
| --- | --- |
| `nodes` and `expansions` are not the same unit | Report both `children/exp` and the bare-tree floor; conclusions are bracketed, not point claims |
| Frozen `guidance-v1` reference is stale | Re-run provari hand in-session; no factor from an old row |
| The 60 s bracket is too short for `deep-win` | Classify censor; the recommendation uses coverage, not a hidden budget increase |
| The decision becomes goal-shopping | Pre-registered rule (D26); re-pin is item 19 and owner-accepted only |
| Measurement expands into a benchmark suite | Two cells, two phases, one driver; no 24-cell sweep |

## Report and deliverables

`report8.md` (≤ 12 KB): headline, provenance, commands, the `R` table, the
coverage bracket, gates, and a single recommendation.

- **Measurements:** `measurements/plan8/{README.md,env.json,pernode.json,
  longbudget.json,run_plan8.py}`.
- **Docs:** `initiative.md` (item 17 status, item 18/19 rows, history),
  `report8.md`; `bench.md` receives no row (no goal claim).
- **No engine, test or default change is expected.**
