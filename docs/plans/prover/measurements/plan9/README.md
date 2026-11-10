# plan9 measurements

Plan: [`../../plan9.md`](../../plan9.md) (item 19, D27 Tier 1 coverage).
Report: [`../../report9.md`](../../report9.md). Parsed output only: no raw
transcripts, provari artifacts, proof files or `__pycache__` are committed.
Temporary proofs are written under `proofs/`, checked, then deleted by the
driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary, corpus/guidance sha256, provari version/commit, product flags, budgets |
| `coverage.json` | Phase A: product at 120 s on core/easy-win/deep-win, seeds 0–2; same-session provari hand references; factors; D14 rows |
| `run_plan9.py` | stdlib-only driver (`env`, `coverage`, `provari`, `d14`, `all`) |

No `replay.json`: Phase B (`REPLAY-BOUND`) was not started. Tier 1 failed and
the owner did not retain the wall target in this session.

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan9/run_plan9.py --phase env
python3 docs/plans/prover/measurements/plan9/run_plan9.py --phase coverage
python3 docs/plans/prover/measurements/plan9/run_plan9.py --phase provari
python3 docs/plans/prover/measurements/plan9/run_plan9.py --phase d14
make test
make lint
```

One atomi/provari process at a time. `coverage` ran first, then the
same-session `provari` block, then `d14` for every proved coverage row.

## Row counts and units

- **Phase A:** 9 atomi coverage rows (3 cells × seeds 0–2) + 9 provari rows
  (3 cells × 3 repeats) + 3 D14 proof rows (easy-win seeds 0–2).
- `expansions` is atomi's main-search node expansion (the analogue of
  provari's `nodes`). `child_evals` is one per child classified during an
  expansion; `nodes` is provari's search node count. `aux` is
  `sim_nodes + settle_nodes`.
- A censored row is never converted: its factor is a lower bound on the
  completed work. D14 proof runs use `--seconds 600 --proof-out` with the
  same flags and seed; the driver requires equal `child_evals` and both
  checkers.

## Phase A — Tier 1 coverage at 120 s

Product point `--guide hand --guide-scale 64 --guide-ef-weight 0`,
`--seconds 120 --memory 192`, seeds 0–2.

| Cell | proved | wall (median) | expansions (median) | child_evals (median) | aux (median) | length | factor vs provari |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| core | **0/3** | 120.019 s | 1,956,864 | 60,152,802 | 414,214,167 | — | **≥ 240.0×** |
| easy-win | **3/3** | 5.765 s | 115,147 | 3,147,781 | 21,700,695 | 18 / 18 / 16 | 54.9× |
| deep-win | **0/3** | 120.026 s | 2,030,592 | 61,220,707 | 429,004,425 | — | **≥ 18.2×** |

Same-session provari hand medians: core 500 ms / 342,289 nodes; easy-win
105 ms / 72,523 nodes; deep-win 6,590 ms / 4,574,912 nodes.

**Tier 1 result: 1/3, fail.** `core` and `deep-win` are search censors
(`exit=timeout`, no `proof_full`, no refutation). The 120 s budget is not
raised and the censored cells are not converted.

## D14

| Cell | seed | timed child_evals | proof child_evals | equal | positions | verdict |
| --- | ---: | ---: | ---: | --- | ---: | --- |
| easy-win | 0 | 3,321,312 | 3,321,312 | yes | 2,493 | accepted |
| easy-win | 1 | 2,993,310 | 2,993,310 | yes | 2,471 | accepted |
| easy-win | 2 | 3,147,781 | 3,147,781 | yes | 2,371 | accepted |

All three proofs pass `atomi validate` and `provari check`. No proof file is
committed; all temporary files were deleted by the driver.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | pass: 9 coverage + 9 provari + 3 D14 rows, parsed JSON only |
| G2 determinism | pass: no engine change; D14 equal `child_evals` on all proved rows |
| G3 goal | **fail**: Tier 1 is 1/3 at 120 s; exact censored cells recorded |
| G4 soundness | pass: no refutation; 3/3 D14 proofs accepted by both checkers; `make test` and `make lint` green |
| G5 docs | pass: no GPL dependency, raw transcript, provari artifact, proof or `__pycache__` |
