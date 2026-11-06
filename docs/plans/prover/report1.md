# Prover Report 1: M0 — bootstrap, correctness, minimal proof output, re-measured baseline

Plan: [`plan1.md`](plan1.md) (initiative [`prover`](initiative.md), backlog
item 1, M0). Executed 2026-11-06 in one session, phases A–F, with no split.

## Headline

- The M0 loop works end to end: FEN → attacker-only DF-PN → proof events →
  strategy store → JSONL → `atomi validate` → `provari check`. All gates are
  green: `make test`, `make test-full` (slow tier + `make veto`) and
  `make lint`.
- **Every claimed win is checker-accepted.** All 8 veto rows are `accepted`,
  and all 4 negative controls are `valid: false` on both sides (no `null`).
  The reverse control agrees on all 8 rows. Beyond the gate, all **41** proofs
  of the 10 M fixture sweep were accepted by both checkers, with
  `positions == won_positions` on every file.
- **Fixture oracle:** 0 `wins.tsv` refutations and 0 `never_prove.tsv` finds.
  At 10 M: 41 proved, 11 censored, 0 refuted. At 2 M (slow tier): 35 / 17 / 0.
  0 export failures; 0 positions written with more than one move.
- **A completeness bug was found and fixed during the session.** Open
  proof/disproof numbers saturated at `INFINITY`, the value that marks
  decided nodes, and the root mapped any non-proved value to `refuted`. Two
  legacy wins (`dec10`, `rem01`) came out falsely `refuted`. There was no
  false win (proofs need proved children). Details: §Problems and findings 1.
- **Baseline, no performance claim for atomi M0.** Same session, 10 s /
  192 MB, provari v0.1.0 hand = 1.000:
  - legacy `default` 45.358 (1 cell);
  - legacy `opt` 6.322 (2 cells);
  - atomi M0 36.365 (1 cell, coverage 1/3).
  - Stretch: v0.2.0 / v0.1.0 = 1.001, with node counts identical at one
    thread.
- **M1 reference number: 162.0 ms per 1 M child evals** on `core` at
  `--evals 10000000`.

## Provenance

| Item | Value |
| --- | --- |
| Host | `ae54d05b36f8`, AMD Ryzen 9 5950X (container; 4 vCPU per plan finding 10; `os.cpu_count()` reports 32), Linux 7.2.13 Fedora 44, x86-64 |
| Toolchain | `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, cargo 1.99.1, Python 3.14.10 |
| Dependencies | `atomic-movegen` 2.4.0, `serde_json` 1.0.154 (published 2026-10-11, ≥ 7 days before use); licence gate: 13 packages, all clean |
| atomi | `atomi 0.1.0 (0c5a882…)`, built from HEAD `0c5a882ef9ab93c0d48d6a3f5176cfb64a0a78bb` on a **dirty tree** (plan1 work uncommitted; agents do not commit); bench binary sha256 `7e2e0c20…3818f` |
| Oracle (veto, primary column) | provari `48686bc6c786d6ea70568f0b2ad4002cf0520206`, `provari 0.1.0`, `/tmp/atomi-oracle/provari` |
| Stretch reference | provari `224ce85dad083b1bc8fe429a9e4349de956f5305`, `provari 0.2.0`, `/tmp/atomi-oracle/provari-v0.2.0` |
| Legacy control | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00`, `git archive` export built in `/tmp/atomic_solver-5f85133` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` (verified) |

The full block is in [`measurements/plan1/env.json`](measurements/plan1/env.json)
and [`measurements/plan1/README.md`](measurements/plan1/README.md).

## Exact commands

```sh
make test; make lint; make oracle; make oracle-stretch; make test-full
cargo build --release
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
python3 bench/run_pernode.py --atomi target/release/atomi \
  --cases core,1eb80b92f92c,db6a741c723f --evals 1000000,10000000 \
  --repeats 3 --out docs/plans/prover/measurements/plan1/pernode.json
