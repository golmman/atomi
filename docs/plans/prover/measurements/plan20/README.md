# plan20 measurements

Plan: [`../../plan20.md`](../../plan20.md) (items 27–28, decisions D12, D14,
D18, D21, D24, D26, D30, D32, D33). Report: [`../../report20.md`](../../report20.md).
Parsed output only: no raw transcript, provari artifact, proof file or
`__pycache__` is committed.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus sha256, provari version, cells, the three arms, budgets |
| `probe.json` | Phase A (G2): default vs item-27 exact replays, three cells, seed 0, auxiliary passes on/off, 2 G child evals / 600 s |
| `gate.json` | Phase C (G3): default, item 27 alone, AUX-V1, three cells × seeds 0–2, at a proof-reaching tree budget and at 300 M child evals / 240 s |
| `matrix.json` | Phase D: the 120 s budget matrix, default vs item 27 vs AUX-V1 |
| `tier2.json` | the 10 s tier: the three arms plus the pinned provari hand (3 repeats per cell) |
| `d14.json` | D14 for every proved matrix row: untimed `--seconds 600 --proof-out` rerun, `child_evals` match, `atomi validate` + `provari check` |
| `run_phase20.py` | stdlib-only driver (`env`, `a`, `gate`, `matrix`, `tier2`, `d14`; `--only`/`--cells` re-run a subset) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan20/run_phase20.py --phase env
python3 docs/plans/prover/measurements/plan20/run_phase20.py --phase a
python3 docs/plans/prover/measurements/plan20/run_phase20.py --phase gate
python3 docs/plans/prover/measurements/plan20/run_phase20.py --phase matrix
python3 docs/plans/prover/measurements/plan20/run_phase20.py --phase tier2
python3 docs/plans/prover/measurements/plan20/run_phase20.py --phase d14
make test
make lint
make test-full
```

One atomi process at a time. `--memory 192` everywhere. Proof files live under
`/tmp/atomi-plan20` and are deleted after validation.

## The arms on the shipped binary

`--guide hand` (the Phase-D default) is the plan-19 default plus
`sim-exact`, `null-exact` and `closure-bits`. The explicit spellings make the
three arms reproducible on the shipped binary and on the pre-change binary the
gate/matrix/tier2 rows were measured with:

| Arm | Shipped spelling | Meaning |
| --- | --- | --- |
| default | `--guide hand --guide-off sim-exact,null-exact,closure-bits` | plan-19 adopted R1+R3+R6 |
| aux_v1_nobits | `--guide hand --guide-off closure-bits` | + item 27 (exact replays) |
| aux_v1 | `--guide hand` | + item 28 (board-free closures) |

`probe.json` was taken with the pre-change binary: its `aux_v1` row is item 27
alone (now `aux_v1_nobits`). Of the auxiliary-off rows, the plan's spelling
`--guide-off sim,closures,null-exact` is kept exactly.

## Why the auxiliary gate is reported as a miss

`aux_nodes` counts `settle_nodes + sim_nodes`, including the null position's
movegen and both exact replays. The 20 M instrument is retired (report19
D33); the tree gate is child evals to proof at a generous cap. The
`aux_nodes`/expansion ≤ 1 clause of G3 is reported as a miss because the
exact replays are per-defence work, not per-expansion work (see report20
§G3); the stack still cuts it 16–28× and cuts the tree 1.7–5.9×.
