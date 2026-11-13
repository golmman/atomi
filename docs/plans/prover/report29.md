# Report 29 — item 36: EFFECT-CHILD-V1 stops at Phase A (`no effect`)

Plan: [`plan29.md`](plan29.md). Data:
[`measurements/plan29/`](../../../measurements/plan29/README.md) (`probe.py`,
`phase_a.json`, `env.json`, README). Decisions: D2, D11, D19, D21, D24, D26,
D30, D34, D36, D38, D39, proposed D41. Host: Ryzen 9 5950X (4 vCPU
container), one process at a time. Repo dirty at `atomi@5b8d7a8`; the owner
commits.

## 0. Verdict

**The Phase A stop rule fired: the effect decides 0.0000 % of the built
children under the product `--guide hand` stack** (0 of 1,067,066 at 5 M,
`deep-win`; 0 of 298,553 / 191,732 / 178,317 at 1 M on `core` / `easy-win` /
`deep-win`) and **0.94–1.94 %** under `--guide none`. The declared threshold
was < 10 %, so item 36 is **closed as `no effect`**: C3 was not implemented,
G-E1 was not run, there is no 120 s matrix, no D14 claim and no `bench.md`
row. The engine stays at `T_3`; the `T_4` distance is unchanged (`deep-win`
7.17×, report27).

The plan's `C2′` fallback ("if the fallback share is high, read
`MoveEffect`'s fields") cannot rescue it, and the plan anticipated the
reason: **the effect cannot answer the only question our children still
ask.** `Mover::effect` marks a child `needs_board` because its terminality is
not readable from the parent — a capture or a commoner move can explode, a
quiet move can mate or stalemate, and D19 keeps the stalemate exact — so the
question is always "is this child a mate or a stalemate?", which MG3 answers
with the same clone-and-`outcome` we are trying to avoid. The
measured ceiling says so directly: only 0.08 % of built children are terminal
at all (869 of 1,067,066), so even a *perfect* board-free terminality would
decide far below the 10 % stop rule.

Delivered instead: a consumer-side pin of the MG3 surface against our own
frozen fixture ([`tests/effects_differential.rs`](../../../tests/effects_differential.rs)),
the upstream ask that this data makes concrete
([`../movegen/handoff2.md`](../movegen/handoff2.md)), three gotchas, and the
Phase A counters as a re-runnable driver.

## 1. What was built (all measurement-only, except the differential)

- **C1 + C2 + Phase A counters.** One
  `Board::effect_context_with_state(&state)` per expanded node over the state
  pass 0 already populates, and per child (a) `EffectContext::effect`,
  (b) `rules::status_after`, (c) the counters of §2. `--stats`-gated; no
  value, class or key on that path reaches the search.
- **`rules::status_after`** (C2 as written): extinction, the non-check clock
  and — the plan's C2 risk made exact — a one-sided material screen
  (`queen_or_pawn_survives`: a surviving queen or pawn rules out
  `material_draw` for *every* clause, so the status is `Open`; its negation
  decides nothing and builds). Measured before the stop rule: the full tsv
  differential (94 positions, 1,605 legal moves, 58 illegal candidates, both
  attackers) was **empty**, and the material screen was exercised (407
  fallback children with no queen and no pawn, 182 of them real material
  draws).
- **Reverted after the stop rule** (§5): the counters, `status_after`,
  `queen_or_pawn_survives` and their unit tests. The repo's only code change
  is the differential test.
