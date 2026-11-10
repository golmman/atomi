# Prover Report 14: item 23 — owner accepts 2/3 as Tier 1 (D28)

Plan: [`plan14.md`](plan14.md) (item 23, D14/D18/D24/D27). Executed
2026-11-10. Decision-only: no engine change, no measurement run, no ledger
row. The owner chose **option A**; **D28** amends D27 so Tier 1 is **2/3**
(`core`, `easy-win`) at `--seconds 120 --memory 192`, with `deep-win`
recorded as 1/3 and excluded from the pass condition.

## Provenance

| Item | Value |
| --- | --- |
| Host | not used (no run) |
| Toolchain | rustc 1.99.1 (58cf8f9c3 2026-10-13), Python 3.14.10 |
| Dependencies | unchanged: `atomic-movegen` 2.4.0, `serde_json` 1.0.154 |
| atomi | HEAD `a4eb440` on a dirty tree (plan14 docs; agents do not commit) |
| provari | 0.1.0, `48686bc6c786d6ea70568f0b2ad4002cf0520206` (not rebuilt) |
| Configuration | Tier 1: `--guide none --seconds 120 --memory 192`, seeds 0–2 |

Commands: `make test` (licence gate + `cargo test --release`), green. No
`make test-full`/`make veto` because no code, search, table, proof-output or
validator path changed.

## What changed

- [`initiative.md`](initiative.md): status paragraph for plan14; success
  condition Tier 1 amended to 2/3 with the `deep-win` 1/3 record; **D28**
  added after D27; backlog priority rewritten; item 23 row added; history
  entry.
- [`bench.md`](bench.md): success-condition quote amended to 2/3; plan14
  update note; **D28 status** block; changelog entry.
- No source, test, flag, default or measurement file changed. The plan-session
  `--stats` key-set probe was already reverted before this session.

## Decision record (D28)

- **Tier 1:** 2/3 on `core`/`easy-win` at 120 s / 192 MB, every claim
  D14-accepted, no fixture regression. `deep-win` is recorded as 1/3 (seed 1
  111.218 s, seeds 0/2 censor at 120 s) and is not a pass condition.
- **Tier 2:** unchanged — same-session 10 s geomean `< 1.0` on shared cells.
- **Fixed:** 120 s budget, no hand-guide/auxiliary reopen, no portfolio.
- **Product candidate:** the bare tree (`--guide none`).
- **Evidence:** the 120 s `deep-win` tree is 50,016,256 expansions /
  40,708,534 unique keys (18.6 % re-expansions, 2,661,718 proof entries,
  `proof_full: false`), ~8.9× provari's 4.57 M nodes. The gap is unique
  positions, not memory, re-expansion, proof pressure or per-node cost.

## Gates

| Gate | Result |
| --- | --- |
| G1 decision | **pass** — A recorded as D28 in `initiative.md` and `bench.md` |
| G2 budget | **pass** — no run above 120 s, no budget raise |
| G3 closed lines | **pass** — no hand-guide/auxiliary reopen, no portfolio |
| G4 code | **pass** — no engine change; probe patch already reverted |
| G5 docs | **pass** — plan, decision record, no GPL dependency/artifact/`__pycache__` |

## Problems and open parts

- The 3/3 goal is not met; `deep-win` remains a recorded censor, not a win.
- Tier 2 (wall geomean `< 1.0`) is still unclaimed and now sits behind a
  2/3 Tier 1.
- Item 5 (M4) is still parked; its resume condition ("3/3 at 10 s") is now
  inconsistent with D28 and needs an owner decision.
- No new search algorithm was authorised (option C not taken).

## Missing tests

None: no code changed. The existing `make test` suite is the only applicable
gate and is green.

## Next steps

1. Owner decision on item 5 (M4): resume under D28, re-scope, or close.
2. Item 6 (M5) head-to-head once the product configuration is settled.
3. If the owner later rejects 2/3, option C (new search algorithm) needs a
   read-only probe with a pre-registered 2× stop rule; option B needs a
   separate goal-definition plan and must not call a bound a win.
