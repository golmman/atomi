# Correctness gotchas (binding)

Short index of bugs already found and the invariant that prevents them. These
are load-bearing: a change that breaks one needs a test and a report finding.
Referenced from `AGENTS.md` §Correctness.

## Search and values

- **`Open` numbers saturate at `OPEN_MAX = INFINITY - 1`, never `INFINITY`.**
  `INFINITY` marks decided values only. An `Open` value that saturated at
  `INFINITY` invented false refutations (plan1).
- **Proofs never live in the TT.** TT partial keys are sound only because a
  collision can cost a win, never invent one. Every proof goes to the full-key
  proof table.
- **The filter changes no result** (no false negatives). A test pins identical
  counters with it on and off.
- **Lazy stalemate (D19, narrowed by G1).** A quiet, non-checking child starts
  `(1, 1)`; its stalemate is found at node entry, **unless** the child's side
  to move has only its commoner and pawns, in which case every child is built.
  Without that gate `rem01` went from 9.4 M evals to censored at 60 M.
- **The per-file en passant key is derived** (`rules::ep_key`) because the
  movegen's Zobrist tables are private; its test pins it against
  `hash_after_with(Legal)`.

- **MG3 cannot decide a child that is not already over, and in our tree it
  decides nothing at all.** `EffectContext::outcome_after` short-circuits only
  extinction, the commoner-only draw and the non-check clock; everything else
  clones and plays the move — the very build it promises to remove. Measured
  (plan29): **0 of 1,067,066** built children decided under `--guide hand`
  (1.9 % under `--guide none`, all extinctions), against a **ceiling of
  0.08 %** (the share of built children that are terminal at all). Do not
  re-propose effect-based child classification without a board-free
  terminality API ([`handoff2`](../movegen/handoff2.md) MG15).
- **`outcome_after` 's `None` conflates "illegal" with "open".** Its doc says
  `None` = illegal, but a legal undecided child also returns `None`. Legality
  comes from the parent's `generate_legal_with_state`/`is_legal`, never from
  `None`.
- **The checker-material clause is ours, not the crate's.** A child the
  crate calls open can be `Failed` for us (the Lichess insufficient-material
  rule), so any board-free status must screen it: a surviving **queen or
  pawn** rules out `material_draw` exactly (no insufficient clause admits
  either), while a surviving knight, rook or single bishop does **not** — the
  "o has only its commoner" clauses admit them. A screen that treats
  N/R/B as sufficient turns real material draws into `Open`, i.e. a false-win
  surface. Without an exact screen, build the child.
- **`Mover::effect` computes a key it discards for built children.** For the
  captures among them (~40 % of `needs_board` children on `deep-win`), the
  `hash_and_rule50_after` result is thrown away because the built child
  supplies the key. Unmeasured, ~1 % wall class; a reordering needs its own
  gate (plan29 §4.5).

## Allocation and performance

- std's `alloc_zeroed` memsets any layout aligned above 16 bytes (no lazy
  calloc pages). `Zeroed<T>` over-allocates with align 16 and rounds up; check
  peak RSS of a mate-in-1 after touching `alloc.rs`.
- `unsafe` sites (lazy allocator, `_mm_prefetch`, `madvise(MADV_HUGEPAGE)`)
  each carry `// SAFETY:` and a measured keep rule (D20).

## Determinism and the guide

- The search is deterministic: same binary + FEN + flags (+ `--seed`) gives
  the same counters. `--proof-out` never changes the trajectory.
- `--guide none --seed 0` must stay counter-identical to m1: the guide's fresh
  pair is the identity `(1, 1)`, `select` only tie-breaks after the DF-PN
  metric, and the equal-metric branch must still update `second` (skipping it
  changes the 1+ε thresholds and the trajectory).
- The guide may only write fresh `Open` numbers; the verified one-ply settle is
  its only exact closure. A settled `Proved` child is stored at the **child's**
  key with its own winning move and emitted, or the writer cannot resolve a
  parent proof whose child was never expanded.
- `settle_nodes` and `sim_nodes` are auxiliary; `child_evals` and `expansions`
  keep D11 meaning and stay main-search only.
- **The item-15 `GROUP` child is virtual.** It is never keyed, looked up or
  stored under its synthetic slot; its members keep the real child keys. A
  group-proved parent event must carry the selected member's move, and the
  pending-proof retention must use that member's key, or the writer fails.
  The flag is default off; item 15 is closed ([`report6`](report6.md)).
  Plan 24's GROUP-V1 (score-free members, default on) also failed —
  `deep-win` 0/3 at 120 s vs a 3/3 control — so the group is not a product
  arm; a re-proposal needs a mechanism-level claim, not another default flip
  ([`report24`](report24.md)).
- The null-side threat signature is a pure function of the board; no null
  position is ever keyed, searched, stored, put on `line` or emitted. The
  opt-in `--guide-null-exact` builds the passed board **ephemerally** for the
  one-ply exact null win and the guide side of the replay; only real child
  keys and proofs leave that module ([report18](report18.md)).
- **`null_win`'s `work.moves` is the full legal move count, not the number
  scanned.** It feeds the auxiliary budget in `Engine::null_replay`; an early
  exit that reports fewer moves changes the budget and therefore the tree.
  Plan 28's S2 therefore routes only `exposed_limit_uncached` through the
  callback stream; a future `null_win` early exit needs a separate tree-identity
  proof ([report28](report28.md)).
- An exact-replay path that proves a child by an immediate explosion must
  return the **child's own** one-move proof entry, not just the length; the
  writer walks real positions and fails `missing` otherwise (plan18 D14).

## Tree size

- **A node-entry stored-value check is redundant with the parent's child
  classification.** `Frame::enter` already looks each child up in the tables;
  a second check at `mid` entry can only fire when the parent's incoming
  thresholds are lower than the stored pair. Measured on `deep-win` at 20 M:
  53 of 668,608 expansions (0.0079 %, plan12). Do not re-propose entry reuse
  without new data.
