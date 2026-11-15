# Report 35 — item 43 (SHAPE-DIFF): the reference's body at our rates is 1,786 ns — `T_4` is not funded, by 3.6 %

Plan: [`plan35.md`](plan35.md) (item 43). Initiative: [`initiative.md`](initiative.md),
D36/D40/D43–D46, plan34's B/C/D. Data:
[`measurements/plan35/`](measurements/plan35/README.md). **No engine
change, no flag, no default, no `bench.md` row**: counts only, the reference
read and patched **only** in `/tmp` (GPL boundary, `PROVENANCE.md` untouched),
and no `ns` that prices anything is the reference's — every rate is ours
(report34 §4 verbatim).

## 1. Verdict

Per node body, priced at our per-op lower bounds, the reference does
**`M` = 1,786 ns** of work on `deep-win` (`core`: 2,117 ns). `T_4` allows
**1,723 ns/exp** (10 s over our 5,803,257 expansions). Under §1's
pre-registered rule the verdict is **NOT FUNDED**, shortfall `M/1,723` =
**1.036×** (`core` 1.229×); addressable pp `(9,246 − 1,786)/92.46` = **80.69
pp** against the 81.3 pp needed (short 0.6 pp = 56 ns/exp).

| Gate | Bar | Result |
| --- | --- | --- |
| G-35a coverage | unmapped share ≤ 20 % of our op budget; `body` vs reported `nodes` recorded | **pass** — unmapped (folded leaves) 3.96 pp = **5.74 %**; `body` 2,099,100 vs `nodes` 3,024,117 = 1.4407 (926,564 replay steps + 1,547 early returns before the node tick) |
| G-35b determinism | 3 runs → identical counts | **pass** — byte-identical counter lines, 3/3 on both cells |
| G-35c arithmetic | our own counts + our rates reproduce 37.005 s ± 2 % | **pass** — **37.004 s**, −0.0027 % (the census's own rounding; 21/21 buckets at 0.0 %) |
| G-35d decision | `M` ≤ 1,723 ns | **NOT FUNDED** — `M` 1,785.9, band ±15 % [1,518, 2,054] ⇒ **the verdict flips inside the band** |

**The cross-check that matters** (not an input; it prices nothing): the
unpatched build at the pin reports 4.40–4.44 s for 2,099,100 bodies =
**2.10 µs/body** of its own wall (four runs, ±1 %). The mapping recovers
**84–85 %** of it (≈320 ns of its scaffolding against our 2,869 ns). Two
readings follow, and the second is the item's real answer:

1. our rates are a genuine lower bound per counted call, and the mapping is
   complete — it explains the reference's body to within 15 % from counts
   alone;
2. `T_4`'s 1,723 ns/exp is **below the reference's own per-body wall**: a
   perfect port of its body shape, at its own speed, on our tree
   (5,803,257 × 2.10 µs) takes **≈12.2 s = 4.4×**, not 5.34×. At `M`
   (a body 15 % cheaper than the reference's measured one) it takes 10.36 s =
   5.15×.

So the reshape is **not funded on the point estimate, and the point estimate
sits on the method's resolution limit** (3.6 % out of a ±15 % band). What the
measurement settles: the reshape is not the 2–4× plan34 B hypothesised — it is
a 1.03–1.23× miss on `T_4`, and it needs beats on the reference's own body to
land.

## 2. Method

A copy of the pin tree in `/tmp`, patched by a script that also lives only in
`/tmp` (`counts.py`, sha in `env.json`): 39 `Cell<u64>` counters, no clock, no
atomics, one thread, bumped at 35 call sites and dumped as one JSON line on
stderr. Counting does not change the search: both cells report the same
`nodes`, mate length and wall as the unpatched build. Runs: `deep-win`
`--guide hand --memory 192 --seconds 120` (a completed run, 4.5 s) and `core`
@10 s, 3 runs each + 1 unpatched run each for tree identity.

**The denominator is the body**, not the reported `nodes`: one counter at the
node-body entry. On `deep-win` the reference reports 3,024,117 nodes for
2,099,100 bodies — the plan's warning was right and the reported count would
have understated every rate by 1.44×.

**Mapping rule** (fixed before the arithmetic): each of our census leaves takes
one reference counter, in that leaf's own unit (one list produced, one child
decided, one probe, one per-move screen, one child built), priced at the
census bucket's `s / calls`. A leaf whose job the reference does not do at all
is **folded**: no `M`, whole bucket addressable, counted in the unmapped share.
A counter that rides inside a priced family is listed as `inside` and never
priced twice. G-35c is an identity under this rule, and its content is
unit-consistency: no bucket dropped, none double-counted, all 21 at their
report34 §4 rates.

## 3. The counts (per body, `deep-win`; ours per expansion)

| | there / body | ours / exp | ratio |
| --- | ---: | ---: | ---: |
| list generations | 2.391 | 13.508 | 5.65× |
| moves yielded per list | 31.0 pseudo (74.0/body) | 31.5 pseudo (237.1/exp) | — |
| legal moves | 36.4 | 53.1 | 1.46× |
| per-move legality screens | 59.2 | 26.8 | 0.45× |
| children decided | 18.20 | 8.17 built / 26.75 evaluated | — |
| proof-store probes | 5.22 | 23.73 | 4.5× |
| small-table lookups | 3.76 | 22.26 | 5.9× |
| filter bit tests | 45.5 | 2.14 | 0.05× |
| child boards built | 3.56 | 128.79 do/undo/clone | 36× |
| table stores | 1.00 | 2.99 | 3.0× |
| hand evaluations | 1.00 | 3.68 | 3.7× |
| null uses / null boards | 0.217 | 0.591 | 2.7× |
| replay steps (its own node count) | 0.441 | 0.819 | 1.9× |
| number combinations | 2.029 | 4.00 | 2.0× |
| per-child tests | exposure 21.7, check 15.8, blast 19.1, threat 4.4 per child | inside the closure leaves | — |

Its body is **2.4× fewer nodes** than ours on `deep-win` (2,099,100 bodies vs
5,803,257 expansions) and cheaper per node in every volume we own, but it
screens more per move (0.45×) and reads its filter far more often (0.05×).

## 4. The mapping and `M` (`deep-win`; `ns` = `p_ops × ns/op`)

| leaf (ours) | ours/exp | ns/op | p/body | ratio | ns | addr pp |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| movegen (legal) | 13.51 | 85.39 | 2.391 | 5.65 | 204.2 | 10.27 |
| closure: frame pass-1 settle+screen | 28.31 | 37.71 | 18.201 | 1.56 | **686.4** | 4.12 |
| legality/status/terminality | 45.86 | 19.13 | 0.493 | 93.0 | 9.4 | 9.39 |
| guide hand_eval | 3.68 | 198.65 | 1.000 | 3.69 | 198.6 | 5.77 |
| proof probe | 46.37 | 10.36 | 5.220 | 8.88 | 54.1 | 4.61 |
| closure: replay/simulation | 1.42 | 291.84 | 0.441 | 3.22 | 128.8 | 3.09 |
| guide class + start numbers | 62.95 | 5.27 | 1.000 | 62.97 | 5.3 | 3.53 |
| do_move/undo_move/clone | 128.79 | 2.47 | 3.559 | 36.2 | 8.8 | 3.34 |
| closure: null pass-2 reply test | 13.88 | 18.26 | — | ∞ | 0.0 | 2.74 |
| MID select/node_value | 4.00 | 43.30 | 2.029 | 1.97 | 87.9 | 0.92 |
| key/rule50 | 73.76 | 1.91 | 40.047 | 1.84 | 76.5 | 0.69 |
| table store | 2.99 | 36.47 | 0.997 | 3.00 | 36.4 | 0.79 |
| needs-board/check test | 26.75 | 3.66 | 59.235 | **0.45** | **216.7** | 0.00 |
| repetition scan | 24.21 | 2.58 | 17.634 | 1.37 | 45.4 | 0.18 |
| exposed-limit cache | 28.65 | 2.02 | — | ∞ | 0.0 | 0.63 |
| allocator | 16.80 | 3.27 | — | ∞ | 0.0 | 0.59 |
| null board | 0.59 | 52.16 | 0.217 | 2.73 | 11.3 | 0.21 |
| prefetch hint | 44.76 | 0.37 | 18.271 | 2.45 | 6.8 | 0.11 |
| TT probe | 22.26 | 0.18 | 3.757 | 5.93 | 0.7 | 0.03 |
| Mover::new | 1.00 | 0.52 | 2.391 | **0.42** | 1.2 | 0.00 |
| filter probe | 2.14 | 0.16 | 45.518 | **0.05** | 7.3 | 0.00 |
| **`M`** | | | | | **1,785.9** | 51.01 |

Three leaves run the *other* way (ratio < 1): the per-move screen, the filter
bit test and the mover setup — the reference does more of these than we do, so
they are not addressable at all and `M` prices them above our pp. The
leaf-wise addressable column (51.01 pp) is the per-leaf share of our volume the
reference does not have; §1's addressable (**80.69 pp**) is the whole wall
minus `M`, i.e. what a body costing exactly `M` would delete. Both are below
81.3 pp, by 0.6 and 30 pp respectively; the verdict uses the first.

## 5. What the numbers say

1. **The sign is set by one rate: the per-child decide.** Our
   `closure: frame pass-1` is 686 ns of `M` (38 %), priced at 37.71 ns per
   child from 18.20 children/body. Price that child at our *status* rate
   (19.13) instead and `M` = **1,448** (FUNDED, 275 ns headroom); at
   settle+status (56.84) and `M` = **2,134** (NOT FUNDED by 24 %). A count-only
   pricing cannot close a 4 % question whose answer moves ±20 % with one
   mapping choice — that is the method finding, and why the band is reported
   beside the verdict. The plan's expected reading (4.57 M nodes at 1.36 µs ⇒
   FUNDED) prices the default guide's 6.2 s; at this cell's `--guide hand` a
   body is ≈2.10 µs.
