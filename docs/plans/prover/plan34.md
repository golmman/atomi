# Plan 34 — kill review (plans 32–33) and item 42 (proposed): the expansion census

Read only: the plan33 row of [`bench.md`](bench.md), reports 31–33 and their
[`measurements`](measurements/plan32/README.md), [`analysis1`](analysis1.md) §3,
[`analysis2`](analysis2.md) §5–§6, D40/D43–D45, [`gotchas.md`](gotchas.md) scope.
Pins: oracle `48686bc`, `atomic-movegen` 2.8.0 (crates.io max).

## Kill review (fixed prompt)

### 1. Absolute distance to the goal

`T_3`, geomean **6.606** at Cap 120 s (plan33). One row, decomposed:

| Cap 120, plan33 | `core` | `easy-win` | `deep-win` | geomean |
| --- | ---: | ---: | ---: | ---: |
| atomi wall (s) | 4.923 | 0.376 | 53.420 | — |
| provari `--guide hand` (s) | 0.534 | 0.142 | 4.523 | — |
| **wall factor** | 9.219 | 2.648 | 11.811 | **6.606** |
| tree: expansions / nodes | 1.24× | 0.52× | 1.92× | **1.08×** |
| per unit: µs/exp vs µs/node | 11.6 / 1.56 | 9.9 / 1.96 | 9.2 / 1.50 | **6.13×** |

The gap is **one factor and it is per-node**: tree 1.08×, per-unit 6.13×.
**`T_4` = 5.34×** (`deep-win` 53.42 → 10 s), necessarily per-node work;
**`T_5` = 6.61× geomean** plus every cell under the reference.

**The named residue cannot pay for it.** Named and not yet removed:
lookups/probes **21.55 pp** (report32 @120 s), pass 1's build + `rules::status`
**10.45 pp** (report33 G-40f), `Frame::enter` **9.3 pp** (report31); union with
the probe overlap removed ≈ **42 pp ⇒ ≤ 1.7×** if all free, against 6.61×
(report32 §7.2: every table access gone ⇒ `deep-win` ≈ 46 s). What remains is
**86.8 pp of "the rest"**, unattributed to any volume (moves, lists, children,
allocator, frame) or rate (ns/op) — that is where 6.13× lives.

**Rate.** Plans 31–33 banked 7.23 → 6.606 (**1.031×/plan**); the six since
`T_3` (11.93) banked 1.81× (1.106×/plan). That is ≈ **63 plans** at the current
rate, ≈ **19** at the six-plan rate — neither touching `deep-win`'s 11.8×.

### 2. Would we build this today?

Split. **Yes** for the hand guide + full-key proof table + own validator and the
`provari check` veto: that is why the tree is at 1.08×, and it is the durable
asset. **No** for the 26 micro-plans: two closed `no effect` after a plan
session (MG15 1.147× the wrong way, EFFECT-CHILD 0.00 %) and the adopted ones
were 0.72–0.78×, 0.986×, 0.926×, each written against a 6.6× target. Starting
today: per-expansion cost is the primary metric from M1, and the node body is
built to the reference's *shape* (bit tests, one move stream, a bounded closure
count), not micro-fixed after. Load-bearing: **D40's "per-node line closed for
good" was falsified by the plans that followed it**, and D45 repeats it while
forbidding the only mechanisms with a measured case.

### 3. What would you cancel?

- **Item 41 as a `prover` item.** Ceiling = G-40f's 10.45 pp ≈ **1.11×**;
  `movegen` has missed two cost bars (MG19/MG20) and MG15 measured 1.147×
  *worse*. Keep it as a `movegen` wishlist line; no `prover` plan waits on it.
- **The micro-plan mould.** No plan 34+ with a projected gain < 1.2×: ours offer
  1.02–1.07×, so the rule alone removes ~55 of the ~63 steps above.
- **D40/D45's "closed for good" sentence** and `analysis2` §7's
  crate-is-the-lever premise (the crate cannot sell 6× while its hot path is
  10.45 pp). **Stale to fix:** `analysis2` §5 P1 (15.5 µs, 11×) and §6 (geomean
  9.681, 7.17×) are pre-plan31; this document is the reading until a revision.
  Open: G-39c's pooled sample (D44), D42–D46 *proposed*.

### 4. Prior-art search (delta only; no pin change)

`analysis1` §3 rows whose atomi state changed since the plan31 review:

