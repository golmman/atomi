# Prover Plan 1: M0 — bootstrap, correctness, minimal proof output, re-measured baseline

Initiative: [`prover`](initiative.md), backlog **item 1 (M0)**. Drafted
2026-11-06 in plan session 1. This is a **bootstrap** plan. It is the one
declared exception to "one mechanism per plan" (working agreement 2), because
nothing exists yet. It creates the package, the gates, the copied corpus and
fixtures, an attacker-only DF-PN with **no heuristics**, the minimal proof
output (JSONL writer + own validator + `provari check` veto), the adapted bench
driver with the re-measured baseline rows, and the per-node cost baseline.
**It makes no performance claim.**

**Revised 2026-11-06** by a decision review against the pinned sources
(§Decision review, R1–R11). The body below already reflects the revision; the
review table records what changed and why.

Prerequisite reading (all of it is summarized below; the plan is
self-contained):

- [`initiative.md`](initiative.md): decisions D1–D16, working agreement,
  backlog;
- legacy (read-only sibling clone `../atomic_solver`, `main`):
  `docs/plans/winprover/HANDOFF.md` (§2 protocol, §3.1/§3.7, §4.1–4.2, §4.7,
  §4.10, §7), `docs/plans/winprover/bench.md` (protocol, reading rules),
  `src/proof_tree/validate.rs` and `src/proof_tree/export.rs` (module docs),
  `src/proof_event.rs`;
- the pinned provari source (scratch clone, **never committed, ideas only**):
  `FORMAT.md` (proof rules), `src/check.rs` (module doc, JSONL reader
  `check`/`Checker`, `unjudged`/`unsettled`), `src/main.rs` (`write_proof`),
  `src/search.rs` (`read_proof`), `src/dfpn.rs` (module doc), `src/walk.rs`;
  `atomic-movegen` 2.4.0 `Board::outcome`/`is_material_draw` docs.

## Why this plan exists

The successor's first job is correctness, not speed (handoff R5). No later
milestone may claim a proved win before an exporter and the independent checker
veto exist. So M0 builds the smallest complete loop:

```
FEN -> attacker-only DF-PN -> proof events -> strategy store -> JSONL
    -> own replay validator -> provari check (external veto)
```

It also prices the starting point: the bench row for the post-2.4.0 legacy
controls (the legacy ledger's numbers predate `atomic-movegen` 2.4.0), and the
per-node cost of a heuristic-free baseline that M1's gate and stop rule are
measured against.

## Plan-session findings (read-only, 2026-11-06)

Verified in sources during the plan session. No code was run against the
engine, because it does not exist yet.

1. **shakmaty is `GPL-3.0-or-later`** (crate metadata, 0.30.1;
   shakmaty-syzygy 0.28.1 too). The handoff's "shakmaty as dev-dependency" is
   therefore excluded (D12). The independent rules implementation is reached
   only through the external `provari check`.
2. **`atomic-movegen` 2.4.0 `Board::hash()` already excludes the halfmove clock
   and fullmove number** (it includes pieces, side, castling, and the en
   passant file in `Always` mode). `hash_with(EnPassantMode::Legal)` /
   `hash_after_with(m, Legal)` drop the en passant file unless a legal en
   passant capture exists. `hash_after_with(.., Legal)` clones and makes the
   move only for double pushes.
3. **The checker's repetition identity is the legal-en-passant position**:
   board, side to move, castling rights, and the en passant square only when an
   en passant capture is legal, without counters (`check.rs` `Place`,
   `FORMAT.md` §Rules). Example: after `e2e4` (no en passant capture
   possible), `Ng8f6 Ng1f3 Nf6g8 Nf3g1` returns to the same position. Under
   `Always` the two positions differ; under `Legal` they are equal. So the
   search's key **must** be `Legal` (D4).
4. **JSONL reader semantics** (`check.rs` `check`/`Checker`):
   - the first non-empty line is the header. `root` is required. `moves` is
     **echoed back unchecked**, because the JSONL reader computes no length. A
     `provari` field routes the file to the compact reader, so it must be
     absent. The attacker is the root's side to move;
   - every other line with a `move` is `{"fen":FEN,"move":UCI}`, keyed by
     **(position, 50-move counter)**. Lines without `move` are ignored. A
     later line with the same key silently replaces an earlier one;
   - at an attacker position: a tablebase win (unused here), else the stored
     move (which must be legal), else *a capture after which the attacker has
     won* settles it (the defender's king exploded, **or the defender mated by
     that capture**), else the proof fails. **A non-capture mate in one is
     not found automatically: it must be written**;
   - at a defender position: over with the attacker winning → done; the game
     over otherwise, or `halfmoves >= 100` while not over → fail; every legal
     reply is followed. A reply that reaches an attacker position that is
     over, or not over at counter `>= 100`, fails;
   - "over" is shakmaty's outcome: a commoner gone, then mate or stalemate,
     then **insufficient material** (finding 6). The clock is not part of it,
     so a mate delivered on the 100th ply counts as a win (this matches
     2.4.0's "mate outranks the clock");
   - edges are recorded between positions (attacker → defender after its
     move, defender → each attacker reply), and **any cycle in the union of
     all walked lines** rejects the proof, not only a repeat along one line;
   - won attacker positions are memoized by (position, counter). Their count,
     capture wins included, is the verdict's `won_positions`;
   - **`valid: null` on small material.** Without `--tablebases`, a failure
     becomes `cannot check: needs the tablebases …` (`valid: null`) when the
     failing position, **or any position above it on its line up to the
     root**, has counter 0, no castling rights and at most 6 pieces. The
     reader cannot exclude that a table would have settled that position. So
     a broken proof of a small endgame reads `null`, not `false`. A valid
     proof never needs the tables.
5. **UCI:** `atomic-movegen` `Move::write_uci` writes standard castling
   (`e1g1`, `e1c1`, …) and lower-case promotion letters, which shakmaty's
   `UciMove` reads in standard castling mode. The legacy exporter's 168/168
   accepted shards cover ordinary moves; castling, promotion and en passant
   need an explicit check (task D6).
6. **Rules divergence: insufficient material (decided, R1).** The checker
   ends a game as a draw when **both** sides have insufficient material under
   Lichess's atomic rule. `atomic-movegen`'s `is_material_draw` covers only K
   vs K. Restated in our words, side S has insufficient material iff the
   opponent O still has its commoner and one of these holds:
   - S has only its commoner;
   - O has other pieces, the board holds only commoners and bishops, all of
     S's bishops stand on one square colour, and all of O's on the other;
   - O has only its commoner, the board has no queen and no pawn, and either
     exactly one knight, bishop or rook is on the board, or the board holds
     only commoners and at most two knights.

   The checker's precedence is: a commoner gone, then no legal move (mate or
   stalemate), then insufficient material. So K + R vs K, K + B vs K, K + N vs
   K and K + N + N vs K are draws there, and **an immediate mate in such a
   position does not count**. A search over plain `Board::outcome()` would
   prove lines through them, and the checker would reject them. Decision:
   the rules layer and the validator both apply the rule (each with its own
   copy). Calling more positions `Failed` can only lose wins, never create
   one. `rules-divergence` stays a class for any divergence still left, and
   it is still a veto.
7. **Legacy bench driver** (`docs/plans/winprover/measurements/bench/run_bench.py`,
   352 lines, stdlib only): runs provari and `atomic_solver` by path, medians
   over repeats, all-repeats-must-win, censored lower bound
   `seconds / reference_wall`, shared-cell geomean. Known parse gap (legacy
   report5 §10): it looks for `exit_reason=` but the CLI prints `reason=`.
8. **Corpus:** legacy `docs/plans/provari/measurements/plan1/corpus.json`,
   sha256 `9cd0169a…47d0e` (verified), keys `source`, `core`, `sample20`.
9. **Legacy fixture inventory** (side-to-move perspective):
   `tests/fixtures/decisive_positions.txt` 46 rows (32 Win / 14 Loss);
   `decisive_remaining.txt` 25 rows (9 Win / 1 Loss / 15 unlabelled
   `unproven_evals` rows); `move_order_positions.txt` 19 rows (10 Win /
   9 Loss); the quick suite is the 46 decisive rows plus `mNN` with
   `NN >= 23` (13 rows). Format `name;fen;expected;note`. Search claims also
   live in `tests/test_terminal_outcomes.rs` (note: its
   `mate_in_1_white_to_move` is actually a 3-ply win), `tests/test_ghi.rs`,
   `tests/test_repetition.rs` (rook alone vs a king with a safe area must not
   be a win). Static rules claims live in `tests/test_position.rs`
   (`fifty_move_checkmate_is_loss`, `fifty_move_stalemate_is_draw`,
   `touching_commoners_with_two_pieces_is_draw`).
10. **Host for this plan's measurements:** this container, AMD Ryzen 9 5950X,
    4 vCPU, Linux 7.2.13 (Fedora 44), x86-64. The legacy rows ran on aarch64.
    Factors are same-session ratios (reading rule 1), so the host change is a
    recorded fact, not a problem.
11. **provari's own JSONL writer** (`main.rs` `write_proof` → `search.rs`
    `read_proof`) chooses each attacker position's move **per (position,
    counter)**: the shortest proof that holds at that counter. It omits the
    positions a capture wins at once, and its header `moves` is the
    **search's** length, which can exceed the length of the walked proof.
    `walk.rs`'s one-move-per-board map belongs to the compact path's
    measuring, not to JSONL output.
