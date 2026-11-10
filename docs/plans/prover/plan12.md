# Prover Plan 12: item 21 — `deep-win` tree size: entry reuse

Initiative: [`prover`](initiative.md), backlog **item 21** (proposed), decisions
**D24**, **D27**. Drafted 2026-11-10 after [`report11`](report11.md). This is a
plan-session deliverable: it does not implement.

## Premise to test

The bare tree is the best measured configuration and the product candidate
(report11). `deep-win` is the only blocker: 1/3 seeds at 120 s, 52.3 M
expansions / 1.46 G child evals, against provari hand's 4.57 M nodes / 6.45 s.
Per-node cost is competitive (bare `R` 1.19/1.33, plan8), so the gap is **tree
size**, not movegen.

Two plan-session probes narrow the hypothesis:

- **Move ordering is not the missing tree lever at a fixed budget.** At
  `--evals 20M`, `deep-win` seed 0: bare 668,608 expansions / 1.08 s; the
  calibrated hand stack with simulation and closures off (plan10 `both_off`)
  666,390 / 2.53 s. The hand ordering does not shrink the tree per eval; it
  only adds per-node cost.
- **TT capacity is not the cause at this horizon.** Bare `deep-win` at
  `--evals 20M`, seed 0: 64 MB 667,386 expansions; 192/512/1024 MB all
  668,608. More table does not reduce the count.

**Hypothesis.** The tree is dominated by **re-expansions** of positions whose
stored value already crosses the incoming thresholds. `search::mid` never
consults the node's own stored value at entry; it always rebuilds the frame.
Provari's `mid` does consult it (idea only, not code). The ratio 52.3 M
expansions / 2.8 M proof entries ≈ 18.5 is consistent with a large
re-expansion factor.

## Absolute distance (report11, same session)

| Cell | bare 120 s | provari hand | factor |
| --- | --- | --- | ---: |
| `core` | 3/3, 17.390 s, 9.27 M exp | 0.492 s / 342,289 | 35.3× |
| `easy-win` | 3/3, 2.008 s, 1.24 M exp | 0.105 s / 72,523 | 19.1× |
| `deep-win` | **1/3**, seed 1 111.218 s; seeds 0/2 censor, 52.3 M exp | 6.453 s / 4,574,912 | ≥ 18.6× |

Tier 1 is **2/3**. This plan targets the `deep-win` tree only; it makes no
Tier 2 claim.

## Scope

**In:** measurement-only attribution counters; one mechanism, **ENTRY-REUSE**
(a node-entry stored-value check), behind one opt-in flag; `deep-win` /
`core` / `easy-win` measurement at 120 s; D14 for every proved cell; docs.

**Out:** a portfolio; a budget above 120 s; reopening the hand-guide/auxiliary
line; move ordering, EF-PN, restarts, simulation or settle changes; M4/`planX`;
a default-flag change before the gate; a new proof format.

## Design

### Phase A — attribution (measurement only)

Add counters to `SearchResult` (reported by `--stats` or the existing JSON):
`unique_keys` (distinct keys expanded), `re_expansions = expansions -
unique_keys`, `entry_proof_hits`, `entry_tt_hits`, `entry_would_return` (a
stored value that already crosses the incoming `(tpn, tdn)`), attacker vs
defender expansions, a depth histogram, and the selected child's class. No
behaviour change: `--guide none --seed 0` must stay counter-identical to m1.

Run `deep-win` seed 0 at `--evals 20M` and at `--seconds 120`, plus `core` and
`easy-win` at `--evals 20M` as controls. **Stop rule:** if
`entry_would_return` is below 10 % of expansions at 20 M, stop before Phase B;
report the attribution and propose a different tree mechanism. Do not
implement a mechanism the data do not support.

### Phase B — ENTRY-REUSE

