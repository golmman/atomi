# Plan 31 measurements — HOTPATH-V2 (item 38): adopted at 0.72–0.78×

Plan: [`plan31.md`](../../docs/plans/prover/plan31.md). Report:
[`report31.md`](../../docs/plans/prover/report31.md). Host, binary sha256,
`atomic-movegen` 2.8.0 checksum and the provari pin: [`env.json`](env.json).
The repo head is `atomi@7f3728d` with the plan-31 change in the working tree;
the owner commits. Every number below belongs to the candidate binary
`sha256 3b6b3f6b…` and the control built from a `git archive` of `7f3728d`.

## What was measured

| File | Contents |
| --- | --- |
| `phase_a.py`, `phase_a.json` | Phase 0: per-expansion multiplicity at 1 M / 5 M evals on the three cells, from a scratch build that carries the plan's temporary counters (never committed) |
| `phase_a.json` `per_expansion` | explosion calls, list length, moves read, hits; null boards; reply lists; total moves read and their shares |
| `profile.py`, `profile_control.json`, `profile_candidate.json` | `perf` self time at 5 M `deep-win` `--guide hand`, before/after |
| `gate.py`, `gate_v2a_1m.json` | G-V2a: tree identity and wall at 1 M evals, 3 cells, 3 interleaved repeats |
| `gate_v2b.json` | G-V2b and the D21 ablations `{F1}` `{F2}` `{F3}` at 5 M `deep-win`, 3 interleaved repeats |
| `gate_v2c_120s.json` | G-V2c: 120 s, seeds 0–2, three cells, control vs candidate |
| `d14.py`, `d14.json` | D14: every proved (cell, seed) exported and claimed with both checkers |
| `bench.json` | the `bench.md` row: `bench/run_bench.py` at Cap 120 s with `--proof-check` |

## Phase 0 — the arithmetic closes (deep-win, 5 M, 168,910 expansions)

From `phase_a.json` (`deep_win`, `evals_cap 5000000`), per expansion:

| Quantity | Per expansion | Share |
| --- | ---: | ---: |
| moves read from every move list | 1,512.2 | 100 % |
| — one-ply explosion test (avg list 38.2) | 1,000.5 | **66.2 %** |
| — null-replay reply streams | 426.5 | 28.2 % |
| — null board, both passes | 51.2 | 3.4 % |
| — the node's own child list | 29.6 | 2.0 % |
| — replay lists | 4.4 | 0.3 % |
| explosion-test **calls** | 26.2 | — |
| — from the null reply test | 16.5 | 63 % |
| — from the guide settle (bits) | 7.9 | 30 % |
| — class / replay / full settle | 1.8 | 7 % |
| explosion-test hits | 5.8 | — |
| reply lists (pseudo-legal) | 14.0 | — |
| null boards (FEN round-trips) | 0.62 | — |

So the plan's two claims hold on 2.8.0: **26 calls per expansion, each scanning
a full ~38-move list to answer a ≤ 4-square question**, and **66 %** (the plan
said 63 %) of all list moves come from that one test. `perf` puts
`generate_pseudo_legal_in_view` at **21.0 %** self and
`immediate_explosion_bits` at **7.4 %** on the control, `from_fen` + `fen_with`
at 2.1 % — i.e. moves ≈ 21 % of wall, which caps the stack at ~0.75× if the
lists go and the answer stays exact. The candidate profile has the same
symbols at **8.3 %** and 0 % (the list cost that remains is the frame's own
child list and the reply stream's walk, which F3 does not remove).

## G-V2b + ablations (5 M `deep-win`, seed 0, 3 interleaved repeats)

| Side | Median wall | Ratio vs control | Tree |
| --- | ---: | ---: | --- |
| control | 2,546 ms | 1.0000 | — |
| `{F1}` existence-without-list | 2,271 ms | **0.8920** | identical |
| `{F2}` `Board::null_for` | 2,440 ms | **0.9584** | identical |
| `{F3}` reply stream | 2,369 ms | **0.9305** | identical |
| `{F1+F2+F3}` | 1,979 ms | **0.7773** | identical |

The product of the three single ratios is 0.795 and the stack measures 0.777
(mildly super-additive: the reply-stream and explosion savings share the same
legality/attack work). G-V2b's bar was ≤ 0.80× with identical
`child_evals`/`expansions` — passed with `5,000,034 / 168,910` on every run.

## G-V2c (120 s, seeds 0–2, control vs stack)

| Cell | seed 0 | seed 1 | seed 2 |
| --- | ---: | ---: | ---: |
| `core` | **0.7500** | **0.7514** | **0.7367** |
| `easy-win` | **0.7328** | **0.7306** | **0.7223** |
| `deep-win` | **0.7541** | **0.7461** | **0.7439** |

Every (cell, seed) ≤ 0.80×, none > 1.0×, `child_evals`/`expansions` identical
on all 18 runs. Control `deep-win` medians 77.7 / 72.4 / 52.5 s against
candidate 58.6 / 54.0 / 39.1 s.

## D14 (120 s, both checkers)

9/9 seeds proved and claimed: `atomi validate` and `provari check` both accept,
with `positions == won_positions` on every export (core 16,991 /
easy-win 2,980 / deep-win 93,092 positions, lengths 15 / 19 / 36).

## Bench row (Cap 120 s, `--guide hand`)

| Cell | reference (provari hand) | candidate | factor |
| --- | ---: | ---: | ---: |
| `core` | 0.527 s | 5.288 s | **10.034** |
| `easy-win` | 0.135 s | 0.388 s | **2.874** |
| `deep-win` | 4.459 s | 58.439 s | **13.106** |
| geomean (3/3 claimed) | | | **7.23** |

## Reproduce

    # Phase 0 (needs the scratch counter build; not in this repo)
    python3 measurements/plan31/phase_a.py \
        --probe /tmp/atomi-plan31-p0/target/release/atomi \
        --control target/release/atomi --evals 1000000,5000000 \
        --out measurements/plan31/phase_a.json

    # gates
    python3 measurements/plan31/gate.py \
        --side control=/tmp/atomi-plan31-control/target/release/atomi \
        --side all=target/release/atomi --cells core,easy-win,deep-win \
        --evals 0 --seconds 120 --seeds 0,1,2 --repeats 3 \
        --out /tmp/gate_v2c.json
    python3 measurements/plan31/d14.py --atomi target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --out /tmp/d14.json
    python3 bench/run_bench.py \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
        --atomi target/release/atomi --atomi-stack "hand=--guide hand" \
        --cases core,1eb80b92f92c,db6a741c723f --seconds 120 --repeats 3 \
        --proof-check \
        --provari-check /tmp/atomi-oracle/provari/target/release/provari \
        --out measurements/plan31/bench.json
