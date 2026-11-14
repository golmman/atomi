# Movegen Report 3 — `atomic-movegen` 2.8.0 adopted; the wall is flat a third time

Plan: [`plan3.md`](plan3.md). Initiative: [`initiative.md`](initiative.md),
D-MG1–D-MG7. Handoff answered: [`handoff3.md`](handoff3.md) (MG19–MG20). Data:
[`measurements/movegen/plan3/`](../../../measurements/movegen/plan3/README.md)
(`bench.json`, `tree_probe.json`, `pernode.json`, `pernode_make.json`,
`env.json`, README). Host: Ryzen 9 5950X, one process at a time. Repo dirty at
`atomi@d675701`.

## 0. Verdict — keep

**`atomic-movegen` 2.8.0, gates 1–4 green, paired `m1/m0` geomean 1.0025 (tree
identical 3/3), 5 M probe 0.9996×, per-expansion 1,999.7 ns (m0: 2,006.1 in
the same session). Keep: the pin is neutral on the wall, carries the two queries
handoff3 asked for, and no rules divergence exists anywhere.** The adoption
claims no rung (D30/D40 stand), and — unlike plan1/plan2 — **no `prover` plan
follows**: handoff3 §3's cost gate is failed, not superseded (plan3 §8).

## 1. What was done

- **Step 1 — pin and lock.** `.crate` re-fetched (232,852 bytes), sha256
  `ac68cf10…f2006b` equal to the index checksum and to the new `Cargo.lock`
  checksum; MIT, MSRV 1.85, edition 2024, `pubtime` 2026-11-13T22:45:11Z, not
  yanked. Extracted in `/tmp/plan3-mg/atomic-movegen-2.8.0/`; the crate's own
  harness as published is green: **173 unit + 11 integration binaries (51
  cases, incl. `perft_all_positions_depth_4`) + 16 doctests**, nothing ignored
  (2.7.0: 163 + 10 / 45 + 12). `src/zobrist.rs` is byte-identical to 2.7.0
  (`337af88d…`), so MG11's key schema stands; `src/` changed in `board.rs`,
  `effects.rs`, `lib.rs`, `movegen.rs`, `position.rs`.
- **Step 2 — bump.** `Cargo.toml` `"2.7"` → `"2.8"` + `Cargo.lock` (nothing
  else). The frozen surface `tests/fixtures/movegen_2.4.0.tsv` is **unchanged
  and green** — the gate-2 diff is empty: the writer, run into `/tmp` on
  2.8.0, regenerates that same sha256 `b8badd09…`, so no move order, perft
  value, key, clock or outcome moved. `make test` 196 passed / 5 ignored,
  `make lint` green. No call site changed; atomi compiles unmodified.
- **Step 3 — the differentials** (test only, §4.1 for the file layout).
  `tests/child_surface_differential.rs` gains 2.8.0: per pseudo-legal move,
  `PostMove::terminal()` equals the built child's `terminal()` (reason
  included) and equals `terminal_after`; `post_move_and_key` returns the same
  summary and a key equal to `child.hash()` and `Board::hash_after(m)`, equal
  to the child's `Legal`-mode key exactly when the child stores no capturable
  ep target and differing by `rules::ep_key(file)` otherwise. Corpus
  unchanged: 94 positions + 2,620 play-out nodes, **51,334 children** (48,562
  legal, 2,772 illegal, 375 of those decided); the key split is exercised on
  **2,866 stored-ep / 48,468 plain** children. New
  `tests/legality_screen_differential.rs` pins the MG20 screen **against the
  generator, never against itself**: `has_legal_move` and
  `has_legal_move_with_state` equal `!generate_legal(..).is_empty()` on 2,859
  nodes, `has_legal_move_after` equals `!generate_legal(built child).is_empty()`
  on **51,633** children (illegal candidates included), and the lazy-stalemate
  premise holds on 24 legal-moveless parents and 679 such children (no legal
  move ⇒ `is_terminal()`). Arms reached: screen-eligible 1,987, in check 109,
  extinct 12, terminal-with-moves 191. **Sensitivity:** three mutations in a
  `/tmp` copy (drop the ep split, `assert_ne!` on the terminal reason, invert
  the screen) each failed the expected row; the restored files are green.
- **Step 4 — downstream.** `make test-full`: slow tier 4/4 (`wins_oracle`,
  `wins_oracle_hand`, `never_prove_oracle`, `false_refutation_regressions`);
  `make veto`: **8/8 accepted, 4/4 controls rejected, `provari check` agreeing
  everywhere**. Counters unmoved: the 5 M `deep-win` probe is the plan28
  control tree (5,000,034 child evals / 168,910 expansions) on **both** sides.
