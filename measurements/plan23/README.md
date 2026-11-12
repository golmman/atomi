# Plan 23 measurements — item 32, proof-retention share (RET-V1)

Plan: [`plan23.md`](../../docs/plans/prover/plan23.md). Report:
[`report23.md`](../../docs/plans/prover/report23.md). Host and binary:
[`env.json`](env.json) (`atomi` sha256 `c25b1f6a…`, git `e681d9f` + dirty
tree). Phase A ran on the pre-default-change binary (`65ddc4a1…`) with an
explicit `--proof-share`; Phase C and `env.json` are the shipped binary.

## Files

| File | Contents |
| --- | --- |
| `phase_a.py` | Phase A driver: 27 timed runs (3 cells × seeds 0–2 × shares 25/50/75) plus one 600 s D14 proof run per proved `deep-win` row; `--recompute` re-evaluates the fixed selection rule from `sweep.json` |
| `sweep.json` | Phase A rows, proof runs, checker verdicts, summary and selection |
| `gate.py` | Phase C driver: same-session provari reference (3 repeats/cell), 9 timed candidate runs at the default share, one 600 s D14 proof run per proved row |
| `gate.json` | Phase C rows, reference medians, factors, guards and the `T_3` verdict |
| `env.json` | Host, CPU, atomi binary sha256, provari version/commit |

Run (one process at a time; Phase A ≈ 55 min, Phase C ≈ 12 min):

    python3 measurements/plan23/phase_a.py \
        --atomi target/release/atomi \
        --provari-check /tmp/atomi-oracle/provari/target/release/provari
    python3 measurements/plan23/phase_a.py --recompute
    python3 measurements/plan23/gate.py \
        --atomi target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari

## Phase A — share sweep (`--guide hand`, 192 MB, 120 s)

| Share | `core` | `easy-win` | `deep-win` | `deep-win` D14 |
| ---: | --- | --- | --- | --- |
| 25 (control) | 3/3, 8.524 s | 3/3, 0.929 s | **0/3** (`proof_full`) | — |
| 50 | 3/3, 8.595 s | 3/3, 0.923 s | **2/3** (seed 1 `proof_full`) | 2/2 accepted |
| 75 | 3/3, 8.687 s | 3/3, 0.920 s | **3/3** (113.477 / 105.297 / 77.875 s) | 3/3 accepted |

`deep-win` proof runs (600 s, same share/seed, `child_evals` identical):

| Share | Seed | Wall | `child_evals` | Entries | Attacker | Defender | Positions | Length | Checkers |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 50 | 0 | 102.309 s | 134,818,742 | 5,620,029 | 5,595,484 | 24,545 | 76,405 | 36 | accept / accept |
| 50 | 2 | 111.744 s | 144,507,930 | 5,540,793 | 5,511,445 | 29,348 | 105,502 | 36 | accept / accept |
| 75 | 0 | 113.477 s | 155,236,740 | 5,716,874 | 5,687,070 | 29,804 | 93,092 | 36 | accept / accept |
| 75 | 1 | 105.297 s | 146,712,125 | 5,275,565 | 5,251,157 | 24,408 | 72,787 | 37 | accept / accept |
| 75 | 2 | 77.875 s | 107,672,243 | 4,179,903 | 4,160,055 | 19,848 | 60,691 | 29 | accept / accept |

**Selection (fixed before the data).** 50 fails (`deep-win` 2/3). 75 passes:
`deep-win` 3/3 accepted, `core`/`easy-win` 3/3 with median-wall ratios
1.019 / 0.990 vs the 25 % control (guard ≤ 1.10). **Chosen share: 75 %.**

## Phase C — the `T_3` gate (default share 75, 120 s)

Reference: provari `--guide hand --seconds 10 --memory 192`, same session,
3 repeats/cell.

| Cell | Candidate walls | Median | Reference | Factor | Control ratio | D14 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `core` | 10.190 / 8.327 / 8.211 | 8.327 s | 0.531 s | **15.68** | 0.977 | 3/3 accepted |
| `easy-win` | 0.798 / 0.910 / 1.439 | 0.910 s | 0.133 s | **6.84** | 0.980 | 3/3 accepted |
| `deep-win` | 112.144 / 105.316 / 77.783 | 105.316 s | 6.661 s | **15.81** | 0.878 | 3/3 accepted |

All nine timed runs proved; all nine D14 proof runs matched `child_evals` and
were accepted by `atomi validate` and `provari check`. `deep-win` proofs:
93,092 / 72,787 / 60,691 positions, lengths 36 / 37 / 29. Geomean over the
three claimed cells at Cap 120 s: **11.93**. `T_3` **reached**; guards pass.
