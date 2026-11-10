# plan12 measurements

Plan: [`../../plan12.md`](../../plan12.md) (item 21, `deep-win` tree size:
entry reuse). Report: [`../../report12.md`](../../report12.md). Parsed output
only: no raw transcripts, provari artifacts, proof files or `__pycache__` are
committed. Phase B (ENTRY-REUSE) and Phase C (120 s matrix + D14) were **not
run**: the Phase A stop rule fired.

## Files

| File | Content |
| --- | --- |
| `env.json` | host, toolchain, git head, atomi binary, corpus/guidance sha256, provari version/commit, bare flags, fixed-eval point |
| `attribution.json` | Phase A: bare tree at 20 M evals on core/easy-win/deep-win, seed 0, with `--stats`; deep-win 120 s with and without `--stats`; the stop-rule record |
| `coverage.json` | Phase B/C record: **not run**, because the Phase A stop rule fired |
| `run_plan12.py` | stdlib-only driver (`env`, `attribution`, `coverage`, `all`) |

No coverage rows and no D14 rows: Phase A stopped before Phase B. The plan's
stop rule is explicit: if `entry_would_return` is below 10 % of expansions at
20 M on `deep-win`, stop and report; do not implement ENTRY-REUSE.

## Commands

```sh
cargo build --release
python3 docs/plans/prover/measurements/plan12/run_plan12.py --phase env
python3 docs/plans/prover/measurements/plan12/run_plan12.py --phase attribution
python3 docs/plans/prover/measurements/plan12/run_plan12.py --phase coverage
make test
make lint
make test-full
```

One atomi process at a time. `attribution` ran first (four 20 M rows and two
120 s deep-win rows), then `coverage` (which only records the stop), then the
gates.

## Row counts and units

- **Phase A:** 3 rows at `--evals 20000000 --memory 192 --seed 0 --stats`
  (core/easy-win/deep-win) + 2 rows at `--seconds 120 --memory 192 --seed 0`
  (deep-win without and with `--stats`). Five rows total.
- `expansions` is a main-search node expansion (the analogue of provari's
  `nodes`). `child_evals` is one per child classified during an expansion.
- `unique_keys` is the distinct-key count from the `--stats` set (capacity
  2^21, load bound 70 %). `re_expansions = expansions - unique_keys`.
- `entry_would_return` counts node-entry stored values whose `(pn, dn)`
  already crosses the incoming `(tpn, tdn)`; `entry_proof_hits` /
  `entry_tt_hits` count the stored-value hits by table. They are exact at
  both budgets (plain counters, not capped).
- The 120 s `--stats` row's `unique_keys` is **capped** at 1,468,006 (it hits
  the set's load bound), so its `re_expansions` is an upper bound, not an
  exact ratio. The exact re-expansion ratio is the 20 M row; the 120 s row is
  used only for its exact entry counters and depth histogram.

## Phase A — attribution at fixed 20 M evals

`--guide none`, `--evals 20000000 --memory 192 --seed 0 --stats`.

| Cell | expansions | unique_keys | re_expansions | re % | proof hits | TT hits | would_return | would % | attacker | defender | ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| core | 630,056 | 586,075 | 43,981 | 6.98 % | 2 | 42,560 | **61** | 0.0097 % | 191,492 | 438,564 | 1,256 |
| easy-win | 669,920 | 622,562 | 47,358 | 7.07 % | 50 | 43,795 | **520** | 0.0776 % | 218,919 | 451,001 | 1,217 |
| deep-win | 668,608 | 620,162 | 48,446 | **7.25 %** | 0 | 46,955 | **53** | **0.0079 %** | 279,978 | 388,630 | 1,161 |

The all-`Quiet` `selected_class` histogram is expected: `--guide none` carries
no move classes, so every selected child is class `Quiet` (rank 9).

## Phase A — deep-win at 120 s

| Flags | exit | expansions | child_evals | ms | proof_entries | unique_keys | entry counters |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `--seconds 120` | timeout | 51,733,504 | 1,428,233,879 | 120,000 | 2,750,084 | — | — |
| `--seconds 120 --stats` | timeout | 49,730,560 | 1,379,493,488 | 120,000 | 2,649,166 | 1,468,006 (capped) | proof 193, TT 3,253,672, **would_return 15,849** (0.0319 %) |

The 120 s `--stats` row has extra `--stats` overhead and a 2^21-slot key set;
the clean wall row is the first. The depth histogram of the 120 s `--stats`
row reaches ply 52 (bucket cap 63), with large mass at plies 7–17 and a long
tail; the 20 M row only reaches ply 17 because the eval cap cuts first.

## Stop rule

At 20 M on `deep-win`: `entry_would_return / expansions` = 53 / 668,608 =
**0.0079 %**, far below the 10 % threshold. The 120 s row is 15,849 /
49,730,560 = 0.0319 %. **Fired.** Phase B (ENTRY-REUSE)
and Phase C (120 s matrix + D14) were not implemented or run. The exact 20 M
re-expansion ratio is 7.25 %, not 10–20×; the premise's 18.5 ratio compared
expansions with proof entries, not with distinct expanded keys.

## Gates

| Gate | Result |
| --- | --- |
| G1 data | pass: 5 parsed attribution rows + stop record; `env.json`; one process at a time |
| G2 stop rule | **pass (fired)**: 0.0079 % < 10 % at 20 M on deep-win |
| G3 mechanism | not applicable: Phase B not started; Tier 1 remains 2/3 (report11) |
| G4 soundness | pass: no refutation; no new proofs; `make test`, `make lint`, `make test-full` and `make veto` green |
| G5 docs | pass: no GPL dependency, raw transcript, provari artifact, proof file or `__pycache__` |
