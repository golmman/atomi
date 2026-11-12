# Prover Plan 23: item 32 — proof-retention share (RET-V1)

Initiative: [`prover`](initiative.md), new backlog **item 32**, decisions
**D14**, **D18**, **D21**, **D30**, **D34**, proposed **D35**. Drafted
2026-11-12 after [`report22`](report22.md). Plan only.

## Kill review (fixed prompt)

1. **Distance.** `T_2`. plan20/21 stack at Cap 600 s: `core` 16.28,
   `easy-win` 6.91 claimed; `deep-win` finds the mate on 3/3 (132.2–175.4 s)
   but **no claim** at 192 MB (`store-full`). `T_3` needs `deep-win` claimed
   **and** at 120 s.
2. **Would we build this?** Yes, as the cheapest decisive step. Plan22's A3/A4
   show that a 192 MB, 50–75 % proof share writes a both-checker-accepted
   `deep-win` proof at **103.4 / 114.7 s** on seed 0 — inside the `T_3` wall.
   Event-log export (plan22 option 1) removes `store-full` but leaves the
   default-share tree at 132–175 s, so it cannot reach `T_3` alone.
3. **Cancel?** Item 29 stays next after this item; event-log export stays a
   fallback, not a parallel plan. Portfolio, restarts, threads, tablebases and
   PDS-PN/PN² stay rejected.
4. **Prior art.** Memory split and aging in lossy transposition tables; the
   change is a *parameter with a measured search effect* in our own table.
5. **Smallest next experiment.** Phase A: `deep-win` seeds 0–2 at shares
   25/50/75, 120 s timed, plus one 600 s proof run per proved seed.

## Decision

**RET-V1**, one independently meaningful mechanism: replace D18's fixed
**quarter** proof-table share with a **single global retention share** chosen
by Phase A (candidate 50 %). The `192 MB` label is unchanged; only the
TT/proof-table split moves. Gate: `T_3` **and** D14 on every proved seed.

## Premise (why the split is a search lever)

At 192 MB the 25 % share gives 2,831,155 usable entries (90 % of 3.15 M slots);
the `deep-win` archive needs **5,639,488** live attacker entries
([`report22`](report22.md) §2). The table fills, proof reuse is lost, and the
tree grows: A2 = 205.2 M child evals / 174.9 s at 25 %, A3 = 134.8 M / 103.4 s
at 50 % with the **same** 192 MB label. A4 (75 %) is 155.2 M / 114.7 s, so the
effect is non-monotone in the share (a larger table also shrinks the TT). This
is not "raising the budget" and not per-cell tuning: one share applies to
every cell and seed. The export succeeds because the archive fits, not because
a check was weakened.

## Scope

**In:** Phase A read-only sweep; one global share as the default; the gate; the
120 s matrix and D14; one ledger row at Cap 120 s; `report23.md`; proposed D35.

**Out:** item 29; event-log export; eviction/two-tier policies; guide, search
base, goal, rungs, memory label; provari code, text or proofs.

## Design

### Phase A — share sweep (read-only)

`--guide hand --memory 192`, seeds 0–2, one process at a time. Timed 120 s
runs for `core`/`easy-win`/`deep-win` at shares **25 (control), 50, 75**.
For each `deep-win` run that proves within the 120 s, one untimed
`--seconds 600 --proof-out` run at the **same share and seed**, `child_evals`
must match, then
`atomi validate` **and** `provari check`. Record wall, `child_evals`,
`proof_entries`, `proof_full`, `proof_attacker_entries`,
`proof_defender_entries`, `proof_archive_refused`, export verdict and checker
results. `--proof-share` is an existing flag; no code changes in Phase A.

**Selection rule (fixed before the data).** Adopt the **smallest** share
(50 before 75) that gives `deep-win` **3/3** at 120 s **and** leaves
`core`/`easy-win` 3/3 with no median-wall regression > 10 % vs the 25 %
control. If both shares fail, Phase D. No other share is tried, so the sweep
cannot become a per-cell search.

### Phase B — default change (only after Phase A selects a share)

Change the default proof share in `src/table/mod.rs` (`split`/`Tables::new`)
and the CLI help. Keep `--proof-share` for measurement; the split invariant
test already covers every percent. The default stays a single global value.

### Phase C — the gate

**T_3 (primary):** `9` timed runs (3 cells × seeds 0–2) at 120 s, 192 MB,
`--guide hand`: all nine **proved** (`core`/`easy-win` may only improve on
plan21); every `deep-win` proof D14-accepted at 600 s with identical
`child_evals`. 2/3 on any cell is a failure, not a bound.

**Guards:** no `core`/`easy-win` median-wall regression > 10 % vs the 25 %
control; no fixture refutation; geomean over claimed cells at Cap 120 s
recorded, never compared across caps (D34).

### Phase D — if the gate fails

Record the sweep, the selected-share result and both directions of the memory
split; no default change; no ledger row. Take event-log export (plan22
option 1) or item 29 to the next review. Do not re-tune per cell or seed.

## Option that changes the formulation

**Event-log export (plan22 option 1).** Make the writer read the
`AttackerProven` event log instead of the table, so `store-full` is
structurally impossible and the table becomes a pure lossy memo. Gain: the
share could then be chosen for tree/TT only (target 12.5 %, TT +12.5 %), and
no archive has to fit. Cost: a new on-disk artifact, a reading writer and an
eviction policy. **Cheapest experiment (free in Phase A):** compare live
attacker entries (5.6 M) with exported positions (76–93 k); the gap is the
stale-proof volume the log would also carry.

## Gates

- **G1 data.** `env.json`, `sweep.json`, `gate.json`, driver + README under
  `measurements/plan23/`; parsed only, one process at a time, same-session
  controls.
- **G2** selection rule evaluated; chosen share recorded with its guards.
- **G3** `T_3` + D14 + guards (Phase C).
- **G4 soundness.** `make test`, `make lint`; if the default changes,
  `make test-full` (table/search change ⇒ slow tier + `make veto`); every
  claimed win D14-accepted; licence gate unchanged.
- **G5 docs.** `report23.md`, D35, item 32 status, one `bench.md` row at
  **Cap 120 s** only after G3, `history.md` line.

## Risks

- **Proof share trades TT**; A1 vs A3 shows the wall is not monotone in the
  share, so the 25 % control and the 75 % arm are both required.
- **Seed variance**: A3/A4 are seed 0 only; a 120 s claim needs 3/3.
- **Export may still refuse** on a seed whose archive exceeds the chosen
  share; that seed is censored, never converted.
- **D33 traps**: use the exact composition counters, never `unique_keys`.

## Deliverables

`measurements/plan23/` (driver derived from plan21/22, parsed JSON + README);
`report23.md`; default-share change and tests if G3 passes; D35 in
`decisions.md`; item 32 status; one `bench.md` row at Cap 120 s; `history.md`
line.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan23.md`. Execute
   Phase A exactly as written (25/50/75, seeds 0–2, `deep-win` proof runs at
   the same share); apply the fixed selection rule. If a share is selected,
   make it the default, run Phase C and claim `T_3` only on 3/3 with D14. Do
   not start item 29, do not try other shares, do not change the 192 MB
   label."
2. Phase A selects a share but `T_3` or a guard fails: Phase D, then item 29.
3. No share passes Phase A: Phase D, and take event-log export to the next
   review.
