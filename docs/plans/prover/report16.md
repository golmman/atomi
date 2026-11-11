# Prover Report 16: item 24 Phase A — the differential and the read-only classify

Plan: [`plan16.md`](plan16.md) (item 24, D14/D18/D21/D24/D26/D27/D28/D29).
Executed 2026-11-10. Phase A only: no search, table, guide or default change.
**G3 passes on `core`**: quiet grouping holds and is specific at the first
move-divergent ply. The conformance line continues to Phase B (plan17).

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X, Linux 7.2.13 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `7b2612a` on a dirty tree (plan16 files; agents do not commit), binary sha256 `b4979c04…74388c` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206` |
| Corpus | `corpus-v1`, sha256 `9cd0169a…47d0e` |
| Configuration | provari `--guide hand --seconds 10 --memory 192`; atomi `--guide none --memory 192`, `core`/`easy-win` seed 0 at 120 s, `deep-win` seed 1 at 120 s |

Commands: the `env`, `diff`, `classify`, `flags` phases in
[`measurements/plan16/README.md`](measurements/plan16/README.md), then
`make test`, `make lint`, `make test-full` (slow tier + `make veto`), one
process at a time.

## What was built (read-only)

`atomi classify "<FEN>" [--guide none|hand] [--guide-off LIST]
[--guide-on LIST] [--guide-params PATH] [--guide-scale N]
[--guide-ef-weight N] [--guide-max-sim N] [--guide-max-settle N]
[--guide-aux-budget N] [--guide-sim-plies N] [--guide-group]
[--guide-fresh-dn] [--guide-replay-exact] [--seed N]
[--attacker white|black]`.

The diagnostic lives in `src/classify.rs`; `main.rs` only dispatches, so
`cli.rs` stays the `prove` parser and both files stay under the size
convention. It runs no search and touches no table. It prints one JSON line
with every legal move's `class`, `base_pn`/`base_dn` (the class pair before
`fresh_dn`, `global_scale` and EF-PN), `start_pn`/`start_dn` (after those),
`order` (the search's fresh-node `select` order: metric, class rank, history,
seeded tie-break, movegen order) and the child FEN. `--attacker` defaults to the
side to move; a defender node is classified by passing the root attacker.
The child FEN lets the driver follow a proof edge without a second move
generator. The sim/settle/aux options are accepted for flag compatibility and
are inert because `classify` does not run those passes. Tests pin the bare
identity (`(1,1)`, movegen order), the hand Threat pair `(4,16)`, the
check/evasion legal count, the attacker override and the CLI default-off
trajectory. No search code changed.

## A1 — proof-tree differential

| Cell | provari positions / length | atomi positions / length | first set-divergent ply | first move-divergent ply |
| --- | ---: | ---: | ---: | ---: |
| `core` | 7,211 / 17 | 10,831 / 14 | **2** | **1** |
| `easy-win` | 714 / 15 | 2,048 / 17 | **1** | **2** |
| `deep-win` | 25,639 / 22 | 49,523 / 25 | **1** | **2** |

`core` reproduces the plan-session probe: the root move and the 27 ply-1
positions are identical; the first set divergence is ply 2 (provari 270,
atomi 377, common 230, provari-only 40, atomi-only 147, 56 common positions
with a different chosen move). The first **move** divergence is ply 1: at 5
of the 27 common ply-1 positions the two provers choose a different second
move. That is the node the classification uses; the ply-2 set difference is
its consequence. `easy-win` and `deep-win` diverge earlier in the set (ply 1,
atomi-only positions) but their first move divergence is ply 2.

The proof lengths differ (17 vs 14, 15 vs 17, 22 vs 25). A different valid
winning move is not a deficiency; Phase A classifies, it does not judge.

## A2 — the five item-15 predicates

Classification is at the first **move**-divergent ply (the parent node where
the second move is chosen), not at the set-divergent child ply. This is the
reading that matches the plan-session probe's "the divergence is in the
attacker's second-move choices". The table is complete: every predicate is
recorded for both the provari-chosen and the atomi-chosen move.

| Cell | quiet grouping holds/specific | fresh `dn` holds/specific | check `pn` holds/specific | EF-PN holds/specific | exact replay holds/specific |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | **3 / 4** | 10 / 0 | 0 / 0 | 10 / 0 | 0 / 0 |
| `easy-win` | 1 / 2 | 2 / 0 | 0 / 0 | 2 / 0 | 0 / 0 |
| `deep-win` | 3 / 3 | 6 / 0 | 0 / 0 | 6 / 0 | 0 / 0 |

Counts are over both sides of each move-divergent position (`core` 5
positions, `easy-win` 1, `deep-win` 3). "Specific" means toggling the
mechanism changes that move's order relative to a sibling.

**`core` detail.** The five move-divergent positions are all at ply 1:

| Position | provari move (class, order) | atomi move (class, order) | quiet grouping |
| --- | --- | --- | --- |
| `r1b1kb1r/…/RNBQKB1R w KQkq - 1 2` | `e4d5` Hanging 36 | `d1e2` Quiet 31 | atomi holds + specific |
| `r1b1kbnr/…/RNBQKB1R w KQkq - 0 2` | `g5h7` Quiet 21 | `e4d5` Hanging 37 | provari holds + specific |
| `r1b1kbnr/…/RNBQKB1R w KQkq - 1 2` | `e4d5` Hanging 38 | `f1b5` Threat 0 | atomi specific, not holds |
| `r1b2bnr/…/RNBQKB1R w KQ - 1 2` | `g5e6` Quiet 17 | `e4d5` Hanging 36 | provari holds, not specific |
| `rnb1kbnr/…/RNBQKB1R w KQkq - 1 2` | `e4d5` Hanging 37 | `f1b5` Threat 0 | atomi specific, not holds |

Quiet grouping is the named divergence: in two positions the divergent move
is `Quiet`, the node has 32–36 quiet moves, the base pair is `(9,9)`, and
turning `--guide-off quiet` changes that move's order relative to a sibling.
The other two mechanisms that hold (`fresh_dn`, EF-PN) hold uniformly and are
not specific: toggling them does not change the divergent move's order. Check
evasions and exact replay do not hold at these attacker nodes. The Threat
moves (`f1b5`) are ordered first by class rank/EF-PN, not by quiet grouping;
they are recorded but do not pass the "affected class" clause.

**G3: pass.** On `core`, quiet grouping holds at the first move-divergent
ply, the divergent move is `Quiet`, and the predicate is specific. This is a
classification, not a causal claim; Phase B tests causality.

## A3 — flag sensitivity (evidence only)

`--guide hand` plus each approximate flag, `core`/`easy-win`, 120 s.

| Cell | hand | group | fresh_dn | replay_exact | ef_off | quiet_off |
| --- | --- | --- | --- | --- | --- | --- |
| `core` | censor | censor | censor | censor | censor | censor |
| `easy-win` | proved, first ply 1 | proved, 1 | proved, 1 | proved, 1 | proved, 1 | proved, 1 |

`core` censors on every hand variant at 120 s (1.97–2.18 M expansions), so
the hand stack cannot produce a `core` proof tree on this host. `easy-win`
proves on every variant and the first divergent ply stays 1; no flag moves it
later. A3 is evidence only; the gate is A2.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass** — `diff.json`, `classify.json`, `flags.json`, `env.json`; parsed JSON only, one process at a time |
| G2 differential | **pass** — first set- and move-divergent ply recorded for all three cells; reproducible from the pinned binary, FENs and flags |
| G3 classification | **pass** — on `core`, quiet grouping holds at the first move-divergent ply, the divergent move is `Quiet`, and the predicate is specific; the five-predicate table is complete |
| G4 soundness | **pass** — no search change; `atomi classify` is read-only; `make test`, `make lint`, `make test-full` green; no GPL dependency, no provari proof committed |
| G5 docs | **pass** — this report; `initiative.md` item 24 status; no ledger row |

## Problems and open parts

- **`deep-win` margin is thin.** Seed 1 proved at 119.049 s inside the
  120 s budget (51,002,209 expansions, length 25). Two earlier runs with an
  earlier build censored at 120 s, so the cell is close to the edge;
  the final pinned binary is the one in `env.json`.
- **Set-divergent vs move-divergent ply.** The plan defines the first
  divergent ply by the FEN set. On `core` that is ply 2, but the move choice
  diverges at ply 1. The classification uses the move-divergent ply; the
  set-divergent ply is recorded for G2. If the owner intended the child ply,
  the classification would be at the third move, not the second.
- **`classify` does not run the sibling simulation or the null-side replay.**
  It reports the `start_numbers` pair before those search-time adjustments.
  That is the pair the five predicates name; the exact-replay predicate is
  therefore not measurable by `classify` and is recorded as `specific: null`.
- **A3 is not a `core` result.** The hand stack censors `core` at 120 s, so
  the flag sensitivity is only measured on `easy-win`, where no flag moves
  the first divergent ply later.
- **No causality.** The differential compares proof trees, not search
  frontiers. Phase B (exact replay) is the causal test.

## Next steps

1. **Recommended:** write plan17 for Phase B (exact replay) under the
   pre-registered gate: ≥ 2× cut in `core` unique expansions vs bare at 20 M
   evals, no `easy-win` regression > 10 %.
2. If Phase B fails, close the conformance line and record the negative; do
   not implement C/D.
3. If Phase B passes, plan C/D (exact `pn`/`dn`, quiet grouping as one child,
   restarts) as separate plans.