- **R23/R24 (proof store, table) — keep.** G1 (plan32) and NULL-PIPE (plan33)
  priced the probe line on **our** side: ~46 lookups/expansion (269.1 M / 5.80 M),
  pass-1 probe 2.10 pp, a dependent probe 12–18 ns against ~14 ns for a skipped
  one. The table is not the gap.
- **R22 (node-entry early return) — small, skipped**: unchanged, 45 of 153 k
  expansions, and `gotchas.md` forbids re-proposal. **R2/R4/R5** rejected,
  **R7/R8** aligned, **B4/R25/C9** as recorded; all unchanged since item 26.

No verdict changes, no new row, no full re-read (pins unchanged). Literature
delta: nothing *inside* DF-PN the rows do not already name; its in-base levers
(TT move ordering, restarts, entry reuse) are B4/R22 — measured `small` here —
and restarts (rejected, item 18). 6.13× is not a missing idea on the table.

## The decision this review puts to the owner

D45: "a plateau with a large multiple still missing goes to a **kill review**,
never to an item." This is it: the missing multiple is implementation a
micro-lever cannot reach (≤ 1.7× named vs 6.13× needed). **Recommend A now, then
B or C on A's number** — A prices the body before another lever is bought.

| | Option | Est. gain | Cheapest decisive experiment |
| --- | --- | --- | --- |
| **A** | Run item 42, decide on its number | 0× direct; prices later plans | the plan below |
| **B** | **Changes the formulation, narrowly**: keep DF-PN, allow an *in-base node-body rewrite* ("v3 expansion": one move stream per expansion, the three guide passes merged over it, children decided by bit tests) — D45's own conclusion, not branch 1 | **2–4×** if the gap is volume (570 list moves, ~46 probes, ~14 built children per expansion vs one reference node at 1.5 µs); 1.0× if it is rate | A's volume counts, then a one-day spike rewriting only the null pass over that stream, same tree |
| **C** | **Changes the problem formulation**: re-anchor the goal (D30) — provari ratio a diagnostic, `T_4` the goal, or coverage + soundness, which `T_3` meets | removes an unreachable clause; 0× on wall | A's number: a removable ceiling < 5.34× makes `T_4` unfundable too, and C = archive-with-record (the risk table's own row) |
| **D** | Archive at `T_3` with its record; cancel the wall leg | — | one decision |

B and C are **owner decisions** (D45 forbids either as an item): this plan may
propose them and may not start them.

## Item 42 (proposed) — EXPANSION-CENSUS

Read-only: no engine change, flag or default, no `bench.md` row. One scratch
instrumented build (rdtsc buckets, plan33's method, −3.24 ns/read).

**Buckets, exhaustive** (`other` **< 10 pp** or the census failed): movegen
(pseudo-legal, legal, view), legality/`status`/terminality,
`do_move`/`undo_move`/`clone`, `key`/`rule50`, TT probe, proof probe, guide
closures by site (`frame`, `null` pass 1, `null` pass 2, `replay`/simulation),
`Frame::enter`, allocator. **Counts per expansion**: moves, lists, children,
probes, `null` calls, evals.

**Gate G-34**: one 120 s `deep-win` seed 0, tree identical to the plan33 row
(5,803,257 expansions), buckets summing to the wall ±2 pp, `other` < 10 pp.

**Decision number** per option: the largest removable bucket (pp ⇒ ×) and the
removable union. **Stop rule: if no bucket ≥ 15 pp or the removable union
< 5.34×, the per-node route cannot reach `T_4`** — the goal question goes to the
owner with that number and no plan 35+ is written in the micro mould.

## Scope

**In:** the census, `measurements/plan34/`, `report34.md`, one `history.md`
line, the delta table. **Out:** any engine/default/flag change, any `bench.md`
row, item 41, a formulation change (D45 stands until the owner narrows it).

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan34.md`. Execute item
   42 as written: an exhaustive-bucket rdtsc census of one 120 s `deep-win`
   seed-0 run (tree must equal 5,803,257 expansions) with per-expansion counts of
   moves, lists, children and probes; `other` < 10 pp. Report the largest
   removable bucket and the removable union against 5.34× (`T_4`) and 6.61×
   (`T_5`). No engine code, flag, default or `bench.md` row."
2. Owner takes **B** (narrow D45 to an in-base node-body rewrite) on A's number;
   the wording lands in `decisions.md` first.
3. Owner takes **C** or **D**: goal section, `bench.md` framing and
   `check_goal.py` change in one session.
4. Sign off G-39c's pooled sample; confirm/withdraw D42–D46.

SESSION COMPLETE
