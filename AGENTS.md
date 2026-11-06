# AGENTS.md

## Goal

`atomi` is a from-scratch **attacker-only atomic chess win prover** in Rust,
built on `atomic-movegen` 2.4.x (MIT). It proves that the side to move can
force a win and writes a proof that this repo's own validator **and** the
external `provari check` accept. Target: beat provari `--guide hand` on the
shared-win bench (geomean wall `< 1.0`, coverage ≥ provari). The current
initiative is `docs/plans/prover/` (read `initiative.md` first). Its
predecessor is the closed `winprover` initiative of `atomic_solver`.

## Architecture

plan1 (M0) has landed; see `docs/plans/prover/report1.md` for the close-out.

- `src/rules.rs` — end-of-game classification (`Won`/`Failed`/`Open`, the
  insufficient-material rule), the position key, `is_zeroing`, `child_key`.
- `src/table.rs` — the df-pn transposition table (4-entry buckets,
  work-guided replacement, `usable` limits on proofs/refutations).
- `src/search/` — `mod.rs` (the `prove` driver: 256 MiB thread, MID loop,
  budgets), `frame.rs` (per-node children + thresholds), `value.rs` (the
  `Proved`/`Refuted`/`Open` algebra; Open numbers saturate at `OPEN_MAX`).
- `src/events.rs` — `ProofEvent`/`ProofSink`/`NullSink` (search's only
  proof-store hook).
- `src/strategy/` — `store.rs` (event-fed non-dominated proof store,
  `--proof-memory` cap) + `write.rs` (the JSONL proof writer).
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
  --proof-out P --proof-memory MB]`, `atomi validate FILE...`, and
  `atomi moves "<FEN>"` (legal UCI moves in movegen order).
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
