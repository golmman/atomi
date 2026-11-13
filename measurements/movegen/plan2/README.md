# Plan 2 measurements — `atomic-movegen` 2.6.0 (`m0`) vs 2.7.0 (`m1`)

Plan: [`plan2.md`](../../../docs/plans/movegen/plan2.md). Report:
[`report2.md`](../../../docs/plans/movegen/report2.md). Host, binaries and
gate verdicts: [`env.json`](env.json). The worktree is uncommitted on top of
`atomi@f88533e`; the owner commits.

Both sides are built from the **same source tree** — only `Cargo.toml` and
`Cargo.lock` differ — so every difference below is the crate's:

| Side | Dependency | Binary |
| --- | --- | --- |
| `m0` (control) | published 2.6.0 | `/tmp/atomi-m0-plan2/target/release/atomi`, sha256 `a70823f5…` |
| `m1` (candidate) | published 2.7.0 | `target/release/atomi`, sha256 `e3bb9d5e…` |

## Files

| File | Contents |
| --- | --- |
| `bench.py` | Wall bench: the three bench cells at Cap 10 s and 120 s, `provari --guide hand` (10 s) + `m0` + `m1` per repeat, alternating, one process at a time; command building, parsing and ledger arithmetic imported from `bench/run_bench.py` |
| `bench.json` | Parsed runs (`runs`), per cell/cap medians and factors (`ledger`), geomeans (`summary`) |
| `tree_probe.py` | 5 M child evals on `deep-win`, seed 0, `--guide hand`: tree identity and the same-tree wall ratio |
| `tree_probe.json` | Parsed probe |
| `pernode.json` | `bench/run_pernode.py` with **both** sides (S4: per-adopted-version per-expansion cost) |
| `pernode_make.json` | `make pernode` (the `m1` side, the target's default) |
| `env.json` | Host, head, pins, checksums, binary sha256s, gate verdicts, the new-test tallies |

`bench.py`/`tree_probe.py` are the plan1 drivers with the version labels and
paths changed; the measurement contract (cells, caps, repeats, parsing) is
unchanged, so the two plans' numbers are directly comparable.

## Wall bench (bare tree, `--guide none`)

The driver passes no `--guide`, so the measured stack is the **bare** tree —
the product candidate (D28), the same stack as the plan11/plan15 rows. This
is a *different stack* from the plan23/25/27 rows, which pass
`--atomi-stack 'hand=--guide hand'`; the two are never compared (reading
rule 4).

| Cap | Cell | provari hand | `m0` 2.6.0 | `m1` 2.7.0 | factor `m0` | factor `m1` | `m1/m0` | Tree |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 10 s | `core` | 0.533 s | censored | censored | ≥ 18.76 | ≥ 18.76 | — | — |
| 10 s | `easy-win` | 0.131 s | 1.868 s | 1.840 s | **14.260** | **14.046** | 0.985 | identical |
| 10 s | `deep-win` | 6.496 s | censored | censored | ≥ 1.54 | ≥ 1.54 | — | — |
| 120 s | `core` | 0.537 s | 18.093 s | 18.194 s | **33.693** | **33.881** | 1.006 | identical |
| 120 s | `easy-win` | 0.136 s | 1.906 s | 1.850 s | **14.015** | **13.603** | 0.971 | identical |
| 120 s | `deep-win` | 6.545 s | censored | censored | ≥ 18.33 | ≥ 18.33 | — | — |

Coverage `m1`: 1/3 at Cap 10 s (geomean 14.046), 2/3 at Cap 120 s (geomean
21.468; `m0`: 21.730). Paired cells (`easy-win`@10 s, `core`@120 s,
`easy-win`@120 s): `m1/m0` geomean **0.987**, tree identical 3/3
(`child_evals` 30,690,269 / 281,668,134 and `expansions` 1,062,931 /
9,796,753 equal on every paired cell). Censoring matches plan1's row of the
same stack, so the pin did not move coverage.

## Tree probe (5 M child evals, `deep-win`, `--guide hand`, seed 0)

| Side | Expansions | `child_evals` | Median `ms` | Runs |
| --- | ---: | ---: | ---: | --- |
| `m0` | 168,910 | 5,000,034 | 2,583 | 2603 / 2579 / 2583 |
| `m1` | 168,910 | 5,000,034 | 2,581 | 2603 / 2581 / 2555 |

Tree **identical to the plan28/plan1 control** (168,910 expansions) on both
sides; `m1/m0` = **0.9992** — flat, inside the ±2–3 % repeat spread of the
same session (the runs overlap).

## Per-expansion cost (S4)

`bench/run_pernode.py --atomi m0=… --atomi m1=… --memory 192 --repeats 3`:

| Cell | Cap | `m0` ms/1M | `m1` ms/1M | `m0` ns/exp | `m1` ns/exp |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 1 M | 72.0 | 72.0 | 2,338.7 | 2,338.7 |
| `core` | 10 M | 62.5 | 62.5 | 1,999.7 | 1,999.7 |
| `easy-win` | 1 M | 65.0 | 66.0 | 2,049.8 | 2,081.4 |
| `easy-win` | 10 M | 59.5 | 57.1 | 1,851.0 | 1,776.4 |
| `deep-win` | 1 M | 68.0 | 66.0 | 2,019.0 | 1,959.6 |
| `deep-win` | 10 M | 58.1 | 57.0 | 1,747.1 | 1,714.0 |

`child_evals` and `expansions` are identical on every cell and cap — the tree
does not move, so the ns/expansion difference is per-node cost, and it is
inside the session spread (both signs, ≤ 4 %).

`make pernode` (`m1` only, `core` 10 M, 192 MB): **63.3 ms/1M child evals,
2,025.3 ns/expansion** — the S4 trend point for adopted 2.7.0 (2.6.0's was
2,054.3 in the plan1 session; cross-session, so a trend line, not a ratio).

## Reproduce

    # m0: a copy of this worktree with Cargo.toml at "2.6" and the 2.6.0 lock
    cargo build --release            # inside that copy (/tmp/atomi-m0-plan2)
    cargo build --release            # here: m1 on 2.7.0

    python3 measurements/movegen/plan2/bench.py \
        --m0 /tmp/atomi-m0-plan2/target/release/atomi \
        --m1 target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc --caps 10,120 --repeats 3 \
        --out measurements/movegen/plan2/bench.json
    python3 measurements/movegen/plan2/tree_probe.py \
        --m0 /tmp/atomi-m0-plan2/target/release/atomi \
        --m1 target/release/atomi --repeats 3 \
        --out measurements/movegen/plan2/tree_probe.json
    python3 bench/run_pernode.py \
        --atomi m0=/tmp/atomi-m0-plan2/target/release/atomi \
        --atomi m1=target/release/atomi \
        --cases core,1eb80b92f92c,db6a741c723f \
        --evals 1000000,10000000 --memory 192 --repeats 3 \
        --out measurements/movegen/plan2/pernode.json
    make pernode PERNODE_OUT=/tmp/atomi-pernode-plan2.json
