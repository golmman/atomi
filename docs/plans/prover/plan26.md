# Prover Plan 26: item 34 — NULL-V2, the per-node residual

Initiative: [`prover`](initiative.md), proposed backlog **item 34**, decisions
**D11**, **D14**, **D18**, **D21**, **D24**, **D26**, **D30**, **D34**,
**D36**, **D37**, proposed **D38**. Drafted 2026-11-12 after [`report25`](report25.md).

## Kill review (fixed prompt)

1. **Distance.** `T_3`; `T_4` needs `deep-win` 83.5 s → 10 s (**8.35×**);
   `core` 6.54 s and `easy-win` 0.72 s fit. The tree is at parity (D36); the
   wall is per-expansion cost, 11.5–15.5×. A read-only probe (5 M evals,
   `deep-win`, seed 0, `/tmp` only) puts the residual in the null replay's
   **pass 2**: 105,120 `null_win` calls, 3.38 M `exposed_limit` calls,
   3.89 M replies, only 3,238 pass-2 hits (**0.096 %**). Each call builds a
   full legal reply list before rejecting the first reply; 24.5 % of null
   calls repeat.
2. **Would we build this?** Yes, as the **last per-node micro-plan**: exact,
   tree-neutral, cheap, and the probe shows 0.90× wall at fixed evals. But
   the gain is 10–20 %, not 8.35×; hence the stop rule below.
3. **Cancel?** Nothing new. PDS-PN/PN², restarts, portfolio, threads,
   tablebases and the labels stay rejected; item 30 is a fallback. The
   per-node micro-line is cancelled **if** this stack misses.
4. **Prior art (delta).** Pin unchanged → changed rows only: R1–R6/R11–R13/
   R18–R19 adopted; R9 closed negative; NULL-V1 adopted (D37). The finding
   is not a mechanism row. No full re-read.
5. **Smallest next experiment.** The probe above: pseudo-legal reply stream
   with lazy legality plus a direct-mapped `exposed_limit` cache. At 5 M
   evals the tree is **identical** (168,910 expansions, 5,000,034
   `child_evals`, 105,120 null calls) and wall is 3.00 s → 2.68 s (0.90×,
   probe overhead included). A status cache was probed and **rejected** (63 % hits,
   slower: 2.82 s).

## Decision

One named stack, one gate, declared before data.

**NULL-V2 (exact; no trajectory change):**
(a) **EARLY-REPLY** — in `exposed_limit`, replace `generate_legal` + full
reply list with the pseudo-legal stream and lazy `is_legal_with_state`,
stopping at the first non-accepted reply. The result is order-independent
(all accepted vs one rejected; `limit` is a max over accepted replies), so
the tree is unchanged.
(b) **EXPOSED-CACHE** — a deterministic direct-mapped cache of
`exposed_limit` results keyed by `(child key, rule50, attacker)`, storing
**both** positive and negative answers. `exposed_limit` is board-only, so
both are exact; a full-key compare on hit makes collisions recompute. The
cache is inside the 192 MB label, compensated from TT/filter, and is part of
the deterministic flag set.

Gate **G-N2**: 120 s, seeds 0–2, three cells, same-session control = current
`--guide hand`; `deep-win` median ≤ **0.85×** control, `core`/`easy-win`
≤ 1.1×, `child_evals`/`expansions` to proof **identical** to the control,
every proved seed D14-accepted. If G-N2 passes, adopt NULL-V2. If it misses,
**close the per-node micro-line**: the next plan is a formulation decision
(PDS-PN/PN² or a goal re-pin), not another micro-plan. **Proposed D38**
records the decomposition, the gate and the stop rule.

## Premise

`exposed_limit` is called once per null move in pass 2 and almost always
fails on the first defender reply, yet it generates the child's complete
legal move list. The probe shows 3.38 M calls for 3,238 successes and 34.5 %
repeats. The reply list is the first-order residual; the cache removes the
repeats, the early exit the tail of the list.

## Scope

**In:** Phase A cache sizing and the positive-null ceiling; Phase B
EARLY-REPLY + EXPOSED-CACHE; Phase C the gate and D14; `report26.md`; a
`bench.md` row at Cap 120 s. **Out:** item 30; search base, starting
numbers, guide structure, tree levers, budgets, goal/rungs; provari
material; any weakening of the null-board invariant; caching
table-dependent results.

