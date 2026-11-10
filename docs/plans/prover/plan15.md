# Prover Plan 15: item 5 (M4) decision + item 6 (M5) head-to-head under D28

Initiative: [`prover`](initiative.md), backlog **item 5 (M4)** and **item 6
(M5)**, decisions **D3**, **D14**, **D17**, **D18**, **D24**, **D27**,
**D28**. Drafted 2026-11-10 after [`report14`](report14.md). Plan-session
deliverable: it decides M4 and plans M5; it does not implement.

## Decision: close item 5 (M4)

**Close M4 as superseded; do not resume it under D28 and do not re-scope it
to `.pvr`.** The evidence:

- M4's scale gate is already met by D18 + D14. Plan11's `deep-win` seed-1
  proof is **49,523 positions / 2,746,342 entries / 4.0 MB**,
  `proof_full: false`, equal `child_evals`, accepted by `atomi validate` and
  `provari check`. That is larger than planX's scale case (`core` aux 120 s:
  14,332 positions / 691 k entries).
- The event-fed worker existed to remove the proof-table **export** cap. The
  largest available proof fits; no `store-full` occurred. The remaining
  proof-table pressure is a **search-memory** effect (plan13 Phase A
  proof-share), not an export effect; the worker would not fix it.
- `.pvr` is an interoperability/format optimisation, not a Tier 1 or Tier 2
  lever; `provari check` reads our JSONL. Defer `.pvr` to the horizon. D3's
  `.pvr` clause is not a blocker for M5.
- Resuming M4 would delay M5 and add no goal lever; re-scoping to `.pvr`
  only would be a non-goal.

Execution records in `initiative.md`/`bench.md`: item 5 **closed**, `.pvr`
deferred, no ledger row. M5 can start.

## M5 plan: item 6 head-to-head under D28

### Premise

D28 makes Tier 1 **2/3** at 120 s / 192 MB; Tier 2 is the same-session 10 s
geomean `< 1.0` on shared cells. Plan11 measured Tier 1 2/3 and 10 s factors
of 19–35×. M5 is therefore expected to **record a Tier 2 failure**, not a
speed win. It must not convert a censored cell, raise the 120 s budget,
reopen the hand-guide/auxiliary line, or build a portfolio.

### Scope

**In:** final same-session head-to-head; Tier 1 confirmation at 120 s; Tier 2
at 10 s; D14 for every claimed win; legacy default/opt controls if the pinned
binary is available; provari v0.2.0 hand stretch if available; default-flag
decision; ledger row; `report15.md`.

**Out:** engine/search/guide changes; a budget above 120 s; hand-guide or
auxiliary reopen; portfolio; `.pvr`; a new speed mechanism; a silent goal
change.

### Design

**Phase A — Tier 1 confirmation (120 s).** `atomi prove FEN --seconds 120
--memory 192 --guide none`, 3 cells × seeds 0–2. Record exit, wall,
`expansions`, `child_evals`, length, `proof_full`. For every proved cell run
D14 (`--seconds 600 --proof-out`, same flags/seed): equal `child_evals`,
`atomi validate` and `provari check` accepted. Expected: `core` 3/3,
`easy-win` 3/3, `deep-win` 1/3 (seed 1). If 2/3 fails, record the exact
censor; never convert it.

**Phase B — Tier 2 head-to-head (10 s).** `atomi ... --seconds 10 --memory
192 --guide none`, 3 cells × 3 repeats; same-session `provari prove ...
--seconds 10 --memory 192 --guide hand`, 3 repeats. Add legacy `default` and
`opt` controls if `/tmp/atomic_solver-5f85133/...` exists; add the v0.2.0
hand stretch column if built. If a control is unavailable, record
`unavailable`; do not substitute. D14 for every atomi cell that wins at 10 s.
Compute per-cell factors and the shared-cell geomean; censored cells get
`≥ 10 / reference_wall` bounds. Expected: only `easy-win` is shared, geomean
≈ 19×; `core`/`deep-win` censored. Tier 2 fails.

