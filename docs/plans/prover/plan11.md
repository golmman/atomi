# Prover Plan 11: item 20 — bare-tree coverage probe and product-configuration decision

Initiative: [`prover`](initiative.md), backlog **item 20** (proposed), decisions
**D6**, **D24**, **D27**. Drafted 2026-11-10 after [`report10`](report10.md).
This is the plans 9–10 kill review plus the next plan. It does not implement;
it is the plan-session deliverable.

## Kill review (plans 9–10, fixed prompt)

- **Absolute distance.** Tier 1 coverage is **1/3** at 120 s: `easy-win` 3/3
  (5.765 s, 54.9×), `core` 0/3 (≥ 240.0×), `deep-win` 0/3 (≥ 18.2×). The
  guide-free tree is better on the two measured cells: bare `core` proves at
  60 s in 17.1 s (34.6×) and bare `easy-win` at 10 s in 1.976 s (18.6×).
  Per-expansion cost is competitive (bare `R` 1.19/1.33); the gap is tree size
  and the hand stack's auxiliary work (92–94 % of the product wall, report10).
- **If we started today.** We would not build the hand-guide stack as the
  default. On the measured cells the bare tree is faster and has more coverage;
  the hand stack is a sunk cost that stays behind flags, not the product.
- **Which plan to cancel.** Item 18 (`RESTART-GEN`) is negative; `REPLAY-BOUND`
  was never started; the hand-guide/auxiliary line is closed. Cancel the
  **portfolio driver** as dominated (below). Do not cancel the bare tree.
- **Prior-art search.** No new prior art changes the picture: provari hand is
  the reference (0.500 s `core`, 6.590 s `deep-win`), and our bare tree is
  17.1 s on `core` with `deep-win` unmeasured. The gap is tree size, not a
  missing published technique.
- **Smallest next experiment.** One bare-tree `--seconds 120 --memory 192` run
  on `deep-win`, same-session provari hand reference, D14 if it proves.

## Premise to test

Report10 recommends a **portfolio** (bare + hand, first proof). The kill review
rejects that formulation as **dominated**: the bare tree is faster and has more
coverage than the hand tree on both measured cells, and the hand tree's only
possible contribution is `deep-win`, where it is censored at 120 s (plan9). A
portfolio therefore cannot add coverage over bare; it only adds a driver. The
decisive unknown is **bare `deep-win` at 120 s**. If bare proves it, the product
configuration should be **bare**, giving Tier 1 3/3. If it censors, no existing
configuration gives 3/3 and the goal is blocked on `deep-win` tree size.

## Absolute distance (report9/report10, same session)

| Cell | product 120 s | bare | provari hand | product factor |
| --- | --- | --- | --- | ---: |
| `core` | 0/3, 1.96 M exp | 3/3 at 60 s, 17.1 s | 0.500 s / 342,289 | ≥ 240.0× |
| `easy-win` | 3/3, 5.765 s | 3/3 at 10 s, 1.976 s | 0.105 s / 72,523 | 54.9× |
| `deep-win` | 0/3, 2.03 M exp | **unknown at 120 s** | 6.590 s / 4,574,912 | ≥ 18.2× |

Bare `core` at 10 s is censored (5.38 M exp); bare `easy-win` is 18.6×. Tier 2
is not near on any configuration; this plan is a coverage/formulation decision,
not a wall claim.

## Scope

**In:** the kill-review record; item 18 closure; a bare-tree 120 s coverage
matrix (3 cells × seeds 0–2), same-session provari hand references, D14 for
every proved cell; the product-configuration decision; the joint docs update.

**Out:** a portfolio driver; `RESTART-GEN`; `REPLAY-BOUND`; hand-guide or
auxiliary tuning; any budget above 120 s; M4/`planX`; new tree or guide
mechanisms; default-flag changes before the decision.

## Design

### Phase A — bare-tree coverage matrix (measurement only)

`--guide none` (the m1-identical configuration), `--seconds 120 --memory 192`,
three cells × seeds 0–2, one process at a time. Record exit, wall,
`expansions`, `child_evals`, length and proof verdict. Re-run provari hand in
the same session for the factor column. For every proved cell run D14
(`--seconds 600 --proof-out`, same flags/seed); require equal `child_evals` and
both checkers. Record `guidance-v1` seed-0 coverage as a diagnostic, not a gate.

### Phase B — decision

- **If bare is 3/3 at 120 s:** adopt bare as the product configuration (owner
  decision), record Tier 1 coverage 3/3, and update `bench.md`/`initiative.md`.
  Tier 2 remains the wall goal; report the bare 10 s factors on shared cells.
- **If `deep-win` censors:** report the exact censor and stop. Do **not** build
  a portfolio (hand `deep-win` also censors, so it cannot add coverage), do not
  raise the budget, and do not reopen the auxiliary line. The next lever is a
  new tree mechanism, which needs a new plan and owner decision.

## Gates

- **G1 data.** 9 bare rows + same-session provari references + D14 rows for
  every proved cell; parsed JSON, `env.json`, one process at a time.
- **G2 determinism.** No engine change; the same binary, FEN, flags and seed
  reproduce the counters.
- **G3 goal.** Tier 1 requires 3/3 at 120 s with 3/3 D14-accepted proofs. A
  censored cell is reported, never converted; the budget is never raised.
- **G4 soundness.** No refutation or never-prove; `make test-full` and
  `make veto` green; every claimed proof accepted by both checkers.
- **G5 docs.** `initiative.md` item 18/20 status and priority, `bench.md`
  history, `report11.md`; no GPL dependency, raw transcript, provari artifact,
  proof file or `__pycache__`.

## Tasks and stop points

- **A** driver + bare coverage matrix + D14; stop if provari controls are
  missing (INCONCLUSIVE, never a frozen reference).
- **B** decision + docs; stop if `deep-win` censors (report blocked, no
  portfolio).
- **C** `report11.md` with both the coverage matrix and exactly one
  recommendation.

## Risks

| Risk | Mitigation |
| --- | --- |
| Bare `deep-win` censors at 120 s | Report the censor; the next lever is a new tree mechanism, not a portfolio; do not raise the budget |
| Bare proof is too large for D14 | Report proof size; the proof table fails closed; do not raise `--memory` |
| Bare is worse on some `guidance-v1` cells | The bench is the 3-cell set; `guidance-v1` is a diagnostic, not a gate |
| Product change is read as goal-shopping | Coverage is stated first; Tier 2 is retained; no budget change; the owner decides |
| The hand stack is discarded without a record | It stays behind flags; the report records its measured loss on the bench cells |

## Formulation options (ideation)

- **A (recommended): bare as the product.** Gain: 3/3 coverage if `deep-win`
  proves; wall improves on `easy-win`/`core`. Cheapest experiment: one bare
  `deep-win` 120 s run.
- **B: portfolio bare + hand.** Gain: none over bare on current evidence (hand
  adds no coverage); cost: a driver and a second configuration. Reject.
- **C: keep hand and add a new tree mechanism for `deep-win`.** Gain: unknown;
  cost: a new stack. Not this plan.
- **D: change the goal to 2/3 or a 600 s budget.** Reject: goal-shopping.

## Deliverables

- Data: `measurements/plan11/{README.md,env.json,coverage.json}`.
- Docs: this plan; `report11.md`; `initiative.md` item 18/20 status and
  priority; `bench.md` history. No ledger row unless Tier 1 passes.
- Engine: no change.
