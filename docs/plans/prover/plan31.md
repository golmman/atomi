# Prover Plan 31: item 38 — HOTPATH-V2, the guide's per-expansion multiplicity

Initiative: [`prover`](initiative.md), backlog **item 38**; decisions **D11**,
**D14**, **D21**, **D26**, **D30**, **D34**, **D39**, **D40**, proposed
**D42**, **D43**. Drafted 2026-11-14 after [`report30`](report30.md) and the
2026-11-14 diagnosis; source probes only.

## Kill review (fixed prompt)

1. **Distance.** `T_3` (geomean 9.681 at Cap 120 s, D39). `T_4` needs
   `deep-win` **71.7 s → 10 s = 7.17×**. The tree is at parity (D36); the wall
   is per expansion (13.4–16.6 µs against provari's 1.45 µs/node), caused by
   multiplicity in **our own guide**: 1,486 moves per expansion, 63 % from the
   one-ply explosion test.
2. **Would we build this today?** Yes, as a *different* mechanism from D40's
   closure: D40 judged a 1.22× stack of builds; this removes guide closures
   (lists, a FEN round-trip), not search work.
3. **Cancel?** No — but the item is **narrowed** (§2): only *existence*
   questions change, and every path whose output is a **move** stays
   untouched, so tree identity is structural, not measured.
4. **Prior art (delta).** Crate pin 2.4.0 → **2.8.0** (movegen plan3):
   `Board::null_for`, `blast_zone`, `attackers_to_color`,
   `for_each_legal_with_state` are **published** (plan28's MG1/MG2 were
   prototyped, never adopted). No provari row, no oracle pin change.
5. **Smallest next experiment.** G-V2b, the 5 M `deep-win` tree-identical
   probe (≈ 30 s): it decides the whole item.

## 0. Push-back on three premises

- **"Three new fixes."** F2 and F3 *are* plan28's S1+S2, already measured as a
  tree-identical **0.8206×** candidate and closed by D40. F1 is the new
  element; plan28's profile is its case — after S1+S2+S3
  `immediate_explosion_bits` is still **14.0 % self** while
  `generate_pseudo_legal` is gone (plan28 README).
- **The 63 % is uncommitted scratch evidence**, taken across two pins. Phase 0
  re-derives it on 2.8.0, and the arithmetic must close: moves are ~21 % of
  wall, so a list-free explosion test alone cannot yield 0.687× unless the
  removed lists also carry `legal`/`populate_state`/alloc. If Phase 0
  disagrees, the item is re-scoped, not patched.
- **`report30` §7 says the next plan is the formulation fork; the
  initiative's later revision says item 38.** Item 38 runs first and **does
  not substitute**: at ~0.69×, `deep-win` lands near 49 s, still ~4.9× from
  `T_4`. The fork stays the next `T_4` decision.

## 1. Why this

Per expansion the guide builds 38.4 pseudo-legal lists, calls the one-ply
explosion test 24.4 times (full ~39-move list, scan 38.9, for a ≤ 4-square bit
question), materialises 14 reply lists and FEN-round-trips the null board. All
three are tree-neutral **if** only existence answers and order-preserving
streams change.

## 2. The stack (HOTPATH-V2)

- **F1 EXISTENCE-WITHOUT-LIST.** New `threat::has_explosion(board, side) ->
  bool`: targets = `Board::blast_zone(enemy commoner) & enemy pieces`; for each,
  attackers = `attackers_to_color(tsq, us, occ) & !commoners(us)`; candidate =
  `Move::make_move(from, tsq)`, confirmed by `blast_casualties(m).all() &
  enemy_commoners` **and** `board.legal(m, &state)` (pins and commoner
  proximity stay exact). Any en-passant target or promotion-capable pawn
  attacker **falls back** to today's `immediate_explosion`. Only the two
  existence-only sites move: `guide/class.rs:126` and `guide/null.rs:214`.
- **F2 NULL-BOARD-IN-PLACE.** `null_board` uses `Board::null_for(attacker)`
  instead of `fen_with` → split/join → `from_fen`. The null position stays
  ephemeral: never keyed, stored, on `line` or emitted.
- **F3 REPLY-STREAM.** `exposed_limit_uncached` scans through
  `for_each_legal_with_state` with `ControlFlow::Break`, no materialised list.

**Witness split (the scope refinement).** `immediate_explosion`,
`immediate_explosion_bits`, `one_ply_settle(_bits)`, `null_win` and
`replay.rs:256` return a **move** that reaches a settle, a proof entry or the
parent's selected move; their bodies and pseudo-legal order stay unchanged, so
no tree-changing output can drift. If Phase 0 puts the witness sites above 5 %
of the remaining wall, a re-ordered witness (F1′) is a **separate** item with
its own tree gate.

## 3. Phases

- **Phase 0 — attribution (read-only).** Temporary counters on a scratch 2.8.0
  copy: calls per explosion site, moves scanned, lists built, FEN builds,
  per-branch reply counts of `exposed_limit`; 1 M evals on three cells →
  `measurements/plan31/phase_a.json`; reverted before committing.
- **Phase A — differential harness** (`tests/explosion_differential.rs`, null
  tests): fixtures + ≥ 50 k positions from random play-outs of the three bench
  FENs + hand cases (promotion-captures, en passant, a capture outside the
  zone, a pinned attacker, a commoner-adjacent piece).
- **Phase B — F1–F3** plus tests. **Phase C — G-V2b** and the D21 ablations
  `{F1}`, `{F2}`, `{F3}`, `{F1+F2+F3}` at 5 M. **Phase D — G-V2c.** **Phase E —
  publish**: D14, `make test-full` + veto, `bench.md` row at Cap 120 s,
  `report31.md`, item 38 status + D43, `history.md` line.

## 4. Gates (declared before data)

- **G-V2a exactness (veto).** `has_explosion == immediate_explosion(..)
  .is_some()` on the corpus; `null_for` equal to the FEN round-trip in FEN
  string, `hash_with(Legal)`, `rule50`, `fullmove_number`; `--evals 1M`
  counters equal to the control on all three cells.
- **G-V2b (the decision).** 5 M `deep-win`, seed 0, 3 interleaved repeats:
  `child_evals` **and** `expansions` identical, median ≤ **0.80×**. A miss at
  ≥ 0.85× closes item 38 (next: item 39); 0.80–0.85× goes to the owner,
  recorded not rewritten.
- **G-V2c.** 120 s × seeds 0–2: counters identical per (cell, seed), every
  cell ≤ 0.80×, none > 1.0×.
- **G-V2d.** `make test`, `make lint`, `make test-full`, `make veto`; D14 on
  every proved seed (target 9/9), both checkers.
- **G-V2e.** One `bench.md` row at Cap 120 s; `env.json` pins the binary
  sha256, the 2.8.0 checksum and the provari commit.

## 5. Ideation — the formulation that beats this

**Parent-side null closure.** F3 removes a list, not the **build** per reply
(14 children/expansion). Answering "does every defender reply leave us an
explosion?" from the parent board with bit closures (target sets, parry
squares, pin candidates) removes those builds: estimated **1.5–2×**, the bulk
of the ~9.4 µs surviving F1–F3. Risk: reproducing `rules::status` exactly is
the soundness surface items 24/25 failed on. **Cheapest experiment:** Phase 0's
per-branch counter — a plan only if ≥ 80 % of replies reach the
`has_explosion` verdict.

