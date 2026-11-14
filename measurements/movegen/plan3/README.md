# Plan 3 measurements — `atomic-movegen` 2.7.0 (`m0`) vs 2.8.0 (`m1`)

Plan: [`plan3.md`](../../../docs/plans/movegen/plan3.md). Report:
[`report3.md`](../../../docs/plans/movegen/report3.md). Host, binaries and
gate verdicts: [`env.json`](env.json). The worktree is uncommitted on top of
`atomi@d675701`; the owner commits.

Both sides are built from the **same source tree** — only `Cargo.toml` and
`Cargo.lock` differ (`src/` and `build.rs` verified byte-identical) — so every
difference below is the crate's:

| Side | Dependency | Binary |
| --- | --- | --- |
| `m0` (control) | published 2.7.0 | `/tmp/atomi-m0-plan3/target/release/atomi`, sha256 `ec24604f…` |
| `m1` (candidate) | published 2.8.0 | `target/release/atomi`, sha256 `d35cd0f8…` |

## Files

| File | Contents |
| --- | --- |
| `bench.py` | Wall bench: the three bench cells at Cap 10 s and 120 s, `provari --guide hand` (10 s) + `m0` + `m1` per repeat, alternating, one process at a time; command building, parsing and ledger arithmetic imported from `bench/run_bench.py` |
| `bench.json` | Parsed runs (`runs`), per cell/cap medians and factors (`ledger`), geomeans (`summary`) |
| `tree_probe.py` | 5 M child evals on `deep-win`, seed 0, `--guide hand`: tree identity and the same-tree wall ratio |
| `tree_probe.json` | Parsed probe |
| `pernode.json` | `bench/run_pernode.py` with **both** sides (S4: per-adopted-version per-expansion cost) |
| `pernode_make.json` | `make pernode` (the `m1` side, the target's default) |
| `pernode*.csv` | The same two runs as the drivers' own CSV rows (kept as written) |
| `env.json` | Host, head, pins, checksums, binary sha256s, gate verdicts, the new-test tallies |

`bench.py`/`tree_probe.py` are the plan2 drivers with the version labels and
paths changed; the measurement contract (cells, caps, repeats, parsing) is
unchanged, so the plans' numbers are directly comparable.

## Wall bench (bare tree, `--guide none`)

The driver passes no `--guide`, so the measured stack is the **bare** tree —
the product candidate (D28), the same stack as the plan11/plan15/movegen
plan1/plan2 rows. It is a *different stack* from the hand-stack rows, never
compared with them (reading rule 4).

| Cap | Cell | provari hand | `m0` 2.7.0 | `m1` 2.8.0 | factor `m0` | factor `m1` | `m1/m0` | Tree |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 10 s | `core` | 0.528 s | censored | censored | ≥ 18.94 | ≥ 18.94 | — | — |
| 10 s | `easy-win` | 0.138 s | 1.878 s | 1.875 s | **13.609** | **13.587** | 0.9984 | identical |
| 10 s | `deep-win` | 6.636 s | censored | censored | ≥ 1.51 | ≥ 1.51 | — | — |
| 120 s | `core` | 0.526 s | 18.246 s | 18.364 s | **34.688** | **34.913** | 1.0065 | identical |
| 120 s | `easy-win` | 0.133 s | 1.887 s | 1.892 s | **14.188** | **14.226** | 1.0026 | identical |
| 120 s | `deep-win` | 6.626 s | censored | censored | ≥ 18.11 | ≥ 18.11 | — | — |

Coverage `m1`: 1/3 at Cap 10 s (geomean 13.587; `m0` 13.609), 2/3 at Cap 120 s
(geomean 22.286; `m0` 22.185), `deep-win` censored at both caps on both sides.
Paired cells (`easy-win`@10 s, `core`@120 s, `easy-win`@120 s): `m1/m0`
geomean **1.0025**, tree identical 3/3 (`child_evals` 30,690,269 /
281,668,134 and `expansions` 1,062,931 / 9,796,753 equal on every paired
cell). The censored cells' `child_evals` differ run to run (a wall-capped run
stops where the clock stops), which is why only paired cells carry a tree
verdict.

## Tree probe (5 M child evals, `deep-win`, `--guide hand`, seed 0)

| Side | Expansions | `child_evals` | Median `ms` | Runs |
| --- | ---: | ---: | ---: | --- |
| `m0` | 168,910 | 5,000,034 | 2,633 | 2633 / 2664 / 2614 |
| `m1` | 168,910 | 5,000,034 | 2,632 | 2632 / 2588 / 2647 |

Tree **identical to the plan28/plan1/plan2 control** (168,910 expansions) on
both sides; `m1/m0` = **0.9996** — flat, well inside the ±2–3 % repeat spread
of the session (the runs overlap).

## Per-expansion cost (S4)

`bench/run_pernode.py --atomi m0=… --atomi m1=… --memory 192 --repeats 3`:

| Cell | Cap | `m0` ms/1M | `m1` ms/1M | `m0` ns/exp | `m1` ns/exp |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 1 M | 69.0 | 72.0 | 2,241.3 | 2,338.7 |
| `core` | 10 M | 62.7 | 61.9 | 2,006.1 | 1,980.5 |
| `easy-win` | 1 M | 67.0 | 66.0 | 2,112.9 | 2,081.4 |
| `easy-win` | 10 M | 57.7 | 57.1 | 1,795.0 | 1,776.4 |
| `deep-win` | 1 M | 66.0 | 66.0 | 1,959.6 | 1,959.6 |
| `deep-win` | 10 M | 57.2 | 58.8 | 1,720.0 | 1,768.1 |

`child_evals` and `expansions` are identical on every cell and cap, so the
ns/expansion difference is per-node cost, and it is inside the session spread
(both signs, ≤ 4 %).

`make pernode` (`m1` only, `core` 10 M, 192 MB): **62.5 ms/1M child evals,
1,999.7 ns/expansion** (the two-side run's `m1` median: 61.9 / 1,980.5) — the
S4 trend point for adopted 2.8.0. 2.7.0's point was 2,025.3 (plan2 session)
and 2,000 in this one's `m0` column; cross-session, so a trend line, not a
ratio.

## Reproduce

    # m0: a copy of this worktree with Cargo.toml at "2.7" and the 2.7.0 lock
    cargo build --release            # inside that copy (/tmp/atomi-m0-plan3)
    cargo build --release            # here: m1 on 2.8.0

    python3 measurements/movegen/plan3/bench.py \
        --m0 /tmp/atomi-m0-plan3/target/release/atomi \
        --m1 target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc --caps 10,120 --repeats 3 \
        --out measurements/movegen/plan3/bench.json
    python3 measurements/movegen/plan3/tree_probe.py \
        --m0 /tmp/atomi-m0-plan3/target/release/atomi \
        --m1 target/release/atomi --repeats 3 \
        --out measurements/movegen/plan3/tree_probe.json
    python3 bench/run_pernode.py \
        --atomi m0=/tmp/atomi-m0-plan3/target/release/atomi \
        --atomi m1=target/release/atomi \
        --cases core,1eb80b92f92c,db6a741c723f \
        --evals 1000000,10000000 --memory 192 --repeats 3 \
        --out measurements/movegen/plan3/pernode.json
    make pernode PERNODE_OUT=measurements/movegen/plan3/pernode_make.json
