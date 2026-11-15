# Initiative: `prover` — a from-scratch, independently-checked atomic win prover that beats provari

## Status

**Current position: `T_3`; next rung `T_4` (all three cells at 10 s).** The
Cap-120 row after plan33 (D46) is `core` 4.923 s, `easy-win` 0.376 s,
`deep-win` 53.42 s, geomean **6.606**, 3/3 claimed, D14 9/9 both checkers.
`T_4` distance is `deep-win` **5.34×** (53.42 s / 10 s). The wall is read
**per node** (D48): 9,246 ns/exp against the reference's 2,111 ns/body =
**4.38×**. Plan38 measured the tree term: a bounded pass at `L = 24` cuts the
`deep-win` proof from 36 to 24 and the wall from 53.3 s to 37.5 s.

**What plans 34–38 established (2026-11-14/15).**

- **plan34 (item 42, census):** the measured per-node op budget is 68.97 pp;
  `T_4` needs ≥ 81.3 pp removed, so no arrangement of measured per-node work
  reaches 5.34× (3.22× ceiling). The stop rule fired.
- **plan35 (item 43, shape differential):** a perfect port of the reference's
  node shape is **NOT FUNDED** — `M` = 1,786 ns/exp against the 1,723 ns `T_4`
  allows (1.036×), and the ±15 % band flips the verdict.
- **plan36 (kill review + item 45):** `M` is **INCONCLUSIVE-BY-UNITS**; item 44
  is cancelled as a shape port; D47/D48 confirmed. Proof lengths are 36/37/29
  on `deep-win` against the reference's mate 19, so the old 2.76× tree ratio is
  void and length is a live axis.
- **plan37 (item 46, overhead profile):** the 28.45 pp `other` is
  **instruction-bound** (74.3 % retired instructions, 1,964 ns/exp), verdict
  **LEVER**; candidates `Frame::enter` 867 ns, `null_win` 598, `Mover::effect`
  318, `populate_state` 269, but the whole class is ≈16.5 pp ≈1.20×, not a
  route to `T_4`. Two plan defects are recorded: the census `outside` is a
  plug, and G-46d is partial; an owner ruling is pending.
- **plan38 (kill review + item 52 Phase A, bounded-length pass):** the
  opt-in `--max-length L` pass is sound and the `L = ∞` control is
  tree-identical. On `deep-win` at 120 s, `L = 20/22/24` prove with lengths
  **20/22/24** against the control's **36**; `L = 24` is **37.5 s** vs
  **53.3 s** (1.42×), while `L = 20/22` are slower (58.4/59.2 s). Every
  bounded proof is D14-accepted by both checkers. The length axis is a real
  tree lever; Phase B (iterative deepening) is the next scoped experiment.

**Consequence for priority.** The per-node micro-line is exhausted as a route
to `T_4` (plans 34–37: 3.22× ceiling, NOT FUNDED, INCONCLUSIVE, ≈1.20×). The
remaining headroom is **tree-changing**: proof length (items 47/52), guide
quality (item 48), and the measurement/gate shape that lets such levers be
judged (items 49/51/53). Plan38 gives the first positive tree row: length
36 → 24 at `L = 24`, 1.42× wall. The next session is **Phase B** (iterative
deepening `L = 12, 14, …`, opt-in) or the owner's `T_4`/`T_5` ruling; not
another per-node micro-plan.

**Open decisions:** D47/D48 confirmed; D49 default policy in force. Item 44
remains unauthorised; item 41 is blocked on items 46/50. `--max-length` is an
opt-in measurement flag, not a default and not a `bench.md` row.

## Reference

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
| Idea source (v0.4.0) | provari `bceb1ee1faf7d9cf2422eb89c931be69ae30619d` (`v0.4.0`), scratch clone read for ideas only (analysis1, D12); its search core is unchanged from v0.1.0 (threads, tablebase play-out, progress); **not** a pin, a veto or a bench column |
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

