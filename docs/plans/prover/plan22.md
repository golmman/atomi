# Prover Plan 22: item 30 — proof-memory headroom

Initiative: [`prover`](initiative.md), backlog **item 30**, decisions **D14**,
**D18**, **D21**, **D30**, **D33**, **D34**. Drafted 2026-11-12 after
[`report21`](report21.md). Plan only.

## Kill review (fixed prompt)

1. **Distance.** `T_2`; plan20 stack at Cap 600 s: `core` 16.28, `easy-win`
   6.91 claimed, `deep-win` found 3/3 (132.2–175.4 s) but **not claimable at
   any cap under the 192 MB label** (`store-full`). `T_3` needs `deep-win` at
   120 s: a claim *and* a ~1.3× wall cut.
2. **Would we build this?** Yes, only this: no rung above `T_2` is claimable
   without it. Item 29 (grouping) comes next in the adopt order but does not
   remove `store-full`.
3. **Cancel?** Nothing new. PDS-PN/PN², restarts, portfolio, threads,
   tablebases stay rejected. The proof-share sweep is measured out as a 3/3
   lever ([report13](report13.md) §Phase A); one row re-checks.
4. **Prior art.** Lossy transposition tables, PN²/df-pn+ transposition
   pruning, AND-node recomputation; the change is a *policy split* in our own
   table.
5. **Smallest next experiment.** Phase A: two counters, four runs.

## Decision

**MEM-V1**, one D21 stack under one gate: make the proof table **two-tier**
inside its existing 16-byte slot — **archive** (attacker proofs, `mv != 0`:
exact, never evicted, `store-full` on refusal) and **memo** (defender proofs,
`mv == 0`: evictable, a miss costs work only). Needs **D35** to revise D18's
"never evicts" for the memo tier.

## Premise (why not "raise the label")

The writer reads attacker entries only — `impl ProofSource for ProofTable::
entries` already filters `mv == 0` (`src/strategy/mod.rs`), `Walker::choose`
runs only from `attacker()`. A `deep-win` export needs **94,686 attacker
positions** (384 MB, seed 1, [report21](report21.md) §3) while the 192 MB
table fills with **2,831,155 entries** and refuses. The label is fixed by the
goal, the split is measured out. The excess is recomputation material: **a
defender proof is implied by its children** and re-derives in one entry pass
per revisit. Evicting that tier moves capacity to the archive without
weakening a proof.

## Scope

**In:** Phase A probe; Phase B policy behind a flag; the gate; D14; one ledger
row; `report22.md`; D35.

**Out:** item 29; any change to the guide, the search base, the goal, the
rungs or the budgets; a larger memory label; the event-log re-formulation
(option 1); provari code, text or proofs.

## Design

### Phase A — composition probe

Add two permanent counters to `ProofTable` and the outcome JSON
(`proof_attacker_entries`, `proof_defender_entries`, `proof_archive_refused`).
Then `--guide hand`, one process at a time, deep-win FEN: A1
`--seconds 600 --memory 384 --proof-out /tmp/p22/a.pvr --stats`; A2 the same at
`--memory 192` (control: `store-full`); A3/A4 as A2 with `--proof-share` 50,
75. Record live attacker/defender entries, `proof_entries`, `proof_full`,
`expansions`, `child_evals`, wall, export verdict, attacker-event count. A1 is
the only run not truncated by refusal (its tree is smaller ⇒ an *upper bound*
proxy, stated as such).

**Stop rule (A).** If A1's attacker entries alone exceed 9/10 of the 192 MB
slot count, MEM-V1 is insufficient: stop before code, record the number, take
option 1 to the next review. Otherwise Phase B.

### Phase B — two-tier policy

- Slot unchanged (16 B, `Zeroed`, no allocator change). Class is `mv != 0`
  (archive) vs `mv == 0` (memo); no new field.
- **Archive insert:** unchanged domination rule and tombstone reuse; refuses
  when archive-live would pass 9/10 of slots and the run offers no memo slot
  to displace. `refused()` means *archive* overflow only.
- **Memo insert:** never refuses. Slot order: a tombstone, the run-ending free
  slot, else displace the oldest **memo** slot in the run (age in the unused
  `mv` field of a memo slot — no size change). A run with no memo slot is a
  **miss** (no store), never a refusal, and never displaces an archive entry.
