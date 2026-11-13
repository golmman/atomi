# Movegen Report 1 — `atomic-movegen` 2.6.0 adopted; the wall is flat

Plan: [`plan1.md`](plan1.md). Initiative: [`initiative.md`](initiative.md),
D-MG1–D-MG7. Handoff answered: [`handoff1.md`](handoff1.md) (R1–R9,
MG1–MG14). Data: [`measurements/movegen/plan1/`](../../../measurements/movegen/plan1/README.md)
(`bench.py`/`bench.json`, `tree_probe.py`/`tree_probe.json`, `pernode.json`,
`pernode_make.json`, `env.json`, README). Host: Ryzen 9 5950X (4 vCPU
container), one process at a time. Repo dirty at `atomi@9617f9b`.

## 0. Verdict — keep

**`atomic-movegen` 2.6.0, gates 1–4 green, paired `m1/m0` geomean 0.9999
(tree identical 3/3), 5 M probe 1.0152×, per-expansion 2,045 ns (m0: 2,064).
Keep: the pin is neutral on the wall and buys the API and the published
contracts; no rules divergence anywhere.** The adoption claims no rung
(D30/D40 stand): the prover's per-node line stays closed, and integrating
the new API is `prover` work.

## 1. What was done

- **Step 1 — pin and lock.** Tarball re-fetched
  (`static.crates.io/.../atomic-movegen-2.6.0.crate`), sha256
  `075cd3bf…2642336b` equal to the index checksum and to the new
  `Cargo.lock` checksum; MIT, MSRV 1.85, `pubtime` 2026-11-13T13:07:20Z, not
  yanked, no 2.5.0 in the index. Extracted in `/tmp/plan1-mg/`; the crate's
  own harness as published is green: **156 unit + 26 integration (incl.
  `perft_all_positions_depth_4`) + 9 doctests**.
- **Step 2 — frozen surface, before the bump.** `tests/movegen_golden.rs` +
  `tests/fixtures/movegen_2.4.0.tsv` (92 KB, 94 positions, 1,605 legal
  moves, 4 perft rows), generated on 2.4.0, sha256 `b8badd09…97f285`.
  Corpus: `veto.tsv` + `terminal.tsv` FEN columns, every `bench/corpus.json`
  cell, the 17 hand-built castling / en-passant / promotion FENs of
  `src/testutil.rs`, the start position. Per position: side, castling, stored
  and legal ep, `rule50`, `game_ply`, `hash`, `hash_with(Legal)`, `outcome`,
  `is_terminal`, **ordered** pseudo-legal and legal lists; per legal move:
  legal-ep child key, `rule50_after`, blast casualties, child `outcome`.
  Perft depth 4 on start / castling / en-passant / promotion-check
  (197,326 / 322,445 / 1,886 / 2,794). The generator also asserts
  `hash_after_with(m, Legal) == child.hash_with(Legal)` and
  `child.rule50() == rule50_after(m)` for every move.
- **Step 3 — bump.** `Cargo.toml` `"2.4"` → `"2.6"` + lock (`Cargo.lock`
  only, plan §6). The frozen fixture is **byte-identical and green on
  2.6.0** — the gate-2 diff is empty; `make test` (191 + the new differential = 192
  passing, 5 ignored) and `make lint` green.
- **Step 4 — downstream.** `make test-full` (slow tier: `wins_oracle`,
  `never_prove_oracle`, `wins_oracle_hand`, `false_refutation_regressions`,
  4/4) and `make veto` **8/8 accepted, 4/4 controls rejected, VETO: PASS**.
- **Step 5 — measure.** Two binaries from the **identical source tree**
  (`m0` = 2.4.0 in `/tmp/atomi-m0-plan1`, `m1` = 2.6.0 here), three cells ×
  Cap 10 s and 120 s × 3 repeats, reference (`provari --guide hand`, 10 s)
  and both sides alternating per repeat; 5 M-eval tree probe; `run_pernode`
  with both sides; `make pernode`.
- **Step 6 — this report**, the ledger/history update and two dated
  `prover/bench.md` rows.

## 2. Numbers

