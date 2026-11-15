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
- **A length-bounded pass must not store bound-dependent refutations.** If a
  remaining-length bound is ever introduced (items 47/52), "no win within `L`"
  is **not** "no win": a proof found under a bound is a genuine proof and may
  be stored, a `Refuted` found under a bound may not go into the full-key table
  unless the key carries the moves left (analysis1 R25's mechanism). The TT's
  proof-free invariant does not save it — the proof table is where a bounded
  pass would write. `Value::Proved { length, limit }` (D4) keeps its meaning:
  `limit` is the 50-move counter, never the bound. An unbounded control run
  (`L = ∞`) must be tree-identical to today.
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
- **MG15's view movegen is not cheaper than building the child.**
  `EffectContext::terminal_after` *re-derives* the post-move view (`analyze`,
  18–36 ns) and then runs a view movegen (≈26 ns); measured on the
  `built-ish` class that is **65.8 ns per child against today's 57.5**
  (`clone`+`do_move`+`outcome()`+key), 1.147×, and 1.29× on captures
  (plan30, [`measurements/plan30`](../../../measurements/plan30/README.md)).
  `terminal_from` is **private**, so combining `post_move` with terminality
  pays `analyze` twice, and `terminal_from` is the single implementation of
  `terminal_after` — the best amortised composite *is* `terminal_after`, so
  there is no cheaper board-free terminality to buy today. Do not re-propose
  a frame swap onto `terminal_after`; a re-proposal needs a public
  `PostMove::terminal()` **and** a measured gate.
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
- **A second bit filter over proof-table keys is net zero at every size that
  keeps the tree** (plan32, item 39 closed `no effect`). A filter probe is a
  *dependent* random access (its answer selects the next probe and a branch),
  while the probe it skips is issued early and overlaps the loop body; the
  measured marginal value of a skipped probe is ≈14 ns (G1 removed 137.7 M of
  538 M probes for 1.9 s of 58.2 s) against 12–18 ns per filter probe at a
  10 %→34 % false-positive rate on 5.7 M keys. Measured same-session, all
  tree-identical: 256 KiB (the label's free slack) 0.975–1.011×, 512 KiB
  0.989–1.004×, 2 MiB from the proof table 1.001–1.016×, 4 MiB 1.026–1.039×.
  The existing filter pays only because it replaces **two** probes and fits in
  L2 while it lives. Do not re-propose a label-funded proof filter at any
  size; a re-proposal needs a mechanism-level claim
  ([`report32`](report32.md)).

## Determinism and the guide

- **A guide site that only accepts `Value::Proved` must call
  `Tables::proved`, never `lookup`** (plan32, G1). Proofs never live in the
  TT, so `lookup`'s `Proved` arm *is* `proofs.get`; at 120 s `deep-win` only
  2.8 % of lookups hit a proof and 90.6 % hit nothing, so the TT probe there
  can never answer. `proved` is exact iff the site rejects every non-`Proved`
  value — a site that also accepts `Open`/`Refuted` silently loses them. Pinned
  by `table::tests::proved_matches_lookup_proved_arm`. Corollary: a proof
  entry's `mv == 0` is a **defender** proof, and `Move::from_bits(0)` decodes
  to `Some(Move::NONE)`, not `None` — decode only non-zero bits
  (`guide/replay.rs::proof_move`), or a defender proof becomes a "winning"
  null move.
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
- **`null_win`'s pass 1 is pipelined in chunks of `K = 16`, and `K` has no
  correctness effect** (plan 33, NULL-PIPE). A chunk is keyed first (make →
  `rules::status` → `key`/`rule50` → unmake), then its proof runs are
  prefetched, then the keys are probed in move order; the least firing index
  and its witness are the ones the byte-serial scan returns, for any `K >= 1`,
  because proof reads are pure and pass 1 stores nothing
  (`null::tests::pipelined_pass1_matches_reference`, K = 1/8/16 over the
  fixtures and every seeded playout position). A change that *stores*, caches
  or reorders **decisions** inside pass 1 is not this mechanism. The D25 rule
  above still binds: `work.moves` is the full legal count set before pass 1 and
  `work.replies` belongs to pass 2. Measured 0.926× on `deep-win` @120 s, tree
  identical 9/9; the gain is the **batch**, not the hint (no-prefetch 0.939× vs
  0.940× with it at K = 8, 5 M, one session). Corollary for tests: a
  differential that compares `work.replies` must disable the null cache
  (`Tables::with_share_and_cache(.., 0)`), or the side that runs second answers
  from the cache the first one filled ([report33](report33.md)).
- An exact-replay path that proves a child by an immediate explosion must
  return the **child's own** one-move proof entry, not just the length; the
  writer walks real positions and fails `missing` otherwise (plan18 D14).
- **The explosion question has two forms, and only one may change per site.**
  `threat::immediate_explosion(_bits)` returns the **witness move** (proof
  entry, `line`, the parent's selected move): its body and pseudo-legal order
  are frozen. `threat::has_explosion` answers *existence* only and is used at
  the two existence-only sites (the `WeakThreat` class question and the null
  reply test). Three facts make the fast form exact and must survive any
  rewrite: a **commoner capture is never legal** (the capturer dies at ground
  zero), so commoners are masked out of the attackers; **`Board::legal` still
  accepts a capture when our commoner is gone** while `generate_legal` emits
  nothing, so the extinction screen is mandatory; and this movegen does **not**
  prune pinned attackers, so the blast and `Board::legal` decide, never a pin
  rule of our own. Pinned by `tests/explosion_differential.rs` (85 k positions
  + 91 k children) — a boolean disagreement there is a tree change.
- **`Board::legal` is not a legality oracle on terminal positions** (see
  above): pair it with the crate's extinction rule whenever it replaces a
  move list.

## Tree size

- **A node-entry stored-value check is redundant with the parent's child
  classification.** `Frame::enter` already looks each child up in the tables;
  a second check at `mid` entry can only fire when the parent's incoming
  thresholds are lower than the stored pair. Measured on `deep-win` at 20 M:
  53 of 668,608 expansions (0.0079 %, plan12). Do not re-propose entry reuse
  without new data.

## Evidence (binding on plans)

- **No gate compares proof length, so length is compared by convention.** D14
  checks that a proof is *sound*, not that it is *short*: two proofs of the same
  win at `length` 36 and 19 both pass. The comparison lives in
  [`bench.md`](bench.md)'s reading rules (every row records our `length` beside
  the same-session reference `mate`). Do not read "both checkers accept" as
  "the proof is as good as the reference's" — on `deep-win` it was twice as
  deep for ~20 plans and nothing failed ([`plan36`](plan36.md) §3).
- **A measurement states its blind spots.** No plan may treat the previous
  instrument's field of view as the list of levers (working agreement 11).

## Scope (not a correctness gotcha, but binding on plans)

- **No formulation change is proposable in `prover`** (D45, owner decision
  2026-11-14). The recurrent "next decision is the formulation fork" line is
  dead: provari reaches the goal in the **same** base (DF-PN with
  proof/disproof numbers, 1.42 µs/node, [`analysis2`](analysis2.md) §5 B1), so
  the gap is implementation inside a formulation the reference proves
  sufficient — tree at parity (D36) and ≈11.3 µs per expansion against
  1.42 µs (≈8×) after plans 31–32. PDS-PN/PN², TT move ordering and restarts
  would replace the base the hand-guide stack is shaped around (D31) and are
  **standing-rejected** here; a goal re-pin (D30) is not proposable while
  progress is measured. A plan may not list one in its options or next steps,
  and no item may be created for one. If implementation **plateaus** with a
  large multiple still missing, the goal question goes to a **kill review**.
  Measure against ≈8×, not D36's pre-plan31 11.5–15.5×.
  **Pending owner ruling (D47, proposed):** this bullet rests on two premises,
  and one of them has been withdrawn — D36's tree-at-parity clause is
  deprecated by [`plan36`](plan36.md) §3 (it divides by the reference's
  reported `nodes`; the two engines prove different lengths on `deep-win`).
  D47 would keep **PDS-PN/PN², TT move ordering, restarts and the goal
  re-pin** standing-rejected and leave open *whether* an in-base search-order
  change (iterative deepening on proof length, items 47/52) may be an item at
  all — that part of D47 is the owner's to decide, not a reading of this
  bullet. Until the owner rules, this bullet
  as written holds: a plan may not name a formulation change, and no item may
  be created for a *different base* — item 52 is not one.
