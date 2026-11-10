# Prover Plan 13: item 22 (proposed) — cheap probes before any conformance stack

Initiative: [`prover`](initiative.md), backlog **item 22** (proposed),
decisions **D14**, **D18**, **D21**, **D24**, **D27**. Drafted 2026-11-10
after [`report12`](report12.md) and the plan-session-11 diagnosis. Plan-session
deliverable: it does not implement.

## Premise to test

The bare tree is the best measured configuration (report11): 2/3 at 120 s,
`deep-win` the blocker. Per-node cost is competitive (`R` 1.19/1.33),
re-expansion 7.25 %, entry reuse dead (report12); the gap is **tree size**
(~11–27× provari). The only known ~20× tree lever is the reference hand guide,
but atomi's variant is non-conforming (D23) and worse than bare.

Three cheaper questions are still untested, ordered by cost, each with a
pre-registered stop rule:

1. **Memory split.** The proof table is a fixed 25 % of `--memory` (D18). The
   bare tree generates 2.8–3.1 M proofs, so the 90 % cap is reached. A
   plan-session probe (single runs, not a measurement): `deep-win` seed 2
   censors at `--proof-share 25` with `proof_full: true` (2,831,155 entries)
   and **proves at 115.2 s** at `--proof-share 50` (length 27, 1.63 G child
   evals, `proof_full: false`). Seed 0 censors at 50 and 75. This is a direct
   Tier 1 lever and needs no hand-guide work.
2. **Move-limit dimension.** The search has no `remaining` key. `deep-win`'s
   depth histogram has mass at plies 7–17 and a tail to 52 (plan12);
   atomi finds length 25 where provari finds 22. A ply-cap probe tests
   whether the deep tail is where the work is.
3. **Conformance probe.** Item 15 tested one mechanism (`GROUP`) alone and
   closed the line; the five documented divergences were never assembled.
   Under the proposed stack rule (a failed gate keeps the mechanism behind a
   flag; the gate is on the stack), the assemblage is the decisive test.

## Absolute distance (report11)

| Cell | bare 120 s | provari hand | factor |
| --- | --- | --- | ---: |
| `core` | 3/3, 17.390 s, 9.27 M exp | 0.492 s / 342,289 | 35.3× |
| `easy-win` | 3/3, 2.008 s, 1.24 M exp | 0.105 s / 72,523 | 19.1× |
| `deep-win` | **1/3**, seed 1 111.218 s; seeds 0/2 censor, 52 M exp | 6.453 s / 4,574,912 | ≥ 18.6× |

Tier 1 is **2/3**. This plan makes no Tier 2 claim and does not raise the
budget.

## Scope

**In:** Phase A proof-share sweep; Phase B a measurement-only `--max-depth`
ply cap; Phase C a conformance probe of the documented divergences, including
one missing mechanism behind a flag (exact sibling proof replay); D14 per
proved cell; docs.

**Out:** a default-flag change (the best split is recorded, not adopted); a
full conformance stack (a follow-up plan); the `remaining` key dimension
(only if Phase B passes); a portfolio; a budget above 120 s; the closed
auxiliary/simulation line; M4/`planX`; a proof-format change.

## Design

### Phase A — proof-share sweep (may end the plan)

Cells `deep-win`, `core`, `easy-win`; seeds 0–2; `--guide none --seconds 120
--memory 192`; shares **25** (control), **40**, **50**, **60**. Record
`exit`, length, wall, `child_evals`, `expansions`, `proof_entries`,
`proof_full`. D14 (`--seconds 600 --proof-out`, same flags/seed) per proved
cell; `atomi validate` + `provari check`.

**Stop rule A:** if some share proves `deep-win` on **3/3** seeds and
`core`/`easy-win` stay 3/3 with wall not worse than bare-25 % by more than
10 %, Tier 1 is green: append the first ledger row since plan2 and stop the
plan. A 2/3-seed `deep-win` is a finding and the plan proceeds. The split is a
measured parameter (proposed D28); the 192 MB label is unchanged.

### Phase B — move-limit probe (if A fails)

Add `--max-depth N` (opt-in, default off): a ply cap that censors at `N`
(`Exit::Depth`), measurement only. Run `deep-win`/`core` at `--evals 20M`,
seed 0, caps 16/20/24/28/32, plus bare; then seeds 1–2 for the best cap.
Record unique expansions, whether a proof is reached, its length.

