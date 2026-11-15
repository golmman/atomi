# Plan 39 — item 52 Phase B (SHORT-PASS): opt-in iterative deepening

Read only: [`report38`](report38.md), [`plan38`](plan38.md), the plan38
[`measurements`](measurements/plan38/README.md), [`gotchas`](gotchas.md)
§Search, D47/D49, items 47/51/52, [`analysis1`](analysis1.md) §3 R25. The
pinned `atomic_solver` clone (`5f85133…`, read-only, outside the repo) is read
for ideas: `docs/plans/ultimattt/{plan4,plan5,report4,report5}.md` and
`src/search/dfpn/{run,core,children}.rs`. No code/text copied. Pins:
`atomic-movegen` 2.8.0; oracle `48686bc`.

## 1. The question

Phase A on `deep-win` (`--guide hand --memory 192 --seed 0`, 120 s): control
length **36** at **53.3 s**; `L = 24` length **24** at **37.5 s** (109.7 M
`child_evals`); `L = 20/22` length 20/22 but **58.4/59.2 s** (166.7/169.4 M).
The curve is **not monotone**. Phase B asks: **does a deepening schedule reach
a short proof faster than a single `L = 24` pass?** The target is 37.5 s; a win
is a tree lever, not a `T_4` route (10 s is still 3.2× away). Linear
`12,14,…,24` is risky because it spends its budget at `L = 20/22`, the slowest
passes; Phase 0 prices `L < 20` first.

**Legacy lesson (our words).** `atomic_solver`'s `ultimattt` line used
**work-bounded chunks**: a child-eval budget per probe, table/ordering state
persists, only path state resets, a shallow probe cannot eat the deadline. Its
`remaining_depth` guard refuses an unsolved bound from a shallower horizon; its
schedule is **doubling**; its first proof is an upper bound a later refinement
shortens. Mapping: a per-iteration `child_evals` cap; proofs and genuine
refutations persist, but **bound-dependent `Open` pairs are discarded** between
iterations; the `L` schedule is measured; the driver stops at the first proof
(refinement is a separate option, §7).

## 2. Phase 0 — read-only `L`-curve probe (no code)

Single bounded passes on `deep-win`, same flags, `L = 12/14/16/18`, at
`--evals 5000000` and `--seconds 120`; reuse the committed `L = 20/22/24`
rows. Record `exit`, `length`, `child_evals`, `expansions`, `ms`,
`proof_entries`, and 5 M tree identity.

**Stop rule.** Proceed only if the `L < 20` passes leave the `L = 24` pass its
budget: cumulative `child_evals` ≤ **0.5×** the `L = 24` pass (≤ 54.9 M) **and**
cumulative wall ≤ **0.5×** 37.5 s (≤ 18.8 s), or one proves. If any single
`L < 20` pass exceeds the `L = 24` pass, stop: deepening cannot win. Ambiguous
→ cheapest schedule.

## 3. Phase 1 — the driver (only if Phase 0 passes)

**CLI.** `--deepen L0,L1,...` (strictly increasing `u16`, default off) and
`--deepen-budget N` (per-iteration cap for all but the last `L`, default 20 M;
`0` = no cap). `--deepen` and `--max-length` are mutually exclusive;
`--deepen` off is the unbounded control.

**Driver.** In `Engine::run`, for each `L`:
1. stop if the global deadline or `--evals` cap fired;
2. set `max_length = Some(L)`;
3. `tables.clear_open()` — zero `Open` TT slots, keep path-independent
   `Refuted`; proofs, filter, null cache and history persist. Proofs and
   refutations are bound-independent; a bound-dependent `Open` is not;
4. reset `line` (empty), `iteration_stop = false`, set
   `iteration_cap = child_evals + budget` (the last `L` is uncapped);
5. call `mid` until the root is decided or the iteration cap fires.

`Proved` → return; `Refuted` → return (a root refutation under a bound is
genuine: every child is genuinely refuted, none `Bounded`); `Bounded` → next
`L`; cap → next `L` (recorded, never a refutation). List exhausted: `Length`
after a `Bounded` last iteration, `Evals` after a capped one, `Timeout` if the
deadline fired.

**Cap.** Add `iteration_cap: Option<u64>` and `iteration_stop` to `Engine`;
`mid`, after the global cap check, returns `Open` when the iteration cap fires.
The driver clears both between iterations.

**Counters.** Add `deepen_iterations` and a per-iteration (`L`, exit,
`child_evals`, `expansions`, `ms`, `length`) record under `--stats`; the final
result and `L = ∞` are unchanged.

## 4. Schedules to measure

