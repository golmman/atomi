# Prover Report 15: item 5 closed, item 6 (M5) head-to-head under D28

Plan: [`plan15.md`](plan15.md) (items 5 and 6; D3/D14/D17/D18/D24/D27/D28).
Executed 2026-11-10. Item 5 (M4) is **closed as superseded**; item 6 (M5) is
**executed**. Tier 1 is **2/3, pass** under D28; Tier 2 is **1/3 shared,
geomean 15.757, fail**. No engine change.

## Provenance

| Item | Value |
| --- | --- |
| Host | `e10de197da58`, AMD Ryzen 9 5950X, Linux 7.2.13 |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `8953596` on a dirty tree (plan15 docs; agents do not commit), binary sha256 `b28bdb1e…62ba71f` |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206` |
| Corpus | `corpus-v1`, sha256 `9cd0169a…47d0e` |
| Configuration | `--guide none --memory 192`; Tier 1 `--seconds 120`, seeds 0–2; Tier 2 `--seconds 10`, seed 0, 3 repeats |

Commands: `make test`, `make lint`, `make test-full` (incl. `make veto`), all
green. Data: [`measurements/plan15/`](measurements/plan15/README.md).

## Item 5 (M4) — closed as superseded

The plan's decision is recorded, not re-litigated: M4's scale gate is already
met by D18 + D14 (plan11's `deep-win` seed-1 proof: 49,523 positions /
2,746,342 entries / 4.0 MB, `proof_full: false`, equal `child_evals`, both
checkers accept). The event-fed worker existed to remove the proof-table
**export** cap; no `store-full` occurred, and the remaining proof pressure is
a search-memory effect. `.pvr` is an interoperability optimisation, not a
Tier 1/Tier 2 lever, and is deferred to the horizon. No ledger row.

## Tier 1 — 2/3 at 120 s (pass under D28)

| Cell | proved | wall (median) | expansions (median) | child_evals (median) | length | factor |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| core | **3/3** | 17.471 s | 9,266,026 | 268,226,989 | 14 / 14 / 24 | 35.8× |
| easy-win | **3/3** | 2.023 s | 1,243,369 | 34,289,765 | 17 / 17 / 19 | 18.9× |
| deep-win | **1/3** | 120.001 s (censor) | 52,751,360 | 1,453,493,923 | 25 (seed 1) | ≥ 18.6× |

`deep-win` seed 1 proves at **113.458 s** (51,002,209 expansions,
1,398,553,959 child evals, length 25); seeds 0 and 2 censor at 120 s. The
counters reproduce plan11 exactly. All seven proved rows pass D14: equal
`child_evals`, `atomi validate` and `provari check` accepted. The 120 s
budget was not raised and no censor was converted.

## Tier 2 — 1/3 shared, geomean 15.757 (fail)

| Cell | atomi | provari hand | factor | coverage |
| --- | ---: | ---: | ---: | --- |
| core | 10.000 s (censor) | 0.488 s / 342,289 nodes | **≥ 20.49** | censored |
| easy-win | 1.686 s | 0.107 s / 72,523 nodes | **15.757** | shared |
| deep-win | 10.000 s (censor) | 6.455 s / 4,574,912 nodes | **≥ 1.549** | censored |

Only `easy-win` is shared, so Tier 2's "coverage stays 3/3" clause cannot be
met; the shared-cell geomean is **15.757**, not `< 1.0`. The three
`easy-win` D14 proofs are accepted by both checkers (equal `child_evals`,
2,048 positions). Censored cells are lower bounds, never factors.

## Default decision

The default is already `--guide none`; no flag changed. Product command:
`atomi prove "<FEN>" --seconds 120 --memory 192` (Tier 1) and
`--seconds 10` (Tier 2). No measured cell favours the hand stack; it stays
behind flags.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | **pass** — 9 Tier 1 + 7 Tier 1 D14 + 9 Tier 2 atomi + 9 provari + 3 Tier 2 D14 rows; parsed JSON, `env.json`, one process at a time |
| G2 Tier 1 | **pass** — 2/3 at 120 s, 7/7 D14 accepted, no fixture regression |
| G3 Tier 2 | **fail (recorded)** — coverage 1/3, geomean 15.757; no conversion |
| G4 soundness | **pass** — no refutation; 10/10 D14 proofs accepted by both checkers; `make test`, `make lint`, `make test-full`, `make veto` green |
| G5 docs | **pass** — report, initiative/bench updates, no GPL dependency, raw transcript, provari artifact, proof or `__pycache__` |

## Problems and open parts

- **The initiative goal is not met.** Tier 1 passes at 2/3 by D28, but Tier 2
  fails on both clauses: coverage is 1/3 and the shared-cell geomean is
  15.757. The absolute gap is unchanged from plan11: `easy-win` ≈ 15.8×,
  `core` ≥ 20.5×, `deep-win` ≥ 1.55× (censored).
- **Tier 2 coverage clause is stale.** D28 amended Tier 1 to 2/3 but left
  Tier 2's "coverage stays 3/3". At 10 s atomi coverage is 1/3, so Tier 2
  cannot be claimed even if the geomean passed. This is recorded as an open
  goal-definition issue; Tier 2 was not silently changed. Because the
  geomean also fails, the outcome is unchanged. A D29 clarification
  (coverage = Tier 1 level, or geomean on shared cells only) is needed if the
  owner wants Tier 2 live.
- **Controls unavailable.** The legacy `default`/`opt` binaries and the
  provari v0.2.0 stretch build are absent; both are recorded `unavailable`,
  not substituted. The gate depends only on atomi + provari hand.
- **No new speed mechanism was authorised or started.** The plan's scope
  excluded engine/search/guide changes, a budget above 120 s, a hand-guide or
  auxiliary reopen, a portfolio and `.pvr`.

## Missing tests

None: no code changed. The existing `make test`/`make test-full` suite is the
applicable gate and is green.

## Next steps

1. **Recommended:** close the initiative with a negative Tier 2 result
   (Tier 1 2/3 by D28, Tier 2 1/3 shared / 15.757). The measured record is
   complete; no non-hand mechanism has a credible 2× case (plan14).
2. If the owner wants Tier 2 live, write a D29 goal-definition plan (coverage
   clause) before any new speed mechanism.
3. If the owner wants a new speed mechanism, it needs a new plan and an owner
   decision; M5 does not start one.
