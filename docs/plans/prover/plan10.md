# Prover Plan 10: item 18 — restarts + core throughput (reopened)

Initiative: [`prover`](initiative.md), backlog **item 18**, decisions **D5**,
**D21**, **D27**. Drafted 2026-11-10 after [`report9`](report9.md). The owner
takes report9's recommendation: **reopen item 18** with Tier 1 coverage at
120 s as the first checkpoint. This document does not implement; it is the
plan-session deliverable.

## Premise to test

Tier 1 failed at **1/3**: `easy-win` is 3/3, `core` and `deep-win` are
censored at 120 s. The failure is a tree/throughput failure, not a deadline
artefact. Item 18 is the parked stack that targets it. If restarts +
generation-aware replacement + one measured per-expansion work item cannot
move `core`/`deep-win`, **stop and report**; do not re-scope, do not start
`REPLAY-BOUND`, do not reopen the closed hand-guide/auxiliary line.

## Absolute distance (report9, same session)

| Cell | product 120 s | provari hand | Factor |
| --- | --- | --- | ---: |
| `core` | 0/3, 1.96 M exp | 0.500 s / 342,289 nodes | ≥ 240.0× |
| `easy-win` | 3/3, 5.765 s | 0.105 s / 72,523 nodes | 54.9× |
| `deep-win` | 0/3, 2.03 M exp | 6.590 s / 4,574,912 nodes | ≥ 18.2× |

`core` does ~5.7× provari's node count (tree problem); `deep-win` does fewer
expansions than provari's nodes yet censors (per-expansion cost problem).
`neither` costs 2.83×/3.44× provari's ns per node (report8), so throughput is
first-order. Coverage is **1/3**; Tier 2 is not near.

## Scope

**In:** Phase A attribution; a deterministic restart schedule +
generation-aware TT replacement behind opt-in flags; **one** per-expansion
work item chosen by Phase A; unit/property tests; a measurement README and
`report10.md`; the joint docs update.

**Out:** `REPLAY-BOUND`; hand-guide/auxiliary tuning (closed); any coverage
budget above 120 s; M4/`planX`; the portfolio (horizon 8); default-flag
changes before measurement; new guide mechanisms; a second stack.

## Named stack (D21)

**`RESTART-GEN`** = (1) deterministic restart schedule, (2) generation-aware
TT replacement, (3) one per-expansion work item selected by Phase A. One
gate, judged on the whole stack; both-direction ablations only after the gate.

## Design

### Phase A — attribution (measurement only)

Profile the product point (`--guide hand --guide-scale 64 --guide-ef-weight
0`) on `core`/`deep-win` at fixed `--evals 20000000 --memory 192`, seeds 0–2.
Decompose per-expansion wall into: frame entry (movegen, effects, guide eval,
settle, simulation), MID descent (board clone/`do_move`), `select`, table
ops. Record `expansions`, `child_evals`, `sim_nodes`, `settle_nodes`,
`aux_nodes`. **Stop rule:** if the only material lever is auxiliary work
(sim/settle), stop and report — that line is closed. Otherwise pick the
largest non-auxiliary item for Phase C.

### Phase B — restarts + generation-aware replacement (opt-in, default off)

- **Restart.** Reset the root's `Open` numbers to `(1, 1)`, clear `line`, keep
  the proof table, path-independent refutations and history. Deterministic
  schedule only: `--restart-every N` expansions and/or `--restart-stall N`
  expansions without root `pn`/`dn` improvement (`0` = off). No wall clock
  (D22/D24).
- **Generation-aware replacement.** TT slots carry a small generation tag; a
  restart increments the generation. Lookups accept current-generation `Open`
  values and all path-independent refutations; replacement prefers stale
  slots. Keep 16-byte slots / 64-byte clusters and no new allocation; the
  exact bit encoding is an implementation choice pinned by tests.
