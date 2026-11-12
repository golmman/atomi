# Prover Plan 28: item 35 — HOTPATH-V1, the per-expansion representation stack

Initiative: [`prover`](initiative.md), proposed backlog **item 35**, decisions
**D2**, **D11**, **D14**, **D21**, **D24**, **D26**, **D30**, **D34**, **D36**,
**D38**, **D39**, proposed **D40**. Upstream wishlist:
[`../movegen/initiative.md`](../movegen/initiative.md). Drafted 2026-11-12
after [`report27`](report27.md) and a read-only `perf` probe.

## Kill review (fixed prompt)

1. **Distance.** `T_3`; `T_4` needs `deep-win` 71.7 s → 10 s (**7.17×**).
   The tree is at parity (D36); the wall is per-expansion cost. A 4 s
   `perf` probe on `deep-win` (`--guide hand --memory 192 --seed 0`) gives
   **15.5 µs/exp** against provari's **1.42 µs/node** (11×). Self-time:
   movegen/legality/terminality ~54 %, guide ~17 %, table lookup ~12 %,
   null-board FEN ~1.6 % (+ allocator), frame ~7 %. `core` is the same.
2. **Would we build this?** Yes, as a **named representation stack**: the
   safe three are tree-identical, and matching provari's per-node cost is
   exactly `T_4`. But a 1.3–1.5× patch does not move the rung; hence the
   stop rule.
3. **Cancel?** The per-node micro-line is closed (D38/D39). This plan
   reopens it **only** as a representation stack. If G-H1 misses, close it
   for good and take the formulation fork (PDS-PN/PN² or a goal re-pin).
   PDS-PN/PN², restarts, portfolio, threads and tablebases stay rejected.
4. **Prior art (delta).** Pin unchanged → changed rows only: D37/D38/D39.
   The profile is our own measurement, not a provari mechanism row.
5. **Smallest next experiment.** The 5 M-eval tree-identical probe with
   S1+S2+S3 (below). It is cheap, reversible and decides whether the line
   can reach 1.5× at all.

## Decision

One named stack, one gate, declared before data.

**HOTPATH-V1** (S1–S3 tree-identical; S4 conditional):
- **S1 NULL-BOARD-IN-PLACE.** Replace `null_board`'s
  `fen_with` → split/join → `from_fen` round-trip with a cheap board
  mutation (`Board::null_for(attacker)`, upstream MG1). The null position is
  still ephemeral and never keyed/stored/emitted.
- **S2 GUIDE-MOVEGEN-STREAM.** Route `threat::immediate_explosion_bits` and
  `null::null_win` through a callback/iterator (`for_each_pseudo_legal`,
  upstream MG2) so no `MoveList` is materialised and the scan stops at the
  first accepted move. The order is the same as `generate_pseudo_legal`, so
  the returned move and the tree are unchanged.
- **S3 LEAN-LOOKUP.** Add a `Tables::lookup` path that probes proofs then TT
  without constructing `EntrySource`; keep `lookup_with_source` for
  `--stats`. Tree-identical.
- **S4 TERMINALITY-FROM-EFFECT** (only if S1–S3 pass G-H1). Replace the
  per-child `rules::status` → `Board::outcome` → `has_legal_move` path with
  an effect-based query (`Board::outcome_after`, upstream MG3), falling back
  to the full legal search only where stalemate is possible. This is the
  only correctness-sensitive item; it has its own gate.

Gate **G-H1**: 5 M child evals, `deep-win`, seed 0, `--guide hand
--memory 192`, same-session control. `child_evals` and `expansions`
**identical** to the control; wall ≤ **0.667×** control (≥1.5×). If it
passes, run the 120 s matrix (seeds 0–2, three cells) and D14; adopt only if
`deep-win` median ≤ 0.9× control and no cell > 1.1×. If G-H1 misses, close
the per-node line for good.

Gate **G-H2** (S4 only): differential `outcome_after` vs `outcome` on the
fixture corpus and the three bench cells; 5 M `child_evals`/`expansions`
identical; wall ≤ 0.8× the S1–S3 candidate. Any divergence drops S4.

## Scope

**In:** S1–S3; S4 only after G-H1; the upstream API in a **local checkout**
for measurement (MG1–MG3); tests; `measurements/plan28/`; `report28.md`; one
`bench.md` row at Cap 120 s after a passed gate; proposed D40. **Out:** the
search base, tree levers, guide algebra, budgets, goal/rungs, provari
material, PDS-PN/PN², restarts, portfolio, threads, tablebases, a committed
path dependency.

