# Initiative: `prover` — a from-scratch, independently-checked atomic win prover that beats provari

## Status

**Open (2026-11-06).** Item 1 (M0) is done: [`plan1.md`](plan1.md),
[`report1.md`](report1.md). Item 2 (M1) is done: [`plan2.md`](plan2.md),
[`report2.md`](report2.md) (D18–D20). Item 3 is the merged **M2+M3
"hand-guide stack"** (D21/D22); item 4 is merged into it. Its stack gate
**failed** ([`report3.md`](report3.md)): `easy-win` misses the 145,046
expansion limit and `core` is censored at 10 s / 20 M. That failure
triggered the D21 kill review; there was no split into mechanism plans.
Plan session 4 did the kill review and wrote [`plan4.md`](plan4.md): item 3 was
re-scoped to **CORE-RESET** as the last attempt (D23). Execution
([`report4.md`](report4.md), 2026-11-09) failed G1: the calibrated hand
stack proves `easy-win` at 10 s on seeds 0–2 (110,533–121,866 expansions,
under the 145,046 limit) but `core` is censored at 10 s; a per-frame
auxiliary budget (`max_sim=1, max_settle=1`) proves `core` at 120 s with a
both-checker-accepted 14,332-position proof, not at 10 s. **Item 3 is
closed**; no ledger row. The pivot options are in `report4.md` (M4 first).
Plan session 6 (2026-11-09) kept M4 but split the pivot: [`plan5`](plan5.md)
is the P0 probe (aux stack on `guidance-v1` at 10 s), [`plan6`](plan6.md) is
item 15 (`GROUP` kill-test), and the M4 implementation was drafted as plan7
(now [`planX`](planX.md), parked; see below). **Plan5 probe
executed (2026-11-09):** the auxiliary stack proves 19/24 frozen cells at
10 s (57/72 rows, all seeds; `easy-win` included), while `core`,
`deep-win`, `b9a7ed7eb368`, `3004d09cef79` and `dd3f89766a40` are censored;
`deep-win` is censored for search at 120 s, not by `proof_full`.
See [`measurements/plan5/`](measurements/plan5/README.md) and
[`report5.md`](report5.md). **Plan6 (item 15 `GROUP` kill-test) executed
(2026-11-09):** the `GROUP` child is behind the default-off `--guide-group`
flag; it does **not** cut `core` (681,518 vs 648,828 flag-off expansions at
20 M evals, 0.96×) and regresses `easy-win` (1.15–2.31×), so **G2 and the
item-15 ≥ 5× G1 clause fail**. The stop rule closes the hand-guide line; no
replay phase was implemented. The planX scale case is unchanged: on the
current tree `core` aux 120 s = 14,332 positions / 691,117 entries,
`easy-win` aux = 3,527 / 65,179, both checker-accepted. See
[`measurements/plan6/`](measurements/plan6/README.md) and
[`report6.md`](report6.md).

**Plan session 7 (2026-11-09) — kill review and re-aim.** The absolute
distance to the goal is not closing: the best product configuration is
`easy-win` at ≈ 5.8 s versus provari hand 0.137 s (**≈ 42× wall**), `core`
censored at 10 s (needs 120 s), and `deep-win` censored at 120 s; coverage
at the product budget is **1/3**. Per-node cost is competitive (M1 53.4 ms
per 1 M child evals), so the gap is auxiliary guide work and tree size, not
movegen. The D21 gate measured expansions, which let a 42× wall regression
pass. **M4 is parked as [`planX`](planX.md)** because proof export does not
move the wall geomean (D14 times runs without `--proof-out`). The next plan
is [`plan7`](plan7.md) (item 16): wall attribution, then a deterministic
global auxiliary budget under a **wall** gate. D24 records the decision;
plan7 proposes D25 for the budget's deterministic semantics.

**Plan7 (item 16) executed (2026-11-09).** The wall premise survives —
removing sibling simulation alone cuts the calibrated `easy-win` wall from a
5.829 s median to 2.096 s (64 %) — but **G3 fails**. The deterministic global
`--guide-aux-budget` and the opt-in `--guide-sim-plies` are implemented
behind flags; no default changed. Best budgeted `easy-win` is 3.7–4.2 s
(26.7–29.9× the reference wall), `core` is censored at 10 s on all seeds,
and `deep-win` is censored for search. Shortening playouts (6 → 2) gives
4.196 s; disabling simulation gives 2.073 s. Soundness holds: no refutation,
all 17 proved D14 proofs accepted by both checkers, `make test-full` and
`make lint` green. Per the plan stop rule, no third hand-guide/auxiliary
plan: the continue / re-pin / close decision is the D24 kill review. See
[`measurements/plan7/`](measurements/plan7/README.md) and
[`report7.md`](report7.md).

**Plan session 8 (2026-11-09) — kill-review outcome and re-aim.** Item 16 is
**closed (negative)**; the hand-guide/auxiliary line (items 3, 15, 16) is not
continued. The review found the distance is not auxiliary work alone: item
16's own fixed-eval runs price `easy-win` at ≈ 15.9 µs per expansion versus
provari's ≈ 1.9 µs per node, so the M1 "per-node is competitive" claim was
made in an incomparable unit (`child_evals` vs nodes). The next plan is
[`plan8`](plan8.md) (item 17): a same-session, per-expansion re-diagnosis plus
a 60 s coverage bracket, under a pre-registered decision rule (proposed D26).
It either opens item 18 (restarts + core throughput, wall-gated) or recommends
item 19 (owner re-pin). No engine change. Priority: 17 → 18 / 19 → M4 (5,
parked) → M5 (6).

**Plan8 (item 17) executed (2026-11-09).** Decision A fires: the `neither`
stack costs **2.83×** (easy-win) and **3.44×** (core) provari's ns per node at
fixed 10 M evals, so core throughput is first-order; the bare tree is
competitive per expansion (`R` 1.19/1.33) but grows the tree. Decision B:
the product still censors `core` and `deep-win` at 60 s (≥ 121.5× and
≥ 9.36×), while `easy-win` is 6.013 s (58.4×); the guide-free tree proves
`core` at 60 s in 16.6–21.0 s (34.6×, all D14 proofs accepted). The single
recommendation is **item 19 (owner re-pin)**; no engine change, no ledger
row. See [`report8.md`](report8.md) and
[`measurements/plan8/`](measurements/plan8/README.md).

**Plan session 9 (2026-11-09) — item 19 accepted, coverage first.** The
plans 7–8 kill review does not overturn plan8: the wall goal is not near-term
(`easy-win` 58.4× at 60 s, `core`/`deep-win` censored), even the bare `core`
floor is 34.6×, and further auxiliary tuning is cancelled. The owner accepts
the re-pin. **D27** makes **Tier 1 coverage** (3/3 at `--seconds 120 --memory
192`, every claim D14-accepted) the next goal and keeps the original wall
geomean as **Tier 2** on the same cells at 10 s. [`plan9`](plan9.md) measures
the coverage tier and authorises one opt-in bounded sibling-proof-replay stack
with a pre-registered stop rule if the wall is retained. `bench.md`, the
success condition, the ledger and D26's status are updated together; no
engine change yet.

