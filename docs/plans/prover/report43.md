# Report 43 — item 56 GUIDE-SCALE-FREE (quantile EF-PN map)

Plan: [`plan43.md`](plan43.md). Initiative: [`initiative.md`](initiative.md),
D33/D47/D49, [`gotchas`](gotchas.md). Data:
[`measurements/plan43/`](measurements/plan43/README.md). **No engine, CLI,
default, test or `bench.md` row changed.** Phase 0/Phase A ran in `/tmp`
builds; the repository gained only data, drivers and this report.

## 1. What was run

- **Phase 0 (fit):** eight fixed `corpus-v1` (`bench/corpus.json`) `sample20`
  **0-based** indices `0,1,2,4,5,6,7,8` (3 = `easy-win`, 10 = `deep-win`
  excluded), seed 0, `--evals 2000000`, under a `/tmp` histogram build
  (`30b4b9fe…`). The histogram records the oriented attacker eval at each
  fresh EF-PN read point, split by node kind and parent/child board. The
  extra `hand_eval` calls do not change the tree: `deep-win` reproduces
  plan33's 5,803,257 expansions exactly.
- **Diagnostics:** `core`, `easy-win`, `deep-win`, and the fixed held-out
  `guidance-v1` 0-based 6–10, seed 0, 120 s. The diagnostic walls are from
  the instrumented build and were not used as controls.
- **Phase A:** the clean `/tmp` build (`cfee2b8e…`) with the frozen quantile
  map behind `ATOMI_EF_MAP`; `deep-win`, `core`, `easy-win`, seed 0, 120 s,
  paired with same-session w = 16 controls.
- **Gates:** `make test`, `make lint`, `make test-full` + veto.

## 2. Phase 0 — freeze (G-43a PASS)

The fit distribution is narrow and skewed, not the ±128 scale plan19 probed:

| Histogram | n | p10 | p20 | p50 | p80 | p90 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OR parent | 6,848,889 | −7 | −1 | 12 | 29 | 38 |
| AND parent | 6,246,023 | −6 | 0 | 12 | 24 | 32 |
| shared parent | 13,094,912 | — | 0 | — | 26 | — |

Frozen before Phase A (full table in the measurement README):

- **Read point:** parent `hand_eval`, oriented to the attacker. This is R2's
  read point, the one the reused factor table belongs to.
- **Edges:** one shared edge pair, the 20th/80th percentiles of the shared
  parent distribution: **lo = 0, hi = 26**. A per-node-kind split is ideation
  B, not this plan (risk iv).
- **Step:** `0` below lo, `1` in `[lo, hi)`, `2` at/above hi.
- **Table:** R2's, reused — OR `pn ×{11,6,1}`, `dn ×{1,11,21}`; AND
  `pn ×{21,11,1}`, `dn ×{1,6,11}`; max/min ratio **21×**.
- **Replacement:** the band replaces the linear `f = clamp(16 + e·w/16, 8,
  32)` weight map (the plan's "removes the arbitrary weight and the 60 %
  saturation"). The control is the default `--guide hand` w = 16 stack.

G-43a: pure function of the eval; fit sample and histograms recorded; factor
ratio 21× inside the ≥4×/≤21× band; no claim or held-out cell in the fit; the
bands separate (0/26), so the map is not a constant. **PASS.**

## 3. Phase A — G-43b FAIL (deep-win passes, easy cells regress)

`--guide hand --memory 192 --seed 0 --seconds 120`, one process at a time.

| Cell | control ms | map ms | ratio | control exp | map exp | control len | map len |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `deep-win` | 54,164 | 41,449 | **0.765×** | 5,803,257 | 4,359,526 | 36 | 34 |
| `core` | 4,965 | 7,900 | 1.591× | 424,249 | 750,081 | 15 | 19 |
| `easy-win` | 366 | 777 | 2.123× | 37,999 | 80,338 | 19 | 19 |

`deep-win` is inside the 0.80× bar and the length does not increase (36 → 34),
so the quantile band is a real `deep-win` tree lever. But `core` is 1.59× and
`easy-win` 2.12× slower, far outside the 10 % guard. **G-43b FAIL ⇒ stop.**

This is exactly plan risk (iii): the map repeats w = 2's `deep-win`/easy-cell
split. It is also consistent with plan42's held-out evidence — the same easy
cells regress under an aggressive `deep-win`-friendly EF-PN map. Phase B was
not run: no `--guide-ef-map quantile`, no unit test, no default change, no
`bench.md` row.

## 4. Gates and boundary

| Gate | Result |
| --- | --- |
| G-43a frozen distribution map | **PASS** — fit sample, histograms, shared edges 0/26, 21× table |
| G-43b `deep-win` ≤0.80×, length not increasing, easy cells ≤10 % | **FAIL** — `deep-win` 0.765× / length 34, but `core` 1.591×, `easy-win` 2.123× |
| G-43c Phase B matrix/D14/held-out | **not applied** — G-43b fail, no Phase B |
| G-43d default | **not applied** — no default change |
| G-43e `make test`, `make lint`, `make test-full` + veto | **PASS** — all tests, `VETO: PASS` |
| G-43f boundary | **PASS** — no new eval term, no model, no provari source, no `T_4`/`T_5` claim, no item 48/44/50 re-open, no default |

## 5. Problems, scope and missing tests

- **The fit sample is a proxy, and it is much narrower than expected.** A
  median of 12 and a p80 of 26 mean the 20th/80th band is a tight split; most
  nodes sit in step 1, where the table's middle factors are still 6/11 (not
  neutral). The map is therefore less "distribution-normalised" than the
  plan hoped: the distribution's scale is ~10, so any three-step table with
  factors 11/6/1 is dominated by the table, not the edges. A z-score map
  (ideation C) would have the same issue unless its factors are re-derived.
