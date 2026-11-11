# Prover Plan 18: item 25 — DF-PN exactness stack

Initiative: [`prover`](initiative.md), backlog **item 25**, decisions **D14**,
**D18**, **D21**, **D24**, **D26**, **D30**, **D31**. Drafted 2026-11-10 after
[`report17`](report17.md) and plan session 18. Plan only.

## Kill review (fixed prompt)

### 1. Absolute distance to the goal now

The engine is at **`T_2`**. `T_3` needs roughly a **2× tree cut** on
`deep-win` (seed 1 proves at 51.0 M expansions; seeds 0/2 censor at 52.8 M).
`T_4` needs ~12× (10 s); `T_5` needs **20–36×** wall plus 3/3 coverage.

| Cell | bare 120 s | provari hand | factor | 10 s |
| --- | --- | ---: | ---: | --- |
| `core` | 3/3, 17.471 s | 0.488 s / 342,289 nodes | 35.8× | censored |
| `easy-win` | 3/3, 2.023 s | 0.107 s / 72,523 nodes | 18.9× | 1.686 s, 15.757× |
| `deep-win` | **1/3**, seed 1 113.458 s; seeds 0/2 censor | 6.455 s / 4,574,912 nodes | ≥ 18.6× | censored |

The **proof-set gap**: `deep-win` 49,523 proof positions vs 51.0 M expansions
(0.10 %), `core` 10,831 vs 9.27 M (0.12 %), `easy-win` 2,048 vs 1.05 M
(0.20 %). The search explores ~500–1,000× more positions than the proof needs:
tree-directedness, not per-node cost.

### 2. If we started today, would we build this?

We would build the bare DF-PN, proof pipeline, validator and oracle veto. We
would **not** build the hand-guide stack, conformance line, portfolio or
restarts. We would stay on DF-PN and port provari's exactness first.

### 3. Which initiative or plan would you cancel?

Close **item 24** negative and cancel Phase C/D. Cancel the hand-guide/
auxiliary line (already closed), portfolio and restarts. **Do not replace
DF-PN with PDS-PN/PN² now:** that moves off the base and makes future provari
ideas harder to adopt. If exactness fails, re-pin the goal under D30 or
authorise a separate new search.

### 4. Prior-art search (re-run)

- **PDS-PN** (2003) and **PN²** (two-level PN, 2003/2004) are the only
  published families that change the search order globally and have a
  plausible 2× tree-size case. They are **rejected for this plan** on
  architectural grounds: they replace the DF-PN base, and provari's ideas
  (simulation, threat replay, guide numbers, proof keeping) are DF-PN-shaped.
- **Provari's DF-PN exactness** is the untested "secret sauce": its sibling
  simulation replays a proved sibling's proof exactly, and its null-threat
  replay verifies the threat on every defence. Atomi's `simulate.rs` is a
  greedy 6-ply playout, and its null replay compares threat signatures. Those
  are concrete, portable gaps on the same base.
- **Deep df-pn/TCA/SNDA** are smaller DF-PN enhancements with no measured 2×
  case; fallback after exactness. **Parallel PN/DFPN**, **MCTS/PN hybrids**,
  **CNN-guided DF-PN**, **multiple-outcome PN**, **weak PN** and the 2008
  calculation technique are out of scope or do not change the picture.
- **TT move ordering** is not a credible 2× lever: DF-PN selects by `pn`/`dn`,
  and the class/history tie-breaks were measured in the closed hand-guide line.

**Conclusion:** stay on DF-PN; port exact simulation and exact null-threat
replay first. PDS-PN/PN² remain a later owner decision.

### 5. Smallest next experiment

A **read-only mechanism differential** of provari's `simulate`/
`simulate_defences`/`replay_threat` against atomi's `simulate.rs`/
`guide.replay`/`replay.rs`, then a **stack-level port** of exact simulation +
exact null-threat replay on `deep-win` seed 0 at 20 M evals / 192 MB, stop
rule **2× unique-expansion cut** vs bare. This is **not** a repeat of plan17:
plan17 added exact replay on top of the greedy simulation; this plan replaces
the greedy simulation and the signature null replay.

## Decision

**D31 (revised): close item 24 negative; authorise item 25 as a DF-PN
exactness stack, not PDS-PN.** Provari proves `deep-win` in 4.57 M nodes with
DF-PN while our bare tree needs 51.0 M expansions. That ~11× gap is inside the
same algorithm family. The exactness gaps are concrete and portable; a new
search base is premature.

