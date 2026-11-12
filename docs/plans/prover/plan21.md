# Plan 21 — item 31: the ledger states its cap

**Item:** 31 (ledger budget convention + the first above-120 s row).
**Owner decision:** D34 (amends D30's 2026-11-11 ledger-split clause).
**Predecessor:** [`plan20`](plan20.md) / [`report20`](report20.md).

## Premise (challenged by the owner, accepted)

`bench.md` currently reads as if the 120 s cap were policy: the goal
staircase, "Current position: `T_2`" and a ledger-local "D30 amended
2026-11-11" paragraph all live there, while the `Cap` column already says the
budget is per row. A column that varies per row cannot also be a property of
the document. Resolution:

- **SSOT for the goal, the rungs and their budgets:** [`initiative.md`](initiative.md)
  (D30 already defines `T_1`–`T_5` there).
- **`bench.md`:** a results ledger only — the metric, the bench set, the
  protocol, the reading rules and one row per executed plan. A row records the
  budget it was measured at; its factor needs no budget at all (provari's side
  is fixed at `--seconds 10`).
- A factor is therefore never "censored at the ledger's cap"; a row is
  `censored (≥ cap/ref)` **at its own cap**, and a longer run is a row, not a
  conversion.

## What this plan does

1. **Measure** the plan20 stack at a 600 s cap, same session, with the
   reference re-measured in the same session (`measurements/plan21/`):
   provari hand × 3 repeats per cell at `--seconds 10`, atomi `--guide hand`
   (AUX-V1) × seeds 0–2 at `--seconds 600 --memory 192`, then **D14 for every
   proved row** (untimed `--seconds 600 --proof-out` rerun, `child_evals`
   match, `atomi validate` + `provari check`). Report20's `deep-win` 3/3 had
   no D14 — it was a gate observation (`d14.json` covers `core`/`easy-win`
   only) — so it cannot become a row before this step.
2. **Rewrite `bench.md`** as a ledger: delete the staircase table, the
   "Current position" line and the D30-amendment convention paragraph; add a
   one-line pointer to `initiative.md`; reword the `Cap` column (budget the
   row was measured at, part of the result) and reading rule 6 (coverage is
   stated in every row; geomeans are compared **only within one cap** —
   the geomean is non-monotone in the cap: 10 s/1 cell ≈ 7.0, 120 s/2 cells
   ≈ 10.8, 600 s/3 cells ≈ 14.0).
3. **Append one row** for the plan20 stack at Cap 600 s, plus its history
   line; keep the existing Cap-120 row untouched.
4. **Record D34** in `decisions.md` superseding D30's ledger-split clause,
   state the split in `initiative.md` (goal section + the `Ledger:`
   convention line), and drop `bench.md` from `scripts/check_goal.py`'s
   `FILES` (the goal sentence is stated once per *normative* doc; `bench.md`
   stops being one).
5. **No engine change.** No default, no flag, no search code.

## Gates

- `make test`, `make lint` (the goal gate now scans three files, not four),
  `make veto` — required because the row makes new proof claims.
- Every claimed win on the new row is D14-accepted; a failure is recorded
  and the cell reported censored at 600 s, never converted.

## Why not raise the product budget

Raising a budget is a goal change (D30/D28 "the 120 s budget is not raised");
it belongs to `initiative.md` and an owner decision. This plan only stops the
*ledger* from hiding a measured result behind a cap it did not choose.

## Out of scope

Item 29 (plain-move grouping), item 30 (`proof_full` headroom), any new
`deep-win` mechanism, and any change to the rungs or the `T_5` threshold.
