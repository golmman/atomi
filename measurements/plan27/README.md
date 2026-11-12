# Plan 27 measurements — NULL-V2 adoption (cleaned)

Plan: [`plan27.md`](../../docs/plans/prover/plan27.md). Report:
[`report27.md`](../../docs/plans/prover/report27.md). Host and binary:
[`env.json`](env.json). The tree is uncommitted on top of `atomi@7cc3494`
(binary sha256 `cc5bfe74…`); the owner commits.

## Files

| File | Contents |
| --- | --- |
| `probe.py` | 5 M-eval `deep-win` probe: default 2^17, disabled `0`, requested 2^21 |
| `probe.json` | Parsed probe: default reproduces the plan26 control (168,910 expansions); 2^21 changes the tree (168,885) |
| `bench.json` | `bench/run_bench.py` at Cap 120 s, seeds 0–2: `provari --guide hand` vs the cleaned default, `--proof-check` |
| `env.json` | Host, head, binary sha256, provari pin, gate and D14 verdicts |

## Tree identity (5 M evals, `deep-win`, seed 0)

| Config | Expansions | Hits / misses | Wall | Tree |
| --- | ---: | ---: | ---: | --- |
| plan26 control | 168,910 | — | — | reference |
| default 2^17 | **168,910** | 1,014,999 / 2,364,622 | 2.603 s | identical |
| `--null-cache-entries 0` | 168,910 | 0 / 0 | 2.865 s | identical |
| `--null-cache-entries 2097152` | 168,885 | 1,200,836 / 2,178,341 | 2.819 s | differs (TT compensation) |

## Bench row — Cap 120 s, seeds 0–2

| Cell | provari hand | atomi default | Factor | D14 |
| --- | ---: | ---: | ---: | --- |
| `core` | 0.520 s | 6.910 s | 13.288 | accepted |
| `easy-win` | 0.137 s | 0.533 s | 3.891 | accepted |
| `deep-win` | 4.413 s | 77.442 s | 17.549 | accepted |

Coverage 3/3, geomean **9.681**. `child_evals` to proof: 12,439,376 / 1,016,547 /
155,236,740, matching the plan26 gate rows exactly. Run with:

    python3 bench/run_bench.py \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc --provari-check \
        /tmp/atomi-oracle/provari/target/release/provari \
        --atomi target/release/atomi --atomi-stack "hand=--guide hand" \
        --cases core,1eb80b92f92c,db6a741c723f \
        --seconds 120 --memory 192 --repeats 3 --proof-check \
        --proof-dir /tmp/plan27-proofs --out measurements/plan27/bench.json