| | m0 2.4.0 | m1 2.6.0 | `m1/m0` |
| --- | ---: | ---: | ---: |
| `easy-win` 10 s | 1.934 s | 1.895 s | 0.980 |
| `core` 120 s | 18.372 s | 18.394 s | 1.001 |
| `easy-win` 120 s | 1.894 s | 1.930 s | 1.019 |
| paired geomean | | | **0.9999** |
| 5 M `deep-win` median `ms` | 2,571 | 2,610 | 1.0152 |
| `core` 10 M ns/expansion | 2,064 | 2,045 | 0.991 |

Tree identical on every paired cell (`child_evals` 30,690,269 / 281,668,134,
`expansions` 1,062,931 / 9,796,753) and on the 5 M probe (5,000,034 /
168,910 = the plan28 control). `deep-win` is censored at both caps on both
sides, matching the plan11/plan15 bare rows (seed 0 censors at 120 s), so
the paired ratio has no deep cell. The bench stack is the **bare** tree
(`--guide none`): the driver passes no guide and the bare tree is the
product candidate (D28), so these rows are not comparable with the
hand-stack rows (bench reading rule 4).

## 3. Release outcomes — 13 of 14 landed (D-MG7)

Verified from the 2.6.0 tarball, `CHANGELOG.md` and `src/`: MG1 `null_for`,
MG2 public `for_each_pseudo_legal` (+ `for_each_legal`), MG3
`EffectContext`/`MoveEffect`/`outcome_after`, MG4 the legality ladder
(internal), MG5 **merged into MG3**, MG6 castling-skip, **MG7 `from_fen`
performance — withdrawn** (MG1 landed instead), MG8 SAN, MG9 `Setup`, MG10
`TerminalReason`/`Terminal`, MG11 `ZOBRIST_VERSION = 1`, MG12 order contract
(+ key stability, `Send + Sync`, no-alloc, `#[non_exhaustive]` policy),
MG13 `CONFORMANCE_CASES`, MG14 the engineering contract. **None of it is
used at an atomi call site yet.**

## 4. Problems and challenges

1. **`make pernode` was red before this session** — the target passed neither
   `--evals` nor `--out`, both required since the driver grew them, so
   `make pernode` exited 2 on the committed tree (any pin, so not a
   regression of this plan). **Scope change:** the target now takes
   `PERNODE_CASES`, `PERNODE_EVALS`, `PERNODE_MEMORY`, `PERNODE_OUT`
   (defaults `core,1eb80b92f92c,db6a741c723f` / `1000000,10000000` / `192` /
   `/tmp/atomi-pernode.json`) and is green (`m1` 64.2 ms/1M, 2,054
   ns/expansion). S4 was already met by the two-sided `run_pernode` run; the
   target is the reproducible form.
2. **Gate 5, "a missing harness is a revert", and the writer.** `make
   test-full` runs `#[ignore]` tests, so a plain `#[ignore]` writer would
   **silently repair** the frozen fixture during a normal full gate —
   destroying the very evidence a veto needs. The writer therefore writes to
   `$ATOMI_WRITE_GOLDEN` only (default `/tmp/movegen_2.4.0.tsv`), and the
   committed fixture can only be rewritten deliberately. This deviates from
   plan step 2's "`#[ignore]` writer" wording in the safe direction.
3. **`measurements/planN/` collides across initiatives** —
   `prover/bench.md` already refers to `measurements/plan1/` for the
   *prover's* plan1 baseline. Movegen data lives in
   `measurements/movegen/plan1/`; the AGENTS.md wording (`measurements/
   planN/`) should say `<initiative>/planN/` for the second initiative. Owner
   to confirm.
4. **crates.io needs a `User-Agent`** (403 without), as the initiative
   already recorded; the local registry index cache also predated 2.6.0, so
   the fetch — not the network — was the whole difference.
5. **The crate's "10–12 % fewer instructions / ~4 % less wall" does not
   reach atomi.** It is visible in the crate's own depth-5 corpus; atomi's
   per-expansion cost moves −0.9 % on `core` and +2 % on `easy-win`, i.e.
   inside noise. Consistent with plan28: `Board::legal` is ~8–9 % self-time
   and the ladder decides only a subset of candidates.
