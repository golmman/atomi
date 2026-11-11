# Prover Plan 20: items 27–28 — AUX-V1, the auxiliary-cost stack

Initiative: [`prover`](initiative.md), backlog **items 27–28**, decisions
**D12**, **D14**, **D18**, **D21**, **D24**, **D26**, **D30**, **D32**,
**D33**. Drafted 2026-11-12 after [`report19`](report19.md). Plan only.

## Kill review (full, fixed prompt)

1. **Distance.** `T_2`. Default 120 s: `core` 2/3 (115–118 s), `easy-win` 3/3
   (7.6–16.8 s), `deep-win` 0/3. Aux off: `core` 3/3 (10.2–14.4 s),
   `easy-win` 3/3 (1.4–1.7 s), `deep-win` 2/3 (101–117 s). `T_3` is blocked
   by aux, not the tree; `T_4` needs ~1.5× `core` and ~10× `deep-win` even
   aux-free; `T_5` needs ~20–36× wall plus 3/3. 10 s: `core` 0/3, `easy-win`
   1/3 (7.58 s), `deep-win` 0/3; provari 0.518/0.133/6.523 s → `easy-win`
   57.0×, `core` ≥19.3×, `deep-win` ≥1.53×.
2. **Would we build this?** Yes: bare DF-PN, proof pipeline, validator, veto,
   numbers algebra (done), then aux cost. Not PDS-PN/PN², portfolio,
   restarts, threads, tablebases. Aux/exp is 200–217 (sim ~160, settle ~55);
   the tree is already 2.8–4.7× smaller than the product hand.
3. **Cancel?** `--shorten`/`--most`, PDS-PN/PN², portfolio, restarts, threads,
   tablebases. Conformance is closed; the hand guide is the product. Keep
   27–28. If AUX-V1 fails, the next kill review chooses a goal re-pin (D30)
   or a new search.
4. **Prior art.** DF-PN + EF-PN (Nagai), exact sibling simulation (Kawano),
   null-threat replay, board-free closures. PDS-PN/PN² rejected. Smallest
   experiment: a no-code probe of the existing exact flags on the new
   default, three cells, seed 0, aux on/off.
5. **Smallest next experiment.** Phase A below.

## Decision

**AUX-V1** is one D21 stack: **item 27** (`--guide-sim-exact` +
`--guide-null-exact` become default; greedy playout and signature prior
removed) and **item 28** (R11–R13 board-free closures), gated together on
`aux_nodes`/expansion ≤ 1 and no tree regression > 10 % on all three cells.
Item 28 is **conditional**: if Phase A's exact replays alone pass, stop at 27.
Defaults change only after G3. The 20 M instrument is replaced by **child
evals to proof at a generous cap** (report19 §2/§7.3); 20 M is secondary. This
is an instrument correction, not a goal change.

## Scope

**In:** Phase A probe; item 27's default; item 28's closures; the gate; the
120 s matrix; D14; `report20.md`.

**Out:** item 29; item 30; search-base changes; budget/memory changes; a goal
change; provari code, text or proofs.

## Baseline to reproduce (report19, evidence not claim)

- Default 120 s: `core` 2/3 (118.0/115.1 s), `easy-win` 3/3 (7.6/16.8/10.4 s),
  `deep-win` 0/3.
- Aux off: `core` 3/3 (10.2/14.1/14.4 s), `easy-win` 3/3 (1.7/1.4/1.7 s),
  `deep-win` 2/3 (101.1/117.1 s).
- Aux/exp 200–217: `sim_nodes` ~160, `settle_nodes` ~55.
- report18: `sim_exact` cuts `sim_nodes` to ~0.01/exp, settle stays ~56/exp;
  `null_exact` proves `core` at 18.3 M evals, cuts `easy-win` 32.7×,
  `deep-win` 0.99×.
- The 20 M cap is blind for censored trees: ~`child_evals`/32 expansions.

## Design

### Phase A — probe with existing flags (no code)

`--guide hand --guide-on sim-exact,null-exact` vs the default, three cells,
seed 0, `--evals 2_000_000_000 --seconds 600 --memory 192 --stats`, aux on and
off (`--guide-off sim,closures,null-exact`). Record `expansions`, `child_evals`,
`sim_nodes`, `settle_nodes`, `aux_nodes`, wall, proof, `proof_full`. G2:
`sim_nodes`/exp ≤ 1 and no tree regression > 10 %; if `aux_nodes`/exp ≤ 1
already, skip Phase B and go to Phase C. Otherwise Phase B.

