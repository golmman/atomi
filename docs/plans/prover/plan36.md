# Plan 36 — kill review (plans 34–35) and item 45 (proposed, M-REMAP): `M` in like units

Read only: [`report34`](report34.md)/[`report35`](report35.md) and their
[`measurements`](measurements/plan34/README.md),
[`analysis1`](analysis1.md) §3, [`analysis2`](analysis2.md) §5–§6,
[`gotchas`](gotchas.md) §Scope, D30/D36/D40/D43–D46, [`bench.md`](bench.md)
cells. Pins: oracle `48686bc` (provari 0.1.0, `/tmp/atomi-oracle/provari`),
`atomic-movegen` 2.8.0.

## Kill review (fixed prompt)

### 1. Absolute distance to the goal

`T_3`. One session, one process at a time, same box, same flags
(`--guide hand --memory 192`, `--seconds 120`/10): `core` 8.33 s, `easy-win`
0.91 s, `deep-win` **53.254 s**, 5,803,257 expansions, 155,236,740 child evals,
**length 36**. `T_4` = all three at 10 s ⇒ `deep-win` needs **5.33×**; `T_5`
geomean < 1.0 is further out.

The per-node term is measured and stable: **9,246 ns/exp** against provari's
**2,111 ns/body** = **4.38×**, and it cannot close `T_4` on any reading
(report35: `M` = 1,786 ns must fall to 1,723). The tree term was quoted as
2.76× (5,803,257 / 2,099,100) and is **not like-for-like** — §5 / Phase 0:
the two engines prove **different mating distances** on `deep-win`. So per-node
is short by 3.6 % on its best day, and the tree term nobody can price may be
the larger hole.

### 2. If we started today, would we build this?

**No** to the reshape (plan34 B / item 44 as scoped): `M` ≥ 1,723 under every
convention we can defend, on an incomparable tree. **Yes** to two measurement
items (hours, no engine change): M-REMAP below and OVERHEAD-PROFILE
([`plan37`](plan37.md)).

### 3. Which initiative or plan would you cancel?

