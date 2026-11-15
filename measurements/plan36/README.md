# Plan 36 measurements — item 45 (M-REMAP): `M` in like units

Plan: [`plan36.md`](../../docs/plans/prover/plan36.md) (kill review of plans
34–35 + item 45). Report: [`report36.md`](../../docs/plans/prover/report36.md).
Host, pins, binary shas, input shas: [`env.json`](env.json).

Two artifacts, one per phase, and **no engine, flag, default, test or
`bench.md` row changes**:

| File | Contents |
| --- | --- |
| `length_probe.py` | Phase 0, read-only: 3 `atomi` seeds per cell (deep-win @120 s, core and easy-win @10 s) and 1 reference run per cell, one process at a time. Writes `length_probe.json` |
| `length_probe.json` | the two engines' proof lengths per cell, our expansions/walls, the reference's `mate`/`nodes`/`ms`, the pre-registered check, and the plan's own unit check marked as *not reproduced* |
| `remap.py` | Phase A, arithmetic only (no run, no clock): maps the reference's 39 counted families and our 21 census buckets into one unit each, prices them under conventions A/B/C, corrects report35's like-unit row table, and checks gates G-36a–d. Writes `remap.json` |
| `remap.json` | every family and bucket with its unit, the A row table with a `refuted` flag and a like-unit column, the A/B/C charges and `M`s, the revised addressable pp, the verdict and the gates |

`remap.py` is ~28 KB (over the 10 KB source guideline): the family mapping, the
three conventions and the four gates are one argument, and the input it audits
(`shape.json`) is fixed, so it is deliberately one artifact rather than three.

## Reproduce

    python3 measurements/plan36/length_probe.py    # ≈ 3 min, 7 + 3 runs
    python3 measurements/plan36/remap.py           # instant, no run

The reference is a separate process (`provari prove`); only its own JSON result
line (integers) is stored. Nothing from the GPL clone enters the repo, and no
`ns` that prices anything is the reference's — every rate in `remap.json` is
ours (report34 §4 verbatim).

## Phase 0 in one table (this session, seed 0–2)

| Cell | our `length` | reference `mate` | our wall (median) | reference wall |
| --- | --- | ---: | ---: | ---: |
| `deep-win` @120 s | 36 / 37 / 29 | 19 | 49.045 s | 4.363 s |
| `core` @10 s | 15 / 26 / 20 | 17 | 3.987 s | 0.490 s |
| `easy-win` @10 s | 19 / 19 / 16 | 15 | 0.393 s | 0.107 s |

`deep-win` never comes near the plan's ≤ 20 bar, so the 2.76× tree ratio is
void (item 47). The plan's unit check ("`core` 17 = our 17") does **not**
reproduce: our committed `core` proof is 15 (plan33's Cap-120 row has the same
15), i.e. we prove `core` *shorter* than the reference, `easy-win` and
`deep-win` longer.

## Phase A in one table (`deep-win`)

| Convention | `M` (ns/exp) | vs the 1,723 ns bar |
| --- | ---: | --- |
| A — report35 verbatim | 1,785.88 | +62.7 (NOT FUNDED) |
| B — per-op, excess-only | 1,678.60 | −44.6 (inside) |
| C — per-op, region-total | 2,200.66 | +477.5 |
| B's own band (B0/B1/B2) | 1,678.60 / 1,643.52 / 1,559.71 | all inside |

Verdict: **INCONCLUSIVE-BY-UNITS** — `min` (1,678.60) ≤ 1,723 < `max`
(2,200.66) for the three pre-registered conventions, and the band straddles
the bar under every reading of the one judgement call B needs. The like-unit
correction of report35's row table drops the addressable share from **51.01 pp
to 44.17 pp** (−6.84 pp): the reference does our per-reply legality work under
`heavy_legality_tests` (33.07/body against our 34.08/exp).
