# Prover Plan 6: item 15 — hand-guide conformance, the `GROUP` child

Initiative: [`prover`](initiative.md), horizon **item 15**. Drafted
2026-11-09 in plan session 6 after the D23 erratum in
[`initiative.md`](initiative.md) §Backlog. Runs before [`planX`](planX.md)
phase D (the `.pvr` freeze), per item 15's trigger. No impl in this document.

## Why this plan, and what it is not

Item 3 was closed on **atomi's variant** of the hand guide; the erratum
records five semantic divergences from provari's. The strongest one is quiet
grouping: plan3 implemented "quiet moves" as a starting-number statement, not
as one deferred child. This plan tests that single divergence. It is a
**kill-test for the hand-guide line**, not a stack adoption: item 15's gate is
narrow (two cells), and a pass only keeps the line alive.

Recorded evidence (item 15, same-session probes, plan4/planX tree):

| Probe | `core` | `easy-win` |
| --- | ---: | ---: |
| current hand stack, 10 s | 148,480 / 149,504 expansions, censored | 190,464, censored |
| + fresh `dn=1` | 143,360, censored | 166,847, **proved** |
| + `dn=1`, EF-PN off | — | 110,339, **proved** |
| `--guide-off sim,closures`, 200 M evals | > 6.9 M expansions | — |
| provari hand | 342,289 nodes, 0.53 s | 72,523 nodes, 0.14 s |

The tree deficit is ≥ 20× on `core`; provari's own ablation prices the whole
hand stack at 25× `core` and 10.9× `easy-win`, with coupled levers (single
levers multiply to 7.3×). The question is whether the missing `GROUP`
semantics recovers a large part of that, or whether the line is dead.

## Scope

**In:** an attacker plain-move `GROUP` child behind one flag; the value
algebra, the proof-writer mapping, unit/integration tests, the two-cell
measurement, and a `report6.md` close/keep verdict.

**Out:** a rules/search-core rewrite, a new proof format, an optimizer or
ablation sweep, default-flag changes, `.pvr`, restarts, portfolio, and any
provari code or binary in the repo. Phase B (exact sibling-proof replay) is
**not** implemented here unless A passes; item 15's cheapest decisive test is
the group alone.

## Design: the `GROUP` child

One flag (working name `--guide-group`), default off.

1. **Class.** On an attacker node, the *plain* moves are the legal moves that
   are not checks, not captures/explosions and not promotions. If the class is
   empty there is no group. The exact predicate is one function, unit-tested
   against `Mover`/`Effect`.
2. **Pseudo-child.** The parent OR frame gets one additional child: a group
   entry keyed by the parent position key plus the class, not a position.
   Starting numbers `(pn, dn) = (9, max(1, #plain))` per item 15. The member
   list is ordered by the guide's existing move order; only one member is
   installed per expansion of the group.
3. **Value algebra.** The group is a deferred OR over its members: members
   not yet classified contribute their own starting numbers; once classified,
   their real values. Group `pn = min` over members, `dn = sum` over members
   (clamped at the existing `OPEN_MAX`), and the group is `Proved` only when
   at least one member is `Proved`, `Refuted` only when every member is
   `Refuted`. While any member is `Open`, the group is `Open`. The heuristic
   `(9, #plain)` is a **starting number only**; it never substitutes a verdict.
4. **Parent propagation.** In an OR parent, the group competes as one child
   for selection; when selected, the frame expands the next member. The
   parent's value is unchanged in meaning: it is still the OR over the real
   children, because the group's proven/refuted result implies a real member.
5. **Proof writer.** A group key is not a position. Descending through a
   proved group must resolve to the member whose value the group's proof
   selected; if no member is proved, the writer fails closed. Proof lengths
   and 50-move limits stay on real children only.
6. **Determinism.** Seed perturbs tie-breaks only (D22); the group ordering
   uses the same tie-break path, so seeds 0–2 are deterministic.

**Invariant to test:** after every member is classified, the group's value
equals the OR combination of the members' values (randomized property test).
A group may be more optimistic earlier; it may never hold a value the real
members do not imply.

## Measurement and gate

Runs at fixed `--evals 20000000 --memory 192`, seeds 0–2, one process at a
time. Baseline: the same binary with the flag off.

- **G1 group activation.** `core` with `--guide-group` reaches
  **< 1,000,000 expansions at 20 M child evals** (≥ 5× cut against the
  recorded 6.9 M+). Proved or censored is recorded; censored at > 1 M fails.
- **G2 easy-win no regression.** `easy-win` proved on seeds 0–2 and
  expansions ≤ the flag-off count on the same seeds; also record against
  the D21 limit 145,046 (not a G2 fail).
- **G3 soundness.** No guidance-set refutation; any proved cell's D14 proof
  accepted by `atomi validate` and `provari check`; the group unit/property
  tests and a crafted position where a plain move is the only win (group
  must prove) and one where every plain move loses (group must not prove)
  are green.
- **G4 determinism.** Seeds 0–2 counters identical on repeat; flag-off
  counters unchanged from the current tree.
- **G5 build/docs.** `make test`, `make lint`, `make test-full` (includes
  `make veto`); no GPL-family licence; `measurements/plan6/` artefacts and
  `report6.md`.

**Stop rule.** If G1 fails, close the hand-guide line in `report6.md`; do not
implement replay, do not re-scope item 15, and make the next plan a wall
lever (D5 restarts, wall-budgeted auxiliary work, or the horizon-8 driver).
If G1 passes but G2 fails, the same close applies. If both pass, the line
stays alive: phase B (event-driven sibling-proof replay, the second half of
item 15) is drafted as a follow-up plan under the same two-cell gate, and a
**separate** D21 24-cell adoption gate is required before the stack changes
the default — this plan does not adopt it.

## Tasks

- **T1** `GROUP` class + pseudo-child + value algebra in `src/guide/`,
  `src/search/frame.rs`; flag in `cli.rs`; unit/property tests.
- **T2** writer mapping + fail-closed test; fixture sweep for refutations.
- **T3** `measurements/plan6/run_group.py` (stdlib), flag-off/on runs,
  `group.json`, `soundness.json`, `env.json`, README.
- **T4** G1–G5, `report6.md` keep/close verdict, docs status line.

## Risks

| Risk | Mitigation |
| --- | --- |
| Group value is optimistic and hides work needed for a proof | Value-OR property test; group reflects members once classified; D14 proof + both checkers |
| Group hides a refutation | Crafted all-plain-lose position; fixture sweep; no-refutation gate |
| `dn=#plain` overflows the parent sum | Clamp at `OPEN_MAX`; test with a 30-move plain set |
| The 20 M gate is too weak (censored mid-tree) | Record censored counts; one 200 M confirmation only if the trend is under 2 M and G1 is otherwise ambiguous |
| Group changes proof shape and breaks `.pvr` later | Plan7 phase D is provisional; format is key/move-based and the group adds no key |
| Two-cell gate over-read as adoption | Stop rule: pass = line alive; D21 24-cell gate is separate |