- `ProofSource` unchanged (already attacker-only) ⇒ the writer's contract and
  every existing proof test are untouched.
- Flag `--proof-memo exact|evict|off` (default `exact` until Phase D; `off`
  drops every memo insert as the cheapest ablation arm).

**Tests.** (1) archive behaviour identical with the memo tier on and off
(differential over the fixture playouts); (2) an export with all memo stores
dropped is byte-identical to one with them kept; (3) refusal still fails
closed (`store-full`) at `--proof-share 1`; (4) `--guide none` trajectory
identity; (5) a memo hit needs a full 64-bit key match, so a collision can
only cost work.

### Phase C — the gate

Three cells × seeds 0–2, `--memory 192`, `--guide hand`:

- **Claimability (primary):** `deep-win` `--seconds 600 --proof-out` is
  `proof: written` on **3/3**, each accepted by `atomi validate` **and**
  `provari check`. 2/3 is a failure, not a bound.
- **Tree (guard):** `core`/`easy-win` at 120 s and `deep-win` at 600 s, no
  `child_evals`-to-proof regression > 10 % vs the `exact` control; wall
  reported, not gated.
- **Ablations after the gate (D21):** `off` vs `evict`, both directions.

### Phase D — only if the gate passes

`--proof-memo evict` becomes the default; 120 s matrix (3 × seeds 0–2) and D14
on every proved row; **one** ledger row at **Cap 600 s** (D34; the 192 MB
label named in the `deep-win` cell text) and a history line. **No rung is
claimed**: `deep-win` is 132–175 s, so `T_3` still needs item 29.

### Phase E — if the gate fails

Record the stack, both ablations and the composition; no default change, no
row, no per-cell re-tuning. The next review picks option 1 or a goal re-pin.

## Option that changes the formulation

**Export from the event log.** `ProofEvent::AttackerProven` already carries
every attacker proof; an append-only log written by the sink (`--proof-out`
only) replaces the table as the writer's source, so the table becomes a pure
**lossy** memo at any share (target 12.5 %, giving the TT +12.5 %). Gain:
`store-full` becomes structurally impossible and the tree may shrink ~10 %;
cost: a new on-disk artifact, a reading writer, an eviction policy. **Cheapest
experiment (free in Phase A):** attacker-event count vs live attacker entries.

## Gates

- **G1 data.** `env.json`, `composition.json`, `gate.json`, `matrix.json`,
  driver + README under `measurements/plan22/`; parsed only, one process at a
  time, same-session controls.
- **G2** composition recorded, stop rule A evaluated.
- **G3** Claimability + tree guard (Phase C).
- **G4 soundness.** `make test`, `make lint`, `make test-full` (table, search
  and proof-output change ⇒ slow tier + `make veto`); every claimed win
  D14-accepted; no fixture refutation; licence gate unchanged.
- **G5 docs.** `report22.md`, D35, item 30 status, one `bench.md` row only
  after G3, `history.md` line.

## Risks

- **The archive alone may not fit** — Phase A decides before any code.
- **The memo tier may cost tree** (an AND revisit re-derives at entry, ~28
  child evals): the `exact` control and both ablations expose it; > 10 % fails
  the gate.
- **`store-full` may become unreachable** and stop protecting the export: the
  starved-share test keeps it honest.
- **D33 caps**: `unique_keys` is unusable here; the new counters are exact.

## Deliverables

`measurements/plan22/` (driver derived from plan21's, parsed JSON + README);
`report22.md`; `src/table/proofs.rs` policy + tests, optional `src/cli.rs` flag
and counters; D35 in `decisions.md`; item 30 status; one `bench.md` row at Cap
600 s; `history.md` line.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan22.md`. Execute
   Phase A (the two composition counters + the four deep-win runs); evaluate
   stop rule A and stop there. If it passes, implement Phase B behind
   `--proof-memo` and run Phase C. Do not start item 29, do not raise a memory
   label, do not change a default before G3, do not touch the guide."
2. Stop rule A fires: report the composition; next review takes option 1.
3. G3 passes: Phase D, then item 29. G3 fails: Phase E, next review.
