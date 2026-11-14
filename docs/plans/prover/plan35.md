# Plan 35 — item 43 (proposed, SHAPE-DIFF): is the reshape funded?

Read only: [`report34`](report34.md) §3–§5,[`analysis2`](analysis2.md) §5–§6,
[`plan34`](plan34.md) §4,[`gotchas`](gotchas.md) §Scope, D36/D40/D43–D46,
[`bench.md`](bench.md) cells.
Pins: oracle `48686bc6c786d6ea70568f0b2ad4002cf0520206`, `atomic-movegen` 2.8.0.

**Why this item.** The census priced *our* body (37.0 s = 68.97 pp in named
calls, `Frame::enter` 51.77 pp + null replay 45.04 pp) and never the
*reference's*: no count of provari's per-node work exists here. `T_4` needs
81.3 pp (43.62 s) deleted, and reshape (plan34 B) claims they exist because a
node of ours does 290 moves / 9.6 lists / 46 probes / 8.2 builds where a
reference node does its (unknown) smaller set. Item 42 measured volume on one
side only; this item measures the other — **counts only, no timing** — and
prices the difference with our measured ns/op. It is the last measurement that
can decide the goal rung; it is not a lever.

## 1. The decision (pre-registered, before any count is taken)

Let, per expansion, `M = Σ_ops (p_ops × ns/op)` where `p_ops` is provari's
call count per *node body* in the census leaf's own unit and `ns/op` is our
plan34 §4 rate (tax-subtracted lower bound). `M` is a **hard lower bound** on
any tree-neutral reshaped body: no body is cheaper than its own accounted calls
at our rates. Wall per expansion is 9,246 ns; `T_4` (tree held at 5,803,257
expansions) allows **1,723 ns/exp** = 81.3 pp.

- **FUNDED ⇔ `M` ≤ 1,723 ns.** Addressable pp = (9,246 − `M`)/92.46 ≥ 81.3, and
  item 44 (the v3 node-body rewrite, plan34 B) is worth an owner ruling; report
  the headroom `1,723 − M` left for un-timed scaffolding.
- **NOT FUNDED ⇔ `M` > 1,723 ns** (report the shortfall `M/1,723`): no reshape
  reaches `T_4`, the goal question returns to the owner with plan34's **C**
  (re-anchor D30) and **D** (archive at `T_3`), and **no plan 36** is written.
- Report `M + R` and `M + R·M/6,376` beside it (`R` = 16,651 ns/exp = the 31.0
  pp outside every census bucket, held constant / scaled with mapped calls):
  they size the risk, they do **not** move the verdict.

**Pre-registered break-even** (calls/exp that alone exhaust 1,723 ns):

| census leaf | our ns/op | break-even |
| --- | ---: | ---: |
| legal movegen (per list-generation call) | 85 | 20.3 |
| closure settle+screen | 37.7 | 45.7 |
| guide `hand_eval` (per move) | 199 | 8.7 |
| status/legality/terminality | 19 | 90.7 |
| proof probe | 10 | 172 |
| `do_move`/`undo`/clone | 2.5 | 689 |
| table store | 36 | 47.9 |

Ours, for scale: 13.51 movegen calls + 2.06 legal lists / **53.08 legal
moves**, 28.3 closure calls, 3.68 `hand_eval`, 26.75 child evals, 8.17 builds,
46 probes. **The verdict hinges on provari's move volume**: ≲ 8 moves/node ⇒
`M` ≈ 1.0–1.3 µs and FUNDED (the 12.2 pp guide stack gone by construction);
≳ 20 ⇒ movegen alone eats the budget and NOT FUNDED. Expected verdict goes
here (`deep-win`: 4.57 M nodes, 1.36 µs/node ⇒ a small body) — count first.

## 2. The count side (scratch clone, counts only)

`/tmp/atomi-oracle/provari` at the pin, **never** in the repo. A count-only
build: a `Cell<u64>` set (no rdtsc, no atomics, single thread) dumped on exit via
an env var; 12–14 sites, one per family. Nothing is timed; **provari's wall is
not used** (other box, session, clock — only our plan34 rates price anything).

Six families, each counted in the census leaf's unit:

1. **moves** — list generations, moves yielded (pseudo and legal), per-move
   guide-number evaluations.
2. **lists** — creations per node and their sizes.
3. **probes** — table, proof-store, filter.
4. **guide numbers per move** — evaluations, class/threat tests, number stores.
5. **child builds** — clones, play/unplay, key+clock, terminality/status, stores.
6. **null scans** — null uses, replies scanned, plays per reply, probes inside.

