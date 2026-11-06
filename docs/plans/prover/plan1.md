# Prover Plan 1: M0 — bootstrap, correctness, minimal proof output, re-measured baseline

Initiative: [`prover`](initiative.md), backlog **item 1 (M0)**. Drafted
2026-11-06 in plan session 1. This is a **bootstrap** plan. It is the one
declared exception to "one mechanism per plan" (working agreement 2), because
nothing exists yet. It creates the package, the gates, the copied corpus and
fixtures, an attacker-only DF-PN with **no heuristics**, the minimal proof
output (JSONL writer + own validator + `provari check` veto), the adapted bench
driver with the re-measured baseline rows, and the per-node cost baseline.
**It makes no performance claim.**

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
  `check`/`Checker`), `src/dfpn.rs` (module doc), `src/walk.rs`.

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
   - the first non-empty line is the header. `root` is required, `moves` is
     echoed back, and a `provari` field routes the file to the compact reader,
     so it must be absent. The attacker is the root's side to move;
   - every other line with a `move` is `{"fen":FEN,"move":UCI}`, keyed by
     **(position, 50-move counter)**. Lines without `move` are ignored;
   - at an attacker position: a tablebase win (unused here), else the stored
     move (which must be legal), else *a capture that wins at once* (the
     defender's king explodes) settles it, else the proof fails. **A mate in
     one is not found automatically: it must be written**;
   - at a defender position: over with the attacker winning → done; the game
     over otherwise, or `halfmoves >= 100` while not over → fail; every legal
     reply is followed;
   - the outcome is tested before the clock, so a mate delivered on the
     100th ply counts as a win (this matches 2.4.0's "mate outranks the
     clock");
   - edges are recorded between positions (attacker → defender after its
     move, defender → each attacker reply), and **any cycle in the union of
     all walked lines** rejects the proof, not only a repeat along one line.
5. **UCI:** `atomic-movegen` `Move::write_uci` writes standard castling
   (`e1g1`, `e1c1`, …) and lower-case promotion letters, which shakmaty's
   `UciMove` reads in standard castling mode. The legacy exporter's 168/168
   accepted shards cover ordinary moves; castling, promotion and en passant
   need an explicit check (task D6).
6. **Rules divergence to expect:** shakmaty's atomic insufficient-material rule
   is broader than `atomic-movegen`'s commoner-only `is_material_draw` (e.g.
   bare king vs king + knight is insufficient there). A line through such a
   position could pass our validator and fail the checker. The class is
   `rules-divergence`, and it is still a veto.
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
- A `Failed` status caused by `rule50 >= 100` is a 50-move refutation with
  `from = 100` (see Search). Every other `Failed` status is unconditional
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
- `--evals N`: a deterministic cap on `child_evals`, checked per expansion.
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
  `cycle == ANY`** (with `from`); open entries are always stored.
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
- Invariant: every proof the table ever stored was emitted with the same
  values. So the sink sees, for each table proof, an equal or dominating
  entry.

### Strategy store and JSONL writer (`strategy/`, D15)

**Store** (`store.rs`, implements `ProofSink`):
`HashMap<u64, SmallSet<(Move, length, limit)>>`. Each key keeps a
**non-dominated** set: a new entry is dropped if an existing one has
`length <=` and `limit >=` it, and it removes entries it dominates (provari's
idea of "shorter or holds further"). The byte cap `--proof-memory MB` (default
1024) is accounted conservatively. When the cap is hit, the store sets `full`,
stops taking entries, and the search continues unchanged. Export then fails
with `store-full`. M4's bounded worker removes this stop.

**Writer** (`write.rs`): walks real positions from the root and needs no
search state beyond the store.

- At an attacker position at counter `c`: on the first visit of this position
  (by key), choose the shortest stored entry with `limit >= c`, and use **that
  one move for this position at every counter** (one move per position, like
  provari's `walk.rs`). The move must be legal (`Board::is_legal`); a
  revisit at a higher counter is re-walked at that counter only if
  `chosen.limit >=` the counter. A revisit at a lower or equal counter is done.
- At a defender position: the position must not be over, unless it is a win
  for the attacker. A non-terminal position at `halfmoves >= 100` fails.
  Follow every legal reply in movegen order. A reply that ends the game must
  be an attacker win.
- A grey-set (on the current walk) revisit is a cycle. With one move per
  position, a DFS with grey/black marks finds every position cycle that the
  checker would find.
- Emit one `{"fen":<fen_with(Legal) with the real counter>,"move":<UCI>}` line
  per distinct (position, counter) actually reached, in DFS order.
  **Every** attacker move is written, mates in one included. The header is
  `{"root":<root FEN>,"moves":<walk length>}`, with no `provari` field.
- **Fail closed:** missing entry → `missing`, illegal move → `missing`,
  cycle → `cycle`, counter above the chosen limit or a 50-move failure →
  `rule50-limit`, I/O error → `io`. Lines are buffered and written to
  `PATH.tmp`, then renamed. A failure writes nothing. Output is
  byte-deterministic.

Why this terminates and is acyclic in practice: each stored entry was proved
from child entries of strictly smaller length, and the non-dominated set keeps
an equal or better entry, so the chosen lengths fall along almost every line.
The one gap is a position first met at a lower counter whose shortest usable
entry is longer. The grey-set check catches it and fails with `cycle` rather
than writing an invalid file. The plan does not try to close that gap in M0:
it is rare, it is visible, and M4 owns the scaled exporter. The report counts
every `cycle`/`rule50-limit` export failure.

### Independent validator (`validate/`)

The legacy validator's rules (`src/proof_tree/validate.rs`), restated for
attacker-only strategy maps, with the JSONL semantics of finding 4
re-implemented over `atomic-movegen` only:

- parse the header: `root` must be present and `provari` absent; `moves` is
  optional, and when present it must equal the computed length. The root is
  parsed with `Board::from_fen_with(.., FenPolicy::AllowTerminal)`. The
  attacker is the root's side to move;
- load every line with `move` into `(position, counter) -> UCI`. The position
  identity is **exact**, not hashed: placement + side + castling + legal en
  passant square (e.g. the first four fields of `fen_with(Legal)`).
  Conflicting duplicates are an error; identical duplicates are allowed;
- walk exactly as the checker does. At an attacker position: a stored move
  that must resolve via `Board::move_from_uci` and be legal; otherwise a
  capture that wins at once; otherwise `missing move`. At a defender
  position: a terminal win → done; any other terminal position or
  `halfmoves >= 100` → fail; follow every legal reply. Memoize won
  (position, counter) pairs;
- record edges between exact positions and reject any cycle with an iterative
  DFS after the walk (the union of all lines, as the checker does);
- compute the length bottom-up (a capture win at once counts 1) and the number
  of won attacker (position, counter) pairs;
- report `{"file","valid","positions","length","error"}` per file;
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
  "proof_positions":…}`. `ms` is search time only (after table allocation).
  `proof` is one of `not-requested`, `not-found`, `written`, `store-full`,
  `missing`, `cycle`, `rule50-limit`, `io`.
- Exit codes: 0 for any search result; 3 when a proof was requested, a win
  was found, and no proof was written; 2 for usage/input errors.
- `validate` prints one JSON line per file and exits with 1 unless all files
  are valid (like `provari check`).
- `--version` prints the package version and the git sha baked in at build
  time (via `build.rs` reading `git rev-parse HEAD` if available, else
  `unknown`; no new dependency).

### Oracle and veto (D16)

- `scripts/build_oracle.sh [DIR]`: clones `https://github.com/aiorla/provari`
  into `${DIR:-/tmp/atomi-oracle}/provari`. It refuses any directory inside
  the repo, checks out `48686bc6c786d6ea70568f0b2ad4002cf0520206`, verifies
  `HEAD`, runs `cargo build --release` (the hand guide needs no features), and
  prints the binary path and `provari --version`. Nothing is vendored or
  committed. `.gitignore` covers likely accidents.
- `scripts/veto.py` (stdlib), run by `make veto` with
  `PROVARI=<path>` (default `/tmp/atomi-oracle/provari/target/release/provari`):
  1. if the oracle is missing: print `INCONCLUSIVE: oracle not built (run make
     oracle)` and exit non-zero (red, not skipped);
  2. for every row of `tests/fixtures/veto.tsv`: run `atomi prove` with
     `--proof-out target/veto/<name>.jsonl`, `atomi validate`, and
     `provari check` (no tablebases), then classify (working agreement 4);
  3. **negative controls** on one accepted proof: drop a line, replace a move
     with another legal move that does not win, and point a move at an illegal
     square. Both our validator and the checker must reject all three (neither
     is vacuous);
  4. **reverse control:** `provari prove "<FEN>" --guide hand --proofs
     target/veto/provari/<name>.jsonl` for the same rows. Our validator's
     verdict must equal `provari check`'s on those files. These
     provari-written proofs stay in `target/` and are never committed;
  5. print a table: row, atomi verdict class, positions, length, checker
     `valid`, reverse-control agreement. Any non-`accepted` class or
     disagreement fails.

### Fixtures (copy, data only, with provenance)

Each file starts with `# from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:<path>`
comment lines and has a row in `PROVENANCE.md`. Format: tab-separated, with a
header row.

| File | Content | Source |
| --- | --- | --- |
| `bench/corpus.json` | **verbatim** (no comment line; provenance only in `PROVENANCE.md` and `bench/README.md`); sha256 checked by tests and the driver | `docs/plans/provari/measurements/plan1/corpus.json` |
| `tests/fixtures/wins.tsv` | `name, fen, source, legacy_note`: every side-to-move `Win` row (32 + 9 + 10 = 51, dedupe by FEN and report the count) | `tests/fixtures/decisive_positions.txt`, `decisive_remaining.txt`, `move_order_positions.txt` |
| `tests/fixtures/never_prove.tsv` | `name, fen, source, reason`: every side-to-move `Loss` row (14 + 1 + 9 = 24), the legacy `Loss`/`Draw` search claims of `test_terminal_outcomes.rs`, the K-vs-K draws, and the rook-alone-vs-safe-king GHI case | the same files, `tests/test_terminal_outcomes.rs`, `tests/test_repetition.rs` |
| `tests/fixtures/terminal.tsv` | `name, fen, expected_status (won/failed/open), refutation (none/rule50/unconditional), source`: static rules claims re-checked against 2.4.x (`fifty_move_checkmate`, `fifty_move_stalemate`, touching commoners with two pieces, extinction both ways, K vs K, a touching-commoner check-immunity case) | `tests/test_position.rs`, `tests/test_terminal_outcomes.rs`, legacy `docs/plans/movegen/report_update_2.4.0.md` §Rules behaviour |
| `tests/fixtures/soundness.tsv` | `name, fen, expectation (prove/never/refute), cap, note` (see Tests) | constructed from the above; record the construction |
| `tests/fixtures/veto.tsv` | `name, fen`: the rows `make veto` must prove and get accepted | chosen from the above |

`wins.tsv` is the outcome oracle: a **refutation** of a row is a regression to
investigate. Censoring is expected in M0. `decisive_remaining.txt`'s 15
unlabelled rows are not copied. The `test_ghi.rs` transposition claim
(`4k3/PP6/8/8/8/8/8/4K3 w - - 0 1` wins) goes into `wins.tsv`. Its
black-to-move counterpart is a `Loss` and goes into `never_prove.tsv`.

### Bench driver (copy + adapt, `bench/run_bench.py`)

Copy legacy `docs/plans/winprover/measurements/bench/run_bench.py` with the
provenance line in its docstring, then adapt it:

1. **Three side kinds:** the reference `provari-<guide>` (unchanged command);
   candidate `atomi:<stack>` (`--atomi BIN`, repeatable
   `--atomi-stack NAME=FLAGS`, command `atomi prove "<FEN>" --seconds 10
   --memory 192 <flags>`, parsing the single JSON line: `found`, `length`,
   `exit`, `child_evals`, `expansions`, `ms`); legacy controls
   `atomic_solver:<stack>` (`--legacy-solver BIN`, `--legacy-stack
   NAME=FLAGS`, the legacy command and parser unchanged, **plus** the
   `reason=` fix for the `exit` field, with a comment citing legacy report5
   §10). Remove the implicit always-added `default` stack: every stack is
   named explicitly.
2. **Corpus:** the default is `bench/corpus.json`. Verify its sha256 against
   `9cd0169a…47d0e` before running, and refuse on a mismatch.
3. **Claimed-win protocol (D14):** with `--proof-check --provari-check BIN`,
   for every `atomi` cell where all timed repeats won, make one extra untimed
   run with `--proof-out` (to `--proof-dir`, default under `/tmp`). Require
   `child_evals` equal to the timed runs, `atomi validate` valid, and
   `provari check` valid. Otherwise the cell becomes censored with the verdict
   class in `note`. Record `proof_verdict` per cell.
4. **Provenance:** record `atomi@<sha>` (from `atomi --version`),
   `solver@<sha>` (`--legacy-commit`), the provari commit and
   `provari --version`, host, platform, CPU model, `rustc -V`, the
   `atomic-movegen` version from `Cargo.lock`, and the dirty-tree flag of this
   repo.
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
  censored is a result.

Plus the session reference walls/nodes/mates line, and a per-plan summary
stating the re-priced gap after 2.4.0. **Do not** compare these factors with
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
test:       python3 scripts/check_licenses.py && CARGO_PROFILE_RELEASE_LTO=thin cargo test --release
test-full:  cargo test --release -- --include-ignored && $(MAKE) veto
test-lite:  cargo test
lint:       cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo doc --no-deps
oracle:     scripts/build_oracle.sh
veto:       cargo build --release && python3 scripts/veto.py --atomi target/release/atomi --provari $(PROVARI)
bench:      python3 bench/run_bench.py … (documented defaults; needs PROVARI, LEGACY_SOLVER)
pernode:    python3 bench/run_pernode.py --atomi target/release/atomi …
```

`make test` targets < ~60 s of test time. Slow tests use
`#[ignore = "slow: ..."]` and never `cfg_attr(debug_assertions, ignore)`.

## Tests

Unit tests sit at the bottom of each module; integration tests live in `tests/`.
"Fast" tests run in `make test`; "slow" tests run in `make test-full`.

**Rules (fast):** `terminal.tsv` round-trip through `Status`. For the key:
the `e2e4 … Nf3g1` shuffle gives equal `Legal` keys and unequal `Always`
keys; a capturable en passant square keeps the keys distinct; the same
position at different counters has equal keys.

**Table (fast):** probe/store round-trip; a proof is reused at counter
`<= limit` and ignored above it; a refutation is reused at `>= from` and
ignored below it; refutations with `cycle != ANY` are never stored; replacement
keeps proofs over open entries.

**Value algebra (fast):** the length/limit rules (zeroing vs non-zeroing,
terminal win → 99, defender min), refutation `cycle`/`from` propagation, and
the frame-exit `cycle` reset, each with hand-built child sets.

**Search (fast):**

- a mate-in-1 by explosion and a mate-in-1 by checkmate: `found`, length 1;
  a mate-in-2: length 2 (fixture choice recorded; the legacy "mate_in_1"
  FEN is a 2-move win);
- `never_prove.tsv` rows that are cheap (K vs K, already-terminal roots,
  stalemate-only lines) are refuted or censored, never found;
- determinism: two runs give identical counters/exit/length. `NullSink` vs
  the store sink give identical counters (proof collection does not change the
  trajectory);
- `--evals` honoured exactly (stops at the first expansion that reaches the
  cap; `exit = "evals"`).

**Soundness (`soundness.tsv`; fast where cheap, slow otherwise):**

- rook alone vs a king with a safe area (`8/8/8/8/2k5/8/8/4KR2 w - - 0 1`):
  never found within its cap (repetition is never a win);
- a mate delivered on the 100th ply (a checkmate-shaped mate-in-1 with the
  counter set to 99): proved, with limit 99;
- the 50-move edge on a mate in two whose every winning line is quiet
  (no zeroing move by either side). Per the limit rules it holds up to counter
  97: the attacker position after the defender's reply sits at counter
  `c + 2`, which must be below 100. Rows: counter 0 and 97 → proved (limit
  97); counter 98 → not proved (refuted or censored, never found). If the
  fixture has a zeroing winning line, pick another one. The exact
  expectations are verified with the checker before they are trusted;
- a position where the only win needs a repetition-free detour: proved, and
  the written proof has no position cycle.

Each constructed row is verified by `make veto` (the checker agrees) before
it is trusted, and its construction is recorded in the fixture file header.

**Strategy store / writer (fast):** non-dominated insertion; the cap sets
`full`; the writer produces byte-identical output twice; a store with a
missing entry, a forced cycle, or an exceeded limit yields the right error and
**no file**; UCI rendering of castling, promotion (all four pieces) and en
passant round-trips through `Board::move_from_uci`.

**Validator (fast):** accepts writer output for the small wins; rejects
hand-made JSONL with a missing move, an illegal move, a move into stalemate, a
50-move-limit line, a position cycle across two lines, a wrong `moves`
header, a `provari` header field, and conflicting duplicates; accepts a proof
that omits a capture-wins-at-once position; the independence test.

**Proof round trip (fast):** for each small win, `prove --proof-out` →
`validate` → valid, with length = the search's length.

**Fixture oracle (slow):** every `wins.tsv` row at `--evals 2_000_000`:
a refutation fails the test, censored is allowed, and every proved row's proof
must validate. Every `never_prove.tsv` row at `--evals 2_000_000`: `found`
fails the test. Record the proved/censored/refuted counts in the report.

**Veto (`make veto`, part of `make test-full`):** as specified above.

## Measurement

1. Build provari (`make oracle`), the legacy binary (export at the pin, above)
   and `atomi` (release).
2. `make veto`: all rows `accepted`, controls rejected, reverse control
   agreeing.
3. Bench, one session:

   ```sh
   python3 bench/run_bench.py \
     --provari /tmp/atomi-oracle/provari/target/release/provari \
     --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
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
- A4. `scripts/build_oracle.sh`; build it once and record `provari --version`.
- Gate: `make test`, `make lint` green.

**Phase B — fixtures + rules**

- B1. Copy `bench/corpus.json` verbatim and add a sha256 test.
- B2. Build `wins.tsv`, `never_prove.tsv`, `terminal.tsv` from the legacy
  sources (a small throwaway script outside the repo, or by hand; not
  committed). Re-check every `terminal.tsv` row against 2.4.x.
- B3. `rules.rs` + rules tests.
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
- D2. `strategy/write.rs` + tests (including fail-closed and determinism).
- D3. `validate/` + tests (including independence).
- D4. `atomi validate` subcommand; proof round-trip tests.
- D5. `soundness.tsv`, `veto.tsv`; `scripts/veto.py` (classification,
  negative controls, reverse control).
- D6. Find and record a proof covering castling, promotion or en passant, if
  any `wins.tsv` row that M0 proves contains one. Otherwise record which
  shapes stay unexercised by the checker (the unit round trip still covers
  the UCI rendering).
- D7. Fixture-oracle slow tests.
- Gate: `make test`, `make test-full` (with veto), `make lint`.

**Phase E — bench + per-node**

- E1. `bench/run_bench.py` (adapted), `bench/README.md` (provenance + usage).
- E2. `bench/run_pernode.py`.
- E3. Build the legacy binary at the pin. Run the bench (Measurement 3) and
  the per-node baseline (Measurement 4).
- E4. Create `docs/plans/prover/bench.md` with the plan1 rows.
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
  validator-accepted and checker-accepted. The negative controls are rejected
  by both, and the reverse control agrees.
- No `never_prove.tsv` row and no soundness negative is ever `found`. No
  `wins.tsv` row is refuted.
- Determinism and sink-independence tests pass.
- `bench.md` exists with the three plan1 rows, the session reference line and
  the provenance pins. Any `atomi` win cell passed D14.
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
| The writer's one-move-per-position choice hits `cycle`/`rule50-limit` | Fails closed; counted in the report; M4 owns the scaled exporter |
| Path-dependent refutations thrash | Per-frame child records; only completeness is at stake. The report records the thrash signal (expansions per proved node) if visible |
| Validator and writer share a bug | The validator is independent (source-level import test) and parses the file, not the store; the checker veto; the reverse control on provari-written proofs |
| Rules divergence (insufficient material, etc.) | `rules-divergence` class; still a veto; documented with the FEN |
| Copied fixtures carry stale legacy claims | Re-checked against 2.4.x; a refuted `wins.tsv` row is investigated (legacy bug, rules change, or ours), never silently dropped |
| Oracle build fails (network/toolchain) | `INCONCLUSIVE`, recorded; the gate stays red; never substituted |
| GPL leakage via copy-paste from the scratch clone | AGENTS.md rule; ideas only; the reviewer checks new code for transliteration; FORMAT.md text not copied |
| Measurement noise on a 4-vCPU container | One process at a time; 3 repeats; medians; same-session ratios; host recorded |

## Report requirements (`report1.md`)

Follow the legacy report shape: headline, provenance table, exact commands,
results, gate table, verdict, not measured, problems and findings, missing
tests, next steps, delivered files. Specifically include:

- the fixture counts (copied / deduped / per file) and every re-check
  disagreement with 2.4.x;
- the veto table (per row: class, positions, length, checker verdict, reverse
  control) and the three negative controls;
- fixture-oracle counts (proved / censored / refuted) and export failure
  counts by class;
- the plan1 bench rows and the re-priced gap. Explicitly: **no performance
  claim** for `atomi` M0;
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
- Docs: `docs/plans/README.md`, `docs/plans/prover/bench.md`,
  `docs/plans/prover/measurements/plan1/{README.md, env.json, bench.json,
  pernode.json, pernode.csv}`, updated `initiative.md` and `AGENTS.md`,
  `docs/plans/prover/report1.md`.
