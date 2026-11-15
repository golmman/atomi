# Report 37 — item 46 (OVERHEAD-PROFILE): the 28.45 pp is instruction-bound, and the census `outside` is a plug

Plan: [`plan37.md`](plan37.md) (item 46). Initiative:
[`initiative.md`](initiative.md), D22/D36/D40/D43–D45. Data:
[`measurements/plan37/`](../../../measurements/plan37/README.md). **No engine,
flag, default, test or `bench.md` row changed**; the `debug = 1` build is a
scratch copy under `/tmp/atomi-plan37/`; the oracle was not used.

## 1. Verdict

**LEVER.** The untimed code is **29.01 % of cycles** against the census's
`other = 28.45 pp` (102 %), and the top class is **retired instructions**:
**1,964 ns/exp = 74.3 % of `other`**, against 199 ns branch mispredicts and
481 ns cache misses. The mechanism: the untimed pass-1/null loops and their
inlined helpers are instruction-bound, not stall-bound. The largest single
candidates are `Frame::enter` self (867 ns), `null_win` self (598 ns),
`Mover::effect` (318 ns) and `populate_state` (269 ns).

Two plan defects are recorded, not fixed:

1. **Phase 0 does not reproduce `other` by per-region subtraction.** The
   census formula `wall − ops_total − outside` gives 28.45 pp by construction,
   but `region total − leaves inside it` gives **31.03 pp**. The gap is the
   mid site leaves (2.735 s), counted in `ops_total` but not in the named
   regions.
2. **The census `outside` is a plug, not a measured region.** The sites build
   wall is 54.600 s; the measured regions sum to 52.272 s, so the true outside
   is 2.328 s (4.34 pp), not 1.384 s (2.58 pp). The mid site leaves (2.735 s)
   exceed the plug, so the outside own time is negative. G-46d passes on the
   measured partition; the `outside` mismatch is the plug.

| Gate | Bar | Result |
| --- | --- | --- |
| G-46a envelope | `other` 28.45 pp ± 0.5, `outside` 2.58 pp | **PARTIAL** — the formula reproduces both; the per-region subtraction gives 31.03 pp (see §3) |
| G-46b tree | 5,803,257 / 155,236,740 / 36 | **PASS** — every profiled run, release and `debug = 1` |
| G-46c coverage | four classes ≥ 80 % of 2,631 ns | **PASS (global)** — untimed 29.01 % vs 28.45 %; class table 2,644 ns; the split is global, not per-region |
| G-46d agreement | sampled region shares ±20 % of census | **PASS** on the measured partition (`enter` −19.7 %, `null` −13.1 %, enter+null+siblings −15.5 %); `outside` +136 % is the plug |
| G-46e decision | LEVER / NO LEVER / INCONCLUSIVE | **LEVER** — top class 74.3 % ≥ 25 %, mechanism stated, candidate list §5 |

## 2. Method (one process at a time)

- **Reference:** `target/release/atomi prove "<deep-win>" --guide hand
  --memory 192 --seconds 120 --seed 0`, wall **53.721 s** (single run) and
  **53.932 s** (median of 3 `perf stat` runs). Tree identical.
- **`perf stat`** (3 repeats, medians): cycles **2.540e11**, instructions
  **7.547e11**, branches **1.143e11**, branch-misses **1.179e9**,
  L1-dcache-load-misses **2.039e9**, cache-misses **1.076e9**. Effective
  **4.709 GHz**, IPC **2.97**.
- **`perf record`** flat, `-F 999`, for cycles, branch-misses,
  L1-dcache-load-misses and cache-misses. **Call-graph is blocked** by
  `perf_event_paranoid = 2` (`-g` fails to open the event), so the profile is
  flat symbol/srcline. This is a deviation from plan37 Phase A.2; the untimed
  symbols are still separable by symbol.
- **Scratch `debug = 1` build** (`git archive HEAD` + `[profile.release]
  debug = 1`): wall **52.606 s** vs release 53.721 s (−2.1 %), tree identical.
  Used only for the line-level timed/untimed split.
- **Penalties measured** by [`latency.c`](../../../measurements/plan37/latency.c):
  branch mispredict **16.2 cycles** (8.143 cycles/iter ÷ 0.502 misses/iter);
  pointer-chase L1 **4.0**, L2 **10.6**, L3 **37.0**, DRAM **197.0** cycles.
  The L2-miss penalty is **33.5 cycles**, derived from the cycle budget
  (`C − I/4 − branch − L1-hit`), not assumed.

## 3. Phase 0 — the envelope, and why it does not close

