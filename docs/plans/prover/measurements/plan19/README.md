# plan19 measurements

Plan: [`../../plan19.md`](../../plan19.md) (item 26, decisions D12, D14, D21,
D26, D30, D32, D33). Report: [`../../report19.md`](../../report19.md).
Parsed output only: no raw transcript, provari artifact, proof file or
`__pycache__` is committed.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, binary/corpus sha256, provari version, cells, stacks, budgets |
| `p1.txt` | the analysis-1 §4 "every class `dn=1`" parameter file |
| `p_band64.txt`, `p_band128.txt`, `p_band192.txt` | the R2 EF-PN band edges probed on `core` |
| `a.json` | Phase A (G2): the plan's 20 M child-evals instrument, 3 cells × seeds 0-2 × {bare, hand, analysis flag stack} + the analysis §4 reproduction row per cell |
| `probe.json` | Phase B: the R2 band-edge probe and the R1/R4-R5 ablation arms on `core`, seed 0, tree only |
| `gate.json` | Phase C (G3): 3 cells × seeds 0-2 × {bare, hand, numbers, numbers_auxoff} at 300 M child evals, plus the per-cell medians and cuts |
| `tree.json` | Phase C tree cut: 3 cells × seeds 0-2 × {hand, flag stack, numbers, adopted}, auxiliary passes off, 2 G child evals, plus the medians and cuts |
| `matrix.json` | Phase D: the 120 s budget matrix (bare, product, adopted, adopted aux-off) + the D14 rows |
| `tier2.json` | the 10 s column: the adopted default (3 cells × seeds 0-2) and the pinned provari hand reference (3 repeats per cell) |
| `run_phase19.py` | stdlib-only driver (`env`, `a`, `probe`, `gate`, `tree`, `matrix`, `tier2`, `d14`, `all`; `--only`/`--cells` re-run a subset) |

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan19/run_phase19.py --phase all
make test
make lint
make test-full
```

One atomi process at a time. `--memory 192` everywhere. Budgets: Phase A
`--evals 20000000` (the plan's instrument); the gate
`--evals 300000000 --seconds 240` (a proof-reaching child-evals budget, see
`report19.md` §G2); the matrix `--seconds 120`. Proof files live under
`/tmp/atomi-plan19` and are deleted after validation.

## The default after plan19

`--guide hand` = the product stack + `fresh-class` + `evasions-count` +
`unlikely-exact` (R1+R3+R6). The pre-plan19 product stack is
`--guide hand --guide-off fresh-class,evasions-count,unlikely-exact`; the
declared six-mechanism stack is `--guide hand --guide-on numbers`. The
`tree.json`/`gate.json` rows were measured before the default change, so
their `hand` stack is the product one.

## Why the gate is not at 20 M child evals

At a fixed `--evals` cap a censored run spends ~`child_evals/32` expansions
on every stack (measured: `core` bare 630,056, product hand 648,828, numbers
600,894 at 20 M), because expansions are bounded by the children per expanded
node, not by the guide. A 2× tree cut is only visible when a stack *proves*,
so the gate budget must be above the cheapest proof. Phase A keeps the 20 M
instrument and shows both facts.
