# Report 32 — item 39 (PROOF-FILTER): the filter is closed `no effect`, G1 shipped at 0.986×

Plan: [`plan32.md`](plan32.md). Initiative: [`initiative.md`](initiative.md),
item 39, decisions **D39/D43**, proposed **D44**. Measurements:
[`measurements/plan32/`](../../../measurements/plan32/README.md). Engine
change: `src/table/mod.rs` (`Tables::proved` + one unit test),
`src/guide/{null,replay}.rs` (six proof-only guide lookups, one collapsed
duplicate). Nothing here came from provari source; the oracle was an external
process (GPL boundary).

## 1. Verdict

**The item's filter mechanism is closed `no effect` by measurement; its exact
residue (G1) is shipped.**

- The item's gate (**tree-identical ≤ 0.92× on the three 120 s cells**) is
  unreachable by a second negative-exact bit filter. The plan session measured
  it before a patch: net zero to negative at every size the 192 MB label can
  fund without changing the tree (256 KiB free slack 0.975–1.011×, 512 KiB
  0.989–1.004×, 2 MiB from the proof table 1.001–1.016×, 4 MiB 1.026–1.039×).
  The one positive row (2 MiB from the **TT**, 0.734×) is a different tree
  (118.2 M vs 155.2 M child evals) and is not the filter's gain.
- G1 — the six guide lookups that accept only `Value::Proved` probe the proof
  table instead of `Tables::lookup` — is exact, tree-identical everywhere
  measured and **0.986–0.987×** on the pooled same-session sample.

| Gate | Bar | Result |
| --- | --- | --- |
| G-39a exactness | `proved` ≡ `lookup`'s `Proved` arm + `make test` | **pass** — `table::tests::proved_matches_lookup_proved_arm`; `make test-full` green |
| G-39b tree identity | 3 cells at 1 M, 9 × (cell, seed) at 120 s | **pass** — 3/3 and 15/15 (`child_evals`, `expansions`, `proof_entries`; `found`/`length` equal) |
| G-39c wall (re-scoped) | median ≤ 0.99 `core`/`deep-win`, ≤ 1.00 `easy-win`, no pair > 1.02 | **n = 3 as written: fail on `core` (0.9930)**; pooled n = 12/12/6: **pass** (0.9859 / 0.9864 / 0.9871) |
| G-39d | `make lint`, `make test-full`, `make veto`, D14 9/9 | **pass** — VETO: 8 claim rows + 4 controls, both checkers agree on every row; D14 9/9 claimed |
| G-39e | one `bench.md` Cap-120 row | **pass** — geomean **7.064** (3/3 claimed, D14 3/3 accepted) |

## 2. What changed (≈40 lines, one commit-sized change)

- `Tables::proved(key, counter) -> Option<ProofEntry>` = `proofs.get`. It is
  `lookup`'s `Proved` arm because **proofs never live in the TT** and a
  terminal `Proved { length: 0 }` is never inserted — the property the unit
  differential pins over a randomised table (defender proofs with `mv == 0`,
  attacker proofs, dominating re-inserts that become no-ops, tombstoning
  inserts, unusable limits, and TT entries stored under the same keys).
- Six guide sites switched: `guide/null.rs` (the streamed `null_win` pass 1 and
  the reference pass 1), `guide/replay.rs` (`simulate_inner`'s key,
  `simulate_defences`'s key, its child key, and the guide's `gkey`). The
  "already proved" branch's **second lookup of the same key collapsed** into
  the first (`pe.mv`).
- `guide/replay.rs` gained `proof_move`: proof entry `mv == 0` is a **defender**
  proof, and `Move::from_bits(0)` decodes to `Some(Move::NONE)`, not `None`. The
  first draft used `from_bits` directly and would have turned a defender proof
  into a "winning" null move; caught in review, now a `gotchas.md` entry.
- `gotchas.md` +2 entries: the second-filter net-zero mechanism (do not
  re-propose at any label-funded size) and the proof-only-probe rule with the
  `mv == 0` corollary.
- No flag, no memory change, no allocation change; `--proof-out` unaffected.

## 3. Gate detail and the one deviation (G-39c)

Control = `git archive` of `69b6f50` built outside the repo
(`sha256 11a8db90…`; the archive has no `.git`, so `build.rs` bakes
`ATOMI_GIT_SHA=unknown` and the bytes differ from the plan-session control
`3b6b3f6b…` — same source tree, no G1). Candidate = `bf7bc4af…`. Interleaved,
one process at a time, `--seed` = repeat.

| Cell | n = 3 median (gate as written) | pooled median | mean | min–max | n |
| --- | ---: | ---: | ---: | --- | ---: |
| `core` | 0.9930 **fail** | 0.9859 | 0.9873 | 0.9738–1.0095 | 12 |
| `easy-win` | 0.9974 | 0.9864 | 0.9842 | 0.9567–1.0149 | 12 |
| `deep-win` | 0.9874 | 0.9871 | 0.9866 | 0.9791–0.9948 | 6 |