12. **Legacy `--outcome-only` prints no `pre_exit:` line** (legacy
    `src/main.rs`, `if !outcome_only` around line 356). The bench command
    always passes it, so fixing `exit_reason=` → `reason=` alone still leaves
    `exit` null on every control run.
13. **provari v0.2.0** (`224ce85dad083b1bc8fe429a9e4349de956f5305`, tagged
    2026-10-30) exists. `check.rs` and `FORMAT.md` are byte-identical to the
    pin. `dfpn.rs`, `proofs.rs` and `table.rs` are rewritten (~2,400 changed
    lines), and it adds `--threads N` (default 1). The `prove` options the
    bench uses are unchanged.
14. **provari's `--memory MB`** covers its table *and* its proofs
    (`dfpn::split`). A `prove` that finds no win exits with 2.

## Decision review (2026-11-06)

The original plan1 decisions were checked against the pinned provari source,
`atomic-movegen` 2.4.0 and the legacy pin. These revisions are folded into the
body. Owner decision on R8: keep v0.1.0 and add a v0.2.0 column.

| # | Was | Now | Why |
| --- | --- | --- | --- |
| R1 | `Status` from `Board::outcome()` only; insufficient material left to the veto as `rules-divergence` | `Status` and the validator also apply the checker's insufficient-material draw (finding 6), after the mate test | It is a known, certain veto source, and it is sound to adopt: more `Failed` positions can only lose wins |
| R2 | Writer: one move per position at every counter, with `rule50-limit` failures accepted as a gap | Writer: per **(position, counter)**, the shortest stored entry that holds at that counter; a post-walk cycle check over position edges, as the checker does | Lengths fall strictly along every line, so the walk always finds a usable entry and never repeats a position on a line. Only cross-line cycles remain, which fail closed. This matches provari's own JSONL writer (finding 11) |
| R3 | Validator: header `moves` must equal the computed length; capture win = explosion only | `moves` is informational (reported as `header_moves`); a capture win includes a capture that mates; `positions` uses the checker's `won_positions` definition and is compared | The checker echoes `moves`, and provari writes the search length (finding 11), so equality would falsely reject the reverse control. Mate-by-capture positions are omitted by provari's writer |
| R4 | Negative controls: "both reject" on any accepted proof | The checker must return `valid: false`, not `null`. Controls run on a row with ≥ 7 pieces and mutate the root's line. Replacements are scanned with an agreement rule, and a 50-move control is added | Small endgames turn every rejection into `null` (finding 4), which would make a control unreadable |
| R5 | Round trip: written length = search length | Written length ≤ search length; `prove` reports both (`length`, `proof_length`) | The per-counter shortest choice can shorten lines |
| R6 | D14 proof run "untimed", with flags unspecified | Proof run with `--seconds 600`, other flags equal | A proof run under the 10 s deadline could be censored by the store's overhead; counters do not depend on the deadline when the search ends by proof |
| R7 | Legacy parse: fix `reason=` | Keep the fix, keep the command unchanged, and record that `exit` is null under `--outcome-only`; censoring comes from the outcome line | Finding 12 |
| R8 | One reference, provari v0.1.0 | v0.1.0 stays the veto and the primary column. A **v0.2.0 hand** stretch column is recorded in the same session (initiative D17) | The current provari is probably faster. The gap must be known before M5 decides whether to re-pin the goal |
| R9 | `make test` with `CARGO_PROFILE_RELEASE_LTO=thin` | `make test` = licence gate + `cargo test --release` (as in `AGENTS.md`). A named profile is the fallback, recorded in the report | The env override disagrees with `AGENTS.md` and rebuilds the release binary that `make veto` uses |
| R10 | Repetition fixture: rook alone vs a king with a safe area | That FEN becomes a rules fixture (K + R vs K is a checker draw, so the root is `Failed`). Repetition is exercised by a sufficient-material hold | Under R1 the old fixture no longer reaches the repetition code |
| R11 | Small gaps | Store uses a `Vec` per key (no new dependency); path-dependent refutations store nothing; value invariants stated; `--evals` overshoot defined; `build.rs` rerun triggers; already-over roots; the transposed GHI FEN named | Precision for the executor |

## Scope and non-goals

**In scope:** everything listed in the header paragraph, specified below.

**Out of scope (later items; do not start them here):** `Mover`/`Effect`,
incremental child keys without make-move, TT clusters/prefetch/filter, lazy
TT allocation (M1, which copies legacy `alloc.rs` then); move classes, EF-PN,
quiet grouping, hand evaluation (M2); static tactics, simulation (M3); bounded
strategy worker, `.pvr` (M4); restarts, portfolio, tablebases, models, a
defender-side mode. **No** crate dependency on `atomic_solver`, and **no**
provari or shakmaty code or dependency (D12).

## Design

### Repository layout after M0

```
Cargo.toml  Cargo.lock  Makefile  AGENTS.md  PROVENANCE.md  .gitignore
LICENSE  README.md
src/
  lib.rs          module wiring + crate docs
  main.rs         subcommand dispatch: prove | validate | --help | --version
  cli.rs          std-only option parsing; unknown options are errors
  rules.rs        attacker-only status over atomic_movegen; position key
  table.rs        M0 transposition table (simple, eager allocation)
  search/         attacker-only DF-PN (mod.rs loop/budgets/counters,
                  frame.rs MID + child frames, value.rs value algebra)
  events.rs       neutral proof-event type + ProofSink trait + NullSink
  strategy/       store.rs (ProofSink impl, capped), write.rs (JSONL writer)
  validate/       independent replay validator (uses no other crate module)
tests/            integration tests + fixtures/ (copied data, provenance)
bench/            corpus.json (verbatim), run_bench.py (adapted),
                  run_pernode.py, README.md
scripts/          build_oracle.sh, veto.py, check_licenses.py
docs/plans/README.md                 initiative index (one row)
docs/plans/prover/bench.md           successor ledger (created here)
docs/plans/prover/measurements/plan1/
```

Source files stay under ~10 KB (the legacy convention). A larger file carries
a one-paragraph justification in its header.

### Package, dependencies, licence gate

- `Cargo.toml`: package and binary `atomi`, edition 2024, MIT. Release profile
  `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`. Dev
  profile with `[profile.dev.package."*"] opt-level = 3` so debug tests stay
  fast.
