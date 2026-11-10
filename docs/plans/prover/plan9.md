# Prover Plan 9: item 19 — coverage-first re-pin and one bounded proof-replay stack

Initiative: [`prover`](initiative.md), backlog **item 19**, decision **D27**.
Drafted 2026-11-09 after the plans 7–8 kill review. The owner accepts the
plan8 recommendation: the single wall-geomean goal is re-pinned to a tiered
goal, **coverage first**. `bench.md`, the success condition, the ledger and
D26's status are updated together with this plan; no silent change.

## Absolute distance (report8, same session)

| Cell | product 60 s | provari hand | Factor |
| --- | --- | --- | ---: |
| `easy-win` | 6.013 s proved | 0.103 s / 72,523 nodes | 58.4× |
| `core` | censored (0.94 M exp) | 0.494 s / 342,289 nodes | ≥ 121.5× |
| `deep-win` | censored (1.02 M exp) | 6.413 s / 4,574,912 nodes | ≥ 9.36× |

Coverage at 10 s is **1/3**. The guide-free tree proves `core` at 60 s in
17.1 s (34.6×), so even the zero-guide floor is far from the wall goal.
`neither` costs 2.83×/3.44× provari's ns per node, so core throughput is
first-order; this is not a guide-only gap. The wall goal is not near-term.

## D27 — the re-pinned goal (owner decision)

**Tier 1, coverage:** on the frozen shared-win set (`core`, `easy-win`,
`deep-win`), prove **3/3** with the product configuration at
`--seconds 120 --memory 192`; every claimed win follows D14 (identical
`child_evals` on a `--seconds 600 --proof-out` run, accepted by
`atomi validate` and `provari check`); no fixture-oracle regression.
A censored cell is never converted, and a longer run is never substituted.

**Tier 2, wall:** only after Tier 1 passes, the original condition applies on
the same three cells at the canonical 10 s budget: geometric mean of
`atomi / provari --guide hand` wall `< 1.0`. It is retained, not deleted.
The node/`child_evals` score option is **not** chosen.

The reference is re-pinned only via D17 at M5. `120 s` is the longest
coverage budget already used by a product proof (`core` aux, report4); it is
not the 600 s D14 export budget. Coverage is stated in every future ledger
row before its wall factors.

## Scope

**In:** D27's wording and the joint docs update; a stdlib coverage driver on
the three cells at 120 s, seeds 0–2, same-session provari references; D14 for
every proved cell; **one** opt-in bounded exact sibling-proof-replay stack
(`REPLAY-BOUND`) as the single wall attempt, with the pre-registered stop
rule below; unit/property tests; `report9.md`.

**Out:** item 18 restarts, a second replay stack, another hand-guide or
auxiliary-budget tuning plan, item-15 `GROUP` re-scoping, M4/`planX`, broad
proof-format changes, a node-score goal, and any budget above 120 s for
coverage.

## Design

### Phase A — coverage-tier baseline (measurement only)

Product point `--guide hand --guide-scale 64 --guide-ef-weight 0`, canonical
`--seconds 120 --memory 192`, three cells × seeds 0–2, one process at a time.
Record exit, wall, `expansions`, `child_evals`, `sim_nodes`, `settle_nodes`,
`aux_nodes`, length and proof verdict. Re-run provari hand in the same
session for the factor column.

For each proved cell run D14 (`--seconds 600 --proof-out`, same flags/seed);
require equal `child_evals` and both checkers. **Tier 1 passes only on 3/3
with 3/3 accepted proofs.** If Phase A already passes, Tier 1 is recorded and
Phase B is entered only if the owner retains the wall target.

### Phase B — `REPLAY-BOUND` (one named stack, default off)

