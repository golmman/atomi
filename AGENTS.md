# AGENTS.md

## Goal

`atomi` is a from-scratch **attacker-only atomic win prover** (Rust,
`atomic-movegen` 2.4.x, MIT): prove the side to move can force a win, with a
proof this repo's validator **and** external `provari check` accept. Target:
beat provari `--guide hand` on the shared-win bench. The goal is the top rung
`T_5` of the necessary-condition staircase in D30. T_5 (goal): all three
cells proved at 10 s, geomean wall < 1.0 vs provari --guide hand, every claim
D14-accepted. Initiative: `docs/plans/prover/` (read `initiative.md`);
predecessor: closed `winprover`.

## Collaboration

- Fixed point: the goal plus the correctness and GPL rules. Everything else
  (milestones, working agreement, decisions) is a means; challenge it.
- Push back first when a plan, decision or request conflicts with the goal or
  rests on an untested assumption.
- Ideation: include ≥ 1 option that changes the problem formulation, plus an
  estimated gain and the cheapest experiment.
- "Should we X?": yes/no first, strongest case each way, cheapest decisive
  experiment.

## Architecture

- `rules.rs` status/key/`ep_key`; `mover.rs` child key without make-move;
  `guide/` hand-guide stack (D21; only fresh `Open` numbers + tie-breaks);
  `table/` TT, filter, full-key proof table, lazy allocator.
- `search/` driver/frames/value algebra; `events.rs` + `strategy/` proof
  events/writer; `validate/` independent validator; `cli.rs`/`main.rs`;
  `tests/`, `bench/`, `scripts/`. Details: module docs and reports.

## Commands

- `make test` — licence gate + `cargo test --release` (every change, ≈ 12 s);
  `make lint` — fmt, clippy `-D warnings`, doc, goal check.
- `make test-full` — slow tier + `make veto`; required for search, table,
  proof-output and validator changes.
- `atomi prove "<FEN>" [--seconds --memory --evals --seed --guide --guide-off
  --proof-out]`, `atomi validate FILE...`, `atomi moves "<FEN>"`. `--memory`
  covers TT, proofs and filter.
- `make oracle` (pinned provari in `/tmp/atomi-oracle`); `make veto` (our
  validator + `provari check`; missing oracle = **red**); `make bench`,
  `make pernode` (`bench/README.md`).

## GPL boundary (hard rules)

- provari (`aiorla/provari`, pins in `initiative.md`; v0.2.0 only as a bench
  stretch column, D17) and shakmaty / shakmaty-syzygy are
  **GPL-3.0-or-later**. Nothing from them enters this repo: no code,
  transliterated code, copied comments or doc text (including `FORMAT.md`), no
  dependency (normal/dev/build), no binary, network, model or provari-written
  proof files.
- provari is read and built only in a scratch clone **outside** the repo: a
  source of **ideas** (in our own words) and an **external process**
  (`provari check`, `provari prove` as bench reference).
- The licence gate (`scripts/check_licenses.py`, in `make test`) fails on any
  GPL-family licence. Never weaken it; escalate.

## Copy-with-provenance rule

- Copy own-licence (MIT) `atomic_solver` material only when `initiative.md`
  marks the item **copy**.
- Every copied file/fragment carries `from atomic_solver@<40-hex sha>:<path>`
  and a `PROVENANCE.md` row (destination, source, source sha256, date,
  `verbatim` or `adapted: <what>`); identity data needs the row only.
- After the copy this repo owns it. Never sync implicitly; a re-copy is a
  deliberate change with a new row.

## Legacy reference

- `atomic_solver` (pin in `initiative.md`) is a **read-only** sibling clone
  (`../atomic_solver`): never a submodule/dependency, never edited here; its
  binary is only an external process (bench controls).
- Legacy conventions (Win/Loss/Draw search, GHI, refinement) do **not** apply.

## Correctness

- Correctness first: a faster false win is a failure. A win is *claimed* only
  after its proof passed `atomi validate` **and** `provari check`; a checker
  disagreement is a veto.
- Repetition and the 50-move limit are never wins. Identity (table,
  repetition, proofs) is `Board::hash_with(EnPassantMode::Legal)`, no clock;
  proofs carry length + limit (D4). `validate/` stays independent.
- The search is deterministic: same binary + FEN + flags (+ `--seed`) = same
  counters; `--proof-out` never changes the trajectory.
- **Binding gotchas** (Open saturation, proofs only in the full-key table,
  lazy stalemate, the guide's exact closure, the derived ep key, the
  allocator) are in [`gotchas.md`](docs/plans/prover/gotchas.md). Read it
  before touching search, table, guide or allocator code.

## Session protocol

- **Plan session:** read the initiative and prior plans/reports; read-only
  probes; write `planN.md` under `docs/plans/prover/` (one sitting, ≤ 8 KB).
- **Execution session:** implement the plan, run the gates, write `reportN.md`
  (tools, problems, open parts, missing tests, next steps; ≤ 12 KB). Scope
  changes go in the report.
- **Kill review after every 2 plans** (fixed prompt, do not implement): "What
  is the absolute distance to the goal now? If we started today, would we
  build this? Which initiative or plan would you cancel? Re-run the prior-art
  search. What is the smallest next experiment?"
- **Docs budget:** plans ≤ 8 KB, reports ≤ 12 KB, this file ≤ 6 KiB, one-line
  status entries. If docs outgrow the code, cut docs, not tests.
- **Measurements:** one process at a time, same-session ratios, reading rules
  in `bench.md`; commit drivers, parsed results, `env.json` and READMEs under
  `measurements/planN/`; never raw transcripts, provari artifacts or
  `__pycache__`.
- **Closure:** end with `SESSION COMPLETE`, deliverables and follow-up
  options, option 1 a copy-pasteable kickoff prompt.
- Items and plans are numbered independently; a plan header names its item.

## Conventions

- Rust 2024; `cargo fmt`, `cargo clippy -- -D warnings`, `cargo doc`. Avoid
  `unsafe` unless a measured win justifies it (documented, guarded).
- Source files under ~10 KB (larger: header justification). Unit tests in
  `#[cfg(test)] mod tests`, integration in `tests/`; slow tests use
  `#[ignore = "slow: ..."]`, never `cfg_attr(debug_assertions, ignore)`.
- Agents use read-only `git`; the owner commits.
