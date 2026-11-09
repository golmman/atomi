# Prover Plan 4: item 3 re-scope — CORE-RESET (D21 kill review and last attempt)

Initiative: [`prover`](initiative.md), backlog **item 3** (M2+M3), re-scoped
by the D21 stop rule. Drafted 2026-11-10 in plan session 4, after
[`report3.md`](report3.md), [`bench.md`](bench.md) and
[`retro.md`](../../notes/retro.md). Baseline: the plan3 tree. This document
does not implement. Item 3 stays open only for this plan: if G1 fails it
closes and the report records the pivot options; there is no plan5 split into
mechanism plans.

## Absolute distance (read-only probes, 2026-11-10, plan3 binary)

Probes, not committed measurements; one process at a time. Goal: geomean wall
`< 1.0` vs provari hand, coverage 3/3.

| Cell | provari hand | atomi bare | atomi hand |
| --- | --- | --- | --- |
| `core` | 0.52 s · 342,289 nodes | **proves**: 16.81 s · 268.2 M evals · 9.27 M expansions · length 14 | **180 s timeout** at 86.3 M evals · 2.92 M expansions; `proof_full` at 2,831,155; 441.9 M sim + 154.3 M settle |
| `easy-win` | 0.14 s · 72,523 nodes | m1 1.482 s · 30.3 M · 1.05 M expansions | 211,254–631,763 expansions; `--guide-off efpn` 110,293 |
| `deep-win` | 6.47 s · 4,574,912 nodes | censored at 10 s | censored at 20 M |

1. **atomi's bare tree ≈ provari's bare tree.** The retro's all-guide-removed
   `core` is 8.65 M nodes; atomi bare needs 9.27 M. The 27× `core` gap is
   **guide effectiveness**, not a broken DF-PN core (per node M1 is
   competitive).
2. **The hand stack does not reproduce provari's 25× `core` reduction.** It
   reaches 2.9 M expansions in 180 s, fills the proof table, and its
   auxiliary work (596 M sim/settle) dominates the wall.
3. **The starting-number magnitude is the untested lever.** The retro's
   warning ("the real lever was the size of the starting proof numbers") was
   never tested; plan3 ablated where `core` never reached that tree.

## D21 kill review (fixed prompt)

- **Absolute distance?** `core` 27× tree / ~32× wall; `easy-win` 14.4× tree
  bare and 1.5–4.4× the target; `deep-win` unproved; coverage 1/3.
- **Build it again?** Not always-on: build the bare DF-PN, calibrate numbers
  on `core`, then add sim/settle (they cost ~7× per node and memory).
- **Cancel?** Always-on sim/settle and "every settle stores a proof". Pause
  M4/M5 until `core` proves; keep the hand idea.
- **Prior art.** Re-run the 2026-11-09 check in Phase 0; the retro appendix is
  the current source.
- **Smallest next experiment.** One class multiplier × EF-PN weight at
  `--evals 50000000` on `core`, settle proofs not inserted (Phase A/B).

**Proposed D23 (owner to confirm):** re-scope item 3 to **CORE-RESET** as the
last attempt. Keep every mechanism; make the stack cheap, calibrated and
memory-safe. No adoption/removal until G1; both-direction ablations decide.

## Scope

**In scope:** calibration (`GuideParams.global_scale`, `ef_weight`),
proof-table pressure, a proof-share memory split, a deterministic auxiliary
budget, the D21 re-run and the item-3 decision. Calibration halves are parsed
like the existing table and applied only to fresh start numbers; the sweep is
`scale ∈ {1,4,16,64,256}` × `ef_weight ∈ {0,8,16,32,64}`.

**Out of scope:** new guide mechanisms, a new search algorithm, portfolio,
restarts, tablebases, `.pvr`, model guide, default flags, and any change to
the value algebra, key, repetition/50-move rules, validator or `OPEN_MAX`.
M4/M5 wait for the item-3 decision.

## Design and measurement

1. **Calibration.** Multiply only fresh start numbers, never decided values.
   Sweep score = `expansions` on `core` at 50 M evals, guard `easy-win` at
   10 M; pick the smallest `core` count with `easy-win` no worse.
2. **Proof-table pressure.** Insert a settled `Proved` child only when the
   parent's OR value selects it (or recompute at write time); a settled
   `Refuted` stays a TT refutation. Test selected and unselected children.
3. **Memory split.** `table::split` takes a proof-share parameter (default
   1/4); byte invariant over every share/MiB; 90% refusal stays.
