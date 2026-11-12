# Report 26 — item 34: NULL-V2 misses G-N2; the per-node micro-line closes

Plan: [`plan26.md`](plan26.md). Data:
[`measurements/plan26/`](measurements/plan26/README.md) (`env.json`,
`phase_a.json`, `gate_n2.json`, `gate.py`, `null_v2.patch`, README).
Decisions: D11, D14, D18, D21, D24, D26, D30, D34, D36, D37, proposed
**D38**. Host: Ryzen 9 5950X (4 vCPU container), one process at a time.

## 0. Verdict

**G-N2 fails on the `deep-win` wall guard: 0.8631× the same-session control
(median 71.666 s vs 83.029 s) against the declared ≤ 0.85×.** The tree is
identical 9/9 (`child_evals`/`expansions`), `core` is 0.8843× and
`easy-win` 0.8631×, both inside the ≤ 1.1× small-cell guard. D14 was not
run: the wall guard failed first, so no claim is made. Per the plan's stop
rule the **per-node micro-line is closed**; the next plan is a formulation
decision (PDS-PN/PN² or a goal re-pin), not another micro-plan. The NULL-V2
source was reverted after the gate; the complete diff is
[`measurements/plan26/null_v2.patch`](measurements/plan26/null_v2.patch).
The engine stays at **`T_3`**; `T_4` distance is unchanged at `deep-win`
**8.35×** (83.5 s → 10 s).

## 1. Phase A — cache sizing

The probe was reproduced on the shipped binary at 5 M child evals,
`deep-win`, seed 0. The cache is taken from the filter first (tree-neutral:
the filter is exact in the negative direction), then from the TT. The
control is the pre-plan binary (`atomi@939d890`, sha256 `01f1d6ce…`).

| Entries | Cache | Hits | Misses | Hit rate | Wall | RSS | Tree vs control |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| control | — | — | — | — | 3.015 s | 217.3 MB | — |
| 2^17 | 1.25 MB | 1,014,999 | 2,364,622 | 30.0 % | **2.557 s** | 217.9 MB | **identical** |
| 2^18 | 2.50 MB | 1,065,092 | 2,314,529 | 31.5 % | 2.655 s | 218.2 MB | differs (168,909 exp) |
| 2^20 | 10.0 MB | 1,166,804 | 2,212,858 | 34.5 % | 2.699 s | 218.5 MB | differs (105,121 null calls) |
| 2^21 | 20.0 MB | 1,200,836 | 2,178,341 | 35.5 % | 2.801 s | 218.0 MB | differs (168,885 exp) |

2^17 is the largest size that fits the filter slack, so the TT and proof
table keep their sizes and the tree is identical to the control. Larger
caches take TT bytes and change the trajectory; they are not eligible for
G-N2. The 2^17 cache is the shipped candidate.

**Positive-null ceiling.** 11,651 of 105,120 `null_win` calls (11.1 %) are
repeats of a previously positive null position. A positive-null cache was
probed and **excluded**: it changed the tree (`null_calls` 104,137 vs
105,120) because it freezes a table-dependent result. The plan's ≥ 5 % wall
test was therefore not reached; the mechanism is not in the stack.

## 2. Phase B — NULL-V2

`src/guide/null.rs`:

- **EARLY-REPLY.** `exposed_limit` now walks the pseudo-legal reply list and
  filters lazily with `Board::is_legal_with_state`, stopping at the first
  legal reply that is not accepted. The result is order-independent (all
  accepted vs one rejected; `limit` is a max over accepted replies), so the
  tree is unchanged. Two `StateInfo`s are used: one for the child's legality
  predicate, one for the scratch make/unmake (the probe's shared-state
  attempt diverged; the differential pins this).
