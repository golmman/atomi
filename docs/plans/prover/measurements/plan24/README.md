# plan24 measurements

Plan: [`../../plan24.md`](../../plan24.md) (item 29, GROUP-V1). Report:
[`../../report24.md`](../../report24.md). Parsed output only: no raw
transcript, provari artifact, proof file or `__pycache__` is committed.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus sha256, provari version, cells, arms, caps |
| `phase_a.json` | diagnosis on the pre-Phase-B binary: two arms × three cells × seeds 0–2 at `--seconds 120 --memory 192 --stats` — `default` (`--guide hand`, group off) and `group_scored` (`--guide hand --guide-group`, item-15 behaviour) |
| `gate.json` | the Phase C rows on the score-free binary: `group_off` control and `default` candidate (group on, score-free), three cells × seeds 0–2 at 120 s |
| `gate_summary.json` | the gate arithmetic over `gate.json` (same-session control medians, ratios, per-cell pass) |
| `d14.json` | D14 for every score-free row that proved: untimed `--seconds 600 --proof-out` rerun, `child_evals` match, `atomi validate` + `provari check` |
| `run_plan24.py` | stdlib-only driver (`env`, `phase_a`, `gate`, `gate_summary`, `d14`; `--cells` re-runs a subset) |

## Binaries

| Build | sha256 (prefix) | Used by |
| --- | --- | --- |
| counters only (Phase A) | `31432bd77ecb` | `phase_a.json` |
| counters + score-free group, default on (Phase B) | `c532d1eda63d` | `gate.json`, `d14.json` |
| after the Phase D revert (= Phase B reverted, counters kept) | `31432bd77ecb` | the shipped tree |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan24/run_plan24.py --phase env
python3 docs/plans/prover/measurements/plan24/run_plan24.py --phase phase_a
# Phase B (score-free members + `product().group = true`), rebuild, then:
python3 docs/plans/prover/measurements/plan24/run_plan24.py --phase gate
python3 docs/plans/prover/measurements/plan24/run_plan24.py --phase d14
# Phase D revert, rebuild, then re-derive the arithmetic (no reruns):
python3 docs/plans/prover/measurements/plan24/run_plan24.py --phase gate_summary
```

One atomi process at a time, `--memory 192` everywhere. Proof files live under
`/tmp/atomi-plan24` and are deleted after validation.

## What the numbers say

- **The plain fraction is large.** In the group arm, plain moves are 38 % of
  all children on `core` (4.21 M of 11.12 M `child_evals`), 47 % on
  `easy-win` and 44 % on `deep-win`; 38 % of expansions install a group
  (`group_children` / `expansions`).
- **Grouping scored (item 15) is a wash on the small cells and a loss on
  `deep-win`:** `core` 9.115 s median vs 8.535 s (`--guide hand`), `easy-win`
  0.760 vs 0.914 s, `deep-win` 1/3 (seed 0 times out at 120 s; 95.2 / 71.2 s
  for seeds 1–2 vs 105.9 / 78.1 s).
- **Score-free members do not rescue it.** `deep-win` is **0/3** at 120 s
  (censored at 155.0–155.5 M `child_evals`; the control proves at
  146.7–155.2 M), `core` 1.045× (8.86 vs 8.48 s) and `easy-win` 0.863×
  (0.808 vs 0.936 s). The gate fails on `deep-win`; the six proved seeds keep
  D14 6/6 accepted.
- **Per-node cost is flat across all three arms** (26.6–30.1 `child_evals` per
  expansion, 19.5–25.8 µs per expansion), so the wall differences are
  trajectory, not throughput.

## Reading rules

- The gate's control is the **same-session** `--guide-off group` arm on the
  score-free binary; the pre-change default is the same trajectory (the
  `phase_a` `default` arm) to the counter.
- `evals_per_exp` is `child_evals`/`expansions`; `ns_per_exp` is
  `wall_s`/`expansions` and inherits host noise. Ratios are same-session only.
