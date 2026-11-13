# Movegen Plan 1: adopt `atomic-movegen` 2.6.0

Initiative: [`movegen`](initiative.md), **D-MG1**–**D-MG7**. Handoff answered:
[`handoff1.md`](handoff1.md) (R1–R9, MG1–MG14). Release: **2.6.0**, published
2026-11-13T13:07:20Z, **MIT**, MSRV 1.85, `.crate` sha256
`075cd3bf…2642336b` (matches the index). No 2.5.0 exists. Baseline: published
**2.4.0** (`"2.4"`, lock 2.4.0). Drafted 2026-11-13 for **S2** and **S4**.

## 1. What 2.6.0 contains (verified from the tarball)

`src/zobrist.rs` is **byte-identical** to 2.4.0 and the changelog has **no
`Fixed` section** — no rules change is announced. New files: `effects.rs`,
`setup.rs`, `conformance.rs`; `Changed` claims the faster legality ladder alters
no API, order, perft, key, FEN or `Outcome`.

**MG outcomes: 13 landed, 1 withdrawn** — **none of it used by atomi yet**
(`landed` = published and usable, not adopted at a call site):

| MG | 2.6.0 | MG | 2.6.0 |
| --- | --- | --- | --- |
| MG1 `null_for` | landed | MG8 SAN | landed |
| MG2 pseudo-legal stream | landed, + `for_each_legal` | MG9 `Setup` | landed |
| MG3 effect context | **landed**, `EffectContext`/`MoveEffect`/`outcome_after` (A + B, as recommended) | MG10 `TerminalReason` | landed |
| MG4 legality ladder | landed, internal (via MG3) | MG11 `ZOBRIST_VERSION = 1` | landed |
| MG5 node-constant data | landed, **merged into MG3** | MG12 order contract | landed |
| MG6 movegen micro-opt | landed (castling skip) | MG13 `CONFORMANCE_CASES` | landed |
| MG7 `from_fen` perf | **withdrawn** (fallback; MG1 landed) | MG14 `Send + Sync`, no-alloc, semver | landed |

## 2. Scope

The plan is the §4 step list — a pin plus a regression check. **Out of scope:**
every call-site change, because the new API is integrated by **dedicated
`prover` plans** with `report1.md` as their kickoff context (the measured
precedent, plan28's MG1+MG2 stack at **1.219×** against a **1.5×** gate, is why
no rung is claimed here and D40 stays closed); the crate's
`unsafe-optimizations` feature; and new asks (the next `handoffN.md` stays
unspent, D-MG7).

## 3. Pre-plan probe (this session; the gates re-run it formally)

Two `/tmp` builds (`/tmp/atomi-mg240`, `/tmp/atomi-mg260`), path-dependent on
the extraction — measurement only, nothing committed (D-MG4).

- **Crate harness as published:** 156 unit + 26 integration (incl.
  `perft_all_positions_depth_4`) + 9 doctests, **all pass**.
- **atomi compiles unchanged**: **no call-site adaptation** — every symbol it
  touches (generators, `Board`/`StateInfo`, `MoveList`/`Move`, `attacks_from`,
  `hash_and_rule50_after`, `FenPolicy`, `Outcome`) still exists.
- **atomi's suite: 191 tests pass on both sides** — no divergence.
- **Tree and wall**, `deep-win`, `--evals 5000000 --seed 0 --guide hand
  --memory 192`, 3 repeats, alternating, one process at a time:

| Side | `child_evals` | Expansions | ms (3 runs) | Median |
| --- | ---: | ---: | --- | ---: |
| 2.4.0 control | 5,000,034 | 168,910 | 2602 / 2585 / 2709 | 2602 |
| 2.6.0 candidate | 5,000,034 | 168,910 | 2607 / 2644 / 2601 | 2607 (**1.002×**) |

  Tree **identical** to 2.4.0 and plan28's control; wall **flat**. The crate's
  "10–12 % fewer instructions" (its corpus) never reaches atomi's wall — the
  ladder is a subset of the ~9 % `Board::legal` self-time, as plan28 predicted.
  **Adoption is neutral, not a win.**