- **Soundness.** A restart changes order only. Proofs never live in the TT;
  the proof table is untouched; no new refutation is invented. `Open`
  saturation stays at `OPEN_MAX`, never `INFINITY`.
- **Tests.** Restart-off is counter-identical to the current default;
  `--guide none --seed 0` stays m1-identical; the schedule is deterministic on
  repeat; generation replacement is deterministic; the proof table survives a
  restart; no false refutation; a replayed proof is D14-accepted.

### Phase C — one per-expansion work item

Implement only the top Phase-A item, behind the same stack flag. No new guide
mechanism and no auxiliary tuning. Candidates (measurement decides): a
reusable board/`StateInfo` instead of a clone per descent; frame-buffer reuse;
a cheaper guide eval with identical decisions. If Phase A finds no non-aux
item with a material share, skip Phase C and report.

### Phase D — measurement

**Checkpoint 1 (Tier 1, D27):** 3/3 at `--seconds 120 --memory 192`, every
claim D14-accepted by both checkers, no fixture regression. **Checkpoint 2
(item 18 gate):** at `--seconds 10 --memory 192`, `core` proved, `easy-win`
≤ 2 s, `deep-win` classified, no `guidance-v1` refutations, ≥ 20 frozen cells
× seeds 0–2. Both-direction ablations after the gate. A censored cell is
never converted and the budget is never raised.

## Gates

- **G1 data.** Same-session provari controls; one process at a time; parsed
  JSON, `env.json`, README under `measurements/plan10/`.
- **G2 determinism.** Restart-off identity; deterministic schedule and
  generation replacement; no wall clock in the search.
- **G3 goal.** Staged: Checkpoint 1 then Checkpoint 2. A lower coverage is
  never called a pass; a censor is reported with its exact cell.
- **G4 soundness.** No refutation or never-prove; `make test-full` and
  `make veto` green; every claimed proof accepted by both checkers.
- **G5 docs.** `initiative.md` item 18/D5 status, `bench.md` history,
  `report10.md`; no GPL dependency, raw transcript, provari artifact, proof
  file or `__pycache__`.

## Tasks and stop points

- **A** attribution driver + Phase A; stop if the only lever is auxiliary.
- **B** `RESTART-GEN` behind flags + tests; stop on any determinism or
  soundness mismatch.
- **C** one per-expansion item + tests; stop if Phase A gave no candidate.
- **D** Checkpoint 1, then Checkpoint 2, then ablations.
- **E** `report10.md` with both checkpoints and exactly one recommendation.

## Risks

| Risk | Mitigation |
| --- | --- |
| Restarts lose proofs or invent refutations | Proof table untouched; only `Open` numbers reset; D14 + both checkers |
| Generation tag breaks the 16-byte slot | Carve from `meta` or a side array; pin with size/encoding tests |
| `easy-win` ≤ 2 s needs aux cuts (closed line) | Checkpoint 1 is coverage first; report the gap, do not reopen aux |
| Restarts are a no-op on `deep-win` (throughput) | Phase A decides; Phase C targets the measured cost |
| Re-pin is read as goal-shopping | D27 stays; Tier 2 retained; coverage stated before wall |

## Formulation option (not this plan)

**Portfolio:** run the bare tree and the hand tree, take the first proof. This
changes *what counts as the product configuration* and drops the
single-config constraint. Estimated gain: `core` coverage at 60 s is already
measured (bare 17.1 s, 34.6×); `deep-win` is unknown. Cheapest experiment: one
bare-tree 120 s run on `deep-win`. It is an owner/goal decision, not item 18;
not in scope here.

## Deliverables

- Data: `measurements/plan10/{README.md,env.json,attribution.json,coverage.json}`.
- Docs: this plan; `report10.md`; `initiative.md` item 18/D5 status and
  priority; `bench.md` history. No ledger row unless a checkpoint passes.
- Engine: `RESTART-GEN` behind opt-in flags; defaults unchanged until the
  gate passes.
