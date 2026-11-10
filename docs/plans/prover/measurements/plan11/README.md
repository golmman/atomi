# plan11 measurements

Plan: [`../../plan11.md`](../../plan11.md) (item 20, bare-tree coverage probe
and product-configuration decision). Report: [`../../report11.md`](../../report11.md).
Parsed output only: no raw transcripts, provari artifacts, proof files or
`__pycache__` are committed. Temporary proofs are written under `proofs/`,
checked, then deleted by the driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary, corpus/guidance sha256, provari version/commit, bare flags, budgets |
| `coverage.json` | Phase A: `--guide none` at 120 s on core/easy-win/deep-win, seeds 0–2; same-session provari hand references; factors; D14 rows |
| `guidance.json` | diagnostic: `--guide none` at 120 s, seed 0, on the frozen 24-cell `guidance-v1` set |
| `run_plan11.py` | stdlib-only driver (`env`, `coverage`, `provari`, `d14`, `guidance`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan11/run_plan11.py --phase env
python3 docs/plans/prover/measurements/plan11/run_plan11.py --phase coverage
python3 docs/plans/prover/measurements/plan11/run_plan11.py --phase provari
python3 docs/plans/prover/measurements/plan11/run_plan11.py --phase d14
python3 docs/plans/prover/measurements/plan11/run_plan11.py --phase guidance
make test
make lint
make test-full
```

One atomi/provari process at a time. `coverage` ran first, then the
same-session `provari` block, then `d14` for every proved coverage row, then
the `guidance` diagnostic, then the gates.

## Row counts and units

- **Phase A:** 9 atomi coverage rows (3 cells × seeds 0–2) + 9 provari rows
  (3 cells × 3 repeats) + 7 D14 proof rows (core 3, easy-win 3, deep-win 1).
- **Diagnostic:** 24 bare rows (guidance-v1 × seed 0).
- `expansions` is atomi's main-search node expansion (the analogue of
  provari's `nodes`). `child_evals` is one per child classified during an
  expansion; `nodes` is provari's search node count. `aux` is
  `sim_nodes + settle_nodes` (zero for the bare tree).
- A censored row is never converted: its factor is a lower bound on the
  completed work. D14 proof runs use `--seconds 600 --proof-out` with the
  same flags and seed; the driver requires equal `child_evals` and both
  checkers.

## Phase A — bare-tree coverage at 120 s

`--guide none`, `--seconds 120 --memory 192`, seeds 0–2.

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
but censors on seeds 0 and 2 at 120 s. The 120 s budget is not raised and the
censored seeds are not converted.

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

All seven proofs pass `atomi validate` and `provari check`. No proof file is
committed; all temporary files were deleted by the driver.

## Diagnostic — guidance-v1 seed 0

`--guide none`, `--seconds 120 --memory 192`, seed 0, 24 cells.

**20/24 proved.** Censored: `deep-win`, `b9a7ed7eb368`, `3004d09cef79`,
`dd3f89766a40`. Plan3's hand stack was 19/24 at 10 s (bare 18/24); this is a
diagnostic, not a gate, and the budgets differ.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | pass: 9 coverage + 9 provari + 7 D14 + 24 diagnostic rows, parsed JSON only |
| G2 determinism | pass: no engine change; D14 equal `child_evals` on all proved rows |
| G3 goal | **fail**: Tier 1 is 2/3 cells at 120 s; `deep-win` 1/3 seeds, exact censor recorded |
| G4 soundness | pass: no refutation; 7/7 D14 proofs accepted by both checkers; `make test`, `make lint`, `make test-full` and `make veto` green |
| G5 docs | pass: no GPL dependency, raw transcript, provari artifact, proof or `__pycache__` |