The census's 9 exclusive regions sum to 100.00 % only because `outside` is
defined as `wall − measured regions`. The measured regions are 52.272 s of the
54.600 s sites wall; the true outside is **2.328 s (4.34 pp)**, not 1.384 s
(2.58 pp). The mid site leaves are **2.735 s (5.10 pp)**, so:

| Quantity | s | pp |
| --- | ---: | ---: |
| named regions total | 52.272 | 97.42 |
| named regions' timed leaves | 34.270 | 63.87 |
| named regions' own time | 18.002 | 33.55 |
| outside plug | 1.384 | 2.58 |
| outside own (plug − mid leaves) | **−1.351** | **−2.52** |
| per-region own sum | **16.651** | **31.03** |
| census `other` (formula) | 15.267 | 28.45 |

So the plan's Phase 0 check is an arithmetic identity of the formula, not a
per-region check. The site-level own times are: enter union 9.737 s, null
pass 1 5.170, null pass 2 1.599, null envelope 1.169, siblings 0.328, outside
plug −1.351; the negative outside is the signature of the plug.

## 4. Phase A — the profile

**The untimed symbols are 29.01 % of cycles**, against `other = 28.45 pp`
(102 %). The top untimed symbols (cycles %, ns/exp in `other`):

| Symbol | cycles % | ns/exp | Region |
| --- | ---: | ---: | --- |
| `Frame::enter` self | 9.51 | 867 | enter pass 1 (plus inlined helpers) |
| `null_win` self | 6.56 | 598 | null pass 1/2 |
| `Mover::effect` | 3.49 | 318 | enter pass 1 |
| `populate_state` | 2.95 | 269 | enter pass 0 |
| `Engine::mid` self | 1.83 | 167 | outside |
| `__memmove_avx_unaligned_erms` | 1.10 | 100 | enter/null Vec copies |
| `Tables::lookup_with_source` | 1.04 | 95 | enter pass 3 / null / replay |
| `Mover::gives_check` | 0.92 | 84 | enter pass 1 |
| `castling_rights_after` | 0.63 | 57 | enter pass 1 |
| `legal_ep_square` | 0.51 | 47 | enter pass 1 |
| `blast_casualties` | 0.47 | 43 | enter pass 1 |

The line-level `debug = 1` profile confirms the split: `frame.rs` own lines
1.94 %, `null.rs` 0.87 %, `mover.rs` 0.73 %, `value.rs` 0.71 %, `search/mod.rs`
2.11 %; the rest are inlined dependency/std lines. `Frame::enter` self is an
**upper bound** on the removable loop body: it includes inlined timed leaves
(`threat.rs`, movegen/status), which the line-level report attributes to their
own files. The pure `frame.rs` loop lines are 1.94 % of cycles = **180 ns**.

**Class model.** Whole-run cycle budget with a 4.0-IPC base:

| Class | whole run | `other` (ns/exp) | share of `other` |
| --- | ---: | ---: | ---: |
| (i) retired instructions | 74.29 % | **1,964** | **74.3 %** |
| (ii) branch mispredicts | 7.52 % | 199 | 7.5 % |
| (iii) cache/DRAM misses | 18.19 % | 481 | 18.2 % |
| (iv) compiler artifacts | 0.00 % | 0 | 0.0 % |
| **sum** | 100.00 % | **2,644** | 100.0 % |

The class fractions are applied to the untimed symbols' 29.01 % of cycles and
scaled to the census's 28.45 pp (this session's 53.932 s wall gives 2,644 ns;
the plan's 2,631 ns is the plan34 wall). (i) is base instruction retirement;
(ii) and (iii) are the stall components of the same cycles; split is global, not
per-region (the per-region own times are in §3). `memmove`
(1.10 %) is inside (i) and listed separately as a removable copy.

**Cross-check (G-46d).** Symbol-mapped shares: `enter` **41.6 pp** vs 51.77
(−19.7 %), `null` **39.1 pp** vs 45.04 (−13.1 %), measured partition
(enter+null+siblings) **81.0 pp** vs 95.8 (−15.5 %), all within ±20 %.
`outside` **6.1 pp** vs 2.58 (+136 %) is the plug, not a measured region; the
mid site leaves are 5.10 pp. The profile is comparable for the measured
regions.

## 5. Phase B — the decision and the candidate list

**LEVER** under the plan's rule: the top class is 74.3 % of `other` (≥ 25 %)
and the mechanism is stated. The candidates, with the prototype cost the next
plan session would need:

