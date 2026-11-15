# Report 34 — item 42 (EXPANSION-CENSUS): 9.25 µs per expansion, and `T_4` needs 81.3 pp of it

Plan: [`plan34.md`](plan34.md) (item 42, proposed; the kill review is §1–§4 of
the plan and is not re-argued here). Initiative: [`initiative.md`](initiative.md),
D30/D36/D40/D43–D45. Measurements:
[`measurements/plan34/`](measurements/plan34/README.md). **No engine
change, no flag, no default, no `bench.md` row**: the census is read-only and
every number comes from scratch builds outside the repo (GPL boundary: provari
was not run, read or linked this session).

## 1. Verdict

**The census is exhaustive and the stop rule fires.** Nine exclusive regions
sum to **100.00 %** of the reference wall by construction, the only residual
(`outside`, i.e. the MID loop and everything not inside a frame) is **2.58 pp
< 10 pp**, and the per-op census below it accounts for **68.97 pp** at
**37.0 s of 53.66 s**. The tree is identical in all **10 runs**
(5,803,257 expansions, 155,236,740 child evals, length 36).

| Gate | Bar | Result |
| --- | --- | --- |
| G-34a tree | 5,803,257 expansions, seed 0, 120 s | **pass** — reference + 9 instrumented runs, all 5,803,257 / 155,236,740 / 36 |
| G-34b buckets sum | wall ±2 pp | **pass** — the 9-region partition is 100.00 % (independent instrumented wall 54.600 s = +1.76 % of the reference, so no bucket carries the harness) |
| G-34c `other` | < 10 pp | **pass** — 2.58 pp (`outside`); the op census covers 68.97 pp, the remaining 28.45 pp is *inside* the named regions (see §5.3) |
| stop rule | no bucket ≥ 15 pp **or** removable union < 5.34× | **triggered** — largest op bucket **12.48 pp**, largest region **36.91 pp** is a loop not a lever; the free-everything op ceiling is **3.22×** |

**Decision numbers.** `T_4` (5.34× on `deep-win`) requires deleting **≥ 81.3 pp
= 43.62 s** of the wall. The **entire** measured op budget is 37.0 s (68.97 pp
⇒ **3.22×** if every measured call were free). `T_5` (6.61×) requires 84.9 pp.
So no arrangement of the measured per-node work reaches `T_4`: per plan34 the
goal question goes to the owner (options B/C/D) and **no plan 35+ is written in
the micro mould**.

## 2. Method (three kinds of run, one process at a time)

Scratch builds from [`instrument.py`](measurements/plan34/instrument.py)
(`src/probe.rs` + 78 `timed!` sites: 67 leaves, 11 region aggregates; nothing enters the repo):

- **reference** — the shipped binary, `--guide hand --memory 192 --seconds 120
  --seed 0`, wall **53.656 s** (9,245.8 ns/expansion);
- **sites** — leaf buckets off, 10 inclusive region aggregates on
  (`+0.94 s`, +1.76 %): the *measured*, exhaustive region totals;
- **ops** — one build per leaf group (7), only that group timed
  (+1.5…+12.7 s), plus the in-situ **tax** (an empty interval, 181 k samples
  per site): the per-op buckets with exact call counts;
- **full** — every leaf timed (`+36.7 s`): the observer-effect control.

Calibration: TSC 3.400037 GHz (`tsc_hz.c`, three 1 s sleeps). Tax of one
measured interval **7.62–8.34 ns** by site; every op is reported as
`max(0, mean − tax) × calls`, i.e. a lower bound.

## 3. The regions (exhaustive, measured)

| Region | s | pp | ns/exp |
| --- | ---: | ---: | ---: |
| `Frame::enter` pass 0 + setup (status, entry movegen, guide ctx, Vecs) | 2.379 | 4.43 | 410 |
| `Frame::enter` pass 1 (per-move loop) | **19.804** | **36.91** | 3,413 |
| `Frame::enter` pass 3 (repetition + lookup) | 4.495 | 8.38 | 775 |
| `Frame::enter` start numbers + frame build | 1.102 | 2.05 | 190 |
| null `pass 1` (key + status + proof scan, 123.2 M moves) | 9.386 | 17.49 | 1,617 |
| null `pass 2` loop (99.9 M exposures, 66.4 M uncached) | 8.506 | 15.85 | 1,466 |
| null-replay per-child envelope (clone/do + replay call) | 6.272 | 11.69 | 1,081 |
| sibling replay | 0.328 | 0.61 | 57 |
| **`outside`** (MID loop, deadline checks, value algebra) | **1.384** | **2.58** | 239 |
| **sum** | **53.656** | **100.00** | 9,246 |