Control: single `L = 24`. S1 linear `12,14,16,18,20,22,24`; S2 step-4
`12,16,20,24`; S3 doubling `12,24`; S4 `16,20,24` if `L = 16/18` is cheap.
Each at 120 s, seed 0, one process at a time; per-iteration and total
wall/`child_evals`/length. The driver stops at the first proof; no refinement.

## 5. Gates

- **G-39a Phase 0** — `L = 12/14/16/18` at 5 M and 120 s, parsed, `env.json`,
  README.
- **G-39b control** — `--deepen` off is tree-identical to G-38a: 5,803,257 /
  155,236,740 / 36 at 120 s; 5,000,034 at 5 M.
- **G-39c single iteration** — `--deepen 24` (last `L` uncapped) reproduces
  Phase A `L = 24`: 109,715,277 `child_evals`, length 24, wall within ±5 %.
- **G-39d table rule** — `clear_open` keeps `Refuted` and proofs, drops
  `Open`; unit test; no bound-dependent `Open` survives into a new `L`.
- **G-39e D14** — every proof accepted by `atomi validate` and `provari check`.
- **G-39f measurement** — four schedules at 120 s on `deep-win` with
  per-iteration counters. A schedule **wins** only if it proves with wall
  < **0.85×** the single `L = 24` control (≤ 31.9 s) and length ≤ 24;
  otherwise the driver stays opt-in (D49).
- **G-39g gates** — `make test` + `make test-full` (including `make veto`).
- **G-39h boundary** — no default, no `bench.md` row, no `T_4` claim, no
  provari source.

## 6. Scope, effort, risks

**In:** Phase 0; driver, `clear_open`, cap and counters, CLI, tests;
`measurements/plan39/`; `report39.md`. **Out:** default, refinement,
bound-keyed table, guide tuning, `bench.md` row, `T_4` claim, provari source.
Effort ≈ 1–2 days.

**Risks:** (i) small `L` expensive — Phase 0 stop rule; (ii) `clear_open` scan
— 48 MB TT ≈ 3 M slots, expected < 10 ms/iteration; (iii) a cap can cut a
pass that would have proved — report `Evals`, never a refutation; (iv) a stale
`Open` could mislead the selector — G-39d; (v) the first proof is an upper
bound (`length ≤ L`); (vi) the `L = ∞` control must stay tree-identical —
G-39b. No bound tag: the 16-byte slot has no spare field; stealing `work` bits
would break G-39b.

## 7. Ideation (one formulation change)

- **A (recommended): driver + `clear_open`.** Gain: reaching `L = 24` in
  < 31.9 s is a 1.18×+ tree lever; cost: cap and scan. Cheapest experiment:
  Phase 0.
- **B (formulation change): bound-keyed refutation table.** Store "no win
  within `L`" under `(key, L)` in a separate table, so `L' > L` can use it as
  a lower bound and skip re-searching (R25's salted key). Gain: 2–4× less
  cumulative work if the `L < 20` passes re-search the same nodes; cost: a new
  table and key dimension. Cheapest experiment: Phase 0's `--stats` key sets —
  `L = 12` vs `L = 16` overlap > 50 % makes B worth a plan.
- **C: refinement after a first proof** (legacy `refine_sppv` analogue).
  Find `L = 24`, then binary-search `L = 20/22`. Gain: length 20 vs 24; cost:
  cold `L = 20/22` are slower, so wall likely worse. Cheapest experiment: warm
  `L = 20` after a completed `L = 24` vs cold `L = 20`; warm < 0.5× cold makes
  C worth a plan.
- **D: work-bounded chunks at fixed `L = 24`** (legacy pure work loop). Gain:
  none on wall; it only helps if the driver must switch `L` mid-search.
  Cheapest experiment: the same `L = 24` pass in 20 M chunks must be
  tree-identical to the single pass.

## 8. Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan39.md`. Execute
   Phase 0 as written: single bounded passes on `deep-win` at `L = 12/14/16/
   18`, 5 M and 120 s, same flags as plan38, and apply the stop rule. If it
   passes, implement Phase 1 (`--deepen`, `--deepen-budget`, `clear_open`,
   per-iteration counters), prove G-39b/c/d, then measure S1–S4 at 120 s with
   D14 for every proof. Data in `measurements/plan39/`, report `report39.md`;
   no default, no `bench.md` row, no provari source."
2. If a schedule wins: scope item 51's gate and a D49 default decision.
3. If Phase 0 or Phase 1 loses: close Phase B negative; the next tree axis is
   item 48 or the owner's `T_4`/`T_5` ruling; do not reopen per-node micro-plans.