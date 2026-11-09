# plan5 measurements

Plan: [`../../plan5.md`](../../plan5.md) (item 5 / M4 probe P0). Everything
here is parsed output; no raw transcripts, provari artifacts or
`__pycache__` are committed. Temporary proof JSONL written by the probe were
removed after the checkers ran; their counters and verdicts are in
`probes.json`.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus/guidance sha256 |
| `coverage.json` | P0-a: 72 rows (24 cells × seeds 0–2), summary and counts |
| `determinism.json` | P0-b: 10 s plain vs `--proof-out`, plus the work-cap explanation probe |
| `probes.json` | P0-c scale case and P0-d `deep-win` censor classification |
| `run_p0.py` | stdlib-only driver (`env`, `coverage`, `determinism`, `probes`, `all`) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase env
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase coverage
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase determinism
python3 docs/plans/prover/measurements/plan5/run_p0.py --phase probes
```

One atomi process at a time. The auxiliary stack is
`--guide hand --guide-scale 64 --guide-ef-weight 0 --guide-max-sim 1
--guide-max-settle 1`; P0-a/P0-b use `--seconds 10 --memory 192`, seeds 0–2.
The driver only stores the parsed JSON-line fields.

## P0-a — coverage at 10 s (24 cells × seeds 0–2)

| Cell | ref nodes | seed 0 | seed 1 | seed 2 | verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| `core` | 342,289 | censored (10.0 s, 45,244,283 ev) | censored (10.0 s, 44,638,059 ev) | censored (10.0 s, 44,692,577 ev) | censored all |
| `easy-win` | 72,523 | proved (5.42 s, 918,551) | proved (4.51 s, 759,177) | proved (4.66 s, 773,238) | proved all |
| `deep-win` | 4,574,912 | censored (10.0 s, 48,666,913 ev) | censored (10.0 s, 50,058,423 ev) | censored (10.0 s, 49,990,729 ev) | censored all |
| `b9a7ed7eb368` | 13,952,139 | censored (10.0 s, 49,327,710 ev) | censored (10.0 s, 49,201,257 ev) | censored (10.0 s, 49,157,323 ev) | censored all |
| `3004d09cef79` | 18,273,275 | censored (10.0 s, 50,609,765 ev) | censored (10.0 s, 49,884,296 ev) | censored (10.0 s, 49,760,102 ev) | censored all |
| `dd3f89766a40` | 17,714,846 | censored (10.0 s, 51,017,477 ev) | censored (10.0 s, 51,735,542 ev) | censored (10.0 s, 51,274,302 ev) | censored all |
| `dec01` | 28,229 | proved (1.51 s, 347,086) | proved (1.28 s, 278,492) | proved (1.32 s, 303,350) | proved all |
| `dec03` | 7 | proved (0.02 s, 2,079) | proved (0.02 s, 2,091) | proved (0.02 s, 2,108) | proved all |
| `dec04` | 68 | proved (0.01 s, 143) | proved (0.01 s, 152) | proved (0.01 s, 165) | proved all |
| `dec05` | 10,654 | proved (0.13 s, 22,561) | proved (0.19 s, 39,422) | proved (0.22 s, 44,606) | proved all |
| `dec07` | 17 | proved (0.01 s, 93) | proved (0.01 s, 93) | proved (0.01 s, 93) | proved all |
| `dec08` | 3 | proved (0.00 s, 5) | proved (0.00 s, 5) | proved (0.00 s, 3) | proved all |
| `dec09` | 1 | proved (0.00 s, 3) | proved (0.00 s, 7) | proved (0.00 s, 3) | proved all |
| `dec10` | 32,863 | proved (0.25 s, 54,637) | proved (0.28 s, 63,685) | proved (0.23 s, 51,875) | proved all |
| `dec11` | 219 | proved (0.06 s, 9,430) | proved (0.06 s, 8,359) | proved (0.07 s, 9,550) | proved all |
| `dec12` | 3,052 | proved (0.07 s, 11,224) | proved (0.09 s, 14,480) | proved (0.08 s, 12,676) | proved all |
| `dec13` | 9,315 | proved (0.91 s, 128,267) | proved (0.58 s, 83,189) | proved (0.74 s, 105,988) | proved all |
| `dec14` | 2,450 | proved (0.28 s, 42,189) | proved (0.29 s, 41,805) | proved (0.30 s, 44,041) | proved all |
| `dec15` | 13,225 | proved (0.09 s, 14,917) | proved (0.09 s, 14,649) | proved (0.07 s, 10,809) | proved all |
| `dec16` | 87 | proved (0.02 s, 1,910) | proved (0.02 s, 1,874) | proved (0.02 s, 1,867) | proved all |
| `dec17` | 10,118 | proved (0.07 s, 7,973) | proved (0.07 s, 8,354) | proved (0.07 s, 7,495) | proved all |
| `dec18` | 6 | proved (0.02 s, 1,049) | proved (0.02 s, 1,028) | proved (0.02 s, 1,018) | proved all |
| `dec19` | 679 | proved (0.03 s, 1,907) | proved (0.03 s, 1,900) | proved (0.03 s, 1,902) | proved all |
| `dec20` | 418 | proved (0.04 s, 3,717) | proved (0.03 s, 2,844) | proved (0.04 s, 4,435) | proved all |

Parenthesised numbers are wall and `expansions` on proved rows, wall and
`child_evals` on censored rows.

Counts: **57/72 rows proved, 19/24 cells proved on every seed** (the 18
`dec*` cells plus `easy-win`); the five corpus cells `core`, `deep-win`,
`b9a7ed7eb368`, `3004d09cef79`, `dd3f89766a40` are censored on all seeds.
No guidance-set refutation.

- Proved walls: min 0.002 s, median 0.064 s, max 5.421 s — the auxiliary
  stack is **not** near 10 s on proved cells; only `easy-win` spends
  seconds, the rest prove quickly. The 10 s wall is spent on the censored
  cells.
- `easy-win` inflation reproduces report4 exactly: 918,551 / 759,177 /
  773,238 expansions on seeds 0 / 1 / 2 (report4: 918,551 / 759,177 /
  773,238), all far over the D21 limit 145,046.
- `easy-win` lengths are 19 / 19 / 17; the other proved cells are length
  2–26.

## P0-b — `--proof-out` neutrality at 10 s

`easy-win` seed 0 is counter-identical with and without `--proof-out`
(`proved`, 27,242,967 child evals, 918,551 expansions) and equal to its
P0-a row. No censored run wrote a proof file.

`core` seed 0 is censored, and its counters vary across repeats:
45,244,283 / 44,370,650 / 44,587,197 child evals (P0-a / plain / with
proof). **This is a wall-deadline finding, not search nondeterminism**: the
scheduler decides how much work fits in 10 s. Under a work bound the same
cell is exact — `--seconds 600 --evals 40000000`, three runs (two plain,
one `--proof-out`): all `exit=evals`, 40,000,029 child evals, 1,295,213
expansions. `--proof-out` is trajectory-neutral; a censored run writes no
proof.

## P0-c — scale case (D14, 600 s)

Largest proved cell by reference nodes: `easy-win` (72,523). Run with the
same auxiliary stack, seed 0, `--seconds 600`, `--proof-out`:

| Field | Value |
| --- | --- |
| wall | 5.35 s |
| `child_evals` | 27,242,967 (equal to P0-a) |
| length | 19 |
| proof positions | 3,527 |
| proof entries | 65,179 |
| JSONL bytes | 293,459 |
| `proof_full` | false |
| `atomi validate` / `provari check` | valid / valid, 3,527 = 3,527 → **accepted** |

## P0-d — `deep-win` at 120 s

One auxiliary-stack attempt with `--proof-out`, seed 0: **censored,
`deadline`** (`exit=timeout`, no proof file). Counters: 559,984,639 child
evals, 20,167,680 expansions, 1,211,031 proof entries, `proof_full=false`.
The censor is the wall deadline, not output saturation; `deep-win` has no
proof at 120 s.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass**: 72 P0-a rows, each with a verdict; P0-c and P0-d classified |
| G2 determinism | **pass with a recorded finding**: proved runs exact; censored 10 s `core` rows differ only by work completed at the deadline, and the work-capped run is exact; `--proof-out` neutral; censored runs write no proof |
| G3 soundness | **pass**: no refutation; the P0-c proof is accepted by both checkers (3,527 positions) |
| G4 artefacts | **pass**: the listed JSON + driver + this README; no raw transcripts, provari artifacts or `__pycache__`; `make test` / `make lint` green |

This is measurement only: no product or speed claim and no ledger row. The
host is the same as plan4 (`e10de197da58`), and the session reference line
(`core` 0.533 s, `easy-win` 0.137 s, `deep-win` 6.567 s) is unchanged.
