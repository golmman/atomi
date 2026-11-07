# plan2 measurements

Parsed results for [`report2.md`](../../report2.md) (M1, per-node cost).
Only drivers and parsed files are committed: no transcripts, no proofs, no
binaries, no `__pycache__`. Environment and binary digests:
[`env.json`](env.json).

| File | What | Driver |
| --- | --- | --- |
| `pernode.json` + `pernode.csv` | M1 per-node gate: m0 / m1a / m1b / m1 / legacy-fe × 3 cells × 1 M, 10 M × 192, 32 MB, 3 interleaved repeats; `gate` block | `bench/run_pernode.py` |
| `ab.json` | M4 `unsafe` keep rules: `m4_round1_core10M`, `m4_round2_core10M` (huge-page base), `m4_lazy_mate_in_1` | `run_ab.py` |
| `wallguard.json` | M2 wall guard: `easy-win` proved to the end, m0 vs m1, 3 repeats | `run_ab.py` |
| `sweep.json` | M3 10 M fixture sweep, m1 vs m0, both checkers on every proof | `run_sweep.py` |
| `bench.json` | M5 bench row (provari v0.1.0 hand reference, v0.2.0 stretch, atomi m1 with D14 proof check) | `bench/run_bench.py` |

| Item | Value |
| --- | --- |
| Date / host | 2026-11-06 · `ae54d05b36f8` (Ryzen 9 5950X container, 4 vCPU, THP `madvise`) |
| rustc | 1.99.1 (58cf8f9c3 2026-10-13) |
| Repo state | HEAD `a4c135d6ee1956588e01733bd101f6373e7d2e72`, **dirty** (plan2 work uncommitted; the owner commits) |
| Sides | `m0` = `git archive 9aa96c9`; `m1a` = Phase A gate tree; `m1b` = Phase B tree without huge pages; `m1` = final tree; `legacy-fe` = legacy plan3 front-end probe (fixed 32 MB scratch table) |
| Oracle | provari 0.1.0 `48686bc6…` (veto + reference), provari 0.2.0 `224ce85d…` (stretch) |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` |

Commands (one process at a time, nothing else running on the host):

    python3 bench/run_pernode.py --atomi m0=/tmp/atomi-m0/target/release/atomi \
      --atomi m1a=/tmp/atomi-m1a/target/release/atomi --atomi m1b=/tmp/atomi-m1/target/release/atomi \
      --atomi m1=target/release/atomi --legacy-probe /tmp/plan2-legacy-probe/probe \
      --legacy-commit 5f85133ea9d751368a682e53e354bd3fc0016d00 \
      --cases core,1eb80b92f92c,db6a741c723f --evals 1000000,10000000 \
      --memory 192,32 --repeats 3 --out docs/plans/prover/measurements/plan2/pernode.json
    python3 docs/plans/prover/measurements/plan2/run_ab.py --label m4_round1_core10M \
      --side m1=/tmp/atomi-m1/target/release/atomi --side nopf=/tmp/plan2-ab-nopf/target/release/atomi \
      --side hugepage=/tmp/plan2-ab-hp/target/release/atomi --side eager=/tmp/plan2-ab-eager/target/release/atomi \
      --case core --flags "--evals 10000000 --memory 192 --seconds 600" --repeats 5 \
      --out docs/plans/prover/measurements/plan2/ab.json
    python3 docs/plans/prover/measurements/plan2/run_ab.py --label m4_round2_core10M \
      --side hugepage=/tmp/plan2-ab-hp/target/release/atomi --side hp_nopf=/tmp/plan2-ab-hp-nopf/target/release/atomi \
      --side hp_eager=/tmp/plan2-ab-hp-eager/target/release/atomi --side m1=/tmp/atomi-m1/target/release/atomi \
      --case core --flags "--evals 10000000 --memory 192 --seconds 600" --repeats 7 \
      --out docs/plans/prover/measurements/plan2/ab.json
    python3 docs/plans/prover/measurements/plan2/run_ab.py --label m4_lazy_mate_in_1 \
      --side m0=/tmp/atomi-m0/target/release/atomi --side hugepage=/tmp/plan2-ab-hp/target/release/atomi \
      --side hp_eager=/tmp/plan2-ab-hp-eager/target/release/atomi --side m1=/tmp/atomi-m1/target/release/atomi \
      --fen "6rk/8/8/8/8/8/8/6QK w - - 0 1" --flags "--memory 192 --seconds 600" --repeats 11 \
      --out docs/plans/prover/measurements/plan2/ab.json
    python3 docs/plans/prover/measurements/plan2/run_ab.py --label m2_wall_guard_easy_win \
      --side m0=/tmp/atomi-m0/target/release/atomi --side m1=target/release/atomi \
      --case 1eb80b92f92c --flags "--seconds 600 --memory 192" --repeats 3 \
      --out docs/plans/prover/measurements/plan2/wallguard.json
    python3 docs/plans/prover/measurements/plan2/run_sweep.py --atomi target/release/atomi \
      --baseline-atomi /tmp/atomi-m0/target/release/atomi \
      --provari /tmp/atomi-oracle/provari/target/release/provari \
      --out docs/plans/prover/measurements/plan2/sweep.json
    python3 bench/run_bench.py --provari /tmp/atomi-oracle/provari/target/release/provari \
      --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
      --provari-stretch /tmp/atomi-oracle/provari-v0.2.0/target/release/provari \
      --provari-stretch-commit 224ce85dad083b1bc8fe429a9e4349de956f5305 \
      --atomi target/release/atomi --atomi-stack 'm1=' \
      --cases core,1eb80b92f92c,db6a741c723f --repeats 3 \
      --proof-check --provari-check /tmp/atomi-oracle/provari/target/release/provari \
      --out docs/plans/prover/measurements/plan2/bench.json

The A/B binaries are scratch copies of the final tree with one change
each (huge-page base for round 2), built in `/tmp/plan2-ab-*`; they share
the final tree's counters exactly (checked on `core` at 1 M before the
timed runs). `m1b` in `pernode.json` is `/tmp/atomi-m1` (the tree before the
huge-page site landed).

Reproduce: `make oracle && make oracle-stretch`; build m0 from
`git archive 9aa96c9`; build the legacy probe per `env.json`; then the
commands above.
