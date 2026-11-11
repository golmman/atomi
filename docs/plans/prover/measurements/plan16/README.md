# plan16 measurements

Plan: [`../../plan16.md`](../../plan16.md) (item 24 Phase A). Report:
[`../../report16.md`](../../report16.md). Parsed output only: no raw
transcript, provari artifact, proof file or `__pycache__` is committed.
Proof files are written under `/tmp/atomi-plan16/` and are never copied here.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary sha256, corpus sha256, provari version, cells, seeds, budgets, flags |
| `diff.json` | per cell: proof sizes, lengths, first set-divergent ply, first move-divergent ply, ply counts, symmetric difference and common positions with different chosen moves |
| `classify.json` | per cell: the move-divergent positions, `atomi classify` under bare/hand and each mechanism toggle, the five item-15 predicates, and a summary |
| `flags.json` | A3: atomi `--guide hand` plus each approximate flag on `core`/`easy-win`; exit, wall, positions, first divergent ply |
| `run_diff.py` | stdlib-only driver (`env`, `diff`, `classify`, `flags`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan16/run_diff.py --phase env
python3 docs/plans/prover/measurements/plan16/run_diff.py --phase diff
python3 docs/plans/prover/measurements/plan16/run_diff.py --phase classify
python3 docs/plans/prover/measurements/plan16/run_diff.py --phase flags
make test
make lint
make test-full
```

One atomi/provari process at a time. `diff` ran first, then `classify`, then
`flags`, then the gates.

## Budget

All runs use the plan's 120 s atomi budget. `deep-win` seed 1 proved at
119.049 s (51,002,209 expansions, length 25); the margin is thin, and two
earlier runs with an earlier build censored at 120 s. `env.json`
records `deep_win_seconds: 120`.

## Row counts and units

- `diff`: 3 cells; provari hand at 10 s, atomi bare at 120 s. `positions`
  is the number of attacker proof lines; `ply` is
  `fullmove - root fullmove`.
- `classify`: 5 `core` + 1 `easy-win` + 3 `deep-win` move-divergent
  positions, each under 7 variants (bare, hand, `--guide-off quiet`,
  `--guide-fresh-dn`, `--guide-off evasions`, `--guide-off efpn`,
  `--guide-replay-exact`).
- `flags`: 6 runs per cell (`hand`, `group`, `fresh_dn`, `replay_exact`,
  `ef_off`, `quiet_off`) on `core` and `easy-win`; `core` censors at 120 s
  on every run, `easy-win` proves on every run.
- A censored row is never converted. The first divergent ply is a tree
  property, not a wall claim.
