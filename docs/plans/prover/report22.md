# Report 22 — item 30: the archive does not fit

Plan: [`plan22.md`](plan22.md). Data:
[`measurements/plan22/`](measurements/plan22/README.md) (`env.json`,
`composition.json`, `phase_a.py`, README). Decisions: D14, D18, D21, D30,
D33, D34. Host: Ryzen 9 5950X, one process at a time. Binary:
`atomi@df9ba5a+d`, sha256 `3b1a3380…`.

## 0. Verdict

**Plan22 Phase A's stop rule fires.** The two permanent composition counters
show that the live **attacker** entries alone are **5,639,488** on the
control run at 384 MB — **1.99×** the entire 192 MB proof-table slot count
(2,831,155 at the 90 % bound), while defender entries are only **22,822**
(0.4 % of live). A memo tier that evicts only defender proofs therefore
cannot recover the space; MEM-V1 is insufficient.

Per plan22 I stopped **before Phase B**: no two-tier policy, no
`--proof-memo` flag, no default change, no Phase C gate, no D35, no ledger
row. Only the Phase A counters landed (they are declared permanent in the
plan) and the result is recorded here.

## 1. Phase A (`composition.json`)

One sequenced process, `deep-win`, `--guide hand`, seed 0, `--seconds 600`,
`--stats`, every written proof checked by both checkers:

| Run | Total MB | Share | Attacker live | Defender live | `proof_entries` | Export | Pos. | Len | Child evals | Wall | Checkers |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | --- |
| A1 | 384 | 25 % (default) | 5,639,488 | 22,822 | 5,662,310 | `store-full` | — | — | 189,477,824 | 158.6 s | — |
| A2 | 192 | 25 % (default) | 2,820,163 | 10,992 | 2,831,155 | `store-full` | — | — | 205,208,471 | 174.9 s | — |
| A3 | 192 | 50 % | 5,595,484 | 24,545 | 5,620,029 | `written` | 76,405 | 36 | 134,818,742 | 103.4 s | accept / accept |
| A4 | 192 | 75 % | 5,687,070 | 29,804 | 5,716,874 | `written` | 93,092 | 36 | 155,236,740 | 114.7 s | accept / accept |

The runs are deterministic on the tree: A2 reproduces report21's 205,208,471
`child_evals` exactly. `proof_entries` stops at 90 % of the table in A1/A2,
as designed (`refused` is still the export's fail-closed signal).

## 2. Stop rule A

| Quantity | Value |
| --- | ---: |
| A1 live attacker entries | 5,639,488 |
| 9/10 × 192 MB default-share slots | 2,831,155 |
| Ratio | **1.99×** |
| A1 truncated by refusal? | **yes** |
| **Fires?** | **yes** |

Plan22 assumed A1 would be the one run *not* truncated by refusal, making
its attacker count an upper-bound proxy. It is itself truncated: at 384 MB
the default 25 % split gives a 96 MiB proof table, and the 189.5 M-eval tree
needs more than its 5.66 M live slots. The proxy is therefore a lower bound
on the archive, and it is already twice the 192 MB slot count. The memo
tier's evictable pool (22.8 k slots, 0.4 %) is three orders of magnitude too
small. MEM-V1 would need to evict **attacker** proofs, which is exactly what
its archive side forbids.

## 3. The report21 384 MB row does not reproduce at the default split

report21 §3 reports 384 MB → `written`, 94,686 positions, length 35. At the
default 25 % share the same label now refuses (`store-full`), and the table
is full of attacker entries. The likely explanation is a different proof
share in that diagnostic (the report does not state one): A3/A4 show that at
192 MB with 50–75 % proof share the export succeeds and both checkers accept
(76,405 / 93,092 positions, length 36, 103.4 / 114.7 s). This is a
**single-seed probe, not a Phase C row**: it is recorded as an observation
for the next review, not as a claim, and no default or ledger row follows.

## 4. What landed

- `src/table/proofs.rs`: permanent `attacker_live`, `defender_live` and
  `archive_refused` state, maintained through insert, domination and refusal;
  accessors `attacker_live()`, `defender_live()`, `refused()`.
- `src/search/mod.rs` + `src/main.rs`: `proof_attacker_entries`,
  `proof_defender_entries`, `proof_archive_refused` in the outcome JSON
  (appended after `proof_full`); `SearchResult` carries the two live counts.
- `tests/search.rs`: the CLI contract checks the three new fields and their
  order. Table unit test `composition_counters` pins the class accounting.
- `measurements/plan22/`: committed Phase A driver, parsed results, env.

`ProofSource` and the writer are untouched; `refused()` still means
`store-full` when export is requested.

## 5. Gates

On the shipped binary (`atomi@df9ba5a+d`, sha256 `3b1a3380…`):

- `make test` — licence gate + 128 unit / 51 integration tests, green;
- `make lint` — goal check, fmt, clippy `-D warnings`, doc, green;
- `make test-full` — slow tier + `make veto`: **PASS** (8/8 accepted rows,
  four controls `valid: false`).

No proof-file semantics changed (counters only), but the table is a
proof-output input, so the full tier and veto were run rather than inferred.

## 6. Problems, open parts and next steps

1. **The blocker is attacker-proof volume, not defender proofs.** The table
   holds millions of attacker entries while the exported proof tree needs
   only 76–93 k positions; most live entries are stale proofs from branches
   that lost the final tree. Any lossy-table design must be allowed to evict
   those, or the writer must read the proof tree from a log instead.
2. **Option 1 (event-log export) is the natural next move.** Plan22's
   alternative formulation makes `store-full` structurally impossible and
   turns the table into a pure lossy memo. Phase A gives the free input:
   ~5.6 M attacker entries live at exit vs ~0.09 M needed by the export.
3. **A proof-share increase is a second candidate** (A3/A4: 192 MB exports
   at 103–115 s on seed 0), but it trades TT space and the tree (134.8–155.2 M
   vs 189.5 M child evals) and plan22 forbids per-cell re-tuning. If the next
   review wants it, it needs its own 3-seed gate, timed controls and D14;
   a single seed is not a claim.
4. **No test is missing for the stop-rule code.** The counters are exact
   (not capped like `unique_keys`); the composition test covers domination
   moving an entry between classes. There is no test for a two-tier policy
   because that code was deliberately not written.
5. **Next review:** take plan22 option 1 (event-log export) or authorise a
   proof-share stack with its own gate. Do not start item 29 ahead of this
   decision, do not raise the memory label, and do not rerun Phase B.

SESSION COMPLETE