- **One seed, one frozen edge pair.** Phase A is seed 0 only; the gate was
  pre-registered, so the negative is decisive for this map, not for the
  family. A different factor table or an additive map is a new test.
- **Interpretation choice:** the map replaced the linear weight map. An
  additive band on top of w = 16 would be a different candidate and was not
  run; the frozen README records the replacement before Phase A.
- **The instrumented diagnostic walls are not controls** (91.4 s vs the
  clean 54.2 s on `deep-win`). Counters are tree-identical; only wall is
  affected. Phase A uses the clean binary.
- **No D14 run.** No proof was claimed and no product stack changed; the
  control stack is the plan33/42 stack already D14-accepted 9/9.

## 6. Next steps

1. **Record the negative and close item 56.** The quantile band is not a
   general default: it buys 0.765× on `deep-win` but costs 1.59–2.12× on
   `core`/`easy-win`, the same split as w = 2 and the plan42 held-out set.
2. **If the owner wants the `deep-win` gain,** follow plan43 §8 option 3: a
   `T_4`-only opt-in under D49, with the held-out direction reported, not a
   default. That needs a new plan (it would re-use the frozen map and add the
   Phase B matrix/D14 on the opt-in stack).
3. **Do not tune the edges post-hoc.** Any further quantile work should
   change the formulation (ideation C's z-score with re-derived factors, or
   ideation B's per-node-kind split), not the 0/26 edges on this table.
4. **The instrument items are still the unblocked path:** items 49
   (BENCH-POWER) and 53 (TREE-METRIC) together, so the next tree lever is
   judged by a gate that can see it.

## 7. Follow-up options

1. **Kickoff (recommended):** “Read `docs/plans/prover/report43.md`. Record
   item 56 as executed negative: the frozen quantile band (shared parent
   edges 0/26, R2 table) is 0.765× on `deep-win` at 120 s seed 0 (length 34
   vs 36) but 1.591× on `core` and 2.123× on `easy-win`, so G-43b fails and
   no Phase B/default is taken. Then scope items 49 + 53 together as the next
   plan: the widened tuning sample and the tree-metric harness, so a
   tree-changing lever is not judged by a blind gate. No engine change; no
   `T_4`/`T_5` claim.”
2. If the owner rules for a `T_4`-only gamble: write a new plan for an opt-in
   `--guide-ef-map quantile` on the `deep-win` row only, carrying the frozen
   edges and the R2 table, with the fixed held-out direction (guidance-v1
   6–10) and D14 9/9 before any claim.
3. If the map family is to be re-tested: re-derive the factors from the fit
   distribution (z-score, ideation C) rather than reusing R2's 11/6/1 and
   21/11/1; the measured distribution's scale is ~10, so the table dominates
   the edges.

SESSION COMPLETE — Phase 0 frozen a shared-parent quantile band (0/26) with
R2's factor table; Phase A gives `deep-win` 0.765× (length 34) but `core`
1.591× and `easy-win` 2.123×, so G-43b fails, Phase B is not run and no repo
engine/default changes. Gates green; data in `measurements/plan43/`, report
in `report43.md`.
