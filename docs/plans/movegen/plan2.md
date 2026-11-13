# Movegen Plan 2: adopt `atomic-movegen` 2.7.0

Initiative: [`movegen`](initiative.md), **D-MG1**–**D-MG7**. Handoff answered:
[`handoff2.md`](handoff2.md) (MG15–MG18). Release: **2.7.0**, published
2026-11-13T16:20:30Z, **MIT**, MSRV 1.85, `.crate` sha256 `cd69dc00…d38e1ad3`
(index match). Baseline: committed **2.6.0**. Drafted 2026-11-13 (S2/S1/S4).

## 1. What 2.7.0 contains (verified from the tarball)

No behaviour change is announced (the only `Fixed` entry is the doc trap
MG16 targets); `src/zobrist.rs` is **byte-identical to 2.6.0** (`337af88d…`), so
MG11's key schema stands; `src/position.rs` is an internal refactor. **MG
outcomes: 4 of 4 landed** (`landed` = usable, not yet called; bonus: `CHILD_CASES`):

| MG | 2.7.0 surface |
| --- | --- |
| MG15 | `EffectContext::terminal_after(board, m) -> Option<Terminal>`, a `Board` wrapper, `is_terminal_after`, `has_legal_move_after`; shape (a), no build |
| MG16 | `AfterState::{Illegal, Undecided, Terminal}` via `child_outcome()`; `outcome_after` = its `.outcome()`, clone fallback gone |
| MG17 | `PostMove` (placement, casualties, ep, clock; facts only) via `post_move()` / `effect_and_post()` |
| MG18 | `MoveClassification` + `classification()` (no mover-commoner `is_attacked_by`), with `effect()` on the same derivation — a new type, not `effect_trusted` |

## 2. Scope

A **pin plus a regression check** (as [`plan1.md`](plan1.md) §2). **Out of
scope:** any call-site change (`mover.rs`, `classify.rs`, `search/frame.rs`,
`guide/`) — MG15–MG18 integration is **`prover`** work, kickstarted by
`report2.md`; `unsafe-optimizations`; new asks (D-MG7).

## 3. Pre-plan probe (this session; the gates re-run it formally)

`/tmp` builds, path-dependent on the extraction (D-MG4, measurement only). The
crate harness as published is green (163 unit + 9 integration binaries + 12
doctests); **atomi compiles unchanged**, its suite **193 passed, 5 ignored** on
2.7.0 (`frozen_surface_matches` included) — gate 2's diff is **already empty**.
Tree and wall, `deep-win`, 5 M evals, seed 0, memory 192, 3 repeats, alternating,
one process:

| Stack | m0 2.6.0 evals / exp | m1 | ms m0 | ms m1 | ratio |
| --- | --- | --- | --- | --- | ---: |
| `--guide none` | 5,000,013 / 166,379 | identical | 306/310/310 | 309/312/313 | 1.006 |
| `--guide hand` | 5,000,034 / 168,910 | identical | 2631/2631/2643 | 2655/2612/2624 | 0.997 |

Trees equal the plan1/plan29 controls, ratios inside the ±2–3 % session spread —
**flat**.

## 4. Steps (gates 1–5 → steps 1–3, 4, 5, 6)

| # | Step | Output |
| --- | --- | --- |
| 1 | Re-fetch the `.crate`, verify sha256 vs the index, extract in `/tmp`, run the crate harness | `env.json` |
| 2 | **Bump** `"2.6"` → `"2.7"` + lock; frozen differential unchanged (veto on diff); `make test`, `make lint` | pin |
| 3 | **Extend** `tests/effects_differential.rs`: `terminal_after`/`child_outcome` vs our built child's `legal`+terminal, `post_move` vs its bitboards/clocks, `classification` vs `effect`; fixture corpus, bench cells, random play-outs. Test only | differential, own commit |
| 4 | `make test-full`, `make veto`, counters unmoved at fixed evals | gate 3 |
| 5 | Bench m0 vs m1 (10 s + 120 s × 3 cells × 3 repeats), `make pernode`, JSONs + `env.json` in `measurements/movegen/plan2/`, dated `prover/bench.md` row | gate 4 / S4 |
| 6 | Keep/revert line, ledger, history, `report2.md` (4/4, challenges, prover kickoff) | decision |

Step 3 is the judgement call: atomi calls none of the new API, so the frozen
fixture cannot guard it, and extending the differential is the cheapest evidence
(`report29`). **Commits** (owner): (1) `deps: atomic-movegen 2.7.0` — `Cargo.toml`
+ `Cargo.lock` only; (2) `test(movegen): pin the 2.7.0 child surface`; (3) docs and
measurements. Squash 1 with 2 and gate 2 loses its evidence.

