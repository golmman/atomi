# plan18 measurements

Plan: [`../../plan18.md`](../../plan18.md) (item 25, D14/D18/D21/D24/D26/D30/D31).
Report: [`../../report18.md`](../../report18.md). Parsed output only: no raw
transcript, provari artifact, proof file or `__pycache__` is committed.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus sha256, provari version, stacks, budgets |
| `diff.json` | Phase A read-only mechanism differential (provari 0.1.0 vs atomi) and the G2 gaps |
| `probe.json` | one parsed 20 M combined-stack run on `deep-win` (the numeric applicability probe) |
| `matrix.json` | Phase C: 15 seed-0 rows (5 stacks × 3 cells) + D14 rows + the G3 summary |
| `run_phase18.py` | stdlib-only driver (`env`, `diff`, `probe`, `matrix`, `d14`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan18/run_phase18.py --phase all
make test
make lint
make test-full
```

One atomi process at a time. `--seconds 600` only guards the wall;
`--evals 20000000` is the budget, as in plan13/plan17 phase C.

## Phase A differential (G2 passes)

The pinned provari clone is read only as a source of ideas. The differential
records the mechanism shape, not source text. Four portable gaps on the same
DF-PN base:

| Gap | provari | atomi before plan18 |
| --- | --- | --- |
| G2-a exact null-threat replay | build the passed position, find a win, replay it on every open defence, lower a fresh parrying defence | signature-only `UNLIKELY` prior; no null proof |
| G2-b exposing one-move shortcut | a defence after which the attacker explodes at once is a one-move proof | replay needed a stored proof or the guide's reply |
| G2-c guide explosion/dropped-mate fallback | the guide's move may be an explosion or a dropped mate | a stored guide proof was required |
| G2-d top-level guide proof override | the null proof is stored like any other | the null position must not be keyed, so the guide move is passed explicitly |

Not ported: provari's check-every-evasion and double-threat null-win cases
(beyond the exposed mate-in-two), and `recomputed` proofs (atomi has no
recomputed proof table).

## Phase C 20 M matrix (G3 fails)

Seed 0, 192 MB, 20 M child evals. Unique expansions are the metric because
`core` and `deep-win` are censored in most stacks.

| Stack | Cell | Proved | Exit | Unique | Expansions | `child_evals` | `sim_nodes` | `aux_nodes` |
| --- | --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| bare | `core` | no | evals | 586,075 | 630,056 | 20,000,008 | 0 | 0 |
| bare | `easy-win` | no | evals | 622,562 | 669,920 | 20,000,034 | 0 | 0 |
| bare | `deep-win` | no | evals | 620,162 | 668,608 | 20,000,012 | 0 | 0 |
| hand | `core` | no | evals | 573,195 | 648,828 | 20,000,002 | 101,745,882 | 137,190,582 |
| hand | `easy-win` | **yes** | proved | 392,006 | 468,301 | 11,842,338 | 60,005,079 | 81,878,370 |
| hand | `deep-win` | no | evals | 584,085 | 663,127 | 20,000,003 | 101,708,746 | 139,376,171 |
| sim_exact | `core` | no | evals | 555,774 | 633,830 | 20,000,005 | 8,089 | 35,578,717 |
| sim_exact | `easy-win` | **yes** | proved | 264,533 | 319,301 | 8,493,083 | 4,645 | 15,908,827 |
| sim_exact | `deep-win` | no | evals | 582,026 | 665,421 | 20,000,025 | 14,643 | 37,841,894 |
| null_exact | `core` | **yes** | proved | 538,167 | 671,966 | 18,308,431 | 93,545,603 | 133,441,086 |
| null_exact | `easy-win` | **yes** | proved | 19,031 | 23,143 | 662,263 | 3,030,702 | 4,460,889 |
| null_exact | `deep-win` | no | evals | 613,763 | 738,458 | 20,000,005 | 103,064,086 | 145,891,071 |
| sim_null | `core` | no | evals | 587,286 | 741,236 | 20,000,031 | 1,829,732 | 45,972,826 |
| sim_null | `easy-win` | **yes** | proved | 34,175 | 39,592 | 986,079 | 25,717 | 2,079,809 |
| sim_null | `deep-win` | no | evals | 614,821 | 726,264 | 20,000,014 | 1,091,904 | 43,739,368 |

Ratios of unique expansions vs bare:

| Stack | `core` | `easy-win` | `deep-win` |
| --- | ---: | ---: | ---: |
| hand | 0.978 | 0.630 | 0.942 |
| sim_exact | 0.948 | 0.425 | 0.939 |
| null_exact | 0.918 | 0.031 | 0.990 |
| sim_null | 1.002 | 0.055 | 0.991 |

The plan gate is `sim_null` (the declared stack):
`deep_win_cut_sim_null = 1.009` (2× required), `core` 1.002 and
`easy-win` 0.055 (no regression). **G3 fails on the `deep-win` cut.** Both
individual flags cut `easy-win` strongly and `null_exact` proves `core` at
18.3 M where bare/hand are censored, but neither flag moves `deep-win` and
the combined stack regresses `core`.

## D14

Every proved seed-0 row re-ran untimed with `--seconds 600
--proof-out` under `/tmp/atomi-plan18`; all five (hand/easy-win,
sim_exact/easy-win, null_exact/core, null_exact/easy-win, sim_null/easy-win)
matched their timed `child_evals` and were accepted by `atomi validate` and
`provari check` (`5/5 accepted`). Proof files were deleted; none is
committed.

## G3 verdict

`false`. The exactness stack is closed negative for the `deep-win` gate; no
120 s matrix and no ledger row. The `null_exact` `core`/`easy-win` cut is
recorded as a separate observation, not as a `T_3` claim.
