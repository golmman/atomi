# Plan 1 measurements — `atomic-movegen` 2.4.0 (`m0`) vs 2.6.0 (`m1`)

Plan: [`plan1.md`](../../docs/plans/movegen/plan1.md). Report:
[`report1.md`](../../docs/plans/movegen/report1.md). Host, binaries and gate
verdicts: [`env.json`](env.json). The worktree is uncommitted on top of
`atomi@9617f9b`; the owner commits.

Both sides are built from the **same source tree** — only `Cargo.toml` and
`Cargo.lock` differ — so every difference below is the crate's:

| Side | Dependency | Binary |
| --- | --- | --- |
| `m0` (control) | published 2.4.0 | `/tmp/atomi-m0-plan1/target/release/atomi`, sha256 `240a894d…` |
| `m1` (candidate) | published 2.6.0 | `target/release/atomi`, sha256 `db8a7f4f…` |

## Files

| File | Contents |
| --- | --- |
| `bench.py` | Wall bench: the three bench cells at Cap 10 s and 120 s, `provari --guide hand` (10 s) + `m0` + `m1` per repeat, alternating, one process at a time; command building, parsing and ledger arithmetic imported from `bench/run_bench.py` |
| `bench.json` | Parsed runs (`runs`), per cell/cap medians and factors (`ledger`), geomeans (`summary`) |
| `tree_probe.py` | 5 M child evals on `deep-win`, seed 0, `--guide hand`: tree identity and the same-tree wall ratio |
| `tree_probe.json` | Parsed probe |
| `pernode.json` | `bench/run_pernode.py` with **both** sides (S4: per-adopted-version per-expansion cost) |
| `pernode_make.json` | `make pernode` (the `m1` side, the target's default) |
| `env.json` | Host, head, pins, checksums, binary sha256s, gate verdicts |

## Wall bench (bare tree, `--guide none`)

The driver passes no `--guide`, so the measured stack is the **bare** tree —
the product candidate (D28), the same stack as the plan11/plan15 rows. This
is a *different stack* from the plan23/25/27 rows, which pass
`--atomi-stack 'hand=--guide hand'`; the two are never compared (reading
rule 4).

| Cap | Cell | provari hand | `m0` 2.4.0 | `m1` 2.6.0 | factor `m0` | factor `m1` | `m1/m0` | Tree |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 10 s | `core` | 0.523 s | censored | censored | ≥ 19.12 | ≥ 19.12 | — | — |
| 10 s | `easy-win` | 0.137 s | 1.934 s | 1.895 s | **14.117** | **13.832** | 0.980 | identical |
| 10 s | `deep-win` | 6.505 s | censored | censored | ≥ 1.54 | ≥ 1.54 | — | — |
| 120 s | `core` | 0.524 s | 18.372 s | 18.394 s | **35.061** | **35.103** | 1.001 | identical |
| 120 s | `easy-win` | 0.134 s | 1.894 s | 1.930 s | **14.134** | **14.403** | 1.019 | identical |
| 120 s | `deep-win` | 6.582 s | censored | censored | ≥ 18.23 | ≥ 18.23 | — | — |

Coverage `m1`: 1/3 at Cap 10 s (geomean 13.832), 2/3 at Cap 120 s (geomean
22.485). Paired cells (`easy-win`@10 s, `core`@120 s, `easy-win`@120 s):
`m1/m0` geomean **0.9999**, tree identical 3/3 (`child_evals` and
`expansions` equal on every paired cell). The censored cells match the
plan11/plan15 bare-tree rows (`core` 17.4 s at 120 s proves; `deep-win`
1/3 — seed 0 censors), so the 2.6.0 pin did not move coverage.

`deep-win` carries no factor at either cap on either side, so the deepest
cell contributes nothing to the paired ratio.

## Tree probe (5 M child evals, `deep-win`, `--guide hand`, seed 0)

| Side | Expansions | `child_evals` | Median wall | Median `ms` |
| --- | ---: | ---: | ---: | ---: |
| `m0` | 168,910 | 5,000,034 | 2.575 s | 2,571 |
| `m1` | 168,910 | 5,000,034 | 2.615 s | 2,610 |

Tree **identical to the plan28 control** (168,910 expansions) on both sides;
`m1/m0` = **1.0152** (1.5 %, inside the ±2–3 % repeat spread of the same
session: `m0` runs are 2,566/2,571/2,624 ms, `m1` 2,589/2,610/2,612 ms —
overlapping, so this is noise, not a slowdown). The plan's §3 pre-probe saw
1.002× on the same cell.

## Per-expansion cost (S4)

`bench/run_pernode.py --atomi m0=… --atomi m1=… --memory 192 --repeats 3`:

| Cell | Cap | `m0` ms/1M | `m1` ms/1M | `m0` ns/exp | `m1` ns/exp |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 1 M | 73.0 | 71.0 | 2,371 | 2,306 |
| `core` | 10 M | 64.5 | 63.9 | 2,064 | 2,045 |
| `easy-win` | 1 M | 65.0 | 66.0 | 2,050 | 2,081 |
| `easy-win` | 10 M | 58.0 | 59.4 | 1,804 | 1,848 |
| `deep-win` | 1 M | 68.0 | 68.0 | 2,019 | 2,019 |
| `deep-win` | 10 M | 59.5 | 59.3 | 1,789 | 1,783 |

`make pernode` (`m1` only, `core` 10 M, 192 MB): **64.2 ms/1M child evals,
2,054 ns/expansion** — the S4 point for adopted 2.6.0.

## Reproduce

    # m0: a copy of this worktree with Cargo.toml at "2.4" and the 2.4.0 lock
    cargo build --release            # inside that copy
    cargo build --release            # here: m1 on 2.6.0

    python3 measurements/movegen/plan1/bench.py \
        --m0 /tmp/atomi-m0-plan1/target/release/atomi \
        --m1 target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc --caps 10,120 --repeats 3 \
        --out measurements/movegen/plan1/bench.json
    python3 measurements/movegen/plan1/tree_probe.py \
        --m0 /tmp/atomi-m0-plan1/target/release/atomi \
        --m1 target/release/atomi --repeats 3 \
        --out measurements/movegen/plan1/tree_probe.json
    python3 bench/run_pernode.py \
        --atomi m0=/tmp/atomi-m0-plan1/target/release/atomi \
        --atomi m1=target/release/atomi \
        --cases core,1eb80b92f92c,db6a741c723f \
        --evals 1000000,10000000 --memory 192 --repeats 3 \
        --out measurements/movegen/plan1/pernode.json
    make pernode PERNODE_OUT=/tmp/atomi-pernode.json
