# Plan 29 measurements — EFFECT-CHILD-V1 (item 36, stop rule)

Plan: [`plan29.md`](../../docs/plans/prover/plan29.md). Report:
[`report29.md`](../../docs/plans/prover/report29.md). Host, binaries and gate
verdicts: [`env.json`](env.json). The repo is dirty on top of
`atomi@5b8d7a8`; the owner commits.

The candidate is a **measurement-only** build: one `EffectContext` per
expanded node plus per-child counters (`--stats` only), reverted after Phase A.
The only committed code change is
[`tests/effects_differential.rs`](../../tests/effects_differential.rs), which
pins `atomic-movegen 2.6.0`'s MG3 surface against our own frozen fixture.

## Files

| File | Contents |
| --- | --- |
| `probe.py` | Phase A driver: `core`/`easy-win`/`deep-win` at 1 M, `deep-win` at 5 M, `--guide hand` and `--guide none`, control + counter build, two runs per config |
| `phase_a.json` | Parsed counters, the `child_evals`/`expansions` identity check and the derived shares |
| `env.json` | Host, head, both binary sha256s, budgets, gate verdicts |

`probe.py` needs the Phase A counter build (reverted, see `report29` §5 for
the patch shape): without it `--stats` has no `effect_*` keys.

## Phase A (counters only, `--stats`)

Per-child counters over the children of every expanded node: `needs_board`
(children `Mover::effect` marks for a board today), `decided` (children
`rules::status_after` decides from the effect: extinction / non-check clock),
`fallback` (the rest), `terminal` (of the fallback children, those the built
path finds `Won`/`Failed` — the ceiling for any effect-based status),
`no_qp` (fallback children with no queen and no pawn left, i.e. where our
checker-material clause needs the child), `boards_saved` (decided ∧ no guide
board read, i.e. C3's saving under `--guide none`).

| Cell | Cap | Guide | `needs_board` | decided | share | ceiling | `no_qp` | tree |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `core` | 1 M | hand | 298,553 | 0 | **0.0000** | 0.0009 | 0 | identical |
| `easy-win` | 1 M | hand | 191,732 | 0 | **0.0000** | 0.0014 | 0 | identical |
| `deep-win` | 1 M | hand | 178,317 | 0 | **0.0000** | 0.0004 | 0 | identical |
| `deep-win` | 5 M | hand | 1,067,066 | 0 | **0.0000** | 0.0008 | 0 | identical |
| `core` | 1 M | none | 259,981 | 2,438 | 0.0094 | 0.0017 | 0 | identical |
| `easy-win` | 1 M | none | 179,569 | 2,639 | 0.0147 | 0.0017 | 0 | identical |
| `deep-win` | 1 M | none | 159,594 | 3,069 | 0.0192 | 0.0040 | 0 | identical |
| `deep-win` | 5 M | none | 861,939 | 16,696 | 0.0194 | 0.0061 | 0 | identical |

The declared stop rule was **< 10 % of built children decided by the effect**;
every hand-guide cell is **0.0 %** and every bare cell **0.9–1.9 %**. All
decisions are extinctions (no clock decision fired: the clock needs a
surviving queen or pawn *and* `rule50_after >= 100`). Under the hand guide the
tree never even contains one: 0 of 394,969 captures at 5 M has the enemy
commoner inside the blast, because the guide's one-ply settle proves those
children a ply earlier.

Two facts make the zero structural rather than accidental:

- the **ceiling** is 0.08 % (869 of 1,067,066): even an effect that decided
  *every* terminal child perfectly could not reach 10 %;
- `no_qp = 0` everywhere: our checker-material clause never needs the child
  in these trees, so 100 % of the fallback is the mate/stalemate question —
  the child-view ask now in
  [`handoff2.md`](../../docs/plans/movegen/handoff2.md).

Instrumentation is observation only: `child_evals` and `expansions` are
identical between the control and the counter build on all eight configs, and
the counters reproduce exactly on a second run of every config.

## What was not run

G-E1 (5 M wall gate), the 120 s matrix, D14 and the `bench.md` row: stopped by
the stop rule before Phase B. No wall number is claimed and no bench row is
added.

## Reproduce

    git archive HEAD | tar -x -C /tmp/atomi-plan29-control
    cargo build --release --manifest-path /tmp/atomi-plan29-control/Cargo.toml
    # re-apply the Phase A counter patch (report29 §5), rebuild, then
    python3 measurements/plan29/probe.py \
        --control /tmp/atomi-plan29-control/target/release/atomi \
        --candidate target/release/atomi \
        --out measurements/plan29/phase_a.json
