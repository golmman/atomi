# Bench drivers

`run_bench.py` measures the sides of `docs/plans/prover/bench.md` on one
host, one process at a time, and writes parsed rows + the derived ledger
(factors, shared-win geomean, coverage). Provenance:
`run_bench.py` is `from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/measurements/bench/run_bench.py`
(adapted — see `PROVENANCE.md`). The corpus `corpus.json` is verbatim
`atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/provari/measurements/plan1/corpus.json`,
sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`
(the driver refuses any other digest).

Sides: `provari-<guide>` (reference), `provari-v0.2.0-<guide>` (stretch
block only), `atomi:<stack>` (JSON parser: found/exit/child_evals/
expansions/ms), `atomic_solver:<stack>` (legacy `--outcome-only` parser).

Usage:

    python3 bench/run_bench.py --provari /tmp/atomi-oracle/provari/target/release/provari \
        --provari-commit 48686bc --atomi target/release/atomi --atomi-stack m0= \
        --legacy-solver /tmp/atomic_solver-5f85133/target/release/atomic_solver \
        --legacy-commit 5f85133 --legacy-stack default= \
        --legacy-stack "opt=--move-class-pn --quiet-grouping --eval-pn or" \
        --provari-stretch /tmp/atomi-oracle/provari-v0.2.0/target/release/provari \
        --provari-stretch-commit 224ce85 --proof-check \
        --provari-check /tmp/atomi-oracle/provari/target/release/provari \
        --cases core --repeats 3 --out results.json

`--dry-run` prints commands only. Note (finding 12): under `--outcome-only`
the legacy binary prints no `pre_exit:` line, so `exit` stays null on
control rows and control censoring comes from the `outcome:` line.

`run_pernode.py` measures per-node cost (`--evals` caps, medians over runs
that reached the cap, `m1_reference` on `core`), JSON + CSV out. Sides:
`--atomi NAME=PATH` (repeatable; a bare `PATH` is the side `atomi`) at every
`--memory` value (comma list), and optionally `--legacy-probe PATH
--legacy-commit SHA` (the legacy plan3 front-end probe, fixed 32 MB table).
Per (case, cap) each repeat runs every side once, one process at a time.
The `gate` block compares sides `m1`, `m0` and `legacy-fe` on `core` at the
largest cap and 192 MB (plan2 G1).
