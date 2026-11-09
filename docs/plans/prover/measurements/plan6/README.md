# plan6 measurements

Plan: [`../../plan6.md`](../../plan6.md) (item 15, the `GROUP` kill-test).
Parsed output only: no raw transcripts, provari artifacts, proof files or
`__pycache__` are committed. Temporary proofs are written under `proofs/`,
checked, then deleted by the driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus/guidance sha256, oracle version |
| `group.json` | core + easy-win, seeds 0–2, flag off/on at 20 M child evals; seed-0 repeats; gate evaluation |
| `soundness.json` | guidance-v1 seed 0 with `--guide-group`: refutation sweep, 17 D14 proofs, both checkers |
| `scale.json` | plan7 scale-case re-check: core aux 120 s and easy-win aux D14 proofs |
| `run_group.py` | stdlib-only driver (`env`, `group`, `soundness`, `scale`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan6/run_group.py --phase env
python3 docs/plans/prover/measurements/plan6/run_group.py --phase group
python3 docs/plans/prover/measurements/plan6/run_group.py --phase soundness
python3 docs/plans/prover/measurements/plan6/run_group.py --phase scale
make test-full
make lint
```

One atomi process at a time. The group stack is
`--guide hand --guide-group`; the baseline is `--guide hand`. The group gate
budget is `--seconds 600 --evals 20000000 --memory 192`, seeds 0–2.

## G1/G2 — the two-cell kill-test

`expansions` (deterministic) at 20 M child evals:

| Cell | seed | off | on | on/off |
| --- | ---: | ---: | ---: | ---: |
| `core` | 0 | 648,828 | 681,518 | 1.05× |
| `core` | 1 | 653,551 | 678,413 | 1.04× |
| `core` | 2 | 647,032 | 678,624 | 1.05× |
| `easy-win` | 0 | 468,301 (proved) | 690,028 (proved) | 1.47× |
| `easy-win` | 1 | 631,763 (proved) | 724,295 (proved) | 1.15× |
| `easy-win` | 2 | 211,254 (proved) | 487,580 (proved) | 2.31× |

- **G1 intent fails.** `core` is still censored at 20 M evals and the group
  costs 1.04–1.05× the flag-off expansions: a **0.96× "cut"**, not the
  required ≥ 5×. The plan's literal `< 1,000,000` is technically met
  (681,518), but the same-session flag-off baseline is already 648,828, so
  that bound does not discriminate the group; the item-15 ≥ 5× clause does
  and fails.
- **G2 fails.** `easy-win` proves on all seeds but its expansions exceed the
  flag-off count on every seed (up to 2.31×). Lengths also change
  (18/18/17 off → 28/28/21 on).
- **G4 passes** on the seed-0 repeats: identical exit/length/child_evals/
  expansions for both cells in both configurations.

Per the plan stop rule, G1 (intent) and G2 both failing **closes the
hand-guide line**. No replay phase is implemented.

## Soundness (guidance-v1, seed 0, group on)

`--seconds 10 --evals 5000000 --memory 192`: 0 refutations in 24 cells. The
19 cells that proved at the 10 s plan5 budget include 17 here (the two larger
ones, `dec01` and `easy-win`, hit the 5 M eval cap); every one of the 17 D14
proofs is accepted by `atomi validate` **and** `provari check`, with equal
position counts. The group unit/property tests, the crafted plain-win
(`MATE_IN_2`) and all-plain-lose (`dec40`) integration tests are green.

## Scale-case re-check (plan7 phase-A dependency)

The group did not cut `core`, so the plan7 scale case is unchanged. Re-run on
the current tree with the plan5 auxiliary stack:

| Cell | budget | result | positions | entries | checkers |
| --- | --- | --- | ---: | ---: | --- |
| `core` | 120 s | proved, length 20 | 14,332 | 691,117 | accepted |
| `easy-win` | 600 s | proved, length 19 | 3,527 | 65,179 | accepted |

These reproduce report4's 14,332 / 691,117 `core` proof and plan5's 3,527 /
65,179 `easy-win` proof. Plan7 phase A therefore keeps `core` (14,332
positions, ≈691 k entries) as the conservative scale case and `easy-win` aux
(3,527 / 65,179) as the smaller one.

## Gates

| Gate | Result |
| --- | --- |
| G1 group activation | **fail** (0.96×, no 5× cut; censored at 20 M) |
| G2 easy-win no regression | **fail** (proved, but 1.15–2.31× the flag-off expansions) |
| G3 soundness | **pass on the executed sweep**: no refutation; all 17 proved proofs accepted by both checkers; property/crafted tests green |
| G4 determinism | **pass**: seed-0 repeats identical; flag-off counts reproduce the pre-change tree |
| G5 build/docs | **pass**: `make test-full` (incl. veto) and `make lint` green; no GPL-family dependency |

The kill verdict and the plan7 scale-case handoff are in
[`../../report6.md`](../../report6.md).
