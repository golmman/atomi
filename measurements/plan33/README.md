# Plan 33 measurements — item 40 (NULL-PIPE): exact, 0.926× on `deep-win`

Plan: [`plan33.md`](../../docs/plans/prover/plan33.md). Report:
[`report33.md`](../../docs/plans/prover/report33.md). Host, binaries and pins:
[`env.json`](env.json). One execution session: the engine change landed first
(`null_win`'s pass 1 pipelined + `Tables::prefetch_proved`), then every gate ran
against a control built from `git archive HEAD` (`01ee0fe`) outside the repo.

| File | Contents |
| --- | --- |
| `execute.py` | the gate driver (adapted from [`plan32`](../plan32/execute.py)): `tree` = counter identity at `--evals`, `wall` = interleaved control/candidate pairs, `--wall-evals` for an evals-capped pair |
| `gate_tree_1m.json` | G-40b small tier: 3 cells at 1 M child evals, identical 3/3 |
| `gate_tree_5m.json` | G-40b: the 5 M `deep-win` probe, identical (5,000,034 / 168,910 / 215,267) |
| `gate_wall_5m.json` | the 5 M probe wall, 3 interleaved pairs, median **0.9227×** |
| `gate_wall_120s_deep.json` | G-40c (binding): `deep-win` @120 s, 3 interleaved pairs, seeds 0–2, median **0.926×**, tree identical 3/3 |
| `gate_wall_120s_small.json` | `core` / `easy-win` @120 s, 3 pairs each (informational, and 6 more tree-identity rows) |
| `tune_5m.json` | the `K` and prefetch tuning, 5 M `deep-win`, n = 4 interleaved (control + K = 8/16/32 + K = 8 without the prefetch) |
| `profile_120s.json` | G-40f: one uninstrumented and one rdtsc-instrumented 120 s `deep-win` run, pass 1 split into `status` / `do_move`+`undo` / probe / clone / `key` |
| `d14.json` | D14: every proved (cell, seed) exported and claimed with both checkers, 9/9 |
| `bench.json` | the `bench.md` row: Cap 120 s, `--proof-check`, geomean **6.606** on 3 claimed cells |

## Gates

Control `78f0c3cb…` (no pipeline), candidate `110ffc23…` (pipeline, `K = 16`;
byte-identical after the later test-only split into `src/guide/null/tests.rs`);
one process at a time, control and candidate interleaved, `--seed` = repeat.
`ratio` = candidate wall / control wall.

| Gate | Bar | Result |
| --- | --- | --- |
| G-40a exactness | `pipelined_pass1_matches_reference` (fixtures + every seeded playout position, K = 1/8/16) + `make test` | **PASS** — plus `pass1_chunk_boundaries_match_the_reference` (a proof on every child index of 4 boards); `make test-full` green |
| G-40b tree identity | 3 cells at 1 M, the 5 M probe, 9 × (cell, seed) at 120 s | **PASS** — 3/3, 1/1, 9/9 (`child_evals`, `expansions`, `proof_entries`, `found`, `length`) |
| G-40c wall (binding) | `deep-win` @120 s median ≤ 0.95× (3 pairs) **and** the 5 M probe ≤ 0.95× | **PASS** — 0.926 (0.9257 / 0.9260 / 0.9334) and 0.9227 (0.9124 / 0.9227 / 0.9556) |
| G-40d | `make lint`, `make test-full`, `make veto`, D14 9/9 | **PASS** — VETO 8 claim rows + 4 controls agree; D14 9/9 both checkers |
| G-40e | one `bench.md` Cap-120 row | **PASS** — geomean **6.606** (plan32 7.064), `deep-win` 53.42 s |
| G-40f | one 120 s profile of pass 1's phases | **PASS** — pass 1 is 13.2 pp of the wall (see below) |
| stop rule | `deep-win` median ≥ 0.98× or G-40b not identical ⇒ ship nothing | **not triggered** |

Informational small cells (`core` 0.9257, `easy-win` 0.9593, n = 3 each): they
agree with `deep-win` in sign, but a 0.4–5 s cell cannot resolve 2 % (§3 of
report32), so neither the gate nor the claim rests on them.

## Tuning: the batch pays, the hint does not

5 M `deep-win`, n = 4, every side interleaved with the same control run
(`tune_5m.json`, ratios of the same repeat):

| Side | median ratio | tree identical |
| --- | ---: | --- |
| K = 8 | 0.940 | yes |
| K = 16 (shipped) | 0.931 | yes |
| K = 32 | 0.921 | yes |
| K = 8, `prefetch_proved` removed | 0.939 | yes |

So the ~6 % comes from **keying a chunk before probing it**, not from the
prefetch hint — the same asymmetry plan32 measured for the proof filter (a
probe issued early already overlaps the loop; a *dependent* probe does not).
K = 16 vs K = 32 is inside noise; K = 16 is shipped (the plan's candidate and a
smaller stack array).

## G-40f — where pass 1's wall is, after the pipeline (`profile_120s.json`)

120 s `deep-win` seed 0: wall 53.03 s (uninstrumented), 5,803,257 expansions,
3,431,807 `null_win` calls, 123,171,039 moves scanned (35.9 per call). The
instrumented build reads rdtsc at both ends of every interval; its own overhead
is the wall delta (1.6 s) over the read count, 3.24 ns per read, given back to
each interval. `ns/exp` divides each phase by the run's **expansions**.

| Phase (pass 1) | s | pp of wall | ns/exp |
| --- | ---: | ---: | ---: |
| `rules::status` | 3.234 | **6.10** | 557 |
| `do_move` + `undo_move` | 2.303 | **4.35** | 397 |
| `proved` + prefetch | 1.115 | **2.10** | 192 |
| `null_board` + scratch clone | 0.204 | 0.38 | 35 |
| `rules::key` + `rule50` | 0.140 | 0.26 | 24 |
| **pass 1 total** | 6.996 | **13.20** | 1,206 |
| the rest of the run | 46.004 | 86.80 | 7,927 |

Reading: the probe line is closed at 2.1 pp (the pipeline already hides most of
the 9.4 pp ceiling — that is the 7.4 pp it banked), and what remains in pass 1
is the child build and the crate's terminality question. `rules::status` at
6.10 pp is the consumer number item 41 asked for: its bar (≥ ~2 pp, else the
upstream ask is a non-item) is met **on our side of the boundary**.

## Reproduce

    git archive HEAD | tar -x -C /tmp/atomi-plan33-control   # 01ee0fe, pre-plan-33
    (cd /tmp/atomi-plan33-control && cargo build --release)
    cargo build --release

    python3 measurements/plan33/execute.py --mode tree \
        --control /tmp/atomi-plan33-control/target/release/atomi \
        --candidate target/release/atomi --evals 1000000 \
        --out measurements/plan33/gate_tree_1m.json
    python3 measurements/plan33/execute.py --mode tree \
        --control /tmp/atomi-plan33-control/target/release/atomi \
        --candidate target/release/atomi --evals 5000000 --cells deep-win \
        --out measurements/plan33/gate_tree_5m.json
    python3 measurements/plan33/execute.py --mode wall \
        --control /tmp/atomi-plan33-control/target/release/atomi \
        --candidate target/release/atomi --cells deep-win --wall-evals 5000000 \
        --repeats 3 --out measurements/plan33/gate_wall_5m.json
    python3 measurements/plan33/execute.py --mode wall \
        --control /tmp/atomi-plan33-control/target/release/atomi \
        --candidate target/release/atomi --cells deep-win --seconds 120 \
        --repeats 3 --out measurements/plan33/gate_wall_120s_deep.json
    python3 measurements/plan33/d14.py --atomi target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --out measurements/plan33/d14.json
    python3 bench/run_bench.py --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
        --atomi target/release/atomi --atomi-stack "hand=--guide hand" \
        --cases core,1eb80b92f92c,db6a741c723f --seconds 120 --repeats 3 \
        --proof-check --provari-check /tmp/atomi-oracle/provari/target/release/provari \
        --out measurements/plan33/bench.json

The tuning variants and the instrumented build are scratch copies of the
control tree with this session's `src/guide/null.rs` (see `env.json`); the
rdtsc calibration is a 1 s `sleep` between two `rdtsc` reads (3.400013 GHz,
stable to 6 digits over three runs).
