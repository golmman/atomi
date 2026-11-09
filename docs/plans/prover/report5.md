# Prover Report 5: item 5 (M4) — P0 probe at the product budget

Plan: [`plan5.md`](plan5.md) (initiative [`prover`](initiative.md), backlog
item 5, M4 probe P0). Executed 2026-11-09 as a measurement-only session: no
engine code changed, no product or speed claim, no ledger row. The plan asked
for a probe, not a report; the owner requested this report after execution.

## Headline

- **Coverage at 10 s is 19/24 cells (57/72 rows), exactly plan3's hand-guide
  coverage.** The auxiliary stack (`scale 64`, `ef_weight 0`,
  `max_sim=1`, `max_settle=1`) proves `easy-win` and all 18 `dec*` cells on
  seeds 0–2. The five corpus cells `core`, `deep-win`, `b9a7ed7eb368`,
  `3004d09cef79`, `dd3f89766a40` are censored on every seed. The aux budget
  adds no coverage, it only changes the price.
- **`easy-win`'s aux inflation reproduces report4 exactly**:
  918,551 / 759,177 / 773,238 expansions on seeds 0 / 1 / 2 — all far over
  the D21 limit 145,046. It is still a 10 s proof, at 4.5–5.4 s.
- **Proved cells are cheap, not near 10 s.** Proved-run walls: min 0.002 s,
  median 0.064 s, max 5.421 s. The 10 s budget is spent on the five
  censored cells, not spread over the auxiliary work.
- **`--proof-out` is neutral on proved runs.** `easy-win` seed 0 has
  identical `child_evals` (27,242,967) and expansions (918,551) with and
  without the flag and equal to its P0-a row. No censored run wrote a proof.
- **A wall-deadline censor is not counter-identical across repeats.** The
  literal G2 check fails on `core` seed 0 (45,244,283 / 44,370,650 /
  44,587,197 child evals across P0-a / plain / `--proof-out`). Under a work
  cap the same trajectory is exact: two plain runs and one `--proof-out` run
  at `--evals 40000000` all end `exit=evals`, 40,000,029 child evals,
  1,295,213 expansions. The difference is scheduler-dependent work at the
  wall deadline, not search nondeterminism.
- **Scale case:** `easy-win` (the largest proved cell by reference nodes)
  exports 3,527 positions, 65,179 proof entries, 293,459 JSONL bytes; the
  proof is accepted by `atomi validate` and `provari check` (3,527 =
  3,527).
- **`deep-win` is censored for search, not output**, at 120 s:
  `exit=timeout`, 559,984,639 child evals, 20,167,680 expansions,
  1,211,031 proof entries, `proof_full=false`, no proof file.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X 16-Core (4 vCPU container), Linux 7.2.13-200.fc44.x86_64 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `95d249899fdf3420bd7266b5d0e91b7f750d53d5` on a **dirty tree** (plan5 measurement files; agents do not commit) |
| Binary | `target/release/atomi` sha256 `aaeed9014798f971a910b428e6d08b6f049edb656a4d7ae41677318f223b8582` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206`, scratch build `/tmp/atomi-oracle` |
| Guidance set | `guidance-v1.tsv`, 24 cells, sha256 `2da2a65b…8c7d` (verified) |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` (verified) |
| Stack | `--guide hand --guide-scale 64 --guide-ef-weight 0 --guide-max-sim 1 --guide-max-settle 1` |

Full digests are in [`measurements/plan5/env.json`](measurements/plan5/env.json).

