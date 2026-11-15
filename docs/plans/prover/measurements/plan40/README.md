# Plan 40 measurements — kill review and item 48 GUIDE-EVAL

Plan: [`plan40.md`](../../plan40.md). Report: [`report40.md`](../../report40.md).
Host, binaries and pins: [`env.json`](env.json). One process at a time.

| File | Contents |
| --- | --- |
| `phase0/` | TERMS-V1 price harness (own code, path dependency on this repo), `results.txt` with the median-of-3 ns/call and term distributions |
| `phaseA/candidate.patch` | the scratch-only TERMS-V1 switch and function applied to `e39beae`; not a repo change |
| `runs.json` | parsed `deep-win` 5 M counters for control, TERMS-V1 and the repeated control |
| `run.sh` | the exact Phase A commands |

## Phase 0 — G-40a PASS

8,293 positions: `corpus-v1` (21) plus the `testutil` seeded legal playouts
(28 fixtures × 5 seeds × up to 120 plies). Median of 3 timed passes, 20 passes
per sample.

| Eval | ns/call |
| --- | ---: |
| current `hand_eval` | **149.11** |
| TERMS-V1 with mobility | **175.24** |
| TERMS-V1 without mobility (trim) | **139.20** |

The bar is **199 ns/call**. TERMS-V1 passes with mobility retained, so the
frozen formula keeps the existing mobility term. No position in the sample
saturates the `[-256, 256]` clamp; TERMS-V1's range is `[-226, 191]`.

Term ranges (scaled, after the fixed weights): material/32 `-76..76`,
pressure×12 `-108..108`, signed targets×20 `-60..60`, exposure×8 `-16..16`,
fuse×16 `-16..16`, checkers×-16 `-16..0`, mobility/4 `-9..8`. The new terms
fire on 5.8–16.5 % of positions (`targets` 16.5 %, exposure 12.5 %, fuse 7.0 %,
checkers 5.8 %).

## Phase A — G-40b FAIL

Same session, same `deep-win` FEN and flags, 5 M `child_evals` cap. The
control reproduces the committed 5,000,034 / 168,910 exactly, twice.

| Run | `child_evals` | expansions | vs control | ms |
| --- | ---: | ---: | ---: | ---: |
| control-1 | 5,000,034 | 168,910 | 1.00000 | 1681 |
| TERMS-V1 | 5,000,005 | **168,879** | **0.99982** | 1652 |
| control-2 | 5,000,034 | 168,910 | 1.00000 | 1652 |

The gate asks for **≤ 152,019** expansions (0.90×). TERMS-V1 moves the tree by
31 expansions, not 16,891. No proof or refutation exists at the cap, so the
"no proof/refutation change" clause is vacuous; the tree clause is decisive.
Phase B (opt-in `--guide-eval rich`, identity and three-cell matrix) was not
built, and no repo source changed.

## Limits

- Phase A is a single seed/cell at one cap; it cannot see a late tree cut
  after 5 M (D33). It can see that the eval has almost no effect on the early
  ordering, which is the gate's subject.
- The eval is read at the same points as today's `hand_eval`; the probe says
  nothing about a different read point or about ideation B (ordering-only).
- The harness's playout sample is the `testutil` construction, not the search
  tree; ns/call is a read-only price, not a search cost.
