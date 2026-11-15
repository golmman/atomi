# Plan 42 measurements — item 55 SCALE-LENGTH (w = 2 × bounded length)

Plan: [`plan42.md`](../../plan42.md). Report: [`report42.md`](../../report42.md).
Host, binaries and pins: [`env.json`](env.json). One process at a time, same
binary, 120 s wall cap, `--guide hand --memory 192 --seed N`, and
`--guide-ef-weight` written **after** `--guide hand` (D33).

| File | Contents |
| --- | --- |
| `run.py` | the driver: Phase 0 scan, Phase A matrix, the held-out set |
| `runs.json` | every 120 s run, parsed counters (resumable by label) |
| `summary.json` | Phase 0 table, held-out rows, gate verdicts |

## Phase 0 — stop rule fired

`deep-win`, seed 0, w = 2, 120 s. The `L = ∞` row reproduces plan41's w = 2
control exactly (3,165,142 expansions / length 25; wall 30,469 vs 30,401 ms).

| `L` | ms | expansions | length | vs `L = ∞` |
| ---: | ---: | ---: | ---: | ---: |
| **∞** | **30,469** | **3,165,142** | **25** | **1.000×** |
| 24 | 66,564 | 7,327,572 | 24 | 2.184× |
| 22 | 34,598 | 3,578,565 | 22 | 1.135× |
| 20 | 37,538 | 3,871,051 | 20 | 1.232× |

No `L < ∞` beats `L = ∞` at w = 2. The pre-registered stop rule fires: the
composition is closed negative and **no Phase A matrix is run**. The two tree
levers overlap on the length axis (w = 2 already gives length 25), and the
bounded pass pays more tree than the shorter proof buys.

## Held-out direction — G-42a FAIL

Five fixed `guidance-v1` cells, **0-based indices 6–10** (the first five
`wins.tsv` `dec*` cells: `dec01`, `dec03`, `dec04`, `dec05`, `dec07`), seed 0,
120 s, w = 2 vs 16. All five prove on both sides.

| Index | Cell | w = 16 ms | w = 2 ms | ratio | w = 2 faster |
| ---: | --- | ---: | ---: | ---: | --- |
| 6 | `dec01` | 171 | 480 | 2.807× | no |
| 7 | `dec03` | 13 | 12 | 0.923× | yes |
| 8 | `dec04` | 7 | 6 | 0.857× | yes |
| 9 | `dec05` | 1,197 | 1,588 | 1.327× | no |
| 10 | `dec07` | 3 | 3 | 1.000× | tie |

**2/5 faster, 1 tie, 2 slower** — the direction gate needs ≥ 3/5. The two
real regressions (`dec01` 2.8×, `dec05` 1.33×) are the same easy-cell
regression plan41 saw on `easy-win` (1.647×); the three near-ties are 3–13 ms
cells where wall is noise. w = 2 is a `deep-win`-specific lever, not a general
default.

## Gates

| Gate | Result |
| --- | --- |
| G-42a Phase 0 reproduced + held-out ≥ 3/5 | **FAIL** — Phase 0 exact, held-out 2/5 |
| G-42b composition ≤ 0.50× | **FAIL** — `L = ∞` best, so the gate is w = 2 alone: 0.604× (plan41) > 0.50×; composition is a no-op |
| G-42c D14 | **not applied** — G-42b fail; plan41 `d14.json` covers w = 2, `L = ∞` 9/9 |
| G-42d default | **not applied** — `ef_weight` stays 16, `max_length` off |
| G-42e `make test`, `make lint`, `make test-full` + veto | **PASS** |
| G-42f boundary | **PASS** — no new term, no model, no provari source, no `T_4`/`T_5` claim |

## Limits

- The held-out set is five easy `dec*` cells; three are 3–13 ms, so their wall
  direction is noise. The two non-trivial cells both regress under w = 2.
- Phase 0 is one seed, one cell. It is decisive for the composition question
  (no `L` is even close to `L = ∞`), not a full `L` curve at w = 2.
- The 5 M probe is not used: D33's fixed-cap trap makes it blind to this lever.
