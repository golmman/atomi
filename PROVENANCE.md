# Provenance

Every file or fragment copied from `atomic_solver` (own MIT material only)
gets a row here. `verbatim` means byte-identical; `adapted` lists what
changed. Source sha256 is computed at copy time against the pinned commit
`5f85133ea9d751368a682e53e354bd3fc0016d00`.

| Destination | Source | Source sha256 | Date | Mode |
| --- | --- | --- | --- | --- |
| `bench/corpus.json` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/provari/measurements/plan1/corpus.json` | `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e` | 2026-11-06 | verbatim |
| `tests/fixtures/wins.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/fixtures/decisive_positions.txt` | `44ef2df9437b70697ab7ee2e26e7492a1077b23ea3a9cc3e98625358280424c8` | 2026-11-06 | adapted: Win rows only, `;` → TSV with `name/fen/source/legacy_note` columns |
| `tests/fixtures/wins.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/fixtures/decisive_remaining.txt` | `f43f1efebb1d966b00097a7d6f427fd87ceef00e98837ea5471f963f83106814` | 2026-11-06 | adapted: labelled Win rows only, same reshaping |
| `tests/fixtures/wins.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/fixtures/move_order_positions.txt` | `26bfb5ec26a4c1f3936b81515f4dce79fca3096ead0cdc8aab49ef3561d2b0f3` | 2026-11-06 | adapted: Win rows only, same reshaping |
| `tests/fixtures/wins.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/test_ghi.rs` | `3f70ba6e620219c3d84e44b755a4466ef603bcdd19501f36cdce609795b30f90` | 2026-11-06 | adapted: the FEN from the transposition Win claim |
| `tests/fixtures/never_prove.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/fixtures/decisive_positions.txt` | `44ef2df9437b70697ab7ee2e26e7492a1077b23ea3a9cc3e98625358280424c8` | 2026-11-06 | adapted: Loss rows only, TSV with `name/fen/source/reason` columns |
| `tests/fixtures/never_prove.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/fixtures/decisive_remaining.txt` | `f43f1efebb1d966b00097a7d6f427fd87ceef00e98837ea5471f963f83106814` | 2026-11-06 | adapted: the labelled Loss row only |
| `tests/fixtures/never_prove.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/fixtures/move_order_positions.txt` | `26bfb5ec26a4c1f3936b81515f4dce79fca3096ead0cdc8aab49ef3561d2b0f3` | 2026-11-06 | adapted: Loss rows only |
| `tests/fixtures/never_prove.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/test_terminal_outcomes.rs` | `0f2a84f504bd05e97a0230a51b96324f7cee0c59089a3d7501e390360649eadc` | 2026-11-06 | adapted: the Loss and Draw search claims' FENs |
| `tests/fixtures/never_prove.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/test_repetition.rs` | `bd81f976760c11391de2a48a1c0101afed92f6e2db551455581565e4079c80d4` | 2026-11-06 | adapted: the rook-alone FEN (checker material draw, R1) |
| `tests/fixtures/never_prove.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/test_ghi.rs` | `3f70ba6e620219c3d84e44b755a4466ef603bcdd19501f36cdce609795b30f90` | 2026-11-06 | adapted: the FEN from the transposition Loss claim |
| `tests/fixtures/terminal.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/test_position.rs` | `6c1aa4e08867db80d94debe4e3f0f7ca838a8e6a03ef245f626003ca7ecb6c30` | 2026-11-06 | adapted: static-claim FENs re-checked against 2.4.0, attacker-perspective columns |
| `tests/fixtures/terminal.tsv` | `atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:tests/test_terminal_outcomes.rs` | `0f2a84f504bd05e97a0230a51b96324f7cee0c59089a3d7501e390360649eadc` | 2026-11-06 | adapted: FENs re-checked against 2.4.0; material rows constructed per plan1 finding 6 |
| bench/run_bench.py | atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/measurements/bench/run_bench.py | sha256 of source | 2026-11-06 | adapted: provari reference/stretch sides; atomi and legacy named stacks (no implicit default); corpus sha256 gate; --proof-check D14 cells; atomi/legacy parsers; expanded provenance block |
| docs/plans/prover/bench.md §Names and ids, §The metric, §The bench set, §Reading rules | atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md | sha256 of source | 2026-11-06 | verbatim sections with provenance comments |
