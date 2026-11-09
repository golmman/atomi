# Prover Plan 5: item 5 (M4) — P0 probe (aux stack at the product budget)

Initiative: [`prover`](initiative.md), backlog **item 5 (M4)**, probe phase
P0 split out of [`planX`](planX.md). Drafted 2026-11-09 in plan session 6
after [`report4.md`](report4.md) and the item-15 erratum. This document does
not implement. Measurement only; no product or speed claim.

## Why this plan

`report4.md` §Next steps asks for the auxiliary-budget stack on the frozen
`guidance-v1` set at the product budget; it has only been run on the two
canonical cells. Plan7 (M4) needs three facts before it can size anything:

1. What the aux stack proves at 10 s on the 24 frozen cells (coverage and
   which cell is the largest proved proof).
2. Whether `--proof-out` is trajectory-neutral **at 10 s** with the aux
   stack (the only determinism evidence today is the untimed 120 s run).
3. Whether `deep-win` has any proof at 120 s, or is censored for search, not
   output.

## Inputs (pinned, no re-measurement)

| Item | Value |
| --- | --- |
| Binary | release build of the current tree (`cargo build --release`); record sha256 |
| Guidance set | `measurements/plan3/guidance-v1.tsv`, 24 cells, sha256 `2da2a65b…8c7d` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |
| Aux stack | `--guide hand --guide-scale 64 --guide-ef-weight 0 --guide-max-sim 1 --guide-max-settle 1` |
| Budget | `--seconds 10 --memory 192`, seeds 0, 1, 2 |
| Reference | plan4 same-session provari medians (`core` 0.533 s, `easy-win` 0.137 s, `deep-win` 6.567 s); no new oracle run unless the host differs |

All flags above are the plan4 aux point; this plan adds no new flag and
changes no default.

## Measurement

- **P0-a (coverage).** 24 cells × seeds 0–2 at 10 s, aux stack. Record
  `proved`/`censored`, `expansions`, `child_evals`, `sim_nodes`,
  `settle_nodes`, `proof_full`, wall. One row per `(cell, seed)`; 72 rows.
- **P0-b (determinism).** For `core` seed 0 and `easy-win` seed 0, repeat
  P0-a with `--proof-out` at the same 10 s budget and compare `child_evals`,
  `expansions` and `exit` exactly. A censored run must write no proof.
- **P0-c (scale case).** For the largest cell proved in P0-a, run the D14
  proof protocol (`--seconds 600`, no eval cap) and record positions, proof
  entries, JSONL bytes, `child_evals` equality and both checkers' verdicts.
  `easy-win` may cite the plan4 chosen proof; `core` may cite
  `measurements/plan4/pressure.json` only if the flags and tree match
  exactly, otherwise re-run.
- **P0-d (`deep-win`).** One 120 s aux attempt with `--proof-out`; classify
  the censor (search / `proof_full` / deadline) and record counters. No
  proof is expected; a proof would move the planX scale case.

Driver: `measurements/plan5/run_p0.py`, stdlib only, subprocess + JSON-line
parsing, one process at a time. No captures of stdout beyond the parsed
fields.

## Gate (data, not product)

- **G1 data.** 72 P0-a rows present, each with a verdict; P0-c and P0-d
  classified. Missing rows are a failure, not a skip.
- **G2 determinism.** P0-b counters identical; any difference is a finding
  and planX does not start until it is explained.
- **G3 soundness.** No guidance-set refutation. Any proved cell's proof is
  accepted by `atomi validate` and `provari check` (or cited as in P0-c).
- **G4 artefacts.** `measurements/plan5/{run_p0.py,probes.json,coverage.json,
  determinism.json,env.json,README.md}`; no raw transcripts, provari
  artifacts or `__pycache__`. `make test`/`make lint` green (no code change
  expected).

A censor is a recorded result. This plan has no speed gate and appends no
ledger row.

## Output the next plans consume

- Coverage table at 10 s per cell (proved / censored) and the aux wall
  distribution on proved cells — expected to be near 10 s, because the aux
  budget spends its time in simulation/settle.
- The scale case for planX: largest proved proof (positions, entries, bytes)
  and whether `deep-win` stays censored.
- The `--proof-out` neutrality answer at 10 s (planX G3 assumes it).
- Whether `easy-win`'s aux inflation (918,551 expansions at 10 s in report4,
  proved but 6.3× the D21 limit) reproduces on seeds 1–2; this is a coverage
  datum, not a gate.

## Tasks

- **T1** build, pin sha256, write `env.json` and `run_p0.py`.
- **T2** P0-a (72 runs), `coverage.json`.
- **T3** P0-b (4 runs), `determinism.json`.
- **T4** P0-c (1–2 D14 runs), P0-d (1 run), `probes.json`.
- **T5** README with commands and parsed tables; docs status line; then stop
  (report5 only if the owner asks for one; this is a probe).

## Risks

| Risk | Mitigation |
| --- | --- |
| Aux runs are slow (sim/settle), 72 × 10 s plus proof runs | Bound by `--seconds 10`; P0-c only on the largest proved cell |
| 10 s walls are host-sensitive | One process at a time; same-session reference; record host in `env.json` |
| `--proof-out` at 10 s hits `proof_full` and changes the run | That is the finding; record it, do not raise `--proof-share` in this plan |
| `deep-win` 120 s attempt is inconclusive | Record the censor class; planX keeps `core` as scale case |
| P0-c proof run exceeds 600 s | D14 allows the cap; a cap hit is recorded, proof not claimed |