- **EXPOSED-CACHE.** A direct-mapped cache keyed by `(child key, rule50,
  attacker)`, storing both positive and negative answers. A hit requires a
  full key and full meta compare; a collision recomputes. The cache is
  board-only, so both signs are exact. It is 2^17 entries × 10 bytes
  (`u64` key + `u8` meta + `u8` value), inside the 192 MB label, taken from
  the filter slack first and then the TT. The proof table keeps its size.
- **`--stats`.** `null_cache_hits`/`null_cache_misses` and the
  observation-only `null_positive_repeats` counter.

Tests: `exposed_limit` equals the clone-based reference on every legal child
of the fixture corpus; the cache matches an uncached call on a repeated
position; the split invariant includes the cache bytes; a filter-only cache
keeps the TT and proof sizes. At `--evals 1 M` the candidate matches the
control on `child_evals`, `expansions`, `sim_nodes`, `settle_nodes` and
`aux_nodes` on all three cells (`null_replies` is lower by design).

## 3. Phase C — G-N2

Control = pre-plan binary (`atomi@939d890`, sha256 `01f1d6ce…`); candidate =
NULL-V2 build (sha256 `0e0f1fb2…`). `--guide hand --memory 192 --seconds
120`, seeds 0–2, control and candidate back to back.

| Cell | Control median | Candidate median | Ratio | Gate |
| --- | ---: | ---: | ---: | --- |
| `core` | 6.723 s | 5.945 s | 0.8843 | pass (≤ 1.1) |
| `easy-win` | 0.716 s | 0.618 s | 0.8631 | pass (≤ 1.1) |
| `deep-win` | 83.029 s | 71.666 s | **0.8631** | **fail (≤ 0.85)** |

`same_tree` 9/9. The cache hit rate at 120 s is 33.5 % on `deep-win`
(33,393,855 hits / 66,428,143 misses, seed 0). **G-N2 fails** on the
`deep-win` threshold by 1.3 percentage points. The gate was declared before
the data and is not re-run to improve it.

## 4. Gates

- `make test` — licence gate + 131 unit / 51 integration tests, green;
- `make lint` — goal check, fmt, clippy `-D warnings`, doc, green;
- `make test-full` — slow tier + `make veto`: **PASS** (8/8 accepted rows,
  four controls `valid: false`) on the reverted tree;
- no claim is made, so no D14 proof is required; the timed gate JSON records
  `proof_skipped: true`;
- licence gate unchanged, no provari material added.

## 5. Problems and open parts

1. **The gate is a near miss, not a mechanism failure.** The stack cuts
   `deep-win` 13.7 % at identical tree; the declared 15 % threshold was not
   met. The plan's stop rule is explicit, so the stack is not adopted. The
   measured 0.8631× is recorded; a future plan that wants the gain needs a
   new gate, not a re-run of this one.
2. **The tree-neutral cache size is capped by the filter slack.** 2^17 is
   the largest cache that leaves the TT and proof table unchanged. Larger
   caches have a better hit rate (35.5 % at 2^21) but change the tree, so
   they cannot satisfy G-N2's identity clause. A 2-way cache at 2^17 entries
   is the obvious next probe if the line is ever reopened; it is not in this
   plan.
3. **The positive-null cache is table-dependent.** It has an 11.1 % repeat
   ceiling but changes the tree, so it is excluded. The plan's risk note was
   correct.
4. **The code is reverted.** The working tree is the pre-plan tree; the
   patch is the deliverable. The owner can re-apply it if a new gate
   authorises the gain.
5. **D14 was not run.** The wall guard failed first; running 600 s proofs
   would not change the verdict. The report makes no claim.

## 6. Next steps

1. Close item 34 and the per-node micro-line (proposed D38).
2. The next kill review takes the formulation decision: PDS-PN/PN² or a goal
   re-pin under D30. Do not write another per-node micro-plan.
3. If the owner wants the measured 13.7 % `deep-win` gain, the cheapest
   experiment is a 2-way 2^17 cache under a new gate; it is a new plan, not
   a continuation of this one.

SESSION COMPLETE