## Scope

**In:** the read-only mechanism differential; opt-in `--guide-sim-exact`
(replace the greedy playout with exact proof replay) and `--guide-null-exact`
(exact null-threat replay), default off; a 20 M isolation matrix on
`deep-win` seed 0 plus `core`/`easy-win` regression; if the gate passes, a
120 s matrix and D14; `report18.md`.

**Out:** PDS-PN/PN²; a default change; parallel search; a portfolio; a budget
or memory raise; reopening the hand-guide/auxiliary line; a goal change;
provari proofs or artifacts.

## Design

### Phase A — mechanism differential (read-only, may stop)

Compare the provari DF-PN exactness paths with atomi's: sibling simulation
(exact proof replay vs greedy playout), null-threat replay (verified vs
signature), and their budgets/cycle rules. Record the gaps and the port
surface. Gate G2: at least one concrete gap that can be ported without
changing the DF-PN base. If none, stop.

### Phase B — exactness stack (opt-in, default off)

Add `--guide-sim-exact` and `--guide-null-exact`. Reuse `guide/replay.rs` for
the exact proof replay; no new search algorithm. `--guide-sim-exact` replaces
the greedy `simulate::playout` adjustment with a verified replay of a proved
sibling's proof. `--guide-null-exact` replaces the signature comparison with
verified one-ply explosion checks on the child and the null side. No null
position is keyed, searched, stored, put on `line` or emitted. Tests pin
default-off counter identity, a mate-in-1/2 and a small win, no false
refutation, and D14 on any proved row.

### Phase C — 20 M isolation (the gate)

`--guide hand --guide-sim-exact --guide-null-exact` vs bare, `--evals
20000000 --memory 192 --seed 0 --stats`, on `deep-win` (gate) and `core`/
`easy-win` (regression). Record unique expansions, expansions, `child_evals`,
`sim_nodes`, `aux_nodes`, wall, `proof_full` and peak memory. Gate G3:
**≥ 2× unique-expansion cut on `deep-win`**, no `core`/`easy-win` regression
> 10 %.

### Phase D — 120 s matrix (only if G3 passes)

3 cells × seeds 0–2 at `--seconds 120 --memory 192`; D14 for every proved
cell. Claim `T_3` only if 3/3 and every claim is accepted by both checkers.
If `T_3` passes, the next plan targets `T_4`/`T_5`; if not, record the censor.

## Gates

- **G1 data.** `diff.json`, `probe.json`, `matrix.json`, `env.json`; parsed
  JSON only, one process at a time.
- **G2 mechanism differential.** At least one concrete exactness gap on the
  DF-PN base; otherwise stop.
- **G3 tree size.** ≥ 2× `deep-win` unique-expansion cut at 20 M, no
  `core`/`easy-win` regression > 10 %.
- **G4 soundness.** Default-off identity; every new flag opt-in; `make test`,
  `make lint`, `make test-full` green; no GPL dependency; no provari proof
  committed.
- **G5 docs.** `report18.md`; `initiative.md` item 25 status; no ledger row
  unless `T_3` passes.

**Stop rule.** G2 fails → close the exactness premise. G3 fails → close the
exactness stack negative; do not implement PDS-PN/PN² in this plan. G3 passes
→ Phase D; a `T_3` pass is the first coverage gain since plan11.

## Risks

Exact simulation may be expensive: use D25 and record `sim_nodes`/`aux_nodes`;
the gate is tree size first. This is not plan17: it replaces the greedy
simulation and signature null replay. If G3 fails, close the stack; PDS-PN/PN²
need a separate decision.

## Deliverables

Data: `measurements/plan18/{README.md,env.json,diff.json,probe.json,matrix.json}`.
Docs: this plan, `report18.md`, `initiative.md` item 25, D31. Engine: two
opt-in flags + tests; no default change.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan18.md`. Execute
   item 25 Phase A (DF-PN exactness differential) and Phase B/C (exact
   simulation + exact null-threat replay on `deep-win` seed 0 at 20 M). Stop
   at G2/G3; do not implement PDS-PN/PN², parallel search, a portfolio, or
   change the default."
2. If G3 fails: close the exactness stack; write a goal re-pin plan under D30
   or a separately authorised PDS-PN/PN² plan.
3. If G3 passes: run the 120 s matrix and D14; claim `T_3` only if 3/3.
