# plan13 measurements

Plan: [`../../plan13.md`](../../plan13.md) (item 22, cheap probes before any
conformance stack). Report: [`../../report13.md`](../../report13.md). Parsed
output only: no raw transcripts, provari artifacts, proof files or
`__pycache__` are committed. Temporary proofs are written under `proofs/`,
checked, then deleted by the driver.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary, corpus/guidance sha256, provari version/commit, bare flags, budgets |
| `phase_a.json` | proof-share sweep: 3 cells × seeds 0–2 × shares 25/40/50/60 at 120 s; summary and stop rule; 26 D14 rows |
| `phase_b.json` | `--max-depth` 16/20/24/28/32 plus bare at 20 M, seed 0, `--stats`; stop rule |
| `phase_c.json` | conformance matrix: 11 stacks × 3 cells at 20 M, seed 0, `--stats`; seed 1 for the best stack; stop rule; 12 D14 rows |
| `run_plan13.py` | stdlib-only driver (`env`, `phase_a`, `d14`, `phase_b`, `phase_c`, `d14_c`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan13/run_plan13.py --phase env
python3 docs/plans/prover/measurements/plan13/run_plan13.py --phase phase_a
python3 docs/plans/prover/measurements/plan13/run_plan13.py --phase d14
python3 docs/plans/prover/measurements/plan13/run_plan13.py --phase phase_b
python3 docs/plans/prover/measurements/plan13/run_plan13.py --phase phase_c
python3 docs/plans/prover/measurements/plan13/run_plan13.py --phase d14_c
make test
make lint
make test-full
```

One atomi process at a time. `phase_a` ran first (36 rows), then `d14` (26
proved rows), then `phase_b` (12 rows), then `phase_c` (33 + 3 rows), then `d14_c`
(11 proved rows), then the gates.

## Row counts and units

- **Phase A:** 36 rows (3 cells × 3 seeds × 4 shares) + 26 D14 rows.
- **Phase B:** 2 bare + 10 cap rows at `--evals 20000000 --stats`.
- **Phase C:** 11 stacks × 3 cells = 33 rows at `--evals 20000000 --stats`,
  plus 3 seed-1 rows for the best stack (`lo_fresh`), plus 11 D14 rows for
  the proved `easy-win` rows.
- `expansions` is a main-search node expansion; `child_evals` is one per
  child classified. `unique_keys` is the distinct-key count from the
  `--stats` set (capacity 2^21, 70 % load bound); at 20 M it is not capped.
- A censored row is never converted. D14 proof runs use `--seconds 600
  --proof-out` with the same flags/seed/share; the driver requires equal
  `child_evals` and both checkers.

## Phase A — proof-share sweep

`--guide none --seconds 120 --memory 192`, seeds 0–2.

| Share | `core` | `easy-win` | `deep-win` | deep-win censors |
| ---: | --- | --- | --- | --- |
| 25 | 3/3, 17.804 s | 3/3, 2.067 s | **1/3** | seed 0, seed 2 |
| 40 | 3/3, 19.344 s | 3/3, 2.068 s | **0/3** | all |
| 50 | 3/3, 19.095 s | 3/3, 2.122 s | **1/3** | seed 0, seed 1 |
| 60 | 3/3, 16.754 s | 3/3, 2.036 s | **0/3** | all |

`deep-win` proves on seed 1 at share 25 (116.052 s, length 25) and on seed 2
at share 50 (114.920 s, length 27). No share is 3/3, so stop rule A does not
fire. The 10 % wall guard on `core`/`easy-win` passes for every share (worst
+8.6 % on `core` at 40). All 26 proved rows pass D14 (equal `child_evals`,
both checkers).

## Phase B — move-limit probe

`--guide none --evals 20000000 --memory 192 --seed 0 --stats`.

| Cap | `deep-win` unique | ratio | proof | `core` unique | ratio | proof |
| ---: | ---: | ---: | --- | ---: | ---: | --- |
| bare | 620,162 | 1.00 | no | 586,075 | 1.00 | no |
| 16 | 284,884 | **2.18×** | no (depth) | 53,800 | 10.89× | no (depth) |
| 20–32 | 620,162 | 1.00 | no | 586,075 | 1.00 | no |

Caps 20–32 are inert at 20 M (the eval cap fires first). Cap 16 cuts the
tree but censors at depth and reaches no proof, so it fails the "reaches the
bare proof length (or proves)" clause. **No cap passes; the `remaining` key
dimension is dropped.**

## Phase C — conformance probe

`--evals 20000000 --memory 192 --seed 0 --stats`. Full stack:
`--guide hand --guide-group --guide-scale 1 --guide-ef-weight 0
--guide-fresh-dn --guide-replay-exact`.

| Stack | `core` unique | cut | `easy-win` | `deep-win` |
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

Best `core` cut is **1.22×** (`lo_fresh`), not 2×. The full stack cuts
`easy-win` 3.2× but leaves `core` censored. Seed 1 for `lo_fresh`: `core`
481,345 unique, `easy-win` proved 130,192, `deep-win` censored. **No stack
passes; the hand-guide line is closed for good.**

## Gates

| Gate | Result |
| --- | --- |
| G1 data | pass: 36 + 26 + 12 + 33 + 3 + 11 parsed rows; `env.json`; one process at a time |
| G2 Phase A | not fired: no share is 3/3 on `deep-win`; exact censors recorded |
| G3 Phase B | pass (no cap): 2× threshold applied exactly; cap 16 reaches no proof |
| G4 Phase C | pass (closed): 2× `core` threshold applied exactly; best 1.22× |
| G5 soundness | pass: 26/26 Phase A and 11/11 Phase C D14 accepted; `make test`, `make lint`, `make test-full`, `make veto` green; new flags default off and trajectory-neutral when off |
| G6 docs | pass: no GPL dependency, raw transcript, provari artifact, proof file or `__pycache__` |
