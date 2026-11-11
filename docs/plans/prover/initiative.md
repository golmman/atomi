# Initiative: `prover` — a from-scratch, independently-checked atomic win prover that beats provari

## Status

**Open (2026-11-10).** On the D30 staircase the engine is at **`T_2`** (`core`+`easy-win` at 120 s); `T_3` (`deep-win` at 120 s) is the next rung; `T_5` (the goal) is not met. The bare tree is the product candidate; item 24 (conformance line) Phase A passed (D29) and Phase B (plan17) is **negative**: exact replay cannot apply at the first divergence and cuts `core` only 1.02×, so the line is closed. Full record: [`history.md`](history.md); decision rationale: [`decisions.md`](decisions.md).

## Reference

This initiative is the successor to `atomic_solver`'s closed `winprover` initiative. Its starting point is the legacy handoff `docs/plans/winprover/HANDOFF.md` (revision 2026-11-06) in <https://github.com/golmman/atomic_solver>, read from `main`; legacy code pin `5f85133ea9d751368a682e53e354bd3fc0016d00`.

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

**Be the fastest atomic win prover on the fair, reproducible shared-win corpus, with every claimed win checked by our own replay validator and by an independent second rules implementation.**

Success condition (from the handoff §2; reformulated as a necessary-condition staircase by D30, superseding the tier clauses of D27/D28/D29):

T_5 (goal): all three cells proved at 10 s, geomean wall < 1.0 vs provari --guide hand, every claim D14-accepted.

The goal is the top rung of a chain of **necessary conditions**, each a subset of the next (`T_5 ⊆ T_4 ⊆ … ⊆ T_1`), so `goal ⇒ every rung` and a failed rung is a hard "not yet":

| Rung | Requirement | Implied by the next |
| --- | --- | --- |
| `T_1` | `core` proved at 120 s | yes |
| `T_2` | `core` + `easy-win` at 120 s | yes (more cells) |
| `T_3` | all three at 120 s | yes (more cells) |
| `T_4` | all three at 10 s | yes (tighter budget) |
| `T_5` | all three at 10 s **and** geomean `< 1.0` | — (the goal) |

Nesting is sound because the search is deterministic (D22) and `--seconds` only stops it, so proving at 10 s implies proving at 120 s; more cells is monotone by definition. The wall clause lives only at `T_5` because `geomean < 1` does not imply each cell `< 1`. A lower rung is necessary, never sufficient; a censored cell is never converted and a longer run is never substituted. Current position: `T_2` (plan15: `core` 3/3 at 17.5 s, `easy-win` 3/3 at 2.0 s, `deep-win` 1/3 at 120 s); the next rung is `T_3`, `deep-win` 3/3 at 120 s.

- every claimed win is accepted by this repo's own replay validator **and** by the pinned `provari check` (every rung);
- this repo's gates are green, and there are no outcome regressions on the copied fixture oracle (§Corpus and fixtures). The legacy solver's gates are not this repo's gates.
- the reference is re-pinned only via D17 at M5; the node/`child_evals` score option is not chosen.

Three legs, one staircase goal:

1. **Search:** an attacker-only DF-PN prover. It is built around provari-style per-node precomputation and search guidance from the start, not bolted onto a general Win/Loss/Draw solver.
2. **Evidence:** a proof file for every claimed win, accepted by our validator and by the external checker. A checker disagreement is a veto.
3. **Fairness:** a pinned oracle, a sha256-pinned corpus, the same budgets on both sides, and same-session ratios.

### Why from scratch (the measured case, short form)

`winprover` ported every portable provari lever into the general solver and plateaued. Its best opt-in stack was **6.30×** provari hand on 2 shared cells (coverage 2/3), and the shipped default was **44.06×** (1/3). Probes measured out the remaining small levers. The opt-in win front-end had the right shape, but it paid a full `Position::outcome()` movegen per child, and even with the opt starting numbers it was 2.2× the general solver's work on `core`. The gap is per-node cost and search efficiency in the engine core (legacy `report5.md`). Those numbers predate `atomic-movegen` 2.4.0, whose rules fixes moved search trajectories. They are the record of the closed initiative, **not** this initiative's baseline. M0 re-measures the baseline.

## Non-goals

