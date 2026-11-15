# Report 38 — item 52 Phase A: a bounded-length pass, and the length axis is a real tree lever

Plan: [`plan38.md`](plan38.md) (item 52 Phase A). Initiative:
[`initiative.md`](initiative.md), D4/D14/D19/D45/D49, [`gotchas`](gotchas.md).
Data: [`measurements/plan38/`](../../../measurements/plan38/README.md). No
default change, no `bench.md` row, no `T_4` claim, no provari source.

## 1. Verdict

**The bounded pass is implemented, sound, and positive on `deep-win`.** At
120 s, `L = 20/22/24` prove with lengths **20/22/24** against the control's
**36**. `L = 24` is also faster (**37.5 s** vs **53.3 s**); `L = 20/22` are
slower (58.4/59.2 s). The `L = ∞` control is tree-identical to the pre-plan
search. Every bounded proof is D14-accepted by both checkers. The length axis
is therefore a **real tree lever**, not a quick win at every `L`: the plan's
"if it proves" branch is open, and Phase B (iterative deepening) now has a
measured target.

| Gate | Bar | Result |
| --- | --- | --- |
| G-38a control | 5,803,257 / 155,236,740 / 36 at 120 s; 5,000,034 at 5 M | **PASS** |
| G-38b bounded proofs | D14-accepted by both checkers | **PASS** — `L = 20/22/24` deep-win + rule50 edge |
| G-38c table rule | no bound-dependent refutation can enter the table | **PASS** — `Value::Bounded` is never stored; algebra test |
| G-38d measurement | `L = 20/22/24` on `deep-win` at 5 M and 120 s | **PASS** — lengths 20/22/24; `L = 24` 37.5 s |
| G-38e gates | `make test` + `make test-full` | **PASS** — including `make veto` |

## 2. Mechanism

- **`Value::Bounded`** is a new value: "no attacker win within the remaining
  length bound at this node". It is **not** a refutation. `pn_dn` maps it to
  `(INFINITY, 0)` so the search treats it as decided, but `Tables::store` and
  `Tt::store` ignore it. It is never emitted as a proof event.
- **Bound propagation.** `Engine::mid` carries `bound: Option<u16>`. An OR
  move spends one attacker move (`child_bound = b - 1`); an AND reply spends
  none (`child_bound = b`). `None` is the unbounded control.
- **Bounded algebra.** `or_value_bounded` / `and_value_bounded` are the
  bound-aware counterparts of `or_value` / `and_value`. A proved child longer
  than the allowance is treated as `Open(1, 1)`, not as a bounded refutation:
  a long proof is a genuine win, but the bounded search must keep looking for
  a shorter one. An OR node is `Bounded` only when every child is `Refuted`
  or `Bounded` and at least one is `Bounded`; an AND node is `Bounded` when a
  reply is `Bounded` and no reply is genuinely `Refuted`. A genuine
  `Refuted` always wins over `Bounded`.
- **Frame entry and auxiliary passes.** `apply_bound` filters the values
  `Frame::enter` classified (table proofs, settled proofs, terminal wins) and
  the item-15 group members, then re-syncs the virtual group child.
  `bound_value` is also applied to every child `mid` return. The exact
  null-threat replay and sibling replay can install a proof after the first
  filter, so the bound is re-applied after those passes too; otherwise the
  selector sees a `Proved` child while the bounded algebra sees `Open`, and
  the search is not the bounded search it claims to be (see §4).
- **CLI.** `--max-length L` (positive `u16`, default off) sets
  `Limits::max_length`. `Exit::Length` (`"length"`) reports an exhausted
  bound. `--max-depth` is unchanged and remains a censor.

## 3. Measurements (one process at a time)

Cell `deep-win`, `--guide hand --memory 192 --seed 0`, same binary. Full
counters in [`runs.json`](../../../measurements/plan38/runs.json).

