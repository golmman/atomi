# Plan 38 measurements — kill-review length-cap probe

Plan: [`plan38.md`](../../docs/plans/prover/plan38.md). Read-only probe for the
plans 36–37 kill review; no engine, flag, default, test or `bench.md` row
changed.

| File | Contents |
| --- | --- |
| `review_probe.json` | four `deep-win` runs: `--evals 5M` control, `--max-depth 24/20` at 5M, and `--max-depth 24` at 120 s |

## Reproduce

    cargo build --release
    atomi prove "<deep-win FEN>" --guide hand --memory 192 --seed 0 --evals 5000000
    atomi prove "<deep-win FEN>" --guide hand --memory 192 --seed 0 --evals 5000000 --max-depth 24
    atomi prove "<deep-win FEN>" --guide hand --memory 192 --seed 0 --seconds 120 --max-depth 24

The finding is that `--max-depth` is a censor (exits `depth` at the first hit),
not a bounded-length pass; it cannot price analysis1 R25.
