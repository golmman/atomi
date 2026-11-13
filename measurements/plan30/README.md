# Plan 30 measurements — MG15 declined (item 37, measurement only)

Plan: [`plan30.md`](../../docs/plans/prover/plan30.md). Report:
[`report30.md`](../../docs/plans/prover/report30.md). Host, head, crate pin and
gate verdicts: [`env.json`](env.json). Parsed results:
[`probe.json`](probe.json). The repo is dirty on top of `atomi@c77f40e`; the
owner commits.

**No engine change.** This directory holds one scratch crate against the
published MIT crate `atomic-movegen = "=2.7.0"` (never linked into `atomi`) and
the runs that reproduce plan30 §1. The stop rule was declared before the
numbers (plan30 §1): *no-go if MG15 ≥ 1.0× the build+`outcome()` cost on
`built-ish`*. Measured **1.147×** (plan session 1.13×): **no-go**.

## Files

| File | Contents |
| --- | --- |
| `probe/Cargo.toml`, `probe/Cargo.lock` | the scratch crate, verbatim from the plan session (`=2.7.0`, `--release`, `lto`, `codegen-units 1`) |
| `probe/src/main.rs` | the plan-session probe, **verbatim** (sha256 `584a3514…`) |
| `probe/src/bin/api_costs.rs` | execution-session addition: the §1 rows `main.rs` does not time (MG17 hash / `post_move`, `child_outcome`, `has_legal_move_after`), each in a node-loop and a flattened shape |
| `probe.json` | parsed medians + every raw run, corpus facts, derived amortised figure, verdict |
| `env.json` | host, head, crate checksum, probe binary sha256s, gate verdicts |

Raw transcripts are not committed (AGENTS.md §Measurements); the parsed runs
are in `probe.json`.

## Corpus and harness

1,248 positions from 12 xorshift play-outs of 40 plies per bench FEN, 39,665
legal moves, **7,864 built-ish** (19.8 %): capture, promotion, en passant,
castling or a commoner move — atomi's `needs_board` minus the quiet-check
prediction. Per variant: one `StateInfo::populate_state` + `EffectContext::new`
per node, best of 5 timed passes over the whole corpus, checksum black-boxed.
Five runs, one process at a time, same session.

## `probe/src/main.rs` — the deciding rows (ns per built child, median of 5 runs)

| Class | n | today (`clone`+`do_move`+`outcome()`+`Legal` hash) | MG15 (`hash_and_rule50_after`+`terminal_after`) | build only | ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| built-ish | 7,864 | **57.5** | **65.8** | 32.0 | **1.147** |
| of which captures | 3,957 | 64.8 | 83.3 | 42.4 | 1.286 |
| quiet | 31,801 | 42.6 | 44.6 | 16.4 | 1.038 |

Floors over the five runs: built-ish ratio 1.095, captures 1.279, quiet 0.997
— **every run is ≥ 1.0 on the class we build**. Plan session: 58.0 / 65.3 /
43.0, ratio 1.13; reproduction is within host noise.

## `probe/src/bin/api_costs.rs` — the rest of the §1 table (ns per built child, median of 3 runs)

| Variant | node-loop | flattened |
| --- | ---: | ---: |
| today (build+`outcome()`+hash) | 55.1 | 43.5 |
| `hash_and_rule50_after` (MG17 hash) | 18.2 | 10.6 |
| `ctx.post_move` (MG17 `analyze`) | 36.2 | 26.9 |
| `ctx.terminal_after` (MG15 body) | 62.2 | 53.6 |
| MG15: hash + `terminal_after` | 71.5 | 61.4 |
| `ctx.child_outcome` | 66.4 | 58.4 |
| `ctx.has_legal_move_after` | 61.8 | 54.1 |
| **ratio MG15 / today** | **1.294** | **1.423** |

Two readings, both hostile to MG15:

- **`terminal_after` alone ≈ today's whole build** (62.2 vs 55.1 node, 53.6 vs
  43.5 flat): the view movegen inside `terminal_from` is not cheaper than
  `do_move` + the board movegen, so "no board write" buys nothing here.
- **The amortised form is `terminal_after`.** `terminal_from` is private, so
  `post_move` + terminality always pays `analyze` twice; and `terminal_from` is
  the single implementation behind `terminal_after`, so the best amortised
  composite *is* `terminal_after` — there is no cheaper board-free terminality
  to be had today. The plan's 48.0 ns estimate (`analyze` 18.1 + `terminal_from`
  29.9) is reproduced as a *derived* figure only: `post_move` + (`terminal_after`
  − `post_move`) = `terminal_after`.

Caveats: not atomi's tree (random play-outs); no `material_draw` clause in the
"today" column (bit tests, ~2 ns, and MG15 needs atomi's own clause anyway,
`report2` §5.1); per-call ns depend on loop shape and code layout (that is why
both shapes are kept) — only the **sign** is shape-stable, and it is negative in
every shape and every class.

## Reproduce

```
cd measurements/plan30/probe
cargo run --release --offline --bin probe30     # the deciding rows
cargo run --release --offline --bin api_costs  # the rest of the table
```

Needs the registry copy of `atomic-movegen 2.7.0` (checksum in `env.json`);
nothing from that crate is linked into `atomi`.
