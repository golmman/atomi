# plan8 measurements

Plan: [`../../plan8.md`](../../plan8.md) (item 17, comparable per-expansion
diagnosis and the wall-goal bracket). Report:
[`../../report8.md`](../../report8.md). Parsed output only: no raw transcripts,
provari artifacts, proof files or `__pycache__` are committed. Temporary proofs
are written under `proofs/`, checked, then deleted by the driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary, corpus/guidance sha256, provari version/commit, stacks, budgets |
| `pernode.json` | phase A: provari hand repeats; bare/neither/no-sim/hand on easy-win/core, seeds 0–2, fixed 10 M evals and canonical 10 s; `R` table and Decision A |
| `longbudget.json` | phase B: product at 60 s on core/deep-win/easy-win and bare on core; same-session provari references; D14 checks; Decision B |
| `run_plan8.py` | stdlib-only driver (`env`, `pernode`, `longbudget`, `provari-b`, `d14`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan8/run_plan8.py --phase env
python3 docs/plans/prover/measurements/plan8/run_plan8.py --phase pernode
python3 docs/plans/prover/measurements/plan8/run_plan8.py --phase longbudget
python3 docs/plans/prover/measurements/plan8/run_plan8.py --phase provari-b
python3 docs/plans/prover/measurements/plan8/run_plan8.py --phase d14
make test
make lint
```

One atomi/provari process at a time. `env` and `pernode` were run first, then
`longbudget`, then `provari-b` (the deep-win reference needed by phase B) and
`d14` (the bare-core proofs). `pernode.json` records the provari repeats used
for phase A, and
`longbudget.json` records the same-session `provari` block used for phase B.

## Row counts and units

- **Phase A:** 6 provari rows (easy-win/core × 3 repeats) + 48 atomi rows
  (4 stacks × 2 cells × 3 seeds × {fixed, canonical}).
- **Phase B:** 10 atomi rows (hand: core 3, deep-win 3, easy-win 1; bare:
  core 3) + 9 provari rows (3 cells × 3 repeats) + 4 D14 proof rows.
- `expansions` is atomi's main-search node expansion (the analogue of
  provari's `nodes`). `child_evals` is one per child classified during an
  expansion; `nodes` is provari's search node count. `ns/exp` is
  `ms·1e6 / expansions`; `children/exp` is `child_evals / expansions`;
  `aux` is `sim_nodes + settle_nodes`; the auxiliary share is
  `aux / (child_evals + aux)`.
- `R` is `(ns/exp atomi) / (ns/node provari)` from fixed-10 M runs, using the
  median over seeds 0–2 for atomi and the median of three same-session
  provari repeats. Canonical 10 s wall is reported separately and is not used
  for `R`.
- A censored 60 s row is never converted: its factor is a lower bound on the
  completed work. D14 proof runs use `--seconds 600 --proof-out` with the same
  flags and seed; the driver requires equal `child_evals` and both checkers.

## Phase A — per-expansion cost

Fixed 10 M evals, medians over seeds 0–2. Provari medians: easy-win
106 ms / 72,523 nodes / 1461.6 ns per node; core 499 ms / 342,289 nodes /
1457.8 ns per node.

| Stack | Cell | atomi ns/exp | exp/s | children/exp | R |
| --- | --- | ---: | ---: | ---: | ---: |
| bare | easy-win | 1732.8 | 577 k | 31.1 | 1.186 |
| bare | core | 1942.2 | 515 k | 32.3 | 1.332 |
| neither | easy-win | 4133.4 | 242 k | 32.0 | **2.828** |
| neither | core | 5020.6 | 199 k | 32.0 | **3.444** |
| no-sim | easy-win | 15,762.1 | 63 k | 27.3 | 10.784 |
| no-sim | core | 20,728.1 | 48 k | 32.9 | 14.219 |
| hand | easy-win | 49,892.7 | 20 k | 27.3 | 34.136 |
| hand | core | 66,400.0 | 15 k | 32.6 | 45.548 |

**Decision A:** `neither` is > 2 on both cells (2.828 / 3.444), so the core is
a first-order gap and item 18 would include core throughput. The bare tree is
competitive per expansion (R 1.19 / 1.33) but pays a larger tree.
Zero-guide floor: bare easy-win proves at 10 s, median 1.976 s (3/3),
18.6× the same-session provari wall.

Canonical 10 s medians (easy-win proved 3/3 for every stack; core censored
everywhere):

| Stack | easy-win wall · exp | core wall · exp |
| --- | ---: | ---: |
| bare | 1.976 s · 1.24 M | 10.000 s · 5.38 M |
| neither | 3.924 s · 0.99 M | 10.000 s · 2.13 M |
| no-sim | 2.074 s · 0.13 M | 10.012 s · 0.49 M |
| hand | 5.770 s · 0.12 M | 10.023 s · 0.15 M |

Fixed-10 M easy-win: bare 0.557 s (321 k exp, censored at the eval cap),
neither 1.292 s (313 k, censored), no-sim 2.075 s (132 k, proved), hand
5.745 s (115 k, proved). Fixed-10 M core censors at the eval cap on every
stack; hand takes 20.4 s for 307 k expansions.

## Phase B — 60 s coverage bracket

Product (`hand`, 60 s / 192 MB), medians over seeds 0–2 except easy-win
(seed 0):

| Cell | proved | wall | expansions | child_evals | aux | factor vs provari |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| core | 0/3 | 60.024 s | 0.94 M | 29.6 M | 204 M | ≥ 121.5× |
| deep-win | 0/3 | 60.007 s | 1.02 M | 31.4 M | 218 M | ≥ 9.36× |
| easy-win | 1/1 | 6.013 s | 121,866 | 3.32 M | 22.8 M | 58.4× |

Same-session provari hand: core 494 ms / 342,289 nodes; deep-win 6413 ms /
4,574,912 nodes; easy-win 103 ms / 72,523 nodes.

**Zero-guide floor (bare on core, 60 s):** proved 3/3 in 16.618 / 20.982 /
17.089 s (median 17.089 s), 9.27–11.25 M expansions, length 14 / 14 / 24.
Median factor 34.6× the same-session provari wall. All three D14 proofs are
accepted by `atomi validate` and `provari check` with equal `child_evals`
(proof positions 10,831 / 11,107 / 11,879). This is a coverage observation for
the guide-free tree, not the product configuration and not a converted censor.

**Decision B:** the product censors `core` and `deep-win` at 60 s and its
easy-win factor is 58.4×, so the rule recommends **re-pin (item 19)**.

## D14

| Stack | Cell | seed | timed evals | proof evals | equal | positions | verdict |
| --- | --- | ---: | ---: | ---: | --- | ---: | --- |
| hand | easy-win | 0 | 3,321,312 | 3,321,312 | yes | 2,493 | accepted |
| bare | core | 0 | 268,226,989 | 268,226,989 | yes | 10,831 | accepted |
| bare | core | 1 | 326,310,697 | 326,310,697 | yes | 11,107 | accepted |
| bare | core | 2 | 267,472,285 | 267,472,285 | yes | 11,879 | accepted |

No proof file is committed; all temporary files were deleted by the driver.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | pass: matrix complete, row counts above, parsed JSON only |
| G2 comparability | pass: same host/session, three provari repeats, `nodes`/`expansions`/`child_evals` defined above |
| G3 decision | pass: `R` and the 60 s bracket recorded; pre-registered Decision A/B; no cell dropped |
| G4 soundness | pass: no refutation; 4/4 D14 proofs accepted by both checkers; `make test` green |
| G5 docs | pass: `make lint` green; no GPL dependency, raw transcript or artifact |