- **No provari or shakmaty code, dependency (of any kind, including dev/build), binary, network or model in this repo.** No vendored GPL source, no committed oracle binary, and no committed provari-written proofs. The oracle runs as a separate process from a scratch directory outside the repo (D12).
- **No crate dependency on `atomic_solver`** (path, git or published). Legacy code and data come over only by copy-with-provenance. The legacy binary is used only as an external process.
- **No learned model.** The hand guide is the product. provari's `--guide model` is at most a separately recorded stretch column (D8).
- **No draw or loss proofs, and no general outcome search.** The engine proves *attacker wins* or reports *no win found* (refuted or censored). Root-`Loss` coverage is out of scope unless a defender-side mode is planned explicitly.
- **No tablebase integration through M5** (D7).
- **No weakening of soundness.** Repetition and the 50-move limit are never wins. A faster false win is a failure.
- **No portfolio or parallel search in the core** (D6). No proof corpus or database before M5 (horizon).

## Working agreement

Adapted from `winprover`'s working agreement.

1. **Measure before planning.** Every plan starts from the committed baseline and read-only probes. Sizing spikes are temporary and reverted.
2. **Stack-level gating.** A plan may add a **named stack** of coupled mechanisms under one gate, stated up front with its constituent mechanisms and its gate. The gate is judged on the whole stack; mechanism-level adopt/reject decisions come only from **both-direction ablations after the stack gate**, not from single-lever runs before it. A single-mechanism plan remains allowed when the mechanism is independently meaningful. M0 (plan1) was a declared bootstrap bundle; M2+M3 (plan3) is the first deliberate stack (D21).
3. **Correctness gates are vetoes.** `make test` runs on every change. `make test-full`, which includes the oracle veto, runs for search, table, proof-output and validator changes. A bench row or speed claim is written only after the gate passes.
4. **Proof claims.** A win is *claimed* only after the emitted proof passed the own validator **and** `provari check`. Every non-accept is classified with one of these labels: `accepted`, `validator-reject`, `checker-reject`, `unchecked` (checker says `valid: null`), `export-failure`, `rules-divergence` (validator accepts, checker rejects, and the cause is a documented rules difference). Only `accepted` counts. `winprover`'s `convention-repeat` class no longer exists: our validator enforces the same no-repetition rule as the checker.
5. **Deterministic budget contract.** `--evals N` bounds `child_evals`. The same binary, FEN and flags give identical counters. Collecting a proof never changes the search trajectory.
6. **Oracle isolation.** provari is built from source at the pin in a scratch directory (`scripts/build_oracle.sh`), and `provari --version` is recorded. If the oracle is unavailable, the result is `INCONCLUSIVE`; it is never substituted or skipped.
7. **Copy with provenance.** Every file or fragment copied from `atomic_solver` carries the line `from atomic_solver@<40-hex sha>:<path>` and has a row in `PROVENANCE.md`. After the copy, this repo owns it. Copies are re-made on purpose, never synced implicitly.
8. **Same-session measurement.** Factors are same-host, same-session ratios. The reading rules in [`bench.md`](bench.md) (created by plan1) apply.
9. **Negative results are recorded.** A mechanism that fails its gate closes with the evidence. Stop rules are honoured (§Backlog).

## Decisions

One-line resolutions; full text, rationale, source take-list and legacy measured-out levers in [`decisions.md`](decisions.md).

