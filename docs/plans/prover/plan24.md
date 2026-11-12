# Prover Plan 24: item 29 — plain-move grouping by default (GROUP-V1)

Initiative: [`prover`](initiative.md), backlog **item 29**, decisions **D14**,
**D18**, **D21**, **D24**, **D26**, **D30**, **D32**, **D33**, **D34**,
proposed **D36**. Drafted 2026-11-12 after [`report23`](report23.md). Plan
only.

## Kill review (fixed prompt)

1. **Distance.** `T_3` (report23: all three at 120 s, 9/9 D14). `T_4` needs
   all three at 10 s; `core` (8.3 s) and `easy-win` (0.9 s) already fit, so
   the rung is **`deep-win` alone: 105.3 s → 10 s, 10.5×**. Decompose the wall
   factor as tree × per-expansion (report23 `gate.json`, seed-1 medians):
   `core` 1.01 × 15.5 = 15.7, `easy-win` 0.59 × 11.5 = 6.8, `deep-win`
   1.18 × 13.5 = 15.8. **The tree is at parity with provari's reported nodes
   (which include simulation steps); the remaining gap is per-expansion cost,
   11.5–15.5×.** Tree levers cannot reach `T_4`; item 29 is the last
   authorised tree lever and is also a per-node lever (plain moves never
   scored).
2. **Would we build this?** Yes, but only as the last authorised item and the
   cheapest per-node probe. The next plan must be per-node cost (item 33
   below), not another tree lever.
3. **Cancel?** Nothing new. Item 29 is re-scoped, not cancelled: its gate is
   per-expansion wall, not the stale 20 M expansion count (D33). If it fails,
   close it and go per-node. PDS-PN/PN², restarts, portfolio, threads,
   tablebases stay rejected.
4. **Prior art (delta).** Pin unchanged, so only changed rows: R1–R6, R18–R19
   and R11–R13 are now adopted (items 26–28); **R9 is the only remaining
   `add` row**. The per-node gap is not a mechanism row; it is the new
   first-order finding.
5. **Smallest next experiment.** Phase A: measure the plain-move fraction and
   the per-child cost on the current default, then the score-free group.

## Decision

**GROUP-V1**, one D21 stack: (a) **plain children are never scored** — skip
`start_numbers` for them; the group's `(9, #plain)` is the only start; (b)
**`group` is on by default**. The group stays **virtual** (gotchas): members
keep real keys, the group is never keyed or stored. Gate: 120 s wall, seeds
0–2, three cells — no cell regresses > 10 % vs the current default and
`deep-win` median improves ≥ 10 %; D14 on every proved seed. If the gate
passes, re-check `T_3` and record the `T_4` distance; `T_4` only if all three
prove at 10 s. **Proposed D36** records the decomposition and authorises item
33 after item 29.

## Premise

`Frame::enter` scores every fresh child with `start_numbers`, then partitions
the plain ones into the group and **discards their starts** (the group's start
is hardcoded `(9, n)`). That is pure per-node waste on the attacker's plain
moves. Plan6 measured the group with the old numbers algebra and scored
members: `core` 0.96×, `easy-win` 1.15–2.31× (a regression). The algebra
changed in item 26, so the group must be re-measured; the score-free change is
the part plan6 did not test.

## Scope

**In:** Phase A counters/probe; score-free grouping; the default; the gate;
the 120 s matrix; D14; `report24.md`; one `bench.md` row at Cap 120 s.

**Out:** item 33 (per-node cost); item 30; search base; budgets; goal/rungs;
provari code, text or proofs.

## Design

### Phase A — diagnosis (read-only)

Add permanent counters `plain_children` and `group_children` (reuse
`child_evals`, `settle_nodes`, `sim_nodes`, `aux_nodes`). On the current
default, 120 s, seeds 0–2, three cells: record plain fraction, `child_evals`/
exp, `aux`/exp, ns/exp. This sizes the score-free cut and the next per-node
plan. No default change.

### Phase B — score-free grouping

In `Frame::enter`, skip the guide's `start_numbers` for plain children that
are still `Open`; keep their `(1, 1)` start. A plain child already decided by
`rules::status` keeps that value. The group's `sync` already uses `(9, #plain)`
until a member is installed (`touched`/`any_decided`), so the value algebra is
unchanged.
Flip `GuideConfig::product().group` to `true`; keep `--guide-group` for
measurement. The virtual-group invariants (real member keys, proof mapping,
pending retention) are unchanged; the existing tests pin them.

### Phase C — the gate

Control: current default (`group` off). Candidate: `group` on, score-free.
120 s, 192 MB, `--guide hand`, seeds 0–2, three cells, one process at a time.
**Primary:** `deep-win` median wall ≤ 0.9× control; `core`/`easy-win` ≤ 1.1×
control. **Secondary:** child evals to proof at 600 s (D33 tree instrument),
ns/exp and aux/exp, no regression > 10 %. D14 on every proved seed. If the
gate passes, re-check `T_3` (all three at 120 s) and record the `T_4`
distance; `T_4` only if all three prove at 10 s.

### Phase D — if the gate fails

Record the control/candidate rows and both directions of the score-free
ablation; no default change; close item 29. The next plan is item 33.

## Option that changes the formulation

**Item 33 — per-node cost.** The wall is 11.5–15.5× per expansion; the tree is
at parity. Profile the per-child work (board builds, movegen, settle, sim,
replay) on `deep-win` at 20 M evals and cut the dominant term. Gain: `T_4`
needs 10.5× on `deep-win`; a 2× per-node cut plus item 29's ~1.4× is still
short, so the plan must target the dominant per-child cost, not a constant
factor. **Cheapest experiment:** Phase A's counters plus a `perf` profile of
`deep-win` at 20 M evals.

## Gates

- **G1 data.** `env.json`, `phase_a.json`, `gate.json`, driver + README under
  `measurements/plan24/`; parsed only, one process at a time, same-session
  control.
- **G2** score-free + default change; the virtual-group tests still pass.
- **G3** the gate (Phase C).
- **G4 soundness.** `make test`, `make lint`; `make test-full` + `make veto`
  (search/guide change); every claimed win D14-accepted; licence gate
  unchanged.
- **G5 docs.** `report24.md`, item 29 status, one `bench.md` row at Cap 120 s
  only after G3, `history.md` line.

## Risks

- **The group regressed in plan6**; the new algebra may not fix it. The gate
  is a real gate, not a foregone conclusion.
- **Score-free members may change the tree**: the group's start is the only
  signal until a member is installed. The ablation separates the two effects.
- **D33 traps**: use child evals to proof, not 20 M expansions; never compare
  `unique_keys` at the cap.
- **The group is virtual** (gotchas): do not key, look up or store it; a
  group-proved parent must carry the selected member's move and key.

## Deliverables

`measurements/plan24/` (driver derived from plan23); `report24.md`; the
score-free change and default if G3 passes; item 29 status; one `bench.md` row
at Cap 120 s; `history.md` line.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan24.md`. Execute
   Phase A (counters + diagnosis), then Phase B (score-free plain members,
   `group` default on), then Phase C exactly as written. Adopt only if
   `deep-win` median ≤ 0.9× control and no cell > 1.1×; run D14 on every
   proved seed. Do not start item 33, do not change the 192 MB label or the
   goal."
2. Gate fails: Phase D, close item 29, then item 33 (per-node cost).
3. Gate passes but `T_4` is not reached: item 33 next; record the distance.
