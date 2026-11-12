# Report 25 — item 33: NULL-V1 passes, NULL-S fails its small-cell guard

Plan: [`plan25.md`](plan25.md). Data:
[`measurements/plan25/`](measurements/plan25/README.md) (`env.json`,
`phase_a.json`, `gate_v1.json`, `gate_s.json`, `bench.json`, drivers,
README). Decisions:
D11, D14, D18, D21, D24, D26, D30, D34, D36, new **D37** (proposed). Host:
Ryzen 9 5950X (4 vCPU container), one process at a time.

## 0. Verdict

**NULL-V1 passes G-V1 and is adopted; NULL-S fails G-S and stays opt-in.**
At 120 s, seeds 0–2, against the same-session pre-plan control, NULL-V1 cuts
`deep-win` to **0.796×** (median 83.498 s vs 104.876 s), `core` to 0.784× and
`easy-win` to 0.766×, with `child_evals`/`expansions` **identical on all nine
pairs** and 9/9 D14 proofs accepted. NULL-S (`--guide-null-screen`) cuts
`deep-win` further to 0.571× but grows `easy-win` 5.7–11.5× in evals and
**3.349× in wall**, so it fails the ≤ 1.1 small-cell guard; it remains the
opt-in heuristic arm. The engine stays at **`T_3`**; `T_4` distance is now
`deep-win` **8.35×** (83.5 s → 10 s), with `core` 6.54 s and `easy-win`
0.72 s already inside 10 s.

## 1. Phase A — attribution

Two permanent `--stats` counters were added: `null_calls` (`null_win`
invocations) and `null_replies` (defender replies examined by the exposed
limit). They are observation-only and enter no budget. `deep-win`, seeds
0–2, medians:

| Cap | Wall | `child_evals` | Expansions | µs/exp | aux/exp | Plain share | `null_calls` | `null_replies` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 M evals | 11.939 s | 20,000,007 | 689,907 | 17.21 | 8.84 | 0 (group off) | 417,458 | 15,561,197 |
| 10 s | 10.010 s | 16,737,580 | 573,440 | 17.46 | 8.89 | 0 (group off) | 351,315 | 13,088,991 |
| 20 M, `--guide-group` | 11.319 s | 20,000,020 | 718,054 | 15.99 | 9.74 | **42.8 %** | 408,682 | 13,170,858 |

The null replay is the first-order per-node term: ~37 replies per call and
`aux_nodes` ≈ 8.8–9.7 per expansion. The pre-change control at 20 M seed 0
measured 21.96 µs/exp against the candidate's 17.33 µs/exp at the same
`child_evals`/`expansions`, a ~0.79× per-node cut before any wall gate.

## 2. Phase B — NULL-V1

`src/guide/null.rs`:

- **PASS2-BITS.** `exposed_limit` now accepts a reply iff the defender
  self-exploded, or (after the 50-move check, a missing attacker commoner and
  the material-draw rule) `threat::immediate_explosion_bits(next, attacker)`
  wins. The old `rules::status` + `immediate_explosion` pair built a child and
  a legal move list per reply.
- **SCRATCH-BOARD.** `null_win` passes 1–2 and `exposed_limit` reuse one
  cloned scratch board with `do_move`/`undo_move`; the clone is per call, not
  per move/reply. `undo_move` does not restore cached attack fields, but every
  query populates its own `StateInfo`, so no stale cache is read.
- **DEFENDER-EVAL.** `Guide::context` computes `hand_eval` only at attacker
  nodes, under `efpn_banded`, or under the opt-in symmetric `ef_pn = Both`;
  `Frame::enter` computes `child_eval` only where `start_numbers` reads it.
  The shipped `ef_pn = OrOnly` never reads a defender eval.

`--guide-null-screen` (NULL-S) is a new opt-in switch: pass 2 runs only when
`threat::signature_for(&null, attacker)` is non-empty; pass 1 is untouched.

Tests: the closure matches a clone-based reference on every legal child of an
eight-FEN fixture corpus (including the three bench cells, a material draw and
a clock position); the scratch `null_win` matches the clone-per-move reference
in `(mv, length, limit, work)`; `undo_move` restores FEN, key, clock, side and
both commoner bitboards; the null-board invariants still pass. `--evals 1 M`
on all three cells is counter-identical to the control (including
`sim_nodes`/`settle_nodes`/`aux_nodes`).