**Phase C — default decision.** The default is already `--guide none`;
confirm no flag change. Record the product command. No measured cell favours
hand; the hand stack stays behind flags.

**Phase D — ledger and closure.** Append the first ledger row since plan2:
Tier 1 **2/3** at 120 s; Tier 2 **1/3 shared** at 10 s, geomean ≈ 19×, fail.
State that the initiative goal is not met (Tier 2). Recommend closing the
initiative with a negative Tier 2 result, or an owner decision for a new
speed mechanism; do not start one in M5.

### Tier 2 coverage inconsistency

D28 amends Tier 1 to 2/3 but leaves Tier 2's "coverage stays 3/3" clause. At
10 s atomi coverage is 1/3 (only `easy-win`), so Tier 2 cannot be claimed even
if the geomean passed. M5 records this as an open goal-definition issue; it
does not silently change Tier 2. Because the geomean also fails, the outcome
is unchanged. If the owner wants Tier 2 live, a D29 clarification is needed
(coverage = Tier 1 level, or geomean on shared cells only).

### Gates

- **G1 data.** 9 Tier 1 rows + 9 Tier 2 atomi rows + 9 provari reference rows
  + legacy/stretch rows or `unavailable` + D14 rows; parsed JSON, `env.json`,
  one process at a time.
- **G2 Tier 1.** 2/3 at 120 s, every claim D14-accepted, no fixture
  regression. A failure records the exact censor.
- **G3 Tier 2.** Geomean `< 1.0` on shared cells at 10 s; coverage recorded.
  Expected fail; record factors/bounds, no conversion.
- **G4 soundness.** No refutation; every claimed proof accepted by both
  checkers; `make test`, `make lint`, `make test-full` and `make veto` green.
- **G5 docs.** `report15.md`; `initiative.md` item 5/6 status and priority;
  `bench.md` ledger row and history; no GPL dependency, raw transcript,
  provari artifact, proof file or `__pycache__`.

### Tasks

- **T1** `measurements/plan15/run_plan15.py` (stdlib) + `env.json`; pin the
  binary, corpus and oracle.
- **T2** Phase A (9 runs) + D14; `tier1.json`.
- **T3** Phase B (9 atomi + 9 provari + controls) + D14; `tier2.json`.
- **T4** ledger row, default decision, `report15.md`; stop at each phase
  boundary.

### Risks

| Risk | Mitigation |
| --- | --- |
| Tier 2 is known to fail; M5 looks like a formality | It is the required final head-to-head and the first ledger row since plan2; record the exact gap, do not dress it up |
| 120 s Tier 1 re-run is expensive | 9 runs, one process at a time; cite plan11 only if the binary/flags are unchanged, otherwise re-run |
| Legacy/stretch binaries are missing | Record `unavailable`; the gate depends only on atomi + provari hand |
| Tier 2 coverage clause is stale | Record the inconsistency; do not silently amend it; the geomean fails anyway |
| M5 drifts into a new mechanism | Scope excludes engine changes; a new mechanism needs a new plan and owner decision |

### Deliverables

- Data: `measurements/plan15/{README.md,env.json,tier1.json,tier2.json}`.
- Docs: this plan; `report15.md`; `initiative.md` item 5/6 status;
  `bench.md` ledger row and history.
- Engine: no change.

### Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan15.md`. Close item
   5 (M4) as superseded and execute item 6 (M5) under D28: Tier 1 2/3 at
   120 s, Tier 2 10 s head-to-head, D14, default decision, ledger row and
   `report15.md`. Do not raise the 120 s budget, reopen the
   hand-guide/auxiliary line, or build a portfolio."
2. If the owner rejects closing M4: re-scope it to `.pvr` only, with a
   separate plan and no M5 dependency.
3. If the owner wants Tier 2 live after M5: write a D29 goal-definition plan
   (coverage clause) before any new speed mechanism.
