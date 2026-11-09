# Prover Plan 3: M2+M3 — the hand-guide stack

Initiative: [`prover`](initiative.md), backlog **item 3 (M2+M3, merged; item
4 is merged into it by D21)**. Drafted 2026-11-09 in plan session 3, after
[`report2.md`](report2.md), [`bench.md`](bench.md) and
[`docs/notes/retro.md`](../../notes/retro.md). Baseline: the plan2 tree
(atomi m1, report2 working tree).

This is the first deliberate **stack plan** under D21. It is not a sequence
of one-mechanism plans: the mechanisms below are coupled, are gated together,
and are judged by both-direction ablations **after** the stack gate. The plan
does not implement anything; the execution session does.

## Goal and absolute distance

The initiative's success condition is still: geomean `atomi / provari --guide
hand` wall `< 1.0` on the shared-win set, coverage at least provari hand's,
and every claimed win accepted by `atomi validate` and `provari check`.
Plan3's gate is a milestone gate, not the goal claim (M5 makes the goal
claim).

| Cell | provari hand reference | atomi m1 (report2) | D21 target |
| --- | --- | --- | --- |
| `core` | 0.529 s · 342,289 nodes · mate 17 | censored at 10 s (≈ 181.8 M child evals) | **proved** at 10 s / 192 MB |
| `easy-win` | 0.136 s · 72,523 nodes · mate 15 | **1.482 s** · 30,335,618 child evals · **1,047,370 expansions** · factor 10.897 | **≤ 145,046 expansions** (2 × 72,523) |
| `deep-win` | 6.767 s · 4,574,912 nodes · mate 22 | censored at 10 s (≈ 186.6 M child evals) | report only (no hard D21 cell) |

- **The remaining gap is tree size.** M1 already made per-node cost
  competitive (53.4 ms per 1 M child evals on `core`, 10 M, 192 MB, report2).
  `easy-win` is 14.44 × provari's nodes at m1; D21 asks for 2 ×, so the stack
  must remove about **7.2 ×** of the tree on that cell.
- **`core` is the harder half.** Proving it at 10 s is not a 2 × node
  requirement; it needs a large enough tree reduction that the whole proof
  fits the budget.
- **The retro's ablation is the design premise.** provari's levers are worth
  10.9 × on `easy-win` and 25 × on `core` together, while the product of the
  single-lever costs on `core` is only 7.3 ×. Several levers are neutral or
  negative in isolation (check evasions 0.89, static closures 1.48 on
  `easy-win`, history 1.40 on `core`, sibling simulation 0.91). A
  one-mechanism decision would reject parts of a winning stack, which is why
  D21 replaces it for this initiative.

## Owner decisions recorded in the initiative

- **D21 (2026-11-09) — stack-level gating.** A plan may land a named stack of
  coupled mechanisms under one gate. The gate is judged on the whole stack;
  individual mechanisms are not pre-gated. After the stack passes,
  leave-one-out and add-one-in ablations decide what stays, over the same
  frozen ≥20-cell set and several seeds.
- **D21 — the first stack** is M2+M3, the **hand-guide stack**: move classes,
  check evasions, quiet grouping, EF-PN hand-eval scaling, per-child static
  closures including the one-ply explosion settle, defender priors
  (`UNLIKELY`, defence history), history tie-breaks, sibling simulation and
  the null-move threat replay.
- **D21 — the gate:** `easy-win` expansions ≤ **145,046** (2 × provari
  hand's 72,523 nodes); `core` **proved** at `--seconds 10 --memory 192`; no
  guidance-set refutations; measured on ≥20 frozen cells and ≥3 seeds. Wall
  and `child_evals` are recorded, never substituted for the node gate.
- **D22 (2026-11-09) — seeds and the guidance set.** `--seed N` perturbs only
  tie-breaks after the guide metric, class and history; seed 0 preserves
  movegen order. Same binary + FEN + flags + seed gives identical counters.
  The gate uses seeds 0, 1, 2, 3. `guidance-v1` is frozen by a provari-hand
  census before any atomi stack run (rule below). Auxiliary guide work is
  reported as `sim_nodes`/`settle_nodes`; `child_evals` and `expansions`
  keep their D11 meaning and stay main-search only.

The initiative's working agreement 2, backlog item 3/4, stop rule, take-list
and risk row were updated in the same plan session. Item 4 remains visible as
“merged into item 3” so its stable id is not reused.

## Plan-session findings (read-only)

No repository code was changed in this session. These findings are from
reading the m1 tree, `report2.md`, `bench.md` and the retro; they size the
plan rather than claim a result.

1. **Where the guide attaches.** `search/frame.rs` already has the three
   passes the guide needs: pass 0 (node status), pass 1 (effects), pass 2
   (filter/prefetch), pass 3 (repetition + lookup + fresh `(1, 1)`).
   `search/mod.rs` has `select` (metric only; ties to movegen order),
   `child_thresholds` (the existing 1+ε widening) and one frame-exit store.
   `value.rs` already separates exact `Proved`/`Refuted` from `Open` numbers.
   The guide therefore needs **no change to the value algebra**; it writes
   only fresh `Open` numbers and the selection tie-break.
2. **No guide surface today.** `Limits` has `seconds`, `evals`, `memory_mb`
   and `filter`; `cli.rs` has no `--seed` or `--guide`; the output contract is
   exactly 14 fields. `select` has no history or seeded tie-break. The
   `Child` record has only `mv`, `zeroing`, `value`.
3. **The census is not the bench set.** Only `core`, `easy-win` and
   `deep-win` are provari-hand wins at the canonical 10 s budget. Report2's
   fixture sweep proves 41 of 52 `wins.tsv` rows at 2 M evals, but that is an
   atomi set, not a provari-shared set. D22 therefore freezes a separate
   guidance set by a provari census at a longer **selection** budget. The
   3-cell bench remains the ledger set; the guidance set is the robustness and
   ablation set.
4. **Seed 0 must be a true no-op.** The current search is deterministic and
   ties to movegen order. The seed may only enter after metric, class and
   history have tied, and seed 0 must return to movegen order. This lets
   `--guide none --seed 0` reproduce m1 counters exactly and keeps `make veto`
   on its current default path.
5. **The one-ply settle has a proof-output consequence.** If a child is
   settled `Proved` without being expanded, the proof table has no entry for
   that child, and the writer cannot resolve the parent's proof by following
   the parent move. The settle must therefore insert the child's own proof
   (key, winning move, length 1, limit 99) into the proof table, and the
   event-fed store must receive the matching `AttackerProven` event. This is
   a design constraint, not an optimisation detail.
6. **Simulation and settle work must not pollute D11.** The legacy
   front-end counted simulation evals in its `child_evals`; atomi does not.
   Plan3 keeps `child_evals`/`expansions` for the main search and adds
   `sim_nodes`/`settle_nodes` diagnostics. Otherwise the node gate and the
   per-node comparison would stop measuring the same thing.
7. **The measurement set must be frozen before atomi runs.** Any census that
   looks at atomi's results first can cherry-pick favourable cells. The
   census and `guidance-v1.tsv` are therefore Phase 0 deliverables, with a
   sha256 recorded in `env.json`.

## Scope and non-goals

**In scope**

- A new `src/guide/` layer: move classes, hand evaluation, threat signatures,
  defender priors, history, bounded simulation and the one-ply explosion
  settle.
- Integration into `frame.rs`, `search/mod.rs`, `value::Child`, `Limits`,
  `cli.rs`/`main.rs` and the output contract.
- `--seed`, `--guide none|hand`, and measurement-only `--guide-off` /
  `--guide-on` lists; one `GuideParams` table with a stable `key=value` dump
  for the legacy optimizer contract (spec only, no optimizer in this repo).
- The guidance census, `guidance-v1.tsv`, a new `run_guidance.py` driver,
  the D21 gate, the D14 proof checks per seed, and both-direction ablations.
- Tests, `bench.md` row (no goal claim), `AGENTS.md` architecture/gotchas,
  `initiative.md` status/history and `report3.md`.

**Out of scope**

- The goal claim and the default-flag decision (M5).
- The M4 scaled proof worker, `.pvr`, tablebase, portfolio, parallelism,
  restarts (D5) and the model guide.
- Any change to the value algebra's proof/refutation rules, the validator,
  the key (`EnPassantMode::Legal`), the proof table's domination rule or the
  `OPEN_MAX` saturation rule.
- Any provari/shakmaty code, text, dependency or committed artifact. The
  census runs the pinned oracle as an external process only.

## Design

### 1. Module layout

```
src/guide/
  mod.rs        Guide, GuideConfig, GuideParams, start_numbers, node context
  class.rs      MoveClass, classification, class table, quiet grouping
  eval.rs       hand_eval, integer EF-PN factor
  threat.rs     ThreatSignature, null-position threat, replay, one-ply settle
  history.rs    fixed history tables, seeded tie-break, deterministic order
  simulate.rs   bounded sibling simulation and its score
