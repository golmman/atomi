# plan21 measurements

Plan: [`../../plan21.md`](../../plan21.md) (item 31, D34). Report:
[`../../report21.md`](../../report21.md). Parsed output only: no raw
transcript, provari artifact, proof file or `__pycache__` is committed.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus sha256, provari version, cells, the stack, the caps |
| `ref.json` | the same-session reference: provari v0.1.0 `--guide hand`, 3 repeats per cell at the canonical 10 s / 192 MB |
| `cap.json` | atomi `--guide hand` (AUX-V1), three cells × seeds 0–2, at `--seconds 600 --memory 192` (no evals cap) |
| `d14.json` | D14 for every proved `cap` row: untimed `--seconds 600 --proof-out` rerun, `child_evals` match, `atomi validate` + `provari check` |
| `mem_ladder.json` | diagnostic, not a row: `deep-win` seed 1 at 384 MB and 768 MB, proof export and both checkers |
| `ledger.json` | factors over the **claimed** cells of the Cap-600 row, the geomean and its cell count |
| `run_phase21.py` | stdlib-only driver (`env`, `ref`, `cap`, `d14`, `ledger`; `--cells` re-runs a subset) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan21/run_phase21.py --phase env
python3 docs/plans/prover/measurements/plan21/run_phase21.py --phase ref
python3 docs/plans/prover/measurements/plan21/run_phase21.py --phase cap
python3 docs/plans/prover/measurements/plan21/run_phase21.py --phase d14
python3 docs/plans/prover/measurements/plan21/run_phase21.py --phase ledger
```

One atomi process at a time, `--memory 192` everywhere. Proof files live under
`/tmp/atomi-plan21` and are deleted after validation.

## What the numbers say

- `core` **16.28** (8.468 s vs 0.520 s) and `easy-win` **6.91** (0.919 s vs
  0.133 s), both D14 3/3 accepted → the Cap-600 row's two claimed factors,
  geomean **10.6** over 2 cells.
- `deep-win` finds a mate on **3/3** seeds (175.408 / 132.185 / 153.740 s) but
  the 192 MB proof table is full on every seed: `proof_full: true`,
  2,831,155 entries, export refused closed (`proof: store-full`, exit 3).
  D14 cannot accept, so the cell is **not claimed** — `23.42` is the wall it
  would carry, never a factor.
- `child_evals` reproduce the plan20 `gate.json` tree rows exactly
  (`core` 12,438,175 / 10,097,122 / 9,846,498; `easy-win` 1,016,547 /
  1,177,648 / 1,803,483; `deep-win` 205,208,471 / 163,653,289 / 188,579,356),
  so the deterministic search is unchanged and the wall differences are host
  noise.

## Why the memory ladder is not a row

The D14 arm must use the same `--memory` label as the timed arm (192 MB).
`mem_ladder.json` only answers *why* the export fails: at 384 MB (94,686
positions, length 35) and 768 MB (74,157 positions, length 37) the seed-1
proof is written and accepted by both checkers. A larger label is a different
bench configuration, so it is recorded as a diagnostic and the claim waits for
item 30.
