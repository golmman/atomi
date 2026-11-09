# Prover Plan 7: item 16 — wall attribution and a deterministic auxiliary budget

Initiative: [`prover`](initiative.md), backlog **item 16**, decision **D24**.
Drafted 2026-11-09 in plan session 7, after the kill review and the item-15
closure. The M4 plan is parked as [`planX`](planX.md). No impl in this
document.

**Premise to test.** Per-node cost is competitive (M1: 53.4 ms per 1 M child
evals); the goal gap is wall time spent in guide auxiliary work, not movegen.
If that premise is false, stop at phase A and re-diagnose instead of building
a budget.

## Absolute distance (same-session, 2026-11-09)

Goal: geomean wall `< 1.0`, coverage 3/3.

| Cell | atomi (best product config) | provari hand | Factor |
| --- | --- | --- | ---: |
| `easy-win` | 5.8 s, 110–122 k expansions, 27 M child evals | 0.137 s / 72,523 nodes | ≈ 42× |
| `core` | censored at 10 s; aux 120 s = 278.5 M evals, length 20 | 0.51 s / 342,289 nodes | ≥ 20× |
| `deep-win` | censored at 120 s (562 M evals, 20.2 M expansions) | 6.57 s / 4,574,912 nodes | ≥ 18× |

Coverage at the product budget is **1/3**. `easy-win` spends 5.8 s on
27 M child evals = ≈ 215 ns/eval, versus M1's 53 ns/eval and provari's
≈ 1.5 µs/node. The gap is the per-expansion auxiliary work (`sim_nodes`,
`settle_nodes`, guide eval/classification), whose default caps are **0 =
unlimited**: a fresh child runs a 6-ply greedy playout, and a settle hit runs
a one-ply legal movegen. `max_sim=1`/`max_settle=1` still allows one playout
and one settle per frame.

## Scope

**In:** an attribution driver over the frozen `guidance-v1` cells; a
deterministic **global auxiliary-work budget** behind opt-in flags; cheaper
auxiliary paths justified by the attribution; unit/integration tests; a
measurement README and `report7.md`.

**Out:** a wall-clock budget inside the search (forbidden by D22); proof
output / `planX`; restarts (D5) and the portfolio (horizon 8); hand-guide
semantics; default-flag changes beyond the measured opt-in; any provari code
or binary in the repo.

## Proposed D25 (owner to confirm)

**Auxiliary work is bounded by a deterministic global work counter, not by
wall time.** The budget is part of the deterministic flag set (`same binary +
FEN + flags + seed = identical counters`, D22). A wall-clock check inside the
search would make trajectories machine-dependent and is rejected. The
counter is in auxiliary work units (made playout moves + settle movegens),
`0` = unlimited, and the plan is judged on wall, not on the budget value.

## Design

### Phase A — attribution (measurement only, no engine change)

One driver, seeds 0–2, on `easy-win`, `core` and (one attempt) `deep-win`.
Two budgets per stack: a fixed `--evals` run (so wall censoring cannot hide
the tree) and the canonical `--seconds 10 --memory 192` run. Stacks:

| Stack | Flags |
| --- | --- |
| bare | `--guide none` |
| hand | `--guide hand` (default caps 0/0) |
| no closures | `--guide hand --guide-off closures` |
| no sim | `--guide hand --guide-off sim` |
| neither | `--guide hand --guide-off sim,closures` |
| aux 1/1 | `--guide hand --guide-max-sim 1 --guide-max-settle 1` |
| sim only 1 | `--guide-max-sim 1 --guide-max-settle 0` |
| settle only 1 | `--guide-max-sim 0 --guide-max-settle 1` |

Record per row: wall, `child_evals`, `expansions`, `sim_nodes`,
`settle_nodes`, evals/s, expansions/s. **Decision rule:** if
`(wall(hand) - wall(no sim,closures)) / wall(hand) < 0.3` on `easy-win`, the
auxiliary premise is false: stop, re-diagnose, do not implement the budget.
Otherwise phase B.

