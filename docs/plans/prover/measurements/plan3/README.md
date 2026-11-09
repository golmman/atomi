# plan3 measurements

Plan: [`../../plan3.md`](../../plan3.md). Report: [`../../report3.md`](../../report3.md).
Everything here is parsed output; no raw transcripts, provari artifacts or
`__pycache__` are committed.

## Files

| File | Content |
| --- | --- |
| `census.json` | provari `--guide hand` census over `core + sample20 + wins.tsv` (deduplicated by FEN), 30 s with a 60 s retry; one row per candidate |
| `guidance-v1.tsv` | the frozen 24-cell guidance set (`name fen source ref_nodes ref_wall_s selection_budget_s`), sha256 `2da2a65b960a65c6d4a722cc6b1b50b802541814f9443f66d077036a47ee8c7d` |
| `env.json` | host, toolchain, pins, binary/corpus/guidance hashes |
| `guidance.json` | D21 node gate (`bare`/`hand`, seeds 0–3, `--evals 20000000`) |
| `wall.json` | `core`/`easy-win` wall/coverage gate (10 s, 3 repeats) |
| `ablation.json` | both-direction ablations (evidence only; the gate failed): `core`/`easy-win`/`deep-win`, seed 0, 10 s / 10 M, 3 repeats |
| `bench.json` | the canonical 3-cell bench row (only when G1–G7 pass) |
| `run_guidance.py` | the stdlib-only driver |

## Commands

```sh
make oracle                                    # /tmp/atomi-oracle, outside the repo
cargo build --release

python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase census --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --candidates bench/corpus.json,tests/fixtures/wins.tsv \
  --seconds 30 --retry-seconds 60 --memory 192 --out .../census.json
python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase freeze --census .../census.json --freeze-out .../guidance-v1.tsv

python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase gate --set .../guidance-v1.tsv \
  --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --atomi target/release/atomi \
  --stack 'bare=--guide none' --stack 'hand=--guide hand' \
  --seeds 0,1,2,3 --seconds 600 --evals 20000000 --memory 192 \
  --out .../guidance.json

python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase wall --set .../guidance-v1.tsv \
  --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --atomi target/release/atomi --stack 'hand=--guide hand' \
  --seeds 0,1,2,3 --repeats 3 --seconds 10 --memory 192 \
  --cells core,easy-win --proof-check \
  --provari-check /tmp/atomi-oracle/provari/target/release/provari \
  --out .../wall.json
```

The census and gate reference runs are **selection** budgets (30/60 s), not the
bench budget. The gate's node comparison is `expansions`, never wall. The
wall phase is the 10 s product condition and re-runs the reference three
times per cell.

**Gate sharding.** The single-process gate did not finish in one sitting, so
the gate command above was run four times with `--cells` set to
`core,easy-win,deep-win,b9a7ed7eb368,3004d09cef79,dd3f89766a40`, the six
`dec01`–`dec08` cells, the six `dec09`–`dec14` cells and the six
`dec15`–`dec20` cells, writing `guidance-shard0..3.json` (scratch, not
committed). The deterministic `rows`/`summary`/`gate` blocks were merged into
`guidance.json`; `expansions` and `child_evals` are seed-deterministic, so
the merge is exact. Reference walls from the contended shard period are not
used for any wall claim; `wall.json` is the clean 10 s measurement.

**Ablation scope.** The committed `ablation.json` is a reduced run, not the
plan's conditional G7 matrix: the three bench cells (not all 24
`guidance-v1` cells), seed 0 (not 0–2), and 10 s / 10 M. `core` and
`easy-win` were run together; `deep-win` was run separately and merged into
the same file (the gate-sharding rule; exact because `expansions` are
deterministic). The full 24-cell × seeds 0–2 ablation remains unrun while
the D21 gate fails.

## Notes

- `run_guidance.py` is new code for this plan (not copied from the legacy
  repository). It only runs provari as an external process.
- The census began after development probes on the three bench cells; the
  candidate order and selection rule were fixed in the plan before the
  census, and no atomi result was used to pick a cell.
- A proof file under `/tmp/atomi-plan3-proofs` is scratch and is never
  committed.