| # | Decision | Resolution |
| --- | --- | --- |
| D1 | Code location | New repository; `atomic_solver` frozen as legacy product, record and optional external cross-check |
| D2 | Board | `atomic_movegen::Board` 2.4.x; own `Mover`/`Effect` layer in M1; new board only if make/unmake cannot leave the hot path |
| D3 | Proof format | provari JSONL first (M0); compact `.pvr` in M4; no native format |
| D4 | Key / rule50 / repetition | Board-only key `hash_with(EnPassantMode::Legal)`; proof length/limit from M0; `Legal` is a correctness requirement |
| D5 | Restarts | A/B after the M2+M3 stack; reopened under item 18, not implemented (plan10 stop rule) |
| D6 | Portfolio | Separate driver outside the search core, after M5 |
| D7 | Tablebases | None through M5; external optional path is a horizon item |
| D8 | Model / NNUE | Hand guide only; `--guide model` is a stretch column, never mixed in |
| D9 | Evidence | Own minimal replay validator + `provari check` veto |
| D10 | Bench ledger | Successor `bench.md` reuses the legacy protocol, cells, corpus, oracle pin and reading rules; re-measured baseline |
| D11 | Per-node counters | `child_evals` and `expansions`; `--evals` bounds `child_evals`; per-node cost is search time per 1 M child evals |
| D12 | GPL boundary | No GPL/LGPL/AGPL crate anywhere, including dev/build; enforced by a `cargo metadata` licence gate |
| D13 | Names | Package/binary `atomi`, initiative `docs/plans/prover/`, rows `atomi@<sha>` |
| D14 | Claimed-win protocol | Timed runs write no proof; one untimed `--seconds 600 --proof-out` run must match `child_evals` and pass both checkers |
| D15 | M0 proof store | Event-fed in-memory store, active only with `--proof-out`; revised by D18 |
| D16 | Oracle gate | `make veto` in `make test-full`; missing oracle is red; controls must be `valid: false`, never `null` |
| D17 | Stretch reference | provari v0.2.0 as a separately labelled bench column; v0.1.0 stays the veto; re-pin decided at M5 |
| D18 | Proof memory from M1 | Full-key proof table always on, never evicts, refuses at 90 % load; `--memory` covers TT, proof table and filter |
| D19 | Lazy terminal detection | Exact for explosion, mate, material draw and 50-move; quiet stalemate deferred unless the child has only commoner and pawns |
| D20 | `unsafe` in M1 | Three sites with `// SAFETY:` comments and measured keep rules: zeroed allocator, prefetch, huge pages |
| D21 | Stack-level gating | Named stack under one gate; both-direction ablations after the gate; first stack is M2+M3 hand-guide |
| D22 | Guidance measurement and seeds | `--seed` perturbs only tie-breaks; seed 0 preserves movegen order; `guidance-v1` frozen by a provari-hand census |
| D23 | CORE-RESET and item-3 closure | Last attempt at item 3; G1 failed on `core`; item 3 closed, no mechanism split, no ledger row |
| D24 | M4 parked; wall is the gate | M4 resumes only after a 3/3 10 s win; search/guide gates are wall-based; auxiliary budgets stay deterministic |
| D25 | Deterministic auxiliary budget | `--guide-aux-budget` bounds aux work by a deterministic counter; opt-in only, not adopted |
| D26 | Per-expansion diagnosis | Judge per expansion and per node, never `child_evals` against provari nodes; `R > 2` makes core throughput first-order |
| D27 | Coverage-first goal re-pin | **Superseded by D30** (tier structure); coverage-before-wall ordering retained as the staircase |
| D28 | Tier 1 amended to 2/3 | **Goal clause superseded by D30**; the `deep-win` 1/3 measurement stands as evidence |
| D29 | Conformance line and Tier 2 coverage | **Tier 2 coverage clause superseded by D30**; item 24 authorised (differential, exact replay, exact `pn`/`dn`, restarts) |
| D30 | Goal staircase (necessary conditions) | `T_1`…`T_5`, each a subset of the next; `T_5` is the goal; supersedes the tier clauses of D27/D28/D29 |

## Backlog

Items are stable semantic identities. Plans are numbered sequentially and independently of items. Plans use stack-level gating (D21), except a single-mechanism plan that is independently meaningful.

**Priority after plan session 16 (2026-11-10), updated by plan17:** the bare tree is the product. On the D30 staircase the engine is at **`T_2`**; the next rung is **`T_3`** (`deep-win` 3/3 at 120 s), a coverage problem, not a wall problem. D29 authorised **item 24, the conformance line**. Phase A (plan16) found quiet grouping as the named `core` divergence; Phase B (plan17) failed both stops: no first move-divergent position is a defender node, and exact replay cuts `core` only 1.02× at 20 M. The conformance line is closed; the next decision is a formulation change or a new search algorithm. The 120 s budget is not raised, the hand-guide/auxiliary line stays closed, and no portfolio is built.