### Phase B — item 28 board-free closures (R11–R13)

Replace the build + legal movegen + verify in `threat::one_ply_settle`/
`settle_screen` with bit tests, in our own words:

- **R11 exposing one-move:** a move that exposes our own king to a blast is
  not a candidate; a defence that exposes the attacker's king is a one-move
  win.
- **R12 double threat with exposure:** ≥ 2 explosion targets plus an exposed
  defender king is a win at once.
- **R13 exposed win in two:** every defence lets the attacker explode the
  defender's king is a win, not a kept node.

The existing exact path stays as fallback and differential oracle. No new
keyed position, no false win. `aux_nodes` keeps D11 meaning; bit tests are
pass-1 effect work, not aux. Tests: a differential over the fixture playouts
that every closure agrees with `one_ply_settle`; a no-false-win test;
`--guide none` identity.

### Phase C — the gate

Three cells × seeds 0–2, `--memory 192`:

- **Tree:** `--evals 2_000_000_000 --seconds 600`, aux on, stacks: default
  (control), AUX-V1. Judge child evals to proof (censored rows are lower
  bounds), no regression > 10 %. The aux-off row
  (`--guide-off sim,closures,null-exact`) is a diagnostic, not the gate.
- **Aux:** `--evals 300_000_000 --seconds 240`, aux on, stacks: default,
  AUX-V1. Judge `aux_nodes`/exp ≤ 1, wall, coverage.
- **20 M secondary:** report expansions/unique, never gate on it.

G3: no tree regression > 10 % on all three cells; `aux_nodes`/exp ≤ 1; no
refutation; D14 on every proved row.

### Phase D — only if G3 passes

`--guide hand` becomes the default + `sim_exact` + `null_exact` + R11–R13. The
greedy playout and signature prior leave the default path (flags kept for
ablation). Run the 120 s matrix (3 cells × seeds 0–2) and D14; `T_3` only if
`deep-win` 3/3 at 120 s with both checkers. Ledger row plus item 27/28 status.
If `T_3`, run the 10 s tier2 and report the `T_4` distance.

### Phase E — if G3 fails

Close items 27–28 negative: record the stack, both-direction ablations, no
per-class re-tuning. Do not start item 29. The next kill review picks the next
lever.

## Gates

- **G1 data.** `env.json`, `probe.json`, `gate.json`, `matrix.json`,
  `tier2.json`; parsed only, one process at a time, drivers + README under
  `measurements/plan20/`.
- **G2 Phase A probe.** As stated; otherwise stop before code.
- **G3 tree + aux.** As stated; judge child evals to proof first,
  `aux_nodes`/exp second.
- **G4 soundness.** `make test`, `make lint`, `make test-full` (search change
  ⇒ `make veto`), `--guide none` identity, no new GPL dependency, no provari
  proof committed, D14 on every proved row.
- **G5 docs.** `report20.md`; item 27/28 status; D33 extension (20 M blindness
  + child-evals-to-proof instrument); a `bench.md` note if the default
  changes.

## Risks

- Exact replays interact negatively (report18: `sim_null` worse than
  `null_exact` on `core`): both-direction ablations; the stack is judged
  whole.
- Closures may be unsound or incomplete: differential against the exact
  settle; D14; no false wins.
- The null replay's own work may dominate: measure in Phase A; if so, add a
  cheap null-side signature screen before `null_win` (part of item 27).
- The aux counter may miss bit-test cost: report wall and per-node cost; the
  gate is on `aux_nodes` plus wall.
- The default change is a scope decision: Phase D only after G3; the report
  records it.

## Deliverables

Data: `measurements/plan20/{README.md,env.json,probe.json,gate.json,
matrix.json,tier2.json}`. Docs: this plan, `report20.md`, `initiative.md`
items 27–28 and the D33 extension. Engine: item 27 default + item 28 closures
behind flags, with tests; defaults only at Phase D.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan20.md`. Execute
   Phase A (the existing exact-replay flags on the adopted default, three
   cells, seed 0, aux on/off, generous cap); stop at G2. If G2 passes,
   implement item 28's board-free closures behind flags and run Phase C. Do
   not start item 29, do not change defaults before G3, do not touch the
   search base."
2. G2 fails: close items 27–28 negative, next kill review.
3. G3 passes: Phase D (default + 120 s matrix + D14); then item 29.
4. G3 fails: Phase E; the plan21 kill review picks the next lever.
