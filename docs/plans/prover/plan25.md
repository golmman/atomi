# Prover Plan 25: item 33 — per-node cost (NULL-V1)

Initiative: [`prover`](initiative.md), backlog **item 33**, decisions **D11**,
**D14**, **D18**, **D21**, **D24**, **D26**, **D30**, **D34**, **D36**,
proposed **D37**. Drafted 2026-11-12 after [`report24`](report24.md).

## Kill review (fixed prompt)

1. **Distance.** `T_3`; `T_4` needs `deep-win` 105.3 s → 10 s (10.5×);
   `core`/`easy-win` already fit. The tree is at parity (D36); the wall is
   **per-expansion cost, 11.5–15.5×**. Scratch-clone probes (not claims) put
   the dominant term in the **exact null replay**: `deep-win` at 5 s is
   23.2 µs/expansion with `--guide hand` and 6.4 µs with `--guide-off
   null-exact`. The board-free explosion closure keeps every counter
   identical (155,236,740 `child_evals` to the proof) and cuts 23.2 → 17.6 µs
   at 5 s / 15.3 µs at 120 s; plus a pass-2 signature screen, 8.4 µs and a
   69.9 s proof. The gap is **null-replay work**.
2. **Would we build this?** Yes: the only authorised next item, the measured
   first-order term, and the exact arm changes no trajectory.
3. **Cancel?** Nothing new. PDS-PN/PN², restarts, portfolio, threads,
   tablebases and the 120 s/192 MB labels stay rejected; item 30 is a
   fallback.
4. **Prior art (delta).** Pin unchanged → changed rows only: R1–R6/R11–R13/
   R18–R19 adopted; R9 closed negative (report24). The per-node gap is a new
   finding, not a mechanism row. No full re-read.
5. **Smallest next experiment.** Phase A's two counters (`null_win` calls,
   `exposed_limit` replies).

## Decision

Two named stacks, one gate each, declared before the data.

**NULL-V1 (exact arm; no trajectory change):**
(a) **PASS2-BITS** — in `exposed_limit`, accept a reply iff the defender
self-exploded or `threat::immediate_explosion_bits(next, attacker)` wins,
after the existing 50-move check. This replaces `rules::status` +
`immediate_explosion`, which run a full movegen and build children per reply;
(b) **SCRATCH-BOARD** — one cloned scratch board + `do_move`/`undo_move` in
`null_win` passes 1–2 and `exposed_limit`, not one clone per move/reply;
(c) **DEFENDER-EVAL** — `hand_eval` only at attacker nodes (and when
`efpn_banded` is on); shipped `ef_pn = OrOnly` never reads a defender eval.

Gate **G-V1**: 120 s wall, seeds 0–2, three cells, same-session control =
current `--guide hand`; `deep-win` median ≤ **0.80×** control, `core`/
`easy-win` ≤ 1.1×, `child_evals`/`expansions` to proof **identical** to the
control, every proved seed D14-accepted.

**NULL-S (heuristic arm; only after G-V1):** NULL-V1 plus **PASS2-SCREEN** —
run pass 2 only when `threat::signature_for(&null, attacker)` is non-empty;
pass 1 (direct win or stored proof) is untouched. Gate **G-S**: `deep-win`
median ≤ **0.67×** control, `core`/`easy-win` ≤ 1.1×, every proved seed
D14-accepted; tree growth is reported, not gated. If G-S passes, adopt it and
keep the screen's on/off ablation; if only G-V1 passes, adopt NULL-V1 and
keep NULL-S opt-in; if both fail, close item 33. **Proposed D37** records the
decomposition and this rule.

## Premise

The null replay runs at **every defender node with an open child**: it
serialises to FEN, rebuilds the null board, scans all attacker moves, then
per null move generates every legal reply and, per reply, clones and runs a
status query plus an explosion query. Probes locate the 17 µs/expansion above
the null-off floor there; the status/explosion pair can become the existing
bit closure.

## Scope

**In:** Phase A attribution; closure; scratch boards; defender-eval guard;
screen arm; 120 s matrix; D14; `report25.md`; one `bench.md` row at Cap 120 s.

**Out:** item 30; search base, starting numbers, guide structure, tree
levers, budgets, goal/rungs; provari material; any weakening of the
null-board invariant.

