# Plan 37 measurements — item 46 (OVERHEAD-PROFILE): the 28.45 pp no counter owns

Plan: [`plan37.md`](../../docs/plans/prover/plan37.md). Report:
[`report37.md`](../../docs/plans/prover/report37.md). Host, binaries and pins:
[`env.json`](env.json). One execution session, one process at a time: the
shipped binary is never changed (no engine code, flag, default or `bench.md`
row); the `debug = 1` build is a copy of the tree **outside** the repo.

| File | Contents |
| --- | --- |
| `profile.py` | the driver: Phase 0 envelope from `measurements/plan34/census_120s.json`, `perf stat` medians, flat `perf record` symbol/srcline parsing, the four-class model and the decision |
| `profile_120s.json` | the parsed integers: stat medians, penalties, untimed-symbol table, class table, candidate list, gates |
| `latency.c` | the measured penalties (pointer chase L1/L2/L3/DRAM; predictable vs data-dependent branch) |
| `env.json` | host, binaries, perf limits, cell, gates |

## Reproduce

    cargo build --release
    python3 measurements/plan37/profile.py --run --out measurements/plan37/profile_120s.json

The `--run` path takes ≈ 12 min: 3 × `perf stat` (120 s each), 4 × flat
`perf record` on the release build, 4 × on the scratch `debug = 1` build, plus
the 120 s reference and debug control runs. Raw `perf.data` stays in `/tmp`;
only the parsed JSON is committed.

## What the profile found

- **The census `outside` is a plug, not a measured region.** The sites build
  wall is 54.600 s; the measured regions sum to 52.272 s, so the true outside
  is 2.328 s (4.34 pp), while the census reports 1.384 s (2.58 pp) by defining
  `outside = reference wall − measured regions`. The mid site leaves are
  2.735 s (5.10 pp), larger than the plug, so the per-region own-time
  subtraction gives **31.03 pp**, not the plan's 28.45 pp.
- **The untimed symbols are 29.01 % of cycles**, against the census's
  `other = 28.45 pp` (102 %). The top untimed symbols are `Frame::enter` self
  9.51 %, `null_win` self 6.56 %, `Mover::effect` 3.49 %, `populate_state`
  2.95 %, `mid` self 1.83 %, `lookup_with_source` 1.04 %, `gives_check` 0.92 %,
  `memmove` 1.10 %.
- **The top class is retired instructions, not stalls.** Whole-run cycle
  budget: retired instructions 74.3 %, branch mispredicts 7.5 %, cache misses
  18.2 %. Scaled to `other`: **1,964 ns retired / 199 ns branch / 481 ns
  cache** (2,644 ns total). The decision is **LEVER** (top class 74.3 % ≥ 25 %).
- **The candidate list** (ns/exp, mechanism, prototype cost) is in
  `profile_120s.json` §phaseB and `report37.md` §5. The largest single
  candidates are `Frame::enter` self (867 ns), `null_win` self (598 ns),
  `Mover::effect` (318 ns) and `populate_state` (269 ns).

## Limits

- `perf record -g` is blocked (`perf_event_paranoid = 2`), so the profile is
  flat symbol/srcline, not call-graph. The region cross-check is therefore
  symbol-mapped, not call-chain-mapped.
- The `debug = 1` build is 2.1 % faster than the release build in this session
  (52.606 s vs 53.721 s); the tree is identical. The line-level attribution is
  used only for the untimed/timed split, not for absolute wall.
- The class model uses a 4.0-IPC base and a derived 33.5-cycle L2-miss
  penalty; the branch (16.2) and L1/L2/L3/DRAM (4.0/10.6/37.0/197.0) penalties
  are measured by `latency.c`.