Inclusive: `Frame::enter` **27.780 s (51.77 pp)**, null replay **24.164 s
(45.04 pp)** ⊃ `null_win` 17.892 s (33.35 pp) ⊃ pass 2 8.506 s, `simulate`
7.674 s (14.30 pp). Reader's summary: **the wall is the expansion body** —
`Frame::enter` plus the null replay are **96.8 pp** of it, and each is one flat
loop, not a hotspot.

## 4. The op buckets (per-op, lower bound)

| Bucket | s | pp | calls | ns/call |
| --- | ---: | ---: | ---: | ---: |
| movegen (legal) | **6.694** | **12.48** | 78.4 M | 85 |
| closure: frame pass-1 settle+screen | **6.196** | **11.55** | 164.3 M | 38 |
| legality/status/terminality | 5.090 | 9.49 | 266.1 M | 19 |
| guide `hand_eval` | 4.247 | 7.92 | 21.4 M | 199 |
| proof probe | 2.789 | 5.20 | 269.1 M | 10 |
| closure: replay/simulation | 2.405 | 4.48 | 8.2 M | 292 |
| guide class + start numbers | 1.925 | 3.59 | 365.3 M | 5 |
| do_move/undo_move/clone | 1.843 | 3.43 | 747.4 M | 2.5 |
| closure: null pass-2 reply test | 1.470 | 2.74 | 80.5 M | 18 |
| MID `select`/`node_value` | 1.005 | 1.87 | 23.2 M | 43 |
| key/rule50 | 0.818 | 1.52 | 428.0 M | 1.9 |
| table store | 0.633 | 1.18 | 17.4 M | 36 |
| move effect (needs-board/check test) | 0.568 | 1.06 | 155.2 M | 3.7 |
| repetition scan | 0.362 | 0.67 | 140.5 M | 2.6 |
| exposed-limit cache | 0.336 | 0.63 | 166.3 M | 2.0 |
| allocator (alloc+dealloc) | 0.319 | 0.59 | 97.5 M | 3.3 |
| null board (`null_for`) | 0.179 | 0.33 | 3.4 M | 52 |
| prefetch hint | 0.097 | 0.18 | 259.7 M | 0.4 |
| TT probe | 0.023 | 0.04 | 129.2 M | 0.2 |
| `Mover::new` / filter probe | 0.005 | 0.01 | 18.2 M | 0.3 |
| **total** | **37.005** | **68.97** | | |

By site: `enter` 18.04 s, null pass 2 6.91, replay 5.10, null pass 1 4.22,
MID 2.74. **No bucket reaches 15 pp**; the largest is the legal movegen at
12.48 pp (≤ 1.14× if free).

**Counts per expansion** (exact, whole run): 26.75 child evals; **53.08 legal
moves in 2.06 lists** (entry 5.80 M + null board 3.43 M + replay 2.73 M lists;
the pass-2 reply stream is not a list — it is counted by its 31.1 `do_move`s
per expansion) and **237.07 pseudo-legal moves in 7.53 lists** (the frame
closure, its own `MoveList` per call);
**290.2 moves and 9.59 lists generated per expansion**; 8.17 children built;
**22.26 TT probes + 23.73 proof probes** (46 probes, 90.5 M/s) + 2.14 filter
probes + 44.76 prefetches; 0.59 null calls (21.22 null moves); 3.68
`hand_eval`s; 0.82 replay calls; 2.99 stores; 8.64 allocations / 8.15
deallocations.

## 5. What the numbers say

1. **The volume is real, the time is not where 6.13× needs it.** 290 moves and
   9.6 lists per expansion, 46 probes, 8 built children — yet the movegen is
   12.5 pp, because the two biggest generators are already bit tests (plan31
   F1/F3): the 237-move pseudo stream is *inside* the closure buckets (it is
   not timed separately), and the pass-2 stream is 66.4 M *lazy* movegens at
   61.7 ns each. Only 37.0 s (69 pp) of a 53.7 s wall is attributable to *any* per-op
   call.
2. **Two site regions are 96.8 pp** (`Frame::enter` 51.77 and the null
   replay 45.04, already exclusive; their two pass-1 loops alone are 36.9 and
   17.5), and 28.45 pp of their bodies is not
   inside any measured call: loop control, the `Child`/`StartInput` stores,
   pipeline overlap between two adjacent intervals, the pass-2 cache-hit path
   and the pass-3 loop's non-lookup tail. That is why the op census is a
   *lower* bound; the region totals are not.