Runs: `provari prove "<deep-win FEN>" --guide hand --seconds 120 --memory 192`
(the plan34 decision cell, a *completed* run) and `core` @ 10 s as a second
cell. Dump a `body` counter at node entry too: **the denominator is `body`, not
the reported `nodes`**, and any deviation is recorded — a reported node that
skips the body would corrupt every rate. A family not countable without a
behavioural change is **unmapped**: printed, excluded from `M`; if more than
20 % of our op budget is unmapped the measurement is **INCONCLUSIVE** (⇒ no
funded verdict, owner gets C/D with that reason).

## 3. The mapping, validated on our side first

`G-35c` is what makes the arithmetic trustworthy: apply the mapping to **our**
census counts and ns/op; it must reproduce the census total **37.005 s ± 2 %**.
A mapping that cannot reproduce our own op budget is not applied to provari's.
Granularity rule: count each family in the unit of the census leaf that owns
its cost (list generation, per-move number, per-child build, per-probe); per-op
`ns` come from report34 §4 verbatim — already lower bounds.

Output table (one row per leaf): `p_ops/node`, `ours/exp`, **volume ratio**,
`p_ops × ns/op`, its pp, and `addressable = ours_pp × (1 − 1/ratio)` when ratio
> 1, else 0. Ops provari lacks (guide number stack, prefetch, filter probe,
`Mover::new`) have ratio = ∞ and their whole bucket is addressable — where the
12.2 pp live: show them, do not let a sum hide them.

## 4. Gates

- **G-35a coverage** — every counted quantity in exactly one family; `body`
  count vs reported `nodes` recorded; unmapped share ≤ 20 % (§2).
- **G-35b determinism** — 3 identical runs give identical counts (or median +
  spread; a nondeterministic node count is a finding).
- **G-35c arithmetic** — §3 reproduces **37.005 s ± 2 %** on our counts.
- **G-35d decision** — `M`, addressable pp, `M + R`, `M + R·M/6,376`, the ±15 %
  band (the movegen observer-effect spread) and whether the verdict flips
  inside it; verdict FUNDED / NOT FUNDED / INCONCLUSIVE per §1.
- **Boundary** — nothing from the clone enters the repo: no patch, no sed
  script, no source fragment, no diff, no doc text; the committed JSON is
  **integers plus our own op names**, the procedure is described in our words.
  The counter patch lives and dies in `/tmp`; `PROVENANCE.md` untouched.

## 5. Scope, effort, risks

**In:** the scratch count build, `measurements/plan35/` (`shape.py` = run +
parse + arithmetic, `shape.json`, `env.json`, README), `report35.md`, one
`history.md` line, item 43's row. **Out:** any engine/flag/default/test change,
any `bench.md` row, any D-decision (a funded verdict goes to the owner as plan34
B's price; D45 stands until the owner narrows it), any rewrite, any provari
code/text, any timing of provari.

**Effort** ≈ 3 h: 12–14 counter sites, 2 cells × 3 runs, the table. **Risks:**
(i) a family not hookable without a semantic change ⇒ unmapped (§2); (ii)
extrapolating our ns/op to a volume never measured ⇒ the ±15 % band and the
`M + R` readings, verdict on the point estimate with the band beside it; (iii) a
reference body folding two of our passes into one loop is not a volume ratio ⇒
mark the leaf "folded", use `body`-relative counts.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan35.md`. Execute item
   43 as written: a count-only instrumented build of provari at pin `48686bc`
   in `/tmp/atomi-oracle` (no timing, no code or patch in the repo) counting the
   six families per node body on `deep-win` @120 s hand and `core` @10 s;
   validate the mapping on our own counts against 37.005 s ± 2 % (G-35c), then
   report `M` against the pre-registered 1,723 ns/exp and the FUNDED / NOT
   FUNDED / INCONCLUSIVE verdict of §1. Data in `measurements/plan35/`, report
   `report35.md`; no engine change, no `bench.md` row."
2. FUNDED ⇒ owner rules on plan34 B (v3 node-body rewrite) with `M` as its
   price; decisions.md first, then an item.
3. NOT FUNDED / INCONCLUSIVE ⇒ owner rules on C or D (goal section, `bench.md`
   framing, `check_goal.py`, one session).
4. Sign off G-39c's pooled sample; confirm/withdraw D44–D46.

SESSION COMPLETE
