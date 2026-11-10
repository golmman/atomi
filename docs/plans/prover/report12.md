# Prover Report 12: item 21 — ENTRY-REUSE stop rule fires; re-expansion is not the `deep-win` lever

Plan: [`plan12.md`](plan12.md) (item 21, D24/D27). Executed 2026-11-10.
Measurement only plus a default-off `--stats` attribution tool: no default
search change, no new proof, no coverage claim. The Phase A stop rule fired
(`entry_would_return` 0.0079 % of expansions at 20 M on `deep-win`), so
**Phase B (ENTRY-REUSE) and Phase C were not implemented or run**. The plan's
hypothesis is refuted; Tier 1 remains 2/3 (report11). No ledger row.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `2341c22` on a dirty tree (plan12 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `161f8a02…03c55` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, sha256 `2da2a65b…c8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Configuration | `--guide none` (bare tree), `--memory 192`, seed 0 |
| Phase A points | `--evals 20000000 --stats` on core/easy-win/deep-win; `--seconds 120` on deep-win (clean and `--stats`) |

Commands: the `env`, `attribution` and `coverage` phases in
[`measurements/plan12/README.md`](measurements/plan12/README.md), then
`make test`, `make lint`, `make test-full` (slow tier + `make veto`), one
process at a time.

## What was built (measurement only)

`--stats` adds plan-12 counters to `SearchResult` and a `"stats"` object to
the `prove` JSON: `unique_keys` (a bounded deterministic open-addressing key
set), `re_expansions`, `entry_proof_hits`, `entry_tt_hits`,
`entry_would_return`, `attacker_expansions`, `defender_expansions`, a depth
histogram and the selected-child class histogram. The node-entry check is a
**read-only** `Tables::lookup_with_source` before `Frame::enter`; it never
writes a table and never returns a value. `--guide none --seed 0` remains
counter-identical to m1 (`tests/search.rs::no_guide_seed_zero_is_m1_identical`
still pins 1,000,032 child evals / 30,786 expansions on `core`), and a new
`stats_are_observation_only` test pins equal `(exit, length, child_evals,
expansions)` with `--stats` on and off. `--entry-reuse` was **not** added:
that is Phase B, and the stop rule blocked it.

## Phase A — attribution at 20 M evals (exact)

Bare tree, seed 0, `--evals 20000000 --memory 192 --stats`.

| Cell | expansions | unique_keys | re_exp | re % | proof hits | TT hits | would_return | would % | attacker | defender |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| core | 630,056 | 586,075 | 43,981 | 6.98 % | 2 | 42,560 | **61** | 0.0097 % | 191,492 | 438,564 |
| easy-win | 669,920 | 622,562 | 47,358 | 7.07 % | 50 | 43,795 | **520** | 0.0776 % | 218,919 | 451,001 |
| deep-win | 668,608 | 620,162 | 48,446 | **7.25 %** | 0 | 46,955 | **53** | **0.0079 %** | 279,978 | 388,630 |

All `Quiet` selected-class histograms are expected: the bare tree carries no
classes.

## Phase A — deep-win at 120 s

| Flags | exit | expansions | child_evals | proof_entries | entry counters |
| --- | --- | ---: | ---: | ---: | --- |
| `--seconds 120` | timeout | 51,733,504 | 1,428,233,879 | 2,750,084 | — |
| `--seconds 120 --stats` | timeout | 49,730,560 | 1,379,493,488 | 2,649,166 | proof 193, TT 3,253,672, **would_return 15,849** (0.0319 %) |

The 120 s `--stats` row has extra `--stats` overhead and a 2^21-slot key set;
its `unique_keys` is **capped** at 1,468,006 (70 % load bound), so its
`re_expansions` (48,262,554) is an upper bound, not an exact ratio. The entry
counters are exact plain counters and are the data point that matters. The
depth histogram reaches ply 52 with large mass at plies 7–17 and a long tail;
the 20 M row reaches only ply 17 because the eval cap cuts first.

## Stop rule: fired

`deep-win` at 20 M: `entry_would_return / expansions` = 53 / 668,608 =
**0.0079 %**, far below the plan's 10 % threshold. Phase B (ENTRY-REUSE) is
not implemented, Phase C is not run, and no `--entry-reuse` flag exists.

**Why the hypothesis fails.** The parent's `Frame::enter` already classifies
every child through `Tables::lookup`, so a child's stored `(pn, dn)` is used
*before* the descent. The proposed node-entry check is therefore a second
lookup of a value the parent already has; it can only fire when the parent's
incoming thresholds are higher than the stored pair, which is rare. The
measured re-expansion factor at 20 M is **7.25 %**, not 10–20×. The plan's
18.5 ratio (52.3 M expansions / 2.8 M proof entries in report11's run; ours
is 51.7 M / 2.75 M) compared expansions with *proofs*, not with distinct
expanded keys; proof entries are a minority of the expanded positions, so
that ratio was not evidence of re-expansion.

## Decision (plan stop rule)

**Do not implement ENTRY-REUSE; report and propose a different tree
mechanism.** Deliberate consequences:

- **No Phase B, no Phase C, no D14.** The stop rule is a plan-level veto; a
  censored cell is not converted and no budget is raised.
- **Tier 1 remains 2/3** at 120 s from report11 (`core` 3/3 17.390 s,
  `easy-win` 3/3 2.008 s, `deep-win` 1/3). This plan does not change coverage.
- **The bare tree remains the product candidate** (report11) and `--stats` is
  a measurement tool, default off, with no trajectory change.
- **The next lever is a different tree mechanism**, not another entry check.
  The data show the 20 M tree is 92.75 % unique keys; a mechanism must reduce
  *unique* expansions, not re-expansions.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 3 twenty-million rows + 2 deep-win 120 s rows + stop record; parsed JSON, `env.json`, one process at a time |
| G2 stop rule | **pass (fired)**: 0.0079 % < 10 % at 20 M on deep-win; Phase B/C not run |
| G3 mechanism | **not applicable**: Phase B not started; Tier 1 remains 2/3 (report11), no coverage claim |
| G4 soundness | **pass**: no refutation; no new claimed proof; `make test`, `make lint`, `make test-full` and `make veto` green |
| G5 docs | **pass**: no GPL dependency, raw transcript, provari artifact, proof file or `__pycache__` |

## Findings

1. **ENTRY-REUSE is dead on the data.** It would skip 53 of 668,608
   `deep-win` expansions at 20 M (0.0079 %) and 15,849 of 49.73 M at 120 s
   (0.0319 %). Both are far below the 10 % stop threshold.
2. **Re-expansion is not the tree lever at 20 M.** `deep-win` is 92.75 %
   unique keys (7.25 % re-expansions); `core` and `easy-win` are 6.98 % and
   7.07 %. The 18.5 ratio in the plan compared expansions with proof-table
   entries, not distinct keys.
3. **The node-entry check is structurally redundant.** The parent's frame
   already looks up the child's stored value during classification; a second
   entry check only adds a rare crossing test. This is the why, not just a
   low count.
4. **The 120 s tree is large and deep.** 51.7 M expansions, 2.75 M proof
   entries, a depth tail to ply 52 with mass at plies 7–17. The 120 s
   distinct-key ratio is capped and is not claimed; the exact 20 M ratio is
   the stop-rule evidence.
5. **No soundness or determinism change.** `--stats` is observation-only; the
   existing m1 identity test and a new on/off trajectory test pass.

## Problems, open parts, missing tests

- **The 120 s distinct-key ratio is not measured.** The `--stats` key set
  caps at 1.47 M under the 192 MB label; an exact 120 s ratio would need a
  measurement-only set well above the label. The 20 M exact ratio is the
  stop-rule point, so this does not affect the decision.
- **No Phase B tests.** They were not written because Phase B was not
  implemented; the plan's ENTRY-REUSE test list is moot.
- **Tier 1 is still red (2/3).** `deep-win` seeds 0 and 2 remain censored at
  120 s; no proof exists for them and none is claimed.
- **No default-flag change.** The bare tree is the measured product candidate;
  adopting it is still an owner decision.

## Next step (exactly one recommendation)

**Open a new plan for option B, move-limit iterative deepening** (a
`remaining`/move-limit key dimension with iterative deepening, the plan-12
ideation option B), starting with a **read-only depth/move-limit probe** on
`deep-win` seed 0 at 20 M evals and at 120 s: measure whether a move limit at
20/24/28 plies cuts the tree by ≥ 2×. If it does not, stop and take a kill
review rather than implementing the key dimension. Do not implement
ENTRY-REUSE, do not reopen the hand-guide/auxiliary line, do not raise the
120 s budget, and do not build a portfolio. The 20 M data say the tree is
mostly unique; the cheapest decisive test of the remaining formulation option
is whether the deep tail is where the work is.

## Deliverables

- Data: [`measurements/plan12/`](measurements/plan12/README.md) —
  `run_plan12.py`, `env.json`, `attribution.json`, `coverage.json`,
  `README.md`.
- Docs: this report; `initiative.md` item 21 status and priority; `bench.md`
  history. No ledger row.
- Engine: `src/search/mod.rs` (`--stats` counters, read-only entry
  attribution), `src/search/keyset.rs`, `src/table/mod.rs`
  (`lookup_with_source`), `src/cli.rs`/`src/main.rs` (`--stats`), tests. No
  default change; no `--entry-reuse`.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report12.md` and
   `measurements/plan12/`. The plan-12 ENTRY-REUSE stop rule fired (0.0079 %
   of expansions at 20 M on `deep-win`); do not implement entry reuse. Write a
   plan for option B, move-limit iterative deepening, with a read-only
   depth/move-limit probe on `deep-win` seed 0 at 20 M evals and at 120 s
   first. Do not reopen the hand-guide/auxiliary line, do not raise the 120 s
   budget, and do not build a portfolio."
2. If the owner prefers option C (proof-table keyed by remaining + iterative
   shortening), that is a larger change to the proof format contract and needs
   its own plan and D-number; this report does not authorise it.
3. If the owner accepts 2/3 as the coverage goal, that is a goal change
   (D27 amendment) and needs a new decision; this report does not authorise
   it.