## Design

### Phase A — attribution (read-only default)

Run `deep-win` at 20 M child evals and 10 s, seeds 0–2, `--stats`: record
`child_evals`/exp, µs/exp, `aux_nodes`/exp, plain share. Add two permanent
deterministic `--stats` counters: `null_calls` and `null_replies`. They are
observation-only and enter no budget. No default change.

### Phase B — NULL-V1

`src/guide/null.rs`: add the closure path (a), a `Board` + `StateInfo`
scratch pair per call (b), and unit tests: closure `exposed_limit` returns the
same `Option<u8>` as a clone-based reference on a fixture corpus; `null_win`
returns identical `(mv, length, limit, work)` both ways; `undo_move` restores
the fields the pass uses; every null-board invariant test still passes.

`src/search/frame.rs` + `src/guide/mod.rs`: compute `hand_eval` only at
attacker nodes or under `efpn_banded` (c). The trajectory must be
bit-identical under `--guide hand`; check `--evals 1 M` on all three cells
before timing.

### Phase C — gates

One process at a time, `--memory 192`, `--guide hand`, seeds 0–2, three
cells, control and candidate back to back. Run **G-V1** first; if it passes,
run **G-S** with the same control. D14 on every proved seed (600 s proof run,
same `child_evals`, both checkers). A failed gate closes that arm; no
per-cell switch and no cap change.

### Phase D — docs and decision

`report25.md` states which arm passed, same-session ratios, tree counters and
the new `T_4` distance; one `bench.md` row at Cap 120 s; the owner confirms or
rejects proposed D37; `history.md` line.

## Option that changes the formulation

**NULL-MEMO instead of the screen.** Cache positive `NullWin` results and
`exposed_limit` answers in a small deterministic direct-mapped cache keyed by
position + clock. Positive null proofs are permanent (D18 never evicts), so
the cache is trajectory-neutral. Gain: repeated defender work removed without
the screen's 2.4× tree; the cost is a small cache outside the label or a
compensated split. **Cheapest experiment:** Phase A's `null_calls` vs
repeats; drop it below a 20 % repeat rate. This is the fallback if G-S's tree
growth is unacceptable.

## Gates

- **G1 data.** `env.json`, `phase_a.json`, `gate_v1.json`, `gate_s.json`,
  drivers/README under `measurements/plan25/`; parsed only, same-session
  control.
- **G2 exactness.** Differential tests; `--evals 1 M` counters match control.
- **G3** G-V1, then G-S as written.
- **G4 soundness.** `make test`, `make lint`; `make test-full` + `make veto`;
  every claim D14-accepted; licence gate unchanged.
- **G5 docs.** `report25.md`, item 33 status, one `bench.md` row at Cap 120 s
  after a passed gate, `history.md` line.

## Risks

- **`undo_move` does not restore cached attack fields** (upstream docs): the
  differential test is the gate; if it diverges, drop SCRATCH-BOARD and keep
  the closure.
- **The closure changes the class of "already Won" replies** (defender
  self-explosion): check defender-commoner extinction first; the differential
  pins it.
- **The screen grows the tree 2.4×** on the probe; it passes only on wall.
- **Null-board invariant**: no null position is keyed, stored, put on `line`
  or emitted; proofs leaving the module are real children's.

## Deliverables

`measurements/plan25/`; `report25.md`; NULL-V1 and, if gated, NULL-S; the
`null_calls`/`null_replies` counters; item 33 status; one `bench.md` row at
Cap 120 s; `history.md` line.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan25.md`. Execute
   Phase A (attribution counters), then Phase B exactly as written (closure,
   scratch boards, defender-eval guard) and prove counter identity at
   `--evals 1 M`. Run G-V1, then G-S only if G-V1 passes. D14 on every proved
   seed. Do not change the 192 MB label, the goal/rungs, or start item 30."
2. G-V1 passes, G-S fails: adopt NULL-V1, keep the screen opt-in, run the
   NULL-MEMO probe next.
3. Both fail: close item 33 with the measured distance; the next kill review
   chooses between NULL-MEMO, the event-log export (item 30) and a goal
   re-pin.
4. Gate passes but `T_4` is not reached: record the residual per-expansion
   gap and name the next dominant term in `report25.md`.
