# Report 21 — item 31: the ledger states its cap

Plan: [`plan21.md`](plan21.md). Data:
[`measurements/plan21/`](measurements/plan21/README.md) (`env.json`,
`ref.json`, `cap.json`, `d14.json`, `mem_ladder.json`, `ledger.json`).
Decisions: D14, D18, D30, D34. Host: Ryzen 9 5950X, 4 vCPU, one process at a
time. No engine change: same binary as plan20 (`atomi@772b3cb+d`, sha256
`4df19305…`).

## 0. Verdict

`bench.md` is now a **results ledger** and the goal, rungs and budgets live
only in `initiative.md` (D34). The plan20 stack was re-measured at **Cap
600 s** with a same-session reference and full D14: `core` **16.28** and
`easy-win` **6.91** are claimed (6/6 accepted), geomean **10.6** over 2
claimed cells.

`deep-win` is **not claimable at any cap under the 192 MB label**, and that is
the session's real finding: the search finds the mate on 3/3 seeds
(132.2–175.4 s) but the proof table is full, the export is refused closed
(`store-full`) and neither checker gets a proof. report20 §0's "prove
`deep-win` 3/3 with a generous cap" was a **search finding, not a claim**;
item 30 (proof-store headroom) is now the gate for any `deep-win` claim, ahead
of the wall.

## 1. Why the convention changed

The `Cap` column varies per row, so a cap that is "policy" contradicts the
column that documents it. Resolution (D34): a row records the budget it was
measured at; `censored` means "not claimed at that cap"; a longer run is its
own row. Two guards were written into `bench.md` (reading rule 6 and the
`Cap`-column note) because the geomean is **non-monotone in the cap**:

| Cap | Claimed cells | Geomean |
| --- | ---: | ---: |
| 10 s (plan20 tier 2) | 1 (`easy-win`) | 7.0 |
| 120 s (plan20 row) | 2 | 10.8 |
| 600 s (plan21 row) | 2 | 10.6 |

A cap picked per plan to lower the headline is cap shopping; a cross-cap
geomean comparison is a reading error. Hence `(cap, coverage, geomean)` is
always stated together and the geomean names its cell count.

## 2. The Cap-600 row (`cap.json`, `ref.json`, `ledger.json`)

Same-session reference (3 repeats per cell): `core` 0.520 s · 342,289 nodes;
`easy-win` 0.133 s · 72,523; `deep-win` 6.564 s · 4,574,912.

| Cell | Walls (s, seeds 0–2) | Median | Factor | D14 |
| --- | --- | ---: | ---: | --- |
| `core` | 10.217 / 8.468 / 8.217 | 8.468 | **16.28** | 3/3 accepted (13,751–16,991 positions, lengths 15/26/20) |
| `easy-win` | 0.806 / 0.919 / 1.431 | 0.919 | **6.91** | 3/3 accepted (2,794–4,100, lengths 19/19/16) |
| `deep-win` | 175.408 / 132.185 / 153.740 | 153.740 | not claimed | 0/3 (`store-full`) |

`child_evals` reproduce the plan20 `gate.json` tree rows **exactly** on all
nine runs (`core` 12,438,175 / 10,097,122 / 9,846,498; `easy-win` 1,016,547 /
1,177,648 / 1,803,483; `deep-win` 205,208,471 / 163,653,289 / 188,579,356),
i.e. the search is unchanged and the wall spread (e.g. `core` 10.2 → 8.2 s)
is host noise, not a different trajectory.

## 3. The `deep-win` finding (`d14.json`, `mem_ladder.json`)

D14 for `deep-win` (untimed, `--seconds 600 --proof-out`, same stack, same
192 MB label): `child_evals` match on all three seeds, `exit: proved`,
`proof_full: true`, `proof_entries: 2,831,155` (the 192 MB proof table's
capacity), `proof: store-full`, exit 3 — **no proof file**, so `atomi
validate` and `provari check` have nothing to accept. Per D14 the cell is
reported censored with the verdict class, never converted.

Cause check (diagnostic, not a row; one seed, 192 MB excluded):

| Label | Proof | Positions | Length | Evals | Wall | Both checkers |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| 384 MB | written | 94,686 | 35 | 139.4 M | 106.4 s | accept |
| 768 MB | written | 74,157 | 37 | 113.5 M | 81.7 s | accept |

So the `deep-win` win is real and exportable — the blocker is the proof store
at the 192 MB label (item 30), not the search and not the wall. A larger label
changes the trajectory (fewer re-inserts), so it is a different bench
configuration, not a substitute for the 192 MB row.

## 4. Document changes (no code)

- `bench.md`: the goal staircase, the "Current position: `T_2`" line, the
  ledger-local "D30 amended 2026-11-11" paragraph and the "cap is policy"
  framing are gone. New: a pointer section ("Goal, rungs and budgets are not
  stated here"), the distance notes retitled as dated history, a rewritten
  censoring bullet (`not claimed (<verdict>)` for a D14 failure), reading rules
  3 and 6 (per-row cap; no cross-cap geomean), the `Cap`-column note, the
  `plan21` plan note and history entry, and the pins line for `atomi@772b3cb`.
- `initiative.md`: one paragraph stating that the goal/rungs/budgets are
  stated **there only**, the revised `Ledger:` convention bullet, the item-30
  status (triggered, claim blocker), the new item-31 row and the status block.
- `decisions.md`: **D34**, superseding D30's 2026-11-11 ledger-split clause.
- `scripts/check_goal.py`: `bench.md` removed from `FILES` (three normative
  docs now); docstring updated. Gate output: "goal check OK: T_5 stated once
  in 3 normative docs".

## 5. Gates

On the shipped binary (`atomi@772b3cb+d`, sha256 `4df19305…`), after the
measurement and the document changes:

- `make test` — licence gate + 127 unit / 51 integration tests, green;
- `make lint` — goal check ("T_5 stated once in 3 normative docs"), fmt,
  clippy `-D warnings`, doc, green;
- `make veto` — **PASS**: 8/8 accepted rows, four controls all
  `valid: false` (drop-root, illegal root move, alternative move, rule50+1).
  This is required here because the row makes new proof claims.

No engine change, so the slow tier (`cargo test -- --ignored`) was not re-run;
plan20's `make test-full` stands for the binary.

## 6. Problems, open parts and next steps

1. **`deep-win` needs item 30 before anything else.** At 192 MB no cap claims
   it: 10 s, 120 s and 600 s all fail on `store-full`, not on time. The
   cheapest lever is the one R17 already names (do not store recomputable
   proofs / mate cache in the TT), because 94,686 positions suffice at 384 MB
   while 2.83 M slots are exhausted at 192 MB.
2. **The label is a confound in every `deep-win` row.** 384 MB and 768 MB
   export and are accepted; a row at a larger label is legal under D34 but
   must carry its own `Cap` *and* label, so the ledger needs the memory label
   named in the row when it varies (proposed: a `Label` note in the cell text
   of the first such row, not a new column yet).
3. **`deep-win` lengths vary by seed** (28 / 36 / 34; provari: mate 22). A
   claim would be a longer-mate proof; the length is a D14 field, not a gate.
4. **Geomean non-monotonicity is now documented but not tested.** A test that
   refuses a row whose `Cap` differs from its neighbours' would be overkill;
   the guard stays a reading rule.
5. **Item 29 (plain-move grouping) is still next in the adopt order**, but
   item 30 now precedes it for `deep-win` claims.

SESSION COMPLETE
