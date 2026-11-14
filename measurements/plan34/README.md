# Plan 34 measurements — item 42 (EXPANSION-CENSUS): the exhaustive-bucket census

Plan: [`plan34.md`](../../docs/plans/prover/plan34.md). Report:
[`report34.md`](../../docs/plans/prover/report34.md). Host, binaries and pins:
[`env.json`](env.json). One execution session, one process at a time: the
shipped binary is never changed (no engine code, flag, default or `bench.md`
row); every instrumented build is a copy of the tree **outside** the repo,
made by [`instrument.py`](instrument.py).

| File | Contents |
| --- | --- |
| `instrument.py` | builds a scratch instrumented tree: adds `src/probe.rs` (rdtsc interval counters per (leaf, site), an inclusive aggregate per region, an in-situ empty-interval *tax* sample, a counting global allocator) and wraps 78 call sites (67 leaves, 11 region aggregates). Options: `--leaves on/off` (aggregates only), `--only <leaf,…>` (one leaf group), `--stride N` (unused by the final design) |
| `census.py` | the driver: reference run + the `sites` build + the `full` build + 7 `--only` builds, tree identity on every run, then the region/op/bucket arithmetic and the decision numbers. Writes `census_120s.json` |
| `census_120s.json` | the census: per-run walls and binary shas, the raw tics/interval counts of every instrumented (leaf, site), the region and op tables, the per-expansion counts, the observer-effect table, the decision block |
| `tsc_hz.c` | the TSC rate (three 1 s sleeps). This session: **3,400,037,455 Hz** |

## Reproduce

    cargo build --release                     # the reference binary (clean tree)
    python3 measurements/plan34/census.py --out measurements/plan34/census_120s.json

≈ 12 min: 10 runs of `atomi prove "<deep-win FEN>" --guide hand --memory 192
--seconds 120 --seed 0`, each 53–90 s. The scratch trees (≈ 1.1 GB) live in
`/tmp/atomi-plan34/<variant>` and are never committed.

## Design (what the earlier attempts got wrong)

The first attempt timed **every** leaf in one build. It is unusable:

- the run inflates 53.66 → **90.32 s**, and an inclusive aggregate's window
  then contains the instrumentation inside it: `enter` reads 42.5 s and the
  null replay 42.9 s against true values of 27.8 and 24.2 s (their sum exceeds
  the engine's own wall);
- sampling one expansion in 32 to cut that cost is **biased here** (1.6–1.8×
  heavy): the sampled expansions of a depth-first walk really are more
  expensive, and two strides disagreed;
- a leaf's measured cost depends on how much else is timed — `mg_legal` is
  **61.67 ns/call** measured alone in the pass-2 stream and **99.66 ns** with
  every leaf timed (`proof_probe@enter` moves the other way, 0.55×).

So the final design is **three levels**, each with one calibration constant:

1. **regions** (`sites` build, +0.94 s over the reference): 10 inclusive
   aggregates, from which the 9 exclusive regions are derived by subtraction.
   These are wall-clock durations and need no calibration; they sum to
   100.00 % of the wall by construction.
2. **ops** (7 `--only` builds): one leaf group per build, so the observer
   effect stays local; **exact** call counts come from the counters, not from
   sampling.
3. **tax**: an rdtsc pair with an empty body, sampled at every site (181 k
   samples per site, ≈ 0.1 s). `op = max(0, mean − tax) × calls`. Tax this
   session: enter 7.73–7.81 ns, null pass 1 7.83–8.34, null pass 2 7.62–7.68,
   replay 8.31–8.34.

`other` = wall − Σ op buckets − `outside` = **28.45 pp**, i.e. time inside the
named regions but not inside any measured call (loop control, the per-child
stores, the pass-2 cache-hit path, pipeline overlap between two adjacent
intervals). The plan's gate is measured on the region partition, where
`other` = **2.58 pp**.

## Runs (same session, reference 53.656 s)

| Run | Wall (s) | What it measures |
| --- | ---: | --- |
| reference | 53.656 | the wall (clean binary, `b9cf8919…`) |
| `sites` | 54.600 | the 10 region aggregates |
| `full` | 90.319 | every leaf (the observer-effect control) |
| `ops-movegen` | 55.147 | `mg_legal` |
| `ops-status` | 57.885 | `status`, `legality`, `mover_check` |
| `ops-board` | 66.354 | `do_move`, `undo_move`, `clone`, `key`, `mover_new`, `null_board` |
| `ops-tables` | 63.351 | `proof_probe`, `tt_probe`, `filter`, `prefetch`, `store`, `null_cache` |
| `ops-closure` | 57.371 | `closure_frame`, `closure_null_p2`, `closure_replay` |
| `ops-guide` | 58.916 | `guide_class`, `guide_eval`, `repetition`, `select` |
| `ops-alloc` | 56.922 | `alloc`, `dealloc` |

Every run: 5,803,257 expansions, 155,236,740 child evals, length 36, 5,716,874
proof entries — the plan-33 row, unchanged.

## The numbers

Nine exclusive regions (100.00 % of the wall): `Frame::enter` pass 1 **36.91
pp**, null pass 1 17.49, null pass 2 15.85, null per-child envelope 11.69,
`Frame::enter` pass 3 8.38, pass 0 + setup 4.43, start numbers 2.05, sibling
replay 0.61, `outside` 2.58. Inclusive: `Frame::enter` 51.77 pp, null replay
45.04 pp.

Per-op buckets (lower bounds, 68.97 pp in total): legal movegen **12.48 pp**,
frame closure **11.55**, status/legality 10.55, `hand_eval` 7.92, proof probe
5.20, replay closure 4.48, guide class 3.59, do/undo/clone 3.43, null pass-2
closure 2.74, select 1.87, key 1.52, store 1.18, repetition 0.67, cache 0.63,
allocator 0.59, null board 0.33, prefetch 0.18, TT probe 0.04.

Per expansion: 26.75 child evals, 53.08 legal moves (2.06 lists) + 237.07
pseudo-legal (7.53 lists), 8.17 children built, 46 probes (22.26 TT + 23.73
proof), 44.76 prefetches, 0.59 null calls (21.22 null moves), 3.68
`hand_eval`s, 8.64 allocations.

**Decision:** the largest op bucket is 12.48 pp and the free-everything op
ceiling is **3.22×** (37.0 s of 53.66 s). `T_4` needs **81.3 pp = 43.62 s**
and `T_5` **84.9 pp** — more than the whole measured op budget. The plan's stop
rule fires; the goal question goes to the owner.
