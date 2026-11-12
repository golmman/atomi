# Plan 26 measurements — item 34, NULL-V2 (per-node residual)

> **Addendum (2026-11-12):** do not re-apply `null_v2.patch` verbatim.
> Its `DEFAULT_ENTRIES` is `1 << 21`, not the measured `1 << 17`: the 5 M
> `deep-win` probe then gives 168,885 expansions (the tree-changing 2^21 arm)
> instead of the control's 168,910. The corrected, adopted implementation is
> [`measurements/plan27/`](../plan27/README.md) (D39).

Plan: [`plan26.md`](../../docs/plans/prover/plan26.md). Report:
[`report26.md`](../../docs/plans/prover/report26.md). Host and binaries:
[`env.json`](env.json). Control is the pre-plan tree built from
`atomi@939d890` in `/tmp/atomi-plan26-control` (sha256 `01f1d6ce…`); the
candidate is the NULL-V2 build (sha256 `0e0f1fb2…`). The NULL-V2 source was
reverted after the failed gate; [`null_v2.patch`](null_v2.patch) is the
complete diff (including the new `src/table/null_cache.rs`) for a future
re-application.

## Files

| File | Contents |
| --- | --- |
| `gate.py` | Phase C driver: same-session control/candidate pairs, 120 s, three cells × seeds 0–2; optional 600 s D14 proof run per proved candidate row |
| `gate_n2.json` | G-N2 rows, summary, guards, verdict (`proof_skipped: true`; the wall guard failed first) |
| `phase_a.json` | Phase A cache sweep at 5 M child evals, `deep-win`, seed 0: 2^17/2^18/2^20/2^21 entries, hit rate, wall, RSS, tree identity; positive-null ceiling counter |
| `null_v2.patch` | The reverted NULL-V2 implementation (EARLY-REPLY + EXPOSED-CACHE + `--stats` counters) |
| `env.json` | Host, CPU, control/candidate/reverted binary sha256 values, provari pin |

Run (one process at a time; the timed gate ≈ 12 min):

    python3 measurements/plan26/gate.py --skip-proof \
        --out measurements/plan26/gate_n2.json

## Phase A — cache sizing (`deep-win`, 5 M evals, seed 0)

| Entries | Cache | Hits | Misses | Hit rate | Wall | RSS | Tree vs control |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| control | — | — | — | — | 3.015 s | 217.3 MB | — |
| 2^17 | 1.25 MB | 1,014,999 | 2,364,622 | 30.0 % | **2.557 s** | 217.9 MB | **identical** |
| 2^18 | 2.50 MB | 1,065,092 | 2,314,529 | 31.5 % | 2.655 s | 218.2 MB | differs (168,909 exp) |
| 2^20 | 10.0 MB | 1,166,804 | 2,212,858 | 34.5 % | 2.699 s | 218.5 MB | differs (105,121 null calls) |
| 2^21 | 20.0 MB | 1,200,836 | 2,178,341 | 35.5 % | 2.801 s | 218.0 MB | differs (168,885 exp) |

The cache is taken from the filter first (tree-neutral: the filter is exact
in the negative direction), then from the TT. 2^17 is the largest size that
fits the filter slack, so the TT and proof table keep their sizes and the
tree is identical to the control. Larger caches take TT bytes and change the
trajectory, so they are not eligible for G-N2.

**Positive-null ceiling.** 11,651 of 105,120 `null_win` calls (11.1 %) are
repeats of a previously positive null position. A positive-null cache was
probed and **excluded**: it changed the tree (`null_calls` 104,137 vs
105,120) because it freezes a table-dependent result.

## G-N2 — NULL-V2, 120 s, seeds 0–2

| Cell | Control walls | Median | Candidate walls | Median | Ratio | Gate |
| --- | --- | ---: | --- | ---: | ---: | --- |
| `core` | 7.879 / 6.723 / 6.553 | 6.723 s | 7.115 / 5.945 / 5.782 | 5.945 s | 0.8843 | pass (≤ 1.1) |
| `easy-win` | 0.628 / 0.716 / 1.112 | 0.716 s | 0.538 / 0.618 / 0.942 | 0.618 s | 0.8631 | pass (≤ 1.1) |
| `deep-win` | 89.383 / 83.029 / 60.626 | 83.029 s | 77.163 / 71.666 / 51.296 | 71.666 s | **0.8631** | **fail (≤ 0.85)** |

`child_evals`/`expansions` are identical to the control on all nine pairs
(`same_tree` 9/9). D14 was not run: the wall guard failed first, so no claim
is made. **G-N2 fails**; the per-node micro-line is closed per the plan's
stop rule.
