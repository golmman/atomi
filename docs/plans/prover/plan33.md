# Plan 33 — item 40 (NULL-PIPE): `null_win` pass 1 keys → prefetch → probe

Plan session, item 40 (re-scoped, §0.2), decisions D37/D39/D43/D44/D45.
Phase-0 numbers re-read from
[`measurements/plan32/counts.json`](../../../measurements/plan32/README.md) (no
new run). Oracle = external process (GPL). Read-only pass over
`src/guide/null.rs`, `src/table/{mod,proofs}.rs`, `src/search/mod.rs`.

## 0. Verdict first

1. **Worth 4–6 pp on `deep-win`, ceiling 9.4 pp** (§2) — not report32 §7's
   2–3 pp: that priced the *TT* half of the probe G1 removed (14 ns), but pass
   1's expensive access is the **144 MiB proof probe** that remains, 43.3 % of
   all lookups. Still only geomean 7.064 → ≈6.9, `deep-win` 56.7 → ≈53 s,
   `T_4` 5.67× → ≈5.3×. **No rung, no provari column moves.** D40 closed the
   per-node *micro*-line; D43/D44 reopened the guide's own closures — this
   class.
2. **Item 40 renumbered (done, D45).** Item 40 now reads NULL-PIPE; the
   upstream `movegen` ask is item 41, and the formulation fork is retired (§6c
   is therefore out of scope: no formulation change is proposable, D45).
3. **Run one bounded session (§5 stop rule).**

## 1. Phase 0 — the numbers the item needs (already measured)

`counts.json`, `deep-win` @120 s, `--guide hand` (`guide_null` = pass 1):

| Quantity | Value |
| --- | ---: |
| `null_calls` / `guide_null` lookups | 3,431,807 / 116,554,079 → **34.0 per call** |
| pass-1 proof hits | 14,789 (0.43 % of calls, 0.013 % of probes) |
| proof key present, unusable at the counter | **0** |
| proof-table load at exit | 5,716,874 / 9,437,184 = **60.6 %** |
| pass-1 share of the 269.1 M lookups | **43.3 %** |
| `lookup_with_source` self @120 s | **21.55 %** (12.7 s) |

Both load-bearing: the list is scanned to the end on **99.6 %** of calls, so the
two-phase rewrite pays ≈0 extra `do_move`/key work (§3); and `ProofTable::get`
walks the run to a FREE slot — ≈2.5 slots at 60.6 % load — so
`prefetch(start)` may not suffice (variant: two lines).

## 2. Gain estimate

Ceiling if pass-1 probes were free: 43.3 % of 21.55 % → **9.4 pp** (5.5 s of
58.2 s). Recoverable = the exposed part: today's not-taken early-return lets
the OOO window start the next `do_move` during a miss, so MLP is already ≈1–2;
a K = 8–16 batch gives 8–16 independent misses (bandwidth ≈130 MB/s, trivial).
**50–65 % recovered → 4.8–6.1 pp**: `deep-win` 56.7 → 53–54 s, ledger 7.064 →
≈6.92; report32's 2–3 pp is the low end.

## 3. The change (≈50 lines, one file + one accessor)

