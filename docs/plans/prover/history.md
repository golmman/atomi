# History and findings — `prover`

Chronological record and the stack-ablation finding. The initiative keeps only the current state; details live in the report files.

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

- **2026-11-10**: plan18 (item 25) executed. G2 passed: the read-only
  differential found four portable exactness gaps (exact null-threat replay,
  the exposing one-move shortcut, the guide explosion fallback and the
  top-level guide-proof override). The opt-in `--guide-sim-exact` and
  `--guide-null-exact` flags landed with tests; `--guide-null-exact` builds
  the passed position only ephemerally for the exact one-ply check and the
  guide side of the replay. G3 **failed**: the declared stack cuts
  `deep-win` unique expansions only 1.009× at 20 M (2× required), with
  `core` 1.002 and `easy-win` 0.055. The individual `null_exact` flag proves
  `core` at 18.3 M and cuts `easy-win` 32.7×, but `deep-win` is insensitive
  (0.990) and the combined stack regresses `core`. D14 5/5 accepted after
  fixing a writer `missing` bug (an exact-replay explosion must return the
  child's own proof entry). No 120 s matrix, no ledger row, no `T_3` claim.
  See [`report18`](report18.md).

- **2026-11-10**: plan session 18 (plans 16–17 kill review). The conformance
  line is **closed negative**: Phase A (plan16) named quiet grouping at the
  first `core` move divergence, but Phase B (plan17) found all nine
  move-divergent positions are attacker nodes, so exact replay cannot apply
  there; in isolation `--guide-replay-exact` cuts `core` only 1.02× at 20 M
  and is slightly larger than `hand` on all three cells. Phase C/D are
  cancelled. The kill review records the absolute distance (`T_2`; `T_3`
  needs ~2× on `deep-win`; `T_5` needs 20–36× wall plus 3/3), the proof-set
  gap (0.10–0.20 % of expansions), and the prior-art re-run. PDS-PN/PN² are
  rejected for now because they replace the DF-PN base; provari's ideas are
  DF-PN-shaped. **D31** authorises **item 25**, a DF-PN exactness stack
  (exact simulation + exact null-threat replay) gated on a ≥ 2× `deep-win`
  unique-expansion cut at 20 M / 192 MB. Wrote [`plan18`](plan18.md). No
  code, no measurements.

- **2026-11-10**: plan16/17 (item 24) executed. Phase A passed on `core`
  (quiet grouping holds and is specific); Phase B failed G2 (no defender
  node at the first move divergence) and G3 (`replay_exact` 1.02× `core`,
  no `easy-win` regression). B3/C/D were not implemented. See
  [`report16`](report16.md) and [`report17`](report17.md).

- **2026-11-10**: plan session 16. The plan15 close recommendation is
  rejected; the initiative stays open. **D29** fixes the stale Tier 2
  coverage clause (2/3, shared cells) and authorises **item 24, the
  conformance line**: Phase A read-only differential, then exact proof
  replay, exact `pn`/`dn`, and restarts, each behind a flag and gated on
  tree size at fixed evals. The diagnosis is search-tree directedness, not
  per-node cost. No code, no measurements.

- **2026-11-10**: plan15 (items 5 and 6) executed. Item 5 (M4) **closed as
  superseded** (D18 + D14 meet the scale gate; `.pvr` deferred). Item 6 (M5)
  head-to-head under D28: Tier 1 **2/3 pass** at 120 s (`core` 3/3 17.471 s /
  35.8×, `easy-win` 3/3 2.023 s / 18.9×, `deep-win` 1/3 seed 1 113.458 s;
  7/7 D14 accepted); Tier 2 **fail** at 10 s (coverage 1/3, shared-cell
  geomean 15.757; `core` ≥ 20.49×, `deep-win` ≥ 1.549× censored). Default
  stays `--guide none`; legacy/v0.2.0 controls unavailable. The stale Tier 2
  coverage clause is recorded as an open goal-definition issue. No engine
  change; `make test`, `make lint`, `make test-full` and `make veto` green.
  See [`report15.md`](report15.md).
- **2026-11-10**: plan14 (item 23) executed. The kill review found no
  non-hand mechanism with a credible 2× case (120 s `deep-win` tree: 50.0 M
  expansions / 40.7 M unique keys, 18.6 % re-expansions, `proof_full: false`,
  ~8.9× provari's 4.57 M nodes). The owner accepted **option A**: **D28**
  amends D27 so Tier 1 is **2/3** (`core`, `easy-win`) at 120 s / 192 MB;
  `deep-win` is recorded as 1/3 and is not a pass condition. The 120 s budget
  is not raised, the hand-guide/auxiliary line stays closed, no portfolio is
  built, and the bare tree is the product candidate. No ledger row, no engine
  change. See [`plan14`](plan14.md).
- **2026-11-10**: plan13 (item 22) executed. All three cheap probes fail:
  proof-share 25/40/50/60 never proves `deep-win` 3/3 (best 1/3);
  `--max-depth` 20–32 inert, cap 16 cuts 2.18× but censors at depth; the
  conformance probe cuts `core` unique expansions at most 1.22×
  (`lo_fresh`), full stack 1.11×. The hand-guide line is closed for good.
  No coverage claim, no ledger row; Tier 1 remains 2/3. 26/26 D14 proofs
  accepted. See [`report13.md`](report13.md).
- **2026-11-10**: plan12 (item 21) executed. Phase A stop rule fired: on
  `deep-win` at 20 M evals only 53 of 668,608 expansions (0.0079 %) would
  have a stored value crossing the incoming thresholds, far below the 10 %
  rule; at 120 s it is 0.0319 %. The parent frame already uses each child's
  stored value, so ENTRY-REUSE is structurally redundant. Phase B/C not
  implemented or run; no proof, no coverage claim, no ledger row. See
  [`report12.md`](report12.md).
- **2026-11-10**: plan11 (item 20) executed. The bare tree (`--guide none`)
  at 120 s / 192 MB: `core` 3/3 at 17.390 s (35.3×), `easy-win` 3/3 at
  2.008 s (19.1×), `deep-win` 1/3 (seed 1 111.218 s, seeds 0/2 censor).
  Tier 1 is 2/3, fail; the censor branch fires, so no portfolio, no budget
  raise, no auxiliary reopen. Bare dominates hand on every cell; adopting
  it is an owner decision. 7/7 D14 proofs accepted. See
  [`report11.md`](report11.md).
- **2026-11-10**: plan10 (item 18) executed. Phase A profiled the product at
  fixed 20 M child evals on `core`/`deep-win`, seeds 0–2: product wall
  38.2–41.8 s, both-off 2.53–3.21 s, auxiliary work 92.4–93.4 % of the
  wall. The stop rule fired; `RESTART-GEN` (Phase B), Phase C and Phase D
  were not implemented or run. No coverage claim, no ledger row. See
  [`report10.md`](report10.md).
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
- **2026-11-12** — plan22 (item 30) executed Phase A only. Added the
  permanent proof-table composition counters (`proof_attacker_entries`,
  `proof_defender_entries`, `proof_archive_refused`) and ran the four
  `deep-win` probes (`--guide hand`, 600 s, seed 0, one process at a time).
  The stop rule fired: the 384 MB control also refuses at the default 25 %
  proof share, with 5,639,488 live attacker entries (1.99× the 192 MB
  9/10 slot count) against 22,822 defender entries, so MEM-V1's memo tier
  cannot recover the space. No two-tier code, no default change, no gate,
  no D35, no ledger row. A 192 MB 50–75 % proof-share probe wrote proofs
  accepted by both checkers (103.4 / 114.7 s) but is one seed and not a
  claim. Gates: `make test`, `make lint`, `make test-full` + veto green.
  Data: [`measurements/plan22/`](measurements/plan22/README.md); report:
  [`report22.md`](report22.md).
- **2026-11-12** — plan23 (item 32, RET-V1) executed. Phase A swept the
  proof share 25/50/75 on seeds 0–2 at 120 s: 50 % still fails `deep-win`
  2/3 (seed 1 `proof_full`), 75 % gives 3/3 with all proofs D14-accepted and
  no `core`/`easy-win` wall regression. The default proof share moved from
  25 % to 75 % of the unchanged 192 MB label (D35). Phase C: all nine 120 s
  runs proved, 9/9 D14 proofs accepted, `core` 15.68, `easy-win` 6.84,
  `deep-win` 15.81, geomean 11.93 at Cap 120 s — **`T_3` reached**. Gates:
  `make test`, `make lint`, `make test-full` + veto green. Data:
  [`measurements/plan23/`](measurements/plan23/README.md); report:
  [`report23.md`](report23.md).
- **2026-11-12** — plan24 (item 29, GROUP-V1) executed and **closed
  negative**. Phase A added the permanent `plain_children`/`group_children`
  counters and measured the plain share: 38–47 % of `child_evals` and ~38 %
  of expansions, with `child_evals`/exp and µs/exp flat across arms. Phase B
  made plain members score-free and `group` on by default; the Phase C gate
  on `deep-win` failed hard (`0/3` at 120 s vs a `3/3` same-session control,
  median 105.032 s; `core` 1.045×, `easy-win` 0.863×), with D14 6/6 accepted
  on the six proved seeds. Phase D reverted the change (the binary is
  byte-identical to the Phase A build) and kept the counters; the group stays
  the opt-in `--guide-group` flag. `T_3` unchanged; the wall is per-expansion
  cost (D36), so item 33 is next. Gates: `make test`, `make lint`,
  `make test-full` + veto green. Data:
  [`measurements/plan24/`](measurements/plan24/README.md); report:
  [`report24.md`](report24.md).
- **2026-11-12** — plan25 (item 33, NULL-V1) executed. Phase A added the
  permanent `null_calls`/`null_replies` counters and measured the exact null
  replay as the first-order per-node term (`deep-win`: ~37 replies per call,
  `aux_nodes` ≈ 8.8–9.7 per expansion; 21.96 → 17.33 µs/exp at 20 M seed 0).
  Phase B implemented NULL-V1: the exposed-limit reply test is the board-free
  closure, one scratch board with `do_move`/`undo_move` replaces the
  clone-per-move/reply path, and `hand_eval` is computed only where it is
  read. G-V1 passed at 120 s, seeds 0–2: `deep-win` 0.796× the same-session
  control (median 83.498 s vs 104.876 s), `core` 0.784×, `easy-win` 0.766×,
  `child_evals`/`expansions` identical 9/9, D14 9/9 accepted. NULL-S
  (`--guide-null-screen`) passed `deep-win` (0.571×) but failed `easy-win`
  (3.349×), so it stays opt-in. `T_3` unchanged; `T_4` distance is
  `deep-win` 8.35×. Gates: `make test`, `make lint`, `make test-full` + veto
  green. Data: [`measurements/plan25/`](measurements/plan25/README.md);
  report: [`report25.md`](report25.md).
- **2026-11-12** — plan26 (item 34, NULL-V2) executed and **closed
  negative**. Phase A swept the exposed-limit cache at 2^17/2^18/2^20/2^21
  entries on `deep-win` at 5 M evals: 2^17 (1.25 MB) is the largest size that
  fits the filter slack, so the TT and proof table keep their sizes and the
  tree is identical to the control (30.0 % hit rate, 3.015 → 2.557 s);
  larger caches take TT bytes and change the tree. The positive-null cache
  ceiling is 11.1 % of `null_win` calls, but it changes the tree and was
  excluded. Phase B implemented EARLY-REPLY (pseudo-legal + lazy legality,
  two `StateInfo`s) and the 2^17 EXPOSED-CACHE. G-N2 at 120 s, seeds 0–2:
  `deep-win` **0.8631×** the same-session control (median 71.666 s vs
  83.029 s) against the declared ≤ 0.85×, `core` 0.8843×, `easy-win`
  0.8631×, tree identical 9/9. The wall guard failed, so D14 was not run and
  no claim is made. The source was reverted; the patch is
  [`measurements/plan26/null_v2.patch`](measurements/plan26/null_v2.patch).
  The per-node micro-line is closed; the next decision is a formulation
  choice (PDS-PN/PN² or a goal re-pin). Gates: `make test`, `make lint`,
  `make test-full` + veto green on the reverted tree. Data:
  [`measurements/plan26/`](measurements/plan26/README.md); report:
  [`report26.md`](report26.md).
