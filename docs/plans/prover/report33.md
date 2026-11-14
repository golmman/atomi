# Report 33 — item 40 (NULL-PIPE): the pipelined `null_win` pass 1, 0.926× on `deep-win`

Plan: [`plan33.md`](plan33.md). Initiative: [`initiative.md`](initiative.md),
item 40, decisions D37/D39/D43/D44/D45, proposed **D46**. Measurements:
[`measurements/plan33/`](../../../measurements/plan33/README.md). Engine
change: `src/guide/null.rs` (pass 1 chunked; `null_win` keeps its signature,
the body moves into a `const K`-generic `null_win_pipelined`),
`src/table/mod.rs` (`Tables::prefetch_proved`, +7 lines), and the two new
differential tests. Nothing here came from provari source; the oracle was an
external process (GPL boundary).

## 1. Verdict

**The item is adopted: exact, tree-identical and 0.926× on `deep-win` @120 s**
(`core` 0.9257×, `easy-win` 0.9593× informational), i.e. **7.4 pp of the
plan's 9.4 pp ceiling**, above its 4.8–6.1 pp projection. The Cap-120 row is
geomean **6.606** (plan32 7.064), `deep-win` **53.42 s**, so `T_4` distance is
**5.34×** (plan32 5.67×). No rung moves, no provari column moves: the goal is
still `T_3` at the ledger's economic cap, and the remaining distance is
implementation (D45), now known to be outside pass 1's probe.

| Gate | Bar | Result |
| --- | --- | --- |
| G-40a exactness | `pipelined_pass1_matches_reference` vs the existing reference + `make test` | **pass** — K = 1/8/16 over the 8 `FIXTURES` boards and their children plus every seeded playout position with the defender to move, empty and seeded proof table, `remaining` = MAX and 1; plus `pass1_chunk_boundaries_match_the_reference` |
| G-40b tree identity | 3 cells at 1 M, the 5 M probe, 9 × (cell, seed) at 120 s | **pass** — 3/3, 1/1 (5,000,034 / 168,910 / 215,267, the plan32 value), 9/9 |
| G-40c wall | `deep-win` @120 s median ≤ 0.95× (3 pairs) **and** the 5 M probe ≤ 0.95× | **pass** — **0.9260** (0.9257 / 0.9260 / 0.9334) and **0.9227** (0.9124 / 0.9227 / 0.9556) |
| G-40d | `make lint`, `make test-full`, `make veto`, D14 9/9 | **pass** — VETO 8/8 claims + 4/4 controls agree; D14 9/9 both checkers |
| G-40e | one `bench.md` Cap-120 row | **pass** — geomean **6.606**, 3/3 claimed |
| G-40f | one 120 s profile of pass 1's phases | **pass** — pass 1 = **13.2 pp** (see §5) |
| stop rule | median ≥ 0.98× or G-40b not identical | **not triggered** |

## 2. What changed (≈120 lines incl. tests)

- `null_win`'s pass 1 became `pass1::<K>`: per chunk of `K = 16`, **(A)** key
  every move (make → `rules::status` → `key`/`rule50` → unmake, stopping at the
  first `Won`), **(B)** `tables.prefetch_proved(key)` over the chunk, then
  `tables.proved(key, counter)` in move order; the first firing index returns
  today's value (`Won` → `(mv, 1, 99)`; proof → `(mv, len+1, rule50 == 0 ? 99 :
  limit-1)`, read from the recorded counter). Pass 2, the screen, the null
  cache, `null_board`, `null_win_reference` and `work` are untouched;
  `work.moves` is still set to the full legal count before pass 1.
- `Tables::prefetch_proved(key)` — the proof half of `Tables::prefetch` (which
  also drags the TT cluster), so a proof-only site prefetches one line, not two
  sets.
- Signature preserved: `null_win` forwards to `null_win_pipelined::<PASS1_CHUNK>`;
  the private generic is what tests instantiate at K = 1 (the byte-serial order),
  8 and 16 (no test-only duplicate of the search path).
- The test module moved to `src/guide/null/tests.rs` (`#[cfg(test)] mod tests;`),
  a pure move done **after** the gates: `null.rs` was 35 KB with them and is
  11.5 KB without, against the repo's ~10 KB convention. The release binary is
  byte-identical before and after the split (`110ffc23…`), so every number here
  was measured on the shipped bytes.
- No flag, no memory or allocation change, `--proof-out` untouched (tree
  identical with and without it, as before).

## 3. The argument, as built (E1–E5)

**E2** is the load-bearing one and it is structural, not measured: proof reads
are pure and pass 1 does **no store and no cache insert**, so probing index `j`
after computing `j+1`'s key returns what the byte-serial loop returned — the
*pipeline moves when a proof is read, never which index wins*. `K` therefore has
no correctness effect for any `K >= 1`, which is why the new differential can
pin all three K values against one reference. **E1** (`work.moves` = the full
legal count, before pass 1, D25) and `work.replies` (pass 2 only) are unchanged
by construction and compared per call in the test. **E3** (work past the firing
index) is exercised by the boundary test: a proof on child `i` for *every* `i`
of four boards, so a fire at `K-1`/`K`/`K+1`/the tail leaves the same answer as
a fire at 0, and the scratch board is always unmade. **E5** holds trivially:
pass 1 reads only `pe.length`/`pe.limit`, never `pe.mv`, so report32's
`from_bits(0)` trap cannot recur here.

