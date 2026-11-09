# Prover Report 6: item 15 — the `GROUP` kill-test closes the hand-guide line

Plan: [`plan6.md`](plan6.md) (item 15, `GROUP` child). Executed 2026-11-09.
Implementation is behind the default-off `--guide-group` flag; no default
changed and no ledger row was appended. The plan's stop rule fired: **G2
fails, and G1 fails its substantive ≥ 5× clause, so the hand-guide line is
closed.** The planX scale case is re-checked and unchanged.

## Headline

- **`GROUP` does not cut `core`.** At `--evals 20000000 --memory 192`:
  flag-off 648,828 / 653,551 / 647,032 expansions; group-on 681,518 /
  678,413 / 678,624 on seeds 0–2. That is **0.96×**, not a cut. `core` is
  censored in both configurations.
- **The plan's literal G1 `< 1,000,000` is non-discriminating.** The current
  tree's flag-off hand stack is already at ≈ 650 k expansions at 20 M evals.
  The informative part of G1 is its ≥ 5× cut against the recorded 6.9 M
  tree, and that fails by a wide margin.
- **G2 fails.** `easy-win` proves on all seeds with the group, but expansions
  rise from 468,301 / 631,763 / 211,254 to 690,028 / 724,295 / 487,580
  (1.15–2.31×). Proof lengths also lengthen (18/18/17 → 28/28/21).
- **Soundness holds on the executed sweep.** Seed 0 of the frozen
  `guidance-v1` set with the group: 0 refutations; all 17 proofs that fit the
  5 M eval cap were accepted by `atomi validate` and `provari check` with
  equal position counts.
- **Scale case unchanged.** The group did not remove the 691 k-entry case.
  Re-run on this tree, the plan5 auxiliary stack gives `core` 120 s =
  14,332 positions / 691,117 entries and `easy-win` = 3,527 / 65,179; both
  accepted by both checkers. Plan7 phase A keeps `core` as the conservative
  scale case.
- **Stop rule:** no replay phase, no re-scope of item 15. The next plan must
  be a wall lever (D5 restarts, wall-budgeted auxiliary work, or the
  horizon-8 driver), not another hand-guide conformance attempt.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `892a3bb` on a **dirty tree** (plan6 files; agents do not commit) |
| Binary | `target/release/atomi` sha256 in [`measurements/plan6/env.json`](measurements/plan6/env.json) |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, 24 cells, sha256 `2da2a65b…8c7d` (verified) |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` (verified) |
| Stacks | off `--guide hand`; on `--guide hand --guide-group`; scale `--guide-scale 64 --guide-ef-weight 0 --guide-max-sim 1 --guide-max-settle 1` |

## Exact commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan6/run_group.py --phase env
python3 docs/plans/prover/measurements/plan6/run_group.py --phase group
python3 docs/plans/prover/measurements/plan6/run_group.py --phase soundness
python3 docs/plans/prover/measurements/plan6/run_group.py --phase scale
make test-full
make lint
```

One process at a time. `group` is 12 gate runs plus four seed-0 repeats;
`soundness` is 24 cells at 10 s / 5 M evals; `scale` is the `core` 120 s and
`easy-win` 600 s auxiliary proofs.

## Implementation

- **Flag.** `--guide-group` is a value-less switch (default off). It maps to
  `GuideConfig::group`, also reachable as `--guide-on group`; `--guide hand`
  itself is unchanged.
- **Predicate.** `Mover::is_plain` is the single exact predicate: legal,
  non-capture, non-promotion, non-en-passant, non-castling, non-commoner and
  non-check. A differential test runs it over every legal move of the
  fixture playouts against a made-child `checkers()` test.
- **Pseudo-child.** `Frame` partitions the plain moves out of the OR children
  and appends one synthetic `Group` child, starting at `(9, max(1, #plain))`.
  The non-plain children keep their movegen order and slots.
- **Value.** A `Group` is a deferred OR over its members: any proved member
  makes the group that member's child value (no extra ply, so the parent's
  `or_value` counts the plain move once); all members refuted makes a
  refutation pre-adjusted by one so the parent's OR lands on the members'
  own combination; otherwise `pn = min`, `dn = sum` over member values. The
  invariant is tested against `or_value` on the real member children.
- **Descent.** When the group is selected, the MID loop chooses one open
  member, computes the member thresholds through the same 1+ε algebra with
  the group as a virtual OR node, descends into the real child, updates the
  member, and re-syncs the synthetic child. Proof events stay on real
  positions: the parent event carries the selected member's move, and
  `proved_pending` retention uses the selected member's key.
- **Writer.** No format change. A group-proved parent resolves to a real
  member move; the existing JSONL DFS writes it and the member's own proof
  entry. Both checkers accepted every proved group proof in the sweep.

## Results

### G1/G2 (`group.json`)

| Cell | seed | off expansions | on expansions | ratio |
| --- | ---: | ---: | ---: | ---: |
| `core` | 0 | 648,828 | 681,518 | 1.05× |
| `core` | 1 | 653,551 | 678,413 | 1.04× |
| `core` | 2 | 647,032 | 678,624 | 1.05× |
| `easy-win` | 0 | 468,301 (proved) | 690,028 (proved) | 1.47× |
| `easy-win` | 1 | 631,763 (proved) | 724,295 (proved) | 1.15× |
| `easy-win` | 2 | 211,254 (proved) | 487,580 (proved) | 2.31× |

