# plan17 measurements

Plan: [`../../plan17.md`](../../plan17.md) (item 24 Phase B, D14/D18/D21/D24/D26/D29/D30).
Report: [`../../report17.md`](../../report17.md). Parsed output only: no raw
transcript, provari artifact, proof file or `__pycache__` is committed.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus/classify sha256, provari version, stacks, budgets |
| `applicability.json` | B1: per first move-divergent position, node kind and exact-replay applicability; G2 summary |
| `isolation.json` | B2: 9 seed-0 rows (3 stacks × 3 cells) + 3 seed-1 rows for the best guided stack; D14 rows; G3 summary |
| `run_phase_b.py` | stdlib-only driver (`env`, `applicability`, `isolation`, `d14`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan17/run_phase_b.py --phase env
python3 docs/plans/prover/measurements/plan17/run_phase_b.py --phase applicability
python3 docs/plans/prover/measurements/plan17/run_phase_b.py --phase isolation
python3 docs/plans/prover/measurements/plan17/run_phase_b.py --phase d14
make test
make lint
make test-full
```

One atomi process at a time. `--seconds 600` only guards the wall;
`--evals 20000000` is the budget, as in plan13 phase C.

## B1 applicability

The source is the committed plan16 classify data (`classify.json`); the
driver runs no search and writes no table. A move-divergent position is a
candidate only if it is a defender (AND) node, the proved child is in the
next ply, and a sibling is still `Open`.

| Cell | Positions | Move sides | Defender nodes | Child in next ply | Applicable |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 5 | 10 | 0 | 0 | 0 |
| `easy-win` | 1 | 2 | 0 | 0 | 0 |
| `deep-win` | 3 | 6 | 0 | 0 | 0 |

G2 fails: every first move-divergent node is an attacker (OR) node, where
`src/search/mod.rs` never starts a sibling replay. B3 is not implemented.

## B2 isolation matrix (seed 0, 20 M evals)

| Stack | Cell | Proved | Exit | Unique | Expansions | `child_evals` | `sim_nodes` | `aux_nodes` |
| --- | --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| bare | `core` | no | evals | 586,075 | 630,056 | 20,000,008 | 0 | 0 |
| bare | `easy-win` | no | evals | 622,562 | 669,920 | 20,000,034 | 0 | 0 |
| bare | `deep-win` | no | evals | 620,162 | 668,608 | 20,000,012 | 0 | 0 |
| hand | `core` | no | evals | 573,195 | 648,828 | 20,000,002 | 101,745,882 | 137,190,582 |
| hand | `easy-win` | **yes** | proved | 392,006 | 468,301 | 11,842,338 | 60,005,079 | 81,878,370 |
| hand | `deep-win` | no | evals | 584,085 | 663,127 | 20,000,003 | 101,708,746 | 139,376,171 |
| replay_exact | `core` | no | evals | 574,769 | 650,662 | 20,000,011 | 101,577,420 | 137,146,211 |
| replay_exact | `easy-win` | **yes** | proved | 396,419 | 470,770 | 11,885,350 | 60,315,524 | 82,271,449 |
| replay_exact | `deep-win` | no | evals | 586,294 | 665,165 | 20,000,015 | 101,654,541 | 139,340,701 |

Seed 1 for the best guided stack (`hand`, lowest `core` unique at seed 0):

| Stack | Cell | Proved | Exit | Unique | Expansions | `child_evals` |
| --- | --- | ---: | --- | ---: | ---: | ---: |
| hand | `core` | no | evals | 578,229 | 653,551 | 20,000,023 |
| hand | `easy-win` | **yes** | proved | 532,626 | 631,763 | 15,732,200 |
| hand | `deep-win` | no | evals | 584,295 | 662,502 | 20,000,020 |

Ratios vs bare: `hand` 0.978 / 0.630 / 0.942; `replay_exact` 0.981 / 0.637 /
0.945 on `core` / `easy-win` / `deep-win`. **Replay exact cuts `core` only
1.02×**, short of the 2× G3 threshold, and is slightly *larger* than `hand`
on all three cells (574,769 vs 573,195 on `core`; 396,419 vs 392,006 on
`easy-win`). It proves `easy-win` with 1,319 more unique expansions than
`hand`. **G3 fails**; no `easy-win` regression.

## D14

The three proved rows (`hand` `easy-win` seed 0, `replay_exact` `easy-win`
seed 0, `hand` `easy-win` seed 1) re-ran untimed with `--seconds 600
--proof-out` and the same `--evals 20000000` cap. Each matched its timed
`child_evals`; `atomi validate` and `provari check` accepted all three.
