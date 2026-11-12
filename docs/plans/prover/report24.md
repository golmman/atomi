# Report 24 — item 29: GROUP-V1 fails its gate, the group stays off

Plan: [`plan24.md`](plan24.md). Data:
[`measurements/plan24/`](measurements/plan24/README.md) (`env.json`,
`phase_a.json`, `gate.json`, `gate_summary.json`, `d14.json`, driver,
README). Decisions: D14, D21, D30, D33, D34, new **D36**. Host: Ryzen 9
5950X (4 vCPU container), one process at a time.

## 0. Verdict

**GROUP-V1 fails.** Score-free plain members plus `group` on by default lose
`deep-win` outright: **0/3 at 120 s** against a **3/3** same-session control
(`--guide-off group`, median 105.032 s). `core` is 1.045× (inside the 1.1
guard), `easy-win` 0.863× (better). The gate required `deep-win` ≤ 0.9× and
every candidate seed proved, so **item 29 is closed negative**: no default
change, the score-free code was reverted (Phase D), the plain-move group stays
the opt-in `--guide-group` kill-test flag it was after item 15. The engine
stays at **`T_3`** (the reverted default reproduces report23's trajectory
counter-for-counter). The permanent Phase A counters stay. `T_4` distance is
unchanged: `deep-win` 105 s → 10 s (10.5×), `core` 8.5 s → 10 s (already
inside), `easy-win` 0.9 s.

## 1. Phase A — diagnosis (pre-change binary `31432bd7…`)

Two arms, three cells, seeds 0–2, 120 s, `--stats`. The `default` arm (group
off) reproduces report23 exactly (`core` 10.244 / 8.535 / 8.228 s; the
counters are observation-only). The `group_scored` arm is item 15's
behaviour with today's algebra (members scored).

| Cell | `plain_children` / `child_evals` | `group_children` / `expansions` | `child_evals`/exp | µs/exp |
| --- | ---: | ---: | ---: | ---: |
| `core` | 37.9–38.2 % (4.4 M of 11.6 M) | 37.8–37.9 % | 29.4–29.9 | 22.8–23.0 |
| `easy-win` | 44.4–47.3 % (0.34–0.44 M) | 39.9–41.6 % | 27.5–28.4 | 21.0–22.4 |
| `deep-win` | 43.8–44.0 % (44.5–76.0 M) | 38.4–43.5 % | 27.6–27.9 | 19.2–20.0 |

