# Prover Plan 16: item 24 (conformance line) — Phase A, the read-only differential

Initiative: [`prover`](initiative.md), backlog **item 24**, decisions **D14**,
**D18**, **D21**, **D24**, **D26**, **D27**, **D28**, **D29**. Drafted
2026-11-10 after [`report15`](report15.md) and plan session 16. Plan-session
deliverable: it does not implement.

## Premise

The gap is tree size, not per-node cost: `core` 9.27 M expansions vs
provari's 342,289 nodes; `easy-win` 1.24 M vs 72,523; `deep-win` 40.7 M
unique keys vs 4.57 M (report15, plan14). D29 authorises item 24 and makes
Phase A the stop-rule gate: a named divergence must explain the first
divergent ply, or the conformance line stops.

**Plan-session probe (read-only, 2026-11-10).** On `core`, provari hand
(`--seconds 10 --memory 192 --guide hand --proofs …`) writes 7,211 proof
positions; atomi bare (`--seconds 120 --memory 192 --guide none
--proof-out …`) writes 10,831. Both choose `d4e5` at the root. After
normalising FENs (drop ep and halfmove; keep placement, side, castling,
fullmove) and grouping by attacker ply, the ply-1 defender-reply sets are
identical (27 positions). The **first divergent ply is 2**: provari has 270
normalised positions, atomi 377. So the divergence is not the root move or
the first defender reply; it is in the attacker's second-move choices and
their defender replies. The proof lengths differ (17 vs 14), so a divergence
is not by itself a deficiency; Phase A classifies it, it does not judge it.

Caveat: provari exposes no search trace. The observable frontier is its
proof tree. Phase A compares proof trees and the guide's node ordering; it
does not claim causality. B/C/D test causality.

## Scope

**In:** proof-tree differential on `core`, `easy-win`, `deep-win` seed 1; a
read-only `atomi classify` diagnostic; classification against the five
item-15 divergences; `report16.md`.

**Out:** implementing B/C/D; any search, table, guide or default change; a
budget above 120 s; a portfolio; reopening the hand-guide/auxiliary line;
committing provari proofs or artifacts.

## Design

### A1 — proof extraction and diff

- provari: `prove FEN --seconds 10 --memory 192 --guide hand --proofs
  <scratch>/provari/<cell>.jsonl` (all three cells prove inside 10 s).
- atomi: `prove FEN --seconds 120 --memory 192 --guide none --proof-out
  <scratch>/atomi/<cell>.jsonl`; `deep-win` seed 1 only (seeds 0/2 censor).
- Both files have the same shape: a root line, then `{fen, move}` lines in
  preorder. Parse with one stdlib parser.
- Normalise FENs: piece placement, side, castling, fullmove; drop ep and
  halfmove (atomi writes legal ep, the movegen's `fen()` writes pseudo ep;
  halfmove differs by path). Group by attacker ply = fullmove − root
  fullmove.
- First divergent ply = shallowest ply whose normalised FEN sets differ.
  Record the symmetric difference, the common FENs with different chosen
  moves, and the move classes. Order-only differences are not divergences.

### A2 — node-level classification

Add `atomi classify "<FEN>" [--guide none|hand] [--guide-off LIST]
[--guide-on LIST]` (read-only; no search, no tables). It prints one JSON
line: for each legal move, `{move, class, start_pn, start_dn, order}` where
`order` is the guide's selection order at a fresh node (class rank, then the
movegen/seed tie-break the search uses). Tests pin a known FEN and the
default-off trajectory (the subcommand never runs the search).

For each first-divergent position, run `classify` under bare and hand and
compare the guide's order to provari's chosen move. Then test the five
item-15 predicates:

| # | Divergence | Predicate at the divergent node |
| --- | --- | --- |
| 1 | Quiet grouping | move is `Quiet` and the node has ≥2 quiet moves; atomi gives each `(9,9)`, provari one deferred child |
| 2 | Fresh `dn=1` | atomi's start `dn` ≠ 1 (quiet 9, threat 16, hanging 9) |
| 3 | Check `pn` = evasions | node in check; atomi's start `pn` = legal count, provari's = min(surviving evasions, 8) |
| 4 | EF-PN on child | child eval ≠ 0 and atomi scales the parent start |
| 5 | Exact replay | defender node; atomi's replay did not prove the reply |

A mechanism **explains** the divergence if its predicate holds, the
divergent move is the one the mechanism would reorder, and the predicate is
**specific**: it changes that move's order relative to at least one sibling.
A predicate that holds uniformly for every child (e.g. fresh `dn` on a node
with no evasions) is recorded but does not pass G3. This is a
classification, not a causal claim.

### A3 — flag sensitivity (cheap, optional)

Re-run atomi with each existing approximate flag toggled (`--guide-group`,
`--guide-fresh-dn`, `--guide-replay-exact`, `--guide-scale 1
--guide-ef-weight 0`, `--guide-off quiet`) and record whether the first
divergent ply moves later. Evidence only; the gate is A2.

## Gate

- **G1 data.** Proofs parsed; `diff.json`, `classify.json`, `env.json`;
  one process at a time.
- **G2 differential.** First divergent ply recorded for each cell;
  reproducible from the pinned binary, FENs and flags.
- **G3 classification.** On `core`, at least one of the five predicates
  holds at the first divergent ply, the divergent move is in the affected
  class, and the predicate is specific (it changes the move's order
  relative to a sibling); the table is complete. If no specific predicate
  holds, the conformance line stops.
- **G4 soundness.** No search change; `atomi classify` is read-only;
  `make test`, `make lint`, `make test-full` green; no GPL dependency, no
  provari proof committed.
- **G5 docs.** `report16.md`; `initiative.md` item 24 status; no ledger row.

**Stop rule.** G3 fails → close the conformance line, record the negative,
do not implement B/C/D. G3 passes → write plan17 for Phase B (exact replay)
with the pre-registered gate: ≥ 2× cut in `core` unique expansions vs bare
at 20 M evals, no `easy-win` regression > 10 %.

## Tasks

- **T1** `measurements/plan16/run_diff.py` (stdlib) + `env.json`; pin the
  binary, corpus and oracle.
- **T2** `atomi classify` in `src/cli.rs`/`src/main.rs` (read-only) + tests.
- **T3** A1/A2/A3 runs; `diff.json`, `classify.json`.
- **T4** G1–G5, `report16.md`, docs status.

## Risks

| Risk | Mitigation |
| --- | --- |
| Proof trees are not search frontiers | Diagnostic only; B/C/D test causality |
| A different valid winning move looks like a divergence | Classify by class/start pair, not move identity; a shorter proof is not a deficiency |
| FEN normalisation hides a real ep difference | Normalise ep and halfmove; test on the plan-session probe |
| Provari proof files are GPL artifacts | Scratch only; commit derived summaries, never the proof file |
| The first divergence is defender-reply ordering | Compare sets, not order; skip order-only differences |
| The hand stack censors `core` at 120 s | Use the bare proof as primary; `classify` tests the hand ordering at the divergent nodes without a hand proof |
| The probe becomes another ten-plan negative | Phase A is one plan with a real stop rule; B/C/D are separate plans |

## Deliverables

- Data: `measurements/plan16/{README.md,env.json,diff.json,classify.json}`.
- Docs: this plan; `report16.md`; `initiative.md` item 24 status.
- Engine: `atomi classify` (read-only) + tests; no search change.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan16.md`. Execute
   item 24 Phase A: the proof-tree differential on `core`/`easy-win`/
   `deep-win` seed 1, the read-only `atomi classify` diagnostic, and the
   five-predicate classification. Stop at G3; do not implement B/C/D."
2. If G3 fails: close the conformance line and write a kill review for a
   new search algorithm (PDS/PN², TT move ordering) or a formulation
   decision.
3. If G3 passes: write plan17 for Phase B (exact replay) under the
   pre-registered 2× gate.