At `mid` entry, after the stop/ply checks and before `Frame::enter`, look up
the node's own key with `Tables::lookup` (proof table first, then TT). If the
stored value is usable and already crosses the incoming thresholds
(`pn >= tpn || dn >= tdn`), return it without expanding and count
`entry_hits`. A stored proof/refutation is exact and already used for children;
a stored `Open` pair is completeness-only, as everywhere else in the TT.

Behind `--entry-reuse` (default off), so the bare baseline is unchanged. The
check must not write the tables; it only skips an expansion. Tests: a stored
`Open` pair above threshold returns without expanding; a stored proof returns;
a stored value below threshold expands; flag-off counters are identical to m1;
flag-on has no refutation or never-prove.

### Phase C — measurement

`deep-win`, `core`, `easy-win` × seeds 0–2 at `--seconds 120 --memory 192`
with `--entry-reuse`, same-session bare control. D14 (`--seconds 600
--proof-out`, same flags/seed) for every proved cell. Record `entry_hits`,
`re_expansions`, wall, `expansions`, `child_evals`, length and verdict.

## Gates

- **G1 data.** Attribution rows and the 9-cell matrix + bare controls + D14
  rows; parsed JSON, `env.json`, one process at a time.
- **G2 stop rule.** Phase B starts only if `entry_would_return` ≥ 10 % of
  expansions at 20 M on `deep-win`.
- **G3 mechanism.** `deep-win` **3/3** at 120 s with 3/3 D14-accepted proofs;
  `core`/`easy-win` stay 3/3 and their wall is not worse than bare by more
  than 10 %. A censored cell is reported, never converted; the budget is not
  raised.
- **G4 soundness.** No refutation or never-prove; every claimed proof accepted
  by `atomi validate` and `provari check`; `make test-full` and `make veto`
  green.
- **G5 docs.** `initiative.md` item 21 status, `bench.md` history,
  `report12.md`; no GPL dependency, raw transcript, provari artifact, proof
  file or `__pycache__`.

## Tasks and stop points

- **A** counters + attribution; stop if the 10 % stop rule fires.
- **B** `--entry-reuse` + tests; stop if the 20 M `deep-win` count does not
  fall.
- **C** 120 s matrix + D14; stop if `deep-win` censors (report the censor).
- **D** `report12.md` + docs; exactly one recommendation.

## Risks

| Risk | Mitigation |
| --- | --- |
| The entry check never fires (the parent's thresholds already exceed the stored value) | Phase A stop rule; do not implement on a hunch |
| A stale `Open` value loses a win (completeness) | Only skip when it crosses the thresholds; compare all cells; no refutations |
| The flag changes the trajectory | Default off; D14 equal `child_evals`; flag-off m1 identity test |
| `deep-win` still censors | Report the exact censor; the next lever is a different tree mechanism, not a budget raise |
| Scope creep into ordering/aux | One mechanism, one flag; the hand/aux line stays closed |

## Formulation options (ideation)

- **A (recommended): ENTRY-REUSE.** Gain: if re-expansion is 10–20×, it could
  bring `deep-win` under 120 s at no per-node cost. Cheapest experiment: the
  Phase A counters on one 20 M `deep-win` run.
- **B: move-limit iterative deepening** (provari's `most`/`remaining` keying).
  Gain: prunes deep lines; cost: a new key dimension and proof-table/TT
  changes. Not this plan.
- **C: proof-table keyed by remaining + iterative shortening.** Gain: shorter
  proofs and reuse; cost: large, and it changes the proof format contract.
- **D: change the goal to 2/3 or raise the budget.** Reject: goal-shopping.

## Deliverables

- Data: `measurements/plan12/{README.md,env.json,attribution.json,coverage.json}`.
- Docs: this plan; `report12.md`; `initiative.md` item 21 status and priority;
  `bench.md` history. No ledger row unless Tier 1 passes.
- Engine: `src/search/mod.rs` (entry check, counters), `src/cli.rs`/`main.rs`
  (`--entry-reuse`, stats), tests. No default change.
