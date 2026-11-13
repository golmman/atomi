# Prover Plan 29: item 36 — EFFECT-CHILD-V1, MG3 in per-child classification

Initiative: [`prover`](initiative.md), backlog **item 36**, **D41** (proposed);
D2, D11, D14, D19, D21, D24, D26, D30, D34, D36, D38, D39. Upstream:
`atomic-movegen` 2.6.0 **MG3** (`Board::effect_context`,
`EffectContext::{effect,is_legal,outcome_after}`),
[`../movegen/report1.md`](../movegen/report1.md) §7.1; drafted 2026-11-13 after
[`report28`](report28.md) and
[`measurements/movegen/plan1/`](../../../measurements/movegen/plan1/README.md).

## Kill review (fixed prompt)

1. **Distance.** `T_3`; `T_4` needs `deep-win` 71.7 s → 10 s (**7.17×**, D39).
   Tree at parity (D36), the wall is per-expansion cost (2,045 ns/expansion).
   Worth **≤ 1.05–1.15×**: moves **no rung**.
2. **Would we build it today?** Only as *dependency realisation*: report28
   closed the per-node micro-line for good (0.8206× vs ≤ 0.667×). New: MG3 is
   **published**, so the per-child board write is no longer our design choice.
   It attacks what plan28's profile left: `legal`, `do_move`,
   `compute_checkers`, `populate_state` ≈ 23 % of self-time.
3. **Cancel?** Yes, cheaply: Phase A's stop rule closes it before any wall
   gate, and it costs one plan before the formulation decision (PDS-PN/PN² or
   a goal re-pin). Those, restarts, portfolio, threads and tablebases stay
   rejected; the 120 s budget is not raised.
4. **Prior art (delta).** Pin unchanged (`48686bc`): one changed row, MG3
   *proposed → published*; no provari row; MIT crate surface.
5. **Smallest next experiment.** Phase A's counters on the existing 5 M probe.

## Decision

One named stack, one gate, declared before data.

**EFFECT-CHILD-V1** — a child's status comes from the parent's
`EffectContext`, not from a built board:

- **C1 ONE-CONTEXT-PER-NODE.** One `EffectContext` per expanded node via
  `effect_context_with_state(&state)`, reusing the `StateInfo` `Frame::enter`
  already populates — never `effect_context()`, which repopulates.
- **C2 STATUS-AFTER.** `rules::status_after(board, attacker, &ectx, m,
  rule50_after) -> Status`: `ectx.outcome_after(board, m)` decides extinction,
  commoner draw and the non-check clock, mapped with the **same precedence,
  perspective and `from`** as `rules::status_with` (`from = 100` iff
  `rule50_after >= 100`). MG3 is not trusted for the rest: the built child's
  `rules::status` stays the decider, with atomi's checker-material clause
  screened exactly (a queen/rook/knight/pawn surviving the blast ⇒ no material
  draw).
- **C3 BUILD-LAST.** A child C2 decides from the effect is classified without
  `do_move` **where the guide reads no child board**; where the hand guide does
  (`classify_exact`'s `gives_check`/`threat_count`/`blasted_cheaply`,
  `hand_eval`, settle, sim, replay) the build stays, so classes — and the
  trajectory — do not move. **Tree identity gates C3**: if the counters move,
  C3 narrows to `--guide none` and C1+C2 stand alone. **C2′** (pre-declared): if
  the fallback share is high, C2 reads `EffectContext::effect`'s fields
  (`explodes_enemy_commoner`, `checks_enemy`, casualties, `is_legal`) — still
  MG3, same differential.

Gate **G-E1**: 5 M `child_evals`, `deep-win`, seed 0 (`--seconds 600
--memory 192 --evals 5000000 --seed 0 --guide hand --stats`), three
alternating repeats, one process at a time, same-session control.
`child_evals` **and** `expansions` identical to the control (5,000,034 /
168,910) and median wall **≤ 0.95×** ⇒ adopt with a wall claim (≥ 1.05×);
**0.95–1.02×** ⇒ adopt *neutral* (no wall claim, no `bench.md` row);
**> 1.02×** ⇒ revert (spread ±2–3 %, `../movegen/report1.md` §4.6).

**Stop rule.** Phase A decided-child share < 10 % of built children ⇒ close as
`no effect`: no wall gate, no bench row. **Pushback:** no reopening of
report28's closure and no delay of the formulation decision beyond one plan.

## Scope

