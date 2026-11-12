# Report 27 — item 34: NULL-V2 adopted by owner override (cleaned)

Plan: [`plan27.md`](plan27.md). Data:
[`measurements/plan27/`](measurements/plan27/README.md) (`env.json`,
`probe.py`, `probe.json`, `bench.json`, README). Decisions: D14, D21, D30,
D34, D37, D38 (confirmed), new **D39**. Host: Ryzen 9 5950X (4 vCPU
container), one process at a time.

## 0. Verdict

**NULL-V2 is adopted by owner override (D39); G-N2's miss stands.** The
declared `deep-win` ≤ 0.85× guard was missed at 0.8631× (report26). Adoption
is justified by the mechanism: at 2^17 entries the cache is exact and
tree-identical, all three cells improved, and the cost is 1.25 MB. The engine
stays at **`T_3`**; `T_4` distance is `deep-win` **7.17×** (71.7 s → 10 s), or
7.74× on the same-session Cap-120 bench row (77.4 s). The per-node micro-line
stays closed; the next substantive decision is still the formulation choice
(PDS-PN/PN² or a goal re-pin under D30).

The stored plan26 patch was **not** the measured code. `DEFAULT_ENTRIES =
1 << 21` made the 5 M-eval `deep-win` probe produce 168,885 expansions (the
tree-changing 2^21 arm) instead of the control's 168,910. The one-line fix to
`1 << 17` reproduces 168,910 and the Phase A hit/miss counts exactly.

## 1. Implementation

- `src/table/null_cache.rs`: the direct-mapped cache (10-byte entries, both
  signs, full-key/meta compare) with `DEFAULT_ENTRIES = 2^17`.
- `src/table/mod.rs`: `Budget.cache_entries` is the single size authority.
  `split_share_with_cache` clamps the request to the filter slack + TT, rounds
  down to a power of two, and `Tables` allocates exactly that; a new test pins
  `null_cache.len() == Budget.cache_entries`. At 2^17 the cache is filter-only,
  so the TT and proof table keep their sizes.
- `src/guide/null.rs`: EARLY-REPLY (pseudo-legal reply stream, lazy legality,
  two `StateInfo`s) plus the cached `exposed_limit` wrapper. The clone-based
  reference test and the cache-vs-uncached test are kept.
- `--null-cache-entries N` (default 2^17, `0` disables) is plumbed through
  `ProveOptions → Limits → Tables::with_share_and_cache`; the parse and default
  are tested. A request is a cap, not a guarantee: only sizes within the filter
  slack are tree-neutral.
- Removed the Phase-A probe instrumentation: `positive_null` and
  `null_positive_repeats` are gone from `SearchResult`, `Engine`, `--stats`
  and the tests. `null_cache_hits`/`null_cache_misses` remain.

## 2. Tree identity (P-27)

5 M child evals, `deep-win`, seed 0, `--guide hand --memory 192`:

| Config | `child_evals` | Expansions | Hits / misses | Tree |
| --- | ---: | ---: | ---: | --- |
| plan26 control | 5,000,034 | 168,910 | — | reference |
| default 2^17 | 5,000,034 | **168,910** | 1,014,999 / 2,364,622 | identical |
| `--null-cache-entries 0` | 5,000,034 | 168,910 | 0 / 0 | identical |
| `--null-cache-entries 2097152` | 5,000,025 | 168,885 | 1,200,836 / 2,178,341 | differs (2^21 arm) |

The default and disabled rows match the plan26 control; the 2^21 row
reproduces the rejected tree-changing arm, confirming the knob and the cliff.

## 3. Bench — Cap 120 s, seeds 0–2

`bench/run_bench.py`, reference `provari-v0.1.0 --guide hand`, candidate
`atomi default (NULL-V2)`, `--proof-check`:

| Cell | Reference | atomi | Factor | `child_evals` | D14 |
| --- | ---: | ---: | ---: | ---: | --- |
| `core` | 0.520 s | 6.910 s | 13.288 | 12,439,376 | accepted |
| `easy-win` | 0.137 s | 0.533 s | 3.891 | 1,016,547 | accepted |
| `deep-win` | 4.413 s | 77.442 s | 17.549 | 155,236,740 | accepted |

Coverage **3/3**, geomean **9.681** (three claimed cells). The `child_evals`
match the plan26 gate rows exactly, and the proof-check is 3/3 accepted by
both `atomi validate` and `provari check`. The plan25 row was geomean 11.093
with the same coverage; this row is a new, independently measured ledger row,
not a rewrite of plan25.

## 4. Gates

- `make test` — licence gate + 140 unit / 51 integration tests, green;
- `make lint` — goal check, fmt, clippy `-D warnings`, doc, green;
- `make test-full` — slow tier + `make veto`: **PASS** (8/8 accepted rows,
  four controls `valid: false`);
- P-27 tree identity — pass (default and `0` identical to the control);
- D14 — 3/3 accepted (`bench.json`, Cap 120 s runs; 600 s proof runs).

No provari material entered the repo; the licence gate is unchanged.

## 5. Problems and open parts

1. **The gate was missed, not moved.** G-N2 was declared before the data and is
   recorded as a miss. D39 is an explicit owner override; it does not revise
   the threshold and cannot be cited as a passed gate.
2. **The stored patch is wrong.** `measurements/plan26/null_v2.patch` still
   carries the 2^21 constant and would change the tree. It is historical and
   must not be re-applied; the cleaned tree is this plan's deliverable.
3. **`T_4` is still far.** 7.17× on the gate median; the cache is a ~14 %
   per-node cut, not a formulation change.
4. **Tunability has a cliff.** Requests above the filter slack take TT bytes
   and change the search; the splitter clamps and reports the allocated size,
   and the CLI flag is measurement-only until a future gate says otherwise.
5. **`src/table/mod.rs` is over the 10 KB guideline** (16.9 KB with tests,
   was 11.6 KB before); the added split/allocation tests live there. A future
   cleanup can move the cache split tests beside `null_cache.rs`.

## 6. Next steps

1. Owner commits the cleaned tree and confirms D38/D39.
2. The next plan is the **formulation decision** (PDS-PN/PN² or a goal re-pin
   under D30), not another per-node micro-plan.
3. If D39 is later reopened, use the `--null-cache-entries` knob for sizing;
   never re-apply the plan26 patch verbatim.

SESSION COMPLETE
