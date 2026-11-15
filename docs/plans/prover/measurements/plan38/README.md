# Plan 38 measurements — item 52 Phase A (bounded-length pass)

Plan: [`plan38.md`](../../plan38.md). Report:
[`report38.md`](../../report38.md). Host, binaries and pins:
[`env.json`](env.json). One process at a time; the control is the same binary
with no `--max-length`.

| File | Contents |
| --- | --- |
| `review_probe.json` | the kill-review probe: `--max-depth` is a censor, not a bounded pass |
| `runs.json` | the parsed `deep-win` runs: control and `L = 20/22/24` at 5 M and 120 s |
| `bounded_deepwin.validation.json` | the `L = 20/22/24` deep-win proofs: sha256 + both checker results (the 5.6–7.4 MiB proof files stay in `/tmp`) |
| `bounded_rule50.jsonl` | a small bounded proof (`--max-length 2`, rule50 edge) |
| `bounded_rule50.validation.json` | that proof accepted by `atomi validate` and `provari check` |
| `run.sh` | the exact commands for `runs.json` |

## Reproduce

    cargo build --release
    docs/plans/prover/measurements/plan38/run.sh

The 120 s rows take ≈ 4 min in total. The bounded proofs are reproduced by

    atomi prove "<deep-win FEN>" --guide hand --memory 192 --seed 0 \
      --seconds 600 --max-length 24 --proof-out /tmp/L24.jsonl
    atomi validate /tmp/L24.jsonl
    /tmp/atomi-oracle/provari/target/release/provari check /tmp/L24.jsonl

and the small rule50 proof by

    atomi prove "k7/8/8/8/8/7Q/8/K6R w - - 0 1" --guide hand --memory 192 \
      --seconds 60 --max-length 2 --proof-out /tmp/bounded.jsonl

## What the measurements found

- **G-38a PASS.** The unbounded control is tree-identical to the pre-plan
  search: 5,000,034 `child_evals` / 168,910 expansions at 5 M, and
  5,803,257 / 155,236,740 / length 36 at 120 s.
- **G-38b PASS.** The `L = 20/22/24` deep-win proofs and the rule50 bounded
  proof are accepted by both checkers.
- **G-38c PASS.** `Value::Bounded` is never stored; the bounded algebra never
  turns a bounded child into a refutation.
- **G-38d PASS (positive).** On `deep-win`, `L = 20/22/24` prove at 120 s with
  lengths 20/22/24 against the control's 36. `L = 24` is also faster
  (37.5 s vs 53.3 s); `L = 20/22` are slower (58.4/59.2 s).
- **G-38e PASS.** `make test` and `make test-full` (including `make veto`).

## Limits

- The 5 M rows are tree-identical to the control because the unbounded search
  has not yet found a proof there; the bound only changes the tree once a
  too-long proof exists. The 120 s rows are the informative ones.
- The bounded pass is a measurement instrument, not a default: no `bench.md`
  row, no default change, no `T_4` claim.
- The first implementation applied the bound only at frame entry and after a
  child return; the exact null/sibling replay could then reinstall a too-long
  proof and the selector saw a different value from the bounded algebra. The
  committed version re-applies the bound after those auxiliary passes. The
  earlier (uncommitted) run timed out at all three `L`; the committed run
  proves. This is a real implementation sensitivity, recorded in report38 §4.