```

`src/guide/mod.rs` is the only public surface. `search` depends on `guide`;
`guide` depends on `rules`, `mover` and `value`, never on `search` or `table`.
Files stay under ~10 KB or carry the usual header justification. The
validator stays independent.

`GuideConfig` has one switch per mechanism so the ablations are runtime
flags, not 18 scratch builds:

```rust
pub struct GuideConfig {
    pub enabled: bool,          // --guide hand
    pub move_classes: bool,     // classes
    pub check_evasions: bool,   // evasions
    pub quiet_grouping: bool,   // quiet
    pub ef_pn: EfPnMode,        // off | or_only | both
    pub static_closures: bool,  // closures (includes the one-ply settle)
    pub defender_priors: bool,  // priors (UNLIKELY + defence history)
    pub history_ties: bool,     // history
    pub sibling_sim: bool,      // sim
    pub null_replay: bool,      // replay
}
```

CLI:

- `--guide none|hand` (default `none`; `hand` is the full stack).
- `--seed N` (default 0; 0 = no seeded perturbation).
- `--guide-off LIST` / `--guide-on LIST` (comma-separated short names
  `classes,evasions,quiet,efpn,closures,priors,history,sim,replay`). These
  are measurement hooks; they do not change the default. The EF-PN mode is a
  `GuideParams` value (`off|or_only|both`), so the `both` A/B is a parameter
  file, not a separate short name.
- `--guide-params PATH` (optional `key=value` file for the optimizer
  contract; absent = built-in table).

The output contract adds `"seed":N`, `"sim_nodes":N` and
`"settle_nodes":N` after `"expansions"`; the existing 14 fields keep their
order and meaning. The driver parses by key, so old drivers keep working.

### 2. Move classes and starting numbers

`MoveClass` is `Threat`, `WeakThreat`, `Hanging`, `Quiet`, `Evasion`. A
class is computed from the node context and the move's effect, without
building the child except through the existing pass-1 board children:

- `Evasion` only when the node is in check; it overrides every other class.
  Its base is the number of legal moves (the check has that many evasions),
  clamped to at least 1.
- `Threat`: the move gives check, or the threat signature says it creates an
  immediate explosion/capture threat on the enemy commoner's blast zone.
- `Hanging`: the move is a capture, or it moves a piece that an enemy piece
  attacks (using `attackers_to_color`/`is_attacked` bitboards), or it leaves a
  non-commoner piece attacked.
- `WeakThreat`: the move creates a threat that the replay says is parried by
  a cheap reply; this is the “looks active but is not” class.
- `Quiet`: everything else.

`GuideParams` holds one `(pn_base, dn_base)` pair per class. The plan's
starting table uses the class bases already recorded in the initiative as an
idea (not copied code): the published value is the **proof-number** base, and
the complementary value is the disproof base.

| Class | `pn_base` | `dn_base` |
| --- | ---: | ---: |
| `Threat` | 4 | 16 |
| `WeakThreat` | 8 | 12 |
| `Quiet` | 9 | 9 |
| `Hanging` | 15 | 9 |
| `Evasion` | `max(1, legal_moves)` | `max(1, legal_moves)` |

The table is from the mover's perspective. At an OR (attacker) node it is used
directly; at an AND (defender) node the pair is swapped, because a move that
promises for the mover is a danger for the attacker. The values are a starting
point exposed in `GuideParams`, not a gate; the ablation phase decides them.

**Quiet grouping** is a starting-number statement, not an AND/OR merge. All
fresh `Quiet` children at a node share the class pair and are ordered by the
static class/score; no child is omitted, merged or given a fake value. The
group representative is only the first in the guide order; when it is
refuted, the next quiet child is next in order.

### 3. Check evasions

When `state.checkers` is non-empty, every legal move is an `Evasion`; its
starting pair comes from the evasion count instead of the class table. This is
the “check = number of evasions” idea. It is independently switchable because
the retro's single-lever measurement on `core`/`easy-win` is not favourable;
the stack interaction decides it later.

### 4. EF-PN hand-eval scaling

`guide/eval.rs` computes a cheap, deterministic integer evaluation in
`[-256, 256]` from:

- material (commoner values the same; pawns/knights/bishops/rooks/queens);
- commoner safety (attacks into the enemy commoner's blast zone, own
  commoner's exposure);
- immediate threat count from the threat signature;
- a bounded mobility proxy (attacked squares, not a second movegen).

`f = clamp(16 + e, 8, 32)`; `f = 16` is neutral. At an OR node, fresh child
numbers become

```
pn = max(1, pn_base * 16 / f)
dn = max(1, dn_base * f / 16)
```

so an attacker-favourable child gets a lower proof number. The default mode is
`or_only`. `both` swaps the scaling at AND nodes as well; it is an ablation,
not the default. All arithmetic saturates at `OPEN_MAX`, never `INFINITY`.
The eval never turns `Open` into `Proved`/`Refuted`.

### 5. Per-child static closures and the one-ply explosion settle

The existing exact closures stay: node-entry status, board-child `status`,
material draw, the 50-move limit and repetition. Static closures add one
**exact** child-level closure:

- **One-ply explosion settle, both sides.** For every fresh child, a
  conservative screen asks whether the child's side to move may have an
  immediate explosion. A screen hit builds the child and runs an exact test:
  enumerate the child's legal moves; for each candidate capture, make it and
  check that the opponent's commoner is gone. The first such move is an
  immediate win for the child's side to move.
  - child side == attacker → `Proved { length: 1, limit: 99, mv: Some(m) }`;
  - child side == defender → `Refuted { cycle: ANY, from: 0 }`.

  The screen is conservative (no false negatives); every screen hit is
  verified by a real move, so there are no false settles. The exact test is
  unit-tested on the playout set against building every child and searching
  one ply.
- The movegen and make-move work of the settle is counted in `settle_nodes`,
  not `child_evals`/`expansions`.
- A settled `Proved` child is **stored in the proof table at the child's
  key** with the child's winning move, length 1, limit 99, and its
  `AttackerProven` event is emitted when the child is an attacker node. A
  settled `Refuted` child is stored as a path-independent TT refutation.
  This is what lets the writer resolve a parent proof whose child was never
  expanded.
- “Exposed king” and “unparried double threat” are class signals, not exact
  closures. “Cannot-win material” is already exact through the rules layer;
  it is not re-implemented as a guess.

`Frame::enter` returns the settled child values as a small `pending` list
(`(child_key, Value)`) in `Entered::Open`; `mid` stores them (proof table or
TT) before the MID loop. This avoids a mutable-borrow tangle and keeps the
event-fed store differential with the proof table.

### 6. Threat signatures, null-move replay and defender priors

`guide/threat.rs` defines a small `ThreatSignature`: immediate explosion
targets, checks, and hanging/commoner-adjacent pieces, in a fixed-size array
(no allocation in the hot path).

- **Null-move threat replay.** At an OR node, compute the threat signature of
  the *null position*: a board clone with the side to move flipped and en
  passant cleared. This position is a heuristic device only; it is never
  searched, keyed, stored, put on `line`, written into a proof or emitted as
  an event. If the side to move is in check, the null replay is skipped.
- **Replay.** After each child move, recompute the signature and compare with
  the null signature. A defender move that removes an attacker threat
  `parries`; one that removes none is `unlikely`.
- **`UNLIKELY` priors.** At an AND node, a reply that parries nothing and
  creates no defender threat gets `pn = max(1, pn_base / 2)` and
  `dn = min(OPEN_MAX, dn_base * 2)`: it neither inflates the AND node's
  proof need nor looks like the likely refutation. A reply that parries a
  real threat or creates a defender threat keeps its class pair. This is our
  implementation of “halve the proof number of defences that parry nothing”;
  the orientation is pinned by a unit test.
- **Defence history.** A fixed array `history[side][from][to]` of saturating
  `u16` counters (2 × 64 × 64, no hash map) records:
  - at an AND node, a child that returns `Refuted` (a defence that worked);
  - at an OR node, a child that returns `Proved` (an attacker move that
    worked).
  These counters scale the starting `dn` (defence) or `pn` (attack) and are a
  tie-break. The table is cleared per run and is deterministic.

### 7. History tie-breaks and seeded order

`select` compares, in order:

1. the current DF-PN metric (`pn` at OR, `dn` at AND);
2. the guide class priority (lower base number first);
3. the history score from `guide/history.rs`;
4. the seeded tie-break `splitmix64(seed ^ key ^ move_bits)` (only for
   `seed != 0`);
5. movegen order.

Seed 0 returns to movegen order. The seed never overrides a non-tied metric or
class. The same binary + FEN + flags + seed gives identical counters; different
seeds may visit a different tree but must agree on the result. `select` and
the history update are the only places the seed enters.

### 8. Sibling simulation

For each fresh, not-yet-settled child, at most once per frame, run a bounded
deterministic playout from the child (max 6 plies, both sides chosen by the
same class/eval guide, no tables, no events, no line mutation). Score the
child by the terminal depth or by the final hand eval. Map the score to a
starting-number adjustment (a lower `pn` for a short simulated attacker win,
a lower `dn` for a short simulated defence). Simulation never writes a
`Value`, never proves or refutes a node, and never touches the proof table.
Its expansions are counted in `sim_nodes` only. The bound is a `GuideParams`
constant; the ablation decides whether the cost is worth it.

### 9. Integration flow

```
Engine::run
  guide = Guide::new(&limits.guide)
  mid(root, key, 0, ...)