- **Kept: `tests/effects_differential.rs`.** The plan's tsv differential,
  narrowed to what survives without C2: `is_legal`/`outcome_after` equal the
  frozen `M` rows, `explodes_enemy_commoner ⇒ Some(Loss)` and a blast that
  contains the enemy commoner, `checks_enemy ⇒ child.checkers()`, illegality
  ⇒ `None`/`false`. It pins the *dependency* (a consumer-side check of MG3
  against our fixture, not the crate's corpus) at ~10 ms in `make test`.

## 2. Phase A measurement

Per-child counters over the children of every expanded node; the driver runs
the control (built from `git archive HEAD`) and the counter build twice per
config, `--seconds 600 --memory 192 --seed 0`, one process at a time.

| Cell | Cap | Guide | `needs_board` | decided | share | ceiling | tree |
| --- | ---: | --- | ---: | ---: | ---: | ---: | --- |
| `core` | 1 M | hand | 298,553 | 0 | 0.0000 | 0.0009 | identical |
| `easy-win` | 1 M | hand | 191,732 | 0 | 0.0000 | 0.0014 | identical |
| `deep-win` | 1 M | hand | 178,317 | 0 | 0.0000 | 0.0004 | identical |
| `deep-win` | 5 M | hand | 1,067,066 | 0 | **0.0000** | 0.0008 | identical |
| `core` | 1 M | none | 259,981 | 2,438 | 0.0094 | 0.0017 | identical |
| `easy-win` | 1 M | none | 179,569 | 2,639 | 0.0147 | 0.0017 | identical |
| `deep-win` | 1 M | none | 159,594 | 3,069 | 0.0192 | 0.0040 | identical |
| `deep-win` | 5 M | none | 861,939 | 16,696 | 0.0194 | 0.0061 | identical |

`ceiling` = the share of built children the *built* path finds `Won`/`Failed`
— the most any effect-based status could decide. Every decision counted is an
extinction; the clock path never fired (it needs a surviving queen or pawn
*and* `rule50_after >= 100`). `child_evals`/`expansions` are identical on all
eight configs and the counters reproduce exactly on the second run: the
instrumentation is observation only.

**Why the hand guide is at zero.** Under `--guide hand` not one of 394,969
captures at 5 M has the enemy commoner inside its blast
(`effect_blast_hits = 0`): the guide's one-ply settle proves an explosion
child a ply earlier, so the tree never contains them. `--guide none` does
(16,696 of 330,069 captures), which is where the 1.9 % comes from — and the
bare tree is not the product (D32).

**Why the fallback is 100 %.** `no_qp = 0` on every config: our
checker-material clause never needs the child board in these trees (both
sides keep pawns), so every fallback child is a mate/stalemate question. That
is precisely the crate's own "clone and `outcome`" fallback, which
`EffectContext`'s docs call temporary. Hence the upstream ask: the plan's
threshold for opening `handoff2.md` was a fallback share > 40 %; measured
100 %.

## 3. Gates

- **G-E0** (tsv differential empty, no fixture rewrite, 1 M counter identity
  × three cells): green. Differential empty over 94 positions / 1,605 moves /
  58 illegal candidates; `tests/fixtures/movegen_2.4.0.tsv` untouched
  (`git status`); counter identity 3/3 at 1 M (`child_evals` and
  `expansions`, §2). The committed `tests/effects_differential.rs` is green
  (`make test`).
- **G-E1 / Phase D**: not run — the stop rule fires first. No wall claim, no
  bench row, no D14 claim.
- **G-E2**: `make test` green (licence gate unchanged, 140 lib tests plus
  the integration tier), `make lint` green (goal check, fmt, clippy
  `-D warnings`, doc),
  `make test-full` green (`--ignored` tier), `make veto` **PASS** (8/8
  accepted, 4/4 controls rejected).
- **G-E3**: `measurements/plan29/{probe.py,phase_a.json,env.json,README.md}`.

## 4. Problems and findings

1. **The plan's C2 scope is inert by construction, and the plan's own stop
   rule was the cheapest way to find out.** Decided share 0.0 % vs a 10 %
   threshold — three orders of magnitude, not a marginal miss. The mechanism
   it wanted (board-free terminality) is real; the API that shipped cannot
   deliver it.
2. **A wrong premise in the plan's Phase A design:** it expected
   `decided_by_effect` to be non-trivial because "explosion children are the
   winning moves". Under the hand guide they are not in the tree at all — the
   settle removes them one ply earlier. The distinguishing counter
   (`blast_hits`) is what proved this, and it is the reason the zero is
   structural: 0 of 394,969.
3. **The material screen must be exact or one-sided; the plan's wording
   ("a queen/rook/knight/pawn surviving the blast ⇒ no material draw") is not
   exact.** An *N*, *R* or single *B* can coexist with `material_draw` (the
   checker's "o has only its commoner" clauses), so a surviving knight is
   not a refutation of a material draw. Only **queen or pawn** is: no
   insufficient-material clause admits either. The implementation used
   queen-or-pawn (sound and complete in that direction); the plan's
   R/N version would have turned real material draws into `Open` — a
   false-win surface. Recorded as a gotcha.
4. **`outcome_after` returns `None` for both "illegal" and "open"**, as the
   plan warned. Nothing in the adopted code reads it as illegality; the
   differential asserts illegality only through `is_legal`.
5. **`Mover::effect` computes work it discards for built children.** For the
   ~40 % of `needs_board` children that are captures (37 % at 5 M), `hash_and_rule50_after`
   is computed and the key is thrown away (the child's own key is used), and
   `gives_check` is skipped only because `||` short-circuits. Reordering the
   cheap predicates before the hash is a ~1 % wall candidate; it does not
   need MG3 and is **not** in this plan's scope (see §6).

## 5. The reverted build (re-instantiable)

Files touched and reverted to `HEAD`: `src/rules.rs` (`status_after`,
`status_of_effect`, `queen_or_pawn_survives`, two unit tests),
`src/search/frame.rs` (the per-node `EffectContext` and the `--stats` counter
block, one extra `Frame::enter` argument),
`src/search/mod.rs` (`EffectStats` + `SearchResult::effects` + `Engine::effects`),
`src/main.rs` (nine `effect_*` keys in the `--stats` JSON). The patch is
~520 lines including the tests; the counter block is 40 lines and
`status_after`+screen 90.

## 6. Open parts and missing tests

- **A child-view terminality API** is the one thing that would move this line
  from 0 % to interesting; it is filed as `handoff2.md` §MG15 with this data
  as evidence. A future plan re-runs Phase A unchanged (the driver is
  committed) against the release that ships it.
- The `Mover::effect` reorder (finding 5) is unmeasured beyond the counter
  that suggests it; it needs its own gate.
- The committed differential does not cover random play-outs (the crate's own
  `tests/effects.rs` does, inside the crate); our evidence is the frozen
  fixture by design.

## 7. Next steps

1. **The formulation decision** (the plan's follow-up option 3): PDS-PN/PN²
   or a goal re-pin under D30. Item 36 is closed; no micro-plan in between.
2. `docs/plans/movegen/handoff2.md` is filed; the next movegen integration
   plan consumes it when a release carries a board-free answer.
3. Do not reopen the per-node micro-line (report28) or EFFECT-CHILD-V1
   without a new crate API in hand.
