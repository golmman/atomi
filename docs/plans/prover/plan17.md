# Prover Plan 17: item 24 Phase B — exact replay applicability and the conformance stop

Initiative: [`prover`](initiative.md), backlog **item 24**, decisions **D14**,
**D18**, **D21**, **D24**, **D26**, **D29**, **D30**. Drafted 2026-11-10
after [`report16`](report16.md) and plan session 17. Plan-session deliverable:
it does not implement.

## Premise

The engine is at **`T_2`**; the next rung is **`T_3`** (`deep-win` 3/3 at
120 s). The initiative says "Phase B (exact replay) is the next plan". That
line is stale in two ways.

1. **Exact sibling replay already exists.** `--guide-replay-exact` landed in
   plan13 (`src/guide/replay.rs`) and was measured inside the conformance
   stack: full stack 1.11× `core`, `lo_replay` 0.02 % (report13). A
   standalone "implement Phase B" plan would repeat plan13.
2. **It does not apply at the first divergence.** Phase A's first
   move-divergent positions are attacker nodes, not in check. In
   `measurements/plan16/classify.json` all five `core` move-divergent
   positions have `exact_replay: {holds: false, specific: null}` with
   `attacker_node=True child_in_next_ply=False`. The named divergence is
   quiet grouping, which is already a tested negative (`--guide-group`,
   plan13: 0.96× `core`, `easy-win` regressions).

So the decisive Phase B question is not "implement replay"; it is "does
exact replay apply at the first divergent node, and if not, can it still cut
the tree by 2× elsewhere?" This plan answers both cheaply and stops if the
answer is no. **Scope change:** Phase B is re-scoped from implementation to
applicability + isolation; the execution report records the change.

Plan-session probe (read-only, committed data): plan13 Phase C full stack
529,239 `core` unique vs bare 586,075 (1.11×); `lo_replay` 529,345 (0.02 %);
`lo_fresh` 480,962 (1.22×). No exactness stack has been run in isolation at
the D29 gate.

## Scope

**In:** a read-only applicability check at the plan16 move-divergent
positions; an isolation matrix of the existing `--guide-replay-exact` at
20 M on `core`/`easy-win`/`deep-win` seed 0; if applicable, the conformance
parameters (`--guide-replay-budget N`, `--guide-replay-once`) and an exact
null-threat replay (`--guide-null-exact`); D14 for proved rows; `report17.md`.

**Out:** a default change; a 120 s matrix; Phase C/D; a new search algorithm;
a portfolio; a budget above 120 s; reopening the hand-guide/auxiliary line;
provari proofs or artifacts.

## Design

### B1 — applicability (read-only, may end the plan)

For every move-divergent position in `measurements/plan16/classify.json`
(`core` 5, `easy-win` 1, `deep-win` 3), record whether exact replay is
applicable: the node is a defender (AND) node, the proved child is in the
next ply, and at least one sibling is still `Open`. The check uses the
existing `atomi classify` output and the proof edges; it runs no search and
writes no table. If no position is applicable, Phase B cannot explain the
first divergence; record that and go to B2 (the tree-size test is still
valid) but do not implement B3.

### B2 — isolation matrix (the D29 gate)

`--guide none` (bare) vs `--guide hand --guide-replay-exact` vs
`--guide hand` (control), `--evals 20000000 --memory 192 --seed 0 --stats`,
on `core`/`easy-win`/`deep-win`. Record `unique_keys`, `expansions`,
`child_evals`, `sim_nodes`, `aux_nodes`, exit and proof length. Seed 1 for
the best stack. This is the same gate as plan13 Phase C, but with replay in
isolation rather than inside the full stack.

### B3 — conformance parameters (only if B1 finds an applicable position)

- `--guide-replay-budget N` (default 1000): the per-sibling attacker-node
  budget. Test 16 (provari's `SIMULATION`) and 1000.
- `--guide-replay-once` (default off): mark a sibling as tried so a later
  proved child does not replay it again. Provari marks `simulated`; atomi
  currently retries.
- `--guide-null-exact` (default off): replace the signature comparison in
  `guide.replay` with verified one-ply explosion checks. `created` uses
  `threat::immediate_explosion` on the child; `parried` uses a new
  `threat::null_immediate_explosion` on the parent's null side. No null
  position is keyed, searched, stored, put on `line` or emitted. The
  signature path stays the default.

All three are opt-in and trajectory-neutral when off. The replay still only
returns verified proofs; the caller stores and emits them.

## Gate

- **G1 data.** `applicability.json`, `isolation.json`, `env.json`; parsed
  JSON only, one process at a time.
- **G2 applicability.** At least one first move-divergent position where
  exact replay applies. If none, record it and stop B3; B2 still runs.
- **G3 tree size.** The exact replay stack cuts `core` unique expansions by
  **≥ 2×** vs bare at 20 M, with no `easy-win` regression > 10 %. G3 is the
  D29 gate and runs regardless of G2; a mechanism may cut the tree without
  explaining the first divergence.
- **G4 soundness.** No search change unless B3 runs; every new flag default
  off and trajectory-neutral when off; `make test`, `make lint`,
  `make test-full` green; no GPL dependency, no provari proof committed.
- **G5 docs.** `report17.md`; `initiative.md` item 24 status; no ledger row.

**Stop rule.** G3 fails → close Phase B negative; do not implement Phase
C/D; recommend a formulation decision or a new search algorithm. G3 passes
→ plan18 for the 120 s matrix and Phase C/D; if G2 also failed, plan18 must
carry a first-divergence diagnostic.

## Tasks

- **T1** `measurements/plan17/run_phase_b.py` (stdlib) + `env.json`; pin the
  binary, corpus and oracle; parse the plan16 classify data.
- **T2** B1 applicability; B2 isolation matrix; B3 only if B1 passes.
- **T3** G1–G5, `report17.md`, docs status.

## Risks

| Risk | Mitigation |
| --- | --- |
| The first divergence is not the tree-size gate; a mechanism can cut elsewhere | B2 runs the isolation matrix regardless of B1; B1 only decides whether to implement B3 |
| The existing replay is bounded and retries; the conformance parameters may change the result | Test 16/1000 and once/retry; record both |
| The plan becomes another negative | B1 is cheap and has a real stop; the negative closes the conformance line |
| Provari proof files are GPL artifacts | Scratch only; commit derived summaries, never the proof file |
| The hand stack censors `core` at 120 s | The gate is 20 M evals, not wall; `core` is censored at 20 M in every stack, so unique expansions are the metric |
| A new flag changes the default trajectory | Default off; a test pins the default-off counters equal to the current binary |

## Deliverables

- Data: `measurements/plan17/{README.md,env.json,applicability.json,isolation.json}`.
- Docs: this plan; `report17.md`; `initiative.md` item 24 status.
- Engine: only if B1 passes; `--guide-replay-budget`,
  `--guide-replay-once`, `--guide-null-exact`; tests pinning default-off
  trajectory neutrality. No default change.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan17.md`. Execute
   item 24 Phase B: the applicability check and the 20 M isolation matrix.
   Stop at G2/G3; do not implement Phase C/D."
2. If G2 fails: close the conformance line and write a kill review for a new
   search algorithm (PDS/PN², TT move ordering) or a formulation decision.
3. If G3 passes: write plan18 for the 120 s matrix and Phase C/D.