mid(board, key, ply, tpn, tdn)
  Frame::enter(board, key, attacker, line, &tables, &guide, &history, seed)
    pass 0: node status
    context = guide.node_context(board, state)     // in-check, eval, signatures
    pass 1: effects; class; starting numbers; settle screen;
            keys; prefetch; collect pending settled values
    pass 2: filter / prefetch
    pass 3: repetition; table lookup; fresh -> guide.start_numbers(...)
  mid stores pending values/events, then the MID loop
  select uses metric -> class -> history -> seed -> movegen
  after each child returns: history.observe(...)
  frame exit: store + attacker proof event, as today
```

`value.rs` algebra is unchanged. `Child` gains one `class: MoveClass` field;
the starting numbers live in `Child.value` as `Open { pn, dn }`. The guide
never edits a `Proved` or `Refuted` value. The root `Engine` owns the `Guide`
and the history table; both are per-run, not global.

### 10. Soundness, determinism and accounting

- **Soundness.** A guide number can only make a node look harder or easier;
  it cannot create a `Proved`/`Refuted` value. Proofs still require proved
  children; refutations still require all children refuted. The one-ply
  settle is exact and its proof is inserted at the child's key. The null
  position never enters the search. Therefore the stack changes only
  completeness and speed.
- **Determinism.** Fixed history arrays, fixed simulation bound, no hash-map
  iteration, no wall-clock decisions beyond the existing deadline. Same
  `seed` + flags = same counters. `--guide none --seed 0` must be
  counter-identical to m1 on the fixed panel.
- **Accounting.** `child_evals` +1 per child record, `expansions` +1 per main
  search node (D11). `settle_nodes` counts settle movegens/makes;
  `sim_nodes` counts simulation expansions. The D21 gate uses `expansions`;
  `child_evals`, `settle_nodes`, `sim_nodes` and wall are reported.
- **OPEN_MAX.** Every guide sum/product saturates at `OPEN_MAX`; no `Open`
  value may equal `INFINITY`. A unit test covers the extreme class/eval/
  history combination.

## Measurement: the frozen guidance set

The 3-cell bench set is not large enough for D21. Plan3 adds a separate
guidance set; the bench ledger stays 3 cells.

### Census and selection rule (`guidance-v1`)

1. Candidates: `core`, the 20 `sample20` cells, and every `wins.tsv` row
   (deduplicate by FEN, first occurrence wins). The candidate list is written
   before any atomi stack run.
2. Run the pinned provari `--guide hand` on each candidate with
   `--seconds 30 --memory 192`. A censored candidate gets one `--seconds 60`
   retry; the budget used is recorded. These are **selection** budgets, not
   the bench budget.
3. A candidate is shared if provari proves. Order shared candidates:
   `core`, `easy-win`, `deep-win`, then the remaining `sample20` in corpus
   order, then `wins.tsv` in file order.
4. Take the first **24** shared candidates, but never fewer than **20**. If
   fewer than 20 shared candidates exist after the 60 s retry, the gate is
   **red** (do not lower the count or hand-pick cells).
5. Write `guidance-v1.tsv` with `name fen source ref_nodes ref_wall_s
   selection_budget_s` and its sha256 in `env.json`. The set is frozen for
   plan3 and later plans; additions require a new set name.

The census is Phase 0. The gate and ablation drivers read the frozen file and
refuse a hash mismatch.

### Drivers

`docs/plans/prover/measurements/plan3/run_guidance.py` (stdlib only):

- inputs: `--phase census|gate|wall|ablation`, `--set guidance-v1.tsv`,
  `--candidates`, `--retry-seconds`, `--freeze`, `--cells`, `--provari`,
  `--provari-commit`, `--atomi`, repeatable `--stack NAME=FLAGS`,
  `--leave-one-out`, `--add-one-in`, `--seeds 0,1,2,3`, `--repeats`,
  `--seconds`, `--memory`, optional `--evals`, `--proof-check`,
  `--provari-check`, `--out`, `--dry-run`;
- for each cell, seed and stack: append `--seed S` to the stack flags, run
  `atomi prove FEN ...`, parse the last JSON line; run the reference once per
  cell (provari is deterministic; repeats are for wall only);
- per-cell rows: `expansions`, `child_evals`, `settle_nodes`, `sim_nodes`,
  `ms`, wall, exit, length, proof fields; per-cell medians and seed spread;
- summaries: `atomi_expansions / provari_nodes` and
  `atomi_child_evals / provari_nodes` per cell, wall factor, coverage,
  geomean over shared cells, and a `gate` block for `easy-win` and `core`;
- `--proof-check`: for every won gate cell and seed, one extra
  `--seconds 600 --proof-out` run with the same flags and seed; require
  identical `child_evals`, `atomi validate` valid, `provari check` valid and
  `positions == won_positions`; any other outcome censors that cell/seed with
  the verdict class;
- proofs go to `/tmp`, never the repo; no raw transcripts; `__pycache__`
  disabled.

Census can be a `--phase census` mode of the same driver, writing
`census.json`. The gate/ablation phases refuse to run if the set is absent or
its hash differs.

### Gate runs

1. **Node gate** (deterministic): guidance set × `bare` and `hand` stacks ×
   seeds 0,1,2,3, `--seconds 600 --evals 20000000 --memory 192`. Parse
   `expansions`. A cell not proved inside the cap is censored and reported
   with a lower bound; it is not a win.
2. **Wall/coverage gate:** `core` and `easy-win` × the same seeds,
   `--seconds 10 --memory 192`, 3 repeats. `core` must be proved on every
   repeat/seed; wall medians and factors are reported.
3. **Bench row:** the canonical 3-cell `run_bench.py` run with
   `--atomi-stack 'hand=--guide hand --seed 0'`, `--proof-check`. No goal
   claim; the row is appended to `bench.md` only if G1–G7 pass.

The two budgets are deliberate: the node gate is deterministic and fair to
all variants; the 10 s wall run is the product condition for `core`.

### Both-direction ablations (after the gate)

Only after the full stack passes D21. Same `guidance-v1`, same seeds
`0,1,2`, `--seconds 600 --evals 10000000` (a fixed deterministic cap; a
censored cell is reported as such), one process at a time.

- **Leave-one-out:** `hand --guide-off X` for each short name below.
- **Add-one-in:** `none --guide-on X` for each short name below, over the
  bare `--guide none --seed S` skeleton.

| Short | Mechanism | Leave-one-out flag | Add-one-in flag |
| --- | --- | --- | --- |
| `classes` | move-class starting numbers | `--guide-off classes` | `--guide-on classes` |
| `evasions` | check = number of evasions | `--guide-off evasions` | `--guide-on evasions` |
| `quiet` | quiet grouping | `--guide-off quiet` | `--guide-on quiet` |
| `efpn` | EF-PN hand-eval scaling (OR only) | `--guide-off efpn` | `--guide-on efpn` |
| `closures` | per-child static closures + one-ply settle | `--guide-off closures` | `--guide-on closures` |
| `priors` | `UNLIKELY` + defence history | `--guide-off priors` | `--guide-on priors` |
| `history` | history tie-breaks | `--guide-off history` | `--guide-on history` |
| `sim` | sibling simulation | `--guide-off sim` | `--guide-on sim` |
| `replay` | null-move threat replay | `--guide-off replay` | `--guide-on replay` |

Report per-cell and geomean node ratios, coverage and seed medians. Decide on
the stack, not on a per-position minimum-gain rule. A mechanism is kept if
removing it makes the full stack worse and adding it to the bare skeleton
makes the skeleton better; a mechanism that hurts both directions is a
removal candidate. If the final composition changes, re-run the D21 gate on
that composition before closing. Ablations cannot turn a failed gate into a
pass.

### Re-measured baseline

Before the stack gate, run m1 (the plan2 binary, rebuilt in `/tmp` from the
report2 tree) and the bare `--guide none --seed 0` build on the same
guidance set and seeds. They must agree on counters; a difference means the
guide integration changed the no-guide trajectory and is a bug, not a stack
effect. The m1/bare rows are the ablation baseline.

## Tests

Fast tier (`make test`; keep the existing ≈ 12 s, add ≤ 3 s):

- **Guide unit tests:** class table exhaustive for both node kinds; every
  starting pair positive and ≤ `OPEN_MAX`; `OPEN_MAX` saturation; quiet
  grouping does not drop or merge children; check-evasion base equals the
  legal-move count; EF-PN neutral at `e=0`, monotone direction, integer
  bounds; null position is never searched (a spy/test wrapper asserts no
  lookup/store/event); threat replay deterministic.
- **Seed:** seed 0 == current movegen-order tie-break; same seed repeated
  identical; different seeds may differ but agree on exit/length class;
  `--guide none --seed 0` is counter-identical to m1 on a fixed panel
  (mate-in-1/2, veto control, pawn ending, `core` at 1 M evals).
- **One-ply settle differential:** over the seeded playout positions, for
  every fresh child, the screen/exact settle agrees with building the child
  and searching one ply; count screens, hits and false positives; no settled
  value contradicts `rules::status`; settled proofs validate and are found
  by the writer.
- **Closure proof output:** a constructed child that is settled `Proved`
  without expansion writes a proof whose positions include the child's
  winning move; `atomi validate` accepts it. The event-fed store and the
  proof table write byte-identical files for that case (D15/D18).
- **History:** bounded counters, deterministic update, defence/attacker
  orientation; seeded hash stable.
- **Simulation:** bounded plies, no table/proof/event/line access, `sim_nodes`
  counts only simulation work, deterministic.
- **Search integration:** guide on vs off may change counters/length but
  never `Proved`↔`Refuted` on the fixture wins and never proves a
  `never_prove` row; `--proof-out` does not change counters with the same
  guide+seed; repeat runs identical.
- **CLI:** `--seed` accepts 0 and positive values; `--guide`, `--guide-off`,
  `--guide-on` parse and reject unknown names; output contract has the new
  fields in order; `--guide none --seed 0` keeps the old 14-value behavior
  plus the new fields.

Slow tier (`make test-full`):

- Fixture oracle with `--guide hand --seed 0`, at 2 M evals: no new
  refutations, no `never_prove` found; censored rows are listed with both
  counters; the 2× re-run rule for any row m1 proves and the stack censors.
- The `dec10`/`rem01` regressions and `make veto` (default path unchanged;
  an additional hand-guide veto row via the CLI).
- The `easy-win` stack proof and the `core` 10 s proof (the gate itself is
  measured in Phase C, not in the fast test).

## Gates (acceptance)

- **G1 D21 stack gate.** Full `hand` stack: `easy-win` `expansions` ≤
  145,046 and `core` proved at `--seconds 10 --memory 192`; every seed
  0,1,2,3 passes (report median and worst seed); measured on `guidance-v1`
  with ≥20 cells and all four seeds.
- **G2 guidance soundness/coverage.** No `wins.tsv` guidance cell refuted;
  no `never_prove` row found; coverage ≥ provari hand on the guidance set;
  every written proof accepted by `atomi validate` and `provari check` with
  equal positions; D14 proof runs have identical `child_evals` for the same
  seed.
- **G3 fixture oracle.** `make test-full` green; no outcome regression on the
  copied fixture oracle; the G4 2× list is empty or each entry is re-run and
  explained.
- **G4 determinism/accounting.** `--guide none --seed 0` counter-identical to
  m1; same seed + flags identical; `--proof-out` does not change counters;
  `settle_nodes`/`sim_nodes` reported and excluded from `child_evals`/
  `expansions`; no `Open` value at `INFINITY`.
- **G5 tests/lint/licence.** `make test`, `make lint`, `cargo build
  --release`, licence gate clean; no new dependency.
- **G6 measurement integrity.** Census before atomi runs; `guidance-v1.tsv`
  hash pinned; ≥20 deduplicated cells, ≥3 seeds; drivers, parsed JSON/CSV,
  `env.json` and README under `measurements/plan3/`; no raw transcripts,
  provari artifacts or `__pycache__`.
- **G7 ablations.** Both directions completed after G1, over the same frozen
  set and seeds; keep/remove decisions and any re-run of D21 recorded. No
  mechanism adopted on a single-lever run.
- **G8 docs/provenance.** No copied code or data; `bench.md` row (no goal
  claim), `AGENTS.md` architecture/commands/gotchas, `initiative.md` status
  and history, `report3.md`, and the provenance block for the census/driver.

## Tasks

The plan is one declared stack; the phases are execution checkpoints, not
separate plans. **Split rule:** stop only at a phase boundary. If the session
ends before Phase C, the gate is not claimed; the report records where it
stopped and the next session resumes the same stack.

**Phase 0 — census, seed and measurement surface (no search behavior)**

- P0.1 Add `--seed N` to `cli.rs`/`main.rs`, `Limits.seed`, the output field,
  and the seed-0 no-op tests. Keep `--guide` absent for now or parse it as
  `none`.
- P0.2 Write `run_guidance.py` (`census` mode first), the frozen-set hash
  check, `env.json` skeleton and the measurement README.
- P0.3 Run the provari-hand census on `core + sample20 + wins.tsv` at
  30 s / 60 s, freeze `guidance-v1.tsv`, record its sha256.
- P0.4 Rebuild m1 in `/tmp` and run the baseline `bare`/m1 rows on the frozen
  set at seeds 0–3; they must agree.
- Gate: census file + hash, driver parse (`ast.parse`), seed tests green; no
  search result changed.

**Phase A — guide substrate and no-op integration**

- A1 `src/guide/{mod,class,eval,threat,history,simulate}.rs` with the types,
  `GuideParams`, `GuideConfig`, the class table, the eval, the threat
  signature, the history table, the seeded hash and simulation stubs.
- A2 `Child.class`; `Frame::enter` takes `&Guide`, `&History`, `seed`;
  returns pending settled values; fresh `Open` goes through
  `guide.start_numbers` (initially the identity `(1,1)`); `select` gets the
  guide/history/seed ordering but with every mechanism off.
- A3 `--guide none|hand`, `--guide-off/on`, `--guide-params`, output fields;
  `--guide none --seed 0` must be counter-identical to m1 on the fixed panel.
- Gate: `make test`, `make test-full`, `make lint`; no-guide identity test;
  no mechanism enabled.

**Phase B — the stack mechanisms (no gate yet)**

- B1 Move classes, check evasions and quiet grouping; tests.
- B2 EF-PN hand eval (`or_only`); tests.
- B3 Static closures and the one-ply explosion settle, including pending
  proof insertion and the event; differential tests.
- B4 Threat signature, null-move replay and `UNLIKELY`/defence priors; tests.
- B5 History tie-breaks and the defender/attacker history update; tests.
- B6 Sibling simulation (`sim_nodes` accounting); tests.
- B7 Wire `hand` to all mechanisms; verify the ablation flags toggle exactly
  one mechanism each.
- Gate: `make test`, `make test-full`, `make lint`; informal counters only.
  Do not claim the stack gate.

**Phase C — D21 gate and both-direction ablations**

- C1 Node gate: `bare`/`hand`, guidance set, seeds 0–3, 20 M cap.
- C2 Wall/coverage gate: `core`/`easy-win`, 10 s, 3 repeats, D14 proof checks.
- C3 Bench row: `run_bench.py` 3 cells, `hand`, seed 0, `--proof-check`.
- C4 If G1–G3 pass, run the leave-one-out and add-one-in ablations; write
  `ablation.json` and the keep/remove table. Re-run D21 if the composition
  changes.
- C5 Write `measurements/plan3/{README.md, env.json, census.json,
  guidance-v1.tsv, guidance.json, guidance.csv, ablation.json, bench.json}`.
- Gate: G1–G7 evaluated and written down. A G1 miss follows the D21 stop
  rule (record, do not split into mechanism plans).

**Phase D — close-out**

- D1 `bench.md`: append the plan3 `hand` row and summary; no goal claim.
- D2 `AGENTS.md`: architecture (`src/guide/`, closure proofs), commands
  (`--seed`, `--guide`), gotchas (seed 0 no-op; settle proofs must be stored;
  simulation/settle accounting; null position never searched; guide changes
  only Open numbers).
- D3 `initiative.md`: item 3 status, D21/D22 history, M4 reference numbers.
- D4 `report3.md` (final task).

## Exact measurement commands (execution session)

```sh
make oracle                         # pinned provari in /tmp/atomi-oracle
cargo build --release