- **Step 5 — measure.** Both sides from the identical source tree (`m0` =
  2.7.0 in `/tmp/atomi-m0-plan3`, `m1` = 2.8.0 here; `src/` and `build.rs`
  verified byte-identical), three cells × Cap 10 s and 120 s × 3 repeats,
  `provari --guide hand` (10 s) per repeat, sides alternating; 5 M tree probe;
  `run_pernode` both sides; `make pernode`.
- **Step 6 — this report** plus the ledger/history/README update and two dated
  `prover/bench.md` rows.

## 2. Numbers

| | m0 2.7.0 | m1 2.8.0 | `m1/m0` |
| --- | ---: | ---: | ---: |
| `easy-win` 10 s | 1.878 s | 1.875 s | 0.9984 |
| `core` 120 s | 18.246 s | 18.364 s | 1.0065 |
| `easy-win` 120 s | 1.887 s | 1.892 s | 1.0026 |
| paired geomean | | | **1.0025** |
| 5 M `deep-win` median `ms` | 2,633 | 2,632 | 0.9996 |
| `core` 10 M ns/expansion | 2,006.1 | 1,980.5 | 0.987 |

Tree identical on every paired cell (`child_evals` 30,690,269 /
281,668,134, `expansions` 1,062,931 / 9,796,753) and on the 5 M probe.
Coverage did not move: 1/3 at Cap 10 s (`easy-win`, factor 13.587; `m0`
13.609), 2/3 at Cap 120 s (geomean 22.286; `m0` 22.185), `deep-win` censored
at both caps. The bench stack is the **bare** tree (`--guide none`, D28), so
these rows are not comparable with the hand-stack rows (reading rule 4).

## 3. Release outcomes — 2 of 2 landed as API, 0 of 2 met the cost bar (D-MG7)

Verified from the 2.8.0 tarball and `CHANGELOG.md`:

- **MG19 landed** — `PostMove::terminal()` (the child's `Terminal`, reason
  included, same precedence as `terminal_after`), `PostMove::rule50()` and
  `EffectContext::post_move_and_key() -> (PostMove, u64)`; *it is* the
  implementation behind `terminal_after`/`child_outcome`, so a consumer that
  keeps the summary pays for the child derivation once. Cost: **missed** —
  with the key the composite is 1.24× `clone`+`do_move`+`outcome()`+key on the
  crate's own `built-ish` corpus (bar: ≤ 0.84×), without the key 0.92×.
- **MG20 landed** — a positive screen (`quiet_legal_witness`) in front of the
  exact ladder, serving `has_legal_move`, `has_legal_move_with_state` and
  `has_legal_move_after`, with a documented equivalence contract to
  `generate_legal_with_state`. Cost: **missed** — 36 → 25–30 ns per child
  against handoff3's ≤ 12 ns bar (3.5× over on our corpus in plan3's probe).
- **No `Fixed` entry** in the 2.8.0 section. `zobrist.rs` unchanged, so no
  stored key is invalidated.

## 4. Problems and challenges

1. **Scope change: the differential's file layout, again.** Extending
   `child_surface_differential.rs` inline with the screen would have produced
   one ~28 KB file. The plan says "extend `tests/child_surface_differential.rs`";
   the child surface is extended there (now 19.7 KB under its existing
   "justification for >10 KB" header) and the screen — a different contract
   (existence vs the generator) — got its own
   `tests/legality_screen_differential.rs` (8.4 KB), both reading one corpus
   parser. Evidence as planned, layout not.
2. **The screen cannot be observed directly.** `quiet_legal_witness` is
   crate-private, so the test compares the *answer*, not the path. The
   "screen-eligible" counter is a re-derivation from public queries
   (`checkers()`, `pinned()`, the pseudo-legal stream filtered to quiet
   non-commoner, non-castling, unpinned moves) and is a **superset** of the
   crate's predicate: it proves the hot arm was reachable, not how often the
   screen hit.
3. **A key detail the plan did not name.** `post_move_and_key`'s key is
   `EnPassantMode::Always` while atomi's identity is `Legal` plus the derived
   `rules::ep_key(file)` (D4/MG11). The differential pins *both* directions:
   the two keys agree on 48,468 children and differ by exactly the file key on
   the 2,866 that store an uncapturable ep target — the check that would break
   first if the crate's `child_key` dropped the `ep[8]` rule.