Corner cases covered by `pass1_chunk_boundaries_match_the_reference`:
`Won` at a low index with a usable proof below it (the **proof** wins), a
non-firing proof (`length > remaining`) before a `Won`, an entry exactly usable
at the counter (`limit == counter`) and one unusable (`limit = counter-1`),
`rule50 == 0` children, short tail chunks (a 4-move null list at `K = 16`) and
a full scan (`None`).

## 4. `K` and the prefetch (the mechanism is the batch, not the hint)

Same session, 5 M `deep-win`, n = 4, every side interleaved with one control
run (`tune_5m.json`, all tree-identical):

| Side | median ratio |
| --- | ---: |
| K = 8 | 0.940 |
| **K = 16 (shipped)** | **0.931** |
| K = 32 | 0.921 |
| K = 8, `prefetch_proved` removed | 0.939 |

The gain is the *separation*: an early-issued probe already overlaps the loop
(plan32's asymmetry), while keying a whole chunk first gives the probes an
independent burst. Prefetch adds nothing measurable at K = 8 (0.940 vs 0.939) —
kept because it is four instructions, never negative in any run here, and the
plan's design point; recorded as *not* the mechanism. K = 16 vs 32 is inside
noise, so the smaller stack array ships.

## 5. Problems met

1. **The first version of the differential test failed and was right to.** The
   null cache lives inside `Tables`, so whichever side of a differential runs
   second answers from the cache the first one filled: `work.replies` 0 vs 25
   while the witness matched. Fix: both new tests build `Tables` with the cache
   disabled (`with_share_and_cache(.., 0)`), which makes every call start from
   the same state. This is a `gotchas.md` entry — it will bite the next
   differential that compares per-call work.
2. **`crate::testutil::positions()` cannot supply "≥ 5,000 positions with the
   defender to move"**: only the alternate plies qualify (4,271 of the run's
   total). The test asserts the *total* is ≥ 5,000 and the corpus takes every
   qualifying position, rather than inventing fills.
3. **The plan's ceiling arithmetic (9.4 pp) was attribution-based** (`lookup`'s
   self time × pass 1's share of lookups) and undercounts inlined cost: the
   instrumented profile puts pass 1 at 13.2 pp *after* the pipeline, so before
   it pass 1 was ≈20 pp. The measured saving (7.4 pp) is still inside the 9.4 pp
   ceiling — but as a share of a larger base, not as a share of the work. Worth
   remembering the next time a plan multiplies a self-time percentage by a call
   share.
4. **rdtsc instrumentation inflates** the 120 s run by 1.6 s (3.24 ns per read
   over ~500 M reads). The driver subtracts that from every interval and keeps
   the raw number beside it; the phase *shares* are the deliverable, not the
   absolute seconds.

## 6. Bench row and D14

Cap-120 row: `core` 4.923 s, `easy-win` 0.376 s, `deep-win` 53.420 s against a
same-session reference of 0.534 / 0.142 / 4.523 s → factors **9.219 / 2.648 /
11.811**, geomean **6.606**. `child_evals` are the plan31/plan32 values
(12,439,376 / 1,016,547 / 155,236,740), i.e. the row's delta is the pass-1
pipeline alone. `--proof-check`: 3/3 accepted by `atomi validate` **and**
`provari check` with matching `child_evals` (proof positions 16,991 / 2,980 /
93,092); `d14.json` extends this to **9/9** over seeds 0–2. `make veto`: 8/8
claim rows accepted by both checkers, 4/4 negative controls agree.

## 7. Missing tests / open parts

- **No test pins `Tables::prefetch_proved` ≡ the proof half of `prefetch`.** The
  prefetch is a hint (D20 `_mm_prefetch`, `// SAFETY:`); a wrong address cannot
  change a result, only a cache state, so there is nothing to assert beyond what
  G-40b already shows (a wrong-but-harmless prefetch is unobservable). Stated,
  not built.
- **The `prefetch` line is measured only at K = 8** (§4). A K = 16 no-prefetch
  side was not built; if the prefetch is ever suspected of hurting, that is the
  missing row.
- **`exposed_limit` (pass 2) is not instrumented.** G-40f splits pass 1 only;
  pass 2 is inside "the rest" (86.8 pp) together with the frame, the guide and
  the proof table. The next attribution step, if wanted, is `Frame::enter` vs
  `lookup_with_source` vs pass 2, not another pass-1 split.

## 8. Next steps (options, ≥ 1 changes the formulation)

1. **Item 41's bar is now measured on our side** (§5 of the measurements
   README: `rules::status` 6.10 pp, `do_move`+`undo` 4.35 pp — both inside the
   child build/terminality question the upstream ask is about). The `movegen`
   session can draft `handoff4` against those numbers; no `prover` plan follows
   from it until a crate release clears the ≥ 2 pp bar.
2. **The remaining attribution inside an expansion** (plan33 §6b, not built):
   pass 3's frame lookups, `simulate_defences` and pass 2 issue three dependent
   probe bursts per expansion of which only pass 3 prefetches. Same mechanism as
   this plan, bigger window; est. 2–4 pp at *this* measurement's shares (the
   probe is 2.10 pp in pass 1 — the ceilings are now much smaller than the
   plan-class estimates of a year of reports).
3. **Kill-review input:** after plan 33 the `deep-win` wall is 53.4 s against a
   4.5 s reference, and pass 1 — the largest single guide site — is 13.2 pp of
   it. Summing the residue this initiative keeps naming (`Frame::enter` 8.4 %,
   `lookup` 21.6 % at 120 s in plan32's profile, reply streams, the built-child
   question) does not close a 11.8× factor; that is the number the next kill
   review should look at, not another micro-plan.
