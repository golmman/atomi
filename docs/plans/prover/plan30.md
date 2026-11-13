# Prover Plan 30: item 37 — MG15 integration, a measured no-go before implementation

Initiative: [`prover`](initiative.md), backlog **item 37**, **D42** (to
propose); D2, D11, D19, D21, D24, D26, D30, D34, D39, D40, D41. Upstream:
`atomic-movegen` 2.7.0 **MG15–MG18**, [`../movegen/report2.md`](../movegen/report2.md)
§5/§7.1, [`../movegen/handoff2.md`](../movegen/handoff2.md) §3. Drafted
2026-11-13 after [`report29`](report29.md), [`analysis2`](analysis2.md) §5 and
[`measurements/movegen/plan2/`](../../../measurements/movegen/plan2/README.md)
(bare tree 2,025 ns/expansion, 21.3 % `needs_board`). Read-only session; scratch
probe outside the repo.

## Kill review (fixed prompt)

1. **Distance.** `T_3`; `T_4` needs `deep-win` **7.17×** (D39). Tree at parity
   (D36); wall = per-expansion cost. A perfect board-free terminality is worth
   **≤ 1.03–1.06×** (§2) — **no rung**.
2. **Would we build it today?** No: the API is **slower than the code it
   deletes** (65.3 vs 58.0 ns per built child, 1.13×), and on the product stack
   the build it would drop is **still required by the guide** (§0). Not a
   micro-line re-open (D40); the first *negative adoption* since item 36.
3. **Cancel?** Yes — already: the probe fires the stop rule in this session,
   so the deliverable is the record, not a patch.
4. **Prior art (delta).** Crate pin 2.6.0 → 2.7.0 (movegen plan2); MG15
   *asked → published*; no provari row changed, no oracle pin change.
5. **Smallest next experiment.** The probe below — done; 1,248 positions,
   39,665 legal moves, five harness runs.

## 0. Push-back on the request (two premises are wrong)

The kickoff says "replace the clone + `do_move` + terminal fallback in
`src/classify.rs` and `src/mover.rs`". Measured against the tree:

- **The terminal fallback is not there.** `src/mover.rs:112-130` only *flags*
  `needs_board`; it never builds a child and never asks for a status.
  `src/classify.rs:178` is the read-only `atomi classify` diagnostic, off the
  search path. The one site on the hot path is `src/search/frame.rs:282-300`
  (build → `rules::status(&child, attacker)`), with further builds at `:328`
  (settle) and `:394` (`child_eval`).
- **A frame swap cannot be tree-identical, and cannot be faster.** Under
  `--guide hand` the board is read by the guide for every built child:
  `child_eval = hand_eval(child)` at attacker nodes (`ef_pn = OrOnly`),
  `classify_exact`'s `gives_check`/`immediate_explosion`/Hanging test
  (`guide/class.rs:127-160`), `defence_likely_to_refute`'s check bit, the
  `closure_bits` settle. Dropping the build moves classes, hence the tree;
  keeping it and *adding* `terminal_after` pays both.

## 1. What MG15 actually costs (measurement)

`/tmp/probe30` (MIT crate only, `=2.7.0`, `--release`, `lto`): 1,248
positions from random play-outs of the three bench FENs, 39,665 legal moves,
best of 5 timed passes, checksums black-boxed. Per **legal move**, one
`populate_state` + `EffectContext::new` per node (0.3 ns/move).

| Path (per child) | built-ish | captures | quiet |
| --- | ---: | ---: | ---: |
| today: `clone`+`do_move`+`outcome()` (+`Legal` hash) | **58.0** | **64.0** | 43.0 |
| MG15: `hash_and_rule50_after`+`ctx.terminal_after` | **65.3** | **82.8** | 45.0 |
| build only (`clone`+`do_move`) | 31.8 | 43.1 | 17.9 |
| `hash_and_rule50_after` / `ctx.post_move` (MG17) alone | 7.5 / 18.1 | | |
| `ctx.child_outcome` / `has_legal_move_after` | 51.7 / 45.4 | | |

`built-ish` = capture, promotion, ep, castling or commoner move (atomi's
`needs_board` minus the quiet-check prediction). **MG15 is 1.13× today's cost
on the class we build (1.29× on captures).** Mechanism: `terminal_after`
*re-derives* the post-move view (`analyze` = 18.1 ns, 31 % of the build), then
runs `populate_state` + a **view** movegen that is not cheaper than `do_move`
+ the board movegen: 29.9 ns vs 23.4 ns. The amortised form
`terminal_from(&PostMove)` (18.1 + 29.9 = 48.0 ns) is **private**.

