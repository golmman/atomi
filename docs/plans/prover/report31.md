# Report 31 — item 38 (HOTPATH-V2): adopted, 0.72–0.78× tree-identical

Plan: [`plan31.md`](plan31.md). Initiative: [`initiative.md`](initiative.md),
item 38, decision **D43**. Measurements:
[`measurements/plan31/`](../../../measurements/plan31/README.md). Engine
change: `src/guide/{threat,null,class,replay}.rs` + one new integration test.
Nothing in this report came from provari source; the oracle is an external
process (GPL boundary).

## 1. Verdict

**Adopted.** The stack (F1 existence-without-list, F2 `Board::null_for`, F3
reply stream) is tree-identical on every measured pair and 0.72–0.78× the
same-session control:

| Gate | Bar | Result |
| --- | --- | --- |
| G-V2a exactness | equality on the corpus; 1 M counters equal | **pass** — 85,819 positions + 91,249 children, 0 disagreements; 57,214 null boards; identical `child_evals`/`expansions` on 3 cells |
| G-V2b (the decision) | 5 M `deep-win`, tree identical, median ≤ 0.80× | **pass** — 0.7773× (control 2,546 ms → 1,979 ms), `5,000,034 / 168,910` on every run |
| G-V2c | 120 s × seeds 0–2, every cell ≤ 0.80×, none > 1.0× | **pass** — 0.7223–0.7541× over 9 (cell, seed) pairs, counters identical |
| G-V2d | `make test`/`lint`/`test-full`/`veto`; D14 9/9 | **pass** — slow tier green, VETO PASS, 9/9 claimed by both checkers |
| G-V2e | one `bench.md` row at Cap 120 s | **pass** — geomean **7.23** (3/3 claimed), best row so far (was 9.681) |

`deep-win` now proves in **58.4 s** at the 120 s cap (control 77.7 s median
over seeds), so `T_4`'s distance is **5.84×** in wall (D39 recorded 7.17×).

## 2. Phase 0 — the arithmetic closes on 2.8.0

The plan demanded that the attribution be re-derived on the shipped pin before
a patch, and that the numbers close. They do
(`measurements/plan31/phase_a.json`, `deep-win`, 5 M evals, 168,910
expansions):

- **26.2** explosion-test calls per expansion (the diagnosis said 24.4), each
  building and scanning a full **38.2**-move list (879.7 of 1,000.5 entries
  read before the answer) to decide a ≤ 4-square bit question; 5.8 of the 26
  calls answer "yes".
- Those calls are **66.2 %** of all list moves read per expansion (the
  diagnosis said 63 %), the null reply streams 28.2 %, the null board 3.4 %,
  the frame's own child list 2.0 %, the replay lists 0.3 % — 1,512 moves read
  per expansion.
- Site split of the 26.2 calls: null reply test **16.5** (63 %), guide settle
  bits **7.9** (30 %), class + replay + full settle **1.8** (7 %).
- `perf` self time on the control: `generate_pseudo_legal_in_view` **21.0 %**,
  `immediate_explosion_bits` **7.4 %**, `from_fen`+`fen_with` 2.1 %. Moves are
  ~21 % of wall, so a list-free explosion test alone cannot reach 0.687×; the
  three fixes together can, and do (candidate profile: 8.3 % / 0 % / 0 %).

**Scope correction (recorded, not patched):** the diagnosis' "1,486 moves per
expansion" counts moves *generated*; my counters count moves *read* from the
lists (1,512/exp including the streams whose tails are never read). Both close
the same arithmetic; the plan's 63 % and 24.4/expansion are 66 % and 26.2 on
2.8.0.

## 3. What shipped

- **F1 `threat::has_explosion(board, side) -> bool`** — no legal list, no
  child. Targets = `Board::blast_zone(enemy commoner) & enemy pieces` (the
  blast zone is symmetric, so `to ∈ zone` ⟺ `commoner ∈ blast_zone(to)`);
  attackers = one `attackers_to_color` lookup; confirmation =
  `blast_casualties(..).all() & enemy commoners` **and** the exact
  `Board::legal`. En passant and promotion-capable pawn attackers fall back to
  `immediate_explosion`. Two sites moved: `guide/class.rs` (`WeakThreat`
  question) and the null reply test.
- **F2 `null_board` = `Board::null_for(attacker)`** — infallible now
  (`Board`, not `Option<Board>`); the FEN/`split`/`join`/`from_fen` round-trip
  is gone. Still ephemeral: never keyed, stored, on `line` or emitted.
- **F3 `exposed_limit_uncached`** walks `for_each_legal_with_state` with
  `ControlFlow::Break`; no reply list. The stream visits exactly
  `generate_legal`'s sequence in its order, so the reply count (`work.replies`)
  and the limit are the list path's.
- **Witness split honoured.** `immediate_explosion`,
  `immediate_explosion_bits`, `one_ply_settle(_bits)`, `null_win`'s move
  selection and `replay.rs`'s witness line are byte-unchanged (`git diff` shows
  additions only in `threat.rs`); the only F1 use in `replay.rs` is F2's call
  signature.

Five single-line-ish facts the code now states, all pinned by tests:

