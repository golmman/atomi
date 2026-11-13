# Movegen Report 2 — `atomic-movegen` 2.7.0 adopted; the wall is flat again

Plan: [`plan2.md`](plan2.md). Initiative: [`initiative.md`](initiative.md),
D-MG1–D-MG7. Handoff answered: [`handoff2.md`](handoff2.md) (MG15–MG18). Data:
[`measurements/movegen/plan2/`](../../../measurements/movegen/plan2/README.md)
(`bench.py`/`bench.json`, `tree_probe.py`/`tree_probe.json`, `pernode.json`,
`pernode_make.json`, `env.json`, README). Host: Ryzen 9 5950X, one process at a
time. Repo dirty at `atomi@f88533e`.

## 0. Verdict — keep

**`atomic-movegen` 2.7.0, gates 1–4 green, paired `m1/m0` geomean 0.987 (tree
identical 3/3), 5 M probe 0.9992×, per-expansion 2,025 ns (m0: 2,000 ns in
the same session). Keep: the pin is neutral on the wall, carries the API the
prover plans need, and no rules divergence exists anywhere.** As with 2.6.0 the
adoption claims no rung (D30/D40 stand); the wall lever is a `prover` claim.

## 1. What was done

- **Step 1 — pin and lock.** `.crate` re-fetched
  (`static.crates.io/.../atomic-movegen-2.7.0.crate`), sha256
  `cd69dc00…d38e1ad3` equal to the index checksum and to the new `Cargo.lock`
  checksum; MIT, MSRV 1.85, edition 2024, 211,311 bytes,
  `pubtime` 2026-11-13T16:20:30Z, not yanked. Extracted in
  `/tmp/plan2-mg/atomic-movegen-2.7.0/`; the crate's own harness as published
  is green: **163 unit + 10 integration binaries (45 cases, incl.
  `perft_all_positions_depth_4`) + 12 doctests**, nothing ignored. `src/zobrist.rs`
  is byte-identical to 2.6.0 (`337af88d…`), so MG11's key schema stands;
  `src/` changed in `board.rs`, `conformance.rs`, `effects.rs`, `lib.rs`,
  `movegen.rs`, `types.rs`, plus the new crate-private `position.rs`.
- **Step 2 — bump.** `Cargo.toml` `"2.6"` → `"2.7"` + `Cargo.lock` (nothing
  else). The frozen surface `tests/fixtures/movegen_2.4.0.tsv` is **unchanged
  and green** — the gate-2 diff is empty — and the fixture writer, run by
  `make test-full` on 2.7.0, regenerates that same sha256 `b8badd09…`:
  no move order, perft value, key, clock or outcome moved. `make test` 193
  passed / 5 ignored, `make lint` green.
- **Step 3 — the child-surface differential** (test only, §4.1 for the file
  layout): `tests/child_surface_differential.rs` pins MG15–MG18 against **our
  own built child** (`clone` + `do_move`), on the frozen corpus (94 positions
  incl. every bench cell) plus 2 plantings of seeded play-outs (2,620 nodes):
  **51,334 children** (48,562 legal, 2,772 illegal, 375 of those with a
  decided child). Per child: `post_move` equals the built child's placement
  (occupied, both colours' and all six piece-type bitboards, all 64 squares),
  side, ep square, castling rights and clock, `casualties`/`is_capture` equal
  `blast_casualties`/`is_capture`, `effect_and_post` equals the two separate
  queries; `terminal_after` equals the built child's `terminal()` (reason
  included), `is_terminal_after` its `is_terminal()`, `has_legal_move_after`
  its `has_legal_move()`, `outcome_after` its `outcome()`; `child_outcome` is
  `Illegal` iff `!is_legal` and `Terminal(t)` iff the child is terminal;
  `classification` equals the effect's rules fields. On the fixture's own
  columns the `PostMove` casualties, clock and child outcome equal the frozen
  2.4.0 `M` rows. **Sensitivity check:** three deliberate mutations of the new
  assertions (clock off by one, the legal-only terminal arm, a flipped
  `is_capture`) each failed the test in the expected row, so the differential
  is not vacuous.
- **Step 4 — downstream.** `make test-full`: slow tier 4/4 (`wins_oracle`,
  `wins_oracle_hand`, `never_prove_oracle`, `false_refutation_regressions`);
  `make veto`: **8/8 accepted, 4/4 controls rejected, `provari check` agreeing
  everywhere**. Counters unmoved: the 5 M `deep-win` probe is the plan28/plan1
  control tree (5,000,034 child evals / 168,910 expansions) on **both** sides,
  and every paired bench cell and every per-node cell has identical
  `child_evals`/`expansions`.
