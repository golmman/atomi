# Plan 22 measurements — item 30, proof-memory composition

Plan: [`plan22.md`](../../docs/plans/prover/plan22.md). Report:
[`report22.md`](../../docs/plans/prover/report22.md). Host and binary:
[`env.json`](env.json) (`atomi` sha256 `3b1a3380…`, git `df9ba5a`).

## Files

| File | Contents |
| --- | --- |
| `phase_a.py` | Phase A driver: four sequential `deep-win` runs, parsed JSON only, both checkers on every written proof |
| `composition.json` | The four rows, checker verdicts and the evaluated stop rule |
| `env.json` | Host, CPU, atomi binary sha256, provari version/commit |

There is no `gate.json` or `matrix.json`: Phase A's stop rule fired, so
Phase B/C were not executed (plan22 §Phase A).

Run (one process at a time, ≈ 9 min):

    python3 measurements/plan22/phase_a.py \
        --atomi target/release/atomi \
        --provari-check /tmp/atomi-oracle/provari/target/release/provari

## Phase A (`deep-win`, `--guide hand`, 600 s, seed 0)

| Run | Total MB | Share | Attacker live | Defender live | `proof_entries` | Export | Positions | Length | Child evals | Wall | Both checkers |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | --- |
| A1 | 384 | 25 % | 5,639,488 | 22,822 | 5,662,310 | `store-full` | — | — | 189,477,824 | 158.6 s | — |
| A2 | 192 | 25 % | 2,820,163 | 10,992 | 2,831,155 | `store-full` | — | — | 205,208,471 | 174.9 s | — |
| A3 | 192 | 50 % | 5,595,484 | 24,545 | 5,620,029 | `written` | 76,405 | 36 | 134,818,742 | 103.4 s | accept / accept |
| A4 | 192 | 75 % | 5,687,070 | 29,804 | 5,716,874 | `written` | 93,092 | 36 | 155,236,740 | 114.7 s | accept / accept |

`live` is `proof_entries`; the table refuses at 90 % of its slots, so A1/A2
stop exactly at that bound. The two composition counters are exact live
class counts; the run never changes them after the search ends.

## Stop rule A

| Quantity | Value |
| --- | ---: |
| A1 live attacker entries | 5,639,488 |
| 9/10 × 192 MB default-share slots (`2,831,155`) | 2,831,155 |
| Ratio | **1.99×** |
| A1 truncated by refusal? | **yes** (`proof_full: true`) |
| **Fires?** | **yes** |

A1 does **not** reproduce report21's 384 MB "written" row: at the default
25 % proof share it also refuses. Its tree is smaller than A2's (189.5 M vs
205.2 M child evals) but the archive it would need is still 1.99× the whole
192 MB proof-table slot count, and the memo tier is only 0.4 % of live
entries. Evicting defenders cannot make MEM-V1 fit.

## Observations (not a gate)

A3/A4 are single-seed, 600 s probes, not Phase C rows: at 192 MB a larger
proof share writes a proof accepted by both checkers (103–115 s). That is a
possible alternative lever for the next review; plan22 forbids per-cell
re-tuning here, and no default, row or claim follows from it.
