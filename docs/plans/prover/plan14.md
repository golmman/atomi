# Prover Plan 14: item 23 (proposed) — kill review and owner decision on the `deep-win` formulation

Initiative: [`prover`](initiative.md), backlog **item 23** (proposed),
decisions **D14**, **D18**, **D24**, **D27**. Drafted 2026-11-10 after
[`report13`](report13.md) and a plan-session `deep-win` probe. Plan-session
deliverable: it does not implement.

## Kill review (fixed prompt)

### 1. Absolute distance to the goal now

| Cell | bare 120 s | provari hand | factor |
| --- | --- | --- | ---: |
| `core` | 3/3, 17.390 s | 0.492 s / 342,289 nodes | 35.3× |
| `easy-win` | 3/3, 2.008 s | 0.105 s / 72,523 nodes | 19.1× |
| `deep-win` | **1/3**, seed 1 111.218 s; seeds 0/2 censor | 6.453 s / 4,574,912 nodes | ≥ 18.6× |

Tier 1 is **2/3**. The plan-session probe (temporary `--stats` key set
`2^26`, reverted; `--guide none --seconds 120 --memory 192 --seed 0`) gives
the missing 120 s tree shape: **50,016,256 expansions, 40,708,534 unique
keys, 9,307,722 re-expansions (18.6 %), 2,661,718 proof entries,
`proof_full: false`**. At 20 M the exact re-expansion ratio is 7.25 %
(report12). So the `deep-win` gap is **~8.9× more unique positions than
provari's 4.57 M nodes**, not memory pressure, re-expansion, proof-table
pressure or per-node cost. The 120 s budget is fixed.

### 2. If we started today, would we build this?

We would build the bare DF-PN, proof output, validator and oracle veto. We
would **not** build the hand-guide stack: its best conformance stack cuts
`core` 1.22× and `easy-win` 3.2× but does not move `deep-win` (report13). We
would not build a portfolio. We would set the coverage target at 2/3 or
define a graded output before spending ten plans on a 3/3 target that the
measured tree does not support.

### 3. Which initiative or plan would you cancel?

Cancel **item 18** (`RESTART-GEN`) and **item 22** (cheap probes) as closed.
Do not reopen the hand-guide/auxiliary line. Do not build a portfolio. The
only open question is the `deep-win` tree, and the new data rule out the
memory/re-expansion family that item 18 targeted.

### 4. Prior-art search (re-run)

Provari's remaining non-hand mechanisms, checked against the data:

- **Move-limit / `remaining` key.** Dead at 120 s: the depth histogram has
  mass at plies 7–17; cap 16 cuts 1.3× but censors, caps 20+ cut < 10 %.
  Plan13's 20 M cap probe was inert because the eval cap fires first.
- **Proof-table pressure (drop mates, filter rebuild).** The 120 s proof
  table is not full (2.66 M entries, `proof_full: false`); not the lever.
- **TT least-work replacement / generation tags.** Re-expansion is 18.6 % at
  120 s, not 2×; a replacement change cannot recover 8.9×.
- **Restarts.** No evidence; re-expansion is not the lever.
- **A different search algorithm** (PDS/PN², TT move ordering, iterative
  threshold deepening) is the only untested family. It is a new search, not
  a cheap mechanism, and has no measured 2× case.

No prior-art mechanism gives a credible 2× unique-expansion cut on
`deep-win` under 120 s / 192 MB.

### 5. Smallest next experiment

The 120 s exact distinct-key probe above. It rules out the memory and
re-expansion family. The next smallest experiment is an **owner decision**,
not another engine probe.

## Premise

No non-hand tree mechanism with a credible 2× case remains. The goal
formulation must change or the coverage target must change. The 120 s budget
is fixed; the hand-guide/auxiliary line and the portfolio stay closed.

## Scope

**In:** the owner decision options, a decision gate, and a decision record
(D28) if accepted. **Out:** implementation, a budget above 120 s, the
hand-guide/auxiliary line, a portfolio, a default-flag change, and a new
search algorithm implementation.

## Decision options

- **A — accept 2/3 as Tier 1 (D27 amendment).** Keep 120 s / 192 MB, record
  `deep-win` 1/3, no ledger row, bare tree as the product candidate. Gain:
  honest, stops drift. Cost: the 3/3 goal is not met.
- **B — redefine Tier 1 as a proof-limit / graded output.** Define exactly,
  e.g. "prove a win within N attacker moves, or report the longest proven
  line within 120 s." `deep-win` seeds 0/2 still have no proof, so this does
  **not** give 3/3 unless "no win within N" counts as a result. If wanted, it
  is a new product goal, not Tier 1.
- **C — authorise a new search algorithm** (PDS/PN² or TT move ordering) as a
  new item with a read-only probe. Gain: possible 2×; cost: large, uncertain,
  a new search. It is not a hand-guide mechanism.
- **D — budget change.** Not authorised; the 120 s budget is fixed.

**Recommendation:** **A** as the default. **C** only if the owner rejects 2/3
and accepts a new search item. **B** only if the goal is redefined; a bound
is never called a win.

## Gates

- **G1 decision.** The owner records A, B or C in `initiative.md` (D28) and
  `bench.md`; no silent goal change.
- **G2 budget.** No run above 120 s; no budget raise.
- **G3 closed lines.** No hand-guide/auxiliary reopen, no portfolio.
- **G4 code.** No engine change in this plan; the probe patch is reverted.
- **G5 docs.** This plan; the decision record; no GPL dependency, raw
  transcript, provari artifact, proof file or `__pycache__`.

## Risks

| Risk | Mitigation |
| --- | --- |
| Accepting 2/3 looks like giving up | Record the exact censor and the 40.7 M unique-key data; the goal is a win prover, not a 3/3 slogan |
| A proof-limit goal is mistaken for a win | Define it as a bound; keep D14 for every claimed win |
| A new search algorithm is another ten-plan negative | Read-only probe first, with a pre-registered 2× stop rule |
| The decision is deferred again | This plan's only deliverable is the decision; no implementation |

## Deliverables

- This plan. No code, no data files; the temporary `--stats` key-set probe is
  recorded here and reverted.
- If the owner accepts A, B or C: update `initiative.md` (D28) and
  `bench.md`, then write the next plan (none for A; a goal-definition plan for
  B; a read-only search probe for C).

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan14.md`. Accept
   option A (2/3 as Tier 1) and record D28; update `initiative.md` and
   `bench.md`. Do not raise the 120 s budget, reopen the hand-guide/auxiliary
   line, or build a portfolio."
2. If the owner rejects A: authorise option C with a read-only probe of
   PDS/PN² or TT move ordering on `deep-win` seed 0 at 20 M and 120 s, stop
   rule 2× unique-expansion cut.
3. If the owner wants B: write a separate goal-definition plan; do not call a
   bound a win.