## 4. Steps (gates 1–5 → steps 1–2, 2–3, 4, 5, 6)

| # | Step | Output |
| --- | --- | --- |
| 1 | Re-fetch, verify sha256 against the index, extract in `/tmp` | `env.json` |
| 2 | **Freeze the 2.4.0 surface before the bump**: `tests/movegen_golden.rs` + `tests/fixtures/movegen_2.4.0.tsv`, `#[ignore]` writer | fixture, own commit |
| 3 | **Bump** `"2.6"` + lock; step 2 green *unchanged* (any diff = veto); `make test`, `make lint` | pin |
| 4 | `make test-full` (slow tier + veto), `make veto` | gate 3 |
| 5 | Bench `m0` = 2.4.0 vs `m1` = 2.6.0, 10 s and 120 s, 3 repeats; `make pernode`; JSON + `env.json` in `measurements/plan1/`; **append** a dated `bench.md` row | gate 4 |
| 6 | Keep/revert line, ledger, history, **`report1.md`** (2.6.0, 13/14, challenges, verdict) — also the kickoff context for the first `prover` plan. No `handoff2.md` (D-MG7) | decision |

**Differential surface** (reused later): `fixture_fens()` +
`veto.tsv`, `terminal.tsv` + the bench cells; per position FEN, side, castling,
ep (stored + legal), `rule50`, `game_ply`, `hash()`, `hash_with(Legal)`,
`outcome()`, `is_terminal()`, the **legal and pseudo-legal order**, and per legal
move the legal-ep child key, `rule50_after`, casualties; ≈ 40 positions /
150 KB. Deep perft stays in the crate harness; MG13's `CONFORMANCE_CASES` may
replace the frozen file (decide by size at step 2).

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

## 6. Commit sequence (the owner commits)

(1) `test(movegen): freeze the 2.4.0 observable surface` — green on 2.4.0 by
construction. (2) `deps: atomic-movegen 2.6.0` — `Cargo.toml`, `Cargo.lock` and
**nothing else** (§3). Squashing them destroys gate 2's only independent
evidence.

## 7. Branch table

| Finding | Action |
| --- | --- |
| Frozen diff empty (expected), gates green | **Keep** |
| **Any** diff at step 3 (moves, order, key, clock, outcome) | **Veto, revert**; report the line |
| `make test-full` or `make veto` red | **Revert**; a checker disagreement is a veto |
| Step 5 slowdown beyond noise | Revert; flat is a keep, reported flat |
| Harness or a test binary missing | **Revert** (gate 5) |

## 8. Options, gain, cheapest experiment

- **A (this plan).** Bump, no call-site change. Gain is **API and contracts,
  not wall** (§3: ≈ 1.00×): the crate now answers "is this legal and what does
  it do" without a board write — the piece every prover-class consumer built
  privately. Cost: one fixture, one pin.
- **B (formulation change).** Spend the landed MG3 as **prover** work: per-child
  legality/outcome from node data; plan28's profile (per-child `outcome`
  ≈ 15–20 % of the wall) puts it at ≈ 1.1–1.15× — a prover plan with a prover
  gate, calibrated on `report1.md` + `measurements/plan1/`.
- **C (stay on 2.4.0).** Rejected: contradicts S2, forfeits 13 landed items,
  and the caret requirement drifts to 2.6.0 on any `cargo update` *without* a
  gate.

**Cheapest decisive experiment — already run (§3, ~10 min):** tree identical,
wall flat, suite green → one session, no rung re-look.

## 9. Acceptance and open questions

**Done** when: the frozen differential is committed and green (diff expected
empty); `make test-full` + `make veto` green; `measurements/plan1/` holds
`env.json`, bench and pernode JSONs and a README; a dated `bench.md` row
appended; `report1.md` carries the 13/14 outcome, the challenges and the
verdict; keep/revert line written; ledger and history updated.

Open at execution: does the MG13 corpus replace atomi's fixture or pin
alongside it? Does the shared `board::is_attacked_by` move atomi's
per-expansion cost (the pernode point answers S4)?
