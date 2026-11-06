# plan1 measurements

Provenance for `bench.json`, `pernode.json` + `pernode.csv` (only parsed
files are committed; no raw transcripts).

| Item | Value |
| --- | --- |
| Date / host | 2026-11-06 · `ae54d05b36f8` |
| Platform / CPU | Linux · AMD Ryzen 9 5950X 16-Core, 32 logical CPUs |
| rustc | rustc 1.99.1 (58cf8f9c3 2026-10-13) |
| atomic-movegen | 2.4.0 (Cargo.lock) |
| Repo state | HEAD `0c5a882ef9ab93c0d48d6a3f5176cfb64a0a78bb`, **dirty** (the owner has not committed; recorded honestly) |
| atomi | `atomi 0.1.0 (0c5a882…)`, sha256 `7e2e0c20…f7381f`, `target/release/atomi` |
| Reference oracle | provari 0.1.0, pin `48686bc6c786d6ea70568f0b2ad4002cf0520206`, `/tmp/atomi-oracle/provari/target/release/provari` |
| Stretch oracle | provari 0.2.0, pin `224ce85dad083b1bc8fe429a9e4349de956f5305`, `/tmp/atomi-oracle/provari-v0.2.0/target/release/provari` |
| Legacy binary | atomic_solver pin `5f85133ea9d751368a682e53e354bd3fc0016d00`, `/tmp/atomic_solver-5f85133/target/release/atomic_solver` |
| Corpus | `bench/corpus.json`, sha256 `9cd0169a…47d0e` (verified by the driver) |

Commands (one process at a time, nothing else on the host):

    make veto
    python3 bench/run_bench.py --provari /tmp/atomi-oracle/provari/target/release/provari \
      --provari-commit 48686bc… --provari-stretch /tmp/atomi-oracle/provari-v0.2.0/target/release/provari \
      --provari-stretch-commit 224ce85… --legacy-solver /tmp/atomic_solver-5f85133/target/release/atomic_solver \
      --legacy-commit 5f85133… --legacy-stack default= \
      --legacy-stack "opt=--move-class-pn --quiet-grouping --eval-pn or" \
      --atomi target/release/atomi --atomi-stack m0= \
      --cases core,1eb80b92f92c,db6a741c723f --repeats 3 \
      --proof-check --provari-check /tmp/atomi-oracle/provari/target/release/provari \
      --out docs/plans/prover/measurements/plan1/bench.json
    python3 bench/run_pernode.py --atomi target/release/atomi \
      --cases core,1eb80b92f92c,db6a741c723f --evals 1000000,10000000 \
      --repeats 3 --out docs/plans/prover/measurements/plan1/pernode.json

Reproduce: `make oracle` builds both provari pins under `/tmp/atomi-oracle`;
the legacy binary is built from `git archive` of pin 5f85133 (see
PROVENANCE.md); then `make bench` / `make pernode`.