## 5. Gates (verbatim, D-MG6)

Every integration plan contains these, in this order (D-MG6):

1. **Pin and lock** — bump `Cargo.toml` + `Cargo.lock`; run the crate's harness
   as published (tests + perft).
2. **Golden rules differential** — freeze and diff the crate's observable
   behaviour: perft to depth 4–5, full legal move lists *in order*, `outcome`,
   `is_terminal`, `hash`/`hash_with`, `rule50`/`game_ply` after every legal
   move, on a fixed corpus. Any difference that is not an announced rules fix
   is a **veto**, not a note. atomi has no perft test today, so the first
   integration plan **adds this differential**; MG13 would replace the frozen
   data with the crate's own corpus.
3. **Downstream gates** — `make test-full` and `make veto` (an independent
   rules implementation).
4. **Measure** — same-session bench at the standard cap, a 10 s row, and
   per-expansion cost; three repeats, alternating builds, one process at a
   time; parsed results only, under `measurements/<plan>/` with `env.json`.
5. **Keep or revert with data** — one line: version, gate result, ratios,
   reason. A regression, a divergence or a missing harness is a revert.

## 6. Branch table

| Finding | Action |
| --- | --- |
| Frozen diff empty, extended differential green, gates green | **Keep** |
| Any frozen diff, a red gate, counters moved, harness missing | **Veto, revert** |
| `terminal_after`/`child_outcome` disagrees with our child on a row | **Revert**, re-file as `handoff3.md` — a lying terminality is a lost win |
| Step 5 slowdown beyond noise | Revert; flat is a keep |

## 7. Options, gain, cheapest experiment

- **A (this plan).** Pin + differential. Gain is **API, not wall** (§3): the crate
  answers "is this child terminal, and why" for every pseudo-legal move with no
  board write — the question `report29` measured as 100 % of built children.
- **B (formulation change).** Stop building children for terminality:
  `terminal_after` replaces the `clone + do_move + terminal` fallback in
  `classify.rs`/`mover.rs`. Addressable share = the probe's `needs_board` share
  ([`plan29/phase_a.json`](../../../measurements/plan29/phase_a.json)): **21.3 %**
  of 5,000,034 `deep-win` hand-stack children (29.9 % `core` 1 M) — ~10–15 % of the
  wall **if** far cheaper than a build. Cheapest experiment: a prover counter build
  at 1 M/5 M on `core` + `deep-win`, tree + wall.
- **C (stay on 2.6.0).** Rejected: contradicts S2, forfeits 4/4, and the caret
  drifts to 2.7.0 on `cargo update` ungated.

**Cheapest decisive experiment (run, §3):** trees identical, suite green incl. the
frozen differential, wall flat → one session, no rung re-look.

## 8. The handoff2 §3 gate, re-read

`handoff2.md` §3 conditions any prover wall measurement on `plan29/probe.py`
showing a decided share **> 10 %**. That gate measured the **effect** path
(0.00 %); MG15 changes the question — `terminal_after` decides **every**
pseudo-legal child, so the share is 100 % by construction and the gate is
**superseded, not passed**. Its successor is the probe's `needs_board` share
(21.3 % hand stack, §7 B) > 10 %: a hand-stack wall measurement is authorised for
the `prover` plan. No rung claimed (D30/D40 stand).

## 9. Acceptance and open questions

**Done** when: pin = `Cargo.toml` + `Cargo.lock` only; the frozen differential
byte-identical; step 3 green and trajectory-neutral; `make test-full` + `make veto`
green; `measurements/movegen/plan2/` holds `env.json`, bench, pernode and README; a
`prover/bench.md` row appended; `report2.md` carries the 4/4 outcome, challenges and
keep/revert line; ledger and history updated.

Open at execution:

1. **Material draw.** `Terminal` carries the crate's *commoner-only* draw, while
   atomi applies the broader Lichess clause (MG17's motivation) and
   `is_terminal_after` mirrors `is_terminal`. The prover integration must map
   `TerminalReason::MaterialDraw` to atomi's own clause; step 3 pins that.
2. **Pseudo-legal input.** `terminal_after` does not reject illegal moves (a
   self-explosion reports an extinct child) and ignores `game_over`; which atomi
   call sites hold legal vs pseudo-legal moves is prover work.
3. Does `PositionView` move per-expansion cost (S4)? `flat` expected; the pernode
   point answers.
