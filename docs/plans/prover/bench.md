# Bench — performance ledger vs. provari

Status: **standing document of the [`prover`](initiative.md) initiative**, the
successor of `winprover`'s ledger. It fixes the bench set, the metric and the
reading rules, and appends **one row per executed plan**, each at the cap it
was measured at (D34). It is not a plan and not a report: plans live in
`planN.md`, their evidence in `reportN.md` and `measurements/planN/`.

| If you want | Read |
| --- | --- |
| the goal, the rungs `T_1`–`T_5`, their budgets and the current position | [`initiative.md`](initiative.md) (D30) |
| the reasoning behind a number, a convention or a closed line | [`decisions.md`](decisions.md) — D10, D11, D30, D32–D34 for this file |
| what happened, plan by plan | [`history.md`](history.md), the plan's `reportN.md` |
| to measure or re-run a row yourself | [`bench/README.md`](../../../bench/README.md) |

This file states no goal, no rung and no cap policy (D34): it records what was
measured, at the cap each row used, and makes no goal claim.

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §The metric, including §Canonical measurement protocol -->
<!-- adapted 2026-11-12: reformatted into lists and footnotes, content unchanged -->

## The metric

```
factor(cell) = median candidate wall [s] / median provari --guide hand wall [s]
```

- **provari `--guide hand` is 1.000 by definition** — the fair
  hand-evaluation-for-hand-evaluation column. The bundled model column
  (`--guide model`) is recorded separately and never mixed in: it needs an
  out-of-scope external artifact (D8).
- **`< 1.0` means we are faster than provari on that cell.**
- **Ledger score** = geometric mean of `factor(cell)` over the row's *claimed
  cells* — cells where both sides prove a win at the row's `Cap` and the claim
  passes D14. Unclaimed cells have no factor: they count in **coverage**, not
  in the mean, and the mean names its cell count.
- **Censoring.** A cell that is not a claimed win at **the row's own cap** is
  *censored*; a meaningful bound is printed as `≥ cap / reference_wall`, a
  *lower* bound on the factor, never a factor. A run that reaches a mate whose
  claim fails (D14, e.g. an export refused as `store-full`) is *not claimed
  (`<verdict>`)* with the found wall. A censored cell is never dropped and
  never converted into a win.
- **Rungs are not a column.** [`initiative.md`](initiative.md) decides which
  rung a set of measurements reaches (D30); rows keep their own session and
  are never rewritten.
- **Deterministic accompaniment.** Every row also records `child_evals` (ours,
  deterministic per stack) and provari's `nodes` for the same cell. They are
  not comparable one-to-one (D11): evals per expansion against provari's nodes
  is a diagnostic, not a ratio to optimise.

### Measurement protocol

| Item | Value |
| --- | --- |
| Reference side | `provari prove "<FEN>" --guide hand --seconds 10 --memory 192`. Fixed: the set is selected so the hand guide proves inside 10 s, so the reference cannot move with a candidate cap |
| Candidate side | `atomi prove "<FEN>" --seconds <cap> --memory 192 [--seed N] <stack flags>`; `--memory` covers the TT, the proof table and the filter (D18) |
| Caps measured | reference 10 s always; candidate at the row's `Cap` (10 s, 120 s, 600 s so far) |
| Repeats | 3 per cell per side, medians reported, one process at a time |
| Session | a row's reference and candidate runs are measured back to back on one host (reading rule 1) |
| Oracle | `provari-v0.1.0`, rebuilt from source per row (`make oracle`), never committed |
| Corpus | `corpus-v1` = [`bench/corpus.json`](../../../bench/corpus.json), sha256 `9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e`; the driver refuses any other digest |
| Tablebases | none (D7) |
| Stretch column | provari `v0.2.0` hand / model, separately labelled, never inside a factor (D17, D8) |

The driver is [`bench/run_bench.py`](../../../bench/README.md) (flags, sides
and the derived ledger there); rows before 2026-11-11 were measured with their
plan's own driver under `measurements/planN/`.

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §The bench set -->
<!-- adapted 2026-11-12: reference values split into columns + footnotes, census prose compressed -->

## The bench set

Three cells, drawn from `corpus-v1` (`core` + the pinned `sample20`).
**Selection rule (fixed):** the bench set is every cell of `core + sample20`
where the pinned provari oracle proves a win **with the hand guide** inside the
canonical budget. A cell where provari itself is censored cannot carry a factor
and is not a bench cell, however interesting it is.

