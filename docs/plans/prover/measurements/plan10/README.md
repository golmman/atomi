# plan10 measurements

Plan: [`../../plan10.md`](../../plan10.md) (item 18, `RESTART-GEN`).
Report: [`../../report10.md`](../../report10.md). Parsed output only: no raw
transcripts, perf data, provari artifacts, proof files or `__pycache__` are
committed. Perf data files are written to a temporary directory and deleted
by the driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary, corpus/guidance sha256, provari version/commit, product flags, fixed-eval budget |
| `attribution.json` | Phase A: product at fixed 20 M evals on core/deep-win, seeds 0–2, with `perf record -e cycles:u` symbol shares; sim-off / closures-off / both-off ablations; medians and the stop-rule record |
| `coverage.json` | Phase D record: **not run**, because the Phase A stop rule fired |
| `run_plan10.py` | stdlib-only driver (`env`, `attribution`, `coverage`, `all`) |

No `coverage` rows and no D14 rows: Phase A stopped before Phase B/C/D. The
plan's stop rule is explicit: if the only material lever is auxiliary work
(sim/settle), stop and report; that line is closed.

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan10/run_plan10.py --phase env
python3 docs/plans/prover/measurements/plan10/run_plan10.py --phase attribution
python3 docs/plans/prover/measurements/plan10/run_plan10.py --phase coverage
make test
make lint
```

One atomi process at a time. `attribution` ran first, then `coverage` (which
only records the stop), then the gates. `perf` is used only for the
function-level symbol profile; the ablation rows are plain atomi JSON.

## Row counts and units

- **Phase A:** 6 product rows (2 cells × seeds 0–2) + 18 ablation rows
  (2 cells × 3 seeds × {sim off, closures off, both off}).
- `expansions` is atomi's main-search node expansion (the analogue of
  provari's `nodes`). `child_evals` is one per child classified during an
  expansion. `aux` is `sim_nodes + settle_nodes`.
- The product point is `--guide hand --guide-scale 64 --guide-ef-weight 0`
  (plan4), `--evals 20000000 --memory 192`, seeds 0–2.
- `perf` is `perf record -e cycles:u -F 4000`; the symbol shares are
  function-level. Inlining prevents an exact section split, so the ablation
  rows, not the symbol shares, carry the auxiliary/non-auxiliary split.

## Phase A — attribution at fixed 20 M evals

Medians over seeds 0–2. All rows censor at the 20 M `child_evals` cap
(`exit=evals`), so the wall is the completed work for the same cap.

| Cell | product | sim off | closures off | both off | aux share | non-aux share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| core | 41.781 s | 12.881 s | 11.179 s | 3.160 s | **92.44%** | 7.56% |
| deep-win | 38.560 s | 12.076 s | 7.442 s | 2.533 s | **93.43%** | 6.57% |

The both-off floor is the non-auxiliary search: movegen at node entry, the
`Mover`/`Effect` pass, guide class/eval, `select`, table ops and the MID
descent. It is 2.5–3.2 s for 20 M child evals, versus 38–42 s for the product.
Removing simulation alone leaves 12–13 s; removing the one-ply settle alone
leaves 7–11 s. The two overlap (both use movegen and `do_move`), so the
individual deltas do not add; the both-off row is the clean floor.

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

## Stop rule

**Fired.** The only material lever is auxiliary work (simulation and the
one-ply settle): 92–94% of the product wall on both cells and all seeds. The
non-auxiliary floor is 6.6–7.6% of the product wall, so even eliminating all
non-auxiliary work cannot close the `core`/`deep-win` gap. The plan closes
that line: no `RESTART-GEN` implementation, no Phase C work item, no Phase D
checkpoints, no re-scope, no `REPLAY-BOUND`.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 6 product + 18 ablation rows; parsed JSON, `env.json`, one process at a time; perf data deleted |
| G2 determinism | **pass**: no engine change; the same binary and flags reproduce the counters |
| G3 goal | **not reached**: Phase A stop rule fired before Checkpoint 1/2; no coverage claim |
| G4 soundness | **pass**: no engine change; `make test-full` (slow tier + `make veto`) and `make lint` green |
| G5 docs | **pass**: no GPL dependency, raw transcript, perf data, provari artifact, proof file or `__pycache__` |