4. **Private/typed API papercuts in the test.** `Bitboard::contains` is
   `pub(crate)`, so the test builds `(pinned & Bitboard::square_bb(sq)).is_empty()`;
   `file_of` returns a `File` newtype, so `rules::ep_key(file as usize)`.
5. **The `--ignored` tier runs the fixture writer** (`write_golden`), which
   writes to `$ATOMI_WRITE_GOLDEN` (default `/tmp`). Gate 2's regeneration was
   therefore run **explicitly into `/tmp`** and diffed against the committed
   fixture; its sha256 is unchanged at the end of the session.
6. **Censored cells carry no tree verdict.** A wall-capped run stops where the
   clock stops, so `child_evals` differ run to run on `core`/`deep-win` at 10 s
   and `deep-win` at 120 s; tree identity is claimed on the paired cells and
   the 5 M probe only.
7. **Per-node has both signs** (`core` 10 M: 0.987 in `m1`'s favour;
   `deep-win` 10 M: 57.2 → 58.8 ms against it): flat. The screen sits on the
   expansion path (`rules::status` → `outcome()` →
   `has_legal_move_with_state`); the wall does not see it.
8. **`make pernode`'s `gate` block is null** for a single-side run (as
   report2 §4.6); `m1_reference` is the S4 point and was right (62.5 ms/1M,
   1,999.7 ns). **`nproc` in the shell reports 4** (cgroup mask) while the
   machine reports 32 CPUs; `env.json` records both, so a future session does
   not read the ratio as a hardware change.

## 5. Plan §9 questions, answered

- **1. Does the screen move atomi's wall?** **No.** `core` 120 s 1.0065,
  `easy-win` 120 s 1.0026, `easy-win` 10 s 0.9984, 5 M probe 0.9996 — inside
  the spread, both signs. Per-node `core` 10 M 2,006 → 1,980 ns (−1.3 %) with
  identical `child_evals`, so the effect, if real, is ~1 % per node, not a
  rung-relevant datum. The `Terminal` precedence refactor is covered by gates
  2–3 and shows no divergence.
- **2. Screen trust boundary.** Pinned: the screen agrees with
  `generate_legal` on 51,633 children and 2,859 parents, and the extinction
  guard is the first statement in the crate's view, so a stale `StateInfo` can
  only produce `false`. atomi passes no stale one today.

## 6. Missing tests / open parts

- The witness path is unobservable (§4.2): no test can assert "the screen
  decided this node" without a crate-side counter — an ask for a future
  handoff, not a blocker.
- MG19's key is pinned against our *built child*, not against a call site: no
  atomi code reads `post_move_and_key`.
- Perft depth 5 still out (as report1 §6); the trigger remains a release that
  announces a rules fix. `unsafe-optimizations` stays out of scope (D-MG6).

## 7. Next steps

1. **No `prover` plan from this release** (plan3 §8). `handoff3.md` §3's gate
   ("MG19 + MG20 … median wall ≤ 0.95× control") is **failed**, not
   superseded: with the key the pair is 0.97× where 0.84× is needed, and
   MG20's 12 ns bar is missed 3.5× over.
2. **`handoff4.md` is unspent** until a session has (a) a formulation that does
   not read the child board and (b) cost bars re-derived from it — the last
   ones came from a `--guide hand` share the bare tree no longer has (D-MG7).
3. **S2 still runs**: a 2.9.0 gets a plan within one plan of noticing it; the
   caret is at `"2.8"`, so a patch drifts silently but a minor does not.

## Keep/revert line (gate 5)

**Keep** — `atomic-movegen` **2.8.0**; gates 1–4 green (crate harness
173 + 51 + 16, frozen differential **diff empty**, both differentials green
and mutation-checked, `make test-full` slow tier 4/4, `make veto` 8/8 accepted
/ 4/4 controls rejected); paired `m1/m0` geomean **1.0025** at tree identity
3/3 (5 M probe 0.9996×, per-expansion 1,999.7 vs 2,006.1 ns); **reason:** flat
on the wall, no rules divergence, and it is the published version that carries
MG19–MG20 — recorded as API, not as a cost win.

## Commit sequence (the owner commits)

1. `deps: atomic-movegen 2.8.0` — `Cargo.toml` + `Cargo.lock`, nothing else;
   the frozen differential is green and unchanged.
2. `test(movegen): pin the 2.8.0 screen and child summary` —
   `tests/child_surface_differential.rs`, `tests/legality_screen_differential.rs`.
3. `docs(movegen): report3, measurements and bench rows` —
   `docs/plans/movegen/{report3,initiative}.md`, `docs/plans/README.md`,
   `docs/plans/prover/bench.md`, `measurements/movegen/plan3/`.
