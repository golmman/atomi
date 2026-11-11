# Prover Report 18: item 25 — exact null replay helps `core`/`easy-win`, not `deep-win`

Plan: [`plan18.md`](plan18.md) (item 25, D14/D18/D21/D24/D26/D30/D31).
Executed 2026-11-10. **G2 passes, G3 fails.** The DF-PN exactness stack is
implemented as two opt-in flags and is sound (5/5 D14 accepted), but the
declared stack cuts `deep-win` unique expansions only **1.009×** at 20 M
(2× required). The stack is closed negative for the `deep-win` gate; no
120 s matrix, no `T_3` claim, no ledger row. The individual `null_exact`
flag is a separate, large `core`/`easy-win` win and is recorded as an
observation, not a coverage claim.

## Scope change

None to the mechanisms. The plan's Phase B named "reuse `guide/replay.rs`
for the exact proof replay"; this execution added the missing
`simulate_with_guide_proof` entry point so the null position never has to be
keyed or stored (the gotcha), and narrowed the null-win search to the exact
one-ply cases the plan authorises. The plan's Phase B also said "verified
one-ply explosion checks on the child and the null side"; the implementation
adds the stored-proof and exposed mate-in-two cases because they are the same
one-ply contract and provari's exact null replay uses them. Check-every-
evasion, double-threat and `recomputed` null wins are **not** ported.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X, Linux 7.2.13 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `6038292` on a dirty tree (plan18 files; agents do not commit) |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206` |
| Corpus | `corpus-v1`, sha256 `9cd0169a…47d0e` |
| Configuration | `--evals 20000000 --memory 192 --seed 0 --stats --seconds 600` |

Data: [`measurements/plan18/`](measurements/plan18/) (`env.json`,
`diff.json`, `probe.json`, `matrix.json`, `README.md`, driver). Commands:
the `all` phase of the driver, then `make test`, `make lint`,
`make test-full`, one process at a time.

## Phase A — mechanism differential (G2 passes)

Read-only comparison of provari's `simulate`/`simulate_defences`/
`replay_threat` with atomi's `guide/simulate.rs`, `guide/replay.rs` and the
signature `Guide::replay`. Four portable gaps on the same DF-PN base:

- **G2-a exact null-threat replay.** provari builds the passed position,
  finds a winning move, replays it on every open defence, and lowers a fresh
  parrying defence. atomi had only the 64-bit signature and the `UNLIKELY`
  prior.
- **G2-b exposing one-move shortcut.** provari treats a defence after which
  the attacker can explode at once as a one-move proof. atomi's replay
  required a stored proof or the guide's reply.
- **G2-c guide explosion/dropped-mate fallback.** provari's guide move may
  be an explosion or a dropped mate; atomi required a stored guide proof.
- **G2-d top-level guide proof override.** provari stores the null proof in
  its table; atomi must not key the null position, so the top guide move is
  passed explicitly.

G2 passes: G2-a..G2-d are portable without changing the search base. Not
ported: check-every-evasion, double-threat and `recomputed` null wins.

## Phase B — the exactness stack (opt-in, default off)

- `--guide-sim-exact`: disables the greedy `simulate::playout` adjustment and
  runs the exact sibling proof replay (`guide/replay.rs`) on a proved
  defence's open siblings. It is the plan's "replace the greedy simulation"
  mechanism; `--guide-replay-exact` remains the additive probe.
- `--guide-null-exact`: at a defender node, `guide/null.rs` builds the
  passed position **ephemerally**, finds an exact winning move (immediate
  explosion/mate, a move into a stored proof, or an exposed mate-in-two),
  and replays its proof on every open defence through the new
  `simulate_with_guide_proof`. A parrying defence gets the `UNLIKELY` prior.
  The null board is never keyed, looked up, stored, put on `line` or emitted;
  only real child keys and proofs leave the module.
- `guide/replay.rs` gained the exposing shortcut. It returns the child's own
  one-move proof as well as the length, because the writer needs that entry.

**Writer bug found and fixed by D14.** The first version of the exposing
shortcut returned the length but no child proof entry. The search still
proved, but `--proof-out` failed with `proof: missing`: the writer walks the
real position graph and could not resolve the attacker child after a defender
reply. The shortcut now returns `(child_key, ProofEntry { mv, length: 1 })`,
the caller stores and emits it, and all five D14 rows pass both checkers.
This is the plan's "D14 on any proved row" gate doing its job.

Tests: default-off CLI identity (`--guide none` + both flags = bare),
a mate-in-2 proved with both flags, null-board flip/ep tests, an immediate
null explosion, a stored null-child proof, a quiet no-win position, a null
guide replay on a non-parrying and a parrying defence, and the existing
config/parameter matrix. `make test`, `make lint`, `make test-full` green;
veto PASS.

## Phase C — 20 M isolation (G3 fails)

Seed 0, 192 MB, 20 M child evals. Full table in
[`measurements/plan18/README.md`](measurements/plan18/README.md).

| Stack | `core` unique | `easy-win` unique | `deep-win` unique | ratio vs bare (core/easy/deep) |
| --- | ---: | ---: | ---: | --- |
| bare | 586,075 | 622,562 | 620,162 | 1 / 1 / 1 |
| hand | 573,195 | **392,006** | 584,085 | 0.978 / 0.630 / 0.942 |
| sim_exact | 555,774 | **264,533** | 582,026 | 0.948 / 0.425 / 0.939 |
| null_exact | **538,167** | **19,031** | 613,763 | 0.918 / 0.031 / 0.990 |
| sim_null (declared) | 587,286 | **34,175** | 614,821 | 1.002 / 0.055 / 0.991 |

`sim_null` proves `easy-win` at 986,079 evals and leaves `core`/`deep-win`
censored at 20 M. The gate:

- `deep-win` cut 1.009× (2× required) — **fail**;
- `core` 1.002, `easy-win` 0.055 — no regression.

The individual ablation is the informative part. `sim_exact` cuts `deep-win`
1.066× and `easy-win` 2.35×; `null_exact` cuts `easy-win` 32.7× and proves
`core` at 18.3 M evals (bare, hand and `sim_exact` are censored there), but
`deep-win` is 0.990 (slightly worse). The two mechanisms interact
negatively on `core` in the combined stack (`sim_null` 587,286 vs
`null_exact` 538,167). `deep-win` is insensitive to both.

## D14

Five proved seed-0 rows re-ran untimed with `--seconds 600 --proof-out`
under `/tmp/atomi-plan18`: `hand`/`easy-win`, `sim_exact`/`easy-win`,
`null_exact`/`core`, `null_exact`/`easy-win`, `sim_null`/`easy-win`. All
five matched their timed `child_evals`; `atomi validate` and `provari check`
accepted all five (`5/5`). Proof files were deleted; none is committed.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass** — `env.json`, `diff.json`, `probe.json`, `matrix.json`; parsed JSON only, one process at a time |
| G2 mechanism differential | **pass** — four portable exactness gaps on the DF-PN base |
| G3 tree size | **fail** — `sim_null` cuts `deep-win` only 1.009× at 20 M (2× required); `core` 1.002, `easy-win` 0.055 (no regression) |
| G4 soundness | **pass** — default-off identity; both flags opt-in; `make test`, `make lint`, `make test-full` green (veto PASS); no GPL dependency; no provari proof committed; D14 5/5 |
| G5 docs | **pass** — this report; `measurements/plan18`; `initiative.md` item 25 closed negative; no ledger row |

## Problems and open parts

- **`deep-win` is the only failing clause and it is the gate.** The proof-set
  gap there is 0.10 %, so the tree is tree-directedness-limited. Exact null
  replay adds proof power on `core`/`easy-win` but does not change the
  `deep-win` trajectory: `sim_nodes` is 103 M on `null_exact`/`deep-win`
  versus 101.7 M on `hand`, and unique expansions rise slightly.
- **The combined stack is worse than its parts on `core`.** `sim_exact`
  removes the greedy playout that `null_exact` was tuned on top of, and the
  interaction changes the `core` trajectory (`sim_null` censored at 20 M;
  `null_exact` proves at 18.3 M). The plan's declared stack is the one
  gated; the individual ablation is recorded, not promoted.
- **The null-win search is narrower than provari's.** Check-every-evasion,
  double-threat and `recomputed` wins are not ported. They are the obvious
  next conformance step if the owner wants to keep the exactness line open,
  but they do not address the `deep-win` insensitivity seen here.
- **The writer contract is stronger after this plan.** Any new exact-replay
  path must return the child's own proof entry for an immediate explosion,
  not just the length; otherwise `--proof-out` fails `missing` even though
  the search result is sound.
- **No 120 s matrix.** The plan's Phase D is conditional on G3; it is not
  run. `T_2` is unchanged; `T_3` remains the next rung.

## Next steps

1. **Recommended:** close item 25 negative for the `deep-win` gate. Do not
   implement PDS-PN/PN² in this plan. At the next kill review, choose between
   (a) a goal re-pin under D30 and (b) a separately authorised new search.
2. If the owner wants one more exactness probe, the only unported null-win
   cases are check-every-evasion and double-threat; they are a small,
   measured extension of `guide/null.rs`, but `deep-win`'s insensitivity
   makes a `deep-win` cut unlikely.
3. Keep `--guide-null-exact` as an opt-in lever for `core`/`easy-win` wall
   (it proves `core` at 18.3 M and `easy-win` at 0.66 M), not as a `T_3`
   claim; a separate plan would need its own `deep-win` gate.
