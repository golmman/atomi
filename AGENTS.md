# AGENTS.md

## Goal

`atomi` is a from-scratch **attacker-only atomic chess win prover** in Rust,
built on `atomic-movegen` 2.4.x (MIT). It proves that the side to move can
force a win and writes a proof that this repo's own validator **and** the
external `provari check` accept. Target: beat provari `--guide hand` on the
shared-win bench (geomean wall `< 1.0`, coverage ≥ provari). The current
initiative is `docs/plans/prover/` (read `initiative.md` first). Its
predecessor is the closed `winprover` initiative of `atomic_solver`.

## Collaboration

- The fixed point is the goal (Goal section) plus the correctness and GPL
  rules. Everything else here and in the initiative (milestones, working
  agreement, decisions) is a means and may be challenged.
- Push back proactively: when a plan, a decision or the owner's request
  conflicts with the goal or rests on an untested assumption, say so first.
- When asked for improvements or "out of the box" ideas, include at least one
  option that changes the problem formulation (what is computed, what counts
  as output, which constraint is dropped), with an estimated gain and the
  cheapest experiment that measures it.
- When asked "should we X?", answer yes/no first, then the strongest case for
  each side, then the cheapest decisive experiment.

## Architecture

plan1 (M0) and plan2 (M1, per-node cost) have landed; see
`docs/plans/prover/report2.md` for the latest close-out.

- `src/rules.rs` — end-of-game classification (`status`, `status_with`
  reusing the node's own movegen; `Won`/`Failed`/`Open`, the
  insufficient-material rule), the position key, `is_zeroing`, `child_key`
  (tests only), `ep_key` (the derived per-file en passant key).
- `src/mover.rs` — `Mover` (per expanded node) + `Effect` (per move): the
  child key, counter and `needs_board` flag without making the move.
- `src/table/` — `mod.rs` (`split` of the `--memory` label, the `Tables`
  lookup/store routing, the prefetch wrapper), `tt.rs` (TT v2: 64-byte
  clusters of 4 × 16-byte slots, 32-bit partial keys, Open values and
  path-independent refutations only), `filter.rs` (one-hash "possibly
  stored" bit filter), `proofs.rs` (the full-key proof table: attacker and
  defender proofs, non-dominated `(mv, length, limit)` per key, never
  evicts, refuses at 90% load), `alloc.rs` (lazy zeroed `Zeroed<T>` +
  `madvise(MADV_HUGEPAGE)`; copied from legacy, see `PROVENANCE.md`).
- `src/search/` — `mod.rs` (the `prove` driver: 256 MiB thread, MID loop,
  budgets, `Output { result, proofs, sink }`, the soundness argument),
  `frame.rs` (frame v2: node-entry status, effects / filter / classify
  passes), `value.rs` (the `Proved`/`Refuted`/`Open` algebra; Open numbers
  saturate at `OPEN_MAX`).
- `src/events.rs` — `ProofEvent`/`ProofSink`/`NullSink` (the search's
  event hook; the CLI uses `NullSink`).
- `src/strategy/` — `ProofSource` (the writer's input: the proof table, or
  the event-fed `store.rs` `ProofStore`, now a test-only differential
  source) + `write.rs` (the JSONL proof writer).
- `src/testutil.rs` — `#[cfg(test)]` seeded playout positions shared by the
  differential unit tests.
- `src/validate/` — the independent replay validator (own rules copy; only
  `std` + `serde_json` + `atomic_movegen`).
- `src/cli.rs` + `src/main.rs` — `atomi prove | validate | moves` and the
  JSON output contract.
- `tests/` — fast gate in `make test`; `#[ignore = "slow: ..."]` tiers
  (`tests/fixture_oracle.rs`); fixtures under `tests/fixtures/`.
- `bench/` — pinned corpus (`corpus.json`, sha256-verified) + the
  `run_bench.py` / `run_pernode.py` drivers (see `bench/README.md`).
- `scripts/` — `build_oracle.sh`, `check_licenses.py`, `veto.py`.

## Commands

- `make test`: licence gate + `cargo test --release` (the fast gate, every
  change; ≈ 12 s).
- `make test-full`: adds the `#[ignore]` slow tier (fixture oracle,
  regressions) and `make veto`. Required for search, table, proof-output
  and validator changes.
- The binary: `atomi prove "<FEN>" [--seconds S --memory MB --evals N
  --proof-out P]`, `atomi validate FILE...`, and
  `atomi moves "<FEN>"` (legal UCI moves in movegen order). `--memory`
  covers the TT, the proof table and the filter (a quarter to proofs);
  there is no separate proof budget (`--proof-memory` was removed in M1).
- `make lint`: `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo doc --no-deps`.
- `make oracle`: builds pinned provari in `/tmp/atomi-oracle` (outside the
  repo). `make veto`: our validator + `provari check` on the veto fixtures. A
  missing oracle makes the veto **red**, never skipped.
- `make bench` / `make pernode`: measurement drivers (see `bench/README.md`).

## GPL boundary (hard rules)

- provari (`aiorla/provari`, pin `48686bc6c786d6ea70568f0b2ad4002cf0520206`;
  v0.2.0 `224ce85dad083b1bc8fe429a9e4349de956f5305` only as a bench stretch
  column, D17) and shakmaty / shakmaty-syzygy are **GPL-3.0-or-later**. Nothing from them
  enters this repo: no code, no transliterated code, no copied comments or doc
  text (including `FORMAT.md`), no dependency of any kind (normal, dev or
  build), no binary, no network or model, and no provari-written proof files.