## 6. Risks

- **The scratch prototype may not reproduce on 2.8.0**: Phase 0 and G-V2b
  decide, and no 120 s run happens before G-V2b passes.
- **F1 completeness.** En passant and promotion-captures are the holes; a
  missing candidate is a false `None` (a lost settle, never a false win) that
  moves the tree — hence the fallback and G-V2a.
- **F3 order and `work.replies`.** `replies` is observation-only and `moves`
  (the budget unit) comes from the null board's legal list — but identity must
  be **tested**, not assumed (plan28's S2 was restricted for this).
- **F2 semantics.** ep, clock, fullmove and game ply must match the FEN path.

## 7. Options

1. **Execute this plan** (F1–F3 with the witness split).
2. **F1 only** — the item's only new mechanism; cheapest, F2/F3 return later.
3. **Skip to the formulation fork** (PDS-PN/PN² or a goal re-pin under D30):
   one decision per session; `deep-win` stays at 71.7 s meanwhile.

## Kickoff (option 1, copy-pasteable)

> Execution session — prover plan31 (item 38, HOTPATH-V2). Implement
> `plan31.md` §2–§3: Phase 0 into `measurements/plan31/`, then F1
> `threat::has_explosion` at the two existence-only sites, F2 `Board::null_for`,
> F3 `for_each_legal_with_state` + `Break`; witness functions untouched. Gates
> G-V2a → G-V2b → G-V2c → G-V2d → G-V2e; no 120 s run before G-V2b. Then
> `report31.md`, item 38 status + D43, `bench.md` row at Cap 120 s.