| # | Milestone | Gate | Status |
| --- | --- | --- | --- |
| 1 | **M0** bootstrap + correctness + proof output + baseline | gates green incl. veto; mate-in-1/2 and a small win both-checker accepted; baseline recorded; no performance claim | **done** — [plan1](plan1.md), [report1](report1.md) |
| 2 | **M1** per-node cost | wall per 1 M child evals ≤ M0 and ≤ legacy front-end; reference win accepted; soundness green | **done** — [plan2](plan2.md), [report2](report2.md); 53.4 ms |
| 3 | **M2+M3** hand-guide stack | D21: `easy-win` ≤ 145,046 expansions; `core` proved at 10 s / 192 MB; no refutations; ≥20 cells × seeds 0–3 | **closed; G1 failed** — [plan3](plan3.md), [plan4](plan4.md), [report3](report3.md), [report4](report4.md) |
| 4 | ~~M3 static tactics + simulation~~ | merged into item 3 (D21) | merged |
| 5 | **M4** scaled proof output | every claimed win replay-validated and checker-accepted at `deep-win` scale; worker budget honoured | **closed as superseded** — [plan15](plan15.md) |
| 6 | **M5** head-to-head | geomean < 1.0, coverage 3/3, no fixture-oracle regressions | **executed; Tier 2 fail** — [plan15](plan15.md), [report15](report15.md) |
| 16 | Wall attribution + deterministic auxiliary budget | `easy-win` wall ≤ 0.5 s and `core` proved at 10 s on seeds 0–2; no refutations | **closed negative** — [plan7](plan7.md), [report7](report7.md) |
| 17 | Comparable per-node re-diagnosis + wall-goal bracket | `R` and 60 s coverage/factor recorded; exactly one recommendation | **done; recommends item 19** — [plan8](plan8.md), [report8](report8.md) |
| 18 | Restarts + core throughput stack | `core` proved at 10 s; `easy-win` ≤ 2 s; `deep-win` classified; no refutations | **negative; stop rule fired** — [plan10](plan10.md), [report10](report10.md) |
| 19 | Goal re-pin (owner decision) | owner accepts; `bench.md`, success condition and ledger updated together | **executed; Tier 1 1/3** — [plan9](plan9.md), [report9](report9.md) |
| 20 | Bare-tree coverage probe + product decision | Tier 1 3/3 at 120 s with 3/3 D14 proofs; censored cells reported, never converted | **executed; Tier 1 2/3** — [plan11](plan11.md), [report11](report11.md) |
| 21 | `deep-win` tree size: entry reuse | stop rule at 20 M; if started, `deep-win` 3/3 at 120 s and no >10 % regression | **stop rule fired** — [plan12](plan12.md), [report12](report12.md) |
| 22 | Cheap probes before any conformance stack | A: `deep-win` 3/3; B: cap ≥ 2× fewer unique expansions; C: `core` ≥ 2× cut | **all fail; hand-guide line closed** — [plan13](plan13.md), [report13](report13.md) |
| 23 | `deep-win` formulation decision (kill review) | owner records A, B or C in `initiative.md` and `bench.md`; no silent goal change | **executed; option A, D28** — [plan14](plan14.md) |
| 24 | **Conformance line** | Phase A: a named divergence explains the first divergent ply, or stop. B/C/D: ≥ 2× cut in `core` unique expansions vs bare at 20 M, no `easy-win` regression > 10 % | **closed negative** — Phase A passed (quiet grouping); Phase B failed G2 (no defender node at the first divergence) and G3 (`replay_exact` 1.02× `core`, no `easy-win` regression); B3/C/D not implemented — [plan16](plan16.md), [report16](report16.md), [plan17](plan17.md), [report17](report17.md) |

**Stop rules (handoff §7).** If M1 does not improve per-node cost over the legacy front-end, the per-node diagnosis is wrong: stop and re-diagnose before the M2+M3 stack. If the hand-guide stack cannot meet D21, do not split it back into mechanism plans: record the stack, its census and both-direction ablations, and take the continue/re-scope/close decision at a kill review. The conformance line (item 24) is a separate stack: its stop rule is the Phase A differential, not the hand-guide stop rule.

**Horizon (unranked, after M5 unless a row names an earlier trigger; carries `winprover` item 8):**

| # | Item | Note |
| --- | --- | --- |
| 7 | Restart schedule A/B (D5) | after the M2+M3 stack; paired with generation-aware replacement |
| 8 | Salt-seeded portfolio driver (D6) | wall-clock play, outside the core |
| 9 | Portable attacker-only export for proofdb transport | needs a JSONL → `proof_tree.bin` importer **in the legacy repo** (also the D9 optional cross-check) |
| 10 | External `high200` corpus | out-of-distribution guard, separate set with a recorded selection rule |
| 11 | Native second rules implementation | only with no shakmaty linkage (D12) |
| 12 | Durable proof corpus | own plan and schema; legacy `docs/spec/global_proof_store.md` as reference |
| 13 | Optional external tablebases (D7) | provari's path is external and optional too |
| 14 | `--guide model` stretch column (D8) | recorded only |
| 15 | Hand-guide conformance (item-3 erratum) | **closed negative** — [plan6](plan6.md), [report6](report6.md): `core` 0.96×, `easy-win` regressions |