| Candidate | ns/exp | Mechanism | Prototype |
| --- | ---: | --- | --- |
| `Frame::enter` self (pass-1 loop) | 867 | per-child stores, Vec pushes, bounds checks | hoist bounds checks / preallocate, 1–2 days |
| `null_win` self (null pass 1/2) | 598 | key/status/proof scan loop control | fuse pass 1/2 or preallocate, 1–2 days |
| `Mover::effect` | 318 | per-move `needs_board` + `gives_check` | skip hash for built children / cache effect, 1 day |
| `populate_state` | 269 | per-node state population | reuse the parent's child state, 1–2 days |
| `Engine::mid` self | 167 | MID value algebra / select wrappers | inline/precompute, 1 day |
| `memmove` | 100 | Vec growth/copies | stack arrays, 0.5 day |
| `lookup_with_source` | 95 | lookup orchestration | fuse TT/proof/filter probes, 1 day |
| `gives_check` | 84 | per-move check test | bitboard precompute, 1 day |

The top two are upper bounds (inlined timed leaves); the cleanest single
mechanism is `populate_state` (269 ns, 2.95 pp, ≈1.6 s of the 53.9 s wall) or
`Mover::effect` (318 ns, 3.49 pp, ≈1.9 s). Neither is a 5.34× lever; together
with the loop bodies the class is ≈16.5 pp (≈1.20×). Phase C was not run: the
plan makes it optional, and the candidate sizes above are the profile's
output. A scratch prototype is the next plan session's job.

## 6. Problems met (method findings)

1. **`perf record -g` is blocked** by `perf_event_paranoid = 2` (`Failure to
   open any events for recording`). Flat `perf record` works, so the profile
   is symbol/srcline. The plan's risk (i) said "if sampling is blocked the
   verdict is INCONCLUSIVE"; sampling is not blocked, only call-graph.
2. **The census `outside` is a plug** (`wall − measured regions`); it absorbs
   the +0.944 s sites-build overhead. This is the largest validation finding
   of the session (§3).
3. **The census per-op buckets are not line-level consistent.** `guide_eval`
   is 4.247 s (7.92 pp), but the line-level `hand_eval` symbol is 3.70 % of
   cycles (≈2.0 s) and `eval.rs` is 1.29 % (≈0.7 s). The total is consistent
   (68.97 pp vs 70.99 % of cycles), but individual buckets are redistributed
   by the rdtsc boundaries; the class model uses the symbol/srcline split.
4. **The `debug = 1` build is 2.1 % faster** (52.606 s vs 53.721 s), tree
   identical; no absolute wall is taken from it.
5. **The plan says "10 regions"; the census has 9 exclusive regions.** The
   per-region table is at site granularity (6 buckets) because the enter site
   spans four regions and the census leaves cannot be split by region without
   a chained-marker build.

## 7. Missing tests / open parts

- **No call-graph profile.** A host with `perf_event_paranoid ≤ 1` would let
  `-g --call-graph dwarf` attribute the shared symbols (`ProofTable::get`,
  movegen, `do_move`) to their call sites and remove G-46d's symbol-mapping
  approximation.
- **No Phase C prototype.** The candidate sizes are profile estimates; the
  `Frame::enter`/`null_win` figures include inlined timed leaves and are upper
  bounds. The next plan session should prototype `populate_state` reuse or the
  `Mover::effect` reorder with a tree-identity gate.
- **The 4.0-IPC base is an assumption.** Branch and cache penalties are
  measured, but base instruction retirement is not; `stalled-cycles-backend`
  (not supported here) would pin it.
- **The census per-op buckets need a line-level audit** before per-region
  subtraction; `guide_eval` 2.1× is the first example.

## 8. Next steps

1. **Owner ruling on the Phase 0/G-46d defect.** The literal G-46d ("else
   INCONCLUSIVE") would make the verdict INCONCLUSIVE because of the `outside`
   plug. I report LEVER on the measured regions and flag the conflict; the
   owner should decide whether the plug is a census defect or a reason to
   re-scope.
2. **If LEVER is accepted:** scope the top candidate next session with a
   scratch prototype and a tree-identity gate. The cheapest decisive
   experiment is `populate_state` reuse (269 ns, 2.95 pp) or the
   `Mover::effect` reorder (318 ns, 3.49 pp); both are tree-neutral in
   principle and measurable in one session.
3. **If the owner prefers the literal gate:** INCONCLUSIVE; the next step is a
   chained-marker census (one rdtsc read per region boundary) or a
   call-graph host.
4. **Do not re-run the census for this question.** The untimed symbols already
   cover 102 % of `other`; the remaining uncertainty is the inlined timed
   leaves, which a call-graph profile would resolve, not a longer census.
