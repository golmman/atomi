# Plan 32 — item 39 (PROOF-FILTER): the filter is measured net-zero; the exact residue is PROOF-PROBE-TRIM

Plan session, item 39, decisions D39/D43. Measurements:
[`measurements/plan32/`](../../../measurements/plan32/README.md). The oracle
stays an external process (GPL boundary). No engine change this session.

## 0. Verdict, first

The item's gate (**tree-identical ≤ 0.92× on the three 120 s cells**) is **not
reachable by a second negative-exact bit filter over proof-table keys**. Measured
same-session at 120 s, the filter is **net zero to negative** at every size that
can be funded without changing the tree (256 KiB free slack: 0.975–1.011×;
512 KiB: 0.989–1.004×; 2 MiB from the proof table: 1.001–1.016×; 4 MiB:
1.026–1.039×), because a filter probe costs about what the probe it skips costs (§2).
The one tree-identical, exact subset that does pay is **G1: the guide sites only
ever accept `Value::Proved`, so they need no TT probe at all** — medians 0.978
`deep-win`, 0.984 `core`, 0.977 `easy-win`, tree-identical 8/8. Plan 32
therefore executes G1 and closes item 39's filter mechanism as `no effect`; the
re-scope (gate 0.92 → ~0.98) is a **scope change for owner sign-off**.

## 1. Phase 0 — what the probe measured (the item requires this before the plan)

Scratch counter build (`--guide hand`, `deep-win`, `measurements/plan32/`);
counters are per *lookup*, so a proof hit never reaches the TT:

| 5 M evals (168,910 exp) | lookups | proof hit | proof miss, **TT hit** | proof miss, TT miss |
| --- | ---: | ---: | ---: | ---: |
| frame child (pass 3) | 602,926 | 142,323 (23.6 %) | **242,406 (40.2 %)** | 218,197 |
| guide null replay | 4,135,356 | 652 | 171,613 | 3,963,091 |
| guide replay | 646,377 | 122,214 (18.9 %) | 33,057 | 491,106 |
| total | 5,384,659 | 265,189 (4.9 %) | **447,076 (8.3 %)** | 4,672,394 (86.8 %) |

At the gate's scale (120 s, 5,803,257 exp, 269,098,870 lookups): proof hit
**7,560,290 (2.8 %)**, TT-only **17,847,542 (6.6 %)**, nothing at all
243,691,038 (90.6 %); frame-child proof hit rate 1.7 %.

So the item's premise holds and then some: **the 144 MiB proof table is probed
for a child that only has a TT entry 8.3 % of lookups at 5 M and 6.6 % at
120 s, and finds nothing at all in 86.8 % / 90.6 %.** Proof-table keys present
but unusable at the counter: **0**.

Why this matters: at 192 MB the existing "TT or proof" filter is **2^22 bits
(512 KiB)** and drops permanently above 1/4 fill; on `deep-win` @120 s it drops
at **store 1,539,730 of 17,355,189 (8.9 %)**, so only **12.4 M of 269.1 M
lookups (4.6 %) are gated** — and `lookup_with_source` is the largest single
self-time term: **21.55 %** at 120 s (16.61 % at 5 M, plan31). Label arithmetic:
proofs 144 MiB + filter 512 KiB + null cache 1.25 MiB + TT 46 MiB = 191.75 MiB
→ **free slack 256 KiB** (`2^21` bits), which cannot hold 5.72 M proof keys.

## 2. Why a second filter cannot pay here

A filter probe is a **dependent** random access: its result selects the next
access and a branch, so it serializes. The probe it skips is issued
speculatively and overlaps the loop body. Measured marginal value of a skipped
probe: G1 removes 137.7 M of 538 M probes for 1.9 s of 58.2 s ≈ **14 ns per
probe**. A 256 KiB–4 MiB filter on this host (5950X: 512 KiB L2/core, 32 MiB
L3/CCX) costs 12–18 ns per probe and carries a 10 %→34 % false-positive rate at
5.7 M keys. Net: ≈0. The existing filter pays only because it replaces **two**
probes and fits in L2 while it lives.

To reach 0.92× at `deep-win` @120 s, 4.7 s must go ≈ 336 M skipped probes (62 %
of all probes); the filter that could do that (≥2 MiB) spends what it saves.
**Ceiling of item 39 as specified: ~1.00×. Below the gate → stop the mechanism.**

Measured matrix (interleaved, same session; control `deep-win` median 58,236 ms):

