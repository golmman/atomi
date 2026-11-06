# Initiative: `prover` — a from-scratch, independently-checked atomic win prover that beats provari

## Status

**Open (2026-11-06).** Item 1 (M0) is planned in [`plan1.md`](plan1.md); no
code exists yet. This initiative is the successor to `atomic_solver`'s closed
`winprover` initiative. Its starting point is the legacy handoff
`docs/plans/winprover/HANDOFF.md` (revision 2026-11-06) in
<https://github.com/golmman/atomic_solver>, read from `main`; legacy code pin
`5f85133ea9d751368a682e53e354bd3fc0016d00`.

| Item | Value |
| --- | --- |
| Repository | `golmman/atomi` (this repo) |
| Package / binary | `atomi` (owner decision, D13); bench rows pin `atomi@<sha>` |
| Initiative directory | `docs/plans/prover/` |
| Rules crate | `atomic-movegen` 2.4.x (crates.io, MIT) |
| Pinned oracle | `aiorla/provari` `48686bc6c786d6ea70568f0b2ad4002cf0520206` (`v0.1.0`), GPL-3.0, scratch build only |
| Stretch reference | provari `224ce85dad083b1bc8fe429a9e4349de956f5305` (`v0.2.0`), bench column only, never the veto (D17) |
| Legacy reference | `atomic_solver` at `5f85133ea9d751368a682e53e354bd3fc0016d00`, read-only sibling clone (`../atomic_solver`), never a dependency |
| Corpus | `corpus-v1`, sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` |

## Goal

**Be the fastest atomic win prover on the fair, reproducible shared-win
corpus, with every claimed win checked by our own replay validator and by an
independent second rules implementation.**

Success condition (from the handoff §2; do not silently change it):

- on the shared-win set, the geometric mean of `atomi / provari --guide hand`
  wall time is `< 1.0`;
- coverage is at least provari hand's (currently 3/3 on the bench cells);
- every claimed win is accepted by this repo's own replay validator **and** by
  the pinned `provari check`;
- this repo's gates are green, and there are no outcome regressions on the
  copied fixture oracle (§Corpus and fixtures). The legacy solver's gates are
  not this repo's gates.

Three legs, one goal:

1. **Search:** an attacker-only DF-PN prover. It is built around
   provari-style per-node precomputation and search guidance from the start,
   not bolted onto a general Win/Loss/Draw solver.
2. **Evidence:** a proof file for every claimed win, accepted by our validator
   and by the external checker. A checker disagreement is a veto.
3. **Fairness:** a pinned oracle, a sha256-pinned corpus, the same budgets on
   both sides, and same-session ratios.

### Why from scratch (the measured case, short form)

`winprover` ported every portable provari lever into the general solver and
plateaued. Its best opt-in stack was **6.30×** provari hand on 2 shared cells
(coverage 2/3), and the shipped default was **44.06×** (1/3). Probes measured
out the remaining small levers. The opt-in win front-end had the right shape,
but it paid a full `Position::outcome()` movegen per child, and even with the
opt starting numbers it was 2.2× the general solver's work on `core`. The gap
is per-node cost and search efficiency in the engine core
(legacy `report5.md`). Those numbers predate `atomic-movegen` 2.4.0, whose rules
fixes moved search trajectories. They are the record of the closed initiative,
**not** this initiative's baseline. M0 re-measures the baseline.

## Non-goals

- **No provari or shakmaty code, dependency (of any kind, including dev/build),
  binary, network or model in this repo.** No vendored GPL source, no committed
  oracle binary, and no committed provari-written proofs. The oracle runs as a
  separate process from a scratch directory outside the repo (D12).
- **No crate dependency on `atomic_solver`** (path, git or published). Legacy
  code and data come over only by copy-with-provenance. The legacy binary is
  used only as an external process.
- **No learned model.** The hand guide is the product. provari's
  `--guide model` is at most a separately recorded stretch column (D8).
- **No draw or loss proofs, and no general outcome search.** The engine proves
  *attacker wins* or reports *no win found* (refuted or censored).
  Root-`Loss` coverage is out of scope unless a defender-side mode is planned
  explicitly.
- **No tablebase integration through M5** (D7).
- **No weakening of soundness.** Repetition and the 50-move limit are never
  wins. A faster false win is a failure.
- **No portfolio or parallel search in the core** (D6). No proof corpus or
  database before M5 (horizon).

## Working agreement

Adapted from `winprover`'s working agreement.

1. **Measure before planning.** Every plan starts from the committed baseline
   and read-only probes. Sizing spikes are temporary and reverted.
2. **One mechanism per plan.** A plan adds one mechanism, or one coupled pair
   stated up front. The exception is M0 (plan1), which is a declared bootstrap
   bundle because nothing exists yet.
3. **Correctness gates are vetoes.** `make test` runs on every change.
   `make test-full`, which includes the oracle veto, runs for search, table,
   proof-output and validator changes. A bench row or speed claim is written
   only after the gate passes.
4. **Proof claims.** A win is *claimed* only after the emitted proof passed the
   own validator **and** `provari check`. Every non-accept is classified with
   one of these labels: `accepted`, `validator-reject`, `checker-reject`,
   `unchecked` (checker says `valid: null`), `export-failure`,
   `rules-divergence` (validator accepts, checker rejects, and the cause is a
   documented rules difference). Only `accepted` counts. `winprover`'s
   `convention-repeat` class no longer exists: our validator enforces the same
   no-repetition rule as the checker.
5. **Deterministic budget contract.** `--evals N` bounds `child_evals`. The same
   binary, FEN and flags give identical counters. Collecting a proof never
   changes the search trajectory.
6. **Oracle isolation.** provari is built from source at the pin in a scratch
   directory (`scripts/build_oracle.sh`), and `provari --version` is recorded.
   If the oracle is unavailable, the result is `INCONCLUSIVE`; it is never
   substituted or skipped.
7. **Copy with provenance.** Every file or fragment copied from
   `atomic_solver` carries the line
   `from atomic_solver@<40-hex sha>:<path>` and has a row in `PROVENANCE.md`.
   After the copy, this repo owns it. Copies are re-made on purpose, never
   synced implicitly.
8. **Same-session measurement.** Factors are same-host, same-session ratios.
   The reading rules in [`bench.md`](bench.md) (created by plan1) apply.
9. **Negative results are recorded.** A mechanism that fails its gate closes
   with the evidence. Stop rules are honoured (§Backlog).

## Decisions

The handoff settled D1, D9 and D10. This plan session (2026-11-06) decides the
rest and adds D12–D16.

| # | Decision | Resolution |
| --- | --- | --- |
| D1 | Code location | **New repository** (handoff §6.1). `atomic_solver` is frozen as the legacy product, the `winprover` record, and an optional external cross-check |
| D2 | Board | **`atomic_movegen::Board` 2.4.x**. M0 uses plain legal movegen and make-move. M1 adds the own `Mover`/`Effect` layer on the 2.4.0 primitives (`hash_after_with`, `hash_and_rule50_after`, `is_legal`, `outcome_with`, `has_legal_move_with_state`, `blast_zone`, `attackers_to_color`). A new board only if M1 shows make/unmake cannot leave the hot path; first ask upstream for the missing primitive (legacy `movegen/feedback2.md` pattern). Any new board is differential-tested against `atomic-movegen` and, through proofs, the external checker (no shakmaty linkage, D12) |
| D3 | Proof format | **provari JSON lines first (M0)**. Header `{"root":FEN,"moves":L}`, which must have no `provari` field: that field routes the checker to its compact reader. Then one `{"fen":FEN,"move":UCI}` line per attacker (position, 50-move counter) pair. Compact `.pvr` comes in M4 (zstd + the format's move order). No `proof_tree.bin` and no native format |
| D4 | Key / rule50 / repetition | **Board-only key + proof length/limit from M0.** The key is `Board::hash_with(EnPassantMode::Legal)`; `atomic-movegen`'s hash already excludes the halfmove clock. **Correction to the handoff:** the `Legal` mode is a *correctness requirement*, not an M1 A/B. The checker treats positions as repeated using the legal-en-passant identity, so an `Always` key would let the search accept lines the checker rejects as repetitions. M1 may A/B the *cost* of computing the key (an effect-level flag), never its meaning |
| D5 | Restarts | A/B after M2, paired with generation-aware replacement (legacy `RestartSchedule::SHIPPED` idea). Not a default before measured |
| D6 | Portfolio | A separate driver outside the search core, after M5 (horizon) |
| D7 | Tablebases | None through M5. Proofs carry no tablebase verdicts, and `provari check` runs without `--tablebases`. An external optional path is a horizon item |
| D8 | Model / NNUE | Hand guide only. `--guide model` may be recorded as a stretch column and is never mixed into the primary one |
| D9 | Evidence | **Settled (handoff §4.2):** our own minimal attacker-only replay validator + `provari check` veto. The legacy validator is reachable only through a file (a horizon importer in the legacy repo) |
| D10 | Bench ledger | **Settled (handoff §2):** the legacy `bench.md` is frozen. The successor's [`bench.md`](bench.md) reuses its protocol, cells, corpus, oracle pin and reading rules, and opens with a re-measured post-2.4.0 baseline (provari hand + legacy `default`/`opt` same-session controls) |
| D11 | Per-node counters | **Both.** `child_evals`: +1 per child classified during an expansion (legacy `child_evals` semantics). `expansions`: +1 per node expansion that generates a child list (the analogue of provari's `nodes`; a diagnostic, not a ratio target). `--evals` bounds `child_evals`. The per-node cost metric is search time per 1 M child evals, from a fixed-`--evals` run |
| D12 | GPL boundary for dependencies (new) | **No GPL/LGPL/AGPL crate anywhere in the dependency graph, including dev- and build-dependencies.** shakmaty 0.30.1 is `GPL-3.0-or-later` (verified in its crate metadata), so the handoff §6's "shakmaty as dev-dependency" is overruled. The second rules implementation is reached only as the external `provari check` process. Enforced by a `cargo metadata` licence gate in `make test` |
| D13 | Names (new) | Package/binary `atomi`, initiative `docs/plans/prover/`, ledger rows `atomi@<sha>` (owner decision 2026-11-06) |
| D14 | Claimed-win protocol for the bench (new) | Timed runs write no proof, like provari's bench command. For each cell `atomi` wins, one extra untimed run with `--proof-out` (and `--seconds 600`, so the deadline cannot censor it; plan1 R6) must show identical `child_evals`, and its proof must pass the validator and `provari check`. Only then does the cell count as a win. Otherwise it is reported as censored with the verdict class |
| D15 | M0 proof store (new; writer revised by plan1 R2) | An event-fed in-memory store, active only with `--proof-out`, with its own cap (`--proof-memory`). It keeps a non-dominated set of `(move, length, limit)` per key. The writer chooses, per **(position, counter)**, the shortest entry that holds at that counter. It checks the position graph for cycles as the checker does, and fails closed. M4 replaces the store with the bounded event-fed strategy worker (no proof-memory stop) |
| D16 | Oracle gate (new; controls revised by plan1 R4) | `make veto` (our validator + `provari check` + corrupted-proof controls + a reverse control that validates provari-written JSONL) is part of `make test-full`. A missing oracle makes the gate **red**, not skipped. A control counts only as `valid: false`: the checker's `null` (small-material "cannot check") is `unchecked` and red |
| D17 | Stretch reference (new, owner decision 2026-11-06) | provari **v0.2.0** (`224ce85`, its `check.rs`/`FORMAT.md` identical to the pin, its search rewritten) is built in scratch and measured in the same session as a separately labelled `provari-v0.2.0-hand` column, with the default single thread. v0.1.0 stays the veto and the primary column of the success condition. Whether to re-pin the goal is decided at M5 from the recorded gap |

## Backlog

Items are stable semantic identities. Plans are numbered sequentially and
independently of items. One mechanism per plan, except item 1.

| # | Milestone | Scope | Gate | Status |
| --- | --- | --- | --- | --- |
| 1 | **M0** bootstrap + correctness + proof output + baseline | Cargo package, small `AGENTS.md`, Makefile gates, licence gate, provenance registry, oracle build script; copied corpus and fixtures; attacker-only DF-PN with 1+ε and uniform starting numbers over plain `atomic-movegen` movegen (no `Mover`, no heuristics); CLI; event sink + proof store + JSONL writer + own validator + `make veto`; adapted bench driver; successor `bench.md` with the re-measured baseline row; per-node cost baseline | gates green (incl. veto); proves a mate-in-1, a mate-in-2 and a small win, each validator- **and** checker-accepted; never proves a fixture loss or a soundness negative; baseline row and per-node baseline recorded; **no performance claim** | **planned: [`plan1.md`](plan1.md)** |
| 2 | **M1** per-node cost | `Mover`/`Effect` per expanded node; no make/unmake for most children; incremental keys; TT clusters + prefetch + filter + lazy zeroed allocation (copy legacy `src/search/tt/alloc.rs`); in-search proof store with length/limit, inside the `--memory` label | wall per 1 M child evals ≤ the M0 baseline **and** ≤ the legacy front-end's (cross-repo, same session); a small reference win proved within a stated cap and checker-accepted; soundness fixtures green | open |
| 3 | **M2** search guidance | move-class starting numbers, EF-PN hand-eval scaling, quiet grouping; constants exposed for an external optimizer (legacy `docs/spec/optimizer_interface.md` contract as spec) | `core` and `easy-win` proved within a stated cap; measured vs M1 | open |
| 4 | **M3** static tactics + simulation | per-child static tactics (immediate explosion, exposed king, check with no safe evasion, unparried double threat, cannot-win material, mate in one) as child scoring; sibling simulation as part of the AND-node loop | `core`/`easy-win` proved at a product-affordable cap; measured | open |
| 5 | **M4** scaled proof output | event-fed bounded strategy worker (no proof-memory stop), `.pvr` export | every claimed win replay-validated and checker-accepted at `deep-win` scale; the worker budget is honoured | open |
| 6 | **M5** head-to-head | 3 cells × 3 repeats, same-session provari hand reference + legacy control rows; default-flag decision | geomean < 1.0, coverage 3/3, no fixture-oracle regressions; otherwise record and decide | open |

**Stop rules (handoff §7).** If M1 does not improve per-node cost over the
legacy front-end, the per-node diagnosis is wrong: stop and re-diagnose before
M2–M3. If M3 cannot prove `core`/`easy-win` at a product-affordable cap, the
from-scratch path is not viable at this scope: record it and close.

**Horizon (unranked, after M5; carries `winprover` item 8):**

| # | Item | Note |
| --- | --- | --- |
| 7 | Restart schedule A/B (D5) | after M2; paired with generation-aware replacement |
| 8 | Salt-seeded portfolio driver (D6) | wall-clock play, outside the core |
| 9 | Portable attacker-only export for proofdb transport | needs a JSONL → `proof_tree.bin` importer **in the legacy repo** (also the D9 optional cross-check) |
| 10 | External `high200` corpus | out-of-distribution guard, added as a separate set with a recorded selection rule |
| 11 | Native second rules implementation | only with no shakmaty linkage (D12) |
| 12 | Durable proof corpus | own plan and schema; legacy `docs/spec/global_proof_store.md` as reference |
| 13 | Optional external tablebases (D7) | provari's path is external and optional too |
| 14 | `--guide model` stretch column (D8) | recorded only |

Cannot-win material is part of M3. Compact `.pvr` is M4. Root-`Loss`
coverage needs a defender-side mode (a new item if ever wanted). `winprover`
item 7 (workspace split) does not apply to a new repository.

## What we take, and how (handoff §3–§4)

Marks: **idea** = re-implement from a description (provari is ideas only,
GPL); **spec** = re-implement from a legacy module doc or document;
**copy** = copy own MIT code or data with provenance, then own it;
**external** = run a binary as a separate process.

| Source | Item | Mark | Milestone |
| --- | --- | --- | --- |
| provari | attacker-only two-outcome value (`proved` / `refuted` with repetition-depth and 50-move dependence) | idea | M0 |
| provari | board-only key + proof length/limit; "shorter or holds further" proof keeping | idea | M0 (TT + store), M1 (proof table) |
| provari | per-frame child numbers (a position keeps its children's numbers while searched) | idea | M0 |
| provari | `Mover`/`Effect` precomputation, child key before materialization, TT prefetch | idea | M1 |
| provari | 4-slot clusters, filter, least-work replacement; 12-byte proof entries | idea | M1 |
| provari | move-class numbers (`THREAT=4`, `QUIET=9`, `HANGING=15`, `WEAK_THREAT=8`, `EVASIONS=8`), EF-PN scaling, quiet grouping, 1+ε (`second + 1 + second/8`) | idea | M0 (1+ε only), M2 |
| provari | per-child static tactics; simulation (`SIMULATION=16`) | idea | M3 |
| provari | JSONL / `.pvr` proof format (FORMAT.md as interoperability spec; text not copied) | idea | M0 (JSONL), M4 (`.pvr`) |
| provari | `provari check` | external (veto) | M0 |
| provari | "the checker uses nothing from the prover" independence test | idea | M0 (our validator) |
| legacy | `ProofEvent` neutral event protocol; bounded worker; memory-budget contract (`src/proof_event.rs`, `src/proof_tree/worker.rs`) | spec | M0 (event type + sink), M4 (worker) |
| legacy | replay validator rules (`src/proof_tree/validate.rs`), JSONL export shape (`src/proof_tree/export.rs`) | spec | M0 |
| legacy | lazy zeroed TT allocation (`src/search/tt/alloc.rs`) and its liveness invariant | copy | M1 |
| legacy | restart schedule + generation-aware victim score | idea | horizon 7 |
| legacy | portfolio driver (`examples/portfolio`) | idea | horizon 8 |
| legacy | bench driver (`docs/plans/winprover/measurements/bench/run_bench.py`) | copy + adapt | M0 |
| legacy | optimizer contract (`docs/spec/optimizer_interface.md`) | spec | M2 |
| legacy | test tiers, drift protocol, repetition/rule50 soundness gate | spec (discipline) | M0 |
| legacy | corpus, bench cells + reference values, decisive fixtures, terminal and soundness fixtures | copy (data only) | M0 |
| legacy | `atomic_solver` binary (`default`/`opt` controls; front-end per-node control) | external | M0 (controls), M1 (front-end) |
| legacy | `examples/egtb_gen3` | external, optional | horizon 13 |

**Not taken (handoff §5):** the general Win/Loss/Draw search, repetition
cache, GHI discipline and refinement rounds; the clock-including Zobrist key;
the proof-tree binary format, the proofdb schema and the three-valued
`Position`/`Outcome` wrapper; `StaticAtomicScorer`; any crate dependency on
`atomic_solver`; any provari or shakmaty code, binary, dependency or model.

### Legacy measured-out levers (context, not binding)

These were measured inside the **general** solver under legacy conventions
(legacy `winprover/initiative.md` §Measured-out levers). The new engine
re-measures before adopting or rejecting:

- node-entry static immediate-explosion closure: neutral/worse, because it
  discards DF-PN ordering feedback. M3 uses the per-child form instead;
- EF-PN on both node kinds: rejected in the general solver (per-case floor).
  M2 measures it against the attacking-branch-only form;
- rule50-free key: ~0–0.7 % in the general solver. This is not a reason to
  drop the board-only key here, which is primarily a soundness and proof-reuse
  design (D4).

## Corpus, fixtures and measurement conventions

- **Primary column:** `provari prove --guide hand`. `--guide model` is a stretch
  column only (D8).
- **Bench cells** (unchanged): `core` (fingerprint `8d0ff8bca7f0`),
  `easy-win` (`1eb80b92f92c`), `deep-win` (`db6a741c723f`). FENs and the
  selection rule are in [`bench.md`](bench.md) (created by plan1, copied from
  the frozen legacy ledger).
- **Corpus:** `bench/corpus.json`, a verbatim copy of legacy
  `docs/plans/provari/measurements/plan1/corpus.json`, sha256
  `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`. The sha256
  is its identity. The bench driver refuses to run on a mismatch. New cases form
  a separate, separately reported set with a recorded selection rule.
- **Budgets:** 10 s and a 192 MB label on both sides. provari:
  `prove "<FEN>" --seconds 10 --memory 192 --guide hand`. atomi:
  `prove "<FEN>" --seconds 10 --memory 192`. Three repeats per cell and side
  for any reported wall ratio.
- **Fixture oracle:** known side-to-move wins copied from the legacy fixtures
  (`decisive_positions.txt`, `decisive_remaining.txt`,
  `move_order_positions.txt`, the quick suite). A *refutation* there is a bug
  to investigate. Censoring is a result. Legacy side-to-move losses are kept as
  **never-prove** negatives: proving one is a soundness failure.
- **Ledger:** [`bench.md`](bench.md). Every plan that touches the search
  appends one `atomi` row. The legacy control rows are re-measured only when a
  host or drift question needs them. Rows pin `atomi@<sha>`, `solver@<sha>` and
  the oracle commit.
- **Artifacts:** drivers, parsed results (`*.json`, `*.csv`), `env.json` and
  provenance READMEs go under `docs/plans/prover/measurements/planN/`. Raw
  transcripts, provari sources, binaries and proofs, and `__pycache__` are never
  committed.

## Risks

| Risk | Mitigation |
| --- | --- |
| GPL contamination (provari, shakmaty) | D12 licence gate; scratch-only oracle; ideas only; no provari text or code copied |
| Rules divergence between `atomic-movegen` and shakmaty (e.g. shakmaty's atomic insufficient-material draws are broader than `atomic-movegen`'s commoner-only `is_material_draw`) | Insufficient material: the checker's rule is adopted in our rules layer and validator (plan1 R1). Anything else: checker veto, the `rules-divergence` class, reported upstream when real |
| The reference moves (provari v0.2.0 rewrote its search) | D17 stretch column from M0; the re-pin decision is at M5 |
| Repetition identity mismatch (en passant) | D4: `EnPassantMode::Legal` key; validator compares exact positions with legal en passant |
| Cross-clock proof reuse produces a position cycle or a 50-move failure in the written proof | Length/limit reuse rules; the writer fails closed on cycle/limit/missing; validator + checker |
| Per-node cost is not the bottleneck | M1 stop rule |
| Scope creep | One mechanism per plan; milestone gates |
| Measurement drift | Same-session ratios; re-measured baseline; reading rules |
| Proof memory cap on deep wins | D15 now, bounded worker in M4 |
| Copied data drifts from its source | Provenance lines + `PROVENANCE.md` with source sha256 |
| The new engine never beats provari | `atomic_solver` stays the legacy product; this repo is archived with its measured record |

## History

- **2026-11-06**: initiative opened in plan session 1. Read the legacy handoff
  (rev. 2026-11-06), `report5.md`, `bench.md`, `movegen/report_update_2.4.0.md`
  and the pinned provari source (scratch clone, not committed). Decided
  D2–D8, D11 and new D12–D16. Recorded two corrections to the handoff:
  no shakmaty dev-dependency (GPL) and `EnPassantMode::Legal` as a correctness
  requirement. Wrote [`plan1.md`](plan1.md) (M0) and `AGENTS.md`.
- **2026-11-06**: plan1 decision review (plan1 §Decision review, R1–R11)
  against the pinned provari `check.rs`/`main.rs`/`search.rs`,
  `atomic-movegen` 2.4.0 and the legacy pin. It adopted the checker's
  insufficient-material draw (R1) and switched the writer to per-(position,
  counter) moves (R2, D15). The header `moves` became informational and
  capture-mates were counted as capture wins (R3). Controls must be
  `valid: false`, never `null` (R4, D16). Added D17 (v0.2.0 stretch column,
  owner decision).
