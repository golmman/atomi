# Prover Report 8: item 17 — the core is first-order per expansion, and the wall goal is not near-term

Plan: [`plan8.md`](plan8.md) (item 17, comparable per-expansion diagnosis and
the 60 s coverage bracket). Executed 2026-11-09. Measurement and decision only:
no engine, test or default change. **Decision A: core throughput is
first-order** (`R > 2` for `neither`). **Decision B: recommend item 19
(owner re-pin)**; the product censors `core` and `deep-win` at 60 s. No ledger
row and no goal claim follow.

## Headline

- **The old "per-node is competitive" claim was a unit error.** In the unit
  the goal uses, `neither` costs **2.83×** (easy-win) and **3.44×** (core)
  provari's ns per node, both above the pre-registered `R = 2` threshold. The
  bare tree is competitive per expansion (`R` 1.19 / 1.33) but pays a 4.4× /
  1.1× larger tree at 10 M evals.
- **The product still censors at 60 s.** Calibrated hand proves `easy-win` in
  6.013 s (58.4×), but `core` (0.94 M expansions, 204 M aux) and `deep-win`
  (1.02 M expansions, 218 M aux) hit the 60 s deadline on all seeds. Factors
  are lower bounds, ≥ 121.5× and ≥ 9.36×.
- **The guide-free tree does cover `core` at 60 s.** `--guide none` proves
  `core` on seeds 0–2 in 16.6 / 21.0 / 17.1 s (median 17.1 s, 34.6×), with
  all three D14 proofs accepted by both checkers. This is the zero-guide
  floor, not the product configuration; it does not convert the product censor.
