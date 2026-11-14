# Movegen Plan 3: adopt `atomic-movegen` 2.8.0

Initiative: [`movegen`](initiative.md), **D-MG1**–**D-MG7**. Handoff answered:
[`handoff3.md`](handoff3.md) (MG19 `PostMove::terminal()`, MG20 a specialised
existential legality test). Release: **2.8.0**, published 2026-11-13T22:45:11Z,
**MIT**, MSRV 1.85, `.crate` sha256 `ac68cf10…f2006b` (index match), 232,852
bytes. Baseline: locked **2.7.0** (`cd69dc00…`). Drafted 2026-11-13 (S2, S4).

## 1. What 2.8.0 contains (verified from the tarball)

No `Fixed` entry in the 2.8.0 section (both belong to older releases);
`src/zobrist.rs` is **byte-identical to 2.7.0 and 2.4.0** (`337af88d…`), so
MG11's key schema stands. Changed: `effects.rs`, `movegen.rs`,
`position.rs` (new `terminal_cheap`/`terminal_tail`, one precedence shared with
the board path), `board.rs`.

**MG outcomes: 2 of 2 landed as API, 0 of 2 met the cost bar** (`report3.md` §3,
D-MG7): MG19 `PostMove::terminal()`, `post_move_and_key() -> (PostMove, u64)`,
`PostMove::rule50()` — met without the key, missed with it (1.24× built-ish on
the crate's corpus); MG20 a positive screen (`has_legal_move_after` 36 →
**25–30 ns**) against a **≤ 12 ns** bar — missed.

The screen also serves `has_legal_move`/`_with_state` (6 → 2–3 ns), which
`Board::outcome()` — hence `rules::status` ([`src/rules.rs`](../../../src/rules.rs)
line 48) — calls: the first pin with a plausible wall effect (small, probably
flat).

## 2. Scope

A **pin plus a regression check**, as plan1 §2 / plan2 §2. **Out of scope:** any
call-site change (`mover.rs`, `classify.rs`, `search/`, `guide/`);
`unsafe-optimizations`, new asks. And, unlike plan2, **the stack**: handoff3 §3
gates a prover plan on the bars, §3/§8 show they are missed, so **no `prover`
plan is written from this release**.

## 3. Pre-plan probe (the gates re-run it formally)

`/tmp` only: two atomi copies path-dependent on 2.7.0 / 2.8.0 (D-MG4).

- **Crate harness green:** **173 unit + 11 integration binaries (51 cases, incl.
  `perft_all_positions_depth_4`) + 16 doctests**, nothing ignored (2.7.0: 163 +
  10 / 45 + 12).
- **atomi compiles unchanged** (`cargo check --release --all-targets`), suite
  **195 passed / 5 ignored** with the frozen differential (report2: 193).
- **Tree and wall**, `deep-win`, 5 M evals, seed 0, `--guide hand --memory 192`,
  3 alternating repeats: `child_evals`/expansions/`aux_nodes`/
  `proof_entries` **identical** both sides (5,000,034 / 168,910 / 1,563,978 /
  215,267); medians 2587 vs 2551 ms → `m1/m0` **0.986**, inside the ±2–3 %
  spread: flat.
- **The handoff3 bar on atomi's corpus** (93 non-terminal fixture positions,
  1,663 children, 459 built-ish, best of 7 whole-corpus passes, ns per child):