## Exact commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase env
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase coverage
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase determinism
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase probes
make test
make lint
```

One process at a time. `coverage` is 24 cells × seeds 0–2 at
`--seconds 10 --memory 192`; `determinism` is the four 10 s runs plus three
`--seconds 600 --evals 40000000` explanation runs; `probes` is the 600 s
D14 run on the largest proved cell and the 120 s `deep-win` attempt.

## Results

### P0-a coverage (`coverage.json`)

| Class | Cells | Rows |
| --- | --- | ---: |
| proved on all seeds | `easy-win`, `dec01`–`dec20` (18) | 57/72 |
| censored on all seeds | `core`, `deep-win`, `b9a7ed7eb368`, `3004d09cef79`, `dd3f89766a40` | 15/72 |

- `easy-win` seeds: 5.42 s / 918,551, 4.51 s / 759,177, 4.66 s / 773,238
  expansions; lengths 19 / 19 / 17.
- Censored `core` reaches 44.6–45.2 M child evals and 1.42–1.47 M
  expansions at the deadline, consistent with report4's 10 s row.
- `deep-win` at 10 s reaches 48.7–50.1 M evals; the other three censored
  corpus cells reach 49–52 M evals.
- Proved walls outside `easy-win` are 0.002–1.51 s (median 0.064 s over all
  proved rows); the 18 `dec*` cells prove on every seed.
- No guidance-set refutation.

### P0-b determinism (`determinism.json`)

| Cell | P0-a | plain | `--proof-out` | equal? |
| --- | ---: | ---: | ---: | --- |
| `easy-win` evals / expansions | 27,242,967 / 918,551 | same | same | yes, all three |
| `core` evals | 45,244,283 | 44,370,650 | 44,587,197 | no (censored) |

The `core` difference is the plan's G2 finding. It is explained by the wall
deadline: a censored run stops whenever the clock expires, so the amount of
work completed is a scheduling result. With `--evals 40000000` and a 600 s
deadline, plain-1 / plain-2 / `--proof-out` are identical
(`exit=evals`, 40,000,029 evals, 1,295,213 expansions). `--proof-out`
remains trajectory-neutral, and a censored run writes no file.

### P0-c scale case (`probes.json`)

Largest proved cell by reference nodes: `easy-win` (72,523). D14 protocol,
same stack, seed 0, 600 s:

| Field | Value |
| --- | ---: |
| wall | 5.35 s |
| `child_evals` | 27,242,967 (equal to P0-a) |
| length | 19 |
| positions | 3,527 |
| proof entries | 65,179 |
| JSONL bytes | 293,459 |
| `proof_full` | false |
| checkers | `atomi validate` valid, `provari check` valid, positions equal |

### P0-d `deep-win` (`probes.json`)

120 s, same stack, seed 0, `--proof-out`: censored **`deadline`**
(`exit=timeout`, no proof file); 559,984,639 child evals, 20,167,680
expansions, 1,211,031 proof entries, `proof_full=false`. This reproduces
planX's recorded 562.7 M / 20.3 M / 1.22 M within session noise.

## Surprises and findings

1. **The aux stack buys no 10 s coverage.** It proves exactly the 19 cells
   plan3's hand stack proved. Its value is the 120 s `core` proof in
   report4, not the 10 s gate. This strengthens the M4-first pivot: there
   is no 10 s stack to ship.
2. **"Expected near 10 s" was wrong for proved cells.** Only `easy-win`
   spends seconds; the median proved wall is 64 ms. The auxiliary budget is
   expensive *per expansion*, not a fixed 10 s tax.
3. **`easy-win` is now a 42× wall win with 12.7× the D21 expansion limit.**
   The aux point is a coverage/proof-scale configuration, not a product
   configuration; the D21 limit is not met by this stack.
4. **Plan5's G2 wording is too strong for censored runs.** "Same binary +
   same flags = same counters" holds only when the run reaches a
   deterministic stop (proof or eval cap). Plan7's D24 replay design already
   avoids this with no deadline and `--evals = recorded + 1`; plan5 supplies
   the empirical justification.
5. **The aux proof is larger than the chosen-stack proof.** `easy-win` aux:
   3,527 positions / 65,179 entries / 293 KB, versus planX's chosen-point
   record of 2,493 / 10,550 / 210 KB. If the aux stack is the M4 scale case,
   the worker sizing must use the larger numbers.
6. **`deep-win` remains a search censor, not a `proof_full` censor** at
   120 s. The planX scale case stays `core` (14,332 positions, ≈691 k
   entries from report4).

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 72 P0-a rows, each with a verdict; P0-c and P0-d classified |
| G2 determinism | **pass with a recorded finding**: proved runs exact; censored 10 s `core` differs only by deadline-completed work; the work-capped run is exact; `--proof-out` neutral; censored writes no proof |
| G3 soundness | **pass**: no refutation; the P0-c proof is accepted by both checkers (3,527 positions) |
| G4 artefacts | **pass**: `run_p0.py`, `env.json`, `coverage.json`, `determinism.json`, `probes.json`, `README.md`; temporary proof JSONL removed; no raw transcripts, provari artifacts or `__pycache__`; `make test` and `make lint` green |

`make test-full` / `make veto` were not run: no engine code changed. The
P0-c proof was checked directly by `provari check`.

## Problems, open parts, missing tests

- **P0-c validates one proof, not all 19 proved cells.** Soundness evidence
  is "no refutation + scale-case proof accepted"; a checker sweep over every
  proved cell is unrun.
- **`core` still has no 10 s proof.** Its scale case remains the report4
  120 s aux proof, which this probe did not re-run (flags/tree differ from
  plan4, so it cannot be cited automatically).
- **Censored 10 s counters are host- and scheduling-sensitive.** They are
  recorded as observations, never as deterministic counters.
- **The P0-b explanation probe uses `--evals 40000000`**, which is outside
  the plan's literal P0-b protocol but inside the same binary/flags; it is
  labelled as an explanation, not a coverage row.
- **No engine tests were added** (no code change). The wall-deadline
  determinism caveat is documented here and in the README, not enforced by
  a test.

## Next steps

1. **Plan6 first** (item 15 `GROUP` kill-test), per the recorded order. If
   it passes, re-check the scale case; a sub-1 M-expansion `core` would
   retire the current 691 k-entry sizing.
2. **Plan7 sizing:** use `easy-win` aux (3,527 positions / 65,179 entries /
   293 KB) if the aux stack is chosen; keep `core` (14,332 positions /
   ≈691 k entries) as the conservative scale case.
3. **Plan7 replay:** no deadline, eval cap one above the recorded count, and
   compare `child_evals`, `expansions`, `exit`, `length`. Plan5's capped
   probe is the direct precedent.
4. **Optional P0-c extension:** export and check proofs for the remaining
   proved `guidance-v1` cells at 10 s; this is cheap for the `dec*` cells
   and closes the G3 "every proved cell" wording.

## Deliverables

- Driver and data:
  [`measurements/plan5/`](measurements/plan5/README.md) —
  `run_p0.py`, `env.json`, `coverage.json`, `determinism.json`,
  `probes.json`, `README.md`.
- Status lines: [`initiative.md`](initiative.md) (status, item 5, history),
  [`docs/plans/README.md`](../README.md), this report.

**SESSION COMPLETE**

Follow-up options:

1. **Kickoff (recommended):** "Read `docs/plans/prover/report5.md`,
   `plan5.md`, `measurements/plan5/`, `plan6.md` and `initiative.md`.
   Execute plan6 (item 15 `GROUP` kill-test); then re-check the scale case
   from the plan5/report4 data before planX phase A."
2. Proceed to planX with `easy-win` aux (3,527 positions / 65,179 entries /
   293 KB) as the scale case and `core` 120 s as the conservative case.
3. Run the optional P0-c extension: export and `provari check` every proved
   `guidance-v1` cell at 10 s.