- **Recommendation: item 19 (re-pin).** The pre-registered rule fires on the
  product censor, independent of the bare-tree observation. Item 18 is not
  started in this plan.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `8189736` on a dirty tree (plan8 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `e9587a6f…c46ec` (the plan7 binary) |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, sha256 `2da2a65b…8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Product point | `--guide hand --guide-scale 64 --guide-ef-weight 0` (plan4) |

Commands: the five `run_plan8.py` phases in
[`measurements/plan8/README.md`](measurements/plan8/README.md), then
`make test` and `make lint`, one process at a time. Phase A and B used the
same host and session; the phase-B provari block was measured after the 60 s
atomi rows.

## Phase A — comparable per-expansion cost

Fixed 10 M evals, medians over seeds 0–2. `R = (ns/exp) / (ns/node)`.
Provari: easy-win 106 ms / 72,523 nodes (1461.6 ns/node); core 499 ms /
342,289 nodes (1457.8 ns/node).

| Stack | Cell | ns/exp | exp/s | children/exp | R |
| --- | --- | ---: | ---: | ---: | ---: |
| bare | easy-win | 1732.8 | 577 k | 31.1 | 1.186 |
| bare | core | 1942.2 | 515 k | 32.3 | 1.332 |
| neither | easy-win | 4133.4 | 242 k | 32.0 | **2.828** |
| neither | core | 5020.6 | 199 k | 32.0 | **3.444** |
| no-sim | easy-win | 15,762.1 | 63 k | 27.3 | 10.784 |
| no-sim | core | 20,728.1 | 48 k | 32.9 | 14.219 |
| hand | easy-win | 49,892.7 | 20 k | 27.3 | 34.136 |
| hand | core | 66,400.0 | 15 k | 32.6 | 45.548 |

`children/exp` is ~31–33 for every stack except hand easy-win (27.3), so the
gap is not a child-count artefact. Bare per-expansion work is close to
provari; the hand guide's optional work is what raises `neither` above the
threshold even with simulation and closures off. **Decision A fires on both
cells.**

Canonical 10 s (easy-win proved 3/3 for every stack; core censored):

| Stack | easy-win wall · exp | core wall · exp |
| --- | ---: | ---: |
| bare | 1.976 s · 1.24 M | 10.000 s · 5.38 M |
| neither | 3.924 s · 0.99 M | 10.000 s · 2.13 M |
| no-sim | 2.074 s · 0.13 M | 10.012 s · 0.49 M |
| hand | 5.770 s · 0.12 M | 10.023 s · 0.15 M |

The zero-guide floor is bare easy-win at 1.976 s median (3/3 proved),
18.6× the same-session provari wall. No guide variant can go below that floor.

## Phase B — 60 s coverage bracket

Product (`hand`, 60 s / 192 MB), medians over seeds 0–2 except easy-win
(seed 0). Provari references are same-session: core 494 ms / 342,289 nodes,
deep-win 6413 ms / 4,574,912 nodes, easy-win 103 ms / 72,523 nodes.

| Cell | proved | wall | expansions | child_evals | aux | factor |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| core | **0/3** | 60.024 s | 0.94 M | 29.6 M | 204 M | **≥ 121.5×** |
| deep-win | **0/3** | 60.007 s | 1.02 M | 31.4 M | 218 M | **≥ 9.36×** |
| easy-win | 1/1 | 6.013 s | 121,866 | 3.32 M | 22.8 M | 58.4× |

The product censor is a search censor (`timeout`, no `proof_full`, no
refutation). A censored row is not converted into a proof.

**Zero-guide floor:** `--guide none` on core proves 3/3 in 16.618 / 20.982 /
17.089 s (median 17.089 s, 34.6×), 9.27–11.25 M expansions, length 14 / 14 /
24. All three D14 proofs are accepted by `atomi validate` and `provari check`
with equal `child_evals` (10,831 / 11,107 / 11,879 positions). The bare tree
therefore covers core at 60 s, but the product does not; the observation is
recorded, not substituted for the decision.

**Decision B fires:** product core and deep-win are censored, so the rule
recommends **item 19 (re-pin)**.

## Soundness

- Guidance-v1 was not re-swept (no engine change); the plan's soundness gate
  is the D14 protocol for every proved phase-B row.
- 4/4 D14 proofs accepted by both checkers: hand easy-win seed 0
  (3,321,312 evals, 2,493 positions) and bare core seeds 0–2.
- No refutation or never-prove appeared; `make test` is green, `make lint` is
  green. `make test-full` is not required (no search/table/proof code change).

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: phase A 6 provari + 48 atomi rows; phase B 10 atomi + 9 provari + 4 D14 rows; parsed JSON only |
| G2 comparability | **pass**: same host/session, three provari repeats, units defined; no old reference row used for a factor |
| G3 decision | **pass**: `R`, the 60 s coverage and the pre-registered recommendation recorded; no cell dropped, no metric changed |
| G4 soundness | **pass**: no refutation; all claimed D14 proofs accepted by both checkers; `make test` green |
| G5 docs | **pass**: `make lint` green; no GPL dependency, raw transcript, proof or `__pycache__` |

## Surprises and findings

1. **Bare vs guide is the whole per-expansion story.** Bare `R` is ~1.2–1.3,
   but its tree is large; `neither` keeps the large tree and adds enough
   guide work to cross `R = 2`. The guide buys tree reduction at a per-node
   price that is still first-order. Any item-18 core work is guide work, not
   movegen work.
2. **The product's 60 s censor is worse than plan7's 10 s censors suggested.**
   `core` spends ~204 M auxiliary units and still only reaches 0.94 M
   expansions. `deep-win` is not close.
3. **The guide-free tree proves core at 60 s.** This was not part of the
   pre-registered decision rule, but it bounds what a tree-only or
   guide-disabled configuration can do. It is 34.6× provari, so it does not
   make the wall goal near-term.
4. **The easy-win wall moved slightly** (5.770 s vs plan7's 5.829 s) and the
   provari core/easy reference is stable (494/103 ms vs 515/139 ms). This is
   normal same-host variation, not a trajectory change.

## Problems, open parts, missing tests

- **Recommendation is re-pin, not a wall fix.** The two owner options under
  item 19 are unchanged: (a) a coverage-first tier (3/3 at a stated budget,
  then wall on shared cells), or (b) a node/`child_evals` score. The plan does
  not choose between them.
- **`deep-win` was not attempted at 120 s**; the 60 s bracket censors it. The
  plan forbids a hidden budget increase.
- **The bare core proof is large** (~0.9 MB JSONL, 10.8–11.9 k positions), so
  it is evidence for coverage, not an M4 scale case.
- **Decision A's "continue" branch is unexercised.** If the owner rejects
  re-pin, item 18 must include core throughput per expansion; the plan does
  not authorise that work.

## Next steps

1. **Owner decision on item 19 (recommended):** choose a coverage-first tier
   or a node/`child_evals` score; update `bench.md`, the success condition and
   the ledger together. No silent change.
2. **If continue is chosen instead:** item 18 is a named stack with core
   throughput (Decision A) and/or restarts, under the wall gate in the backlog.
3. **Do not start another hand-guide/auxiliary plan.** Items 3, 15 and 16 stay
   closed.

## Deliverables

- Data: [`measurements/plan8/`](measurements/plan8/README.md) —
  `run_plan8.py`, `env.json`, `pernode.json`, `longbudget.json`, `README.md`.
- Docs: this report; `initiative.md` item-17 status, item-18/19 rows, D26
  status and history. `bench.md` receives no row (no goal claim).
- Engine: no change.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report8.md` and
   `measurements/plan8/`. Take the item-19 owner decision: choose a
   coverage-first tier or a node/`child_evals` score, then update `bench.md`,
   the success condition and the ledger together. Do not start item 18 unless
   continue is chosen."
2. Conditional: if the owner chooses continue, draft `plan9.md` for item 18
   (core throughput per expansion plus D5 restarts) under the wall gate.
3. Re-run the 60 s bracket on a re-pinned reference after the owner decision;
   do not re-use this session's factors across a new pin.