| Short name | Cell id | FEN | hand wall[^ref] | hand nodes | mate | Why this cell |
| --- | --- | ---: | ---: | ---: | --- | --- |
| `core` | `core`[^coreid] | `r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1` | 0.49 s | 342,289 | 17 | the initiative's motivating FEN; here the hand guide *helps* provari |
| `easy-win` | `1eb80b92f92c` | `rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1` | 0.11 s | 72,523 | 15 | the one easy shared win; here the hand guide *hurts* provari (>2×), so the lever is case-dependent |
| `deep-win` | `db6a741c723f` | `rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1` | 6.20 s | 4,574,912 | 22 | the deepest hand-guide win in the corpus; the coverage cell |

[^ref]: plan1 set-card medians. Every row **re-measures** its own reference
    (same session, rule 1); the values seen so far are 0.49–0.53 s (`core`),
    0.10–0.14 s (`easy-win`), 6.2–6.8 s (`deep-win`). Never take a reference
    from this table for a new factor.

[^coreid]: `core` is the one hand-named cell — it is not in `high200.jsonl`.
    Its corpus fingerprint would be `8d0ff8bca7f0`, so a measurement file
    showing that id means this position.

Cells measured and deliberately **out** of the set, so the exclusion is not
silently revisited:

| Cell id | Why not a bench cell |
| --- | --- |
| `b9a7ed7eb368`, `dd3f89766a40` | provari **hand** censored (10.1 M / 9.3 M nodes); only the out-of-scope bundled model wins them (4.46 s / 1.86 s)[^census] |
| the other 16 `sample20` ids | provari's model is censored on all of them, so the hand guide is expected to be censored too |

[^census]: The item-1 hand census (plan1) re-measured all of `core + sample20`
    with a freshly rebuilt `provari-v0.1.0` at 10 s / 192 MB, both guides side
    by side: provari `--guide hand` wins **exactly the three cells above**
    (2/20 on `sample20`), `--guide model` 5/21 (adding the two model-only wins),
    atomi `default` 1/21. Numbers: `measurements/plan1/`.

If a later census finds an additional hand-guide win, that cell joins the set
by the rule above — short name, measured reference, a new reference row. Earlier
rows are not rewritten.

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §Names and ids -->
<!-- adapted 2026-11-12: cell-id detail and pins compressed; pins live in the ledger table -->

## Names and ids (read this first)

Three kinds of identifier appear here and in the measurement files; **only one
of them is a git commit**.