## Design

**Phase 0 — upstream API (measurement only).** Add MG1/MG2 (and MG3 for S4)
in a local `atomic-movegen` checkout; use a temporary path dependency to
measure, **never commit it**. The wishlist lives in
[`../movegen/initiative.md`](../movegen/initiative.md). If MG1/MG2 are not
available, S1/S2 are blocked: run S3 only and lower G-H1 to ≥1.25×, or close.

**Phase A — attribution (read-only).** Reproduce the 4 s `perf` profile.
Add observation-only counters: `needs_board` children, child lookup
hits/misses, `generate_pseudo_legal` calls per expansion, `null_calls`.

**Phase B — S1–S3.** `null_board` uses the upstream mutation; the guide
paths use the callback stream; `Tables::lookup` is lean. Tests: null-board
differential (same key, side, ep, clock as the FEN path); movegen-order
differential; `--evals 1 M` counters match on all three cells.

**Phase C — G-H1 and the 120 s matrix.** One process at a time, control and
candidate back to back. If G-H1 passes, run the 120 s matrix and D14 on
every proved seed (600 s proof run, same `child_evals`, both checkers).

**Phase D — S4 (conditional).** Only after G-H1. Implement `outcome_after`
upstream, run G-H2, then the 120 s matrix and D14 again. Any divergence
drops S4; S1–S3 stay.

**Phase E — publish and re-pin.** Publish the upstream release (movegen
initiative), bump the dependency from 2.4.x, re-run G-H1 on the published
crate, then `make test-full` + veto. Remove the path dependency before the
commit.

**Phase F — docs and decision.** `report28.md` states the gate result,
same-session ratios, tree counters and the new `T_4` distance; one
`bench.md` row at Cap 120 s; the owner confirms or rejects proposed D40;
`history.md` line.

## Option that changes the formulation

**Effect-only child evaluation.** Never materialise a `Board` for a
non-terminal child: carry the effect, key, counter and class, and build a
board only when the effect says the child may be terminal or needs a settle.
This is S4 as a representation, not an optimisation. Gain: removes much of
`do_move`/`populate_state`/`legal`/`compute_checkers` (~20–25 % of wall).
**Cheapest experiment:** Phase A's `needs_board` rate and the built-vs-non-
built cost split; if >70 % of children are non-board, it is worth a plan.

## Gates

- **G1 exactness.** S1–S3 tree-identical at 5 M; null-board and movegen-order
  differentials; `--evals 1 M` counters match.
- **G2** G-H1, then G-H2 if S4 runs.
- **G3 soundness.** `make test`, `make lint`; `make test-full` + `make veto`;
  every claim D14-accepted; licence gate unchanged.
- **G4 docs/data.** `env.json`, `phase_a.json`, `gate_h1.json`,
  `gate_h2.json`, drivers and `report28.md` under `measurements/plan28/`;
  item 35 status, one `bench.md` row at Cap 120 s after a passed gate,
  `history.md` line, D40.

## Risks

- **Upstream API not available.** S1/S2 are blocked; no `unsafe`, no
  committed path dependency. Fall back to S3 only, or close.
- **S2 order.** The callback must yield the same order as
  `generate_pseudo_legal`; the differential test is the gate. Keep
  `lookup_with_source` for `--stats`; the lean path is the default.
- **S4 correctness.** Terminality is a soundness surface; the differential
  against `outcome` on the fixture corpus and the bench cells is the gate.
  Any divergence drops S4.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan28.md` and
   `docs/plans/movegen/initiative.md`. Execute Phase 0 (local MG1/MG2 only),
   Phase A (counters + profile), Phase B (S1–S3), then G-H1 exactly as
   written. Do not start S4 unless G-H1 passes. If G-H1 passes, run the
   120 s matrix and D14, then Phase E (publish and re-pin)."
2. G-H1 passes, S4 fails: adopt S1–S3, record the residual, take the
   formulation fork.
3. G-H1 misses: close the per-node line for good; the next plan is the
   formulation decision (PDS-PN/PN² or a goal re-pin).
4. Upstream only: file MG1–MG3 in the movegen initiative and do not block
   the prover plan on a release.

SESSION COMPLETE
