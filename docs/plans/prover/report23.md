# Report 23 — item 32: the proof-retention share reaches `T_3`

Plan: [`plan23.md`](plan23.md). Data:
[`measurements/plan23/`](measurements/plan23/README.md) (`env.json`,
`sweep.json`, `gate.json`, drivers, README). Decisions: D14, D18, D21, D30,
D34, new **D35**. Host: Ryzen 9 5950X (4 vCPU container), one process at a
time. Binary: `atomi@e681d9f+d`, sha256 `c25b1f6a…`.

## 0. Verdict

**RET-V1 passes.** Phase A selected the **75 %** proof share: `deep-win` is
3/3 at 120 s with all three proofs D14-accepted, while `core`/`easy-win`
stay 3/3 with no median-wall regression (ratios 1.019 / 0.990 vs the 25 %
control). The default changed from 25 % to 75 %; the `192 MB` label is
unchanged. Phase C then proved **all nine** timed runs at 120 s and accepted
all nine D14 proofs, so the engine reaches **`T_3`** (all three cells at
120 s). Geomean over the three claimed cells at Cap 120 s: **11.93**.

## 1. Phase A — share sweep (`sweep.json`)

Phase A ran on the pre-default-change binary (`65ddc4a1…`) with an explicit
`--proof-share`, so it changes no default; Phase C is the shipped binary
(`c25b1f6a…`). `--guide hand --memory 192`, seeds 0–2, 120 s timed; one 600 s proof run per
proved `deep-win` row at the same share and seed.

| Share | `core` | `easy-win` | `deep-win` | `deep-win` D14 |
| ---: | --- | --- | --- | --- |
| 25 (control) | 3/3, 8.524 s | 3/3, 0.929 s | **0/3** (`proof_full`) | — |
| 50 | 3/3, 8.595 s | 3/3, 0.923 s | **2/3** (seed 1 `proof_full`) | 2/2 accepted |
| 75 | 3/3, 8.687 s | 3/3, 0.920 s | **3/3** (113.477 / 105.297 / 77.875 s) | 3/3 accepted |

The 25 % control reproduces report22's A2 trajectory exactly on seed 0
(146,664,075 `child_evals` at 120 s, `proof_full` at 2,831,155 entries). At
50 %, seed 1 still fills the table (5,662,310 entries, `proof_full`) and
times out; seeds 0 and 2 export and both checkers accept. At 75 %, all three
seeds export: 5,716,874 / 5,275,565 / 4,179,903 entries, 5.69 / 5.25 / 4.16 M
live attacker entries, 60,691–93,092 proof positions, lengths 29–37. Every
proof run matched its timed `child_evals` exactly.

**Selection rule (fixed before the data).** Adopt the smallest share (50
before 75) with `deep-win` 3/3 and `core`/`easy-win` 3/3 with no median-wall
regression > 10 % vs the 25 % control. 50 fails on `deep-win`; 75 passes
(guard ratios 1.019 / 0.990). No other share was tried.

## 2. Phase B — default change

`src/table/mod.rs`: `split` and `Tables::new` now use 75 %; the module and
`Budget` docs say three quarters. `src/cli.rs`: the parsed default is 75 and
the field doc says so. `--proof-share` remains for measurement. The split
invariant test still covers every percent; `split_at_192mb` now pins
9,437,184 proof slots, a 2^24-bit filter and 753,664 TT clusters.

## 3. Phase C — the gate (`gate.json`)

Reference re-measured in the same session: provari `--guide hand --seconds
10 --memory 192`, 3 repeats/cell. Candidate: default share, 120 s, seeds
0–2.

| Cell | Candidate walls | Median | Reference | Factor | Control ratio | D14 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `core` | 10.190 / 8.327 / 8.211 | 8.327 s | 0.531 s | **15.68** | 0.977 | 3/3 accepted |
| `easy-win` | 0.798 / 0.910 / 1.439 | 0.910 s | 0.133 s | **6.84** | 0.980 | 3/3 accepted |
| `deep-win` | 112.144 / 105.316 / 77.783 | 105.316 s | 6.661 s | **15.81** | 0.878 | 3/3 accepted |

All nine timed runs proved. All nine 600 s proof runs matched `child_evals`
and were accepted by `atomi validate` **and** `provari check`; `deep-win`
proofs are 93,092 / 72,787 / 60,691 positions, lengths 36 / 37 / 29. The
`core`/`easy-win` walls are at or below the 25 % control and below plan21's
Cap-600 medians (8.468 / 0.919 s), so the guard and the "may only improve"
clause hold. `T_3` is reached; the geomean is recorded at Cap 120 s only
(D34).

## 4. Gates

On the shipped binary (`atomi@e681d9f+d`, sha256 `c25b1f6a…`):

- `make test` — licence gate + 128 unit / 51 integration tests, green;
- `make lint` — goal check, fmt, clippy `-D warnings`, doc, green;
- `make test-full` — slow tier + `make veto`: **PASS** (8/8 accepted rows,
  four controls `valid: false`).

The default change touches the table and proof output, so the full tier and
veto were run rather than inferred. No proof-file semantics changed.

## 5. Problems and open parts

1. **The first Phase A driver pass under-counted D14.** `accepted()` looked
   only at a timed row's `checks`, which timed runs never have, so the
   selection printed `chosen_share: null` even though the 75 % proof runs
   were accepted. The driver now reads the proof run's checks and
   `child_evals_match`; `--recompute` re-evaluated the committed
   `sweep.json` without rerunning anything. The raw rows were unaffected.
2. **The share is a search lever, not a budget increase.** The 192 MB label
   is unchanged; 75 % moves space from the TT to the proof table. The
   non-monotone effect (50 % is worse than 75 % on `deep-win`) is why the
   fixed rule required both arms.
3. **`deep-win` is still the wall.** Its median is 105.3 s at 120 s, so
   `T_4` (10 s) is far away; the next rung is not a share question.
4. **No new test is missing for the share itself.** The split invariant
   covers every percent and the 192 MB pin was updated; the D14 contract is
   exercised by the gate data, not by a unit test.
5. **Next:** item 29 (plain-move grouping) is next in the adopt order. The
   event-log export remains a fallback, not a parallel plan.

SESSION COMPLETE