1. **A commoner capture is never a legal explosion** (the capturer is
   destroyed at ground zero), so masking commoners out of the attackers is
   exact. *First draft did not mask*, because a test FEN I had mis-read (the
   pawn was on the fifth rank, so the "capture" was quiet) made a both-gone
   capture look legal. Corrected before the gate; the hand case
   `k7/p7/K7/8/8/8/8/8 w` pins it (the move list has no `a6a7`).
2. **An extinction screen is required.** `Board::legal` still accepts a
   capture when our commoner is gone, while `generate_legal` emits nothing;
   without the screen `has_explosion` answered `true` where the list path
   answers `None`. The random play-outs stop at extinction, so only the hand
   case `k7/8/8/8/8/8/8/R7 w` catches it — the kind of asymmetry a differential
   over product nodes alone would have missed.
3. **The movegen does not prune "pinned" attackers** (`d3xe4` by a pinned
   bishop stays in the legal list), so neither may the existence test: the
   blast and `Board::legal` decide, never a pin rule of our own.
4. **F2 is exact beyond the FEN text**: key, half-move clock, fullmove number
   and the derived game ply agree with the round-trip on 57,214 (position,
   attacker) pairs, 322 of them with an en-passant square.
5. **The fallback is exercised, not assumed**: a synthetic sweep of 100
   promotion-capture and 184 en-passant positions (both colours, every pawn
   file, both capture diagonals, every placement of the enemy commoner in the
   blast zone) runs through the same differential.

## 4. Ablations (D21, 5 M `deep-win`, 3 interleaved repeats)

`{F1}` 0.8920×, `{F2}` 0.9584×, `{F3}` 0.9305×, `{F1+F2+F3}` **0.7773×**. The
product of the single ratios is 0.795, so the stack is mildly super-additive
(F3 and F1 share the legality/attack work). Order of value: F1 > F3 > F2 —
F2 was the smallest single piece (2.1 % of wall) yet the cheapest to adopt.

## 5. Problems and open parts

- **The plan's "0.687×" was a 120 s figure, not a 5 M one.** At 120 s the stack
  reproduces the prototype's shape (0.72–0.75×); at 5 M it is 0.777×. The gate
  was declared at 5 M and is met there; both numbers are in the record so no
  future row can quote one as the other.
- **The reference moved less than the set card suggests.** The `deep-win`
  provari reference measured **4.459 s** here (0.527 s `core`, 0.135 s
  `easy-win`), within 1 % of plan27's own same-session reference (4.413 s), so
  the two rows' factors happen to be comparable at ~1 % — but rule 1 still
  forbids *reading* across sessions, and rows are never rewritten. The set
  card's 6.2–6.8 s is the plan1 artefact, not a drift in this session. The
  cross-row statement this report makes is therefore the wall: 77.4 → 58.4 s on
  `deep-win`, geomean 9.681 → 7.23 on the same reference, 5.84× from `T_4`.
- **What is left in per-expansion cost** (candidate `perf`, self time):
  `Tables::lookup_with_source` 16.6 %, `Frame::enter` 9.3 %,
  `generate_pseudo_legal_in_view` 8.3 % (now the frame's child list and the
  reply stream's walk), `do_move` 7.2 %, `legal_with_counts` 6.9 %,
  `attackers_to_color` 6.0 %, `checkers` 5.3 %, `null_win` 5.5 %. The reply
  stream still builds 14 scratch children per expansion and reads 28 % of the
  moves: plan31 §5's **parent-side null closure** is the next guide lever
  (estimated 1.5–2×) and this profile is its starting point.
- **Not taken (deliberately, out of plan scope):** `has_explosion` calls
  `populate_state` once per call (26/expansion) although the state is only
  needed when a candidate reaches `Board::legal`; a lazy/pass-through state
  variant is worth ≤ 0.5 % of wall by the profile and needs its own tree gate.
- **`null_win`'s `work.moves` semantics are untouched** (full legal count, not
  moves scanned), as `gotchas.md` requires; F3 changed only the reply pass.

## 6. Missing tests

None known for the two rewrites. The existence differential runs on every fast
`make test` (0.14 s): 85,819 positions, 91,249 children, 11 hand cases, the
promotion/en-passant sweeps, and the null-board equality. The reply stream is
pinned by the pre-existing `exposed_limit` differential (result **and** reply
count against the clone-per-move reference) plus the veto fixtures, which run
the null replay end to end and both checkers.

## 7. Next steps

1. **Item 39 (PROOF-FILTER)** is now the largest single remaining term
   (`lookup_with_source` 16.6 % self; ~18 % of the profile per plan31's
   ordering) — its gate is `≤ 0.92×` on the three 120 s cells, tree-identical.
2. The **formulation fork** stays the standing `T_4` decision (D30): the stack
   moved `deep-win` 77.7 → 58.4 s, and even a perfect 2× per-node leaves it at
   ~29 s against a 10 s rung. Nothing here changes that arithmetic.
3. If a guide-side step is preferred over item 39, plan31 §5's parent-side
   null closure is the measured next step (28 % of the moves read, 14 builds
   per expansion); it needs the `rules::status` reproduction exactness bar that
   items 24/25 failed on, so it is a plan, not a patch.