Replace the greedy sibling playout with a deterministic **exact replay of an
already-proved sibling defence** when the stack is enabled. The proof table is
the only source of a line; a missing or stale entry fails closed (no settle).
Replay may settle an open sibling as proved only when it reaches a winning
terminal inside the remaining 50-move limit; events are emitted on the real
sibling position and its real key, as in the `GROUP` path. No synthetic key,
no null-position search, no new proof format.

Flags: `--guide-replay-visits N` (`0` = off) plus a `replay` mechanism in the
deterministic guide flag set. The bound counts positions visited by one replay,
never wall time. Defaults stay unchanged; `--guide none --seed 0` must stay
counter-identical to m1.

Tests before measurement: replay-off is counter-identical to the current
default; a replayed proof has the same winning value and an emitted proof
accepted by both checkers; an injected missing proof fails closed; the visit
bound is deterministic on repeat.

### Pre-registered stop rule for `REPLAY-BOUND`

At fixed `--evals 20000000 --memory 192`, seeds 0–2, compare `REPLAY-BOUND`
against the current `hand` stack **before** any canonical-wall claim:

1. median `core` expansions fall by **≥ 2×**; and
2. median `easy-win` expansions do **not** rise by more than **10%**; and
3. no refutation on `guidance-v1` seed 0; and
4. any claimed proof is D14-accepted by both checkers.

If any clause fails, **close the replay line immediately**: no re-scope, no
second replay stack, no auxiliary tuning. If it passes, run canonical 10 s
and 120 s measurements; Tier 2 is still only claimed if Tier 1 is green.

## Gates

- **G1 data.** Phase A 9 cells (3 cells × 3 seeds) + same-session provari; D14
  rows for every proved cell; parsed JSON, `env.json`, one process at a time.
- **G2 determinism.** Replay-off counter-identical to the current default;
  `--guide-replay-visits` deterministic; no wall clock in the search.
- **G3 goal.** Tier 1 requires 3/3 at 120 s with 3/3 D14-accepted proofs. If
  Tier 1 fails, report the exact censored cells; do not call a lower coverage
  a pass and do not raise the budget.
- **G4 soundness.** No refutation or never-prove; `make test-full` and
  `make veto` green for the search/table/proof change; `make lint` green.
- **G5 docs.** `bench.md`, the success condition, the ledger and D26 status
  already carry D27; `report9.md`; no GPL dependency, raw transcript,
  provari artifact, proof file or `__pycache__` in the repo.

## Tasks and stop points

- **A** driver + coverage baseline + D14; stop if provari controls are
  missing (INCONCLUSIVE, never frozen reference).
- **B1** `REPLAY-BOUND` behind the opt-in flag + tests; stop on any mismatch
  or failed stop-rule clause.
- **B2** stop-rule matrix at 20 M evals; then canonical measurements.
- **C** `report9.md` with both tiers stated and exactly one next
  recommendation.

## Risks

| Risk | Mitigation |
| --- | --- |
| 120 s is still too short for `deep-win` | Report the censor; Phase B is the one wall/tree attempt; do not raise the budget |
| Replay proves a false win | Proof table is the only source; terminal re-check; D14 + both checkers; fail closed |
| Replay reopens the closed hand-guide line | Sibling proof replay only; no `GROUP`, no aux tuning, one stack and a stop rule |
| Re-pin is read as goal-shopping | D27 pre-registers both tiers; existing bench rows are not rewritten; Tier 2 stays the wall goal |
| Coverage tier hides wall regressions | Every ledger row states coverage first and wall factors only on shared cells |

## Report and deliverables

`report9.md` (≤ 12 KB): D27 wording, coverage matrix, D14 evidence, replay
stop rule, gates and one recommendation.

- **Measurements:** `measurements/plan9/{README.md,env.json,coverage.json,
  replay.json}` (only if Phase B runs).
- **Docs:** `initiative.md` (success condition, D26 status, D27, items 18/19,
  history), `bench.md` (goal, ledger semantics, history), `report9.md`.
- **Engine:** replay flag and code only if Phase B is authorised; defaults
  unchanged.