## 3. Phase C — gates

Control = pre-plan binary built from `atomi@a257e6f` (sha256 `e3e9fc25…`);
candidate = shipped tree (sha256 `144ef4a4…`). `--guide hand --memory 192
--seconds 120`, seeds 0–2, control and candidate back to back.

**G-V1 (NULL-V1, screen off):**

| Cell | Control median | Candidate median | Ratio | Gate |
| --- | ---: | ---: | ---: | --- |
| `core` | 8.350 s | 6.543 s | 0.784 | pass (≤ 1.1) |
| `easy-win` | 0.938 s | 0.718 s | 0.766 | pass (≤ 1.1) |
| `deep-win` | 104.876 s | 83.498 s | **0.796** | pass (≤ 0.80) |

`same_tree` 9/9; D14 9/9 accepted (`core` 13.8–17.0 k positions, lengths
15/26/20; `easy-win` 2.8–4.1 k, 19/19/16; `deep-win` 60.7–93.1 k, 36/37/29).
**G-V1 passes.**

**G-S (NULL-V1 + `--guide-null-screen`):**

| Cell | Control median | Candidate median | Ratio | Tree vs control | Gate |
| --- | ---: | ---: | ---: | --- | --- |
| `core` | 8.946 s | 6.162 s | 0.689 | 1.32–1.55× evals | pass (≤ 1.1) |
| `easy-win` | 0.940 s | 3.148 s | **3.349** | 5.7–11.5× evals | **fail** |
| `deep-win` | 104.775 s | 59.801 s | **0.571** | 1.46–1.76× evals | pass (≤ 0.67) |

D14 9/9 accepted. The screen is a real `deep-win` lever but a small-cell
disaster; it stays opt-in, as the plan's rule requires.

**Bench row (Cap 120 s).** A separate same-session ledger run measured the
provari reference and the candidate: `core` 0.533 / 7.824 s, `easy-win`
0.132 / 0.607 s, `deep-win` 4.367 / 88.330 s, factors 14.679 / 4.598 /
20.227, geomean **11.093** over 3 claimed cells. The D14 evidence is
`gate_v1.json` (same binary and flags, 9/9 accepted).

## 4. Gates

- `make test` — licence gate + 131 unit / 51 integration tests, green;
- `make lint` — goal check, fmt, clippy `-D warnings`, doc, green;
- `make test-full` — slow tier + `make veto`: **PASS** (8/8 accepted rows,
  four controls `valid: false`);
- every claimed win in `gate_v1.json` is accepted by `atomi validate` **and**
  `provari check`; no claim is made for the failed NULL-S arm;
- licence gate unchanged, no provari material added.

## 5. Problems and open parts

1. **G-V1 is close to its threshold** (0.796 vs 0.80). The 20 M probe
   (0.79×) and the 120 s gate agree, so the effect is real, but a re-run on a
   noisy host could land on either side. The gate was declared before the
   data and is not re-run to improve it.
2. **NULL-S's tree growth is not a small-cell effect only.** `deep-win` also
   grows 1.46–1.76× in evals; it wins on wall because the screen removes
   pass-2 work. The `easy-win` failure is the same mechanism at a different
   balance, not a separate bug.
3. **The residual per-node gap is still large.** NULL-V1 removes ~20 % of
   the per-expansion cost; the null replay's pass-1 status/movegen and the
   remaining per-child work are the next candidates. The plan's NULL-MEMO
   fallback (cache positive `NullWin`/`exposed_limit` answers) is the
   cheapest next probe if the screen is revisited.
4. **`plain_share` is zero without `--guide-group`.** The Phase A group arm
   is the only way to read it; the default arm records 0 by construction.
5. **Pre-existing gap, not caused here:** `initiative.md`/`bench.md` link
   `measurements/plan22/` and `measurements/plan23/`; plan24's directory is
   also absent from the committed tree. Owner to decide (commit or re-link).

## 6. Next steps

1. Adopt NULL-V1 (done in this tree); keep `--guide-null-screen` opt-in.
2. Next per-node probe: NULL-MEMO (positive `NullWin`/`exposed_limit` cache)
   or the remaining pass-1 null work; measure `null_calls` repeats first.
3. `T_4` needs `deep-win` 8.35×; the tree is at parity, so the next plan
   should stay on per-expansion cost, not tree levers.

SESSION COMPLETE