# Phase 0 census (selection budgets; scratch oracle only)
python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase census --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --candidates bench/corpus.json,tests/fixtures/wins.tsv \
  --seconds 30 --retry-seconds 60 --memory 192 \
  --out docs/plans/prover/measurements/plan3/census.json
# then freeze guidance-v1.tsv from census.json (same driver, --freeze)

# D21 node gate
python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase gate --set docs/plans/prover/measurements/plan3/guidance-v1.tsv \
  --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --atomi target/release/atomi \
  --stack 'bare=--guide none' --stack 'hand=--guide hand' \
  --seeds 0,1,2,3 --seconds 600 --evals 20000000 --memory 192 \
  --out docs/plans/prover/measurements/plan3/guidance.json

# core / easy-win wall gate (10 s product condition)
python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase wall --set docs/plans/prover/measurements/plan3/guidance-v1.tsv \
  --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --atomi target/release/atomi --stack 'hand=--guide hand' \
  --cells core,1eb80b92f92c --seeds 0,1,2,3 --repeats 3 \
  --seconds 10 --memory 192 --proof-check \
  --provari-check /tmp/atomi-oracle/provari/target/release/provari \
  --out docs/plans/prover/measurements/plan3/wall.json

# bench row after the gate (no goal claim)
python3 bench/run_bench.py \
  --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --atomi target/release/atomi --atomi-stack 'hand=--guide hand --seed 0' \
  --cases core,1eb80b92f92c,db6a741c723f --repeats 3 --proof-check \
  --provari-check /tmp/atomi-oracle/provari/target/release/provari \
  --out docs/plans/prover/measurements/plan3/bench.json