**The deviation, stated plainly:** at the 120 s cap `core` proves in 4.3–5.5 s
and `easy-win` in 0.4–1.4 s, so one pair carries ~1–2 % host noise against a
1.5–2 % effect; the three gate seeds gave `core` 1.0095 / 0.9905 / 0.9930
(median 0.9930, 0.3 pp over the bar) and nine further seeds 0.9738–1.0006.
Only `deep-win` (the full 120 s budget, or 36–58 s where the mate lands early)
has a low-noise ratio at n = 3. I extended the sample at **disjoint seeds**
and report both readings rather than rewriting the gate: **G-39c as written is
not met on `core`, is met on the pooled sample.** Shipping G1 rests on the
exactness (tree-identical 18/18, a proof-level argument) plus the pooled wall;
the owner decides whether the pooled sample satisfies the gate. Re-running the
gate itself would not help — 3 pairs on a 4.5 s cell cannot resolve 2 %.

Side observation, no action: `deep-win` seed 2 proves in 38.8 s (vs 54–58 s for
seeds 0/1, 36–49 s in the extension), so the `deep-win` median wall is
seed-dominated; the Cap-120 row's 56.7 s is a seed-0/1 property.

## 4. Bench row and D14

Cap-120 row: `core` 5.101 s, `easy-win` 0.377 s, `deep-win`
56.663 s against a same-session reference of 0.524 / 0.133 / 4.436 s → factors
**9.735 / 2.835 / 12.773**, geomean **7.064** (plan31: 7.23), so `T_4` distance
is **5.67×**. `child_evals` are byte-identical to plan31's row
(12,439,376 / 1,016,547 / 155,236,740), confirming the tree did not move.
`--proof-check`: 3/3 accepted by `atomi validate` **and** `provari check` with
matching `child_evals`; `d14.json` extended this to **9/9** over seeds 0–2.
`make veto`: 8/8 claim rows accepted by both checkers, 4/4 negative controls
agree (drop-root, illegal root, alt-move, rule50+1).

## 5. Problems met

1. **Clippy (1.99)** rejected the first drafts: two `match`/`let…else` forms
   needing `?` and `rng(..) % 3 == 0` needing `is_multiple_of`. Fixed; `make
   lint` green.
2. **`Move::from_bits(0) = Some(Move::NONE)`** — see §2, the one real trap in
   this change. A `proved`-based site must decode with a zero check.
3. **Unused imports** after the switch (`Value` left `null.rs`/`replay.rs`
   non-test code); moved into the `#[cfg(test)]` modules.
4. **`execute.py` crashed** on the first extension run (`bars` looked up
   `deep-win` in a filtered cell map) *after* the runs, losing that data; fixed
   and re-run. Recorded because it cost 9 pairs.
5. **A `target/release/atomi` rebuild changes the binary sha** (baked git sha),
   so every number here names its sha and the control's.

## 6. Missing tests / open parts

- **No unit test pins the six guide sites themselves**; their exactness rests
  on `proved ≡ lookup`'s `Proved` arm (unit) plus tree identity (integration,
  18/18). A direct differential (replay/null with `proved` vs `lookup`) would
  be economical with a test-only `Tables` switch; not built, because the tree
  identity over 18 runs already covers every guide site on both cells that
  reach them.
- **`core`'s ratio is unresolved at n = 3** by construction (§3). A cheap
  future protocol fix — not part of this item — would fix `--evals` instead of
  wall for the short cells and gate the wall only where the cap binds.
- The `deep-win` seed dependence (§3) is unexamined; seeds 0–2 differ by 33 %.

## 7. Next steps (options, ≥1 changes the formulation)

1. **Null-reply probe pipelining** (plan32 §6b; reformulation: hide the
   latency instead of removing the access). Pass 1 of `null_win` becomes
   keys → prefetch → probe; 137.7 M guide probes × 14 ns ⇒ est. **2–3 pp**, no
   memory. Needs a `work.moves`/witness exactness proof and its own tree
   identity gate.
2. **Fewer lookups** (plan32 §6c): 90.6 % of 269 M lookups find nothing, and
   116.5 M are null-replay replies probed ~34 per null call for 14,789 hits.
   Est. up to **11 pp**, but any exact cheaper test is table-dependent, so it
   needs a witness-preserving argument first — a plan, not a patch.
3. **Formulation fork (D30) unchanged**: even a perfect probe elimination
   (21.55 % self time at 120 s) leaves `deep-win` at ≈46 s against the 10 s
   rung. The next real decision is the formulation choice (PDS-PN/PN² or a goal
   re-pin), not another micro-plan.