Two conclusions, both load-bearing for the next plan. (1) **The lever is
large**: two of five children in the whole search are attacker plain moves, so
if grouping helped the tree at all it would show here. (2) **It does not help
the tree**: `evals`/exp is flat (26.6–30.1) across every arm, so the group is
purely a trajectory change, and the scored group already loses `deep-win`
(seed 0 censored at 120 s / 172.5 M evals; seeds 1–2 95.2 / 71.2 s vs the
control's 105.9 / 78.1 s, a wash). Grouping at ~40 % of nodes does not
increase the search's direction — it only resequences the plain moves into one
synthetic child whose start `(9, #plain)` is a per-node constant.

## 2. Phase B — the score-free change

`Frame::enter` pass 3 skips `guide.start_numbers` for a fresh child flagged
plain, leaving the identity `(1, 1)`; `GuideConfig::product()` (the base of
`hand()`) sets `group: true`. The virtual-group invariants were untouched
(member keys, proof mapping, pending retention): the integration group tests
and the new `plain_children`/`group_children` assertions passed, and the
pending-proof unit test needed only a widened key lookup — a settled plain
child's key now lives on a group member, and the group slot borrows that key
with its own adjusted value (found and fixed during execution, §5).

`make test` (128 unit + 51 integration) green; `deep-win` proved with the
group on for 1–3 seeds depending on the run (the score-free tree is roughly
the scored one).

## 3. Phase C — the gate (`gate.json`, `gate_summary.json`)

Shipped-binary arms, `--memory 192`, 120 s, seeds 0–2, same session. Control =
`--guide hand --guide-off group` (the pre-change default); candidate =
`--guide hand` (group on, score-free).

| Cell | Control walls | Median | Candidate walls | Median | Ratio | Gate |
| --- | --- | ---: | --- | ---: | ---: | --- |
| `core` | 10.229 / 8.480 / 8.364 | 8.480 s | 8.860 / 8.234 / 11.128 | 8.860 s | 1.045 | pass (≤ 1.1) |
| `easy-win` | 0.822 / 0.936 / 1.471 | 0.936 s | 0.832 / 0.700 / 0.808 | 0.808 s | 0.863 | pass |
| `deep-win` | 114.323 / 105.032 / 77.498 | 105.032 s | **timeout × 3** (120 s) | — | 1.143 (censored) | **fail** |

The candidate `deep-win` runs censored at 155.1 / 155.5 / 155.0 M
`child_evals`; the control proved at 155.2 / 146.7 / 107.7 M. So the censored
runs spent *more* work than the control needed (tree factor at censoring
1.057) — the score-free group is not "slower per node" (evals/exp 27.6–28.2
vs 26.8–27.9, ns/exp 21.3–21.8 µs vs 19.5–20.1 µs, both within host noise),
it simply does not find the defender refutations in the same order. Both
directions of the ablation therefore agree: scored members (`phase_a`
`group_scored`, `deep-win` 1/3) and score-free members (0/3) are worse than
the group-off control (3/3).

D14 on the candidate's six proved seeds (`d14.json`): every 600 s rerun
matched `child_evals` and both checkers accepted — 6/6 `accepted`, proofs
2,223–2,423 positions (`easy-win`, lengths 19–22) and 14,500–15,351
(`core`, lengths 14–18). The score-free arm is **sound**; it is only slower on
the coverage cell. Note the mate lengths differ from the control's
(`easy-win` 19/19/16 → 22/21/19, `core` 15/26/20 → 14/18/15): a group child
is a longer-mate search over plain moves, which is legal but not free.

## 4. Phase D — revert

`Frame::enter` and `GuideConfig::product()` are back to their pre-plan-24
text (`group: false`; scored fresh pairs), the `group` field doc now records
the negative result and the reason. The reverted binary is byte-identical to
the Phase A binary (`31432bd7…`), and `--guide hand` on `core` seed 0
reproduces 12,439,376 `child_evals` / 424,249 expansions / length 15 — the
control's exact row. Kept: the permanent counters `plain_children` and
`group_children` (in `--stats`), their integration assertions, and the widened
pending-proof unit test (it passes in both configurations and documents that
the group slot borrows a member key). `T_3` is unaffected.

## 5. Gates

- `make test` — licence gate + 128 unit / 51 integration tests, green on the
  final tree;
- `make lint` — goal check, fmt, clippy `-D warnings`, doc, green;
- `make test-full` — slow tier + `make veto`: **PASS** (8/8 accepted rows,
  four controls `valid: false`), run on the final tree (search/guide touched);
- every claimed win in `d14.json` is accepted by `atomi validate` **and**
  `provari check`; no claim is made for `deep-win` (0/3);
- licence gate unchanged, no provari material added.

## 6. Problems and open parts

1. **Driver bug (found and fixed before reporting).** `gate_summary` first
   summarised the control from a median over *both* arms (18 rows), which
   printed `core` 8.67 s and `deep-win` 117.16 s. Fixed to summarise
   `group_off` and `default` separately, plus a `gate_summary` phase that
   re-derives the arithmetic from the committed `gate.json` without rerunning
   anything. No row was affected.
2. **Unit-test contract change.** With the group on, a settled plain child's
   key is on a group member, and the group slot reuses a member's key with the
   group's pre-adjusted refutation (`from` +1 for a zeroing member). The
   pending-proof test now uses the member lookup for member keys and skips the
   group slot. This is the "group is virtual" invariant restated, not a new
   exception.
3. **`easy-win` improves 13.7 % and `core` is 4.5 % worse** under grouping.
   Item 29 is closed as a product lever because of `deep-win`; the small-cell
   numbers are recorded as observations only (no per-cell switch is
   authorised, and per-cell configuration is tuning on the bench set).
4. **Phase A measures the cut but not the cause.** The counters size the
   plain share (38–47 %) but not why sequencing plain moves together hurts
   `deep-win`. If the group is ever re-proposed it needs a mechanism-level
   claim (why one synthetic OR child should be better than the scored
   per-move order), not another default flip.
5. **Pre-existing gap, not caused here:** `initiative.md`/`bench.md` link
   `measurements/plan22/` and `measurements/plan23/`; neither directory is in
   the committed tree. Owner to decide (commit or re-link).
6. **Next is item 33 (per-node cost)**, as proposed D36: the wall is
   11.5–15.5× per expansion and tree levers are exhausted for this line.

## 7. Next steps

1. Item 33 plan: profile the per-child work (`board.clone` + `do_move`,
   movegen, `Mover::effect`, settle screen, `hand_eval`, sim replay) on
   `deep-win` at 20 M evals; the Phase A instrument says `child_evals`/exp and
   µs/exp are flat across arms, so the cut must come from a per-child term.
2. Keep the Phase A counters in every future `--stats` row; they are the only
   measure of how much of the tree the guide's children represent.

SESSION COMPLETE