`null_win`'s pass 1 becomes chunked: per chunk of `K` moves **(A)** `do_move` →
`rules::status == Won` → `key`/`rule50` → `undo_move`, recording `(key, counter)`
and the first `Won` index; **(B)** `tables.prefetch_proved(key)` for the chunk,
then `tables.proved(key, counter)` in move order. First firing index wins, with
today's value: `Won` → `(mv, 1, 99)`; proof → `(mv, pe.length + 1, rule50 == 0 ?
99 : pe.limit.saturating_sub(1).min(99))`. Pass 2, `exposed_limit`, the null
cache, `null_board`, `work.replies` are untouched; `work.moves` keeps its line
(`= moves.len()`, before both passes).

New API `Tables::prefetch_proved(key)` → `ProofTable::prefetch`, the proof half
of `Tables::prefetch` (which also drags the TT). Stack only: `[(u64, u16); K]`
+ `Option<u32>`, K ∈ {8, 16}; no allocation. `K` has **no** correctness effect
(E2 holds for any K ≥ 1), so tuning needs no new exactness run. Files: `src/guide/null.rs`, `src/table/mod.rs` (+3), tests,
`gotchas.md`, `report33.md`, `measurements/plan33/`.

## 4. The exactness proof required (the item's condition)

**E1 — `work.moves`/the D25 branch** (gotchas: it is the full legal move count,
not the number scanned). It stays `= u64::from(moves.len())` before pass 1; the
pipeline must not count scanned moves nor touch `work.replies` in pass 1.
`Engine::null_replay` skips on `aux_budget − aux_nodes < work.moves`, so a
decrease is a trajectory change — pinned by G-40a/G-40b.

**E2 — witness identity.** Today pass 1 returns the *least index* `j` with
`won(child_j)` or `proved(key_j, c_j) = Some(pe) ∧ pe.length ≤ remaining`. Proof
reads are pure and pass 1 does **no store and no cache insert**, so probing `j`
after computing `j+1`'s key gives the same answer: the pipeline changes read
*timing*, never the *order of decisions*. The witness `(null, mv, length,
limit)` feeds `simulate_with_guide_proof` and the emitted proof — output-bearing,
not heuristic. `limit` uses the stored `rule50`, as today.

**E3 — work past a fire index is unobservable**: the `do_move`/`undo_move`
pairs and prefetches past `j` touch no counter, table, `line` or `sink`; the
scratch board is undone (pinned by `null::tests`) and the function returns.

**E4 — no shipped per-probe counter**: instrumentation stays in a scratch
build; it moves wall and breeds a gate on a non-tree quantity.

**E5 — pass 1 accepts both proof arms.** A null child has the defender to move,
so its proof is normally an AND proof (`mv == 0`); pass 1 reads only
`length`/`limit`, so report32's `from_bits(0)` trap cannot recur — publishing
`pe.mv` would be it.

Corner cases: `Won` at 0; `Won` at `j` with a usable proof at `j' < j` (**proof
wins**); non-firing proof (`length > remaining`) before a `Won`; fires at `K−1`,
`K`, `K+1`; short tail chunk; full scan (→ `None`); entry unusable at the
counter; `rule50 == 0` child.

## 5. Gates

| Gate | Bar |
| --- | --- |
| G-40a exactness | new `null.rs` test `pipelined_pass1_matches_reference` against the existing `null_win_reference`: the 8 `FIXTURES` children **plus** `crate::testutil::positions()` (≥ 5,000 seeded playouts, defender to move) — `NullWin`, `work.moves`, `work.replies` and the null key equal for K = 1/8/16; `make test` |
| G-40b tree identity | `child_evals`/`expansions`/`proof_entries`/`found`/`length` equal on 3 cells at 1 M, the 5 M `deep-win` probe and 9 × (cell, seed) at 120 s (G-39b) |
| G-40c wall | interleaved same-session, `--seed` = repeat; **binds on `deep-win` @120 s** (3 pairs, median ≤ 0.95×) **and** its 5 M probe (≤ 0.95×). `core`/`easy-win` informational — a 4.5 s cell cannot resolve 2 % (report32 §3/§6). K = 8 vs 16 breaks ties |
| G-40d | `make lint`, `make test-full`, `make veto`, D14 9/9 both checkers |
| G-40e | one `bench.md` Cap-120 row, expected geomean ≈6.9 |
| G-40f | **same bench session, no extra process**: one 120 s `deep-win` profile
splitting wall into `clone`+`do_move` / `rules::status` / `key`+`rule50` vs the
rest, in ns/expansion and pp of 7.064. Consumers: item 41 (bar ≥ ~2 pp, else the
ask is a non-item) and `prover`'s floor (sum the D43/report32 §7 residues) |
| **stop rule** | `deep-win` median ≥ 0.98× **or** G-40b not identical → ship nothing: close item 40 `no effect`, record the attribution (the probe overlapped better than §2 assumes) |

## 6. Options (≥1 changes the formulation)

- **(a) This plan.** Cheapest decisive experiment: prototype behind an env
  switch in a scratch copy, one interleaved 120 s `deep-win` pair; ≥ 0.99× stops
  it before G-40a.
- **(b) Reformulation within the base — the *expansion* as the batching unit.**
  One expansion issues three dependent probe bursts: frame pass 3 (1.7 % proof
  hit), null pass 1 (34 probes), `replay::simulate_defences` (per defence); only
  pass 3 prefetches. Same mechanism, bigger window, est. 6–8 pp. Cheapest
  experiment: after (a), apply it to `simulate_defences` alone.
- **(c) Retired** (D45): even perfect probe elimination leaves `deep-win` ≈46 s,
  so the remaining distance is implementation, not a formulation question.

## 7. Risks / not to re-propose

Prefetch is a hint (the `// SAFETY:` in `table::prefetch`, D20): no new
`unsafe`. Do not re-propose a proof-key bit filter (D44), a TT/proof shrink, a
scanned `work.moves` (D25), `lookup` at a proof-only site (D44), or any
formulation change (D45: PDS-PN/PN², TT move ordering, restarts, or a goal
re-pin — not proposable in `prover`, and §6c is excluded by that). The report
names both binary shas (report32 §5.5).
