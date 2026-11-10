# Prover Report 9: item 19 — Tier 1 coverage fails at 1/3; no replay stack started

Plan: [`plan9.md`](plan9.md) (item 19, D27 coverage-first re-pin). Executed
2026-11-10. Measurement only: no engine, test or default change. **Tier 1
fails**: the product proves `easy-win` 3/3 at 120 s but censors `core` and
`deep-win` on all seeds. Coverage is **1/3**, not 3/3. `REPLAY-BOUND` was not
started: Tier 1 did not pass and the owner did not retain the wall target in
this session. No ledger row and no Tier 2 claim follow.

## D27 wording (the goal this report measures)

**Tier 1, coverage:** on the frozen shared-win set (`core`, `easy-win`,
`deep-win`), prove **3/3** with the product configuration at
`--seconds 120 --memory 192`; every claimed win follows D14 (identical
`child_evals` on a `--seconds 600 --proof-out` run, accepted by
`atomi validate` and `provari check`); no fixture-oracle regression. A
censored cell is never converted, and a longer run is never substituted.

**Tier 2, wall:** only after Tier 1 passes, the original condition applies on
the same three cells at the canonical 10 s budget: geometric mean of
`atomi / provari --guide hand` wall `< 1.0`. It is retained, not deleted.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `38c9de6` on a dirty tree (plan9 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `8dec6b8f…9d909` (rebuilt from the same source) |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, sha256 `2da2a65b…8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Product point | `--guide hand --guide-scale 64 --guide-ef-weight 0` (plan4) |

Commands: the four `run_plan9.py` phases in
[`measurements/plan9/README.md`](measurements/plan9/README.md), then
`make test` and `make lint`, one process at a time. Coverage ran first, then
the same-session provari block, then D14.

## Phase A — Tier 1 coverage at 120 s

Product point, `--seconds 120 --memory 192`, seeds 0–2. Medians over the three
seeds; `easy-win` is 3/3, so its median is a completed wall.

| Cell | proved | wall | expansions | child_evals | aux | length | factor |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| core | **0/3** | 120.019 s | 1,956,864 | 60,152,802 | 414,214,167 | — | **≥ 240.0×** |
| easy-win | **3/3** | 5.765 s | 115,147 | 3,147,781 | 21,700,695 | 18 / 18 / 16 | 54.9× |
| deep-win | **0/3** | 120.026 s | 2,030,592 | 61,220,707 | 429,004,425 | — | **≥ 18.2×** |

Same-session provari hand medians: core 500 ms / 342,289 nodes; easy-win
105 ms / 72,523 nodes; deep-win 6,590 ms / 4,574,912 nodes.

**Tier 1 result: 1/3, fail.** `core` and `deep-win` are search censors
(`exit=timeout`, no `proof_full`, no refutation). The censored cells are not
converted and the 120 s budget is not raised. The easy-win wall is stable
against plan8 (5.765 s vs 6.013 s); the provari references are also stable
(105/500/6,590 ms vs 103/494/6,413 ms). The core/deep-win lower bounds are
larger than plan8's 60 s bounds because the run is twice as long, not because
the tree changed.

## D14 evidence

Every proved coverage row was re-run with `--seconds 600 --proof-out`, same
flags and seed. All three have equal `child_evals` and both checkers accept.

| Cell | seed | timed child_evals | proof child_evals | equal | positions | atomi | provari |
| --- | ---: | ---: | ---: | --- | ---: | --- | --- |
| easy-win | 0 | 3,321,312 | 3,321,312 | yes | 2,493 | valid | valid |
| easy-win | 1 | 2,993,310 | 2,993,310 | yes | 2,471 | valid | valid |
| easy-win | 2 | 3,147,781 | 3,147,781 | yes | 2,371 | valid | valid |

No proof file is committed; the driver deleted every temporary proof.

## Phase B — `REPLAY-BOUND` not started

The plan authorises one opt-in bounded sibling-proof-replay stack only after
Tier 1 is recorded and only if the owner retains the wall target. Neither
condition holds: Tier 1 is 1/3, and the owner did not retain the wall target
in this session. The pre-registered stop rule is therefore unexercised:

1. median `core` expansions fall by ≥ 2×; and
2. median `easy-win` expansions do not rise by more than 10%; and
3. no refutation on `guidance-v1` seed 0; and
4. any claimed proof is D14-accepted by both checkers.

No replay flag, code, test or `replay.json` was added. The hand-guide line
stays closed; no auxiliary tuning was attempted.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 9 coverage + 9 provari + 3 D14 rows; parsed JSON, `env.json`, one process at a time |
| G2 determinism | **pass**: no engine change; D14 equal `child_evals` on all proved rows |
| G3 goal | **fail**: Tier 1 is 1/3 at 120 s; exact censored cells recorded; no lower coverage called a pass and no budget raised |
| G4 soundness | **pass**: no refutation or never-prove; 3/3 D14 proofs accepted by both checkers; `make test` and `make lint` green. `make test-full`/`make veto` not required (no search/table/proof change) |
| G5 docs | **pass**: no GPL dependency, raw transcript, provari artifact, proof file or `__pycache__` |

## Findings

1. **The 120 s budget does not close the coverage gap.** `core` and
   `deep-win` are still search censors on every seed. The product's tree is
   the blocker, not the deadline alone: at 120 s `core` reaches ~1.96 M
   expansions and `deep-win` ~2.03 M, while provari hand needs 0.34 M and
   4.57 M nodes respectively. `deep-win` is closer in factor (≥ 18.2×) but
   still censored.
2. **`easy-win` is the only Tier 1 cell and it is stable.** 3/3 at 5.5–6.1 s,
   D14 accepted on all seeds. Its factor is 54.9×, so Tier 2 is not near even
   on the one shared cell.
3. **The re-pin did not hide a wall regression.** The same-session provari
   references and the easy-win wall match plan8 within normal variation. The
   failure is coverage, not a changed trajectory.
4. **No replay evidence exists.** The one authorised wall/tree attempt was
   not run, so the report makes no claim about sibling-proof replay.

## Problems, open parts, missing tests

- **Tier 1 is red.** The plan forbids raising the 120 s budget or converting a
  censor; the only remaining path to 3/3 is a search/tree change.
- **`REPLAY-BOUND` is unmeasured.** Its stop rule, tests and flag remain
  unimplemented. Starting it now would violate the plan's entry condition.
- **No ledger row.** A failed Tier 1 is not a performance claim; the ledger
  stays at the plan8 state.
- **`make test-full`/`make veto` were not run** because no search, table,
  proof-output or validator code changed. D14 already exercised both checkers
  on the three claimed proofs.

## Next step (exactly one recommendation)

**Owner decision to reopen item 18 (restarts + core throughput) under its
existing wall gate, with Tier 1 coverage at 120 s as the first checkpoint.**
The measured failure is a tree/throughput failure on `core` and `deep-win`;
item 18 is the parked stack that targets exactly that, and D27 already prices
core throughput as first-order (`neither` `R` = 2.83/3.44). Do not raise the
120 s budget, do not start `REPLAY-BOUND` without a new owner decision, and do
not reopen the closed hand-guide/auxiliary line.

## Deliverables

- Data: [`measurements/plan9/`](measurements/plan9/README.md) —
  `run_plan9.py`, `env.json`, `coverage.json`, `README.md`.
- Docs: this report; `initiative.md` item-19 status, D27 status, priority and
  history; `bench.md` absolute-distance update and history. No ledger row.
- Engine: no change.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report9.md` and
   `measurements/plan9/`. Take the owner decision on reopening item 18
   (restarts + core throughput) with Tier 1 coverage at 120 s as the first
   checkpoint. Do not raise the 120 s budget and do not start
   `REPLAY-BOUND` without a new owner decision."
2. If the owner instead retains the wall target, authorise `REPLAY-BOUND`
   under plan9's pre-registered stop rule; Tier 1 remains red until a search
   change closes `core` and `deep-win`.
3. If the owner accepts 1/3 coverage as the current state, record that
   decision explicitly and close item 19 without a Tier 2 claim.