6. **Bench-session spread is ±2–3 %** on the same binary (5 M probe: 2,566–
   2,624 ms). The 1.015× probe ratio and the 0.980–1.019 paired cell ratios
   are all inside it; the honest statement is **flat**, not "slightly
   slower" or "slightly faster".

## 5. Plan §9 questions, answered

- **Does MG13's corpus replace atomi's fixture or pin alongside it?**
  **Pin alongside** — keep our own fixture as the differential, and treat
  `CONFORMANCE_CASES` as an *additional* layer a later plan may read, for two
  reasons: our corpus is chosen by atomi's own rules interest (`terminal.tsv`
  clauses, the veto rows, the bench cells), and freezing our own generated
  data keeps the evidence independent of whatever a future release publishes.
  MG13's value is real but as a second opinion, not as a replacement.
- **Does the shared `board::is_attacked_by` move the per-expansion cost?**
  **No measurable effect.** `core` 10 M: 2,064 → 2,045 ns/expansion
  (−0.9 %), `easy-win` 1,804 → 1,848 (+2.4 %), `deep-win` 1,789 → 1,783.
  The `S4` trend line starts at **2,045 ns/expansion** for 2.6.0.

## 6. Missing tests / open parts

- Perft depth 5 is not in the differential (the crate's harness covers depth
  4 on 41 positions; ours depth 4 on 4). A slower `#[ignore]` perft-5 tier
  would be the next tightening if a rules-announcing release lands.
- No test asserts atomi's dependence on `ZOBRIST_VERSION` (MG11 landed); the
  frozen fixture is the *de facto* cross-version key guard, which is worth a
  comment in `rules.rs` but no new test.
- 2.6.0's own `unsafe-optimizations` feature is untouched (out of scope,
  D-MG6); it stays an opt-in the prover could measure later.
- The hand stack was measured only at fixed 5 M evals, not at wall caps. If
  a `prover` plan adopts MG3, re-run the wall bench there with
  `--atomi-stack 'hand=--guide hand'`.

## 7. Next steps (kickoff context for the first `prover` plan)

1. **MG3 first** (`effect_context()` + `outcome_after`): plan28's profile put
   per-child `outcome` at ~15–20 % of the wall and plan1's option B estimated
   ≈ 1.1–1.15×. Gate as a prover plan (same-session wall, tree identity to
   the current tree at fixed evals, `make veto`), calibrated on these
   measurements and `report28.md`.
2. **MG1 `null_for`** and **MG2 `for_each_pseudo_legal`** — already
   prototyped in plan28 at 1.219× for the whole S1+S2+S3 stack; the crate now
   *publishes* them, so re-measure the same stack against published 2.6.0
   before asking for anything new.
3. **MG10 `TerminalReason`** for diagnostics only — no wall claim.
4. `handoff2.md` stays **unspent**: nothing new to ask upstream today.

## Keep/revert line (gate 5)

**Keep** — `atomic-movegen` **2.6.0**; gates 1–4 green (crate harness
156+26+9, frozen differential **diff empty**, `make test-full` + `make veto`
8/8 accepted, 4/4 controls rejected); paired `m1/m0` geomean **0.9999** at
an identical tree (5 M probe 1.0152×, per-expansion 2,045 vs 2,064 ns);
**reason:** neutral on the wall, no rules divergence, and it is the
published version that carries the API the prover plans need.

## Commit sequence (the owner commits)

Plan §6's ordering is kept, and the two atomi-side fixes get their own
commits so commit 2 stays `Cargo.toml` + `Cargo.lock` only:

1. `test(movegen): freeze the 2.4.0 observable surface` —
   `tests/movegen_golden.rs`, `tests/fixtures/movegen_2.4.0.tsv`,
   the `PROVENANCE.md` note. Green on 2.4.0 by construction.
2. `deps: atomic-movegen 2.6.0` — `Cargo.toml`, `Cargo.lock`, nothing else.
   The frozen differential is green and unchanged.
3. `build: make pernode runnable again` — `Makefile`.
4. `docs(movegen): report1, measurements and bench rows` —
   `docs/plans/movegen/{report1,initiative}.md`, `docs/plans/README.md`,
   `docs/plans/prover/bench.md`, `measurements/movegen/plan1/`.

Squashing 1 into 2 destroys gate 2's only independent evidence.