**In:** `frame.rs`'s child loop, `rules::status_after`, the tsv
differential, counters, docs, `measurements/plan29/`. **Out:** the search
base, `Mover::effect`, guide algebra, budgets/rungs, MG3 elsewhere, provari,
PDS-PN/PN², restarts, portfolio, threads, tablebases.

## Design

**A — sizing (observation only, revertible).** `--stats`: `needs_board`
children, `decided_by_effect` (extinction / draw / clock), `fallback_to_child`,
`boards_saved` (C3 only), on the 5 M probe and `--evals 1000000` × three cells.

**B — C1+C2, then C3.** Differential first (`tests/effects_differential.rs`):
for every `P` row of the frozen tsv (94 positions, 1,605 legal moves) and every
legal move, `ectx.outcome_after` equals the `M` row's `child_outcome`,
`ectx.is_legal` is true, and `status_after` equals
`rules::status(&moved_child, attacker)` for both attackers; for an illegal
candidate, `None` / `false`. Assert the material-draw fallback fires. Then C3
behind that test plus counter identity at `--evals 1000000` × three cells.

**C — G-E1.** `measurements/plan29/probe.py` (plan28's probe shape, own
`--evals` command; parsing via `bench/run_bench.py`) writes `gate_e1.json`.
Baseline **2,045 ns/expansion** (`measurements/movegen/plan1/pernode.json`);
the control read 2.558 s (plan28) / 2.610 s (movegen plan1), so every ratio is
**re-measured same-session**.

**D — 120 s matrix + D14 (G-E1 only).** Seeds 0–2 × three cells, 192 MB,
control/candidate alternating: adopt only if `deep-win` median ≤ 0.9× and no
cell > 1.1×; D14 on every proved seed (600 s `--proof-out`, identical
`child_evals`, both checkers). One `bench.md` row at Cap 120 s with
`--atomi-stack 'hand=--guide hand'`.

**E — docs and gates.** `report29.md`, gotchas, item 36 status, D41, one
`history.md` line. **G-E0** tsv differential empty, no fixture rewrite
(`$ATOMI_WRITE_GOLDEN` only), counter identity at 1 M × three cells. **G-E2**
`make test`, `make lint`, `make test-full`, `make veto` (8/8 accepted, 4/4
rejected), no fixture-oracle refutation, licence gate unchanged. **G-E3**
`env.json`, `phase_a.json`, `gate_e1.json`, `gate_e2.json`, drivers and README
under `measurements/plan29/`.

## Option that changes the formulation

**Child-view movegen (upstream).** MG3's docs call the clone-and-`outcome`
fallback an implementation detail that "may disappear once a child-view
generator can prove *no legal move* without a board write". Asking for it
removes the **last** board write from the per-child path — estimated
**1.1–1.2×** on top of C2 if the fallback share is high, and the only route
toward provari's 1.42 µs/node that leaves the search alone. **Cheapest
experiment:** Phase A's `fallback_to_child / boards_built` (> 40 % ⇒
`handoff2.md`).

## Risks and gotchas to add

- **`outcome_after` conflates illegal with open.** Its doc says `None` =
  illegal, but a legal open child also returns `None`. Never read `None` as
  illegal: legality comes from the parent's `generate_legal_with_state`. Ask
  upstream for a two-level answer.
- **The material-draw clause is ours, not the crate's** (checker rule vs K vs
  K): without the exact screen C2 turns `Failed` children into `Open`. Screen
  + differential are the guard.
- **Guide classes read the child board**: a skipped build changes
  `gives_check`/`threat_count`/`blasted_cheaply` and can move the tree.
- **Double work.** `Mover::effect` and `EffectContext::effect` overlap: the
  counters must show the added `analyze` is cheaper than the removed
  `do_move`+`outcome`. Castle children keep the crate's inner `Board::legal`
  fallback and stay built.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan29.md`,
   `report28.md` and `docs/plans/movegen/report1.md`. Execute Phase A
   (counters only) and apply the stop rule. If it passes, implement C1+C2, run
   the differential and the 1 M identity, then C3 and G-E1 as written; Phase D
   only if G-E1 passes."
2. G-E1 neutral band: keep C1+C2 (+C3 if tree-identical), no wall claim or
   bench row; take the formulation decision.
3. Stop rule or revert: close item 36 as `no effect`; the formulation decision
   is the next plan. Upstream asks go to `handoff2.md` either way.
SESSION COMPLETE