- provari is read and built only in a scratch clone **outside** the repo. It
  is a source of **ideas** (described in our own words) and an **external
  process** (`provari check`, `provari prove` as the bench reference).
- The licence gate (`scripts/check_licenses.py`, part of `make test`) fails on
  any GPL-family licence in the resolved dependency graph. Never weaken it to
  make a build pass; escalate instead.

## Copy-with-provenance rule

- Only own-licence (MIT) material from `atomic_solver` may be copied. Copy it
  only when `docs/plans/prover/initiative.md` marks the item **copy**. Items
  marked idea/spec are re-implemented; items marked external run as a
  process.
- Every copied file or fragment carries the line
  `from atomic_solver@<40-hex sha>:<path>` in its header (where the format
  allows comments), and gets a row in `PROVENANCE.md`: destination, source,
  source sha256 at copy time, date, `verbatim` or `adapted: <what>`.
  Verbatim data whose bytes are its identity (`bench/corpus.json`, sha256
  `9cd0169a…47d0e`) gets the `PROVENANCE.md` row only.
- After the copy this repo owns the file. Never sync it implicitly. A re-copy
  is a deliberate change with a new provenance row.

## Legacy reference

- `atomic_solver` (<https://github.com/golmman/atomic_solver>, code pin
  `5f85133ea9d751368a682e53e354bd3fc0016d00`) lives in a **read-only** sibling
  clone (`../atomic_solver`). It is never a submodule, never a crate
  dependency (path, git or published), and never edited from here.
- The legacy binary is used only as an external process (bench control rows),
  built from `git archive` of the pin into a scratch directory.
- Legacy conventions (general Win/Loss/Draw search, GHI discipline,
  refinement, the "RAM = TT only" CLI contract, its ~30 KB `AGENTS.md`) do
  **not** apply here.

## Correctness rules

- Correctness first: a faster false win is a failure. A win is *claimed* only
  after its proof passed `atomi validate` **and** `provari check`. A checker
  disagreement is a veto.
- Repetition and the 50-move limit are never wins. Position identity (table,
  repetition, proofs) is `Board::hash_with(EnPassantMode::Legal)`, with no
  clock. Proofs carry length + limit (decision D4).
- `src/validate/` must stay independent: it uses only `std`, `serde_json` and
  `atomic_movegen`, never other `crate::` modules (enforced by a test).
- Gotchas (learned in plan1):
  - Open proof/disproof numbers saturate at `OPEN_MAX = INFINITY - 1`,
    never at `INFINITY` — `INFINITY` marks decided values only; an Open
    value saturating at INFINITY caused false refutations (completeness
    bug, fixed in plan1).
  - provari v0.1.0 accepts `--guide hand` (and `model`/`none`/`network`)
    although its short `--help` omits the flag; it is documented in its
    README only.
- Gotchas (learned in plan2):
  - Proofs never live in the TT (partial keys there are sound only because
    a collision can cost a win, never invent one); every proof goes to the
    full-key proof table.
  - The filter changes no result (no false negatives); a test pins identical
    counters with it on and off.
  - Lazy stalemate (D19 + G1): a quiet non-checking child starts `(1, 1)`
    and its stalemate is found at its node entry, **unless** the child's
    side to move has only commoner + pawns (then every child is built).
    Without that gate `rem01` went from 9.4 M evals to censored at 60 M.
  - The per-file en passant key is derived (`rules::ep_key`) because the
    movegen's Zobrist tables are private; its test pins it against
    `hash_after_with(Legal)`.
  - std's `alloc_zeroed` memsets any layout aligned above 16 bytes (no lazy
    calloc pages): `Zeroed<T>` over-allocates with align 16 and rounds up.
    Check peak RSS of a mate-in-1 after touching `alloc.rs`.
- The search is deterministic: the same binary, FEN and flags give the same
  counters. Collecting a proof must not change the trajectory.

## Session protocol

- **Plan session:** read the initiative and prior plans/reports, run
  read-only probes only, and write `docs/plans/prover/planN.md`
  (self-contained, sized for one sitting).
- **Execution session:** implement `planN.md`, run the gates, and write
  `reportN.md` as the final task (tools used, problems, unresolved parts,
  missing tests, next steps). Scope changes go into the report as findings.
- Items (stable ids) and plans (sequential numbers) are independent. A plan
  header names its item.
- Measurements: one process at a time, same-session ratios, reading rules in
  `docs/plans/prover/bench.md`. Commit drivers, parsed results, `env.json` and
  READMEs under `docs/plans/prover/measurements/planN/`. Never commit raw
  transcripts, provari artifacts or `__pycache__`.
- End every session with a closure block: `SESSION COMPLETE`, deliverables,
  and follow-up options, where option 1 is a copy-pasteable kickoff prompt.

## Conventions

- Rust 2024; `cargo fmt`, `cargo clippy -- -D warnings`, `cargo doc`. Avoid
  `unsafe` unless a measured win justifies it (documented, guarded).
- Source files stay under ~10 KB; larger ones carry a header justification.
  Unit tests go in `#[cfg(test)] mod tests` at the bottom of each module;
  integration tests go in `tests/`. Slow tests use `#[ignore = "slow: ..."]`,
  never `cfg_attr(debug_assertions, ignore)`.
- Agents use read-only `git` commands. The owner commits.
