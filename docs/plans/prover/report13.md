# Prover Report 13: item 22 — the three cheap probes all fail; the hand-guide line closes

Plan: [`plan13.md`](plan13.md) (item 22, D14/D18/D21/D24/D27). Executed
2026-11-10. Measurement plus two default-off engine probes (`--max-depth`,
`--guide-fresh-dn`, `--guide-replay-exact`): no default search change, no
coverage claim, no ledger row. **Phase A** (proof-share sweep) does not prove
`deep-win` 3/3 on any share; **Phase B** (ply cap) has no passing cap;
**Phase C** (conformance stack) cuts `core` unique expansions by at most
**1.22×**, far below the 2× stop rule. The hand-guide line is closed for good.
Tier 1 remains 2/3 (report11).

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `c8e18f7` on a dirty tree (plan13 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `ba531aba…6125ad` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, sha256 `2da2a65b…c8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Configuration | `--guide none` (bare), `--memory 192`, seeds 0–2 |

Commands: the `env`, `phase_a`, `d14`, `phase_b`, `phase_c` phases in
[`measurements/plan13/README.md`](measurements/plan13/README.md), then
`make test`, `make lint`, `make test-full` (slow tier + `make veto`), one
process at a time.

## What was built (opt-in only)

- **`--max-depth N`** (default off): a ply cap that censors with
  `Exit::Depth`. `None` is the old 4096-ply cap; a test pins the default-off
  trajectory equal to the explicit 4096 cap, and a cap below the proof depth
  censors without inventing a win.
- **`--guide-fresh-dn`** (default off): a fresh child's attacker-oriented
  disproof number is 1, not the invented complementary base.
- **`--guide-replay-exact`** (default off): Kawano-style exact sibling proof
  replay in `src/guide/replay.rs`. At a defender node, a proved defence's
  proof is replayed on the other open defences; every defender reply must be
  a stored proof, an immediate win, or the same reply in the guide. It only
  returns verified proofs, which the caller stores and emits. A cycle on the
  search or replay line is never used. The replay budget is 1000 attacker
  nodes per sibling; its work is counted in `sim_nodes`/`aux_nodes`.
- Both flags are inert under `--guide none`; a test pins that trajectory.

## Phase A — proof-share sweep (stop rule A: not fired)

`--guide none --seconds 120 --memory 192`, shares 25/40/50/60, seeds 0–2.

| Share | `core` | `easy-win` | `deep-win` | deep-win censors |
| ---: | --- | --- | --- | --- |
| 25 | 3/3, 17.804 s | 3/3, 2.067 s | **1/3** | seed 0, seed 2 (120 s) |
| 40 | 3/3, 19.344 s | 3/3, 2.068 s | **0/3** | all three |
| 50 | 3/3, 19.095 s | 3/3, 2.122 s | **1/3** | seed 0, seed 1 |
| 60 | 3/3, 16.754 s | 3/3, 2.036 s | **0/3** | all three |

`deep-win` proves only on seed 1 at share 25 (116.052 s, length 25) and on
seed 2 at share 50 (114.920 s, length 27). No share is 3/3, so the stop rule
does not fire. The 10 % wall guard on `core`/`easy-win` is satisfied by every
share (worst +8.6 % on `core` at 40), but it is moot without 3/3. The
plan-session probe (seed 2 proves at share 50, seed 0 censors at 50/75) is
reproduced; the split is not a 3/3 lever.

**D14:** 26 proved Phase A rows re-ran at `--seconds 600 --proof-out` with
the same share/seed; all 26 have equal `child_evals` and are accepted by
`atomi validate` and `provari check`.

## Phase B — move-limit probe (stop rule B: no passing cap)

`--guide none --evals 20000000 --memory 192 --seed 0 --stats`, caps
16/20/24/28/32 plus bare.

| Cap | `deep-win` unique | ratio vs bare | proof | `core` unique | ratio vs bare | proof |
| ---: | ---: | ---: | --- | ---: | ---: | --- |
| bare | 620,162 | 1.00 | no | 586,075 | 1.00 | no |
| 16 | 284,884 | **2.18×** | no (depth) | 53,800 | 10.89× | no (depth) |
| 20–32 | 620,162 | 1.00 | no | 586,075 | 1.00 | no |

Caps 20–32 are inert at 20 M: the eval cap fires before the search reaches
those plies. Cap 16 does cut the tree by 2.18× on `deep-win`, but it censors
at depth and reaches no proof, so it fails the "reaches the bare proof length
(or proves)" clause. **No cap passes; the `remaining` key dimension is
dropped.**

## Phase C — conformance probe (stop rule C: line closed)

`--evals 20000000 --memory 192 --seed 0 --stats`, 11 stacks (bare, hand,
full, four leave-one-out, four add-one-in). The full stack is
`--guide hand --guide-group --guide-scale 1 --guide-ef-weight 0
--guide-fresh-dn --guide-replay-exact`.

| Stack | `core` unique | cut vs bare | `easy-win` | `deep-win` |
| --- | ---: | ---: | --- | --- |
| bare | 586,075 | 1.00 | censored | censored |
| hand | 573,195 | 1.02 | proved 468,301 | censored |
| **full** | 529,239 | **1.11** | proved 146,276 | censored |
| lo_group | 530,248 | 1.11 | proved 139,513 | censored |
| lo_efpn | 564,317 | 1.04 | proved 205,187 | censored |
| lo_fresh | **480,962** | **1.22** | proved 173,208 | censored |
| lo_replay | 529,345 | 1.11 | proved 162,116 | censored |
| add_group | 565,891 | 1.04 | proved 690,028 | censored |
| add_efpn | 522,218 | 1.12 | proved 110,293 | censored |
| add_fresh | 574,061 | 1.02 | proved 418,058 | censored |
| add_replay | 574,769 | 1.02 | proved 470,770 | censored |

The best `core` cut is **1.22×** (`lo_fresh`), not 2×. The full stack is
1.11×. The assembled divergences do cut `easy-win` hard (full 146,276 vs hand
468,301, 3.2×; `add_efpn` 110,293, matching the item-15 probe), but `core`
stays censored and the stop rule is `core`-specific. Seed 1 for the best
stack (`lo_fresh`) is recorded: `core` 481,345 unique, `easy-win` proved
130,192, `deep-win` censored. All 11 Phase C proved rows (including the seed-1
best stack) pass D14: equal `child_evals`, accepted by both checkers. **No
stack passes; the hand-guide line is closed for good.**

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 36 Phase A rows + 26 D14 rows + 12 Phase B rows + 33 Phase C rows + 3 seed-1 rows + 11 Phase C D14 rows; parsed JSON, `env.json`, one process at a time |
| G2 Phase A | **not fired**: no share proves `deep-win` 3/3; exact per-seed censors recorded |
| G3 Phase B | **pass (no cap)**: the 2× unique-expansion threshold is applied exactly; cap 16 reaches no proof |
| G4 Phase C | **pass (closed)**: the 2× `core` threshold is applied exactly; best 1.22×; line closed, not re-scoped |
| G5 soundness | **pass**: no refutation; 26/26 Phase A and 11/11 Phase C D14 proofs accepted by both checkers; `make test`, `make lint`, `make test-full` and `make veto` green; both new flags default off and trajectory-neutral when off |
| G6 docs | **pass**: this report; `initiative.md` item 22; `bench.md` history; no ledger row; no GPL dependency, raw transcript, provari artifact, proof file or `__pycache__` |

## Findings

1. **The proof-share split is not a 3/3 lever.** It moves `deep-win` between
   0/3 and 1/3 depending on the seed, but no share proves all three seeds.
   The plan-session seed-2 result is real and reproduced; it is not enough.
2. **The deep tail is not where the 20 M work is.** Caps 20–32 are inert at
   20 M; only cap 16 changes the tree, and it censors before a proof. The
   `remaining` key dimension is not justified by this probe.
3. **The conformance stack is a real `easy-win` lever, not a `core` lever.**
   The full stack cuts `easy-win` 3.2× and `add_efpn` reproduces the item-15
   110,293 count, but `core` moves only 1.11–1.22×. The five documented
   divergences do not explain the `core` gap.
4. **Exact sibling replay is sound but small.** It is implemented and
   default-off; the full stack's `core` cut is 1.11×, and removing replay
   (`lo_replay`) changes `core` by 0.02 %. It is not the missing 20×.
5. **The hand-guide line is closed on the plan's own stop rule.** This is a
   real stop, not a per-mechanism one: no further hand-guide conformance
   plan.

## Problems, open parts, missing tests

- **The 120 s `deep-win` tree is still the blocker.** No Phase A share, no
  Phase B cap and no Phase C stack proves it 3/3 at 120 s. Tier 1 remains
  2/3.
- **The replay is a probe, not a product mechanism.** It is bounded (1000
  attacker nodes per sibling), default off, and its work is counted in
  `sim_nodes`/`aux_nodes`; it was not tuned.
- **No 120 s Phase C matrix.** The stop rule fired at 20 M, so the plan did
  not build the follow-up stack or run the 120 s matrix.
- **No default-flag change.** The bare tree remains the measured product
  candidate; adopting it is still an owner decision.

## Next step (exactly one recommendation)

**Take a kill review before any further `deep-win` tree plan.** The three
cheap probes are exhausted: the split is seed-dependent, the ply cap is inert
at 20 M, and the conformance stack is a `core`-neutral `easy-win` lever. The
next plan must either (a) change the formulation (a proof-limit / graded
output, or a budget change, both owner decisions), or (b) attack the
`deep-win` tree with a mechanism not in the hand-guide family. Do not reopen
the hand-guide/auxiliary line, do not raise the 120 s budget, and do not
build a portfolio.

## Deliverables

- Data: [`measurements/plan13/`](measurements/plan13/README.md) —
  `run_plan13.py`, `env.json`, `phase_a.json`, `phase_b.json`,
  `phase_c.json`, `README.md`.
- Docs: this report; `initiative.md` item 22 status; `bench.md` history. No
  ledger row.
- Engine (opt-in only): `--max-depth`, `--guide-fresh-dn`,
  `--guide-replay-exact`; `src/guide/replay.rs`; tests pinning default-off
  trajectory neutrality. No default change.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report13.md` and
   `measurements/plan13/`. The three cheap probes failed: no proof-share is
   3/3 on `deep-win`, no ply cap passes, and the conformance stack cuts
   `core` by only 1.22×. The hand-guide line is closed. Run the kill review
   and write a plan for a `deep-win` tree mechanism outside the hand-guide
   family, or an owner decision on the formulation (proof limit / budget).
   Do not reopen the hand-guide/auxiliary line, do not raise the 120 s
   budget, and do not build a portfolio."
2. If the owner accepts 2/3 as the coverage goal, that is a D27 amendment
   and needs a new decision; this report does not authorise it.
3. If the owner wants the `easy-win` conformance gain as a product lever,
   that is a separate plan: the full stack proves `easy-win` at 146,276
   expansions (3.2× better than hand) but does not move `core`.
