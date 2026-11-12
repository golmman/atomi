# Prover Plan 27: item 34 — adopt NULL-V2 under an owner override

Initiative: [`prover`](initiative.md), backlog **item 34**, decisions **D14**,
**D21**, **D30**, **D34**, **D37**, proposed **D38**, new **D39**. Drafted
2026-11-12 after [`report26`](report26.md).

## Kill review (fixed prompt)

1. **Distance.** `T_3`; `T_4` needs `deep-win` 83.5 s → 10 s (**8.35×**);
   `core` 6.54 s and `easy-win` 0.72 s fit. Item 34's measured stack cuts
   `deep-win` to 71.7 s at an identical tree (0.8631×), which is **7.17×** from
   the rung. The gain is real but does not move a rung.
2. **Would we build this?** The code is already built and measured. The
   question is whether to keep it: it removes per-node work, is exact, adds
   1.25 MB, and every cell improved. Yes — but not by retroactively lowering
   the failed gate.
3. **Cancel?** The per-node micro-line stays closed. No new per-node
   experiments; the next substantive decision is the formulation choice
   (PDS-PN/PN² or a goal re-pin under D30).
4. **Prior art (delta).** Pin unchanged → changed rows only: D37 adopted
   NULL-V1; D38 recorded G-N2's miss. No new mechanism row.
5. **Smallest next experiment.** Re-apply the stored patch and check whether
   its 5 M-eval tree matches the measured candidate. It does **not**:
   `DEFAULT_ENTRIES` is `1 << 21`, giving 168,885 expansions instead of the
   control's 168,910. The constant is a one-line regression; with `1 << 17`
   the probe reproduces 168,910 exactly.

## Decision

**Adopt NULL-V2 as an owner override (D39), with the gate record unchanged.**
G-N2 missed its declared `deep-win` ≤ 0.85× guard at 0.8631×; `plan26` and
`report26` remain the historical record of that miss. Adoption is justified by
the mechanism, not by a revised threshold: exact, tree-identical, all three
cells faster, 1.25 MB, and no correctness surface.

The shipped form is the **cleaned** implementation, not the stored patch:

1. **2^17 default** (`DEFAULT_ENTRIES = 1 << 17`) — the measured
   filter-slack size, tree-identical at 5 M evals.
2. **One size authority** — `Budget::cache_entries` is the allocated size;
   `Tables` allocates from it, so the splitter's accounting and the cache can
   never disagree.
3. **Tunable** — `--null-cache-entries N` (default 2^17, `0` disables) in the
   deterministic flag set. A request is a cap: the splitter clamps it to the
   label and rounds down to a power of two.
4. **Probe-only instrumentation removed** — `positive_null` and
   `null_positive_repeats` are gone; `null_cache_hits`/`misses` remain as
   observation-only counters.

## Scope

**In:** re-apply and clean the NULL-V2 patch; flag plumbing; the 5 M tree probe;
`make test`, `make lint`, `make test-full`; a Cap-120 s bench row with D14;
`plan27.md`, `report27.md`, `measurements/plan27/`, D39, item-34 status, the
ledger and history. **Out:** the gate text (not rewritten), the per-node
micro-line (stays closed), search base, tree levers, budgets, goal/rungs,
provari material.

## Design

- `src/table/null_cache.rs`: the direct-mapped `(key, rule50, attacker)` cache,
  both signs, full-key compare, `0` value = empty. `DEFAULT_ENTRIES = 2^17`.
- `src/table/mod.rs`: `split_share_with_cache(mb, pct, requested_entries)` clamps
  and rounds once, returns `Budget.cache_entries`; `Tables::with_share_and_cache`
  allocates exactly that. The cache is charged to the filter slack first, then
  the TT; at 2^17 it stays filter-only, so the TT and proof table keep their
  sizes and the tree is unchanged.
- `src/guide/null.rs`: EARLY-REPLY (pseudo-legal stream + lazy legality, two
  `StateInfo`s) and the cached `exposed_limit` wrapper.
- `src/cli.rs` / `src/search/mod.rs` / `src/main.rs`: `--null-cache-entries`
  through `ProveOptions → Limits → Tables`; the counter plumbing.

## Gates

- **P-27 tree identity:** at 5 M evals, `deep-win`, seed 0, default 2^17 must
  reproduce the plan26 control's 168,910 expansions and hit/miss counts;
  `--null-cache-entries 0` must also be tree-identical.
- **Correctness:** `make test`, `make lint`, `make test-full` + veto green.
- **Bench:** Cap 120 s, seeds 0–2, `--guide hand`, three cells; every proved
  cell D14-accepted by `atomi validate` and `provari check`.
- **Non-goal:** G-N2 is not re-run or re-scored. D39 records the override; a
  future gate is never opened by this plan.

**Proposed D39** records the override, the cleaned implementation and the
corrected patch. Stop rule: if P-27 or any correctness gate fails, revert and
leave item 34 closed negative.