**Plan10 (item 18) executed (2026-11-10) — Phase A stop rule fired.** The
owner reopened item 18 as [`plan10`](plan10.md) with Tier 1 coverage at 120 s
as Checkpoint 1. Phase A profiled the product point at fixed 20 M child evals
on `core`/`deep-win`, seeds 0–2. The product wall is 38.2–41.8 s; with
simulation and the one-ply settle disabled the same search is 2.53–3.21 s.
Auxiliary work is **92.4–93.4%** of the product wall on both cells and all
seeds, so the plan's stop rule fired: the only material lever is the closed
auxiliary line. `RESTART-GEN` (Phase B), the Phase C work item and the Phase D
checkpoints were **not implemented or run**; no coverage claim and no ledger
row. See [`report10.md`](report10.md) and
[`measurements/plan10/`](measurements/plan10/README.md).

**Plan11 (item 20) executed (2026-11-10) — bare tree is 2/3; `deep-win`
censors.** The plan's kill review rejected the portfolio as dominated and
measured the bare tree (`--guide none`) at 120 s / 192 MB, seeds 0–2. `core`
is 3/3 at 17.390 s (35.3×) and `easy-win` 3/3 at 2.008 s (19.1×), both far
better than the hand product (0/3 and 5.765 s / 54.9×). `deep-win` is **1/3**:
seed 1 proves at 111.218 s (1.399 G child evals, length 25, D14 accepted),
seeds 0 and 2 censor at 120 s. Tier 1 is **2/3 cells, fail**; the censor
branch fires, so no portfolio, no budget raise and no auxiliary reopen. All
seven D14 proofs are accepted by both checkers. The bare tree dominates hand
on every measured cell; adopting it as the product is an owner decision. The
next lever is a `deep-win` tree-size mechanism, which needs a new plan. See
[`report11.md`](report11.md) and
[`measurements/plan11/`](measurements/plan11/README.md).