2. **`T_4` asks for less than the reference's body.** 1,723 ns/exp is 82 % of
   its ≈2.10 µs/body, and we carry 2.76× its node count (2,099,100 →
   5,803,257): a tree-neutral reshape must win per node *and* live with our
   tree.
3. **Where the addressable pp really are**: movegen **10.27 pp** (5.65× its
   list volume), status/terminality **9.39** (93× its query volume), hand_eval
   5.77, proof probe 4.61, guide class 3.53, do_move/clone 3.34, replay 3.09,
   null pass-2 2.74, plus 1.4 pp of cache/allocator/filter. The 12.2 pp "guide
   number stack" of report34 §5.3 is **9.5 pp** here, because the reference
   still evaluates once per body (1.00 × 199 ns = 2.15 pp): the stack's *per
   move* part is what is absent, not the guide.
4. **Two plan slips, recorded, neither changes a verdict.** (a) `R` =
   16,651 ns/exp does not correspond to 31.03 pp: 31.03 pp of the 9,246 ns
   wall is **2,869 ns/exp**; 16,651 ns would be 180 pp. Reported both readings
   of the plan's formula: `M + R` = **4,655** (constant) / **2,590** (scaled,
   `M·1.45`); the literal `M + 16,651` = 18,437 ns exceeds the wall and cannot
   be read as ns/exp. (b) "≳ 20 moves ⇒ movegen alone eats the budget" prices
   a *move*; report34's 85 ns is per **list-generation call** (26 legal moves
   per list here), so 20 moves is 0.76 calls = 65 ns.