3. **The removable union.** A tree-neutral rewrite can delete: the guide number
   stack (hand_eval 7.92 + class/start 3.59 + `Mover::new`/repetition 0.68 ≈
   **12.2 pp**), a leaner probe schedule (half of 5.20 ≈ 2.6 pp), the allocator
   (0.59 pp) and the prefetch hint (0.18) — **≈ 15.6 pp ⇒ 1.18×**. Adding the
   frame's one-ply closure (11.55 pp, a D21 *tree* change) and the replay
   closure (4.48) ⇒ **≈ 31.6 pp ⇒ 1.46×**. Even D45's "node-body rewrite" with
   every measured call free is 3.22×.
4. **Consequence for `T_4`/`T_5`.** `T_4` needs 43.62 s of the 53.66 s wall to
   disappear — 6.6 s *more* than the whole op census and 12 pp more once the
   untimed scaffolding is counted. `T_5` needs 45.6 s. The 6.13× per-unit gap
   is therefore not a sum of removable buckets; it is the shape of the
   expansion (one full pass over the moves plus a per-child classification
   pass at entry, and a null scan per defender node), which is a formulation
   cost, not a hot line.

## 6. Problems met (method findings worth keeping)

1. **A single all-leaf census cannot be trusted.** With every leaf timed the
   run inflates 53.66 → 90.32 s and the inclusive aggregates read 42.5 s
   (`enter`) / 42.9 s (null replay) — 1.6–1.8× their true 27.8 / 24.2 s,
   because an aggregate's wall-clock window contains the hours of
   instrumentation *inside* it. The box total was also self-inconsistent:
   enter + null replay exceeded the engine's own wall.
2. **Per-expansion sampling is biased here.** Sampling one expansion in 32
   (unbiased by index) still gave 1.6–1.8× heavy means: the sampled
   expansions *are* heavier (longer frames), so `k`-in-`N` sampling of a
   depth-first walk is not an unbiased estimator of per-expansion cost. Two
   independent strides disagreed; abandoned in favour of the isolation builds.
3. **The observer effect is per-interval, not proportional.** `mg_legal`
   measures **61.67 ns** alone in the pass-2 stream and **99.66 ns** with every
   leaf timed (1.62×); other leaves move the other way (`proof_probe@enter`
   0.55×, `guide_eval` 0.98×). Hence one build per group and the additive tax;
   the remaining 28.45 pp is the honest residual of that model.
4. **plan33's pass-1 profile does not reproduce as a pass total.** G-40f put
   the whole `null_win` pass 1 at 6.996 s (13.2 pp); the region measurement puts
   pass 1 at **9.386 s (17.49 pp)**. G-40f measured its own sub-phases
   (status/do/key/probe), each of which excludes the between-interval time
   found here; the plan-33 *shares* stand, the sum was a lower bound.
5. The committed drivers reproduce the final design in one command.

## 7. Missing tests / open parts

- **No independent check of the region boundaries.** They rest on the sum
  identity (regions = wall) plus the aggregate pairs (`A_enter` ⊃ its three
  loops, `A_null_replay` ⊃ `A_null_win` ⊃ pass 2); a second measurement of one
  region (e.g. `A_pass3` by a single-leaf build) is not built.
- **The 28.45 pp residual is not split further.** It is inside named regions
  but not inside any call timer; splitting it needs a chained-marker scheme
  (one read per boundary) that would trade accuracy for resolution.
- **The tax is one constant per site.** Cheap ops (< 8 ns) clamp to 0, so
  `TT probe`, `filter probe` and the prefetch hint are lower bounds of ~0.
- `allocs`/`deallocs` count `realloc` as an allocation.

## 8. Next steps

1. **Owner decision on plan34's B/C/D, with this number.** The census supports
   the plan's own reading: the missing multiple is implementation *volume in
   the expansion shape* (two passes at entry, one null scan per defender
   node), so a node-body rewrite (B) is the only in-base option — but its
   *measured* prize is 15.6 pp tree-neutral / ≈31.6 pp with the closures
   (1.18× / 1.46×), not 2–4×: B's premise ("the gap is volume") is now priced
   and mostly falsified. C/D are the honest alternatives.
2. **If B is taken anyway**, the ordering by measured size is: the guide number
   stack (hand_eval 7.92 pp + class/start 3.59 pp) → the frame closure
   (11.55 pp, tree-changing) → the pass-3 lookup schedule (8.38 pp region).
   None reaches 15 pp alone; together ≈ 31.6 pp ⇒ 1.46×.
3. **Do not write a `prover` micro-plan** (plan34's own rule): no bucket is
   ≥ 15 pp and no combination of them reaches 5.34×.
4. `analysis2` §5 P2's "~54 % is movegen + legality + terminality" is
   superseded: measured **23.0 pp** for the three buckets together (12.48 +
   9.49 + 1.06 ⇒ ≤ 1.30×) — the rest of the wall is the guide's own passes and the
   inlined expansion body.
