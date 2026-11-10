# Prover Report 11: item 20 — bare tree is 2/3 at 120 s; `deep-win` censors

Plan: [`plan11.md`](plan11.md) (item 20, D6/D24/D27). Executed 2026-11-10.
Measurement only: no engine, test or default change. The bare tree proves
`core` and `easy-win` 3/3 and `deep-win` 1/3 at 120 s / 192 MB, so **Tier 1
fails at 2/3 cells**. The plan's censor branch fires: no portfolio, no budget
raise, no auxiliary reopen. No ledger row.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `b52e186` on a dirty tree (plan11 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `e416f075…12c811` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, sha256 `2da2a65b…8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Configuration | `--guide none` (bare tree), `--seconds 120 --memory 192`, seeds 0–2 |
| D14 | `--seconds 600 --proof-out`, same flags/seed |

Commands: the five `run_plan11.py` phases in
[`measurements/plan11/README.md`](measurements/plan11/README.md), then
`make test`, `make lint`, `make test-full` (slow tier + `make veto`), one
process at a time.

## Phase A — bare-tree coverage at 120 s

| Cell | proved | wall (median) | expansions (median) | child_evals (median) | length | factor vs provari |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| core | **3/3** | 17.390 s | 9,266,026 | 268,226,989 | 14 / 14 / 24 | 35.3× |
| easy-win | **3/3** | 2.008 s | 1,243,369 | 34,289,765 | 17 / 17 / 19 | 19.1× |
| deep-win | **1/3** | 120.001 s (censor) | 52,301,824 | 1,462,372,673 | 25 (seed 1) | **≥ 18.6×** |

Per-seed walls: core 17.057 / 21.514 / 17.390 s; easy-win 1.684 / 2.174 /
2.008 s; deep-win censor / **111.218 s (proved)** / censor. Same-session
provari hand medians: core 492 ms / 342,289 nodes; easy-win 105 ms / 72,523
nodes; deep-win 6,453 ms / 4,574,912 nodes.

**Tier 1 result: 2/3 cells, fail.** `deep-win` proves on seed 1 at 111.2 s
(1.399 G child evals, length 25) but censors on seeds 0 and 2 at 120 s
(1.461 G / 1.462 G child evals, `exit=timeout`, no refutation). The 120 s
budget is not raised and the censored seeds are not converted.

### Bare vs the hand product (same cells, plan9/plan10)

| Cell | hand product 120 s | bare 120 s | bare gain |
| --- | --- | --- | --- |
| core | 0/3, ≥ 240.0× | **3/3**, 17.390 s, 35.3× | +1 cell, ≥ 6.8× wall |
| easy-win | 3/3, 5.765 s, 54.9× | **3/3**, 2.008 s, 19.1× | 2.9× wall |
| deep-win | 0/3, ≥ 18.2× | **1/3**, ≥ 18.6× | +1 seed |

The bare tree dominates the hand product on every measured cell: more
coverage and a lower wall on the shared cells. The hand stack's auxiliary
work (92–94 % of its wall, report10) is pure cost on this bench.

## D14

| Cell | seed | timed child_evals | proof child_evals | equal | positions | entries | bytes | verdict |
| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | --- |
| core | 0 | 268,226,989 | 268,226,989 | yes | 10,831 | 561,656 | 885,832 | accepted |
| core | 1 | 326,310,697 | 326,310,697 | yes | 11,107 | 707,687 | 906,323 | accepted |
| core | 2 | 267,472,285 | 267,472,285 | yes | 11,879 | 586,600 | 966,903 | accepted |
| easy-win | 0 | 30,335,618 | 30,335,618 | yes | 2,048 | 69,772 | 172,521 | accepted |
| easy-win | 1 | 36,039,603 | 36,039,603 | yes | 2,348 | 76,827 | 197,931 | accepted |
| easy-win | 2 | 34,289,765 | 34,289,765 | yes | 2,390 | 76,187 | 201,222 | accepted |
| deep-win | 1 | 1,398,553,959 | 1,398,553,959 | yes | 49,523 | 2,746,342 | 4,036,530 | accepted |

All seven proofs pass `atomi validate` and `provari check` with equal
`child_evals`. The `deep-win` proof is 4.0 MB / 2.75 M entries, well inside
the 192 MB label. No proof file is committed; the driver deleted them.

## Diagnostic — guidance-v1 seed 0

`--guide none`, 120 s, seed 0: **20/24 proved**. Censored: `deep-win`,
`b9a7ed7eb368`, `3004d09cef79`, `dd3f89766a40`. Plan3's hand stack was 19/24
at 10 s (bare 18/24); the budgets differ, so this is a diagnostic, not a gate.
The bare tree is not worse on the frozen set.

## Decision (plan Phase B, censor branch)

**`deep-win` censors, so Tier 1 is not reached and the plan's stop rule
fires.** Deliberate consequences:

- **No portfolio.** The hand tree also censors `deep-win` (plan9), so a
  bare + hand driver cannot add coverage; it would only add a second
  configuration. Rejected as dominated.
- **No budget raise.** 120 s is the Tier 1 budget; the censored seeds are
  reported, never converted.
- **No auxiliary/guide reopen.** The hand stack is measured worse on every
  bench cell; it stays behind flags.
- **Product configuration.** The data answer the plan's formulation question
  in favour of **bare**: it dominates hand on coverage and wall. Adopting it
  as the default is an owner decision (the plan only auto-adopts on 3/3), but
  no measured cell favours hand. Tier 1 remains **2/3** either way.
- **Next lever.** `deep-win` tree size is the blocker. That is a new tree
  mechanism (not a portfolio, not auxiliary tuning, not a budget change) and
  needs a new plan and owner decision.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 9 coverage + 9 provari + 7 D14 + 24 diagnostic rows; parsed JSON, `env.json`, one process at a time |
| G2 determinism | **pass**: no engine change; D14 equal `child_evals` on all 7 proved rows |
| G3 goal | **fail**: Tier 1 is 2/3 cells at 120 s; `deep-win` 1/3 seeds, exact censor recorded |
| G4 soundness | **pass**: no refutation; 7/7 D14 proofs accepted by both checkers; `make test`, `make lint`, `make test-full` and `make veto` green |
| G5 docs | **pass**: no GPL dependency, raw transcript, provari artifact, proof file or `__pycache__` |

## Findings

1. **The bare tree is the best measured configuration.** It proves `core`
   3/3 at 17.4 s (35.3×) and `easy-win` 3/3 at 2.0 s (19.1×), both far better
   than the hand product (core 0/3, easy-win 5.765 s / 54.9×). The kill
   review's premise is confirmed by measurement.
2. **`deep-win` is the only blocker, and it is seed-sensitive.** Seed 1
   proves at 111.2 s; seeds 0 and 2 censor at 120 s. The tree is ~1.4 G child
   evals, ~52 M expansions, versus provari's 4.57 M nodes. The gap is tree
   size, not per-node cost (bare `R` 1.19/1.33, plan8).
3. **The bare proof is large but fits.** `deep-win` seed 1: 49,523 positions,
   2.75 M entries, 4.0 MB, accepted by both checkers. D14 is not the
   constraint at 192 MB.
4. **The guidance set does not regress.** Bare is 20/24 at 120 s seed 0,
   versus hand 19/24 at 10 s; no refutations.

## Problems, open parts, missing tests

- **Tier 1 is still red (2/3).** The plan does not authorise a new tree
  mechanism; the owner must decide whether to open one for `deep-win`.
- **`deep-win` seeds 0 and 2 are censored, not refuted.** No proof exists for
  them; the censor is the exact result.
- **No default-flag change.** The bare configuration is measured, not made
  the default; that is an owner decision.
- **No new tests.** No engine change; the existing gates cover the run.

## Next step (exactly one recommendation)

**Adopt the bare tree as the product configuration (owner decision) and open
a new plan for a `deep-win` tree-size mechanism; do not build a portfolio,
raise the budget, or reopen the auxiliary line.** The bare tree is 2/3 at
120 s and dominates hand on every measured cell; `deep-win` is 1/3 and is the
only blocker. The cheapest decisive experiment for the next plan is a
tree-size attribution on `deep-win` (where the 52 M expansions go versus
provari's 4.57 M nodes), not another configuration.

## Deliverables

- Data: [`measurements/plan11/`](measurements/plan11/README.md) —
  `run_plan11.py`, `env.json`, `coverage.json`, `guidance.json`, `README.md`.
- Docs: this report; `initiative.md` item 18/20 status and priority;
  `bench.md` history. No ledger row.
- Engine: no change.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report11.md` and
   `measurements/plan11/`. Adopt the bare tree as the product configuration
   (owner decision) and write a plan for a `deep-win` tree-size mechanism:
   attribute the 52 M expansions against provari's 4.57 M nodes, then propose
   one tree mechanism. Do not build a portfolio, do not raise the 120 s
   budget, and do not reopen the hand-guide/auxiliary line."
2. If the owner rejects the bare product, keep the hand stack behind flags
   and record Tier 1 as 1/3; the `deep-win` blocker is unchanged.
3. If the owner accepts 2/3 as the coverage goal, that is a goal change and
   needs a new D-number; this report does not authorise it.