Stop rule (declared before the number, per D21): *no-go if
MG15 ≥ 1.0× the build+`outcome()` cost on `built-ish`*. Measured 1.13×:
**no-go**.

Caveats: not atomi's tree (random play-outs), no `material_draw` in the
"today" column (bit tests, ~2 ns — and MG15 needs atomi's own clause anyway,
report2 §5.1). Both sides pay the same `populate_state`/movegen shapes, so
the sign is not a corpus artefact.

## 2. Ceiling arithmetic

21.3 % `needs_board` × 29.6 children/expansion ≈ **6.3 builds/expansion**;
even at the cheapest measured build+status (58 ns) that is **≈ 315 ns of
2,025 ns/expansion (15.5 %)**. A free, exact, board-free terminality would
cap at ~1.18× on the bare stack — and under `--guide hand` the guide keeps
the build (§0), so the realisable share is ~0 %. Same shape as item 36 from
the other end: that one could not *decide* the children, this one cannot
*afford* to.

## 3. Deliverable for the execution session (measurement only)

No engine change, no flag, no default, no bench row — a record plan:

1. `measurements/plan30/{probe, probe.json, env.json, README.md}`: the scratch
   crate `/tmp/probe30/{Cargo.toml,src/main.rs}` copied verbatim (if gone:
   xorshift play-outs, 12×40 plies from the three bench FENs, best-of-5 timed
   passes over the five variants of §1's table), parsed output, host/`atomi@<sha>`/
   crate checksum in `env.json`.
2. `gotchas.md`: one row — **MG15's view movegen is not cheaper than building
   the child** (65.3 vs 58.0; `analyze` 18.1 of 48.0), and `terminal_from` is
   private, so `post_move` + terminality always pays `analyze` twice.
3. `initiative.md`: item 37 **closed `no effect`, pre-measured**; D42
   proposed ("MG15 declined on measurement; the child build stays the
   prover's terminality path"); next plan = the formulation decision.
4. `../movegen/handoff3.md` (delta, ideas only): MG15 shipped, but the bar was
   soundness, not cost — the ask narrows to (a) `PostMove::terminal()`
   (amortised, ~48.0 vs 58.0) and (b) a specialised existential-legality test
   (handoff2 shape (b)); MG16–MG18 cost 0 in atomi (`Mover` is ours).
5. Gates: `make test`, `make lint`, `make test-full`, `make veto` green
   **unchanged** (no engine code moves); `tests/child_surface_differential.rs`
   stays the crate pin.
6. `report30.md` recording §1–§2 plus "no probe run on the tree".

## 4. If the owner overrides (Plan 31, conditional)

Only with a new crate API in hand (`PostMove::terminal()`), gated as one
stack: post-move facts (MG17) + `checks_enemy` (MG18) + atomi-side board-free
`classify`/Hanging predicates, so **defender-node** children (where
`child_eval` is 0) can skip the build. Ceiling: (58.0 − 48.0) ns on a
convertible share of ≤ 10 % of children ⇒ **≤ 1.05× wall**; gate G-M15: 5 M
`deep-win` seed 0, counters identical, median wall ≤ 0.95× control, then
120 s × seeds 0–2 + D14. It moves no rung.

## 5. Options (follow-ups)

1. **Execute this plan** (record + handoff3), then take the formulation fork.
2. **Formulation fork first**: PDS-PN/PN² or a goal re-pin under D30 — the
   initiative's standing next decision; this plan only clears the queue.
3. **Formulation-changing option** (not a plan): a *sufficient* "certainly
   not terminal" screen — one demonstrably legal reply (safe quiet move, no
   check, no pin, no commoner contact) proves the child is not mate/stalemate
   with no movegen; it attacks the question, not the answer. Gain < 2 % wall
   under `--guide hand` (the guide still reads the board), so it only matters
   **after** `child_eval`/`classify` stop needing the child. Cheapest
   experiment: Phase-A counters for the child-free class/eval share.

## Kickoff (option 1, copy-pasteable)

> Execution session — prover plan30 (item 37, MG15 declined). Implement
> `plan30.md` §3 only: copy `/tmp/probe30` into `measurements/plan30/` (+
> `probe.json`, `env.json`, README), the `gotchas.md` row, item 37's closure +
> proposed D42 in `initiative.md`, `../movegen/handoff3.md`, `report30.md`. No
> engine change, no bench row. Gates: `make test`, `make lint`,
> `make test-full`, `make veto` green unchanged.
