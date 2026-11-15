# `prover` — document index

The initiative is [`initiative.md`](initiative.md). It is the only normative
statement of the goal, the rungs and the current priority. Everything else is
either a decision record, a measurement record, or a historical plan/report.

## Read in this order

| Document | Role |
| --- | --- |
| [`initiative.md`](initiative.md) | Goal, rungs, current status, decisions summary, prioritized backlog, risks |
| [`decisions.md`](decisions.md) | Full text and rationale for D1–D49; source take-list; legacy measured-out levers |
| [`gotchas.md`](gotchas.md) | Binding correctness gotchas — read before touching search, table, guide or allocator |
| [`bench.md`](bench.md) | Results ledger and measurement reading rules; no goal claims (D34) |
| [`history.md`](history.md) | Chronological record and findings; the durable narrative |
| [`analysis1.md`](analysis1.md), [`analysis2.md`](analysis2.md) | Prior-art differentials against provari; mechanism tables |
| `planN.md`, `reportN.md` | Per-session plan and report; historical, not normative |
| `measurements/planN/` | Committed drivers, parsed results, `env.json`, provenance READMEs |

## Current state (2026-11-15)

- Position: **`T_3`**; next rung **`T_4`** (all three cells at 10 s).
- Cap-120 row: `core` 4.923 s, `easy-win` 0.376 s, `deep-win` 53.42 s,
  geomean 6.606, 3/3 claimed, D14 9/9. `T_4` distance **5.34×**.
- Wall read per node (D48): **4.38×** (9,246 ns/exp vs 2,111 ns/body).
- The per-node route to `T_4` is closed by measurement (plans 34–37). The live
  axes are tree-changing: proof length (items 47/52), guide quality (item 48),
  and the gate/instrument shape (items 49/51/53).
- Next session: the **kill review due after plan37**, not another micro-plan.

## Conventions

- Plans are numbered sequentially and independently of backlog items; a plan
  header names its item.
- Plans ≤ 8 KB, reports ≤ 12 KB. If docs outgrow the code, cut docs, not tests.
- Measurements: one process at a time, same-session ratios, reading rules in
  [`bench.md`](bench.md); commit drivers, parsed results, `env.json` and
  READMEs under `measurements/planN/`; never raw transcripts, provari artifacts
  or `__pycache__`.
- The GPL boundary is hard: provari and shakmaty are read-only, scratch-only,
  ideas-only. See `AGENTS.md` and D12.