The stack-level ablation premise and atomi's reduced both-direction run are
recorded in [§Findings](#findings).
This initiative is the successor to `atomic_solver`'s closed `winprover`
initiative. Its starting point is the legacy handoff
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

Success condition (from the handoff §2; amended by D27, plan9, 2026-11-09 —
the original wall-only wording is retained as Tier 2, not silently changed):

**Tier 1 — coverage (new first goal).** On the frozen shared-win set
(`core`, `easy-win`, `deep-win`), prove **3/3** with the product configuration
at `--seconds 120 --memory 192`; every claimed win passes D14 (equal
`child_evals` on a `--seconds 600 --proof-out` run, accepted by `atomi
validate` and the pinned `provari check`); no fixture-oracle regression. A
censored cell is never converted and a longer run is never substituted.

**Tier 2 — wall (after Tier 1).** On the same three cells at the canonical
10 s budget, the geometric mean of `atomi / provari --guide hand` wall time is
`< 1.0`; coverage stays 3/3.

- every claimed win is accepted by this repo's own replay validator **and** by
  the pinned `provari check` (both tiers);
- this repo's gates are green, and there are no outcome regressions on the
  copied fixture oracle (§Corpus and fixtures). The legacy solver's gates are
  not this repo's gates.
- the reference is re-pinned only via D17 at M5; the node/`child_evals` score
  option is not chosen.

Three legs, one tiered goal:

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
2. **Stack-level gating.** A plan may add a **named stack** of coupled
   mechanisms under one gate, stated up front with its constituent
   mechanisms and its gate. The gate is judged on the whole stack;
   mechanism-level adopt/reject decisions come only from **both-direction
   ablations after the stack gate**, not from single-lever runs before it.
   A single-mechanism plan remains allowed when the mechanism is
   independently meaningful. M0 (plan1) was a declared bootstrap bundle;
   M2+M3 (plan3) is the first deliberate stack (D21).
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

The handoff settled D1, D9 and D10. The 2026-11-06 plan session decided
the rest and added D12–D17; plan2 added D18–D20; the 2026-11-09 plan
session added D21–D22.

| # | Decision | Resolution |
| --- | --- | --- |
| D1 | Code location | **New repository** (handoff §6.1). `atomic_solver` is frozen as the legacy product, the `winprover` record, and an optional external cross-check |
| D2 | Board | **`atomic_movegen::Board` 2.4.x**. M0 uses plain legal movegen and make-move. M1 adds the own `Mover`/`Effect` layer on the 2.4.0 primitives (`hash_after_with`, `hash_and_rule50_after`, `is_legal`, `outcome_with`, `has_legal_move_with_state`, `blast_zone`, `attackers_to_color`). A new board only if M1 shows make/unmake cannot leave the hot path; first ask upstream for the missing primitive (legacy `movegen/feedback2.md` pattern). Any new board is differential-tested against `atomic-movegen` and, through proofs, the external checker (no shakmaty linkage, D12) |
| D3 | Proof format | **provari JSON lines first (M0)**. Header `{"root":FEN,"moves":L}`, which must have no `provari` field: that field routes the checker to its compact reader. Then one `{"fen":FEN,"move":UCI}` line per attacker (position, 50-move counter) pair. Compact `.pvr` comes in M4 (zstd + the format's move order). No `proof_tree.bin` and no native format |
| D4 | Key / rule50 / repetition | **Board-only key + proof length/limit from M0.** The key is `Board::hash_with(EnPassantMode::Legal)`; `atomic-movegen`'s hash already excludes the halfmove clock. **Correction to the handoff:** the `Legal` mode is a *correctness requirement*, not an M1 A/B. The checker treats positions as repeated using the legal-en-passant identity, so an `Always` key would let the search accept lines the checker rejects as repetitions. M1 may A/B the *cost* of computing the key (an effect-level flag), never its meaning |
| D5 | Restarts | A/B after the M2+M3 stack (D21), paired with generation-aware replacement (legacy `RestartSchedule::SHIPPED` idea). Not a default before measured. **Status:** reopened under item 18/[`plan10`](plan10.md) (2026-11-10) as part of the `RESTART-GEN` stack, behind opt-in flags. Plan10's Phase A stop rule fired (auxiliary work is 92–94% of the product wall), so `RESTART-GEN` was **not implemented**; item 18 is negative pending the owner's next decision |
| D6 | Portfolio | A separate driver outside the search core, after M5 (horizon) |
| D7 | Tablebases | None through M5. Proofs carry no tablebase verdicts, and `provari check` runs without `--tablebases`. An external optional path is a horizon item |
| D8 | Model / NNUE | Hand guide only. `--guide model` may be recorded as a stretch column and is never mixed into the primary one |
| D9 | Evidence | **Settled (handoff §4.2):** our own minimal attacker-only replay validator + `provari check` veto. The legacy validator is reachable only through a file (a horizon importer in the legacy repo) |
| D10 | Bench ledger | **Settled (handoff §2):** the legacy `bench.md` is frozen. The successor's [`bench.md`](bench.md) reuses its protocol, cells, corpus, oracle pin and reading rules, and opens with a re-measured post-2.4.0 baseline (provari hand + legacy `default`/`opt` same-session controls) |
| D11 | Per-node counters | **Both.** `child_evals`: +1 per child classified during an expansion (legacy `child_evals` semantics). `expansions`: +1 per node expansion that generates a child list (the analogue of provari's `nodes`; a diagnostic, not a ratio target). `--evals` bounds `child_evals`. The per-node cost metric is search time per 1 M child evals, from a fixed-`--evals` run |
| D12 | GPL boundary for dependencies (new) | **No GPL/LGPL/AGPL crate anywhere in the dependency graph, including dev- and build-dependencies.** shakmaty 0.30.1 is `GPL-3.0-or-later` (verified in its crate metadata), so the handoff §6's "shakmaty as dev-dependency" is overruled. The second rules implementation is reached only as the external `provari check` process. Enforced by a `cargo metadata` licence gate in `make test` |
| D13 | Names (new) | Package/binary `atomi`, initiative `docs/plans/prover/`, ledger rows `atomi@<sha>` (owner decision 2026-11-06) |
| D14 | Claimed-win protocol for the bench (new) | Timed runs write no proof, like provari's bench command. For each cell `atomi` wins, one extra untimed run with `--proof-out` (and `--seconds 600`, so the deadline cannot censor it; plan1 R6) must show identical `child_evals`, and its proof must pass the validator and `provari check`. Only then does the cell count as a win. Otherwise it is reported as censored with the verdict class |
| D15 | M0 proof store (new; writer revised by plan1 R2; **store revised by D18**) | An event-fed in-memory store, active only with `--proof-out`, with its own cap (`--proof-memory`). It keeps a non-dominated set of `(move, length, limit)` per key. The writer chooses, per **(position, counter)**, the shortest entry that holds at that counter. It checks the position graph for cycles as the checker does, and fails closed. M4 replaces the store with the bounded event-fed strategy worker (no proof-memory stop). From M1 the writer reads the proof table (D18); the event-fed store is a test-only differential source |
| D16 | Oracle gate (new; controls revised by plan1 R4) | `make veto` (our validator + `provari check` + corrupted-proof controls + a reverse control that validates provari-written JSONL) is part of `make test-full`. A missing oracle makes the gate **red**, not skipped. A control counts only as `valid: false`: the checker's `null` (small-material "cannot check") is `unchecked` and red |
| D17 | Stretch reference (new, owner decision 2026-11-06) | provari **v0.2.0** (`224ce85`, its `check.rs`/`FORMAT.md` identical to the pin, its search rewritten) is built in scratch and measured in the same session as a separately labelled `provari-v0.2.0-hand` column, with the default single thread. v0.1.0 stays the veto and the primary column of the success condition. Whether to re-pin the goal is decided at M5 from the recorded gap |
| D18 | Proof memory from M1 (plan2; revises D15) | A fixed-size **proof table** (full 64-bit keys, 16-byte slots, a non-dominated `(mv, length, limit)` set per key, never evicts, refuses inserts at 90% load) is the search's proof memory **and** the writer's source. It is always on, so collecting a proof cannot change the trajectory (D14 by construction). `--memory` covers the TT, the proof table (a quarter) and the filter; `--proof-memory` is removed. The event sink stays the search's hook for M4's bounded worker; the CLI uses `NullSink`. A full proof table fails the export closed (`store-full`) |
| D19 | Lazy terminal detection (plan2, narrowed by gate G1) | Child classification is exact for explosion, mate, material draw and the 50-move limit. A **stalemate reached by a quiet, non-checking move** is not detected at child initialization (the child starts Open `(1, 1)`; its node-entry status, computed from the movegen every node runs, refutes it) — **unless the child's side to move has only its commoner and pawns**, in which case every child is built (G1, plan2 finding: without it the pawn-ending row `rem01` went from 9.4 M evals to censored at 60 M). `child_evals` keeps its D11 meaning |
| D20 | `unsafe` in M1 (plan2) | Three sites, each with `// SAFETY:` comments and a measured keep rule: (1) the copied lazy zeroed allocator (`Zeroed<T>`, over-allocated with align 16 so calloc's lazy pages also serve the 64-byte-aligned clusters); (2) the x86-64 `_mm_prefetch` wrapper (kept: 3.3% on the final tree); (3) `madvise(MADV_HUGEPAGE)` via a 3-line `extern "C"` (Linux, x86-64/aarch64; kept: −18%) |
| D21 | Stack-level gating and the hand-guide stack (owner decision 2026-11-09; replaces working agreement 2 for this initiative) | A plan may add a **named stack** of coupled mechanisms under **one gate**. The gate is judged on the whole stack; individual mechanisms are not pre-gated. After the stack passes, **both-direction ablations** (leave-one-out from the full stack, add-one-in to the bare skeleton) decide which mechanisms stay, over the same frozen ≥20-cell set and several seeds. The first stack is **M2+M3 "hand-guide stack"** (item 3): move classes, check evasions, quiet grouping, EF-PN hand-eval scaling, per-child static closures including the one-ply explosion settle on every child for both sides, defender priors (`UNLIKELY`, defence history), history tie-breaks, sibling simulation and the null-move threat replay. **Gate:** `easy-win` `expansions` ≤ 2 × provari hand's 72,523 = **145,046 nodes**; `core` **proved** at `--seconds 10 --memory 192`; no guidance-set refutations; measured on **≥20 frozen cells and ≥3 seeds**; wall and `child_evals` are recorded, never substituted for the node gate |
| D22 | Guidance measurement and seeds (owner decision 2026-11-09) | `--seed N` perturbs **only tie-breaks after** the guide metric, class and history; seed 0 preserves movegen order. The seed is part of the deterministic flag set (same binary + FEN + flags + seed = identical counters) and is carried into D14 proof runs. The gate uses seeds 0, 1, 2, 3. `guidance-v1` is frozen by a **provari-hand census before any atomi stack run** (rule in plan3): ≥20 deduplicated cells from `core + sample20` and `wins.tsv` where provari hand proves inside the stated selection budget. Auxiliary guide work (simulation, settle movegens, threat replay) is reported as `sim_nodes`/`settle_nodes`; `child_evals` and `expansions` keep D11 meaning and stay main-search only |
| D23 | CORE-RESET and item-3 closure (plan4, executed 2026-11-09) | Item 3 was re-scoped once, as the last attempt, to **CORE-RESET**: starting-number calibration (`global_scale`, `ef_weight`), settled-proof retention only when the parent value selects it, a `--proof-share` memory split, and a per-frame `max_sim`/`max_settle` auxiliary budget, all under D21. **G1:** `core` proved at 10 s and `easy-win` ≤ 145,046 on seeds 0–2. G1 **failed** on `core`; item 3 **closed**, no mechanism split into a further plan, no ledger row. The 120 s auxiliary-budget `core` proof is recorded as the M4-first pivot. Owner to confirm the closure wording. **Erratum (post-closure investigation):** plan3 §2 implemented quiet grouping as a starting-number statement, not the one-child merge, and `simulate.rs` is a greedy playout, not proof replay, so the closure covers **atomi's variant only**; the missing group is measured at 6.9 M `core` expansions at 200 M child evals with sim/closures off. See horizon 15 |
| D24 | M4 parked; wall is the gate (plan session 7, 2026-11-09) | The drafted M4 plan is parked as [`planX`](planX.md); it resumes only after a configuration wins 3/3 at 10 s. The next plan is [`plan7`](plan7.md) (item 16). **Gates for search/guide work are wall-based** (same-session `atomi / provari --guide hand` wall, plus coverage), with `child_evals`, `expansions`, `sim_nodes` and `settle_nodes` recorded as diagnostics. A node-only gate is not sufficient evidence for the goal. Auxiliary budgets must stay **deterministic** (same binary + FEN + flags + seed = identical counters, D22); a wall-clock budget inside the search is not allowed. The kill review is due after plans 5–6 and is recorded in the status section |
| D25 | Deterministic auxiliary budget (proposed by plan7; owner to confirm) | Auxiliary work (simulation moves + settle movegens) is bounded by a search-global deterministic work counter, never by wall time. `--guide-aux-budget N` (`0` = unlimited) is part of the deterministic flag set; the search reports `aux_nodes` and `aux_skipped`. Once exhausted, optional sim/settle work is skipped; the last admitted operation may overshoot by its own work. The budget changes the tree, never the meaning. **Status:** implemented behind the opt-in flag and measured in plan7; not adopted as a default. Item 16 closed negative in the plan-session-8 kill review; the flag stays opt-in only |
| D26 | Per-expansion diagnosis and goal decision (proposed by plan8; owner to confirm) | Per-node cost is judged per **expansion** and per node, never `child_evals` against provari nodes (the M1 unit mismatch). `easy-win`/`core` are re-measured same-session at fixed 10 M evals and 10 s. `R = (ns/expansion) / (provari wall/node) > 2` makes core throughput part of item 18; `R ≤ 2` leaves item 18 as tree/restarts only. If the 60 s bracket censors `core` or `deep-win`, or their factors exceed 3×, item 19 (re-pin: coverage-first tier or node/`child_evals` score) is recommended to the owner. Re-pin is never silent. **Status:** executed in plan8 (2026-11-09): `neither` `R` = 2.83/3.44, product 60 s censors core/deep-win; the rule recommends item 19. The owner accepted in plan session 9; re-pin recorded as D27 and written as [`plan9`](plan9.md) |
| D27 | Coverage-first goal re-pin (plan9, owner decision 2026-11-09) | The single wall-geomean goal is superseded by a tiered goal. **Tier 1:** 3/3 on `core`/`easy-win`/`deep-win` at `--seconds 120 --memory 192`, every claim D14-accepted, no fixture regression; a censored cell is not converted and the budget is not raised. **Tier 2:** after Tier 1, the original same-session geomean `< 1.0` at the canonical 10 s on the same cells. The node/`child_evals` score is not chosen; the reference is re-pinned only via D17 at M5. `bench.md`, the success condition, the ledger and D26's status are updated together. Plan9 may run one opt-in bounded sibling-proof-replay stack under a pre-registered stop rule; no second replay stack and no further auxiliary tuning. **Status:** executed in plan9 (2026-11-10): Tier 1 **fails at 1/3** (`easy-win` 3/3, `core`/`deep-win` censored at 120 s); Tier 2 not claimed; `REPLAY-BOUND` not started |

## Backlog

Items are stable semantic identities. Plans are numbered sequentially and
independently of items. Plans use stack-level gating (D21), except a
single-mechanism plan that is independently meaningful.

**Priority after plan11 (2026-11-10):** the bare tree is the best measured
configuration: 2/3 cells at 120 s (`core` 3/3 17.390 s / 35.3×, `easy-win`
3/3 2.008 s / 19.1×) and it dominates the hand product on every bench cell.
`deep-win` is the only blocker (1/3 seeds; seed 1 proves at 111.218 s, seeds
0/2 censor). The portfolio is rejected as dominated, the 120 s budget is not
raised, and the hand-guide/auxiliary line stays closed. The next step is an
owner decision to adopt bare as the product configuration and a new plan for
a `deep-win` tree-size mechanism (attribute the 52 M expansions against
provari's 4.57 M nodes first). Then item 5 (M4, parked; Tier 2 or a recorded
decision is still required) → item 6 (M5). Items 3, 15, 16, 17, 18 and 19 are
closed; item 20 is executed (Tier 1 2/3).

| # | Milestone | Scope | Gate | Status |
| --- | --- | --- | --- | --- |
| 1 | **M0** bootstrap + correctness + proof output + baseline | Cargo package, small `AGENTS.md`, Makefile gates, licence gate, provenance registry, oracle build script; copied corpus and fixtures; attacker-only DF-PN with 1+ε and uniform starting numbers over plain `atomic-movegen` movegen (no `Mover`, no heuristics); CLI; event sink + proof store + JSONL writer + own validator + `make veto`; adapted bench driver; successor `bench.md` with the re-measured baseline row; per-node cost baseline | gates green (incl. veto); proves a mate-in-1, a mate-in-2 and a small win, each validator- **and** checker-accepted; never proves a fixture loss or a soundness negative; baseline row and per-node baseline recorded; **no performance claim** | **done: [`plan1`](plan1.md), [`report1`](report1.md)** |
| 2 | **M1** per-node cost | `Mover`/`Effect` per expanded node; no make/unmake for most children; incremental keys; TT clusters + prefetch + filter + lazy zeroed allocation (copy legacy `src/search/tt/alloc.rs`); in-search proof store with length/limit, inside the `--memory` label | wall per 1 M child evals ≤ the M0 baseline **and** ≤ the legacy front-end's (cross-repo, same session); a small reference win proved within a stated cap and checker-accepted; soundness fixtures green | **done: [`plan2`](plan2.md), [`report2`](report2.md)** — `core`/10 M/192 MB: **53.4** ms per 1 M child evals (m0 166.7, legacy front-end 137.3, same session) |
| 3 | **M2+M3** hand-guide stack | move classes, check evasions, quiet grouping, EF-PN hand-eval scaling; per-child static closures including the one-ply explosion settle on every child (both sides); defender priors (`UNLIKELY`, defence history); history tie-breaks; sibling simulation; null-move threat replay; constants in one `GuideParams` table exposed for an external optimizer (legacy `docs/spec/optimizer_interface.md` contract as spec) | **D21 stack gate:** `easy-win` expansions ≤ 145,046 (2 × provari hand's 72,523 nodes); `core` proved at 10 s / 192 MB; no guidance-set refutations; ≥20 frozen cells × seeds 0–3; both-direction ablations after the gate | **closed; G1 failed** — [plan3](plan3.md), [report3](report3.md), [plan4](plan4.md), [report4](report4.md). Plan3: frozen 24-cell `guidance-v1`; hand coverage 19/24; `easy-win` 211,254–631,763 expansions; `core` censored at 20 M and 10 s. Plan4 CORE-RESET (D23) last attempt: calibrated point scale 64 / EF weight 0 proves `easy-win` at 10 s on seeds 0–2 (**121,866 / 110,533 / 115,147**, all ≤ 145,046) but `core` is still censored at 10 s; auxiliary `max_sim=1/max_settle=1` proves `core` at 120 s (278.5 M evals, length 20, proof accepted by both checkers) but not at 10 s. No refutations, no ledger row. Pivot: M4 first |
| 4 | ~~M3 static tactics + simulation~~ | **merged into item 3** (D21, 2026-11-09): per-child static tactics, one-ply explosion settle, sibling simulation, defender priors, history, null-move replay | — | merged |
| 5 | **M4** scaled proof output | event-fed bounded strategy worker (no proof-memory stop), `.pvr` export | every claimed win replay-validated and checker-accepted at `deep-win` scale; the worker budget is honoured | **parked (D24)** — probe [`plan5`](plan5.md) executed ([data](measurements/plan5/README.md): 19/24 cells at 10 s, scale case `easy-win` 3,527 positions / 65,179 entries / 293 KB, `deep-win` censored for search at 120 s); implementation [`planX`](planX.md). Plan6 re-check ([data](measurements/plan6/README.md)): `core` aux 120 s = 14,332 positions / 691,117 entries, `easy-win` aux = 3,527 / 65,179; both accepted. Resumes only after a 3/3 10 s win |
| 6 | **M5** head-to-head | 3 cells × 3 repeats, same-session provari hand reference + legacy control rows; default-flag decision | geomean < 1.0, coverage 3/3, no fixture-oracle regressions; otherwise record and decide | open |
| 16 | **Wall attribution + deterministic auxiliary budget** | same-session attribution of wall on `easy-win`/`core` (main search vs sim vs settle vs threat replay vs guide eval); a deterministic global auxiliary-work budget (not wall-clock) and cheaper aux paths; wall-gated measurement | **wall gate:** `easy-win` wall ≤ 0.5 s (factor ≤ 3.6) and `core` proved at 10 s / 192 MB on seeds 0–2, with no guidance-set refutations; `deep-win` attempted at 10 s and classified; counters recorded, never substituted for wall | **executed; G3 failed** — [plan7](plan7.md), [report7](report7.md), [data](measurements/plan7/README.md). Wall premise survives (no-sim 64 % share); best budgeted `easy-win` 3.7–4.2 s (26.7–29.9×), `core` censored at 10 s, `deep-win` censored for search. `--guide-aux-budget`/`--guide-sim-plies` behind flags; no default changed. **Closed (negative)** by the plan-session-8 kill review; no further auxiliary/guide plan |
| 17 | **Comparable per-node re-diagnosis + wall-goal bracket** | `easy-win`/`core` per-expansion wall vs provari's per-node wall, same session (fixed 10 M evals and 10 s, bare/no-sim/hand); 60 s coverage/factor bracket on `core`/`deep-win`; pre-registered decision rule. No engine change | **decision gate:** `R = (ns/exp) / (provari wall/node)` and the 60 s coverage/factor recorded; exactly one recommendation, continue (item 18) or re-pin (item 19); no cell dropped, no metric changed | **done; recommends item 19** — [`plan8`](plan8.md), [`report8`](report8.md), [data](measurements/plan8/README.md). `neither` `R` = 2.828/3.444 (core throughput first-order); product 60 s: core 0/3 (≥ 121.5×), deep-win 0/3 (≥ 9.36×), easy-win 6.013 s (58.4×); bare core 60 s 3/3 at 17.1 s (34.6×), D14 accepted. No engine change |
| 18 | **Restarts + core throughput stack** | D5 restart schedule + generation-aware replacement; core per-node work only if item 17 prices it first-order; one named stack (D21) | **wall gate:** `core` proved at 10 s / 192 MB, `easy-win` ≤ 2 s interim (the 0.5 s gate was below the bare tree's 1.97 s; the goal geomean is decided at M5), `deep-win` classified at 10 s; no guidance-set refutations; ≥ 20 frozen cells × seeds 0–2 | **negative; Phase A stop rule fired** — [`plan10`](plan10.md), [`report10`](report10.md), [data](measurements/plan10/README.md). Fixed 20 M evals: product 41.781 s (`core`) / 38.560 s (`deep-win`); both-off (sim+closures) 3.160 s / 2.533 s; auxiliary share **92.44% / 93.43%**. `RESTART-GEN` not implemented, Phase C/D not run, no coverage claim, no ledger row. Recommendation: close item 18 and take the portfolio formulation decision |
| 19 | **Goal re-pin (owner decision)** | tiered goal: coverage-first (3/3 at a stated budget), then wall on shared cells; or a node/`child_evals` score; the reference is re-pinned only via D17 at M5 | owner accepts; `bench.md`, the success condition and the ledger updated together | **executed; Tier 1 failed 1/3** — [`plan9`](plan9.md), [`report9`](report9.md), [data](measurements/plan9/README.md). Product at 120 s / 192 MB: `easy-win` 3/3 (5.765 s, 54.9×, D14 accepted on all seeds), `core` 0/3 (≥ 240.0×), `deep-win` 0/3 (≥ 18.2×). Tier 2 not claimed; no ledger row. `REPLAY-BOUND` not started (Tier 1 red, wall target not retained). Recommendation: owner decision on reopening item 18 |
| 20 | **Bare-tree coverage probe + product decision** | kill review of plans 9–10; bare tree (`--guide none`) at 120 s / 192 MB on the 3 cells × seeds 0–2; same-session provari hand references; D14 for every proved cell; `guidance-v1` seed-0 diagnostic; product-configuration decision. No portfolio, no budget raise, no engine change | **coverage gate:** Tier 1 3/3 at 120 s with 3/3 D14-accepted proofs; a censored cell is reported, never converted; exactly one recommendation | **executed; Tier 1 2/3** — [`plan11`](plan11.md), [`report11`](report11.md), [data](measurements/plan11/README.md). Bare: `core` 3/3 17.390 s (35.3×), `easy-win` 3/3 2.008 s (19.1×), `deep-win` 1/3 (seed 1 111.218 s, seeds 0/2 censor at 120 s, ≥ 18.6×). 7/7 D14 proofs accepted; guidance-v1 20/24. Bare dominates hand on every cell; portfolio rejected. No ledger row. Recommendation: adopt bare (owner) and open a `deep-win` tree-size plan |

**Stop rules (handoff §7).** If M1 does not improve per-node cost over the
legacy front-end, the per-node diagnosis is wrong: stop and re-diagnose
before the M2+M3 stack. If the hand-guide stack cannot meet D21, do not split
it back into mechanism plans: record the stack, its census and both-direction
ablations, and take the continue/re-scope/close decision at a kill review.

**Horizon (unranked, after M5 unless a row names an earlier trigger; carries `winprover` item 8):**

| # | Item | Note |
| --- | --- | --- |
| 7 | Restart schedule A/B (D5) | after the M2+M3 stack; paired with generation-aware replacement |
| 8 | Salt-seeded portfolio driver (D6) | wall-clock play, outside the core |
| 9 | Portable attacker-only export for proofdb transport | needs a JSONL → `proof_tree.bin` importer **in the legacy repo** (also the D9 optional cross-check) |
| 10 | External `high200` corpus | out-of-distribution guard, added as a separate set with a recorded selection rule |
| 11 | Native second rules implementation | only with no shakmaty linkage (D12) |
| 12 | Durable proof corpus | own plan and schema; legacy `docs/spec/global_proof_store.md` as reference |
| 13 | Optional external tablebases (D7) | provari's path is external and optional too |
| 14 | `--guide model` stretch column (D8) | recorded only |
| 15 | Hand-guide conformance (item-3 erratum) | **Not** an item-3 restart: attacker plain-move group as **one** child (`GROUP`; `pn=9`, `dn=#plain`), plus event-driven sibling-proof replay, behind one flag. Gate: `core` < 1 M expansions at 20 M child evals (≥5× cut) and no `easy-win` regression; otherwise close the hand-guide line. **Trigger:** before M4 freezes the proof format, or at an M4 stall. No optimizer/ablation sweep. **Closed (2026-11-09) — negative.** [`plan6`](plan6.md), [`report6`](report6.md): `core` 681,518 vs 648,828 flag-off expansions (0.96×, no 5× cut); `easy-win` 1.15–2.31× regressions. The hand-guide line is closed; no replay phase. |

**Item 15 evidence (investigation only, no code).** A conformance read of
provari's hand guide against the measured plan3 stack found five semantic
divergences: quiet moves are one deferred `GROUP` child, not a per-child
starting number; sibling simulation and null-threat replay are exact proof
replays, not playouts/priors; EF-PN scales by the parent's 0/1/2 band, not the
child's continuous eval; fresh disproof numbers are 1, not invented
complementary bases; a check's proof number is its surviving evasions capped
at 8, not the parent's legal-move count. Same-session probes on the current
tree: `easy-win` 190,464 expansions (censored) → 166,847 with `dn=1`
(proved) → 110,339 with `dn=1` and EF-PN off (proved); `core` 149,504 →
143,360 (censored); `core` with `--guide-off sim,closures` still needs
>6.9 M expansions at 200 M child evals, against provari's 342,289 nodes. The
blocker is the tree, not only per-node cost. Cheapest decisive test: the
`GROUP` child alone behind one flag, `core` and `easy-win` at 20 M child
evals; a <5× `core` cut kills the hand-guide line.

Cannot-win material is part of the M2+M3 stack. Compact `.pvr` is M4. Root-`Loss`
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
| provari | move-class numbers (`THREAT=4`, `QUIET=9`, `HANGING=15`, `WEAK_THREAT=8`, `EVASIONS=8`), EF-PN scaling, quiet grouping, 1+ε (`second + 1 + second/8`) | idea | M0 (1+ε only), M2+M3 (D21) |
| provari | per-child static tactics; one-ply explosion settle on every child; simulation (`SIMULATION=16`) | idea | M2+M3 (D21) |
| provari | null-move threat replay; defender priors (`UNLIKELY`, defence history); history tie-breaks | idea | M2+M3 (D21) |
| provari | JSONL / `.pvr` proof format (FORMAT.md as interoperability spec; text not copied) | idea | M0 (JSONL), M4 (`.pvr`) |
| provari | `provari check` | external (veto) | M0 |
| provari | "the checker uses nothing from the prover" independence test | idea | M0 (our validator) |
| legacy | `ProofEvent` neutral event protocol; bounded worker; memory-budget contract (`src/proof_event.rs`, `src/proof_tree/worker.rs`) | spec | M0 (event type + sink), M4 (worker) |
| legacy | replay validator rules (`src/proof_tree/validate.rs`), JSONL export shape (`src/proof_tree/export.rs`) | spec | M0 |
| legacy | lazy zeroed TT allocation (`src/search/tt/alloc.rs`) and its liveness invariant | copy | M1 |
| legacy | restart schedule + generation-aware victim score | idea | horizon 7 |
| legacy | portfolio driver (`examples/portfolio`) | idea | horizon 8 |
| legacy | bench driver (`docs/plans/winprover/measurements/bench/run_bench.py`) | copy + adapt | M0 |
| legacy | optimizer contract (`docs/spec/optimizer_interface.md`) | spec | M2+M3 (D21) |
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
  discards DF-PN ordering feedback. The hand-guide stack uses the per-child
  form instead;
- EF-PN on both node kinds: rejected in the general solver (per-case floor).
  The stack measures it against the attacking-branch-only form (D21);
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
- **Guidance set (`guidance-v1`, D21/D22):** separate from the 3-cell bench.
  It is frozen by a provari-hand census (plan3) over `core + sample20` and
  `wins.tsv`: ≥20 deduplicated cells where provari hand wins inside the
  stated selection budget, in a fixed order. Its hash, census and per-cell
  reference nodes/walls live under `measurements/plan3/`. The bench geomean
  still uses only the 3 bench cells.
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
| Scope creep | Named stacks with one stated gate (D21); milestone gates |
| Measurement drift | Same-session ratios; re-measured baseline; reading rules |
| Proof memory cap on deep wins | D15 now, bounded worker in M4 |
| Copied data drifts from its source | Provenance lines + `PROVENANCE.md` with source sha256 |
| The new engine never beats provari | `atomic_solver` stays the legacy product; this repo is archived with its measured record |

## Findings

### The stack, not the lever, is the unit of judgement (2026-11-09)

This is the retro todo-5 finding. It records the ablation premise and atomi's
own re-measurement; it does not adopt or remove any mechanism (the D21 gate
failed, so the D21 stop rule applies).

**Premise — provari v0.1.0 scratch ablation** (pin `48686bc`, single runs,
`--guide hand --seconds 30 --memory 192`; node counts relative to the full
stack; the `nodes` counter may exclude simulation visits):

| Removed from provari | `core` | `easy-win` | `deep-win` |
| --- | ---: | ---: | ---: |
| nothing (full) | 342k | 73k | 4.58M |
| null-move threat replay | 1.37× | 1.03× | 1.18× |
| sibling simulation | 0.91× | 0.78× | 0.88× |
| `UNLIKELY` defence priors | 1.05× | 1.01× | 1.13× |
| history tie-break | 1.40× | 0.86× | 0.95× |
| check = number of evasions | 0.89× | 0.79× | 0.87× |
| per-child static closures | 0.99× | 1.48× | 1.04× |
| move-class numbers (flat) | 2.30× | 4.29× | 2.21× |
| guide (EF-PN) | 1.96× | 0.44× | 1.27× |
| **all of the above** | **25×** (8.65M) | **10.9×** (788k) | censored at 30 s |

The product of the single-lever costs on `core` is 7.3×, against the measured
25× for the whole stack. The levers are coupled; judging them one at a time
understates the stack and can invert a lever's sign (`EF-PN` is 1.96× worse
on `core` and 0.44× on `easy-win`).

**atomi re-measurement — reduced, both directions**
([`measurements/plan3/ablation.json`](measurements/plan3/ablation.json)): the
three bench cells, `--guide hand`, seed 0, `--seconds 10 --evals 10000000
--memory 192`, 3 repeats. `expansions` are deterministic; apart from the one
proved `easy-win` run, every cell below is censored at 10 s, so those walls
are lower bounds only. `lo_X` removes `X` from the full stack; `ai_X` adds
`X` to `--guide none`.

| Stack (seed 0) | `core` | `easy-win` | `deep-win` |
| --- | ---: | ---: | ---: |
| full | 148,480 | 185,344 | 174,080 |
| `lo_efpn` | 144,384 | **110,293 (proved)** | 175,104 |
| `lo_classes` | 137,216 | 160,768 | 174,080 |
| `lo_evasions` | 146,432 | 196,608 | 173,056 |
| `lo_quiet` | 160,768 | 223,232 | 194,560 |
| `lo_priors` | 156,672 | 174,080 | 178,176 |
| `lo_history` | 148,480 | 199,680 | 175,104 |
| `lo_replay` | 157,696 | 172,032 | 178,176 |
| `lo_closures` | 323,559 | 316,559 | 341,774 |
| `lo_sim` | 311,735 | 386,992 | 328,390 |
| best `ai_*` (`evasions`) | 298,900 | 313,561 | 330,089 |
| bare skeleton (report3) | 312,551 | 312,551 | not separately run |

Reading (evidence only, per D21):

- `EF-PN` is the `easy-win` cost: removing it proves `easy-win` in 110,293
expansions, under the 145,046 gate, while the full stack is 1.3–4.4× above
the gate. The reverse holds on `core` at this horizon, matching provari's 0.44× /
1.96× split.
- `closures` and `sim` are the `easy-win` tree levers (removing either pushes
the censored count up), but they dominate per-node cost and `core`/`deep-win`
stay censored on every stack.
- No `ai_*` stack proves a gate cell or comes near the full stack; only
`ai_evasions` reduces `core`'s censored count (298,900 vs bare 312,551).
Adding one mechanism to the bare skeleton needs the rest of the coupled
stack, exactly the retro's point.
- No keep/remove decision follows from a failed gate. The plan3-specified
full ablation (24 cells × seeds 0–2, `--seconds 600 --evals 10000000`) is the
conditional G7 step and remains **unrun**; this reduced run and the provari
premise are the recorded starting point for the D21 kill review.

## History

- **2026-11-10**: plan session 10. The owner takes report9's recommendation
  and reopens **item 18** (restarts + core throughput) as
  [`plan10`](plan10.md). The plan stages Tier 1 coverage at 120 s as
  Checkpoint 1 and item 18's existing 10 s wall gate as Checkpoint 2, under
  the named `RESTART-GEN` stack (D21): a deterministic restart schedule,
  generation-aware TT replacement, and one Phase-A-selected per-expansion
  work item. `REPLAY-BOUND` stays unstarted; the hand-guide/auxiliary line
  stays closed; the 120 s budget is not raised. No code, no measurements, no
  ledger row.
- **2026-11-10**: plan9 (item 19) executed. Tier 1 coverage at 120 s / 192 MB:
  `easy-win` 3/3 (5.765 s median, 54.9×, D14 accepted on seeds 0–2),
  `core` 0/3 (≥ 240.0×) and `deep-win` 0/3 (≥ 18.2×) censored. Coverage is
  **1/3**, so Tier 1 fails; Tier 2 is not claimed and no ledger row is
  appended. `REPLAY-BOUND` was not started (Tier 1 red, wall target not
  retained). `make test` and `make lint` green. Data:
  [`measurements/plan9/`](measurements/plan9/README.md); report:
  [`report9.md`](report9.md). Recommendation: owner decision on reopening
  item 18 (restarts + core throughput) with Tier 1 at 120 s as the first
  checkpoint.
- **2026-11-09**: plan session 9 (plans 7–8 kill review). The review confirmed
  the absolute distance (easy-win 58.4× at 60 s, core ≥ 121.5× and deep-win
  ≥ 9.36× censored, bare core floor 34.6×, coverage 1/3) and cancelled further
  hand-guide/auxiliary tuning. The owner accepted item 19: **D27** re-pins the
  goal to Tier 1 coverage (3/3 at 120 s, D14-accepted) and retains the wall
  geomean as Tier 2. Wrote [`plan9`](plan9.md): coverage baseline plus **one**
  opt-in bounded sibling-proof-replay stack with a pre-registered stop rule.
  Updated `bench.md`, the success condition, the ledger and D26 status
  together. No code, no measurements, no ledger row.
- **2026-11-09**: plan8 (item 17) executed. Phase A: same-session provari
  repeats (easy-win 106 ms / 72,523 nodes; core 499 ms / 342,289) and four
  stacks × easy-win/core × seeds 0–2 × {fixed 10 M, 10 s}. `R` (fixed): bare
  1.19/1.33, `neither` **2.83/3.44**, no-sim 10.8/14.2, hand 34.1/45.5, so
  Decision A prices core throughput as first-order. Phase B (60 s): product
  core 0/3 (≥ 121.5×) and deep-win 0/3 (≥ 9.36×), easy-win 6.013 s (58.4×);
  bare core 3/3 at 17.1 s (34.6×), D14 accepted by both checkers. Decision B
  recommends **item 19 (re-pin)**. No engine change; `make test` and
  `make lint` green. Data: [`measurements/plan8/`](measurements/plan8/README.md);
  report: [`report8.md`](report8.md).
- **2026-11-09**: plan session 8 (post-plan7 kill review). Closed item 16
  (negative) and the hand-guide/auxiliary line. Recorded that the M1
  per-node claim used `child_evals` against provari nodes; `easy-win` is
  ≈ 15.9 µs per expansion vs provari ≈ 1.9 µs per node in plan7's own
  fixed-eval rows. Wrote [`plan8`](plan8.md) (item 17): same-session
  per-expansion re-diagnosis plus a 60 s coverage bracket under proposed
  D26. Opened item 18 (restarts + core throughput, conditional) and item 19
  (owner re-pin). No code, no measurements.
- **2026-11-09**: plan7 (item 16) executed. Phase A attribution (8 stacks,
  easy-win/core seeds 0–2, deep-win seed 0, fixed 10 M evals and 10 s):
  simulation is 64 % of the calibrated easy-win wall; the seed-0 decision
  proxy is 0.275 but the three-seed median is 0.330. Phase B added the
  deterministic global `--guide-aux-budget` (`aux_nodes`/`aux_skipped`) and
  opt-in `--guide-sim-plies`; no default changed. The budget grid lowers
  easy-win to ≈ 3.8–3.9 s but grows the tree ~8×; sim 2 plies gives 4.196 s,
  no-sim 2.073 s. Phase C: chosen 100 k budget, easy-win 3.711–4.162 s
  (26.7–29.9×), core censored at 10 s, deep-win censored for search. **G3
  fails**; G1/G2/G4/G5 pass (0 refutations, 17 D14 proofs accepted by both
  checkers). No ledger row; D25 recorded as proposed. Data:
  [`measurements/plan7/`](measurements/plan7/README.md); report:
  [`report7.md`](report7.md).
- **2026-11-09**: plan session 7 (kill review). Parked the M4 plan as
  [`planX`](planX.md) and added D24: M4 resumes only after a 3/3 10 s win;
  search/guide gates are wall-based; auxiliary budgets stay deterministic.
  Wrote [`plan7`](plan7.md) (item 16): wall attribution, then a deterministic
  global auxiliary budget under a wall gate. No code, no measurements.
- **2026-11-09**: plan6 (item 15 `GROUP` kill-test) executed. `--guide-group`
  (default off) implements the plain-move `GROUP` pseudo-child, value
  algebra, descent and proof mapping with unit/property/crafted tests.
  `core` at 20 M evals: 681,518 vs 648,828 flag-off expansions on seed 0
  (0.96×, censored both ways); `easy-win` regresses 1.15–2.31× and lengthens
  (18/18/17 → 28/28/21). No refutations on the seed-0 guidance sweep; all 17
  proved group proofs accepted by both checkers. Scale case re-checked:
  `core` aux 120 s 14,332 positions / 691,117 entries, `easy-win` aux 3,527 /
  65,179, both accepted. G2 and the substantive G1 fail → hand-guide line
  **closed** by the stop rule; no replay phase. Data:
  [`measurements/plan6/`](measurements/plan6/README.md); report:
  [`report6.md`](report6.md).
- **2026-11-09**: plan5 (P0 probe) executed. The auxiliary stack on the
  frozen `guidance-v1` set at 10 s proves 19/24 cells on seeds 0–2 (57/72
  rows); `easy-win` reproduces the plan4 inflation exactly (918,551 /
  759,177 / 773,238 expansions) and its D14 proof (3,527 positions,
  65,179 entries, 293,459 bytes) is accepted by both checkers. `core` and
  `deep-win` are censored; `deep-win` at 120 s is a deadline censor, not
  `proof_full`. `--proof-out` is neutral on proved runs; censored 10 s
  counters vary only with the work completed at the wall deadline.
  Measurement only: no ledger row, no code change. Data:
  [`measurements/plan5/`](measurements/plan5/README.md); report:
  [`report5.md`](report5.md).
- **2026-11-09**: plan session 6 relabelled the drafted M4 plan as plan7
  (later parked as [`planX`](planX.md)); P0 is now [`plan5`](plan5.md) and
  item 15 is [`plan6`](plan6.md). Order: plan5 (probe) → plan6 (GROUP
  kill-test) → plan7 (M4, `.pvr` freeze provisional until plan6 closes).
  No code, no measurements.
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
- **2026-11-06** — plan1 (M0) executed: all gates green incl. `make veto`.
  The false-refutation bug (Open numbers saturating at `INFINITY` instead
  of `OPEN_MAX`) was found and fixed; the 10 M fixture sweep is 0 refuted.
  Bench baseline: legacy `default` 45.358 (1 cell), legacy `opt` 6.322
  (2 cells), atomi M0 36.365 (1 cell, no performance claim); stretch
  provari v0.2.0/v0.1.0 hand = 1.001 with identical single-thread node
  counts (342,289 / 72,523 / 4,574,912). M1 reference: **162.0 ms per
  1 M child evals** on `core` at `--evals 10000000`.
- **2026-11-06** — plan2 (M1) executed in phases A–D, all gates green
  (G1–G8). Per-node on `core`/10 M/192 MB: m0 166.7 → m1A (memory half)
  77.3 → m1B (compute half) 63.8 → m1 (+ huge pages) **53.4** ms per 1 M
  child evals; legacy front-end 137.3. D18–D20 adopted; D19 narrowed by
  the stalemate gate G1 after `rem01` regressed under fully lazy
  stalemates. Found and fixed: std's `alloc_zeroed` memsets over-aligned
  layouts, so the TT was not lazily allocated. 10 M sweep: 41 / 11 / 0,
  all 41 proofs accepted by both checkers. Bench: atomi m1 `easy-win`
  10.897 (1 cell, no goal claim).
- **2026-11-09** — owner decision in plan session 3 (D21/D22): replaced
  "one mechanism per plan" with **stack-level gating**; merged M2+M3 into
  the **hand-guide stack** (item 3; item 4 merged) with the D21 gate
  (`easy-win` ≤ 145,046 expansions, `core` proved at 10 s, ≥20 frozen
  cells × seeds 0–3); added the null-move threat replay, defender
  priors/history, history tie-breaks and the per-child one-ply explosion
  settle to the take-list. Wrote `plan3.md`. No code changed.
- **2026-11-09** — plan3 executed. The nine mechanisms, `--seed` and the
  ablation flags landed; `--guide none --seed 0` stayed counter-identical
  to m1 (with a measured +8.2 % per-node plumbing cost). The 73-candidate
  provari census froze `guidance-v1` (24 cells, sha256 `2da2a65b…8c7d`).
  D21 **failed**: `easy-win` 211,254–631,763 expansions (limit 145,046),
  `core` censored at 20 M child evals and at 10 s, coverage 19/24 vs
  provari 24/24. No refutations; `make test-full` and the veto passed; one
  easy-win D14 proof was accepted by both checkers. The D21 stop rule is
  active: record, kill-review, do not split into mechanism plans. No ledger
  row was appended.
- **2026-11-09** — retro follow-up (todos 5–8): recorded the stack-level
  ablation finding above (provari premise + atomi's reduced both-direction
  run, extended to `deep-win`); put the absolute distance at the top of
  `bench.md`; added the plan/AGENTS size caps and the kill-review prompt to
  `AGENTS.md`; trimmed `AGENTS.md` to the cap. No engine code changed.
- **2026-11-09** — plan4 (CORE-RESET) executed; D23 G1 **failed** on
  `core` at 10 s, so **item 3 is closed**. Calibration (25-point
  `scale × ef_weight` sweep) proved `easy-win` at 10 s on seeds 0–2
  (121,866 / 110,533 / 115,147 expansions, limit 145,046) but left `core`
  at 1.61 M expansions / 50 M evals. Proof-table pressure, `--proof-share`
  and a per-frame auxiliary budget landed with tests. The auxiliary budget
  (`max_sim=1, max_settle=1`) is the first configuration to prove `core`
  (120 s, 278.5 M evals, length 20); both checkers accepted the 14,332
  position proof. No ledger row. Recorded M4-first as the pivot.
- **2026-11-09** — plan session 4: D21 kill review plus new read-only probes
  (bare `core` proves at 268 M evals / 9.27 M expansions / 16.8 s; hand
  `core` times out at 180 s after 2.92 M expansions and fills the proof table
  at 2.83 M entries). Wrote [`plan4.md`](plan4.md): item 3 re-scoped to
  **CORE-RESET** (calibration of starting numbers, settle-proof retention,
  memory split, auxiliary budget), proposed D23, G1 = `core` proved at 10 s
  and `easy-win` ≤ 145,046; if G1 fails the item closes. No code changed.