- **Cancel**: item 44 as a *shape* port (plan34 B's v3 rewrite) — its price was
  computed against a mate-19 tree while ours is mate-36.
- **Deprecate**: **D36**'s "tree at parity" clause and analysis2 §6's
  decomposition (1.18 × 13.5) — both divide by provari's reported `nodes`,
  which is 1.44× its node bodies here.
- **Keep**: item 41 (external ask), 45/46, D45's ban on the formulation fork.

### 4. Prior-art delta (analysis1 §3; pin unchanged ⇒ changed rows only)

R11–R13 **adopted** (item 28; analysis2 C2–C4 closed), R9 **closed negative**
(item 29), R23/R24 keep — the per-node axis has **no unported mechanism left**.
Two rows come into focus: **R25** (length-limited passes / `--shorten`, "skip
for now") is the only row on *proof length*, **R17** the only memory row. No
full re-read (pin unchanged).

### 5. Smallest next experiment

Both are hours: the **length probe** (Phase 0, read-only, 3 runs) and
**M-REMAP** (Phases A–B, arithmetic on counters we already own — zero runs,
zero code), which answers "is `M` really 1,786 ns" with no new risk.

## Item 45 — M-REMAP

**Why.** report35's `M` prices the reference's per-child test volume at one
rate and folds the rest "inside". Two rows are not in like units:

- `legality/status/terminality` — ours 45.86 calls/exp against theirs **0.49**
  calls/body, quoted as 93× and 9.39 pp addressable. Ours are per *moving
  piece* inside our null streams (pass 1 20.20, `enter` 6.97, replay 3.67,
  `legality@null_p2` 13.88); theirs are per *body*. Their per-reply legality is
  almost certainly in `heavy_legality_tests` = **33.1/body**, which is within
  3 % of our own per-reply legality volume (20.20 + 13.88 = **34.08**).
- the per-child test families — `exposing` 21.7, `blast` 19.1, `check` 15.8,
  `threat` 4.4, `quiet` 32.6, `evasion` 0.71 per body (5.1 light tests per
  child + 1.8 heavy) are counted but never priced: 59.2 "screens"/body at our
  cheapest screen rate (3.66 ns) absorbs all of them.

**Decision (pre-registered).** Report `M` under three conventions and take the
verdict on the **lowest** (most favourable to FUNDED):

- **A — report35 verbatim** (inclusive child decision, tests inside):
  `M` = 1,786 ns. Reference.
- **B — per-op, excess-only.** For each test family charge
  `max(0, theirs_per_body − ours_per_exp)` at our *cheapest* matching rate
  (`mover_check` 3.66, `guide_class` 5.27, `repetition` 2.58,
  `legality@replay` 22.9, `status` 19.13), never pricing a family twice.
- **C — per-op, region-total.** As B, but first reconcile the region totals
  (their child-decide region vs our `closure_frame` + `mover_check` +
  `status`/`legality` + `key`), so that a family their loop folds into the
  decide is charged once at the region's own rate.

Verdicts: **FUNDED** ⇔ `min(A,B,C)` ≤ 1,723 ns; **NOT FUNDED** ⇔
`min(A,B,C)` > 1,723 (report the shortfall and the band
[`min`, `max`]); **INCONCLUSIVE-BY-UNITS** ⇔ `min` ≤ 1,723 < `max` — then only
the item 44 spike can resolve it, at the owner's discretion.

**Phase 0 — length probe (read-only, ≤ 5 min).** Three `atomi prove
deep-win --guide hand --memory 192 --seconds 120` seeds 0–2 (length, wall,
expansions; report23 recorded 36/37/29 at 25 % proof share) and one provari
`--seconds 120 --memory 192` (measured this session: `mate` 19, 3,024,117
reported nodes, 4,412 ms; `core` `mate` 17 = our `length` 17 ⇒ unit check
passes). Pre-registered: if our `length` is never ≤ 20, the 2.76× ratio is void
and the length axis becomes item 47 (SHORT-WIN) at the owner's discretion —
**not** an item in this plan.

**Phase A — the mapping audit (no runs).** Inputs are committed integers:
`measurements/plan35/shape.json` (39 reference families) and
`measurements/plan34/census_120s.json` (our 67 leaves × site). New
`measurements/plan36/remap.py` emits `remap.json`: per family ours vs theirs in
one unit, the charge, the three `M`s, revised addressable pp (leaf-wise and
wall-minus-`M`), and a `refuted` flag per row whose ratio was a unit artifact.
A family not puttable in one unit is `unhashable` with its pp share;
> 20 % unhashable ⇒ INCONCLUSIVE.

**Phase B — the record.** Correct the two numbers `initiative.md` quotes
(`deep-win` per-node 4.38×, not 11–13×; tree ratio void pending item 47),
propose D36's supersession, one `history.md` line (report35 is not rewritten).

**Gates.** G-36a: every one of the 21 census buckets is in exactly one of
`priced` / `inside` / `folded` / `unhashable`, summing to 100 % of the op
budget. G-36b: convention A reproduces 1,785.9 ns from the committed files
(± 1 ns) — else the audit is not reading what report35 read. G-36c: no family
priced twice (asserted on the leaf set). G-36d: `A`, `B`, `C`, `min`, `max`
and the verdict. **Boundary:** nothing from the clone enters the
repo (integers + our op names only); no engine, flag, default, test or
`bench.md` row changes; no provari source, text or diff.

**Scope, effort, risks.** In: Phases 0–B, `measurements/plan36/`, `report36.md`,
the kill-review record. Out: any rewrite, any `bench.md` row, any D-decision
(the supersession is *proposed*), item 47's scoping. Effort ≈ 4 h. Risk: B/C
need one judgement — which op is "their heavy legality" — so price it at both
22.9 and 3.66 ns and take `min`. If Phase 0 shows our length is set by the
*proof share* rather than the guide, the length axis needs its own probe first.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan36.md`. Run the kill
   review's Phase 0 length probe (3 `atomi` seeds, read-only), then item 45
   M-REMAP Phases A–B: price the reference's per-child test families in our
   units from `measurements/plan35/shape.json` + `measurements/plan34/census_120s.json`,
   report `M` under conventions A/B/C, the verdict on `min`, the revised
   addressable pp. Data in `measurements/plan36/`, report `report36.md`; no
   engine/flag/default/`bench.md` change; nothing from the clone in the repo."
2. `min` > 1,723 ⇒ owner rules on plan34's C/D; item 44 is not written as a
   shape port.
3. Our length ≫ 19 ⇒ scope item 47 (SHORT-WIN, R25's idea in our words) in the
   next plan session, after [`plan37`](plan37.md).
4. [`plan37`](plan37.md) (item 46) may run before or after this plan.

SESSION COMPLETE