| Run | exit | length | child_evals | expansions | ms |
| --- | --- | ---: | ---: | ---: | ---: |
| control 5 M | evals | — | 5,000,034 | 168,910 | 1,690 |
| L=20 5 M | evals | — | 5,000,034 | 168,910 | 1,694 |
| L=22 5 M | evals | — | 5,000,034 | 168,910 | 1,682 |
| L=24 5 M | evals | — | 5,000,034 | 168,910 | 1,709 |
| control 120 s | proved | 36 | 155,236,740 | 5,803,257 | 53,341 |
| L=20 120 s | proved | 20 | 166,682,537 | 6,206,650 | 58,436 |
| L=22 120 s | proved | 22 | 169,409,413 | 6,216,502 | 59,198 |
| L=24 120 s | proved | 24 | 109,715,277 | 4,109,113 | 37,503 |

The 5 M rows are tree-identical to the control because the unbounded search
has not yet found a proof there; the bound only changes the tree once a
too-long proof exists. The 120 s rows are the informative ones.

**Bounded proofs (G-38b).** The untimed `--seconds 600 --proof-out` runs
reproduce the timed `child_evals` exactly (166,682,537 / 169,409,413 /
109,715,277) and both checkers accept all three proofs:

| L | length | positions | own validator | provari check |
| ---: | ---: | ---: | --- | --- |
| 20 | 20 | 88,351 | valid | valid |
| 22 | 22 | 93,053 | valid | valid |
| 24 | 24 | 70,745 | valid | valid |

The small rule50 edge (`k7/8/8/8/8/7Q/8/K6R w - - 0 1`, `--max-length 2`)
also round-trips both checkers; its proof `limit` is 97, not the bound.

## 4. Problems and scope changes

- **The first implementation was not the bounded search it claimed to be.**
  It applied the bound at frame entry and after a child return, but the exact
  null/sibling replay could reinstall a too-long `Proved` child afterwards.
  `node_value` filtered it, but `select`/`child_thresholds` saw `pn = 0` and
  drove the search with the wrong numbers. That version timed out at
  `L = 20/22/24` (507–685 M `child_evals`). Re-applying `apply_bound` after
  the auxiliary passes fixed the inconsistency and the same runs prove. The
  lesson is recorded because it is a real measurement trap: a bounded pass
  must keep the selector and the value algebra on the same child view.
- **`L = 20/22` are slower than the control despite shorter proofs.** The
  bounded search spends more `child_evals` (166–169 M vs 155 M) to find the
  shorter proof. `L = 24` is the only measured wall win (37.5 s vs 53.3 s,
  1.42×). A Phase B deepening driver should not assume monotonicity in `L`.
- **No default, no `bench.md` row, no `T_4` claim.** Phase B (iterative
  deepening `L = 12, 14, …`) is not started; the plan makes it conditional on
  Phase A proving, which it did.

## 5. Missing tests / open parts

- No test drives a bounded search through a *stored longer proof* and then a
  shorter proof in the same table. The algebra, the `L = ∞` control and the
  deep-win proofs cover the soundness direction; a same-table differential
  would be the next instrument if Phase B is scoped.
- No test covers the item-15 group under a bound. The group is default off
  and the bounded measurements use `--guide hand` (group off); `Group::sync`
  was extended to propagate `Bounded`, but it has no dedicated bounded test.
- The bounded pass has no iterative-deepening driver (Phase B) and no
  `--max-length` output field; the CLI contract is unchanged.
- The `L = 20/22/24` proof files are 5.6–7.4 MiB and stay in `/tmp`; their
  sha256 and both checker results are in
  [`bounded_deepwin.validation.json`](../../../measurements/plan38/bounded_deepwin.validation.json).

## 6. Next steps

1. **Scope Phase B** (iterative deepening `L = 12, 14, …`) behind an opt-in
   flag, with the same D14 protocol. The measured target is `L = 24` at
   37.5 s; the open question is whether a deepening schedule reaches a short
   proof faster than a single `L = 24` pass.
2. **Item 51's tree-change gate** now has a measured tree lever: length
   36 → 24 at `L = 24`, with a 1.42× wall win. Use it as the gate's first
   positive row.
3. Keep `--max-length` as an opt-in measurement flag; it is not a product
   default and has no `bench.md` row.
