# Prover Report 10: item 18 — Phase A stops `RESTART-GEN`; the lever is auxiliary

Plan: [`plan10.md`](plan10.md) (item 18, D5/D21/D27). Executed 2026-11-10.
Measurement only: no engine, test or default change. **Phase A's stop rule
fired.** At fixed 20 M child evals the product wall is 38.2–41.8 s on
`core`/`deep-win`, while the same search with simulation and the one-ply
settle disabled is 2.53–3.21 s. Auxiliary work is **92.4–93.4%** of the
product wall on both cells and all seeds. The plan closes that line, so
`RESTART-GEN` (Phase B), the Phase C work item and the Phase D checkpoints
were **not implemented or run**. No coverage claim, no ledger row.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `12b9b64` on a dirty tree (plan10 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `8dec6b8f…9d909` (unchanged from plan9) |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, sha256 `2da2a65b…8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Product point | `--guide hand --guide-scale 64 --guide-ef-weight 0` (plan4) |
| Phase A budget | `--evals 20000000 --memory 192`, seeds 0–2 |

Commands: the three `run_plan10.py` phases in
[`measurements/plan10/README.md`](measurements/plan10/README.md), then
`make test` and `make lint`, one process at a time. `perf record -e cycles:u
-F 4000` supplied the function-level profile; the perf data files were
temporary and are not committed.

## Phase A — attribution at fixed 20 M evals

All rows censor at the 20 M `child_evals` cap (`exit=evals`), so the wall is
the completed work for the same cap. Medians over seeds 0–2.

| Cell | product | sim off | closures off | both off | aux share | non-aux share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| core | 41.781 s | 12.881 s | 11.179 s | 3.160 s | **92.44%** | 7.56% |
| deep-win | 38.560 s | 12.076 s | 7.442 s | 2.533 s | **93.43%** | 6.57% |

The both-off floor is the non-auxiliary search: node-entry movegen, the
`Mover`/`Effect` pass, guide class/eval, `select`, table ops and the MID
descent. It is 2.5–3.2 s for 20 M child evals. Removing simulation alone
leaves 12–13 s; removing the one-ply settle alone leaves 7–11 s. The two
overlap (both use movegen and `do_move`), so the individual deltas do not
add; the both-off row is the clean floor.

### Function-level perf profile (median share, product)

| Symbol | core | deep-win |
| --- | ---: | ---: |
| `generate_legal_with_state` | 26.84% | 27.83% |
| `Board::legal` | 20.38% | 17.06% |
| `Frame::enter` | 13.31% | 13.08% |
| `guide::eval::hand_eval` | 7.24% | 7.82% |
| `Board::do_move` | 5.91% | 5.99% |
| `attacks_from` | 5.27% | 5.82% |
| `attackers_to_color` | 4.99% | 5.81% |
| `compute_checkers` | 4.41% | 4.84% |
| `threat::immediate_explosion` | 2.26% | 2.13% |
| `populate_state` | 1.92% | 2.16% |
| `has_legal_move_with_state` | 1.84% | 1.78% |
| `__memset_avx2_unaligned_erms` | 1.05% | 0.94% |

Category sums (median): movegen 67.5% / 67.2%, frame entry 23.7% / 24.0%,
MID descent 6.4% / 6.5%, table 0.2% / 0.2%, other 2.0% / 2.0%. The movegen
share includes the auxiliary movegens (settle and simulation), which is why
the function profile alone overstates the non-auxiliary cost; the ablation
rows are the attribution.

## Stop rule and consequence

**Fired.** The plan's Phase A stop rule: *if the only material lever is
auxiliary work (sim/settle), stop and report — that line is closed.* The
non-auxiliary floor is 6.6–7.6% of the product wall, so even eliminating all
non-auxiliary work cannot close the `core`/`deep-win` gap. The plan forbids
re-scoping, `REPLAY-BOUND` and reopening the hand-guide/auxiliary line.

Consequences, all deliberate:

- **No `RESTART-GEN`.** No restart schedule, no generation-aware TT
  replacement, no Phase C work item, no new flags, no tests.
- **No Phase D.** Checkpoint 1 (Tier 1 at 120 s) and Checkpoint 2 (item 18
  wall gate at 10 s) were not run; `coverage.json` records `not-run`.
- **No coverage claim and no ledger row.** The plan9 Tier 1 result (1/3)
  stands unchanged.
- **No auxiliary tuning.** The measured lever is exactly the closed line.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 6 product + 18 ablation rows; parsed JSON, `env.json`, one process at a time; perf data deleted |
| G2 determinism | **pass**: no engine change; the same binary and flags reproduce the counters |
| G3 goal | **not reached**: Phase A stop rule fired before Checkpoint 1/2; no coverage claim |
| G4 soundness | **pass**: no engine change; `make test-full` (slow tier + `make veto`) and `make lint` green |
| G5 docs | **pass**: no GPL dependency, raw transcript, perf data, provari artifact, proof file or `__pycache__` |

## Findings

1. **The product wall is auxiliary work.** Simulation and the one-ply settle
   are 92–94% of the fixed-20 M wall on both `core` and `deep-win`, on every
   seed. The non-auxiliary search is 2.5–3.2 s for the same 20 M child evals.
   This is the same direction as plan7's 64% simulation share on `easy-win`,
   but larger on the two censored cells.
2. **The function profile is consistent but not the attribution.** Movegen is
   the largest symbol group (67%), but it includes the auxiliary movegens.
   The both-off ablation is the clean non-auxiliary floor; the symbol shares
   are recorded as diagnostics only.
3. **`RESTART-GEN` cannot be the next lever under the plan's own rule.** A
   restart schedule and generation-aware replacement change the tree/order;
   they do not remove the auxiliary work that is 92–94% of the wall. The plan
   says stop rather than re-scope.
4. **The plan9 distance is unchanged.** `core` and `deep-win` remain censored
   at 120 s; `easy-win` remains the only Tier 1 cell. No new coverage or wall
   number is claimed.

## Problems, open parts, missing tests

- **Item 18 is not closed by this report.** The plan says stop and report; it
  does not authorise a re-scope or a new stack. The owner must decide whether
  to close item 18 negative or authorise a different formulation.
- **`RESTART-GEN` is unimplemented.** No restart flag, generation tag, test
  or measurement exists. The plan's Phase B/C deliverables are intentionally
  absent.
- **No Phase D data.** `coverage.json` is a stop record, not a coverage
  measurement. No D14 proof was produced or checked in this session.
- **`make test-full`/`make veto` were run anyway** (no search, table,
  proof-output or validator code changed): the slow fixture-oracle tier and
  the veto pass. `make test` and `make lint` are green.

## Next step (exactly one recommendation)

**Do not implement `RESTART-GEN`; close item 18 as negative and authorise the
portfolio probe as the next plan.** The cheapest decisive experiment is one
bare-tree `--seconds 120 --memory 192` run on `deep-win` (the plan's
formulation option): the bare tree already proves `core` at 60 s in 16.6–21.0 s
(plan8), and `deep-win` is the unknown. This changes *what counts as the
product configuration* (bare + hand, first proof), which is an owner/goal
decision, not a re-scope of item 18. Do not raise the 120 s budget, do not
start `REPLAY-BOUND`, and do not reopen the closed hand-guide/auxiliary line.

## Deliverables

- Data: [`measurements/plan10/`](measurements/plan10/README.md) —
  `run_plan10.py`, `env.json`, `attribution.json`, `coverage.json`, `README.md`.
- Docs: this report; `initiative.md` item 18/D5 status and priority;
  `bench.md` history. No ledger row.
- Engine: no change.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report10.md` and
   `measurements/plan10/`. Close item 18 as negative and authorise the
   portfolio probe: one bare-tree `--seconds 120 --memory 192` run on
   `deep-win`, same-session provari hand reference, D14 if it proves. Do not
   implement `RESTART-GEN`, do not raise the 120 s budget, do not start
   `REPLAY-BOUND`, and do not reopen the hand-guide/auxiliary line."
2. If the owner rejects the portfolio, close item 18 and record the goal as
   blocked on a new formulation; Tier 1 remains 1/3 and Tier 2 is not near.
3. If the owner reopens the auxiliary line despite the plan's stop rule,
   that is a new owner decision and needs a new plan; this report does not
   authorise it.