`core`: censored in all six runs. `easy-win`: proved in all six. The gate is
judged on the whole two-cell test:

| Gate | Result |
| --- | --- |
| G1 `< 1 M` literal | pass (681,518), but the baseline is already 648,828, so this is not evidence of a cut |
| G1 ≥ 5× cut (item 15) | **fail** (0.96×) |
| G2 proved all seeds | pass |
| G2 no expansion regression | **fail** (every seed regresses) |
| G4 seed-0 repeats identical | pass |

The plan's stop rule is explicit: if G1 fails, close; if G1 passes but G2
fails, the same close applies. Both the substantive G1 clause and G2 fail.

### Soundness (`soundness.json`)

Seed 0, `--guide hand --guide-group`, `--seconds 10 --evals 5000000`: 24
cells, **0 refuted**, 17 proved within the cap, 7 censored (the five corpus
cells, `dec01`, `easy-win`). All 17 proofs: `atomi validate` valid,
`provari check` valid, equal positions. The group property tests, the
crafted `MATE_IN_2` plain-win test and the `dec40` all-plain-lose test are in
`make test`; `make test-full` (slow fixture oracle + veto) and `make lint`
are green.

### Scale-case re-check (`scale.json`)

The planX phase-A dependency was “if `GROUP` cuts `core` below 1 M
expansions, the 691 k-entry case no longer exists”. It did not:

| Cell | budget | positions | entries | checkers |
| --- | --- | ---: | ---: | --- |
| `core` aux | 120 s | 14,332 | 691,117 | both accepted |
| `easy-win` aux | 600 s | 3,527 | 65,179 | both accepted |

So planX phase A keeps `core` (14,332 positions / ≈ 691 k entries) as the
conservative worker-sizing case and may use `easy-win` aux (3,527 / 65,179)
for the smaller proof. The `.pvr` freeze is unaffected by the group (the
flag adds no key and stays off).

## Surprises and findings

1. **The group is close to neutral on the full hand stack.** It changes the
   tree by a few percent (slightly worse), not by the order of magnitude the
   conformance hypothesis predicted. Whatever remains of provari's `core`
   gap is not this one divergence in isolation.
2. **Plan6's literal G1 was already satisfied by the baseline.** The recorded
   6.9 M figure came from `--guide-off sim,closures` at 200 M evals, not from
   the current hand stack at 20 M. A gate should state the same-session
   baseline it cuts against; otherwise `< 1 M` passes with no mechanism.
3. **`easy-win` regression is robust across seeds** (1.15–2.31×), while the
   proof stays valid. The group's deferred ordering lengthens the chosen
   proofs (18/18/17 → 28/28/21), consistent with treating a quiet set as one
   child.
4. **The scale case was re-derived from both prior sources, not cited.**
   Re-running `core` aux reproduced report4's 14,332 / 691,117 exactly;
   `easy-win` aux reproduced plan5's 3,527 / 65,179. The planX dependency is
   therefore satisfied on the current tree.

## Problems, open parts, missing tests

- **The full 24 × seeds 0–2 guidance soundness sweep was not run.** The
  executed seed-0 sweep found no refutation; the stop rule closed the line at
  G1/G2 before the wider sweep was needed. Any future revival must run it.
- **The group semantics are the plan's, not a byte-for-byte provari match.**
  The member install order is deterministic best-`pn` (with the existing
  class/history/seed tie-break), not necessarily provari's exact deferred
  order; item 15's cheapest decisive test was the group *as specified in
  plan6*, and it failed.
- **No replay phase** (plan6 phase B) was implemented, by the stop rule.
- **No ledger row** was appended: this is a mechanism kill-test, not a
  product stack, and no goal claim follows.

## Next steps

1. **Do not re-scope item 15.** Close the hand-guide line and record the
   negative result.
2. **Plan7 phase A may start** with the unchanged scale case: `core` aux
   14,332 positions / 691,117 entries (conservative), `easy-win` aux 3,527 /
   65,179. The group is not part of the M4 worker design.
3. **Make the next search plan a wall lever**, per the stop rule: D5 restarts,
   wall-budgeted auxiliary work, or the horizon-8 driver. The hand-guide
   conformance hypothesis is measured out.
4. If a future harness reports the recorded 6.9 M tree again, state its exact
   flags and budget next to the gate; do not compare a 20 M current-stack run
   to it.

## Deliverables

- Engine: `--guide-group` behind a default-off flag (`src/{cli,main}.rs`,
  `src/guide/mod.rs`, `src/mover.rs`, `src/search/{frame,mod}.rs`) and its
  unit/property/integration tests.
- Data: [`measurements/plan6/`](measurements/plan6/README.md) —
  `run_group.py`, `env.json`, `group.json`, `soundness.json`, `scale.json`,
  `README.md`.
- Docs: this report; `initiative.md` item-15 closure, item-5 scale note and
  history.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report6.md`,
   `plan6.md`, `measurements/plan6/` and `initiative.md`. Item 15 is closed.
   Execute planX phase A against the re-checked scale case (`core` aux
   14,332 positions / 691,117 entries; `easy-win` aux 3,527 / 65,179)."
2. Draft the D5 restart / wall-budget plan instead of M4; the hand-guide line
   is closed and the next search plan must be a wall lever.
3. Run the full 24 × seeds guidance soundness sweep before any item-15
   revival (not required for the current closure).