Nesting is sound because the search is deterministic (D22) and `--seconds` only stops it, so proving at 10 s implies proving at 120 s; more cells is monotone by definition. The wall clause lives only at `T_5` because `geomean < 1` does not imply each cell `< 1`. A lower rung is necessary, never sufficient; a censored cell is never converted and a longer run is never substituted. Current position: `T_3` (item 32, plan23); the next rung is `T_4`, all three at 10 s.

**The goal, the rungs and their budgets are stated here and nowhere else** (D34). [`bench.md`](bench.md) is the results ledger: it records what was measured and the cap each row used, and makes no goal claim — a longer run is its own row, and a cell is never "converted" by changing a cap. `deep-win` is the wall (`T_4` needs 10 s).

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
10. **Kill review includes the oracle delta, and "skip" rows expire.** The prior-art step re-runs the literature search **and** updates the [`analysis1`](analysis1.md) §3 mechanism table — only rows whose atomi state changed since the last review, plus any row touched by a pin change; a full re-read only when the oracle pin changes. The table is the durable asset; the delta is time-boxed. A row marked *skip for now* / *recorded only* **expires at every kill review**: the review re-argues at least one of them with a number against the current plateau. Rationale by measurement: R25 (*length-limited passes*, the reference's own `--shorten`/`--most`, off by default there) was skipped for ~30 plans while our proofs ran twice as deep as the reference's — a mechanism table of the reference's **defaults** is not a table of its **mechanisms**.
11. **A measurement names its blind spots, and the agenda is larger than the instrument.** Every census, profile or differential row states the structure it cannot see (bench.md's reading rules), and no plan may treat the previous instrument's field of view as the list of levers. Rationale by measurement: the plan34 census timed *named calls*, so the 31 pp between them (item 46) and everything outside a node (proof length, tree depth) had no line item and could not be nominated — the per-node agenda was the instrument's shape, not the problem's.

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
| D8 | Model / NNUE | Hand guide only; `--guide model` is a stretch column, never mixed in; item 48 is hand-cut code in our own words |
| D9 | Evidence | Own minimal replay validator + `provari check` veto |
| D10 | Bench ledger | Successor `bench.md` reuses the legacy protocol, cells, corpus, oracle pin and reading rules; legacy controls are a frozen baseline, not rows |
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
| D25 | Deterministic auxiliary budget | `--guide-aux-budget` bounds aux work by a deterministic counter; opt-in only (D49: no net wall win, `core` censored) |
| D26 | Per-expansion diagnosis | Judge per expansion and per node, never `child_evals` against provari nodes; methodology, not a mechanism |
| D27 | Coverage-first goal re-pin | **Superseded by D30** (tier structure); coverage-before-wall ordering retained as the staircase |
| D28 | Tier 1 amended to 2/3 | **Goal clause superseded by D30**; the `deep-win` 1/3 measurement stands as evidence |
| D29 | Conformance line and Tier 2 coverage | **Tier 2 coverage clause superseded by D30**; item 24 authorised |
| D30 | Goal staircase (necessary conditions) | `T_1`…`T_5`, each a subset of the next; `T_5` is the goal; supersedes the tier clauses of D27/D28/D29 |
| D31 | Conformance close and DF-PN exactness stack | Item 24 closed negative; PDS-PN/PN² rejected for now (**standing** since D45); item 25 authorises exact simulation + exact null-threat replay |
| D32 | Product premise after analysis1 | The bare tree is provari minus the guide; the guide's starting-number algebra is a measured 2.6–3.6× tree lever; authorises items 26–29 |
| D33 | Instrumentation traps | Guide flags must follow `--guide hand`; `--stats` unique counts are capped; fixed-`--evals` caps hide tree cuts; `aux_nodes`/expansion is the wrong denominator |
| D34 | Ledger budget convention | `bench.md` is a results ledger; goal/rungs/budgets live only in `initiative.md`; a row records its cap |
| D35 | Proof-retention share | Default proof-table share 75 % of `--memory` (was 25 %); item 32 reaches `T_3` with 9/9 D14 accepted |
| D36 | Wall decomposition and item 29 closure | GROUP-V1 closed negative; wall is a first-order finding; **tree clause and `tree × per-expansion` decomposition superseded by D48** |
| D37 | Per-node cost and NULL-V1 | Exact null replay is the first-order term; NULL-V1 adopted; NULL-S stays opt-in |
| D38 | NULL-V2 and the micro-line closure | G-N2 missed (`deep-win` 0.8631× vs ≤ 0.85×); per-node micro-line closed; "not adopted" clause superseded by D39 |
| D39 | NULL-V2 adoption by owner override | Adopted despite the G-N2 miss; 2^17 filter-only cache, tree-identical; `--null-cache-entries` is the deterministic knob |
| D40 | Hot-path representation stack | HOTPATH-V1 not adopted (0.8206× vs ≥ 1.5× required); per-node micro-line closed for good |
| D41 | Effect-based child classification | EFFECT-CHILD-V1 closed `no effect`: the effect decides 0 of 1,067,066 built children under `--guide hand` |
| D42 | MG15 declined on measurement | MG15 closed `no effect`, pre-measured at 1.147×; the child build stays the prover's terminality path |
| D43 | HOTPATH-V2 adopted | Tree-identical guide closures, 0.7223–0.7541× at 120 s; Cap-120 geomean 7.23 |
| D44 | PROOF-FILTER closed, G1 adopted | Filter is net zero to negative at every tree-identical size; exact residue G1 adopted at 0.986×; geomean 7.064 |
| D45 | Formulation fork retired | provari reaches the goal in the same base (DF-PN); the gap is implementation inside a proven-sufficient formulation; PDS-PN/PN² standing-rejected |
| D46 | Null-reply probe pipelining | NULL-PIPE adopted: `null_win` pass 1 in chunks of `K = 16`; exact, tree-identical, 0.926×; geomean 6.606 |
| D47 | Formulation ban narrowed | In-base search-order/objective changes (iterative deepening on proof length) are admissible; a new base and the goal re-pin stay standing-rejected |
| D48 | Wall decomposition corrected | Per node **4.38×** (9,246 ns/exp vs 2,111 ns/body); D36's tree clause and decomposition withdrawn; `T_4` distance 5.34× |
| D49 | Default iff a measured wall-time win | A mechanism becomes a default only on a measured wall-time reduction on the product `--guide hand` stack; otherwise opt-in |

## Backlog

Items are stable semantic identities. Plans are numbered sequentially and independently of items. Plans use stack-level gating (D21), except a single-mechanism plan that is independently meaningful. **Next kill review: due after plan37** (plans 34–35 were reviewed by [`plan36`](plan36.md)); per working agreement 10 it re-argues at least one skipped prior-art row with a number.

### Current priority (2026-11-15, after plan37)

The per-node route to `T_4` is closed by measurement (plans 34–37). The queue
below is ordered by expected distance to the goal per session, not by item
number.

1. **Kill review (due; plans 36–37).** Not an item. Rule on plan37's G-46d
   defect (the census `outside` is a plug), accept or qualify the LEVER verdict,
   and re-rank the tree-changing items below. This is the next session.
2. **Item 53 — TREE-METRIC.** The instrument for items 47/48/52: a fixed-context
   tree metric at the 5 M probe and a fixed cap, with `--guide none` as a
   standing control. Cheap, read-only, and it stops every tree question from
   costing a 120 s × seeds session.
3. **Items 47 + 52 — SHORT-WIN / SHORT-PASS.** The length axis is the only
   measured tree-changing lever with headroom: our `deep-win` proofs are
   36/37/29 against the reference's mate 19. Item 47 scopes the question; item
   52 is the bounded-length mechanism. Read item 47's Phase 0 verdict first.
4. **Item 48 — GUIDE-EVAL.** The only untested in-tree lever besides items
   44/50: does a real hand-cut term set beat the starting-number algebra's
   `child_evals` at like cost? Read-only probe first.
5. **Item 49 — BENCH-POWER.** Widen the tuning sample outside the three claim
   cells; feeds item 51 and stops ±15 % noise from deciding levers.
6. **Item 51 — TREE-GATE.** Owner ruling on a second gate shape for
   tree-changing changes (no counter-identity requirement, ≥ 1.15× bar at the
   widened sample). Depends on item 49.
7. **Item 50 — EFFECT-SHAPE.** The funding test for item 44. Run only if the
   owner still wants item 44 after the kill review; otherwise close item 44.
8. **Item 41 — upstream `movegen` ask.** Blocked on items 46/50; do not draft
   `handoff4` before then.
9. **Plan37 follow-up (item 46 candidates).** `populate_state` reuse (269 ns)
   or the `Mover::effect` reorder (318 ns) is the cheapest tree-neutral
   experiment, but the whole class is ≈1.20×. Opportunistic, not a route to
   `T_4`; it needs an item before a plan.

### Active and proposed items

| # | Item | Status / next action |
| --- | --- | --- |
| 41 | **Upstream: terminality cheaper than the build** (`movegen` handoff4) | **proposed, blocked on 46/50.** A child-terminality and lazy child-side existence query at ≤ 0.6× `clone` + `do_move` + `outcome()` + `Legal` key, plus an observable MG20 screen arm. Under D45 the hand guide reads every built child, so the ask must be a cheaper *composite*; plan33 G-40f measures the realisable ceiling. Do not draft `handoff4` before items 46/50 |
| 44 | **Node-body rewrite** (plan34 B, "v3 expansion") | **not authorised — owner decision (D45).** Keep DF-PN, allow an in-base expansion rewrite. Priced at 15.6 pp tree-neutral / ≈31.6 pp with closures (1.18× / 1.46×, not 2–4×); plan36 §3 cancels it as a shape port and keeps only its funding test (item 50). A ruling lands in `decisions.md` first |
| 46 | **The 28.45 pp no counter owns** (OVERHEAD-PROFILE) | **executed — LEVER** ([plan37](plan37.md), [report37](report37.md)). Untimed symbols are 29.01 % of cycles vs `other` 28.45 pp; class split 1,964 ns retired / 199 branch / 481 cache (74.3 / 7.5 / 18.2 %). Candidates: `Frame::enter` 867 ns, `null_win` 598, `Mover::effect` 318, `populate_state` 269. Defects: per-region subtraction gives 31.03 pp, and `outside` is a plug (true 2.328 s vs 1.384 s); G-46d partial. Owner ruling pending; follow-up candidate scoping is priority 9 |
| 47 | **Proof length** (SHORT-WIN) | **Phase 0 answered; still not scoped.** `deep-win` `length` is 36/37/29 at 120 s against the reference's mate 19, so the 2.76× tree ratio is void and the length axis is live. The unit check this row used to quote (`core` 17 = 17) is wrong: our `core` proof is 15 against mate 17, and `easy-win` is 19/19/16 against 15, so the relation is cell-dependent. Scoping is the owner's call after the kill review; item 52 is the mechanism |
| 48 | **Guide quality** (GUIDE-EVAL) | **proposed — unblocked after plans 36–37.** Does a real hand-cut term set (material, explosion targets, fuses, pressure, checkers) beat the algebra's `child_evals` at like cost? Gates: `ns`/call ≤ `hand_eval`'s 199, then 5 M `deep-win` `child_evals`, then the three cells' tree, then 120 s ≤ 0.85× with D14 9/9. Read-only probe first; D8 stands (own code, no model) |
| 49 | **Tuning resolution** (BENCH-POWER) | **proposed — unblocked after plans 36–37.** Run today's binary on `corpus-v1` cells outside the bench set (fixed 10 s, seed 0) and report the factor/tree-identity distribution: does a widened tuning bench predict the three claim cells' direction? Not item 10 (coverage); no tuning cell becomes a claim cell, nothing enters `bench.md` |
| 50 | **Per-child decide in masks** (EFFECT-SHAPE) — funding test for item 44 | **proposed — unblocked after plans 36–37.** Rewrite only the per-child screen+decide as bit tests on the move effect (no board build) and time the decide in ns/child. Funding bar: ≤ 34.3 ns/child (today 37.71 at our composite rate; 19.13 at our `status` rate). Plan29's effect decides 0 of 1,067,066 built children, so this is a *screen* rewrite, not terminality. If item 46 finds no lever, this is the last per-node question |
| 51 | **Evidence shape** (TREE-GATE) | **proposed — owner ruling, no plan of its own.** Pre-register a second gate shape for tree-changing changes: admissible when `length` does not increase and `expansions`, `child_evals`, `length` and wall are reported at the 5 M probe and at 120 s × seeds with D14 9/9 — no counter-identity requirement — plus a standing ≥ 1.15× bar at the widened sample. Depends on item 49; pairs with D47 |
| 52 | **Bounded-length pass** (SHORT-PASS) — mechanism for item 47 | **proposed — read item 47's verdict first.** A remaining-length bound per frame, raised by iterative deepening (`L` = 12, 14, …); at attacker nodes a bound of 0 is "no win within `L`". Phase A: bounded pass, proofs-only in the full-key table (bound-dependent `Refuted` values may not be stored), an unbounded `L = ∞` control that must be tree-identical, `make test-full` + veto. Phase B: deepening driver behind an opt-in flag. Gates at `L` = 20/22/24 on `deep-win`; default stays `L = ∞` until the knob wins under item 51. Outcome may be negative |
| 53 | **Tree-metric harness** (TREE-METRIC) | **proposed — the instrument for items 47/48/52.** A fixed-context metric per cell (`expansions`, `child_evals`, `length`, ns/exp at the 5 M probe and a fixed cap) with `--guide none` as a standing control. Gates: reproduces the committed values byte-for-byte on the three cells and the 5 M probe; two same-binary runs identical; ranking agrees in sign with the 120 s ranking on three historical levers (NULL-PIPE, HOTPATH-V2, G1). Never a bench cell, never a factor |

### Closed items

| # | Item | Outcome |
| --- | --- | --- |
| 1 | **M0** bootstrap + correctness + proof output + baseline | **done** — [plan1](plan1.md), [report1](report1.md) |
| 2 | **M1** per-node cost | **done** — [plan2](plan2.md), [report2](report2.md); 53.4 ms |
| 3 | **M2+M3** hand-guide stack | **closed; G1 failed** — [plan3](plan3.md), [plan4](plan4.md), [report3](report3.md), [report4](report4.md) |
| 4 | ~~M3 static tactics + simulation~~ | merged into item 3 (D21) |
| 5 | **M4** scaled proof output | **closed as superseded** — [plan15](plan15.md) |
| 6 | **M5** head-to-head | **executed; Tier 2 fail** — [plan15](plan15.md), [report15](report15.md) |
| 16 | Wall attribution + deterministic auxiliary budget | **closed negative** — [plan7](plan7.md), [report7](report7.md) |
| 17 | Comparable per-node re-diagnosis + wall-goal bracket | **done; recommends item 19** — [plan8](plan8.md), [report8](report8.md) |
| 18 | Restarts + core throughput stack | **negative; stop rule fired** — [plan10](plan10.md), [report10](report10.md) |
| 19 | Goal re-pin (owner decision) | **executed; Tier 1 1/3** — [plan9](plan9.md), [report9](report9.md) |
| 20 | Bare-tree coverage probe + product decision | **executed; Tier 1 2/3** — [plan11](plan11.md), [report11](report11.md) |
| 21 | `deep-win` tree size: entry reuse | **stop rule fired** — [plan12](plan12.md), [report12](report12.md) |
| 22 | Cheap probes before any conformance stack | **all fail; hand-guide line closed** — [plan13](plan13.md), [report13](report13.md) |
| 23 | `deep-win` formulation decision (kill review) | **executed; option A, D28** — [plan14](plan14.md) |
| 24 | **Conformance line** | **closed negative** — Phase A passed, Phase B failed G2/G3 — [plan16](plan16.md), [report16](report16.md), [plan17](plan17.md), [report17](report17.md) |
| 25 | **DF-PN exactness stack** | **closed negative for G3** — `deep-win` 1.009× (2× required); `--guide-null-exact` alone proves `core` at 18.3 M and cuts `easy-win` 32.7× — [plan18](plan18.md), [report18](report18.md) |
| 26 | **Starting-number algebra** (R1–R6) | **done** — R1+R3+R6 adopted; `core` 4.74×, `easy-win` 3.34×, `deep-win` 2.80× vs the product hand — [plan19](plan19.md), [report19](report19.md) |
| 27 | **Exact replays by default** (R18+R19) | **done** — tree cut 5.88× `core`, 4.65× `easy-win`, `deep-win` 0/3 → 3/3 at 128–175 s — [plan20](plan20.md), [report20](report20.md) |
| 28 | **Board-free closures** (R11–R13) | **done** — R11 `closure-bits` cuts aux 5–7.5× and `easy-win` tree 2.52×; R13 already in the exact null replay; R12 subsumed — [plan20](plan20.md), [report20](report20.md) |
| 29 | **Plain-move grouping by default** (R9) | **closed negative** — score-free members give `deep-win` 0/3 at 120 s vs 3/3 control; reverted, `--guide-group` stays opt-in — [plan24](plan24.md), [report24](report24.md) |
| 30 | **Memory headroom** (R17) | **triggered; resolved by item 32** — at 192 MB the 600 s `deep-win` runs found the mate but the proof table refused the export (`store-full`) — [plan21](plan21.md), [report21](report21.md), [plan22](plan22.md), [report22](report22.md) |
| 31 | **Ledger budget convention** | **done** — D34; `bench.md` is a results ledger, `check_goal.py` scans three normative docs — [plan21](plan21.md), [report21](report21.md) |
| 32 | **Proof-retention share** (RET-V1) | **done** — 75 % selected and made default (D35); all nine 120 s runs proved, D14 9/9; **`T_3` reached** — [plan23](plan23.md), [report23](report23.md) |
| 33 | **Per-node cost** (NULL-V1) | **done; NULL-V1 adopted, NULL-S opt-in** — `deep-win` 0.796×, tree identical 9/9, D14 9/9 — [plan25](plan25.md), [report25](report25.md) |
| 34 | **Per-node residual** (NULL-V2) | **G-N2 missed; adopted by owner override (D39); micro-line closed** — `deep-win` 0.8631× vs ≤ 0.85×, tree identical 9/9 — [plan26](plan26.md), [report26](report26.md), [plan27](plan27.md) |
| 35 | **Hot-path representation stack** (HOTPATH-V1) | **closed negative (G-H1 miss)** — tree-identical S1+S2+S3 is 0.8206× vs ≤ 0.667×; S4 not started — [plan28](plan28.md), [report28](report28.md) |
| 36 | **Effect-based child classification** (MG3) | **closed `no effect` (stop rule)** — the effect decides 0 of 1,067,066 built children; counters and `status_after` reverted — [plan29](plan29.md), [report29](report29.md) |
| 37 | **Board-free child terminality** (MG15) | **closed `no effect`, pre-measured** — 65.8 ns vs today's 57.5 (1.147×); no engine change, ask narrowed upstream — [plan30](plan30.md), [report30](report30.md) |
| 38 | **Hot-path v2** (HOTPATH-V2) | **closed — adopted (D43)** — tree-identical, 0.7223–0.7541× at 120 s; Cap-120 geomean 7.23 — [plan31](plan31.md), [report31](report31.md) |
| 39 | **Proof-table probe gating** (PROOF-FILTER) | **filter closed `no effect`, G1 shipped** — net zero to negative at every tree-identical size; G1 0.986×; geomean 7.064 — [plan32](plan32.md), [report32](report32.md) |
| 40 | **Null-reply probe pipelining** (NULL-PIPE) | **adopted (D46)** — `K = 16`, exact, tree-identical, `deep-win` 0.926×; geomean 6.606 — [plan33](plan33.md), [report33](report33.md) |
| 42 | **Expansion census** (EXPANSION-CENSUS) | **executed — stop rule fires** — 68.97 pp measured op budget vs ≥ 81.3 pp needed; no bucket ≥ 15 pp; no arrangement of measured per-node work reaches 5.34× — [plan34](plan34.md), [report34](report34.md) |
| 43 | **Shape differential** (SHAPE-DIFF) | **executed — NOT FUNDED** — `M` = 1,786 ns/exp vs 1,723 bar (1.036×); ±15 % band flips the verdict — [plan35](plan35.md), [report35](report35.md) |
| 45 | **`M` in like units** (M-REMAP) | **executed — INCONCLUSIVE-BY-UNITS** — A 1,785.88 / B 1,678.60 / C 2,200.66 ns against the 1,723 bar; the band straddles it. Two findings survive: the reference's per-reply legality is `heavy_legality_tests` (33.07/body vs our 34.08/exp), so the addressable pp fall 51.01 → 44.17; its child-decide loop costs our region 77.1 ns/child, which item 50 tests — [plan36](plan36.md), [report36](report36.md) |

### Stop rules

**Handoff §7.** If M1 does not improve per-node cost over the legacy front-end, the per-node diagnosis is wrong: stop and re-diagnose before the M2+M3 stack. If the hand-guide stack cannot meet D21, do not split it back into mechanism plans: record the stack, its census and both-direction ablations, and take the continue/re-scope/close decision at a kill review. The conformance line (item 24) is a separate stack: its stop rule is the Phase A differential, not the hand-guide stop rule. **Item 26 stop rule:** if the numbers stack misses its 20 M gate, the hand-guide line closes for good — no per-class re-tuning, no mechanism split, and the owner takes the goal re-pin at the next kill review. Items 26–29 are one adopt order: a plan may not run item 27–29 ahead of a failed item 26.

**Per-node line (plans 34–37).** The census stop rule fired (item 42), the shape port is NOT FUNDED (item 43), `M` is INCONCLUSIVE-BY-UNITS (item 45), and the untimed overhead is instruction-bound with a ≈1.20× ceiling (item 46). No further per-node micro-plan is authorised; the goal question goes to the owner at the kill review, not to another micro-plan.

### Horizon (unranked, after M5 unless a row names an earlier trigger; carries `winprover` item 8)

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
- **Instrumentation traps (D33):** guide flags must follow `--guide hand`, which resets the guide configuration; `--stats` unique-key counts come from a fixed 2^21 set that stops inserting at 70 % (1,468,006) and then reports a capped lower bound — a count at that value is not comparable.
- **Ledger:** [`bench.md`](bench.md). Every plan that touches the search appends one `atomi` row; the table holds `atomi` rows only. A factor is budget-free (candidate wall / the fixed provari hand wall). A row records the **cap it was measured at** (D34): `censored` means "not claimed at that cap", a longer run is its own row, and geomeans are only compared within one cap. Which rung a row reaches — and the 120 s/10 s budgets that define the rungs — is stated in the goal section above, not in the ledger. The legacy `solver@5f85133` controls are a frozen one-session plan1 baseline (D10 amendment, 2026-11-11), never ledger rows. Rows pin `atomi@<sha>` and the oracle commit.
- **Artifacts:** drivers, parsed results (`*.json`, `*.csv`), `env.json` and provenance READMEs go under `docs/plans/prover/measurements/planN/`. Raw transcripts, provari sources, binaries and proofs, and `__pycache__` are never committed.

## Risks

| Risk | Mitigation |
| --- | --- |
| GPL contamination (provari, shakmaty) | D12 licence gate; scratch-only oracle; ideas only; no provari text or code copied |
| Rules divergence between `atomic-movegen` and shakmaty (e.g. shakmaty's atomic insufficient-material draws are broader than `atomic-movegen`'s commoner-only `is_material_draw`) | Insufficient material: the checker's rule is adopted in our rules layer and validator (plan1 R1). Anything else: checker veto, the `rules-divergence` class, reported upstream when real |
| The reference moves (provari v0.2.0 rewrote its search) | D17 stretch column from M0; the re-pin decision is at M5 |
| Repetition identity mismatch (en passant) | D4: `EnPassantMode::Legal` key; validator compares exact positions with legal en passant |
| Cross-clock proof reuse produces a position cycle or a 50-move failure in the written proof | Length/limit reuse rules; the writer fails closed on cycle/limit/missing; validator + checker |
| Per-node cost is not the bottleneck | M1 stop rule; plans 34–37 closed the per-node route to `T_4` |
| Scope creep | Named stacks with one stated gate (D21); milestone gates |
| Measurement drift | Same-session ratios; re-measured baseline; reading rules |
| Proof memory cap on deep wins | D15 now, bounded worker in M4; D35 proof share |
| The numbers probe is single-seed, no repeats, no D14 | Item 26's gate: 20 M stack probes first, then the 120 s × seeds 0–2 matrix and D14 before any rung claim; censored cells are never converted |
| Guide flags silently discarded: `--guide hand` assigns a fresh `GuideConfig` | Put every guide flag after `--guide hand`; the order-independent merge is part of item 26, and pre-2026-11-11 drivers are audited for the trap (D33) |
| `--stats` unique-key counts are capped (2^21 set, stops at 70 % = 1,468,006) | Never compare a count at the cap; re-derive earlier >1.47 M "unique" figures before citing them (D33) |
| Copied data drifts from its source | Provenance lines + `PROVENANCE.md` with source sha256 |
| The new engine never beats provari | `atomic_solver` stays the legacy product; this repo is archived with its measured record |
| The goal's **wall clause** is unreachable inside the per-node line (`T_4` needs ≥ 81.3 pp; the whole measured op budget is 68.97 pp ⇒ 3.22×, report34) | plan34's stop rule; the kill-review route (D45) rather than a new item; and the pre-registered decision point at item 47 — **no length movement ⇒ the goal question returns to the owner**, not to another micro-plan |
| A lever with real headroom is unpriced because no instrument sees it (proof length sat unmeasured for ~20 plans) | bench.md's proof-length companion column; working agreement 10–11 (skipped rows expire, measurements name their blind spots); gotchas §Evidence |

## Findings

The stack, not the lever, is the unit of judgement: the provari v0.1.0 scratch ablation and atomi's reduced both-direction re-measurement are recorded in [`history.md#findings`](history.md#findings). The short version: single-lever costs understate the coupled stack (provari `core` 7.3× product vs 25× measured), and a lever's sign can invert (`EF-PN` is 1.96× worse on `core`, 0.44× on `easy-win`).

**What the 2026-11-11 differential adds.** The failed D21 gate was not a verdict on the guide stack but on atomi's *specification* of it, and the bare tree is the control that shows this: 8.65 M (provari, guide removed entirely) vs 9.27 M (atomi bare) means the unguided search is at parity, so all of the 25–27× is the guide. The reason the port yields ~1.0× is that three coupled conventions are wrong: a fresh child's deciding number is its class while the other number is 1 (atomi writes the class pair into both, inflating every OR node's `dn` sum ~9× and every AND node's per-defence `pn` 9–16×); EF-PN is banded and asymmetric from the *parent's* eval at both node kinds (atomi's is symmetric, from the child's, OR-only, 4× range against 231×); and the class predicates are cheap, exact, board-free counts (atomi's are a coarse re-invention). With those corrected by flags alone the guide goes from a net loss to 2.6–3.6× — the first >2× tree lever in 19 plans — and the auxiliary per-node cost becomes the second-order problem (aux-off `core` 17.8 s, aux-on 106 s). The probe is single-seed evidence, not a claim; item 26 is where it must become one. Details in [`analysis1.md`](analysis1.md).

**What plans 34–37 add.** The per-node gap is not a tree gap and not a shape gap: the measured op budget cannot reach `T_4` (item 42), a perfect port of the reference's shape is not funded (item 43), the unit audit is inconclusive (item 45), and the untimed overhead is instruction-bound with a ≈1.20× ceiling (item 46). The live axes are tree-changing: proof length (items 47/52), guide quality (item 48), and the gate/instrument shape (items 49/51/53).

## History

The chronological record, including the plan-session decisions and the stack-ablation finding, is in [`history.md`](history.md). Details live in the report files.
