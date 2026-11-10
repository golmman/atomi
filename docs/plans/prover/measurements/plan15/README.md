# plan15 measurements

Plan: [`../../plan15.md`](../../plan15.md) (item 5 closure + item 6 M5
head-to-head under D28). Report: [`../../report15.md`](../../report15.md).
Parsed output only: no raw transcripts, provari artifacts, proof files or
`__pycache__` are committed. Temporary proofs are written under `proofs/`,
checked, then deleted by the driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary, corpus/guidance sha256, provari version/commit, control availability, flags, budgets |
| `tier1.json` | Phase A: `--guide none` at 120 s on core/easy-win/deep-win, seeds 0–2; coverage; D14 rows |
| `tier2.json` | Phase B: `--guide none` at 10 s, 3 repeats per cell; same-session provari hand references; factors, bounds, geomean; D14 rows |
| `run_plan15.py` | stdlib-only driver (`env`, `tier1`, `tier1_d14`, `tier2`, `tier2_d14`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan15/run_plan15.py --phase env
python3 docs/plans/prover/measurements/plan15/run_plan15.py --phase tier1
python3 docs/plans/prover/measurements/plan15/run_plan15.py --phase tier1_d14
python3 docs/plans/prover/measurements/plan15/run_plan15.py --phase tier2
python3 docs/plans/prover/measurements/plan15/run_plan15.py --phase tier2_d14
make test
make lint
make test-full
```

One atomi/provari process at a time. `tier1` ran first, then its D14 block,
then the same-session `tier2` block (atomi then provari), then its D14 block,
then the gates.

## Row counts and units

- **Tier 1:** 9 atomi rows (3 cells × seeds 0–2) + 7 D14 proof rows (core 3,
  easy-win 3, deep-win 1).
- **Tier 2:** 9 atomi rows (3 cells × 3 repeats, seed 0) + 9 provari rows
  (3 cells × 3 repeats) + 3 D14 proof rows (easy-win repeats 0–2).
- `expansions` is atomi's main-search node expansion (the analogue of
  provari's `nodes`). `child_evals` is one per child classified during an
  expansion; `nodes` is provari's search node count. `aux` is
  `sim_nodes + settle_nodes` (zero for the bare tree).
- A censored row is never converted: its factor is a lower bound on the
  completed work. D14 proof runs use `--seconds 600 --proof-out` with the
  same flags and seed; the driver requires equal `child_evals` and both
  checkers.

## Phase A — Tier 1 confirmation at 120 s

`--guide none`, `--seconds 120 --memory 192`, seeds 0–2.

| Cell | proved | wall (median) | expansions (median) | child_evals (median) | length | factor vs provari |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| core | **3/3** | 17.471 s | 9,266,026 | 268,226,989 | 14 / 14 / 24 | 35.8× |
| easy-win | **3/3** | 2.023 s | 1,243,369 | 34,289,765 | 17 / 17 / 19 | 18.9× |
| deep-win | **1/3** | 120.001 s (censor) | 52,751,360 | 1,453,493,923 | 25 (seed 1) | **≥ 18.6×** |

Per-seed walls: core 17.041 / 21.546 / 17.471 s; easy-win 1.703 / 2.152 /
2.023 s; deep-win censor / **113.458 s (proved)** / censor. The counters are
identical to plan11's bare-tree run; the factors use the same-session
provari hand medians from Phase B (core 488 ms, easy-win 107 ms, deep-win
6,455 ms).

**Tier 1 result: 2/3 cells, pass under D28.** `deep-win` proves on seed 1 at
113.458 s but censors on seeds 0 and 2 at 120 s. The 120 s budget is not
raised and the censored seeds are not converted.

## D14 — Tier 1

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

## Phase B — Tier 2 head-to-head at 10 s

`--guide none`, `--seconds 10 --memory 192`, seed 0, 3 repeats. Same-session
provari: `prove ... --seconds 10 --memory 192 --guide hand`, 3 repeats.

| Cell | atomi proved | atomi wall (median) | provari wall (median) | factor | coverage |
| --- | ---: | ---: | ---: | ---: | --- |
| core | 0/3 | 10.000 s (censor) | 0.488 s / 342,289 nodes | **≥ 20.49** | censored |
| easy-win | **3/3** | 1.686 s | 0.107 s / 72,523 nodes | **15.757** | shared |
| deep-win | 0/3 | 10.000 s (censor) | 6.455 s / 4,574,912 nodes | **≥ 1.549** | censored |

**Tier 2 result: fail.** Coverage is **1/3** (only `easy-win` is shared), so
the "coverage stays 3/3" clause cannot be met; the shared-cell geomean is
**15.757**, far above `< 1.0`. Censored cells are lower bounds, not factors.

## D14 — Tier 2

| Cell | repeat | timed child_evals | proof child_evals | equal | positions | entries | bytes | verdict |
| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | --- |
| easy-win | 0 | 30,335,618 | 30,335,618 | yes | 2,048 | 69,772 | 172,521 | accepted |
| easy-win | 1 | 30,335,618 | 30,335,618 | yes | 2,048 | 69,772 | 172,521 | accepted |
| easy-win | 2 | 30,335,618 | 30,335,618 | yes | 2,048 | 69,772 | 172,521 | accepted |

All three proofs pass `atomi validate` and `provari check`.

## Controls

| Control | Status |
| --- | --- |
| legacy `default` / `opt` | **unavailable** (`/tmp/atomic_solver-5f85133/...` absent) |
| provari v0.2.0 hand stretch | **unavailable** (not built) |

No substitute was used.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | pass: 9 Tier 1 + 7 Tier 1 D14 + 9 Tier 2 atomi + 9 provari + 3 Tier 2 D14 rows, parsed JSON only |
| G2 Tier 1 | **pass**: 2/3 at 120 s, 7/7 D14 accepted, no fixture regression |
| G3 Tier 2 | **fail (recorded)**: coverage 1/3, shared-cell geomean 15.757; no conversion |
| G4 soundness | pass: no refutation; 10/10 D14 proofs accepted by both checkers; `make test`, `make lint`, `make test-full` and `make veto` green |
| G5 docs | pass: no GPL dependency, raw transcript, provari artifact, proof or `__pycache__` |
