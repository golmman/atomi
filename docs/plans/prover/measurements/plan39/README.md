# Plan 39 measurements — item 52 Phase B (SHORT-PASS), Phase 0

Plan: [`plan39.md`](../../plan39.md). Report:
[`report39.md`](../../report39.md). Host, binaries and pins:
[`env.json`](env.json). One process at a time; the control is the same binary
with no `--max-length`.

| File | Contents |
| --- | --- |
| `runs.json` | the parsed `deep-win` runs: same-session unbounded control and `L = 24`, plus the Phase 0 `L = 12/14/16/18` probes at 5 M and 120 s |
| `run.sh` | the exact commands for `runs.json` |
| `env.json` | host, binary sha256, pins |

There are no proof files: no new proof was claimed and the Phase 0 stop rule
fired, so Phase 1 (the deepening driver) was not implemented.

## Reproduce

    cargo build --release
    docs/plans/prover/measurements/plan39/run.sh

The 120 s rows take ≈ 6 min in total.

## What the measurements found

- **Same-session reference (G-39b control).** The unbounded control is
  tree-identical to G-38a: 5,803,257 expansions / 155,236,740 `child_evals` /
  length 36, wall **52.184 s**. The single `L = 24` pass is exactly the Phase A
  row: 4,109,113 / 109,715,277 / length 24, wall **37.190 s**.
- **Every small-`L` pass is more expensive than `L = 24`.** The best wall is
  `L = 16` at **45.500 s** (1.21× its `child_evals`, 1.22× its wall;
  `L = 14` is 45.526 s). `L = 12` is 83.136 s and `L = 18` is 65.820 s.
  All four exceed both the `child_evals` and the wall of the `L = 24` pass.
- **The cumulative bar fails by > 11×.** The four cold passes cost
  650,951,995 `child_evals` and 239.982 s against the bars 54,857,638 and
  18.595 s.
- **Stop rule: STOP.** No deepening schedule can reach `L = 24` faster when
  every prefix pass alone costs more than the whole `L = 24` pass. Phase 1 was
  not built; the driver stays absent (opt-in, never a default either way).
- **5 M tree identity.** `L = 18` is tree-identical to the control
  (5,000,034 / 168,910); `L = 12/14/16` differ slightly
  (5,000,003/5,000,021/5,000,037 `child_evals`), so a bound below 18 already
  changes the early tree, not only the tree after a proof exists.

## Limits

- The Phase 0 passes are **cold** single passes. A real deepening driver would
  retain proofs, refutations and TT state, so its cumulative work is not the
  literal sum of these cold runs. The plan's stop rule uses the cold passes as
  the price of each prefix; the observed gap is so large (the cheapest prefix
  alone is 1.22× the control's wall) that state retention cannot reverse it.
- The "or one proves" clause in plan39 §2 is literally satisfied (all four
  prove), but the plan's absolute clause — any single `L < 20` pass exceeding
  the `L = 24` pass ⇒ stop — is decisive. Every one does. The tension is
  recorded in report39 §4.
- No `bench.md` row, no default change, no `T_4` claim, no provari source.