4. **Auxiliary budget.** `GuideParams.max_sim`/`max_settle` per frame (0 =
   unlimited); after the cap, children keep class/EF-PN starts but get no
   sim/settle. Deterministic; counters unchanged.

- **A (no wall claim).** `core` at `--evals 50000000`, `easy-win` at
  `--evals 10000000`, seed 0, 192 MB: record expansions, child evals, proved,
  length, `proof_full`, aux nodes. Confirm the point on frozen `guidance-v1`
  (24 cells) × seeds 0–2; set and sha256 (`2da2a65b…8c7d`) unchanged.
- **B.** `core` hand at `--seconds 120 --evals 300000000`: require
  `proof_full = false`; repeat with the auxiliary budget.
- **C (D21 re-run).** `guidance-v1` × `bare`/`hand` × seeds 0–2,
  `--seconds 600 --evals 20000000 --memory 192`; canonical `core`/`easy-win`
  at 10 s, 3 repeats, D14 proof check.
- **D.** G1 pass: both-direction ablation (same set, seeds 0–2, 10 M cap) and
  a ledger row after G1–G5. G1 fail: close item 3, no ledger row.

Fast tests: scale/weight round-trip; scale 1/weight 16 counter-identical to
plan3; decided values never scaled; settle proof round trip (selected and
unselected); memory-split invariant; aux budget deterministic; no-guide
identity. Slow: fixture oracle, `make veto`, settled-child proof round trip.

## Gates

- **G1 (D21 re-run).** `hand` proves `core` at `--seconds 10 --memory 192` on
  seed 0 and `easy-win` `expansions ≤ 145,046`; ≥20 cells × seeds 0–2; no
  guidance-set refutation. Wall/`child_evals` recorded, not substituted. A
  failing non-zero seed fails G1 (report the worst seed).
- **G2 soundness.** No refutation/never-prove; every claimed proof accepted
  by `atomi validate` and `provari check`; `--proof-out` neutral.
- **G3 fixtures.** `make test-full` green; no outcome regression.
- **G4 determinism.** No-guide counters identical to m1; same seed+flags
  identical; aux counters separate; no `Open` at `INFINITY`.
- **G5 build.** `make test`, `make lint`, `cargo build --release`, licence
  gate clean, no new dependency.
- **G6 docs.** JSON/CSV, `env.json` and README under `measurements/plan4/`;
  no raw transcripts, provari artifacts or `__pycache__`; report and
  initiative/bench updates.

## Tasks (stop only at a phase boundary)

- **P0** Prior-art check; probes in `measurements/plan4/probes.json`;
  `run_sweep.py` (stdlib only).
- **A** Calibration fields/CLI/params/tests; sweep; freeze point.
- **B** Settle retention + writer fallback; memory split + aux budget; tests
  and the `core` pressure diagnostic.
- **C** D21 node gate; 10 s `core`/`easy-win` gate + D14 proof check.
- **D** G1 pass: both-direction ablation + ledger row (no goal claim). G1
  fail: close item 3 with pivot options (M4 first, or a new formulation).
  Then `report4.md`, `initiative.md` (D23, status, history), `bench.md`.

## Risks

| Risk | Mitigation |
| --- | --- |
| Calibration overfits `core`/seed 0 | Sweep scores only expansions; confirm on 24 cells × seeds 0–2; `easy-win` guard |
| Bigger numbers hurt `easy-win` | EF-PN off already reaches 110 k; sweep both; report the Pareto point |
| Settle retention loses a proof path | Test selected/unselected children; writer fails closed; D14 check |
| Share/aux change the tree | Both are parameters; default today; ablate after G1 |
| Item 3 still fails G1 | Explicit kill: close the item, no plan5 split, record the pivot |

## Report and deliverables

`report4.md`: headline, provenance, commands, kill-review answers, probe
table, sweep/chosen point, proof-table/auxiliary results, G1 per cell/seed
(expansions, child evals, wall, proof verdict), gates G1–G6, verdict, not
measured, problems, missing tests, next steps, deliverables, and a closure
block with a copy-pasteable next-session prompt. State whether item 3 is closed.

- **Engine:** `src/guide/{mod,eval}.rs`, `src/search/frame.rs`,
  `src/table/mod.rs`, tests.
- **Measurements:** `measurements/plan4/{run_sweep.py, README.md, env.json,
  probes.json, sweep.json, gate.json, wall.json, ablation.json?}`.
- **Docs:** `initiative.md`, `bench.md`, `AGENTS.md` if needed,
  `docs/plans/README.md`, `report4.md`.