# both-direction ablations (after G1)
python3 docs/plans/prover/measurements/plan3/run_guidance.py \
  --phase ablation --set docs/plans/prover/measurements/plan3/guidance-v1.tsv \
  --provari /tmp/atomi-oracle/provari/target/release/provari \
  --provari-commit 48686bc6c786d6ea70568f0b2ad4002cf0520206 \
  --atomi target/release/atomi \
  --stack 'full=--guide hand' \
  --leave-one-out classes,evasions,quiet,efpn,closures,priors,history,sim,replay \
  --add-one-in classes,evasions,quiet,efpn,closures,priors,history,sim,replay \
  --seeds 0,1,2 --seconds 600 --evals 10000000 --memory 192 \
  --out docs/plans/prover/measurements/plan3/ablation.json
```

## Validation

```sh
make test
make lint
make oracle
make test-full
cargo build --release
python3 - <<'PY'
import ast, pathlib
for p in [*pathlib.Path("bench").glob("*.py"),
          *pathlib.Path("scripts").glob("*.py"),
          *pathlib.Path("docs/plans/prover/measurements/plan3").glob("*.py")]:
    ast.parse(p.read_text())
PY
sha256sum bench/corpus.json             # 9cd0169a…47d0e
git status --porcelain && git add --dry-run .
```

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| The stack overfits one seed or one cell | Frozen ≥20-cell set; seeds 0–3; report median and worst seed; no per-cell minimum-gain adoption |
| The stack gate fails (likely on `core`) | Record the stack result and the both-direction ablations; do not split back into mechanism plans; take the D21 stop-rule decision at a kill review |
| One-ply settle proofs are missing from the writer | Pending settled values are stored at the child key and emitted; a unit test and a slow proof round-trip pin it |
| Simulation hides cost in the wrong counter | `sim_nodes`/`settle_nodes` separate; D11 `child_evals`/`expansions` unchanged; wall reported |
| Null-move position leaks into the search | The null clone is local to `ThreatSignature`; a test wrapper asserts no key/table/line/event access; no proof can contain it |
| History/seed breaks determinism | Fixed arrays, no hash-map iteration, seed only after metric/class/history; same-seed repeat tests |
| Guide changes a decided value | Unit test: guide writes only fresh `Open`; one-ply settle is the only new exact closure and is verified by a real legal move |
| `Open` saturates at `INFINITY` again | Every guide arithmetic path clamps to `OPEN_MAX`; extreme-combination test |
| Census picks cells after seeing atomi | Census is Phase 0, before any atomi stack run; set hash pinned; driver refuses mismatch |
| Runtime of 24 cells × 4 seeds × ablations | Separate deterministic eval caps for gate/ablation; one process at a time; census budgets recorded |
| GPL contamination from reading provari | Ideas only; oracle is an external process; no source, comments, constants-as-code or proofs committed |
| `--guide hand` becomes an accidental default before M5 | Default stays `none`; bench row names the stack; M5 decides the default |
| Frame/prefetch changes erase M1 gains | Per-node check on `core`/10 M at the end, same session; report a regression |

## Report requirements (`report3.md`)

Headline, provenance, exact commands, results, gate table (G1–G8), verdict,
not measured, problems and findings, missing tests, next steps, delivered
files, and a closure block with a copy-pasteable next-session prompt.

Specifically:

- the census table and the frozen `guidance-v1` hash;
- the D21 gate table: per cell/seed `expansions`, `child_evals`, wall, proof
  verdict; the `easy-win` 145,046 comparison and the `core` 10 s result;
- the guidance-set coverage/geomean nodes and walls;
- the D14 proof results for every seed (identical counters and both checkers);
- the no-guide identity result (m1 vs bare) and the seed determinism result;
- the both-direction ablation matrices and the keep/remove decision;
- what each of the nine mechanisms did, including the individually negative
  levers; no claim that a single lever caused the stack result;
- the M1 per-node regression check and the `settle_nodes`/`sim_nodes` cost;
- any closure-proof/writer issue found, and any deviation from this plan;
- the bench row with **no goal claim**.

## Deliverables

- **Engine:** `src/guide/{mod,class,eval,threat,history,simulate}.rs`; updated
  `src/search/{mod,frame}.rs`, `src/search/value.rs` (`Child.class`),
  `src/cli.rs`, `src/main.rs`, `src/lib.rs`.
- **Drivers:** `bench/run_bench.py` seed/guide flags as needed;
  `docs/plans/prover/measurements/plan3/{run_guidance.py, README.md}`.
- **Measurements:** `docs/plans/prover/measurements/plan3/{census.json,
  guidance-v1.tsv, env.json, guidance.json, guidance.csv, wall.json,
  ablation.json, bench.json}`.
- **Docs:** `bench.md`, `AGENTS.md`, `initiative.md`, `docs/plans/README.md`,
  `report3.md`.
- **Scratch, never committed:** `/tmp/atomi-m1`, the census/proof files, the
  provari scratch clone and binary.