| Variant (on top of G1) | deep-win @120 s | core @120 s | easy-win @120 s | tree |
| --- | --- | --- | --- | --- |
| G1 alone | **0.9822 / 0.9743** | **0.9765 / 0.9839 / 0.9866** | 0.9766 / 0.9664 / 1.0132 | identical 8/8 |
| + 256 KiB proof filter (free slack) | 0.9749 / 0.9772 | 0.9935 / 1.0096 / 0.9962 | 0.9896 / 0.9897 / 1.0106 | identical |
| + 2 MiB from the proof table | 1.0009 / 1.0162 | — | — | identical |
| + 2 MiB from the **TT** | 0.7341 / 0.7424 | — | — | **no**: 118.2 M vs 155.2 M child_evals, 4.23 M vs 5.80 M exp |

The TT-funded row is a **different tree**, so its wall is not the filter's and
the item's tree-identity gate forbids it; §6 excludes it rather than banking it.

## 3. The deliverable: G1 `PROOF-PROBE-TRIM`

`Tables::proved(key, counter) -> Option<ProofEntry>` (no TT probe) replaces
`Tables::lookup` at the six guide lookups that only accept `Value::Proved`
(7 today): `guide/null.rs` 2 (the null replay's pass 1, the streamed pass);
`guide/replay.rs` 4 — `simulate_inner`'s key and `gkey`, `simulate_defences`'s
key and child key — and the "already proved" branch's second `lookup` of the same key
collapses into the first (`pe.mv`). Order, counters, `work.moves` and every
witness line stay byte-identical.

**Exactness argument (a test, not prose):** the old expression matched
`Value::Proved` only; `proofs.get` returns exactly the proof table's answer, and
`Proved { length: 0 }` is never inserted, so `proved` ≡ the matched arm for every
`(key, counter)`; `mv != 0` ≡ `Some(mv)`. Pinned by a unit test comparing
`proved` against `lookup`'s `Proved` arm over a randomised table (defender
proofs, dominating inserts, unusable limits) plus the D14/veto path.

## 4. Gates

| Gate | Bar |
| --- | --- |
| G-39a exactness | the `proved` ≡ `lookup`-`Proved` differential above (unit) + `make test` |
| G-39b tree identity | `child_evals`/`expansions`/`proof_entries` equal on 3 cells at 1 M and 9 × (cell, seed) at 120 s |
| G-39c wall (**re-scoped**) | 3 interleaved repeats/cell at 120 s: median ≤ 0.99 `deep-win` and `core`, ≤ 1.00 `easy-win`, no (cell, seed) > 1.02. Expected medians 0.977–0.984 (§1). **The item's ≤ 0.92× is declared unmet by measurement, not by a missed run** |
| G-39d | `make lint`, `make test-full`, `make veto`, D14 9/9 both checkers |
| G-39e | one `bench.md` Cap-120 row (rows are never rewritten); geomean 7.23 → expect ≈7.1 |
| stop rule | if G-39c fails, ship nothing; close item 39 `no effect` citing §1–§2 |

## 5. Files and order

`src/table/mod.rs` (+`proved`, +unit test), `src/guide/null.rs`,
`src/guide/replay.rs`, `gotchas.md` (+2 entries from §2), `report32.md`,
`measurements/plan32/phase0.py`. One commit-sized change; ≈40 lines.

## 6. Options (≥1 changes the formulation)

- **(a) G1, this plan.** 2–2.5 % at 120 s, free, exact, tree-identical. Cost: one
  session. Cheapest experiment: done (§1).
- **(b) Null-reply probe pipelining** (reformulation: hide the latency instead
  of removing the access). Restructure `null_win`'s pass 1 into
  keys-then-prefetch-then-probe: 137.7 M guide probes × 14 ns ⇒ est. **2–3 pp**,
  no memory. Needs a `work.moves`/witness exactness proof (gotchas: the early
  exit must not change the budget) and its own tree-identity gate.
- **(c) Fewer lookups** (formulation change): 90.6 % of the 269 M lookups find
  nothing; 116.5 M are null-replay replies probed ~34 per null call to find
  14,789 hits. Any exact cheaper test is table-dependent, so this needs a
  witness-preserving argument first — a plan, not a patch. Est. up to **11 pp**.
- **(d) Excluded by measurement:** label-funded 2^21–2^25 proof filters (0.0
  net); TT-funded filters (tree change; the 0.734× is the tree's); raising the
  existing filter's drop threshold (2^22 bits saturate at ~6 M keys).
- **(e) Formulation fork (D30) unchanged:** even a perfect probe elimination
  (21.55 %) leaves `deep-win` at ≈46 s against the 10 s rung.

## 7. Not to re-propose

A proof-key bit filter at any label-funded size (net-zero, §2); `lookup` at a
guide site that only accepts `Proved`; a TT or proof-table shrink to buy cache
(tree/retention change); `may_hold` as the proof gate (drops at 9 % of the
search; a proof-only plane halves the index space).
