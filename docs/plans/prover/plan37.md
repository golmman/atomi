# Plan 37 — item 46 (proposed, OVERHEAD-PROFILE): the 28.45 pp no counter owns

Read only: [`report34`](report34.md) §2–§4 and
[`measurements/plan34/`](measurements/plan34/README.md) (the census),
[`report35`](report35.md) §4 (`R`), [`gotchas`](gotchas.md) §Scope, D22/D36/
D40/D43–D45. Pins: `atomic-movegen` 2.8.0. **The oracle is not used** — no GPL
exposure at all.

## Why this item

The census accounts for **68.97 pp** (37.005 s of 53.656 s) inside named calls.
The rest is **`other` = 28.45 pp** — time inside a named region but inside no
measured call — plus **`outside` = 2.58 pp** (the MID loop and everything not in
a frame): **31.03 pp = 2,869 ns/exp**, i.e. more than any single named bucket
except `movegen` (12.48 pp). The reference's equivalent is **≈325 ns** (325 of
its 2,111 ns/body, report35's cross-check): **8.8×**.

It is the largest one-line gap in the budget and **no item has ever measured
it** (item 42 timed calls, not the loop between them; item 43 priced the same
calls against the reference). It is also the item most likely to settle a
question report35 left open: if the call level cannot reach `M` ≤ 1,723, the
only remaining per-node headroom is here — or nowhere.

The number is **real**, not an artifact: region aggregates are wall-clock
durations that need no calibration and sum to 100.00 % by construction (the
`sites` build costs +0.94 s, 1.8 %). What is missing is its **composition**.

## 1. The decision (pre-registered)

Classify the 2,631 ns of `other` (+ 239 ns `outside`) into four classes, each
with a measured ns estimate: **(i)** retired instructions in the regions' own
loops (per-child stores, index arithmetic, bounds checks), **(ii)** branch
mispredicts, **(iii)** cache/DRAM misses (the child structs, the move buffers,
the pass-2 cache-hit path), **(iv)** compiler artifacts (spills, allocator
calls, un-inlined helpers). The remainder is reported as `unexplained`.

- **LEVER** ⇔ the top class ≥ **25 %** (≥ 658 ns = 7.1 pp) **and** it has a
  stated mechanism (not "the loop is slow"): it becomes a candidate item, to be
  scoped in the next plan session with a prototype measured in scratch.
- **NO LEVER** ⇔ no class reaches **10 %** (263 ns = 2.8 pp): 28.45 pp is a
  spread over dozens of small effects, the per-node line is closed on
  measurement rather than on decision (D40), and the goal question returns to
  the owner as plan34's C/D with items 45–46 in hand.
- **INCONCLUSIVE** ⇔ the classification covers < 80 % of the 2,631 ns. Reported
  with the gap; no lever claimed.

Predictions are **not** inputs: I expect (ii)+(iii) to dominate, but a vote
against that changes no gate.

## 2. Method

**Phase 0 — the exact envelope (no runs, ~20 min).** From
`measurements/plan34/census_120s.json`, compute each of the 10 regions' *own*
time = region total − Σ timed leaves inside it, and check it reproduces
`other` = 28.45 pp ± 0.5 pp. This binds Phase A: the profile must explain *this*
residue, per region, not the wall in general. Also list, per region, the
non-call work the census already names (per-child stores, the pass-2 cache-hit
path, pipeline overlap) as the candidate list to confirm or refute.

**Phase A — symbol- and instruction-level profile (~5 h).** Release build of the
current tree, `atomi prove "<deep-win FEN>" --guide hand --memory 192
--seconds 120 --seed 0`:
1. `perf stat -e cycles,instructions,branches,branch-misses,L1-dcache-load-misses,
   LLC-load-misses,stalled-cycles-frontend` — whole run, 3 repeats, medians:
   a first-order read of "spread out" versus "one dominant stall".
2. `perf record -F 999 -g --call-graph dwarf` + `perf annotate` on the hot
   symbols (`Frame::enter`, null pass 1/2, `replay`, `Mover`/movegen, `select`),
   line attribution from a **scratch** build with `debug = 1` (its wall measured
   against a same-session control, any codegen deviation stated).