Cannot-win material is part of the M2+M3 stack. Compact `.pvr` is M4. Root-`Loss` coverage needs a defender-side mode (a new item if ever wanted). `winprover` item 7 (workspace split) does not apply to a new repository.

## Corpus, fixtures and measurement conventions

- **Primary column:** `provari prove --guide hand`. `--guide model` is a stretch column only (D8).
- **Bench cells** (unchanged): `core` (fingerprint `8d0ff8bca7f0`), `easy-win` (`1eb80b92f92c`), `deep-win` (`db6a741c723f`). FENs and the selection rule are in [`bench.md`](bench.md) (created by plan1, copied from the frozen legacy ledger).
- **Guidance set (`guidance-v1`, D21/D22):** separate from the 3-cell bench. It is frozen by a provari-hand census (plan3) over `core + sample20` and `wins.tsv`: ≥20 deduplicated cells where provari hand wins inside the stated selection budget, in a fixed order. Its hash, census and per-cell reference nodes/walls live under `measurements/plan3/`. The bench geomean still uses only the 3 bench cells.
- **Corpus:** `bench/corpus.json`, a verbatim copy of legacy `docs/plans/provari/measurements/plan1/corpus.json`, sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`. The sha256 is its identity. The bench driver refuses to run on a mismatch. New cases form a separate, separately reported set with a recorded selection rule.
- **Budgets:** 10 s and a 192 MB label on both sides. provari: `prove "<FEN>" --seconds 10 --memory 192 --guide hand`. atomi: `prove "<FEN>" --seconds 10 --memory 192`. Three repeats per cell and side for any reported wall ratio.
- **Fixture oracle:** known side-to-move wins copied from the legacy fixtures (`decisive_positions.txt`, `decisive_remaining.txt`, `move_order_positions.txt`, the quick suite). A *refutation* there is a bug to investigate. Censoring is a result. Legacy side-to-move losses are kept as **never-prove** negatives: proving one is a soundness failure.
- **Ledger:** [`bench.md`](bench.md). Every plan that touches the search appends one `atomi` row. The legacy control rows are re-measured only when a host or drift question needs them. Rows pin `atomi@<sha>`, `solver@<sha>` and the oracle commit.
- **Artifacts:** drivers, parsed results (`*.json`, `*.csv`), `env.json` and provenance READMEs go under `docs/plans/prover/measurements/planN/`. Raw transcripts, provari sources, binaries and proofs, and `__pycache__` are never committed.

## Risks

| Risk | Mitigation |
| --- | --- |
| GPL contamination (provari, shakmaty) | D12 licence gate; scratch-only oracle; ideas only; no provari text or code copied |
| Rules divergence between `atomic-movegen` and shakmaty (e.g. shakmaty's atomic insufficient-material draws are broader than `atomic-movegen`'s commoner-only `is_material_draw`) | Insufficient material: the checker's rule is adopted in our rules layer and validator (plan1 R1). Anything else: checker veto, the `rules-divergence` class, reported upstream when real |
| The reference moves (provari v0.2.0 rewrote its search) | D17 stretch column from M0; the re-pin decision is at M5 |
| Repetition identity mismatch (en passant) | D4: `EnPassantMode::Legal` key; validator compares exact positions with legal en passant |
| Cross-clock proof reuse produces a position cycle or a 50-move failure in the written proof | Length/limit reuse rules; the writer fails closed on cycle/limit/missing; validator + checker |
| Per-node cost is not the bottleneck | M1 stop rule |
| Scope creep | Named stacks with one stated gate (D21); milestone gates |
| Measurement drift | Same-session ratios; re-measured baseline; reading rules |
| Proof memory cap on deep wins | D15 now, bounded worker in M4 |
| Copied data drifts from its source | Provenance lines + `PROVENANCE.md` with source sha256 |
| The new engine never beats provari | `atomic_solver` stays the legacy product; this repo is archived with its measured record |

## Findings

The stack, not the lever, is the unit of judgement: the provari v0.1.0 scratch ablation and atomi's reduced both-direction re-measurement are recorded in [`history.md#findings`](history.md#findings). The short version: single-lever costs understate the coupled stack (provari `core` 7.3× product vs 25× measured), and a lever's sign can invert (`EF-PN` is 1.96× worse on `core`, 0.44× on `easy-win`). No keep/remove decision follows from the failed D21 gate; the full 24-cell ablation remains unrun.

## History

The chronological record, including the plan-session decisions and the stack-ablation finding, is in [`history.md`](history.md). Details live in the report files.