| Path | all | built-ish |
| --- | ---: | ---: |
| today: `clone`+`do_move`+`outcome()`+`Legal` key | 44.4 | 48.3 |
| build only (`clone`+`do_move`) | 23.8 | 27.4 |
| `post_move_and_key`+`terminal()` | 42.0 | 46.9 (**0.97×**) |
| `post_move`+`terminal()` (no key) | 39.5 | 44.4 (0.92×) |
| `terminal_after` | 39.2 | 43.8 |
| `has_legal_move_after` (MG20) | 38.3 | 42.6 (**3.5× over the bar**) |

  Our corpus is tactical (handoff3's play-outs: 57.5 ns built-ish), so levels
  differ, the direction does not: the bar wants ≈0.84× **with** the key and
  ≤ 12 ns; neither is reached.

## 4. Steps (gates 1–5 → steps 1–3, 4, 5, 6)

| # | Step | Output |
| --- | --- | --- |
| 1 | Re-fetch, verify sha256 vs the index, extract, run the crate harness | `env.json` |
| 2 | **Bump** `"2.7"` → `"2.8"` + lock; frozen differential unchanged (any diff = veto); `make test`, `make lint` | pin |
| 3 | **Extend** `tests/child_surface_differential.rs`: `post_move(..).terminal()` (reason included) and `post_move_and_key`'s key vs our built child over the existing 51,334-child corpus; and, the only rules-adjacent change, **`has_legal_move` / `_with_state` / `_after` vs `!generate_legal(..).is_empty()`** — the screen against the generator, not itself. Test only | differential |
| 4 | `make test-full`, `make veto`, counters unmoved at fixed evals | gate 3 |
| 5 | m0 vs m1 (10 s + 120 s × 3 cells × 3 repeats), `make pernode`, JSONs + `env.json` under `measurements/movegen/plan3/`, dated `prover/bench.md` row | gate 4 / S4 |
| 6 | Keep/revert line, ledger, history, `report3.md` | decision |

**Commits** (owner): (1) `deps: atomic-movegen 2.8.0` — `Cargo.toml` +
`Cargo.lock` only; (2) `test(movegen): pin the 2.8.0 screen and child summary`;
(3) docs and measurements. Step 3 is the judgement call: the frozen fixture
cannot guard the screen, which could turn a `Legal` into a `Stalemate`.

## 5. Gates (verbatim, D-MG6)

Every integration plan contains these, in this order (D-MG6):

1. **Pin and lock** — bump `Cargo.toml` + `Cargo.lock`; run the crate's harness
   as published (tests + perft).
2. **Golden rules differential** — freeze and diff the crate's observable
   behaviour: perft to depth 4–5, full legal move lists *in order*, `outcome`,
   `is_terminal`, `hash`/`hash_with`, `rule50`/`game_ply` after every legal
   move, on a fixed corpus. Any difference that is not an announced rules fix
   is a **veto**, not a note. atomi has no perft test today, so the first
   integration plan **adds this differential**; MG13 would replace the frozen
   data with the crate's own corpus.
3. **Downstream gates** — `make test-full` and `make veto` (an independent
   rules implementation).
4. **Measure** — same-session bench at the standard cap, a 10 s row, and
   per-expansion cost; three repeats, alternating builds, one process at a
   time; parsed results only, under `measurements/<plan>/` with `env.json`.
5. **Keep or revert with data** — one line: version, gate result, ratios,
   reason. A regression, a divergence or a missing harness is a revert.

## 6. Branch table

| Finding | Action |
| --- | --- |
| Frozen diff empty, extended differential green, gates green | **Keep** |
| Any frozen diff, red gate, counters moved, harness missing, slower beyond the spread | **Veto, revert**; flat is a keep |
| The screen disagrees with `generate_legal` on one child | **Revert**, re-file — a hidden legal move is a false win |

## 7. Options, gain, cheapest experiment

- **A (this plan).** Pin + differential. Gain is **API, not a rung**.
- **B (formulation).** `post_move`+`terminal()` where atomi builds a child only
  for terminality. Declined: with the key the composite is 0.97× ours / 1.24×
  the crate's, and the hand stack needs the child for the guide (handoff3 §3:
  ~0 %, D42). Re-open only from a prover formulation.
- **C (stay on 2.7.0).** Rejected: S2; the caret drifts on `cargo update`.

**Cheapest decisive experiment (run, §3):** the 5 M `deep-win` probe, tree
identical at 0.986, plus the corpus table.

## 8. The handoff3 §3 gate, re-read

It authorises a prover integration only against a pair "MG19 + MG20 … median
wall ≤ 0.95× control". The pair cannot deliver: with the key it is 0.97× where
0.84× is needed, and MG20's 12 ns bar is missed 3.5× over. **The gate is failed,
not superseded** — this plan pins the version and writes **no prover plan**. A
further crate ask needs a new `handoff4.md` (D-MG7) *and* a prover formulation
that does not read the child board.

## 9. Acceptance and open questions

**Done** when: pin = `Cargo.toml` + `Cargo.lock` only; frozen differential
byte-identical; step 3 green, trajectory-neutral; gates 3–4 green;
`measurements/movegen/plan3/` holds `env.json`, bench, pernode and README; a
`prover/bench.md` row; `report3.md` carries the outcome, challenges and
keep/revert line; ledger and history updated.

Open at execution:

1. **Does the screen move atomi's wall?** `rules::status` → `outcome()` →
   `has_legal_move_with_state` is on the expansion path; flat per probe, the
   pernode point answers. A win there is the first rung-relevant datum since
   plan1 — the `Terminal` precedence refactor is covered by gates 2–3.
2. **Screen trust boundary.** Step 3 pins it against our generator; the view's
   extinction guard keeps a stale `StateInfo` conservative — atomi must not
   start passing stale ones.