## Design

### Phase A — sizing (read-only default)

Reproduce the 5 M-eval probe on the shipped binary. Sweep the cache at
2^18/2^20/2^21 entries (compact `u64` keys + `u8` values, 9–18 MB) and
record hit rate, wall and memory. Measure the **positive-null cache**
ceiling (key = null key + clock, positive only) with a counter; include it
only if it adds ≥ 5 % wall. No default change.

### Phase B — NULL-V2

`src/guide/null.rs`: EARLY-REPLY with a **separate** `StateInfo` for the
child's legality and the scratch make/unmake (the probe's first attempt
shared one and diverged; the differential test pins this). Add the cache as
a small module with a deterministic size, full-key verification and
`--stats` hit/miss counters. The cache is allocated from TT/filter, not on
top of the label. Tests: `exposed_limit` equals the clone-based reference on
every legal child of the fixture corpus; the cache matches an uncached call
on a repeated-position fixture; `--evals 1 M` counters match on all three
cells.

### Phase C — gate

One process at a time, `--memory 192`, `--guide hand`, seeds 0–2, three
cells, control and candidate back to back. Run G-N2. D14 on every proved
seed (600 s proof run, same `child_evals`, both checkers). A failed gate
closes the per-node micro-line; no per-cell switch and no cap change.

### Phase D — docs and decision

`report26.md` states the gate result, same-session ratios, tree counters,
cache hit rate and the new `T_4` distance; one `bench.md` row at Cap 120 s;
the owner confirms or rejects proposed D38; `history.md` line.

## Option that changes the formulation

**NULL-APPROX.** Replace pass 2 with a cheap sufficient screen (e.g. only
null moves that create an immediate explosion threat), gated on wall with a
tree-growth guard. Gain: removes the per-reply scan; cost: NULL-S grew
`easy-win` 3.35×, so this needs its own gate. **Cheapest experiment:**
Phase A's per-null-move threat-signature hit rate.

## Gates

- **G1 data.** `env.json`, `phase_a.json`, `gate_n2.json`, drivers/README
  under `measurements/plan26/`; parsed only, same-session control.
- **G2 exactness.** Differential tests; `--evals 1 M` counters match;
  cache hit/miss counters deterministic.
- **G3** G-N2 as written.
- **G4 soundness.** `make test`, `make lint`; `make test-full` + `make veto`;
  every claim D14-accepted; licence gate unchanged.
- **G5 docs.** `report26.md`, item 34 status, one `bench.md` row at Cap 120 s
  after a passed gate, `history.md` line.

## Risks

- **Shared `StateInfo`** between child legality and scratch make/unmake: the
  probe diverged on this; use two states and pin it with the differential.
- **Cache collisions**: full-key compare on hit; a hash collision is the
  same risk as the TT and is accepted.
- **Memory label**: the cache is compensated from TT/filter; a regression in
  `core`/`easy-win` fails the gate.
- **Pseudo-legal order** differs from legal order: the result is
  order-independent, but the differential test is the gate.
- **Positive-null cache** is table-dependent: only positives are cached.
- **Null-board invariant**: no null position is keyed, stored or emitted.

## Deliverables

`measurements/plan26/`; `report26.md`; NULL-V2 and its cache; the `--stats`
cache counters; item 34 status; one `bench.md` row at Cap 120 s; a
`history.md` line.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan26.md`. Execute
   Phase A (cache sizing and positive-null ceiling), then Phase B exactly as
   written (EARLY-REPLY with two `StateInfo`s, EXPOSED-CACHE with full-key
   verification and a compensated memory split) and prove counter identity
   at `--evals 1 M`. Run G-N2. D14 on every proved seed. Do not change the
   192 MB label, the goal/rungs, or start item 30."
2. G-N2 passes but `T_4` is still far: record the residual and take the
   formulation decision (PDS-PN/PN² or a goal re-pin) at the next kill
   review; do not write another micro-plan.
3. G-N2 fails: close the per-node micro-line with the measured distance; the
   next kill review chooses between the formulation decision and item 30.
4. Cache hit rate < 20 %: keep EARLY-REPLY only, drop the cache.

SESSION COMPLETE
