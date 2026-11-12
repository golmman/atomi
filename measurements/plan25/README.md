# Plan 25 measurements — item 33, per-node cost (NULL-V1 / NULL-S)

Plan: [`plan25.md`](../../docs/plans/prover/plan25.md). Report:
[`report25.md`](../../docs/plans/prover/report25.md). Host and binaries:
[`env.json`](env.json). Control is the pre-plan binary built from
`atomi@a257e6f` in `/tmp/atomi-plan25-control` (sha256 `e3e9fc25…`); the
candidate is the shipped tree (sha256 `144ef4a4…`).

## Files

| File | Contents |
| --- | --- |
| `phase_a.py` | Phase A driver: `deep-win`, seeds 0–2, at 20 M child evals, at 10 s, and a `--guide-group` arm at 20 M for the plain share |
| `phase_a.json` | Parsed Phase A rows and derived per-expansion costs |
| `gate.py` | Phase C driver: same-session control/candidate pairs, 120 s, three cells × seeds 0–2; optional candidate flags; one 600 s D14 proof run per proved candidate row |
| `gate_v1.json` | G-V1 (NULL-V1, screen off) rows, summary, guards, verdict |
| `gate_s.json` | G-S (NULL-V1 + `--guide-null-screen`) rows, summary, guards, verdict |
| `bench.json` | Cap-120 s ledger session: same-session provari reference + candidate, 3 repeats/cell |
| `env.json` | Host, CPU, both binary sha256 values, provari pin |

Run (one process at a time; Phase A ≈ 1 min, each gate ≈ 15 min):

    python3 measurements/plan25/phase_a.py
    python3 measurements/plan25/gate.py --out measurements/plan25/gate_v1.json
    python3 measurements/plan25/gate.py --out measurements/plan25/gate_s.json \
        --deep-threshold 0.67 --candidate-flags=--guide-null-screen
    python3 bench/run_bench.py --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc --atomi target/release/atomi \
        --atomi-stack 'hand=--guide hand' \
        --cases core,1eb80b92f92c,db6a741c723f --seconds 120 --memory 192 \
        --repeats 3 --out measurements/plan25/bench.json

## Phase A — attribution (`deep-win`, `--stats`)

| Cap | Median wall | `child_evals` | Expansions | µs/exp | aux/exp | Plain share | `null_calls` | `null_replies` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 M evals | 11.939 s | 20,000,007 | 689,907 | 17.21 | 8.84 | 0 (group off) | 417,458 | 15,561,197 |
| 10 s | 10.010 s | 16,737,580 | 573,440 | 17.46 | 8.89 | 0 (group off) | 351,315 | 13,088,991 |
| 20 M evals, `--guide-group` | 11.319 s | 20,000,020 | 718,054 | 15.99 | 9.74 | **42.8 %** | 408,682 | 13,170,858 |

The null replay is the dominant per-node term: ~37 replies per `null_win`
call and `aux_nodes` ≈ 8.8–9.7 per expansion. The pre-change control at 20 M
seed 0 measured 21.96 µs/exp against the candidate's 17.33 µs/exp (same
`child_evals` and `expansions` to the eval), so NULL-V1 is a ~0.79× per-node
cost cut with no trajectory change.

## G-V1 — NULL-V1 (screen off), 120 s, seeds 0–2

| Cell | Control walls | Median | Candidate walls | Median | Ratio | Gate |
| --- | --- | ---: | --- | ---: | ---: | --- |
| `core` | 10.075 / 8.350 / 8.160 | 8.350 s | 7.988 / 6.543 / 6.488 | 6.543 s | **0.784** | pass (≤ 1.1) |
| `easy-win` | 0.832 / 0.938 / 1.487 | 0.938 s | 0.623 / 0.718 / 1.103 | 0.718 s | **0.766** | pass (≤ 1.1) |
| `deep-win` | 112.250 / 104.876 / 76.995 | 104.876 s | 89.332 / 83.498 / 61.197 | 83.498 s | **0.796** | pass (≤ 0.80) |

`child_evals`/`expansions` are identical to the control on all nine pairs
(`same_tree` 9/9). D14: 9/9 candidate proofs exported at 600 s, matched
`child_evals`, and were accepted by `atomi validate` and `provari check`
(`core` 13.8–17.0 k positions, lengths 15/26/20; `easy-win` 2.8–4.1 k,
19/19/16; `deep-win` 60.7–93.1 k, 36/37/29). **G-V1 passes.**

## G-S — NULL-S (`--guide-null-screen`), 120 s, seeds 0–2

| Cell | Control median | Candidate median | Ratio | Tree vs control | Gate |
| --- | ---: | ---: | ---: | --- | --- |
| `core` | 8.946 s | 6.162 s | 0.689 | not identical (1.32–1.55× evals) | pass (≤ 1.1) |
| `easy-win` | 0.940 s | 3.148 s | **3.349** | 5.7–11.5× evals | **fail** |
| `deep-win` | 104.775 s | 59.801 s | **0.571** | 1.46–1.76× evals | pass (≤ 0.67) |

D14: 9/9 candidate proofs accepted. G-S **fails** the `easy-win` guard, so
NULL-V1 is adopted and `--guide-null-screen` stays opt-in (default off).

## Bench row (Cap 120 s)

Same-session reference / candidate walls: `core` 0.533 / 7.824 s,
`easy-win` 0.132 / 0.607 s, `deep-win` 4.367 / 88.330 s; factors 14.679 /
4.598 / 20.227, geomean **11.093** over 3 claimed cells. The D14 evidence is
`gate_v1.json` (same binary and flags, 9/9 accepted).