- **Step 5 — measure.** Both sides from the identical source tree (`m0` =
  2.6.0 in `/tmp/atomi-m0-plan2`, `m1` = 2.7.0 here), three cells × Cap 10 s and
  120 s × 3 repeats, `provari --guide hand` (10 s) and both sides alternating
  per repeat; 5 M tree probe; `run_pernode` both sides; `make pernode`.
- **Step 6 — this report** plus the ledger/history/README update and two dated
  `prover/bench.md` rows.

## 2. Numbers

| | m0 2.6.0 | m1 2.7.0 | `m1/m0` |
| --- | ---: | ---: | ---: |
| `easy-win` 10 s | 1.868 s | 1.840 s | 0.985 |
| `core` 120 s | 18.093 s | 18.194 s | 1.006 |
| `easy-win` 120 s | 1.906 s | 1.850 s | 0.971 |
| paired geomean | | | **0.987** |
| 5 M `deep-win` median `ms` | 2,583 | 2,581 | 0.9992 |
| `core` 10 M ns/expansion | 2,000 | 2,000 | 1.000 |

Tree identical on every paired cell (`child_evals` 30,690,269 /
281,668,134, `expansions` 1,062,931 / 9,796,753) and on the 5 M probe.
Coverage did not move: 1/3 at Cap 10 s (`easy-win`, factor 14.046; `m0`
14.260), 2/3 at Cap 120 s (geomean 21.468; `m0` 21.730), `deep-win` censored
at both caps on both sides — the plan1/plan11 bare-tree pattern. The bench
stack is the **bare** tree (`--guide none`, D28), so these rows are not
comparable with the hand-stack rows (bench reading rule 4).

## 3. Release outcomes — 4 of 4 landed (D-MG7)

Verified from the 2.7.0 tarball and `CHANGELOG.md`: **MG15** `terminal_after`
+ the `Board` wrapper, `is_terminal_after`, `has_legal_move_after` (shape (a),
no board write); **MG16** `AfterState::{Illegal, Undecided, Terminal}` via
`child_outcome`, `outcome_after` = its `.outcome()` with the clone fallback
removed; **MG17** `PostMove` via `post_move`/`effect_and_post` (facts only);
**MG18** `MoveClassification` + `classification()` (no mover-commoner
`is_attacked_by`), sharing one derivation with `effect()`. Bonus, unasked:
`conformance::CHILD_CASES` (one row per `AfterState` branch and per
`TerminalReason`) and the doc-only `outcome_after` trap fix. **No atomi call
site uses any of it yet.**

## 4. Problems and challenges

1. **Scope change: the differential's file layout.** The plan says "extend
   `tests/effects_differential.rs`". Doing that inline would have produced one
   ~20 KB test file (AGENTS: ~10 KB), so the frozen-tsv reader moved to
   `tests/common/mod.rs` and the child surface got its own
   `tests/child_surface_differential.rs`; both are under 8 KB and read one
   parser. The evidence is the same, the layout is not.
