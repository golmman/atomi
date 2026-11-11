# Prover Report 17: item 24 Phase B — exact replay does not apply and does not cut

Plan: [`plan17.md`](plan17.md) (item 24, D14/D18/D21/D24/D26/D29/D30).
Executed 2026-11-10. Phase B only: no search, table, guide or default change.
**G2 fails and G3 fails.** The first move divergence is at an attacker node,
where exact sibling replay never starts; in isolation `--guide-replay-exact`
cuts `core` only **1.02×** at 20 M evals (2× required) and is slightly larger
than `hand` on all three cells. Phase B is a negative; B3 and Phase C/D are
not implemented.

## Scope change

The plan premise was "implement Phase B (exact replay)". Two read-only facts
re-scoped it before any code was written:

1. Exact sibling replay already exists (`--guide-replay-exact`, plan13,
   [`src/guide/replay.rs`](../../../src/guide/replay.rs)). Implementing it
   again would repeat plan13.
2. The plan16 classify data already recorded `exact_replay` holds 0/18 at the
   first move-divergent positions. The decisive question is applicability
   plus tree size, not implementation.

This execution therefore answers B1 (applicability) and B2 (isolation), and
stops before B3 per the plan's G2 stop. The re-scope is recorded here as the
plan requires.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X, Linux 7.2.13 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `f856702` on a dirty tree (plan17 files; agents do not commit), binary sha256 `4ffbdbe1…d75582` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206` |
| Corpus | `corpus-v1`, sha256 `9cd0169a…47d0e` |
| Plan16 source | `classify.json`, sha256 in `env.json` |
| Configuration | `--evals 20000000 --memory 192 --seed 0 --stats --seconds 600` (the wall limit only guards; `--evals` is the budget, as in plan13 phase C) |

Commands: the `env`, `applicability`, `isolation` and `d14` phases in
[`measurements/plan17/README.md`](measurements/plan17/README.md), then
`make test`, `make lint`, `make test-full`, one process at a time.

## B1 — applicability (G2 fails)

For each of the 9 first move-divergent positions (18 move sides: the
provari-chosen and the atomi-chosen move), the check records whether the node
is a defender (AND) node, whether the proved child is in the next ply, and
whether a sibling is still `Open`. It reads the committed plan16 classify
data; it runs no search and writes no table.

| Cell | Positions | Move sides | Defender nodes | Child in next ply | Applicable |
| --- | ---: | ---: | ---: | ---: | ---: |
| `core` | 5 | 10 | 0 | 0 | 0 |
| `easy-win` | 1 | 2 | 0 | 0 | 0 |
| `deep-win` | 3 | 6 | 0 | 0 | 0 |

**All nine are attacker (OR) nodes** (`attacker_node=True`,
`child_in_next_ply=False` in plan16). In `src/search/mod.rs`,
`replay_siblings` is guarded by `!frame.attacker_node`: a proved child at an
attacker node never starts a sibling replay. Exact replay therefore **cannot
explain the first move divergence** on any cell. This is the mechanism
reading of plan16's `exact_replay holds 0/18`; it is not a new search result.

G2 fails, so B3 (`--guide-replay-budget`, `--guide-replay-once`,
`--guide-null-exact`) is **not implemented**. The plan's stop is honoured:
the conformance parameters would tune a mechanism that cannot fire at the
divergence B1 was asked about.

## B2 — isolation matrix (G3 fails)

`--guide none` (bare) vs `--guide hand --guide-replay-exact` vs
`--guide hand` (control), 20 M evals, seed 0, 192 MB. Unique expansions are
the metric because `core` is censored at 20 M in every stack.

| Stack | Cell | Proved | Exit | Unique | Expansions | `child_evals` | `sim_nodes` | `aux_nodes` |
| --- | --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| bare | `core` | no | evals | 586,075 | 630,056 | 20,000,008 | 0 | 0 |
| bare | `easy-win` | no | evals | 622,562 | 669,920 | 20,000,034 | 0 | 0 |
| bare | `deep-win` | no | evals | 620,162 | 668,608 | 20,000,012 | 0 | 0 |
| hand | `core` | no | evals | 573,195 | 648,828 | 20,000,002 | 101,745,882 | 137,190,582 |
| hand | `easy-win` | **yes** | proved | 392,006 | 468,301 | 11,842,338 | 60,005,079 | 81,878,370 |
| hand | `deep-win` | no | evals | 584,085 | 663,127 | 20,000,003 | 101,708,746 | 139,376,171 |
| replay_exact | `core` | no | evals | 574,769 | 650,662 | 20,000,011 | 101,577,420 | 137,146,211 |
| replay_exact | `easy-win` | **yes** | proved | 396,419 | 470,770 | 11,885,350 | 60,315,524 | 82,271,449 |
| replay_exact | `deep-win` | no | evals | 586,294 | 665,165 | 20,000,015 | 101,654,541 | 139,340,701 |

Seed 1 for the best guided stack at seed 0 (`hand`, lowest `core` unique):

| Stack | Cell | Proved | Exit | Unique | Expansions | `child_evals` |
| --- | --- | ---: | --- | ---: | ---: | ---: |
| hand | `core` | no | evals | 578,229 | 653,551 | 20,000,023 |
| hand | `easy-win` | **yes** | proved | 532,626 | 631,763 | 15,732,200 |
| hand | `deep-win` | no | evals | 584,295 | 662,502 | 20,000,020 |

Ratios vs bare: `hand` 0.978 / 0.630 / 0.942; `replay_exact` 0.981 / 0.637 /
0.945 on `core` / `easy-win` / `deep-win`. **G3: fail.** `replay_exact` cuts
`core` **1.02×**, not 2×. The `easy-win` clause passes (0.637, an
improvement, not a regression), but the `core` clause is the gate.

The more informative comparison is against the control, because replay is
opt-in on top of `hand`. Replay exact is slightly **larger** than `hand` on
every cell: `core` 574,769 vs 573,195 (+0.27 %), `easy-win` 396,419 vs
392,006 (+1.13 %), `deep-win` 586,294 vs 584,085 (+0.38 %). It spends
101.6–101.7 M simulation nodes and does not remove expansions. The isolation
matrix reproduces plan13's `add_replay` row (574,769 `core` unique) exactly,
so the measurement is stable; replay's 1.02× `core` cut is the same small
effect plan13 saw inside the full stack.

## D14

The three proved rows (`hand`/`easy-win` seed 0, `replay_exact`/`easy-win`
seed 0, `hand`/`easy-win` seed 1) each got one untimed `--seconds 600
--proof-out` run with the same `--evals 20000000` cap. All three matched
their timed `child_evals`; `atomi validate` and `provari check` accepted all
three (`3/3 accepted`). Proof files were written under `/tmp/atomi-plan17`,
checked, then deleted; none is committed.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass** — `applicability.json`, `isolation.json`, `env.json`; parsed JSON only, one process at a time |
| G2 applicability | **fail** — no first move-divergent position is a defender node; exact replay cannot apply at the first divergence; B3 not implemented |
| G3 tree size | **fail** — `replay_exact` cuts `core` 1.02× vs bare at 20 M (2× required); no `easy-win` regression (0.637) |
| G4 soundness | **pass** — no search change; no new flag; `make test`, `make lint`, `make test-full` green (veto PASS); no GPL dependency; no provari proof committed |
| G5 docs | **pass** — this report; `initiative.md` item 24 status; no ledger row |

## Problems and open parts

- **B1's sibling clause is not evaluated at these nodes.** All nine positions
  are attacker nodes, so the answer is already "not applicable". A defender
  node would need a proof edge to decide whether a sibling is still `Open`;
  `classify` does not carry that set. The driver records `sibling_open: null`
  with that reason rather than guessing. No conclusion depends on it.
- **The first divergence is not the tree-size gate.** G3 is about tree size
  at fixed evals; G2 is about the first divergence. Replay fails both, but
  for different reasons: it cannot fire at the divergence, and where it can
  fire it does not cut the tree. This is the plan's risk row, resolved
  negatively.
- **`deep-win` is censored in every Phase B row**, including the best guided
  stack at seed 1 (584,295 unique, 20 M evals). Phase B does not move the
  `T_3` coverage rung.
- **No causality beyond the mechanism guard.** B1 uses the search code's
  attacker-node guard and plan16's proof-edge detail; it does not rerun the
  divergence with replay forced on an attacker node (that would not be the
  mechanism under test).

## Next steps

1. **Recommended:** close the conformance line (item 24 Phase B) as a
   negative. Do not write Phase C/D plans. Take a formulation decision or a
   new-search-algorithm plan (PDS/PN², TT move ordering, or a fresh
   first-divergence diagnostic) at the next kill review.
2. If the owner wants one more conformance probe, the only unmeasured
   candidate is the **defender-side** first divergence (the first ply where a
   defender's reply choice differs), not the attacker-side first divergence
   measured here. It would need a new diagnostic, not B3.
3. Keep the bare tree as the product candidate; `T_2` is unchanged and the
   next coverage target remains `T_3` (`deep-win` 3/3 at 120 s).