### Phase B — deterministic budget and cheaper aux

1. `GuideParams.aux_budget: u64` (CLI `--guide-aux-budget N`, `0` =
   unlimited). A search-global counter increments by `sim_nodes` and
   `settle_nodes`; once exhausted, optional sim/settle is skipped. Record
   `aux_nodes` and `aux_skipped`. Determinism and exactness tests.
2. Cheaper paths, only if phase A shows them: reduce `SIM_PLIES` (6 → 2/3),
   skip playout for children already statically settled, and avoid a second
   legal-movegen when the settle screen already has the list. Each is a
   leave-one-out measurement, not a bundle.
3. Calibrate a small budget grid (e.g. 10 k / 100 k / 1 M / unlimited) on the
   two cells; pick the point that meets the wall gate without refutations.

### Phase C — wall gate and classification

Run the chosen configuration on seeds 0–2 at `--seconds 10 --memory 192`.
`deep-win` gets one 10 s attempt and is classified (proved / censored for
search / censored for output). No proof run unless a cell is claimed; D14
applies if it is.

## Gates

- **G1 data.** Phase A matrix complete, parsed JSON under
  `measurements/plan7/`, one process at a time, same-session controls.
- **G2 determinism.** Same flags + seed = identical counters under a work
  cap; `--guide-aux-budget` changes the tree only, never the meaning; no wall
  clock in the search. A repeated fixed-eval run is byte-identical.
- **G3 wall.** `easy-win` wall ≤ **0.5 s** (factor ≤ 3.6) and `core` proved
  at 10 s / 192 MB, seeds 0–2; no guidance-set refutations. Counters are
  recorded as diagnostics, never substituted for wall.
- **G4 soundness.** No refutation or never-prove; any claimed proof accepted
  by `atomi validate` and `provari check`; `make test` green, `make
  test-full` green for search/guide changes.
- **G5 build/docs.** `make lint`; measurement README + `env.json`; no GPL
  dependency; no raw transcripts, provari artifacts or `__pycache__`.

If G3 fails after phase B, **do not** start a third hand-guide plan: record
the result and take the continue / re-pin / close decision at the kill
review (D24). The likely re-pin options are a slower-provari corpus or a
node-based goal; both are owner decisions, not silent changes.

## Tasks and stop points

- **A** attribution driver + matrix; stop if the 0.3 rule fires.
- **B1** `--guide-aux-budget` + tests; stop if a work cap cannot reproduce
  the uncapped trajectory's counters at the same cap.
- **B2** cheaper-aux leave-one-out; keep only measured gains.
- **C** wall gate, `deep-win` classification, `report7.md`.

## Risks

| Risk | Mitigation |
| --- | --- |
| Auxiliary work is not the wall cost | Phase A decision rule; stop before coding |
| A global budget starves the tree and loses coverage | Budget grid; wall gate includes coverage and refutations |
| Budget breaks determinism | Counter-based only; D22 exactness test; no `Instant` in the search |
| `easy-win` is provari's unusually fast cell and the gate is harsh | Gate at ≤ 3.6× this plan; the geomean is decided at M5, not here |
| `deep-win` is out of reach at 10 s | Classify the censor; do not hide it, do not raise the budget silently |
| Cheaper aux is not semantics-preserving | Auxiliary work is heuristic only; proofs still come from the main search, checked by both validators |

## Report and deliverables

`report7.md`: headline, provenance, commands, the phase A matrix, the budget
grid, wall factors, gates, problems, and a closure block with the kill-review
options. State explicitly whether the wall premise survived.

- **Engine:** `src/guide/mod.rs` (`GuideParams.aux_budget`),
  `src/search/{frame.rs,mod.rs}` (global counter, skip), `src/cli.rs`,
  `src/main.rs` (counters), tests.
- **Measurements:** `measurements/plan7/{README.md,env.json,attribution.json,
  budget.json,wall.json}`.
- **Docs:** `initiative.md` (item 16 status, history), `bench.md` (no goal
  claim), `report7.md`; `planX.md` stays parked.