| Kind | Example | Meaning |
| --- | --- | --- |
| Cell short name | `core`, `easy-win`, `deep-win` | use this in prose, plans, reports and ledger rows; defined in [The bench set](#the-bench-set) |
| Cell id | `1eb80b92f92c` | provari's record id in its corpus file — **not** a git hash: the first 12 hex of `sha256(FEN)` at the pinned commit, unique inside that file. Tooling and `corpus.json` key by it |
| Commit pin | `atomi@b52e186`, `provari-v0.1.0` | a real commit: the oracle build, or the tree a row was measured at; full sha in the [pins](#pins) table |

Rule of thumb: prose uses short names, tooling uses cell ids, rows record the
commit pins. A cell id is stable forever; a short name is stable as long as the
cell is in the set.

<!-- from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/bench.md §Reading rules -->
<!-- adapted 2026-11-12: examples shortened; rule 4 now points at the stack-gate decision -->

## Reading rules (do not violate these when appending a row)

1. **Same session, same host.** A factor compares two walls measured back to
   back on one host. Cross-host precision is ~±15–20 % (the `core` reference
   alone has measured 0.403–0.529 s, and our per-eval throughput differed by
   ~40 % between an x86 and an aarch64 host), so a difference between rows is
   not a lever effect until it is re-measured, or the drift is stated.
2. **Wall is the product metric; `child_evals` is the mechanism metric.** A
   child-eval cut that does not move the wall is a mechanism finding, not a
   speed win — and a wall win at identical evals is a per-node cost effect
   (D11).
3. **Censored is a result — at the row's cap.** `≥ cap/reference` bounds and
   coverage counts are the honest form. A longer run is **its own row** with its
   own `Cap`; a cap is never chosen to drop a difficult cell (cap shopping,
   rule 6).
4. **Opt-in ≠ shipped.** The `Stack` column is part of the result: only a
   default-flag decision (a passed stack gate, D21) turns a stack row into a
   product row. Until then every stack row stays visible.
5. **Correctness gates precede the number.** A row is written only after the
   plan's gates passed (`make test`; `make test-full`/`make veto` for
   behaviour-changing levers) and, for new proof shapes, after the D14 claim
   protocol accepted the exported proof.
6. **Coverage is stated in every row, and geomeans are compared only within one
   cap.** State `(cap, coverage, geomean)` together. The geomean is
   **non-monotone in the cap** — a longer cap claims more cells, hence more
   factors in the mean — so a cross-cap comparison is a reading error and a cap
   picked per plan to improve the headline is cap shopping. A rung is reached
   only through what `initiative.md` says.

## Ledger

**Convention (D34).** The ledger is a **results history**: one row per executed
plan, each with the budget it was measured at. `Cap` is part of the result, so
`censored` always means "not claimed at this cap". A factor itself needs no
budget — it is `candidate wall / provari hand wall`, and the reference is
re-measured per row inside its own 10 s budget. Decisions append no row.

### Factors

| Plan[^cells] | Date | Host[^host] | Stack | `core` | `easy-win` | `deep-win` | Geomean[^geo] | Cap |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `plan1`[^p1] | 2026-11-06 | `ae54d05b36f8` | atomi **m0** | censored | **36.365** | censored | **36.365** (1) | 10 s |
| `plan2`[^p2] | 2026-11-06 | `ae54d05b36f8` | atomi **m1** | censored | **10.897** | censored | **10.897** (1) | 10 s |
| `plan11`[^p11] | 2026-11-10 | `e10de197da58` | atomi **bare** (`--guide none`) | **35.3** | **19.1** | 1/3 | **25.9** (2) | 120 s |
| `plan15`[^p15] | 2026-11-10 | `e10de197da58` | atomi **bare** (`--guide none`) | **35.8** | **18.9** | 1/3 | **26.0** (2) | 120 s |
| `plan19`[^p19] | 2026-11-11 | `e10de197da58` | atomi **default** (R1+R3+R6) | 2/3 | **78** | 0/3 | **78** (1) | 120 s |
| `plan20`[^p20] | 2026-11-12 | `e10de197da58` | atomi **default** (AUX-V1) | **16.3** | **7.0** | 0/3 | **10.8** (2) | 120 s |
| `plan21`[^p21] | 2026-11-12 | `e10de197da58` | atomi **default** (AUX-V1) | **16.28** | **6.91** | not claimed | **10.6** (2) | 600 s |
| `plan23`[^p23] | 2026-11-12 | `e10de197da58` | atomi **default** (RET-V1) | **15.68** | **6.84** | **15.81** | **11.93** (3) | 120 s |
| `plan25`[^p25] | 2026-11-12 | `e10de197da58` | atomi **default** (NULL-V1) | **14.679** | **4.598** | **20.227** | **11.093** (3) | 120 s |
| `plan27`[^p27] | 2026-11-12 | `e10de197da58` | atomi **default** (NULL-V2) | **13.288** | **3.891** | **17.549** | **9.681** (3) | 120 s |
| `movegen plan1`[^mg1] | 2026-11-13 | `e10de197da58` | atomi **bare** (`--guide none`), 2.6.0 | censored | **13.832** | censored | **13.832** (1) | 10 s |
| `movegen plan1`[^mg1] | 2026-11-13 | `e10de197da58` | atomi **bare** (`--guide none`), 2.6.0 | **35.103** | **14.403** | censored | **22.485** (2) | 120 s |
| `movegen plan2`[^mg2] | 2026-11-13 | `e10de197da58` | atomi **bare** (`--guide none`), 2.7.0 | censored | **14.046** | censored | **14.046** (1) | 10 s |
| `movegen plan2`[^mg2] | 2026-11-13 | `e10de197da58` | atomi **bare** (`--guide none`), 2.7.0 | **33.881** | **13.603** | censored | **21.468** (2) | 120 s |
| `movegen plan3`[^mg3] | 2026-11-14 | `e10de197da58` | atomi **bare** (`--guide none`), 2.8.0 | censored | **13.587** | censored | **13.587** (1) | 10 s |
| `movegen plan3`[^mg3] | 2026-11-14 | `e10de197da58` | atomi **bare** (`--guide none`), 2.8.0 | **34.913** | **14.226** | censored | **22.286** (2) | 120 s |
| `plan31`[^p31] | 2026-11-14 | `e10de197da58` | atomi **HOTPATH-V2** (`--guide hand`, D43) | **10.034** | **2.874** | **13.106** | **7.23** (3) | 120 s |
| `plan32`[^p32] | 2026-11-14 | `e10de197da58` | atomi **HOTPATH-V2 + G1** (`--guide hand`, D43) | **9.735** | **2.835** | **12.773** | **7.064** (3) | 120 s |

[^host]: the plan's `env.json` machine id. Factors are same-session ratios, so a
    row is valid on its own host and never against another row's (rule 1).

[^geo]: geometric mean over the cells **claimed** at that cap; the parenthesised
    number is the cell count. Never compared across caps (rule 6).

[^cells]: Cell syntax — a **bold number** is a claimed factor (candidate wall /
    provari hand wall, both from the row's own session). `censored` means no
    claimed win at the row's cap (plan1/plan2 used no seeds; `--seed` arrived
    with D22). A seed fraction (`1/3`, `2/3`) means proved on that many of
    seeds 0–2, hence censored and carrying no factor; `not claimed` means the
    mate was found but the claim failed D14 (verdict in the row's note). Walls,
    `child_evals` and D14 verdicts of each row are in its footnote.

[^p1]: **plan1 — M0, heuristic-free DF-PN** (uniform starting numbers, movegen
    order, no guide): the correctness floor and M1's per-node baseline, no
    performance claim. `easy-win` 4.982 s / 30,245,030 child evals; D14 proof
    2,083 positions, length 17, accepted by `atomi validate` and `provari
    check`. Censored: `core` ≥ 18.76 (59.5 M evals), `deep-win` ≥ 1.52 (61.8 M).
    The legacy `solver@5f85133` controls of this session are in the [plan1
    note](#plan1), not in this table (D10 amendment).

[^p2]: **plan2 — M1**, same tree as m0, only per-node cost changed. `easy-win`
    1.482 s / 30,335,618 child evals (m0: 30,245,030); D14 proof 2,048
    positions, length 17, accepted by both checkers. Censored: `core` ≥ 18.9
    (181.8 M evals), `deep-win` ≥ 1.48 (186.6 M) — ≈ 18 M child evals/s instead
    of ≈ 6 M.

[^p11]: **plan11 — the bare tree** (`--guide none`). `core` 17.390 s and
    `easy-win` 2.008 s, 3/3 seeds each; `deep-win` 1/3 — seed 1 proves at
    111.218 s (1.399 G child evals, length 25), seeds 0/2 censor at 120 s
    (≥ 18.6). 7/7 D14 proofs accepted by both checkers. The bare tree dominated
    the product hand on every measured cell, so no portfolio was built.

[^p15]: **plan15 — same stack as plan11**, an independent session. `core`
    17.471 s and `easy-win` 2.023 s (3/3); `deep-win` 1/3 — seed 1 113.458 s,
    seeds 0/2 censor. 7/7 D14 accepted. The plan's 10 s head-to-head (coverage
    1/3, shared-cell geomean 15.757) is in [`report15.md`](report15.md).

[^p19]: **plan19 — default `--guide hand` as R1+R3+R6.** `core` 2/3 (118.0 /
    115.1 s, seed 2 censors), `easy-win` 3/3 (median 10.4 s), `deep-win` 0/3
    (≥ 18.4). Tree-only the stack cuts `core` 4.74×, `easy-win` 3.34× and
    `deep-win` 2.80× against the product hand, but the wall is
    auxiliary-bound: the tree cut does not pay for the auxiliary passes, so at
    this cap the default is slower than the bare tree on every cell. D14 16/16
    accepted. Numbers: [`measurements/plan19/`](measurements/plan19/README.md).

[^p20]: **plan20 — AUX-V1** (exact sibling simulation, exact null-threat
    replay, board-free closures). `core` 3/3 at 8.36 s (2/3 → 3/3) and
    `easy-win` 3/3 at 0.91 s; `deep-win` 0/3 at this cap (≥ 18.5), with the
    mate found at 128–175 s and `proof_full` appearing — the item-30 trigger.
    Tree-only vs the plan19 default: `core` 5.88×, `easy-win` 4.65×,
    `deep-win` 1.66× in child evals. `aux_nodes` per expansion is reported as a
    miss (7–13) because exact replays are per-defence work (D33). Numbers:
    [`measurements/plan20/`](measurements/plan20/README.md).

[^p21]: **plan21 — the same stack at Cap 600 s**, reference re-measured in the
    same session. `core` 8.468 s median (10.217 / 8.468 / 8.217) / 10.10 M
    child evals, `easy-win` 0.919 s (0.806 / 0.919 / 1.431) / 1.18 M; 6/6 D14
    accepted, proofs 13,751–16,991 and 2,794–4,100 positions. `deep-win` is
    **not claimed**: the mate is found on 3/3 seeds (175.408 / 132.185 /
    153.740 s, 188.6 M evals, lengths 28 / 36 / 34) but the 192 MB proof table
    is full (`proof_full`, 2,831,155 entries) and the export fails closed
    (`proof: store-full`, exit 3), so neither checker has anything to accept.
    At 384 MB the seed-1 proof exports and both checkers accept it
    (94,686 positions, length 35), so the blocker is the proof store, not the
    search — item 30. The walls reproduce plan20's (same trajectory), so this
    geomean is not comparable with the plan20 Cap-120 one (rule 6). Numbers:
    [`measurements/plan21/`](measurements/plan21/README.md).

[^p23]: **plan23 — RET-V1**, the default proof share moved from 25 % to
    75 % of the 192 MB label (D35). At Cap 120 s all three cells are 3/3:
    `core` 10.190 / 8.327 / 8.211 s, `easy-win` 0.798 / 0.910 / 1.439 s,
    `deep-win` 112.144 / 105.316 / 77.783 s. **9/9 D14 proofs** matched
    `child_evals` and were accepted by `atomi validate` and `provari check`;
    `deep-win` proofs are 93,092 / 72,787 / 60,691 positions, lengths 36 / 37
    / 29. The same-session reference is 0.531 / 0.133 / 6.661 s. This is the
    first `deep-win` claim at 192 MB and the row that reaches `T_3`; its
    Cap-120 geomean is not comparable with plan21's Cap-600 one (rule 6).
    Numbers: [`measurements/plan23/`](measurements/plan23/README.md).

[^p25]: **plan25 — NULL-V1** (closure reply test, one scratch board,
    defender-eval guard). At Cap 120 s all three cells are 3/3: `core`
    7.824 s, `easy-win` 0.607 s, `deep-win` 88.330 s; the same-session
    reference is 0.533 / 0.132 / 4.367 s, so the factors are 14.679 / 4.598 /
    20.227 and the geomean is 11.093 over 3 claimed cells. The same-session
    control (pre-plan binary) was 8.350 / 0.938 / 104.876 s, so NULL-V1 is
    0.784× / 0.766× / 0.796× the control with `child_evals`/`expansions`
    identical 9/9; D14 9/9 accepted (gate_v1). NULL-S
    (`--guide-null-screen`) cuts `deep-win` to 0.571× but fails `easy-win`
    (3.349×) and stays opt-in. Numbers:
    [`measurements/plan25/`](measurements/plan25/README.md).

[^p27]: **plan27 — NULL-V2** (EARLY-REPLY + the 2^17 filter-only
    exposed-limit cache; D39 owner override of the missed G-N2). At Cap 120 s
    all three cells are 3/3: `core` 6.910 s, `easy-win` 0.533 s, `deep-win`
    77.442 s; the same-session reference is 0.520 / 0.137 / 4.413 s, so the
    factors are 13.288 / 3.891 / 17.549 and the geomean is 9.681 over 3
    claimed cells. **3/3 D14 proofs** matched `child_evals` and were accepted
    by `atomi validate` and `provari check`. The default cache is 2^17 entries
    (1.25 MB, filter slack only, tree-identical); the stored plan26 patch's
    `1 << 21` constant is corrected. Numbers:
    [`measurements/plan27/`](measurements/plan27/README.md).

[^mg1]: **`movegen` plan1 — a dependency-only adoption**, not a prover stack:
    `atomic-movegen` 2.4.0 → 2.6.0 with no call-site change, so the tree is
    identical by construction and the wall is expected flat. Both caps are
    measured against a **control built from the same source tree on published
    2.4.0** (`m0`), in the same session, alternating per repeat. Cap 120 s:
    `core` 18.372 s (`m0`) / 18.394 s (`m1`), `easy-win` 1.894 / 1.930,
    `deep-win` censored on both. Cap 10 s: `easy-win` 1.934 / 1.895,
    `core`/`deep-win` censored on both. Paired cells (`easy-win`@10 s,
    `core`@120 s, `easy-win`@120 s): `m1/m0` geomean **0.9999**, tree
    identical 3/3. A 5 M-eval `deep-win` probe (`--guide hand`, seed 0)
    reproduces the plan28 control tree (168,910 expansions) on both sides at
    **1.015×** — inside the session's repeat spread, i.e. flat. The stack is
    the **bare** tree (D28) and is not compared with the hand-stack rows
    (rule 4); the censoring matches the plan11/plan15 bare rows, so coverage
    did not move. No D14 claim (no proof or search change); `make veto`
    8/8 accepted with `provari check` agreeing. Numbers:
    [`measurements/movegen/plan1/`](measurements/movegen/plan1/README.md).

[^mg2]: **`movegen` plan2 — a dependency-only adoption**, not a prover stack:
    `atomic-movegen` 2.6.0 → 2.7.0 with no call-site change (the release adds
    MG15–MG18: `terminal_after`, `child_outcome`, `post_move`,
    `classification`; atomi calls none of them yet), so the tree is identical
    by construction and the wall is expected flat. Both caps are measured
    against a **control built from the same source tree on published 2.6.0**
    (`m0`), in the same session, alternating per repeat. Cap 120 s:
    `core` 18.093 s (`m0`) / 18.194 s (`m1`), `easy-win` 1.906 / 1.850,
    `deep-win` censored on both. Cap 10 s: `easy-win` 1.868 / 1.840,
    `core`/`deep-win` censored on both. Paired cells (`easy-win`@10 s,
    `core`@120 s, `easy-win`@120 s): `m1/m0` geomean **0.987**, tree
    identical 3/3. A 5 M-eval `deep-win` probe (`--guide hand`, seed 0)
    reproduces the plan28/plan1 control tree (168,910 expansions) on both
    sides at **0.9992×**; per-expansion (`make pernode`, `core` 10 M,
    192 MB) **2,025 ns** (`m0` in the same session: 2,000; the plan1 point
    for 2.6.0 was 2,054, cross-session). The stack is the **bare** tree (D28)
    and is not compared with the hand-stack rows (rule 4); the censoring
    matches the plan1 row of the same stack, so coverage did not move. No D14
    claim (no proof or search change); `make veto` 8/8 accepted with
    `provari check` agreeing. Numbers:
    [`measurements/movegen/plan2/`](measurements/movegen/plan2/README.md).

[^mg3]: **`movegen` plan3 — a dependency-only adoption**, not a prover stack:
    `atomic-movegen` 2.7.0 → 2.8.0 with no call-site change (the release adds
    MG19 `PostMove::terminal()` / `post_move_and_key` and MG20 a positive
    existence screen; atomi calls none of them, and handoff3 §3's cost gate is
    failed, so no `prover` plan follows), so the tree is identical by
    construction and the wall is expected flat. Both caps are measured against
    a **control built from the same source tree on published 2.7.0** (`m0`), in
    the same session, alternating per repeat. Cap 120 s: `core` 18.246 s (`m0`)
    / 18.364 s (`m1`), `easy-win` 1.887 / 1.892, `deep-win` censored on both.
    Cap 10 s: `easy-win` 1.878 / 1.875, `core`/`deep-win` censored on both.
    Paired cells (`easy-win`@10 s, `core`@120 s, `easy-win`@120 s): `m1/m0`
    geomean **1.0025**, tree identical 3/3. A 5 M-eval `deep-win` probe
    (`--guide hand`, seed 0) reproduces the plan28 control tree (168,910
    expansions) on both sides at **0.9996×**; per-expansion (`make pernode`,
    `core` 10 M, 192 MB) **1,999.7 ns** (`m0` in the same session: 2,006.1;
    the plan2 point for 2.7.0 was 2,025, cross-session). The stack is the
    **bare** tree (D28) and is not compared with the hand-stack rows (rule 4);
    the censoring matches the plan2 row of the same stack, so coverage did not
    move. No D14 claim (no proof or search change); `make veto` 8/8 accepted
    with `provari check` agreeing. Numbers:
    [`measurements/movegen/plan3/`](measurements/movegen/plan3/README.md).

[^p31]: **plan31 — HOTPATH-V2** (item 38, D43): the guide's per-expansion
    multiplicity removed by three tree-neutral rewrites — the one-ply explosion
    test answered by existence (`threat::has_explosion`: blast-zone candidates
    from `blast_zone`/`attackers_to_color`, confirmed by `blast_casualties` and
    `Board::legal`, with an en-passant/promotion fallback to the list path), the
    null board from `Board::null_for` instead of a FEN round-trip, and the null
    reply test on the legal stream with `ControlFlow::Break`. The stack is
    **unconditional** (no flag), so this is a product row under rule 4; the
    hand guide itself stays opt-in. At Cap 120 s all three cells are claimed:
    `core` 5.288 s (12,439,376 child evals), `easy-win` 0.388 s (1,016,547),
    `deep-win` **58.439 s** (155,236,740); the same-session reference is 0.527 /
    0.135 / 4.459 s (342,290 / 72,523 / 3,024,117 nodes), so the factors are
    10.034 / 2.874 / 13.106 and the geomean is **7.23** over 3 claimed cells.
    **3/3 D14 proofs** matched `child_evals` and were accepted by `atomi
    validate` and `provari check` (16,991 / 2,980 / 93,092 positions, lengths
    15 / 19 / 36). Tree identity: `child_evals`/`expansions` identical to a
    `git archive` control on 3 cells at 1 M evals, on the 5 M `deep-win` probe
    (5,000,034 / 168,910) and on 9 x 120 s (cell, seed) pairs, where the stack
    is 0.7223–0.7541× the control; the 5 M probe ratio is 0.7773× and the D21
    ablations are `{F1}` 0.8920 / `{F2}` 0.9584 / `{F3}` 0.9305. `deep-win`
    77.442 s (plan27) → 58.439 s on essentially the same reference (4.413 s),
    i.e. `T_4` distance 5.84×; rows are never rewritten, so the cross-row
    difference is stated with each row's own reference. Numbers:
    [`measurements/plan31/`](../../../measurements/plan31/README.md).

[^p32]: **plan32 — G1 PROOF-PROBE-TRIM** (guide sites that only accept
    `Value::Proved` probe the proof table instead of `lookup`; item 39's
    PROOF-FILTER is closed `no effect`, nothing added). At Cap 120 s all three
    cells are 3/3: `core` 5.101 s, `easy-win` 0.377 s, `deep-win` 56.663 s
    against a same-session reference of 0.524 / 0.133 / 4.436 s, so the factors
    are 9.735 / 2.835 / 12.773 and the geomean is 7.064 over 3 claimed cells.
    The tree is **byte-identical** to plan31's (`child_evals` 12,439,376 /
    1,016,547 / 155,236,740, `expansions` and `proof_entries` equal), so the
    row's delta against plan31 is the probe trim alone, measured same-session
    at 0.986–0.987× on the three cells (n = 6–12 interleaved pairs). **3/3 D14
    proofs** matched `child_evals` and were accepted by `atomi validate` and
    `provari check` (9/9 over seeds 0–2 in `d14.json`). Numbers:
    [`measurements/plan32/`](../../../measurements/plan32/README.md).

**One engine per table (D10 amendment, 2026-11-11).** Only `atomi@*` rows form
a series. The legacy `solver@5f85133` controls measured in plan1 (`default`
45.358, 1/3; `opt` 6.322, 2/3) are a frozen one-session baseline of a different
engine: recorded in the [plan1 note](#plan1) and `measurements/plan1/`, never
ordered as if they were `atomi` rows (rule 1).
They left the table because a mixed series invites exactly that comparison.

### Pins

`+d` marks a row measured on a dirty tree[^dirty]; every row so far has one.

| Pin | HEAD | Rows |
| --- | --- | --- |
| `atomi@0c5a882+d` | `0c5a882ef9ab93c0d48d6a3f5176cfb64a0a78bb` | plan1 |
| `atomi@a4c135d+d` | `a4c135d6ee1956588e01733bd101f6373e7d2e72` | plan2 |
| `atomi@b52e186+d` | `b52e186e44f617d5930eaae2916d902d464f438b` | plan11 |
| `atomi@8953596+d` | `89535961ebdd73e78d7b2267177da1eb87bc3216` | plan15 |
| `atomi@e71289b+d` | `e71289b630f87137774810403492eedba10d22be` | plan19 |
| `atomi@772b3cb+d` | `772b3cb767a07b28bcd7260e83a13f190f2bbd1b` | plan20, plan21 |
| `atomi@e681d9f+d` | `e681d9f2c45c4bb45e9434fbf817217504ed368e` | plan23 |
| `atomi@a257e6f+d` | `a257e6f28d29748c6208374881773b6e69435f6d` | plan25 |
| `atomi@7cc3494+d` | `7cc34948a65566790bab77d0f9c8a0c1ffb91838` | plan27 |
| `atomi@9617f9b+d` | `9617f9b3c3feebded778892f7b29da57305c0679` | `movegen plan1` |
| `atomi@f88533e+d` | `f88533eb8135cdfb81d89aa2de9fbf9102f1c8d9` | `movegen plan2` |
| `atomi@d675701+d` | `d675701fe9c00c5deb5b643ab5c7340e29b96794` | `movegen plan3` |
| `atomi@7f3728d+d` | `7f3728d96354d330b490143cff5c1ef737cd4dfe` | `plan31` |
| `atomi@69b6f50+d` | `69b6f501fa7f4179f235e6c866f0f1fa78d3c42b` | `plan32` |
| `provari-v0.1.0` | `48686bc6c786d6ea70568f0b2ad4002cf0520206` | the 1.000 side of every factor |
| `solver@5f85133` | `5f85133ea9d751368a682e53e354bd3fc0016d00` | the frozen legacy control of the [plan1 note](#plan1), not a row |

[^dirty]: the work was uncommitted at measurement time, so the pin names HEAD,
    not the tree the binary was built from. The binary is identified by its
    sha256 in that plan's `measurements/planN/env.json`.

plan21 ran after plan20 was committed, so its `env.json` HEAD is
`c2eda585e0fcfa74bacca80bce4c65735b4ac071`; its binary is byte-identical to
plan20's (`4df19305…`), which is why both rows carry the same engine pin.

## Plan notes

Only `plan1` needs a note of its own — D10 references it as the home of the
frozen legacy baseline. Every other row's context is in its footnote and in
`reportN.md`.

<!-- the anchor `#plan1` is referenced by D10 and by the ledger footnotes -->

### plan1

Numbers: [`measurements/plan1/bench.json`](measurements/plan1/bench.json);
per-node: [`measurements/plan1/pernode.json`](measurements/plan1/pernode.json).

- **Frozen legacy controls** (post-`atomic-movegen` 2.4.0, one session):
  `default` **45.358** on one shared cell (`easy-win`), coverage 1/3;
  `opt` **6.322** on two (`core` 7.43, `easy-win` 5.38), coverage 2/3; both
  censored on `deep-win`. This is the initiative's re-measured baseline gap: a
  different engine from every `atomi` row, measured once in plan1's session —
  never a ledger row, never ordered against `atomi` rows (rule 1).
- **Per-node baseline (D11) and M1's reference:** `core` at `--evals 10000000`,
  192 MB, median **162.0 ms per 1 M child evals** (≈ 5.18 µs per expansion,
  ≈ 32 child evals per expansion, peak RSS ≈ 131 MiB — the M0 table rounds its
  bucket count down to a power of two). M1's own same-session gate (plan2) put
  m1 at 53.4 ms, m0 at 166.7 and the legacy front-end at 137.3.
- **Moving target (stretch, D17):** provari v0.2.0 hand / v0.1.0 hand =
  0.987 / 1.007 / 1.010, geomean **1.001** on 3 cells. At its default single
  thread v0.2.0 reproduces v0.1.0's node counts exactly (342,289 / 72,523 /
  4,574,912), so the ratio is wall noise around an unchanged trajectory;
  `--threads N` was not measured. Kept out of every goal geomean.

## History

Chronological entries — including every plan session that appended no row — are
in [`history.md`](history.md) (through plan18) and, for plans 19 onward, in
the [backlog table](initiative.md#backlog) of `initiative.md` and D32–D39 of
[`decisions.md`](decisions.md). Only ledger-structure changes are kept here:

- **2026-11-12** — D34 (item 31, [plan21](plan21.md)): the ledger states no
  goal, no rung and no cap policy; `Cap` is the budget a row was measured at;
  `scripts/check_goal.py` scans three normative documents instead of four.
- **2026-11-12** — reformatted for reading only, no measurement: the tables
  carry footnotes instead of inline annotations; the inherited sections are
  restructured into tables and lists (same content, examples shortened;
  `PROVENANCE.md` now marks them adapted); the dated distance log and the
  per-plan notes were dropped in favour of `initiative.md`, `history.md` and
  the reports.
- **2026-11-11** — D10 amendment: the legacy `solver@5f85133` rows left the
  table; `atomi` rows only.
- **2026-11-06** — file opened for the `prover` initiative: the inherited
  sections carry inline provenance comments and plan1 appended the first row.
