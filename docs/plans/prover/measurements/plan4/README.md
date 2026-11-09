# plan4 measurements

Plan: [`../../plan4.md`](../../plan4.md). Report: [`../../report4.md`](../../report4.md).
Everything here is parsed output; no raw transcripts, provari artifacts or
`__pycache__` are committed.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, pins, binary/corpus/guidance hashes |
| `probes.json` | plan4 read-only probes on `core`/`easy-win`, bare/hand, 50 M evals |
| `sweep.json` | calibration sweep `scale × ef_weight`, `core` 50 M / `easy-win` 10 M, seed 0 |
| `gate.json` | canonical `core`/`easy-win` D21 re-run at 10 s, seeds 0–2 (reduced set) |
| `wall.json` | 10 s wall/coverage records for the same cells (from `gate.json` rows) |
| `pressure.json` | proof-table pressure / auxiliary-budget diagnostic on `core` |
| `run_sweep.py` | stdlib-only probe/sweep driver |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan4/run_sweep.py \
  --phase probe --out docs/plans/prover/measurements/plan4/probes.json
python3 docs/plans/prover/measurements/plan4/run_sweep.py \
  --phase sweep --out docs/plans/prover/measurements/plan4/sweep.json
```

The sweep uses one process at a time. `expansions` are deterministic; walls
are single-session records and are never substituted for the node gate.

## Notes

- The sweep scores `core` at 50 M child evals and guards `easy-win` at 10 M.
  A guard cell that hits the cap is recorded with its censored expansion
  count, not treated as a win.
- The D21 re-run is reduced to the two canonical gate cells at 10 s, seeds
  0–2, because the full 24-cell × 3-seed matrix did not fit this sitting.
  The full-set result for the plan3 tree is in
  [`../plan3/guidance.json`](../plan3/guidance.json).
- No provari-written proof and no oracle artifact is stored here. The
  pinned oracle is only invoked as an external process by the plan3 driver
  and by `make veto`.