- Dependencies: `atomic-movegen = "2.4"` (lockfile at 2.4.0) and
  `serde_json = "1"` (lockfile at a release published ≥ 7 days earlier). No
  other runtime dependency, and no dev-dependencies unless one is justified in
  the report. `Cargo.lock` is committed.
- `scripts/check_licenses.py` (stdlib): runs
  `cargo metadata --format-version 1 --locked`, walks every package in the
  resolved graph (all targets, normal + dev + build), and fails if a licence
  expression is missing or contains `GPL` (which also catches LGPL/AGPL). It
  prints the offending package and how it was reached. It runs first in
  `make test`.

### Rules layer (`rules.rs`)

The attacker is the root's side to move for the whole search, matching the
JSONL reader.

- `Status::{Won, Failed, Open}` of a position for the attacker. It is computed
  with `Board::outcome()` (one rules precedence, 2.4.0): the side to move
  losing (commoner gone, or mated, including on the clock's last ply) is
  `Won` if the side to move is the defender and `Failed` if it is the
  attacker. `Draw` (stalemate, K vs K, `rule50 >= 100` without mate) is
  `Failed`. A side-to-move `Win` (the opponent has no commoner) is `Won` if
  the opponent is the defender and `Failed` otherwise.
- **Checker-compatible material draw (R1):** when `outcome()` returns `None`
  (so the position is neither extinct nor without legal moves), the position
  is `Failed` if both sides have insufficient material by finding 6's rule
  (`rules::insufficient_material(board, color)`, a pure bitboard test). It
  never applies to a mated or extinct position, because `outcome()` decides
  those first, in the checker's order.
- A `Failed` status at `rule50 >= 100` is a 50-move refutation with
  `from = 100` (see Search). This includes a stalemate at counter 100 or
  more, which `outcome()` does not distinguish from the clock; that is
  conservative, because the entry is only reused at counter 100 or more. Every
  other `Failed` status, including the material draw, is unconditional
  (`from = 0`).
- `key(board) = board.hash_with(EnPassantMode::Legal)`,
  `child_key(board, m) = board.hash_after_with(m, EnPassantMode::Legal)`. This
  is the only position identity used by the table, the path (repetition)
  check, the events and the store. The key has no clock (D4).
- `is_zeroing(board, m)` uses `Board::is_zeroing` (pawn move or capture).
- No heuristics, no move scoring.

### Search (`search/`)

Attacker-only depth-first proof-number search (Nagai df-pn) with the 1+ε
threshold rule. All starting numbers are uniform (proof = disproof = 1 for
every open child). There is no move ordering beyond movegen order, and no
TT-independent tactics.

**Values** (`value.rs`), per node:

- `Proved { length: u16, limit: u8, mv: Move }`. `length` is in attacker moves
  (a defender node has no move). `limit` is the highest 50-move counter at
  which this proof holds, following `FORMAT.md` §Counter:
  - a terminal win (a defender position that is over) has `length = 0` and
    allows 99;
  - an attacker node proved by move `m` into defender node `D`:
    `length = 1 + D.length`, and the limit is 99 if `D` is a terminal win
    (a move that wins at once), else 99 if `m` is zeroing, else
    `D.limit - 1`;
  - a defender node `D` with all replies `r` proved:
    `length = max(child.length)` and
    `limit = min over r of (99 if r zeroing else child.limit - 1)`;
  - an OR node proved by several children takes the shortest proof (ties:
    higher limit, then first in movegen order).
- `Refuted { cycle: u16, from: u8 }`. `cycle` is the ply index on the current
  line of the shallowest repeated position the refutation relies on
  (`ANY = u16::MAX` when none). `from` is the lowest counter at which it holds
  (0 = any counter):
  - a repetition (child key equal to the key at line index `i`) gives
    `Refuted { cycle: i, from: 0 }`;
  - an OR node with all children refuted takes `cycle = min(child.cycle)` and
    `from = max(0, max over non-zeroing children of child.from - 1)`;
  - an AND node refuted by a child takes that child, preferring
    `cycle == ANY` and then the lowest `from`, and adjusts `from` the same way
    (zeroing → 0);
  - on frame exit, if `cycle >= the node's own ply index`, set `cycle = ANY`:
    the repeated position is the node itself, so the refutation holds from
    every path.
- `Open { pn, dn }`. Saturating `u32` with `INFINITY = 1 << 30`. OR: `pn = min`,
  `dn = sum`. AND: `pn = sum`, `dn = min`.
- **Invariants** (`debug_assert!`ed where the value is formed): a proof
  formed at counter `c` has `limit >= c`, and a refutation formed at counter
  `c` has `from <= c`. Hence `child.limit - 1` never underflows for a
  non-zeroing child (its counter is `c + 1 >= 1`), and the OR rule may skip
  zeroing children (their `from` is 0). A defender limit is at most 99
  automatically, because every term is.

**Soundness frame.** In attacker-only semantics a wrong *refutation* only loses
a win (completeness). A false *win* can arise only from a wrong proof. So the
proof rules are strict (only proved children, terminal wins, and table proofs
reused at a counter `<= limit`), and the refutation rules may be conservative.

**Frames** (`frame.rs`, the provari idea "a position keeps its children's
numbers while it is searched"). On entry, the node generates its legal moves
(`movegen::generate_legal`) and initializes one child record per move. Each
initialized child is `child_evals += 1`, and the entry is `expansions += 1`.
Initialization order:

1. make the move on a clone; `Status::Won` → proved terminal;
   `Status::Failed` → refuted (`from` per the rules layer);
2. the child key is on the current line at index `i` → `Refuted { cycle: i }`;
3. table probe: a proof is usable iff the child's counter is `<= limit`; a
   refutation is usable iff the child's counter is `>= from`; open → stored
   `pn`/`dn`; a miss or an unusable entry → `(1, 1)`.

Then the standard MID loop runs: while `pn < pn_threshold` and
`dn < dn_threshold`, select the best child (OR: minimum pn; AND: minimum dn;
ties to the first) and the second best. The child thresholds are
`min(own threshold, widen(second))` with `widen(x) = x + 1 + x / 8` (ε = 1/8)
on the selecting side, and `threshold - own + child` on the other side. The
search recurses and updates **only that child's record** in the frame. The
child records persist for the frame's lifetime, so path-dependent results
(which the table never stores) cannot make the loop spin.

On exit the frame computes the node value and stores it (see Table). If the
node is an attacker node that just became proved, it emits one event
(see Events).

**Termination and budgets.**

- `--seconds S`: the deadline is checked every 1024 expansions.
- `--evals N`: a deterministic cap on `child_evals`, checked after each
  expansion. The search stops at the first expansion after which
  `child_evals >= N`. The overshoot is less than one expansion's child count,
  and the reported `child_evals` is the true count.
- A root that is already over (a Strict-valid FEN that is mate, stalemate, a
  material draw, or at `rule50 >= 100`) reports `exit = "refuted"` with
  `expansions = 0`. Roots without a commoner fail Strict parsing (exit 2);
  such positions are exercised at the rules layer only.
- Root proved → `exit = "proved"`. Root refuted → `exit = "refuted"` (no win
  exists under the format's rules, as far as this search can tell; it is
  **not** a draw or loss claim). Deadline → `"timeout"`. Evals cap →
  `"evals"`. A ply cap of 4096 → `"depth"`, which censors the run (it is not
  a refutation).
- The search runs on a spawned thread with a 256 MiB stack, so recursion depth
  is never the limit below the ply cap.
- Determinism: no randomness, and no hash-map iteration anywhere in the search
  path. The same binary, FEN and flags give identical `child_evals`,
  `expansions`, exit and length.

### Table (`table.rs`, deliberately simple)

- `--memory MB` (default 192) is the table's size label. The table has
  `floor(MB × 2^20 / bucket_size)` buckets, rounded down to a power of two,
  with 4 entries per bucket. It is allocated eagerly (lazy zeroed allocation
  is M1's copy of legacy `alloc.rs`). `ms` excludes allocation (CLI section).
- Entry: full 64-bit key, `pn`, `dn`, `work` (expansions spent below the node
  during its last frame), and a payload: `Proved { mv, length, limit }`,
  `Refuted { from }`, or `Open`.
- Store rules: proofs are always stored; refutations **only when
  `cycle == ANY`** (with `from`); open entries are always stored. A
  path-dependent refutation (`cycle != ANY`) stores **nothing**: an existing
  entry for the key stays as it was.
  Replacement: same key, else an empty slot, else the lowest-`work` non-proof
  entry, else the lowest-`work` entry. Dropping a table proof is sound,
  because the output lives in the strategy store.

### Events (`events.rs`)

- `ProofEvent::AttackerProven { key: u64, mv: Move, length: u16, limit: u8 }`.
  This is the only variant in M0. It stays an enum, like the legacy
  `ProofEvent` spec, so later kinds need no signature change.
- `trait ProofSink { fn proven(&mut self, event: ProofEvent); }`. The search
  is generic over the sink and depends only on `events`, never on `strategy`
  (the legacy dependency direction). `NullSink` is a no-op used when no proof
  is requested.
- Invariant: every **attacker** proof the table ever stored was emitted with
  the same values, so the sink holds an equal or dominating entry for it.
  Defender proofs are not emitted. They are re-derived by the writer from
  their replies' attacker entries, which were emitted when those were proved.
  An attacker position is never `Won` at child initialization (a defender
  move cannot explode the defender's own commoner), so every attacker proof
  is formed in a frame and emitted there.

### Strategy store and JSONL writer (`strategy/`, D15)

**Store** (`store.rs`, implements `ProofSink`):
`HashMap<u64, Vec<(Move, length, limit)>>` (std only, no new dependency;
lookups only, never iterated, so it stays deterministic). Each key keeps a
**non-dominated** set: a new entry is dropped if an existing one has
`length <=` and `limit >=` it, and it removes entries it dominates (provari's
idea of "shorter or holds further"). The byte cap `--proof-memory MB` (default
1024) is accounted conservatively. When the cap is hit, the store sets `full`,
stops taking entries, and the search continues unchanged. Export then fails
with `store-full`. M4's bounded worker removes this stop.

**Writer** (`write.rs`, R2): walks real positions from the root and needs no
search state beyond the store. It runs on the search's big-stack thread (or
iteratively).

- At an attacker position at counter `c` (memoized by (key, counter)): choose
  the **shortest stored entry with `limit >= c`**. Ties go to a move already
  chosen for this key at another counter, then the higher limit, then the
  lower `Move::to_bits`. The move must be legal (`Board::is_legal`). Each
  (key, counter) is walked once.
- At a defender position: the position must not be over, unless it is a win
  for the attacker. A non-terminal position at `halfmoves >= 100` fails.
  Follow every legal reply in movegen order. A reply that ends the game must
  be an attacker win.
- Record position edges exactly as the checker does (attacker → defender
  position after the chosen move, defender → each non-terminal attacker
  reply; key = `rules::key`, counters ignored). After the walk, run an
  iterative grey/black DFS over them. Any cycle → `cycle`.
- Emit one `{"fen":<fen_with(Legal) with the real counter>,"move":<UCI>}` line
  per distinct (position, counter) actually reached, in DFS order.
  **Every** attacker move is written, mates in one included. The header is
  `{"root":<root FEN>,"moves":<walk length>}`, with no `provari` field. The
  walk length is computed bottom-up from the written moves (≤ the search's
  length, R5).
- **Fail closed:** store full → `store-full`; cycle → `cycle`; no usable entry
  → `missing`; illegal move → `missing`; a 50-move failure → `rule50-limit`;
  I/O error → `io`. Lines are buffered and written to `PATH.tmp`, then
  renamed. A failure writes nothing. Output is byte-deterministic.

Why it terminates, and why `missing`/`rule50-limit` indicate a bug: let the
entry `e` chosen at (X, c) have length ℓ. `e` was formed from reply entries of
length < ℓ, and the limit algebra makes each of them hold at the counter its
position has when X is at any counter `<= e.limit`, so at least at the reached
counter. The non-dominated set keeps an equal or better entry for each. So
every reached attacker (position, counter) has a usable entry, of length
< ℓ. Lengths fall strictly along every line, so the walk is finite. A position
cannot repeat on one line either: at a higher counter the usable set only
shrinks, so the chosen length could not have fallen. The residual is a
**cross-line** cycle through a position that received different moves at
different counters, which the checker rejects as well. The post-walk check
catches it, and the export fails closed. The report counts every export
failure by class and the number of positions written with more than one move.
M4 owns the scaled exporter.

### Independent validator (`validate/`)

The legacy validator's rules (`src/proof_tree/validate.rs`), restated for
attacker-only strategy maps, with the JSONL semantics of finding 4
re-implemented over `atomic-movegen` only:

- parse the header: `root` must be present and `provari` absent. `moves` is
  **informational** (R3): it is reported as `header_moves`, never required to
  match, because the checker echoes it and provari writes the search's
  length (finding 11). The root is parsed with
  `Board::from_fen_with(.., FenPolicy::AllowTerminal)`. The attacker is the
  root's side to move;
- load every line with `move` into `(position, counter) -> UCI`. Each line's
  FEN is parsed (same policy) and **re-rendered**. The position identity is
  **exact**, not hashed: placement + side + castling + legal en passant square
  (the first four fields of the re-rendered `fen_with(Legal)`), so a
  provari-written line matches even if its text differs. Conflicting
  duplicates are an error (stricter than the checker, which keeps the last);
  identical duplicates are allowed;
- end-of-game rules: `Board::outcome()`, then the material draw of finding 6
  (the validator's own copy), then the clock, as in the rules layer;
- walk exactly as the checker does. At an attacker position: a stored move
  that must resolve via `Board::move_from_uci` and be legal; otherwise a
  **capture after which the attacker has won** (the defender's commoner gone,
  or the defender mated); otherwise `missing move`. At a defender position: a
  terminal win → done; any other terminal position or `halfmoves >= 100` →
  fail; follow every legal reply. Memoize won (position, counter) pairs. The
  walk runs on a 256 MiB-stack thread or iteratively;
- record edges between exact positions and reject any cycle with an iterative
  DFS after the walk (the union of all lines, as the checker does);
- compute the length bottom-up (a capture win at once counts 1) and
  `positions` = the number of won attacker (position, counter) pairs,
  capture wins included. This is the checker's `won_positions` definition,
  so on a valid proof the two must be equal (the veto compares them);
- report `{"file","valid","positions","length","header_moves","error"}` per
  file;
- **independence:** `validate/` uses only `std`, `serde_json` and
  `atomic_movegen`, and nothing from `crate::` outside `validate`. A unit test
  reads the module's source files and fails on any other `crate::`/`super::`
  import (the provari idea of a checker that "uses nothing from the prover").
  A helper both sides need is written twice.

### CLI and output contract (`main.rs`, `cli.rs`)

```
atomi prove "<FEN>" [--seconds S] [--memory MB] [--evals N]
                    [--proof-out PATH.jsonl] [--proof-memory MB]
atomi validate PROOF.jsonl...
atomi --help | --version
```

- Root FENs are parsed with `FenPolicy::Strict`. A bad FEN or option prints
  `{"error":"..."}` and exits with 2. Unknown options are errors.
- `prove` prints exactly one JSON line on stdout:
  `{"found":true,"length":7,"exit":"proved","child_evals":…,"expansions":…,
  "ms":…,"seconds":10,"memory_mb":192,"evals_limit":null,"proof":"written",
  "proof_positions":…,"proof_length":…}`. `ms` is search time only (after
  table allocation). `length` is the search's length; `proof_length` is the
  written walk's length (`null` unless written; ≤ `length`, R5). `proof` is
  one of `not-requested`, `not-found`, `written`, `store-full`, `missing`,
  `cycle`, `rule50-limit`, `io`.
- Exit codes: 0 for any search result; 3 when a proof was requested, a win
  was found, and no proof was written; 2 for usage/input errors.
- `validate` prints one JSON line per file and exits with 1 unless all files
  are valid (like `provari check`).
- `--version` prints the package version and the git sha baked in at build
  time (via `build.rs` reading `git rev-parse HEAD` if available, else
  `unknown`; no new dependency). `build.rs` emits `cargo:rerun-if-changed`
  for `.git/HEAD` and the ref it points to, so the sha is not stale after a
  commit.

### Oracle and veto (D16)

- `scripts/build_oracle.sh [DIR]`: clones `https://github.com/aiorla/provari`
  into `${DIR:-/tmp/atomi-oracle}/provari`. It refuses any directory inside
  the repo, checks out `48686bc6c786d6ea70568f0b2ad4002cf0520206`, verifies
  `HEAD`, runs `cargo build --release` (the hand guide needs no features), and
  prints the binary path and `provari --version`. Nothing is vendored or
  committed. `.gitignore` covers likely accidents.
- **Stretch reference (R8, initiative D17):** `scripts/build_oracle.sh
  --ref v0.2.0 [DIR]` builds `224ce85dad083b1bc8fe429a9e4349de956f5305` into
  `${DIR:-/tmp/atomi-oracle}/provari-v0.2.0` the same way (`HEAD` verified
  against the full sha, not the tag). Only the bench uses it, as a
  separately labelled reference column (`--threads` left at its default of
  1). It is **never** the veto, and never mixed into the primary column.
- `scripts/veto.py` (stdlib), run by `make veto` with
  `PROVARI=<path>` (default `/tmp/atomi-oracle/provari/target/release/provari`):
  1. if the oracle is missing: print `INCONCLUSIVE: oracle not built (run make
     oracle)` and exit non-zero (red, not skipped);
  2. for every row of `tests/fixtures/veto.tsv`: run `atomi prove` with
     `--proof-out target/veto/<name>.jsonl`, `atomi validate`, and
     `provari check` (no tablebases), then classify (working agreement 4).
     `accepted` also requires our `positions` to equal the checker's
     `won_positions` (R3); a mismatch is a disagreement;
  3. **negative controls (R4)** on the accepted proof of the control row,
     named in `veto.tsv`. It must have ≥ 7 pieces at the root and no capture
     that wins at once there; prefer a root with castling rights (legacy
     `dec08`/`dec09`-like mates in two qualify). Mutate **the root's line**,
     so that the first failure is at the root, which keeps the checker's
     small-material `null` (finding 4) out of play:
     - drop the root's line;
     - replace the root's move with an illegal UCI move (a legal-looking
       move to an occupied own square);
     - replace the root's move with each other legal root move in UCI order
       until one is rejected by both. A candidate both accept (another win
       in one) is skipped and counts as agreement. A candidate where the
       verdicts differ fails the gate;
     - **50-move control:** take the accepted proof of the counter-97
       soundness row (Tests), add 1 to the counter field of every line and of
       the root, and require both to reject it (the attacker reaches counter
       100 before its mate). That row is a mate in two by a **pawnless
       attacker against a bare defending commoner**. A commoner cannot
       capture, and nothing else is there to capture, so neither side has a
       zeroing move. Every line is quiet, the shift re-keys every line
       consistently, and no counter is 0, so the checker answers `false`, not
       `null`.

     For every control, our validator must report `valid: false`, and the
     checker must report `"valid": false`. A checker `null` is
     `unchecked`, which makes the gate red ("control row too small: pick one
     with more material"); it is never read as a rejection;
  4. **reverse control:** `provari prove "<FEN>" --guide hand --proofs
     target/veto/provari/<name>.jsonl` for the same rows (exit 2 = no win
     found, which fails the gate for that row). Our validator's `valid` must
     equal `provari check`'s, and on valid files `positions` must equal
     `won_positions`. Header `moves` is not compared (R3). These
     provari-written proofs stay in `target/` and are never committed;
  5. print a table: row, atomi verdict class, positions vs `won_positions`,
     length, `header_moves`, checker `valid`, reverse-control agreement, and
     the control verdicts. Any non-`accepted` class, `unchecked` control or
     disagreement fails.

### Fixtures (copy, data only, with provenance)

Each file starts with `# from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:<path>`
comment lines and has a row in `PROVENANCE.md`. Format: tab-separated, with a
header row.

| File | Content | Source |
| --- | --- | --- |
| `bench/corpus.json` | **verbatim** (no comment line; provenance only in `PROVENANCE.md` and `bench/README.md`); sha256 checked by tests and the driver | `docs/plans/provari/measurements/plan1/corpus.json` |
| `tests/fixtures/wins.tsv` | `name, fen, source, legacy_note`: every side-to-move `Win` row (32 + 9 + 10 = 51, dedupe by FEN and report the count) | `tests/fixtures/decisive_positions.txt`, `decisive_remaining.txt`, `move_order_positions.txt` |
| `tests/fixtures/never_prove.tsv` | `name, fen, source, reason`: every side-to-move `Loss` row (14 + 1 + 9 = 24), the legacy `Loss`/`Draw` search claims of `test_terminal_outcomes.rs`, the K-vs-K draws, and the rook-alone case `8/8/8/8/2k5/8/8/4KR2 w - - 0 1` (a checker material draw under R1, so `Failed` at the root; its repetition role moves to `soundness.tsv`, R10) | the same files, `tests/test_terminal_outcomes.rs`, `tests/test_repetition.rs` |
| `tests/fixtures/terminal.tsv` | `name, fen, expected_status (won/failed/open), refutation (none/rule50/unconditional), source`: static rules claims re-checked against 2.4.x (`fifty_move_checkmate`, `fifty_move_stalemate`, touching commoners with two pieces, extinction both ways, K vs K, a touching-commoner check-immunity case), plus the **material-draw rows of R1** (constructed): `failed`/`unconditional` for K+R vs K, K+B vs K, K+N vs K, K+N+N vs K, and kings + bishops with each side's bishops on one colour and the two sides on different colours; `open` for K+Q vs K, K+P vs K, K+R+R vs K, K+B+B vs K, K+N vs K+N, and kings + bishops on the same colour; `won` for a defender mated in a K+R vs K position (mate before material) | `tests/test_position.rs`, `tests/test_terminal_outcomes.rs`, legacy `docs/plans/movegen/report_update_2.4.0.md` §Rules behaviour; finding 6 for the material rows |
| `tests/fixtures/soundness.tsv` | `name, fen, expectation (prove/never/refute), cap, note` (see Tests) | constructed from the above; record the construction |
| `tests/fixtures/veto.tsv` | `name, fen`: the rows `make veto` must prove and get accepted | chosen from the above |

`wins.tsv` is the outcome oracle: a **refutation** of a row is a regression to
investigate. Censoring is expected in M0. Under R1, a legacy `Win` whose every
win passes through a checker material draw is legitimately refuted. That
disposition (`checker-rules`) is allowed only with the evidence recorded in
the row's note: the drawn position on the legacy line, and its material. The
row stays in the file. `decisive_remaining.txt`'s 15 unlabelled rows are not
copied. The `test_ghi.rs` transposition claim
(`4k3/PP6/8/8/8/8/8/4K3 w - - 0 1` wins) goes into `wins.tsv`. Its
black-to-move counterpart `QQ2k3/8/8/8/8/8/8/4K3 b - - 0 1` is a `Loss` and
goes into `never_prove.tsv`.

### Bench driver (copy + adapt, `bench/run_bench.py`)

Copy legacy `docs/plans/winprover/measurements/bench/run_bench.py` with the
provenance line in its docstring, then adapt it:

1. **Four side kinds:** the reference `provari-<guide>` (unchanged command,
   the primary column); an optional **stretch reference**
   `provari-v0.2.0-<guide>` (`--provari-stretch BIN`, the same command, R8):
   its rows are measured in the same session, and their factors are reported
   in a separate `stretch` block, never in the primary ledger or geomean;
   candidate `atomi:<stack>` (`--atomi BIN`, repeatable
   `--atomi-stack NAME=FLAGS`, command `atomi prove "<FEN>" --seconds 10
   --memory 192 <flags>`, parsing the single JSON line: `found`, `length`,
   `exit`, `child_evals`, `expansions`, `ms`); legacy controls
   `atomic_solver:<stack>` (`--legacy-solver BIN`, `--legacy-stack
   NAME=FLAGS`, the legacy command and parser unchanged). Change the parser's
   `exit_reason=` key to `reason=` (comment citing legacy report5 §10), and
   record in the README that under `--outcome-only` the legacy binary prints
   no `pre_exit:` line, so `exit` stays null for control rows (finding 12,
   R7). Control censoring comes from the `outcome:` line, as before. Remove
   the implicit always-added `default` stack: every stack is named
   explicitly.
2. **Corpus:** the default is `bench/corpus.json`. Verify its sha256 against
   `9cd0169a…47d0e` before running, and refuse on a mismatch.
3. **Claimed-win protocol (D14, R6):** with `--proof-check --provari-check
   BIN`, for every `atomi` cell where all timed repeats won, make one extra
   untimed run with the same flags plus `--proof-out` (to `--proof-dir`,
   default under `/tmp`) and `--seconds 600` in place of `--seconds 10`, so
   the deadline cannot censor the proof run. Require `exit = "proved"`,
   `child_evals` equal to the timed runs, `proof = "written"`,
   `atomi validate` valid, `provari check` valid, and `positions` =
   `won_positions`. Otherwise the cell becomes censored with the verdict
   class in `note`. Record `proof_verdict` per cell.
4. **Provenance:** record `atomi@<sha>` (from `atomi --version`),
   `solver@<sha>` (`--legacy-commit`), the provari commit and
   `provari --version` of both references, host, platform, CPU model,
   `rustc -V`, the `atomic-movegen` version from `Cargo.lock`, and the
   dirty-tree flag of this repo.
5. Keep everything else: medians, all-repeats-must-win, censored lower bound,
   shared-cell geomean, `--dry-run`, no transcripts, and
   `sys.dont_write_bytecode`.

The legacy binary is built from an export of the pin, outside both repos:

```sh
mkdir -p /tmp/atomic_solver-5f85133
git -C ../atomic_solver archive 5f85133ea9d751368a682e53e354bd3fc0016d00 \
  | tar -x -C /tmp/atomic_solver-5f85133
(cd /tmp/atomic_solver-5f85133 && cargo build --release)
```

### Successor ledger (`docs/plans/prover/bench.md`)

Create it from the frozen legacy ledger. Copy **verbatim**, with provenance
lines: §Names and ids, §The metric, the censoring convention, §The bench set
(cells, FENs, selection rule, excluded cells), and §Reading rules. Do not edit
the reading rules' text. Add a short "Successor reading notes" paragraph after
them that maps the legacy references: "item 6 default decision" → M5;
"item-2 checker" → own validator + `provari check` (D9/D14); the candidate
command → `atomi prove "<FEN>" --seconds 10 --memory 192`. Add a protocol note
that atomi's `--memory` labels the table only in M0, and that from M1 every
in-search store must live inside the label. The goal section is the
initiative's success condition. The ledger table has the legacy columns plus a
`Pins` column (`atomi@…` · `solver@…` · `provari-v0.1.0`).

**Plan1 rows** (one session, same host, 3 cells × 3 repeats, 10 s / 192 MB,
provari hand = 1.000):

- `plan1` · `control` · legacy **default** (`solver@5f85133`, no flags);
- `plan1` · `control` · legacy **opt** (`solver@5f85133`,
  `--move-class-pn --quiet-grouping --eval-pn or`);
- `plan1` · **atomi M0** (`atomi@<sha>`, no flags). Expect censored cells;
  censored is a result;
- `plan1` · `stretch` · **provari v0.2.0 hand** (`provari@224ce85`), as a
  factor against v0.1.0 hand. It is labelled stretch reference, kept out of
  the goal geomean, and recorded only (R8, D17).

Plus the session reference walls/nodes/mates line, and a per-plan summary
stating the re-priced gap after 2.4.0, and the v0.2.0/v0.1.0 ratio as the
size of the moving target. **Do not** compare these factors with
legacy rows (reading rule 1). Nothing is written to the legacy
`winprover/bench.md`. The legacy repo is read-only for this initiative, and
its frozen note already points to the successor.

### Per-node cost baseline (D11, `bench/run_pernode.py`)

- Command: `atomi prove "<FEN>" --evals N --seconds 600 --memory 192` for
  `N ∈ {1_000_000, 10_000_000}` on the three cells, 3 repeats each, one
  process at a time.
- Record the median `ms` (search only), process wall, peak RSS
  (`resource.getrusage(RUSAGE_CHILDREN).ru_maxrss` deltas or `/proc`, stdlib
  only), `child_evals`, `expansions`, and exit. Derive
  **ms per 1 M child evals**, ns per expansion, and child evals per expansion.
- A run that ends before its cap (proved/refuted) is reported as such, with
  its own numbers, and excluded from the per-1 M median.
- **M1 reference number:** the median ms per 1 M child evals on `core` at
  `N = 10_000_000`. If that run ends early, use the largest cap that runs to
  the cap. Report all cells as context.
- Write `docs/plans/prover/measurements/plan1/pernode.json` + `.csv`. The
  legacy front-end's same-session per-node number is **M1's** measurement.
  M0 only quotes the legacy plan3 context (~0.34–0.37 s per 1 M front-end
  evals on aarch64 host `67db7a1089e2`), labelled cross-host and not
  comparable.

### Makefile

```
test:       python3 scripts/check_licenses.py && cargo test --release
test-full:  test, then cargo test --release -- --ignored && $(MAKE) veto
test-lite:  cargo test
lint:       cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo doc --no-deps
oracle:     scripts/build_oracle.sh
oracle-stretch: scripts/build_oracle.sh --ref v0.2.0
veto:       cargo build --release && python3 scripts/veto.py --atomi target/release/atomi --provari $(PROVARI)
bench:      python3 bench/run_bench.py … (documented defaults; needs PROVARI, LEGACY_SOLVER; PROVARI_STRETCH optional)
pernode:    python3 bench/run_pernode.py --atomi target/release/atomi …
```

`make test` targets < ~60 s of test time. No profile env overrides (R9): the
test binaries and `make veto`'s `target/release/atomi` share the release
profile, so neither rebuilds the other. If the target is missed because of
LTO link time, add a named profile (`[profile.gate]`, `inherits = "release"`,
`lto = "thin"`) used by `make test` only, and record it in the report. Tests
pass `--memory 16` (or build small tables directly), never the 192 MB default,
so parallel test threads do not each allocate a bench-size table. Slow tests
use `#[ignore = "slow: ..."]` and never `cfg_attr(debug_assertions, ignore)`.

## Tests

Unit tests sit at the bottom of each module; integration tests live in `tests/`.
"Fast" tests run in `make test`; "slow" tests run in `make test-full`.

**Rules (fast):** `terminal.tsv` round-trip through `Status`, including the
R1 material rows (both colours as attacker, both colours to move). The
validator's copy of the material rule runs over the same rows in its own
unit test, which reads `terminal.tsv` itself (no shared helper). For the key:
the `e2e4 … Nf3g1` shuffle gives equal `Legal` keys and unequal `Always`
keys; a capturable en passant square keeps the keys distinct; the same
position at different counters has equal keys.

**Table (fast):** probe/store round-trip; a proof is reused at counter
`<= limit` and ignored above it; a refutation is reused at `>= from` and
ignored below it; refutations with `cycle != ANY` are never stored; replacement
keeps proofs over open entries.

**Value algebra (fast):** the length/limit rules (zeroing vs non-zeroing,
terminal win → 99, defender min), refutation `cycle`/`from` propagation, and
the frame-exit `cycle` reset, each with hand-built child sets; the invariants
(`limit >= c`, `from <= c`) hold on every hand-built case.

**Search (fast):**

- a mate-in-1 by explosion and a mate-in-1 by checkmate: `found`, length 1;
  a mate-in-2: length 2 (fixture choice recorded; the legacy "mate_in_1"
  FEN is a 2-move win). Gate fixtures must have checker-sufficient material
  at the root (a K + R vs K "mate in one" is a checker draw at the root, R1);
- a mate in one inside a checker material draw (a K + R vs K position with
  an immediate rook mate) is **not** found (`exit = "refuted"`, R1);
- `never_prove.tsv` rows that are cheap (K vs K, already-terminal roots,
  stalemate-only lines, the rook-alone row) are refuted or censored, never
  found;
- determinism: two runs give identical counters/exit/length. `NullSink` vs
  the store sink give identical counters (proof collection does not change the
  trajectory);
- `--evals` honoured: the run stops at the first expansion after which
  `child_evals >= N`, with `exit = "evals"` and an overshoot smaller than
  that expansion's child count.

**Soundness (`soundness.tsv`; fast where cheap, slow otherwise):**

- **repetition hold (R10):** a position with checker-*sufficient* material
  where the defender holds only by repetition or the clock. Example shape:
  K + R + R vs a bare commoner that can keep touching the attacker's
  commoner (touching commoners cannot be checked). Never found within its
  cap. The construction and its evidence go in the file header: the legacy
  binary's `Draw` verdict for the FEN (external process, built in E3 or
  earlier) and atomi's `refuted`/censored result. If the legacy binary proves
  it a win or is censored on it, pick another;
- a mate delivered on the 100th ply (a checkmate-shaped mate-in-1 with
  sufficient material and the counter set to 99): proved, with limit 99;
- the 50-move edge on a mate in two by a **pawnless attacker against a bare
  defending commoner**, so no zeroing move exists for either side (this row
  also feeds the veto's 50-move control). Per the limit rules it holds up to
  counter 97: the attacker position after the defender's reply sits at
  counter `c + 2`, which must be below 100. Rows: counter 0 and 97 → proved
  (limit 97); counter 98 → not proved (refuted or censored, never found). The
  exact expectations are verified with the checker before they are trusted;
- a position where the only win needs a repetition-free detour: proved, and
  the written proof has no position cycle.

Each constructed row is verified by `make veto` (the checker agrees) before
it is trusted, and its construction is recorded in the fixture file header.

**Strategy store / writer (fast):** non-dominated insertion; the cap sets
`full`; the writer produces byte-identical output twice. A hand-built store
where one position is reached at two counters and its shortest entry does not
hold at the higher one: the writer writes **two different moves** for the
two (position, counter) keys, and the result validates (R2). A hand-built
store that forces a cross-line cycle gives `cycle`, a store with a missing
entry gives `missing`, and a full store gives `store-full`, each with **no
file**. UCI rendering of castling, promotion (all four pieces) and en passant
round-trips through `Board::move_from_uci`.

**Validator (fast):** accepts writer output for the small wins; rejects
hand-made JSONL with a missing move, an illegal move, a move into stalemate, a
move into a checker material draw, a 50-move-limit line, a position cycle
across two lines, a `provari` header field, and conflicting duplicates.
Accepts a proof whose header `moves` differs from the computed length, and
reports both. Accepts proofs that omit a capture-wins-at-once position, both
by explosion and by a capture that mates. Matches lines whose FEN text
differs from our rendering but parses to the same position. The independence
test.

**Proof round trip (fast):** for each small win, `prove --proof-out` →
`validate` → valid, with `length` (validator) = `proof_length` = header
`moves`, and `proof_length <=` the search's `length` (R5).

**Fixture oracle (slow):** every `wins.tsv` row at `--evals 2_000_000`:
a refutation fails the test unless the row carries a recorded `checker-rules`
disposition (§Fixtures), censored is allowed, and every proved row's proof
must validate. Every `never_prove.tsv` row at `--evals 2_000_000`: `found`
fails the test. Record the proved/censored/refuted counts in the report.

**Veto (`make veto`, part of `make test-full`):** as specified above.

## Measurement

1. Build provari (`make oracle`), the v0.2.0 stretch reference
   (`make oracle-stretch`), the legacy binary (export at the pin, above) and
   `atomi` (release).
2. `make veto`: all rows `accepted`, every control `valid: false` on both
   sides, reverse control agreeing.
3. Bench, one session:

   ```sh
   python3 bench/run_bench.py \
     --provari /tmp/atomi-oracle/provari/target/release/provari \
     --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
     --provari-stretch /tmp/atomi-oracle/provari-v0.2.0/target/release/provari \
     --provari-stretch-commit 224ce85dad083b1bc8fe429a9e4349de956f5305 \
     --legacy-solver /tmp/atomic_solver-5f85133/target/release/atomic_solver \
     --legacy-commit 5f85133ea9d751368a682e53e354bd3fc0016d00 \
     --legacy-stack 'default=' \
     --legacy-stack 'opt=--move-class-pn --quiet-grouping --eval-pn or' \
     --atomi target/release/atomi --atomi-stack 'm0=' \
     --cases core,1eb80b92f92c,db6a741c723f --repeats 3 \
     --proof-check --provari-check /tmp/atomi-oracle/provari/target/release/provari \
     --out docs/plans/prover/measurements/plan1/bench.json
   ```

4. `python3 bench/run_pernode.py --atomi target/release/atomi --cases
   core,1eb80b92f92c,db6a741c723f --evals 1000000,10000000 --repeats 3 --out
   docs/plans/prover/measurements/plan1/pernode.json`.
5. Write `measurements/plan1/README.md` (a provenance table: host, CPU, rustc,
   commits, the `atomic-movegen` version, the oracle `--version`, commands) and
   `env.json`.

## Tasks

The tasks run in phases. Each phase ends with its gate green. **Split rule:**
if the session cannot finish, stop at the end of a phase, never mid-phase.
Phases C and D are **one unit**: no search result is reported as a win before
proof output and the veto exist. Phase E alone may move to `plan2`, recorded in
`report1.md`.

**Phase A — bootstrap**

- A1. `Cargo.toml`/`Cargo.lock`, `src/lib.rs`, `src/main.rs` (`--help`,
  `--version`), `build.rs` (git sha), `.gitignore` (`target/`,
  `__pycache__/`, `logs/`, `*.log`, `*.out`, `*.err`, `*.pvr`, `*.tmp`).
- A2. `scripts/check_licenses.py`; Makefile (all targets above).
- A3. `PROVENANCE.md` (columns: destination, source
  `atomic_solver@<sha>:<path>`, source sha256 at copy, date, `verbatim` /
  `adapted: …`); `docs/plans/README.md` (one `prover` row: open, item 1).
- A4. `scripts/build_oracle.sh` (with `--ref v0.2.0`); build both once and
  record both `provari --version` outputs. Build the legacy binary from the
  pin export (§Bench driver) here, because D5 needs it as evidence for the
  repetition-hold fixture (R10).
- Gate: `make test`, `make lint` green.

**Phase B — fixtures + rules**

- B1. Copy `bench/corpus.json` verbatim and add a sha256 test.
- B2. Build `wins.tsv`, `never_prove.tsv`, `terminal.tsv` from the legacy
  sources (a small throwaway script outside the repo, or by hand; not
  committed). Re-check every `terminal.tsv` row against 2.4.x. Add the R1
  material rows.
- B3. `rules.rs` (including `insufficient_material`) + rules tests.
- Gate: `make test`, `make lint`.

**Phase C — search + CLI**

- C1. `value.rs` + value-algebra tests.
- C2. `table.rs` + table tests.
- C3. `events.rs` (`ProofEvent`, `ProofSink`, `NullSink`).
- C4. `search/` (frames, MID, budgets, counters, thread, ply cap) + search
  tests.
- C5. `cli.rs` + `prove` output contract.
- Gate: `make test`, `make lint`. No win claims yet.

**Phase D — proof output + veto**

- D1. `strategy/store.rs` + tests.
- D2. `strategy/write.rs` + tests (per-(position, counter) choice, post-walk
  cycle check, fail-closed, determinism).
- D3. `validate/` + tests (including independence and its own copy of the
  material rule).
- D4. `atomi validate` subcommand; proof round-trip tests.
- D5. `soundness.tsv`, `veto.tsv` (naming the control row); `scripts/veto.py`
  (classification, `positions`/`won_positions` agreement, the R4 controls
  including the 50-move control, reverse control).
- D6. Find and record a proof covering castling, promotion or en passant, if
  any `wins.tsv` row that M0 proves contains one. Otherwise record which
  shapes stay unexercised by the checker (the unit round trip still covers
  the UCI rendering).
- D7. Fixture-oracle slow tests.
- Gate: `make test`, `make test-full` (with veto), `make lint`.

**Phase E — bench + per-node**

- E1. `bench/run_bench.py` (adapted), `bench/README.md` (provenance + usage).
- E2. `bench/run_pernode.py`.
- E3. Run the bench (Measurement 3, both references and the legacy
  controls) and the per-node baseline (Measurement 4).
- E4. Create `docs/plans/prover/bench.md` with the plan1 rows, including the
  v0.2.0 stretch row.
- Gate: Python files parse (`ast.parse`); `make test`.

**Phase F — close-out**

- F1. Update `AGENTS.md` (architecture section, commands now real).
- F2. Update `initiative.md` (item 1 status, history, the M1 reference
  number).
- F3. Write `report1.md` (final task).

## Acceptance

- `make test`, `make test-full` (including `make veto`) and `make lint` are
  green, and `cargo build --release` is clean.
- The licence gate passes: no GPL-family licence anywhere in the resolved
  graph. There is no `atomic_solver` dependency of any kind.
- The two mates in one, the mate in two and one small win (the smallest
  `wins.tsv` row M0 proves within 10 M child evals, recorded) are each proved,
  validator-accepted and checker-accepted, with our `positions` equal to the
  checker's `won_positions`. Every negative control (R4, including the 50-move
  control) is `valid: false` on both sides, never `null`, and the reverse
  control agrees.
- No `never_prove.tsv` row and no soundness negative is ever `found`. No
  `wins.tsv` row is refuted, except rows with a recorded `checker-rules`
  disposition (R1).
- Determinism and sink-independence tests pass.
- `bench.md` exists with the three plan1 rows, the v0.2.0 stretch row, the
  session reference line and the provenance pins. Any `atomi` win cell passed
  D14 (R6 protocol).
- The per-node baseline is recorded, and the M1 reference number is stated in
  `initiative.md`.
- Every copied file has a provenance line (where the format allows it) and a
  `PROVENANCE.md` row. `bench/corpus.json` is byte-identical (sha256).
- No provari source, binary or proof, no transcripts and no `__pycache__` are
  in the tree (`git status --porcelain` and `git add --dry-run .` show only
  intended files).

## Validation

```sh
make test
make lint
make oracle            # once per host
make oracle-stretch    # once per host; bench only
make test-full         # includes make veto
cargo build --release
python3 - <<'PY'
import ast, pathlib
for p in [*pathlib.Path("bench").glob("*.py"), *pathlib.Path("scripts").glob("*.py")]:
    ast.parse(p.read_text())
PY
sha256sum bench/corpus.json   # 9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e
git status --porcelain
```

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| The uniform-numbers search proves very little within budgets | Expected; M0 makes no performance claim. Gate positions are small. Censored is a result |
| The per-(position, counter) writer produces a cross-line cycle | Post-walk cycle check, as the checker does; fails closed with `cycle`; counted in the report with the multi-move position count; M4 owns the scaled exporter |
| The checker answers `null` instead of `false` on a broken small-material proof | Controls run on a ≥ 7-piece row and fail at the root; `null` is `unchecked` and red, never read as a rejection (R4) |
| The material rule is restated wrongly (search and validator agree with each other, not with the checker) | `terminal.tsv` rows on both sides of every clause; the veto and the reverse control would show `rules-divergence`; the rule's sources are the checker's observable behaviour and the Lichess rules |
| The v0.2.0 column shows a much larger gap than v0.1.0 | Recorded as a fact (stretch, D17); the goal re-pin question goes to M5, not to plan1 |
| Path-dependent refutations thrash | Per-frame child records; only completeness is at stake. The report records the thrash signal (expansions per proved node) if visible |
| Validator and writer share a bug | The validator is independent (source-level import test) and parses the file, not the store; the checker veto; the reverse control on provari-written proofs |
| Rules divergence beyond insufficient material | `rules-divergence` class; still a veto; documented with the FEN; insufficient material itself is adopted (R1) |
| Copied fixtures carry stale legacy claims | Re-checked against 2.4.x; a refuted `wins.tsv` row is investigated (legacy bug, rules change, or ours), never silently dropped |
| Oracle build fails (network/toolchain) | `INCONCLUSIVE`, recorded; the gate stays red; never substituted |
| GPL leakage via copy-paste from the scratch clone | AGENTS.md rule; ideas only; the reviewer checks new code for transliteration; FORMAT.md text not copied |
| Measurement noise on a 4-vCPU container | One process at a time; 3 repeats; medians; same-session ratios; host recorded |

## Report requirements (`report1.md`)

Follow the legacy report shape: headline, provenance table, exact commands,
results, gate table, verdict, not measured, problems and findings, missing
tests, next steps, delivered files. Specifically include:

- the fixture counts (copied / deduped / per file) and every re-check
  disagreement with 2.4.x, and every `checker-rules` disposition with its
  evidence;
- the veto table (per row: class, positions vs `won_positions`, length,
  `header_moves`, checker verdict, reverse control) and every negative control
  with both verdicts;
- fixture-oracle counts (proved / censored / refuted) and export failure
  counts by class, with the number of positions written with more than one
  move;
- the plan1 bench rows and the re-priced gap, and the v0.2.0/v0.1.0 stretch
  ratio. Explicitly: **no performance claim** for `atomi` M0;
- the per-node table and the M1 reference number;
- a session closure block with the kickoff prompt for plan2 (M1, per-node
  cost), or for the Phase E remainder if it was split.

## Deliverables

- Package and gates: `Cargo.toml`, `Cargo.lock`, `build.rs`, `Makefile`,
  `.gitignore`, `PROVENANCE.md`, `scripts/{build_oracle.sh, veto.py,
  check_licenses.py}`.
- Engine: `src/{lib.rs, main.rs, cli.rs, rules.rs, table.rs, events.rs}`,
  `src/search/`, `src/strategy/`, `src/validate/`.
- Tests and data: `tests/*.rs`, `tests/fixtures/{wins, never_prove, terminal,
  soundness, veto}.tsv`.
- Bench: `bench/{corpus.json, run_bench.py, run_pernode.py, README.md}`.
- Oracles (scratch, never committed): `/tmp/atomi-oracle/provari` (v0.1.0)
  and `/tmp/atomi-oracle/provari-v0.2.0`.
- Docs: `docs/plans/README.md`, `docs/plans/prover/bench.md`,
  `docs/plans/prover/measurements/plan1/{README.md, env.json, bench.json,
  pernode.json, pernode.csv}`, updated `initiative.md` and `AGENTS.md`,
  `docs/plans/prover/report1.md`.