2. **The crate harness count in plan §3 was off by one binary** ("9
   integration binaries"): 2.7.0 has **10** (`child_state` is new). Test
   *cases* went 26 → 45. No action, recorded for the next adoption.
3. **`Outcome` and `TerminalReason` are `#[non_exhaustive]`**, so the atomi
   mapping table cannot be written as an exhaustive match; the test has a
   wildcard arm that **panics loudly** on a new variant instead of silently
   mapping it — the same reason `rules.rs` maps `Some(_)` conservatively.
4. **The reason histogram is worth keeping**: the corpus reaches all five
   `TerminalReason` branches (Extinction 185, FiftyMoveRule 176, MaterialDraw
   107, NoMovesInCheck 85, Stalemate 89), more than the plan assumed; the test
   asserts the four non-clock branches appear and prints all five.
5. **Cross-session per-node drift is ~2.7 %.** 2.6.0's S4 point was 2,054
   ns/expansion (plan1 session) and is 2,000 in this one on the same binary
   pin; 2.7.0's point is 2,025. The trend line must be read as a session-normalized
   series, exactly like the wall rows (bench reading rule 1).
6. **`make pernode` reports a null `gate` block** because the target runs one
   side named `atomi`, not `m0`/`m1`; `m1_reference` is the S4 point and was
   right. Harmless, but the block is meaningless for a single-side run.
7. **Reproducibility bonus**: rebuilding `m0` from a fresh copy of the current
   `src/` produced the binary the drafting session built (sha256 `a70823f5…`),
   so "same tree, only the pin differs" is checkable, not asserted.
8. **Session spread** is ±2–3 % on the same binary; 0.987 / 0.9992 / 1.000
   are all inside it. The honest statement is **flat**.

## 5. Plan §9 questions, answered

- **1. Material draw.** Pinned in `terminal_after_pins_the_atomi_mapping`:
  all 20 crate `MaterialDraw` children of the corpus satisfy atomi's broader
  clause, and **182** legal children are `Failed` for atomi (its Lichess
  clause) while the crate reports *undecided*. So a prover call site must map
  the **reason** (or re-test with `rules::material_draw` on the child facts),
  never read `None` as "the game continues". MG16's `Illegal` arm is what makes
  this safe: `child_outcome` separates "illegal" from "undecided", and only
  the latter needs atomi's own draw test.
- **2. Pseudo-legal input.** Now measured, not assumed: on 2,772 illegal
  candidates `terminal_after` reports whatever the *unreachable* child is —
  375 of them a decided child — while `child_outcome` says `Illegal` for every
  one. A call site that holds only pseudo-legal candidates must use
  `child_outcome` (or `is_legal`) for trust and may use `terminal_after` alone
  only for moves it already knows are legal. `game_over` parents are covered
  by the fixture's terminal rows (every move `Illegal`).
- **3. Does `PositionView` move the per-expansion cost?** **No.** `core` 10 M:
  2,000 → 2,000 ns/expansion with identical `child_evals`; `easy-win` 1,851 →
  1,776 and `deep-win` 1,747 → 1,714 (both inside noise). The S4 trend point
  for 2.7.0 is **2,025 ns/expansion** (`make pernode`, `core` 10 M, 192 MB).

## 6. Missing tests / open parts

- The differential pins the crate to **our built child**, not to our *search*:
  no atomi code calls MG15–MG18 yet, so the call-site equivalence is unguarded
  until a `prover` plan integrates them (it must keep the child
  `Outcome`/`Terminal` precedence and the `Illegal` arm).
- Perft depth 5 is still out (unchanged from report1 §6); a rules-announcing
  release is the trigger for that tier.
- The play-outs are a fixed 2,620 nodes from the corpus FENs and do not read
  the crate's `CHILD_CASES`; cheap to add if a release touches the child view.
- `unsafe-optimizations` remains out of scope (D-MG6).

## 7. Next steps (kickoff context for the `prover` plan)

1. **MG15 first**: replace the `clone + do_move + terminal` fallback in
   `classify.rs`/`mover.rs` with `terminal_after` (keeping `child_outcome`
   where the illegal/undecided split matters). Authorised: the probe's
   `needs_board` share is **21.3 %** of hand-stack children (29.9 % `core`
   1 M), over the 10 % gate that handoff2 §3 replaced. Gate it as a prover
   plan: tree identity at fixed evals, same-session wall against a control
   binary, `make veto`.
2. **MG18** (`classification` for moves from `generate_legal_with_state`)
   removes a second legality query — same style of gate, smaller.
3. **MG17** (`post_move`) for clock/casualty reads where a call site builds
   the child anyway.
4. `handoff3.md` stays **unspent**: no new ask today (D-MG7).

## Keep/revert line (gate 5)

**Keep** — `atomic-movegen` **2.7.0**; gates 1–4 green (crate harness
163 + 45 + 12, frozen differential **diff empty**, new child-surface
differential green and mutation-checked, `make test-full` slow tier 4/4,
`make veto` 8/8 accepted / 4/4 controls rejected); paired `m1/m0` geomean
**0.987** at tree identity 3/3 (5 M probe 0.9992×, per-expansion 2,025 vs
2,000 ns); **reason:** flat on the wall, no rules divergence, and it is the
published version that carries MG15–MG18 for the prover plans.

## Commit sequence (the owner commits)

1. `deps: atomic-movegen 2.7.0` — `Cargo.toml` + `Cargo.lock`, nothing else;
   the frozen differential is green and unchanged.
2. `test(movegen): pin the 2.7.0 child surface` — `tests/common/mod.rs`,
   `tests/child_surface_differential.rs`, the `tests/effects_differential.rs`
   reader extraction.
3. `docs(movegen): report2, measurements and bench rows` —
   `docs/plans/movegen/{plan2,report2,initiative}.md`, `docs/plans/README.md`,
   `docs/plans/prover/bench.md`, `measurements/movegen/plan2/`.
