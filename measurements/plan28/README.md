# Plan 28 measurements — HOTPATH-V1 (G-H1 miss)

Plan: [`plan28.md`](../../docs/plans/prover/plan28.md). Report:
[`report28.md`](../../docs/plans/prover/report28.md). Host and binaries:
[`env.json`](env.json). The repo is clean at `atomi@4ac58a9`; the owner
commits. The candidate is a **measurement-only** build: a local
`atomic-movegen` checkout with MG1 (`Board::null_for`) and MG2 (public
`for_each_pseudo_legal`) reached through a path dependency. Nothing from that
checkout is committed.

## Files

| File | Contents |
| --- | --- |
| `probe.py` | 5 M-eval `deep-win` probe: control, S3-only, S1+S2+S3 candidate, three repeats, one process at a time |
| `gate_h1.json` | Parsed G-H1 result: tree identity, medians, ratios, verdict |
| `phase_a.json` | 4 s `perf` self-time profile (control and candidate) and the 5 M `--stats` counters |
| `env.json` | Host, head, binary sha256, dependency, provari pin, gate status |

## G-H1 (5 M child evals, `deep-win`, seed 0)

| Config | `child_evals` | Expansions | Median wall | Ratio | Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| control (published 2.4.0) | 5,000,034 | 168,910 | 2.558 s | 1.0000 | 1.000× |
| S3 only | 5,000,034 | 168,910 | 2.560 s | 1.0008 | 0.999× |
| S1+S2+S3 candidate | 5,000,034 | 168,910 | 2.099 s | **0.8206** | **1.219×** |

The tree is identical in every run (`child_evals` and `expansions` match the
control exactly). The gate required ≤ 0.667 (≥ 1.5×); the candidate missed at
0.8206. Per the plan, the per-node line is closed and no 120 s matrix, D14
claim or bench row follows.

## Attribution

The 4 s profile moved the cost as expected: `generate_pseudo_legal` (20.7 %
self) disappears as a separate symbol and `from_fen`/`fen_with`/`malloc` drop
out, while `immediate_explosion_bits` (14.0 %), `ProofTable::get` (13.3 %),
`Board::legal` (9.9 %), `do_move` (6.5 %) and `compute_checkers` (6.1 %) remain.
The safe stack removes the FEN round-trip and the materialised guide move
lists, but not the per-child board/legality/terminality work that S4 would
have addressed. S4 was not started because G-H1 missed.

## Reproduce

    python3 measurements/plan28/probe.py \
        --control /tmp/atomi-plan28-control \
        --s3 /tmp/atomi-plan28-s3 \
        --candidate /tmp/atomi-plan28-candidate \
        --repeats 3 --out measurements/plan28/gate_h1.json

The candidate binary was built from a copy of the repo with
`atomic-movegen = { path = "/tmp/amg/atomic-movegen-dev" }`; the control and
S3 binaries use the published `atomic-movegen 2.4.0`.