3. Per-region cross-check: sampled share × region total within ±20 % of the
   census region total — an independent tool agreeing with the instrumented
   build is the point.

**Phase B — reconcile and decide.** Split each region's own-time into the four
classes with a stated estimate and error bar (mispredict cost = misses ×
measured penalty, etc.), sum to `other`, and take the §1 decision. Write the
per-region table even if the answer is NO LEVER — it is the artifact that
retires the question.

**Phase C — optional, scratch only, ~2 h.** For the largest candidate, a
micro-prototype in a `/tmp` copy of the tree (e.g. `#[repr(C)]` child struct, a
hoisted bounds check, a stack buffer instead of a `Vec`), measured against a
same-session `git archive` control (rule 1), tree identity required. Its only
job is to size the candidate; **nothing enters the repo**, and a prototype that
breaks tree identity is reported invalid, not fixed here.

## 3. Gates

- **G-46a envelope** — Phase 0 reproduces `other` = 28.45 pp ± 0.5 pp and
  `outside` = 2.58 pp from committed integers.- **G-46b tree identity** — every profiled run reports 5,803,257 expansions,
  155,236,740 child evals, length 36 (profiling may change the wall, never the
  tree); the scratch `debug = 1` build is checked the same way.
- **G-46c coverage** — the four classes cover ≥ 80 % of the 2,631 ns at region
  granularity, with `unexplained` printed per region.
- **G-46d independent agreement** — sampled region shares within ±20 % of the
  census region totals, else the profile is declared not comparable and the
  verdict is INCONCLUSIVE.
- **G-46e decision** — LEVER / NO LEVER / INCONCLUSIVE per §1, with the class
  table and a candidate list (ns, mechanism, prototype cost).
- **Boundary** — no engine, flag, default, test or `bench.md` change; no
  provari; scratch trees under `/tmp/atomi-plan37/` only; committed =
  `measurements/plan37/` (driver, parsed integers, `env.json`, README) and
  `report37.md`.

## 4. Scope, effort, risks

**In:** Phases 0–B (C optional), `measurements/plan37/`, `report37.md`, one
`history.md` line. **Out:** any implementation in the repo, any D-decision, any
`bench.md` row, any D14 claim (nothing is proved, nothing changes).

Effort ≈ 1 day. **Risks:** (i) `perf` availability — this host runs as root with
`perf_event_paranoid = 2`, so counting and sampling are expected to work; a 1 s
smoke run precedes anything long, and if sampling is blocked the verdict is
INCONCLUSIVE rather than a fallback to in-tree interval timing (census §2:
strided sampling is biased here, 1.6–1.8× heavy); (ii) release builds carry no
debug info — hence the scratch `debug = 1` build and G-46b; (iii) region
granularity is coarse (`enter`/`mid`/`null_p1`/`null_p2`/`replay`), so the plan
localises `other` *per region*, not per line, and says so; (iv) one 53 s
process is long for sampling — `-F 999`, 3 repeats, never two at once.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan37.md`. Execute item
   46 as written: Phase 0 reproduces `other` = 28.45 pp from
   `measurements/plan34/census_120s.json`; Phase A profiles the release binary
   (`perf stat` + `perf record`/`annotate`, scratch `debug = 1` build for line
   attribution) on `deep-win --guide hand --memory 192 --seconds 120 --seed 0`;
   Phase B classifies the 2,631 ns into retired instructions / branch
   mispredicts / cache misses / compiler artifacts per region and takes the
   LEVER / NO LEVER / INCONCLUSIVE decision of §1. Phase C optional, `/tmp`
   only. Data in `measurements/plan37/`, report `report37.md`; no engine, flag,
   default or `bench.md` change; the oracle is not used."
2. LEVER ⇒ scope the top candidate as an item next session, with Phase C's
   measured size as its expected gain and a tree-identity gate.
3. NO LEVER ⇒ owner rules on plan34's C/D with reports 35–37: the per-node line
   is closed by measurement, and the only open axis is item 47 (SHORT-WIN,
   pending [plan36](plan36.md) Phase 0).
4. Run [plan36](plan36.md) first if the M-REMAP verdict is the decision that
   matters more this week; the two plans do not interact (no shared runs).

SESSION COMPLETE