5. **`T_5` (6.61× geomean) is further out of reach than `T_4`** under the same
   arithmetic, and no cell of this measurement points at a missing lever:
   every family is counted, and the largest one (`movegen`) is already a bit
   test loop in both engines.

## 6. Problems met

1. **The reported node count is not the body count** (1.44× on `deep-win`): the
   reference counts its replay steps (926,564) as nodes. Every rate would have
   been 30 % low; the plan's denominator warning was decisive.
2. **A fixed-size list's capacity read as its length**: the first build counted
   the list's 256-slot capacity, giving 766 "moves" per body. Caught because
   every list came out exactly 256 long; the fixed counter gives 31.0 per
   list.
3. **`cannot_win` never fires** in either cell (0 calls): the material test is
   live code, not a cost, on these positions.
4. **The mapping has one irreducible judgement call** (besides the units): what
   counts as "one child decided" against our per-child and per-move closure
   calls. §5.1 prices its cost.

## 7. Missing tests / open parts

- **No census of our `core`**: its ratios are not shown, and its `M` uses the
  `deep-win` rates (2,117 ns/body; its own unpatched body is 2.60 µs, so the
  mapping recovers 81 % there too — the same story).
- **`R`'s constancy is assumed**: the 2,869 ns/exp outside every call is ours,
  the reference's is 312 ns; a reshape inherits neither, so `M + R` is a risk
  reading only.
- **The per-child decide is priced, not measured.** The cheapest decisive
  experiment is a one-day spike rewriting only our per-child decide to the
  reference's shape (bit tests on the move effect, no board build), timed on
  our own tree — no reference timing needed.

## 8. Next steps

1. **Owner: plan34's C or D.** Item 43's answer is NOT FUNDED (1.036×), the
   headroom for un-timed scaffolding is **−63 ns**, and the reshape is at best
   marginal: no plan 36 is written under §1.
2. **If the knife edge must be resolved**, buy the spike of §7 (range
   1.45–2.13 µs/exp = 10.4–12.2 s on `deep-win`, 4.4–5.2×) as the only
   in-base option with a measured case.
3. Sign off G-39c's pooled sample; confirm/withdraw D44–D46.