```

Fixture sweep (not committed, outside the repo): every `wins.tsv` row at
`atomi prove FEN --evals 10000000 --memory 192 --proof-out …`, then
`atomi validate` and `provari check` on every written proof.

## Results

### Fixtures

| File | Rows | Source / construction |
| --- | ---: | --- |
| `wins.tsv` | 52 | 32 `decisive_positions` + 9 `decisive_remaining` + 10 `move_order` side-to-move `Win` rows + the `test_ghi.rs` transposition FEN; 0 duplicates by FEN |
| `never_prove.tsv` | 31 | 24 side-to-move `Loss` rows (14 + 1 + 9), 3 `test_terminal_outcomes.rs` Loss claims, 2 K-vs-K draws, the rook-alone R1 row, the `QQ2k3` black-to-move GHI counterpart |
| `terminal.tsv` | 52 | legacy static claims re-checked against 2.4.0 + the R1 material rows (both colours to move, both owners) |
| `soundness.tsv` | 6 | constructed (header records the construction and evidence) |
| `veto.tsv` | 8 | chosen from the above |

The re-check against 2.4.x found **no disagreement** with the copied legacy
static claims: a mate at counter 100 is a loss, a stalemate at 100 is a draw,
and touching commoners in K vs K are a draw. No `checker-rules` disposition
was needed: no `wins.tsv` row is refuted.

### Veto (`make veto`, final tree)

| Row | Class | positions / won_positions | Length | header `moves` | Checker | Reverse control |
| --- | --- | ---: | ---: | ---: | --- | --- |
| `m1_explosion` | accepted | 1 / 1 | 1 | 1 | true | agree |
| `m1_checkmate` | accepted | 1 / 1 | 1 | 1 | true | agree |
| `m2` | accepted | 2 / 2 | 2 | 2 | true | agree |
| `smallest_win` (`m29_white`) | accepted | 1 / 1 | 1 | 1 | true | agree |
| `mate_100th_ply` | accepted | 1 / 1 | 1 | 1 | true | agree |
| `rule50_97` | accepted | 2 / 2 | 2 | 2 | true | agree |
| `detour` (`dec08`) | accepted | 2 / 2 | 2 | 2 | true | agree |
| `control` (`dec11`, 30 pieces, `KQkq`) | accepted | 308 / 308 | 4 | 4 | true | agree (provari's own proof: 336 / 336) |

Negative controls on `control` (plus the 50-move control on `rule50_97`):

| Control | atomi validate | provari check |
| --- | --- | --- |
| drop the root's line | false | false |
| illegal root move `e1f1` (commoner onto an own piece) | false ("illegal move") | false |
| alternative root move `b2b4` (first in sorted UCI order) | false | false |
| every counter + 1 on `rule50_97` | false | false |

The reverse control runs `provari prove FEN --guide hand --proofs …`, then
both checkers. Their `valid` agreed on all 8 rows, with
`positions == won_positions`.

### Fixture oracle

| Run | proved | censored | refuted | export failures | multi-move positions |
| --- | ---: | ---: | ---: | ---: | ---: |
| `wins.tsv` @ 2 M (slow tier) | 35 | 17 | 0 | 0 | 0 |
| `wins.tsv` @ 10 M (sweep) | 41 | 11 | 0 | 0 | 0 |
| `never_prove.tsv` @ 2 M (slow tier) | 0 found of 31 | — | — | — | — |

The 10 M censored rows are `rem07`–`rem13` and `m20`–`m23_white`. The
largest proofs are `rem01` (length 65, 9,429,006 evals), `dec10` (25,
2,600,650), `dec01` (22, 4,677,134) and `dec15` (21). The smallest
`wins.tsv` win M0 proves within 10 M is `m29_white` (34 child evals,
length 1). The smallest non-trivial checker-accepted win in the veto is
`control` (`dec11`, length 4, 150,775 evals). All 41 sweep proofs:
`atomi validate` valid, `provari check` valid, positions equal.

**Proof shapes (D6).** Promotion **is** exercised and checker-accepted: 537
promotion lines across `dec05`, `dec10`, `dec15`, `dec21`,
`ghi_transposition` and `rem01`. **Castling and en passant are not
exercised** by any written proof (0 king castling moves with rights, 0 en
passant captures). Only the unit round trip covers their UCI rendering
(castling both sides and wings, en passant, all four promotion pieces via
`Board::move_from_uci`).

### Bench (plan1 rows, same session)

| Side | `core` | `easy-win` | `deep-win` | Geomean | Coverage |
| --- | ---: | ---: | ---: | ---: | ---: |
| provari v0.1.0 hand (reference) | 0.533 s · 342,289 nodes · mate 17 | 0.137 s · 72,523 · mate 15 | 6.567 s · 4,574,912 · mate 22 | 1.000 | 3/3 |
| atomi **m0** | censored (≥ 18.76) 10.060 s / 59.5 M | **36.365** (4.982 s / 30,245,030) | censored (≥ 1.52) 10.061 s / 61.8 M | **36.365** (1 cell) | 1/3 |
| legacy **default** | censored (≥ 18.76) 10.020 s / 66.2 M | **45.358** (6.214 s / 43.3 M) | censored (≥ 1.52) 10.019 s / 67.6 M | **45.358** (1 cell) | 1/3 |
| legacy **opt** | **7.43** (3.960 s / 17.2 M) | **5.38** (0.737 s / 3.23 M) | censored (≥ 1.52) 10.019 s / 46.2 M | **6.322** (2 cells) | 2/3 |
| stretch: provari v0.2.0 hand / v0.1.0 | 0.987 | 1.007 | 1.010 | 1.001 (stretch block only) | 3/3 |

- **D14.** The atomi `easy-win` cell re-ran with `--seconds 600` and
  `--proof-out`. It showed identical `child_evals` (30,245,030; all three
  timed repeats were equal), `proof: written`, 2,083 positions and length 17,
  and both checkers accepted it: `proof_verdict = accepted`.
- **Re-priced gap.** After 2.4.0, the best legacy control (`opt`) is 6.322 on
  two shared cells and is censored on `deep-win`. This is the initiative's
  baseline gap. Per reading rule 1, it is not compared with frozen legacy
  rows.
- **No performance claim for atomi M0.**
- **Moving target.** v0.2.0 at its default single thread reproduces
  v0.1.0's node counts exactly on all three cells. The 1.001 ratio is wall
  noise around an unchanged trajectory. (`--threads 2` does change the
  search: `core` 420,116 nodes, mate 16, in a spot check. This is not
  part of the protocol and not recorded as a row.)

Ledger: [`bench.md`](bench.md). Raw parsed data:
[`measurements/plan1/bench.json`](measurements/plan1/bench.json).

### Per-node baseline (D11)

The medians cover 3 repeats, all of which ran to the cap (no early runs).

| Cell | Cap | ms per 1 M child evals | ns per expansion | child evals / expansion | Peak RSS | Process wall |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `core` | 1 M | 159.0 | 5,165 | 32.5 | 130.7 MiB | 0.223 s |
| `core` | 10 M | **162.0** | 5,183 | 32.0 | 130.8 MiB | 1.680 s |
| `easy-win` | 1 M | 145.0 | 4,573 | 31.5 | 130.5 MiB | 0.206 s |
| `easy-win` | 10 M | 155.2 | 4,828 | 31.1 | 130.5 MiB | 1.619 s |
| `deep-win` | 1 M | 150.0 | 4,454 | 29.7 | 130.7 MiB | 0.211 s |
| `deep-win` | 10 M | 158.8 | 4,775 | 30.1 | 130.9 MiB | 1.652 s |

- **M1 reference: 162.0 ms per 1 M child evals** (`core`, 10 M).
- Legacy plan3 context (cross-host aarch64 `67db7a1089e2`, **not
  comparable**): about 0.34–0.37 s per 1 M front-end evals. The
  same-session legacy front-end number is M1's measurement.
- RSS: the M0 table rounds its bucket count down to a power of two, so the
  192 MB label allocates 128 MiB.

## Gate table

| Gate | Result |
| --- | --- |
| `make test` (licence gate + `cargo test --release`, 54 fast tests) | green, ≈ 12 s |
| `make test-full` (+ 3 slow tests in 9.4 s + `make veto`) | green, VETO: PASS |
| `make lint` (fmt, clippy `-D warnings`, doc) | green |
| `cargo build --release` | clean, no warnings |
| Licence gate | 13 packages, no GPL family, no `atomic_solver` dependency |
| Python `ast.parse` (bench/, scripts/) | clean |
| `sha256sum bench/corpus.json` | `9cd0169a…47d0e` |
| `git add --dry-run .` | only intended files (no `target/`, proofs, transcripts, `__pycache__`) |
| Determinism / sink independence | tests green: identical counters on repeat runs and NullSink vs store; bench repeats bit-identical in `child_evals` on proved cells |

## Verdict

**M0 acceptance met.** Specifically:

- the two mates in one, the mate in two and the small wins are proved,
  validator-accepted and checker-accepted, with matching positions;
- the controls are all `false`/`false`, and the reverse control agrees;
- no never-prove or soundness negative is found, and no `wins.tsv` row is
  refuted;
- the ledger, the stretch row, the per-node baseline and the M1 reference
  are recorded;
- provenance is complete and the corpus is byte-identical.

## Not measured

- The legacy win front-end's same-session per-node number (M1's job).
- provari v0.2.0 with `--threads > 1` (outside the protocol).
- `--guide model` (D8, stretch only).
- The 10 M fixture sweep is a one-off session measurement. Its table is not
  committed (it lives outside the repo); the counts are above.

## Problems and findings

1. **False refutations from Open saturation (fixed).** At 10 M, the first
   fixture sweep refuted `dec10` and `rem01`. provari proved both, with
   proofs accepted by both checkers, so this was an atomi bug, not a legacy
   claim or a rules issue.
   - **Narrowing.** I bisected over provari's `rem01` proof to the minimal
     position `1k6/8/8/p4p1p/P4P1P/4K1p1/8/8 b - - 1 54`, refuted after
     23,594 evals although every sub-position of its proof is provable
     standalone.
   - **Ruled out.** Disabling the frame-exit cycle reset (GHI) or the
     table's refutation stores changed nothing.
   - **Root cause.** Instrumentation showed the root leaving its MID loop as
     `Open { pn: 18, dn: 2^30 }`. dn sums grow through transpositions (an
     AND node at ply 6 had dn ≈ 1.05e9) and saturated at `INFINITY`. That
     tripped the root threshold and made saturated Open children look
     proved to their parents. `Engine::run` then mapped every non-proved
     root value to `refuted`.
   - **Fix.** Open sums saturate at `OPEN_MAX = INFINITY - 1`. The root
     re-enters MID until it is decided or a budget stops it, and only
     `Value::Refuted` maps to `exit = "refuted"`. The pawn ending now proves
     in 26,285 evals, `dec10` in 2.60 M and `rem01` in 9.43 M; all are
     checker-accepted.
   - **Regression tests.** The pawn ending is in the fast tier; `dec10` and
     `rem01` are in the slow tier, with proofs validated.
   - **Not a soundness failure.** No false win was possible, because a proof
     needs proved children, and Open values never count as proved in the
     value algebra. An Open child with dn = INF only misguided selection.
   - **Process note.** The first fix attempt declared `OPEN_MAX` but left
     both clamps at `INFINITY`. The symptom (huge latency, never a
     refutation) was first misdiagnosed as threshold thrash. A diff against
     the scratch experiment found the missed edit.
2. **Legacy fixtures: no stale claims surfaced.** Every 2.4.0 static re-check
   agreed with the legacy claims.
3. **K + R vs K "mate" rows cannot be constructed.** A commoner never
   controls squares (it cannot capture) and touching commoners are immune to
   check. So a lone rook (or knight or bishop) can never mate a bare
   commoner, and a queen mates only from an adjacent square. The planned
   `won`-in-K+R-v-K `terminal.tsv` row and the "mate in one inside a material
   draw" search test therefore became:
   - `won` coverage via the extinction rows and `fifty_move_checkmate`;
   - "the rook-alone root is refuted with 0 expansions".
4. **provari v0.1.0 accepts `--guide hand`.** Its short `--help` omits the
   option, but its README documents it, `--guide hand` runs, and a bogus
   value errors. An intermediate veto version had dropped the flag; it is
   passed explicitly now, as are the bench commands. Recorded in `AGENTS.md`.
5. **v0.2.0 single-thread is trajectory-identical to v0.1.0** on the bench
   cells (bench §Results). The moving target is small at 1 thread. Whether a
   threaded reference is fair is an M5 question.
6. **Validator `positions`.** A first version also counted won defender
   memo entries (3 vs the checker's 2 on `m2`). The veto caught it, and the
   validator now counts won attacker (position, counter) pairs only, the
   checker's `won_positions` definition.
7. **Deviations from plan1, recorded here:**
   - `terminal.tsv` has an extra `attacker` column (`w`/`b`/`both`), because
     a status needs an attacker colour;
   - `veto.tsv` has an extra `role` column (`prove`/`control`/`rule50`);
   - a diagnostic `atomi moves "<FEN>"` subcommand: veto.py needs the legal
     root moves for the alternative-move control, without a chess library;
   - the bench driver's proof-check also flags `nondeterministic` when the
     timed repeats disagree among themselves;
   - four files exceed ~10 KB and carry header justifications:
     `src/search/value.rs` 15.4 KB, `src/strategy/write.rs` 16.6 KB,
     `src/validate/mod.rs` 14.2 KB (mostly unit tests) and `tests/search.rs`
     11.7 KB. `bench/run_bench.py` (~17 KB) keeps the source driver's
     justification;
   - no `[profile.gate]` was needed: `make test` takes ≈ 12 s with full LTO.
8. **Path-dependent thrash** (risk table) was not visible as a problem. The
   saturation bug was the only pathology found.

## Missing tests

- **A real repetition-free-detour win.** The `detour` row is legacy `dec08`,
  a mate in 2, and does not force the proof around a repetition. Construct
  one where the shortest-looking line repeats a position.
- **The two-counter writer case end to end.** The per-(position, counter)
  choice is unit-tested at the `choose()` level only, and no real proof so
  far needed two moves for one position (multi-move = 0 everywhere).
- **A cross-line cycle written by the writer.** It is tested at the
  cycle-detector level plus a hand-made ping-pong file for the validator,
  not via a real store.
- **A line FEN with a non-capturable en passant square** in the validator's
  identity test (needs a real ep-bearing win).
- **Castling and en passant inside a checker-accepted proof** (D6).
  Promotion is covered.

## Next steps

1. **plan2 = M1 (item 2, per-node cost).** `Mover`/`Effect` per expanded
   node, child keys without make/unmake, TT clusters + prefetch + filter,
   lazy zeroed allocation (copy legacy `alloc.rs`), and the in-search proof
   store inside the `--memory` label.
   - Gate: ms per 1 M child evals ≤ **162.0** (this report) **and** ≤ the
     legacy front-end's, measured in the same session.
   - Stop rule: handoff §7 (initiative §Backlog).
2. Keep the `OPEN_MAX` rule in M1's value representation. Re-run the 10 M
   fixture sweep with both checkers as part of the M1 gate evidence; it is
   cheap.
3. Add the missing tests above when the M1/M4 code paths touch them.
   Castling and en passant need a fixture row whose proof contains them.

## Delivered files

- **Package and gates:** `Cargo.toml`, `Cargo.lock`, `build.rs`, `Makefile`,
  `.gitignore`, `PROVENANCE.md`, `scripts/{build_oracle.sh, veto.py,
  check_licenses.py}`.
- **Engine:** `src/{lib.rs, main.rs, cli.rs, rules.rs, table.rs, events.rs}`,
  `src/search/{mod.rs, frame.rs, value.rs}`,
  `src/strategy/{mod.rs, store.rs, write.rs}`, `src/validate/mod.rs`.
- **Tests:** `tests/{corpus, rules, search, validate, fixture_oracle}.rs`,
  `tests/fixtures/{wins, never_prove, terminal, soundness, veto}.tsv`.
- **Bench:** `bench/{corpus.json, run_bench.py, run_pernode.py, README.md}`.
- **Docs:** `docs/plans/README.md`, `docs/plans/prover/bench.md`,
  `docs/plans/prover/measurements/plan1/{README.md, env.json, bench.json,
  pernode.json, pernode.csv}`, updated `AGENTS.md` and `initiative.md`, this
  report.
- **Scratch, never committed:** `/tmp/atomi-oracle/provari` (v0.1.0),
  `/tmp/atomi-oracle/provari-v0.2.0`, `/tmp/atomic_solver-5f85133`.

## SESSION COMPLETE

**Deliverables.** M0 is implemented and every gate is green, including the
oracle veto. The baseline ledger and the per-node baseline are recorded, and
the M1 reference number is 162.0 ms per 1 M child evals. A completeness bug
(Open saturation) was found and fixed, with regression tests. Nothing is
committed: the owner commits the working tree (45 new files plus edits to
`AGENTS.md` and `initiative.md`).

**Follow-up options.**

1. Plan M1. Kickoff prompt:

   > Plan session for `docs/plans/prover/plan2.md` (initiative `prover`,
   > backlog item 2, M1 per-node cost). Read `AGENTS.md`,
   > `docs/plans/prover/initiative.md`, `plan1.md` and `report1.md` first.
   > The baseline is atomi M0 at 162.0 ms per 1 M child evals on `core` at
   > `--evals 10000000` (`measurements/plan1/pernode.json`). Run read-only
   > probes only: profile M0's per-child cost (clone + `do_move` +
   > `outcome()` per child), read the `atomic-movegen` 2.4.0 primitives
   > named in D2, and read the legacy `src/search/tt/alloc.rs` and the
   > legacy front-end at pin `5f85133`. Then write a self-contained
   > `plan2.md` for one sitting. It must cover the `Mover`/`Effect` layer,
   > child keys without make/unmake, TT clusters + prefetch + filter + lazy
   > zeroed allocation, and the in-search proof store inside the `--memory`
   > label. Keep the `OPEN_MAX` saturation rule. The gate is a same-session
   > per-node measurement against both M0 and the legacy front-end, a
   > checker-accepted small win within a stated cap, green soundness
   > fixtures, and a 10 M fixture sweep with both checkers. State the stop
   > rule (initiative §Backlog).

2. Add the missing tests first (a real detour win, the two-counter writer
   case end to end, castling/en passant inside a checker-accepted proof),
   as a small housekeeping plan before M1.
3. Commit the plan1 tree and re-run `make bench` on the committed sha, so
   that the ledger's `atomi@…` pin names a clean tree instead of
   `0c5a882+dirty`.