**Stop rule B:** a cap passes only if it reaches the bare proof length (or
proves) with **≥ 2× fewer unique expansions** at 20 M. No such cap → drop the
`remaining` key dimension and go to Phase C. If one passes, Phase B2 (the
`remaining` key + iterative deepening, with the proof-table keying it
implies) is a separate plan; this plan stops with the recommendation.

### Phase C — conformance probe (if A and B fail)

Assemble the documented divergences behind existing flags where possible:
`--guide-group` (one-child quiet GROUP), `--guide-scale 1 --guide-ef-weight 0`
(EF-PN off/band), and a new opt-in `--guide-fresh-dn` if the fresh disproof
number is not already a parameter. Implement the one missing mechanism,
**exact sibling proof replay**, behind `--guide-replay-exact` (default off).
Run a leave-one-out / add-one-in at `--evals 20M`, seed 0,
`core`/`easy-win`/`deep-win`; seed 1 for the best stack.

**Stop rule C:** no assembled stack cuts `core` unique expansions by **≥ 2×**
vs bare at 20 M → close the hand-guide line for good (a real stop, not a
per-mechanism one). If it passes, write a follow-up plan for the full
conformance stack with a 120 s matrix; this plan does not build it.

## Gates

- **G1 data.** Phase A/B/C rows and a D14 row per proved cell; parsed JSON,
  `env.json`, one process at a time.
- **G2 Phase A.** 3/3 `deep-win` + `core`/`easy-win` 3/3 within 10 % wall →
  Tier 1 green and the ledger row; otherwise the exact per-seed censors.
- **G3 Phase B.** The ≥ 2× unique-expansion threshold is applied exactly; a
  cap that only shifts wall does not pass.
- **G4 Phase C.** The ≥ 2× `core` threshold is applied exactly; a failed
  stack closes the line and is recorded, not re-scoped.
- **G5 soundness.** No refutation or never-prove; every claimed proof accepted
  by `atomi validate` and `provari check`; `make test`, `make lint`,
  `make test-full` and `make veto` green; `--max-depth` and
  `--guide-replay-exact` default off and trajectory-neutral when off.
- **G6 docs.** `report13.md`; `initiative.md` item 22 status; `bench.md`
  history and a ledger row only if Tier 1 passes; no GPL dependency, raw
  transcript, provari artifact, proof file or `__pycache__`.

## Risks

| Risk | Mitigation |
| --- | --- |
| A split that helps `deep-win` hurts `core`/`easy-win` (smaller TT) | Phase A's 10 % guard; report per-cell, not globally |
| A cap that censors the proof looks like a tree cut | The "reaches the proven length" condition in stop rule B |
| The conformance probe re-runs the item-15 failure | The assemblage is the test; stop rule C is a real stop |
| Scope creep into the full stack, portfolio or budget | Phase C only writes a follow-up plan; no default change, no budget raise |
| The probe becomes another 10-plan negative | Each phase is cheaper than the next and has its own stop rule; A may end the plan |

## Formulation options (ideation)

- **A (recommended): the three-phase probe.** Gain: the cheapest phase can add
  the first ledger row in 10 plans; each later phase is gated by a measured
  stop. Cheapest experiment: Phase A's 12 runs.
- **B — change what counts as output:** a *proof limit* (a `remaining`-keyed
  search reporting the longest line it can prove, not win/no-win at a budget).
  Gain: censored cells become graded output; cost: a new value algebra and
  proof contract. Cheapest test: Phase B.
- **C — drop the 120 s constraint:** measure time-to-proof on `deep-win`.
  Gain: seed 1 proves at 111 s, so the cell is close; cost: a goal change
  (D27 amendment), not authorized. Cheapest test: one labelled 300 s run.
- **D — drop the hand-guide constraint:** a portfolio of bare + a conforming
  hand stack (D6, after M5). Gain: union of coverage; cost: needs a second
  configuration that does not exist yet. Cheapest test: Phase C.

## Deliverables

- Data: `measurements/plan13/{README.md,env.json,*.json}`.
- Docs: this plan; `report13.md`; `initiative.md` item 22 status; `bench.md`
  history and a ledger row only if Phase A passes.
- Engine (opt-in only): `--max-depth`; `--guide-replay-exact` if Phase C
  runs; tests pinning default-off trajectory neutrality. No default change.
