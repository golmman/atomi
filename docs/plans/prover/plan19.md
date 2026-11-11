# Prover Plan 19: item 26 — NUMBERS-V1, the starting-number algebra

Initiative: [`prover`](initiative.md), backlog **item 26**, decisions **D12**,
**D14**, **D21**, **D26**, **D30**, **D32**, **D33**. Drafted 2026-11-11 after
[`analysis1`](analysis1.md) and the analysis revision of `initiative.md`. Plan
only: no engine change in this session.

## Kill review (delta — full review was plan18)

1. **Distance now.** `T_2`. Bare 120 s: `core` 3/3 17.471 s (35.8×), `easy-win`
   3/3 2.023 s (18.9×), `deep-win` 1/3 (113.458 s). New fact: the bare tree
   *is* provari-without-guide (`core` 9.27 M vs 8.65 M expansions), so the
   25–27× is the guide and the product mapping is a net loss. `T_3` needs
   `deep-win` 3/3; the numbers stack nearly reached it (1/3, 107 s).
2. **Would we build this?** Yes — the only >2× tree lever in 19 plans, same
   search base. Not: PDS-PN/PN², portfolio, restarts, threads, tablebases.
3. **Cancel?** The `--shorten`/`--most` line; items 27–29 wait behind this
   gate.
4. **Prior art.** DF-PN + EF-PN guidance (Nagai 1999/2002) plus provari's
   hand conventions; nothing later changes a numbers-only port. Cheapest
   decisive experiment: analysis §4 at 20 M (Phase A, no code).

## Decision

Adopt **NUMBERS-V1** as one D21 stack (R1–R6, analysis1 §3): a fresh child's
deciding number is its class, the other number 1; EF-PN banded from the
**parent's** eval at both node kinds; check number = surviving evasions (cap
8); threat number = explosion-target count; hanging = a *quiet* move whose
piece is blasted cheaply; `UNLIKELY` → `dn = 2` when not check, no
blast-capture of an attacker piece, no king contact. Defaults change only
after G3 (D32).

## Scope

**In:** the CLI order fix (D33); Phase A reproduction with existing flags; the
six mechanisms behind flags, default off; the 20 M gate; if it passes, the
default change, the 120 s matrix, D14 and the item-26 status.

**Out:** items 27–29; auxiliary cost work; memory/table changes; search-base
changes; any goal, budget or memory change; provari code, text or proofs.

## Baseline to reproduce (analysis1, evidence not claim)

Tree only, aux off, `core` seed 0: bare 9.27 M; product hand 12.79 M (0.72×);
`dn=1` params + `fresh-dn` + `ef-weight 0` **3.59 M (3.56× vs product)**;
`easy-win` 1.05 M → 0.67 M; `deep-win` proved at 23.4 M. Aux on, 120 s, seeds
0–2: `core` 3/3 (16.5–18.0 s), `easy-win` 3/3 (3.0–7.5 s), `deep-win` 1/3
(106 s). Per node the guide costs 30× (`core` 5 M child evals: 0.34 s bare,
10.2 s hand; `aux_nodes` 34.1 M).

## Design

### Phase 0 — CLI order fix (D33)

`--guide hand|none` replaces the whole `GuideConfig`, silently dropping guide
switches given earlier. Apply **base first, then switches** regardless of
argv order; one test per order and one for `--guide none` identity. Drivers
assert the resolved config.

### Phase A — reproduction with existing flags (no code)

`--evals 20000000 --memory 192 --seed {0,1,2} --stats`, three cells, two
stacks: `--guide hand` and `--guide hand --guide-params p1.txt --guide-on
fresh-dn --guide-ef-weight 0` (`p1.txt`: every `*_dn=1`). Record `expansions`
(primary, deterministic), `unique_keys` + `unique_keys_capped`, wall,
`aux_nodes`, `proof_full`. **No unique count at the 1,468,006 cap is used**
(D33): a capped cell is reported from `expansions` only. G2: the product
stack reproduces a net loss vs bare and the numbers stack a ≥ 2× win on
`core`.

### Phase B — NUMBERS-V1 (R1–R6, default off)

One `--guide-on/off` switch per mechanism, so the
ablation runs after the gate; `--guide-on numbers` turns the six on.
Default off.

- **R1 fresh pair** (`fresh-class`). OR child `(class, 1)`, AND child
  `(1, class)` with the AND class ∈ {1, 2} (2 when the defence concedes a
  threat, else 1); the class table stays in `--guide-params`. Replaces the
  class-pair-on-both-sides and the `fresh-dn` OR-only patch. Tests: both node
  kinds, class ∈ {1,2}, `(1,1)` for `--guide none`.
- **R2 EF-PN banded** (`efpn-banded`). Step ∈ {0,1,2} from the **parent's**
  eval oriented to the mover, both node kinds, `pn ×{11,6,1}`, `dn
  ×{1,11,21}`. The edge is in our `hand_eval` units (`[-256,256]`, not
  centipawns): probe {64,128,192} at 20 M on `core`, fix it, record it. No
  child eval on this path; no build for OR quiet moves.
- **R3 check number** (`evasions-count`). Evasions still `Open` after pass 1
  (a decided or settled one is not surviving), cap 8, instead of the parent's
  total legal count — a post-pass over `starts`, no second build, no movegen.
  Tests: a 1-survivor and a >8-survivor position.
- **R4 threat number** (`threat-count`). Explosion targets in the built child:
  1 → 4, ≥ 2 → 1; a threat itself blasted cheaply → 8; else quiet 9. Helper +
  tests.
- **R5 hanging** (`hanging-quiet`). A *quiet* move whose moved piece the
  defender blasts cheaply, from our own documented material table (commoner
  excluded). Replaces "any capture or attacked piece".
- **R6 `UNLIKELY`** (`unlikely-exact`). AND children only: not check, no
  blast-capture of an attacker piece, no king contact → `dn = 2` (replacing
  `pn/2, dn*2`).

Soundness: fresh `Open` numbers only (gotchas); `--guide none` and the plan18
flags stay counter-identical; no new keyed position.

### Phase C — the 20 M gate (G3)

Three cells × seeds 0–2, `--evals 20000000 --memory 192 --stats`, stacks:
bare, product hand (arm A), NUMBERS-V1 (arm B), and arm B with aux off
(`--guide-max-sim 0 --guide-max-settle 0`) to separate tree from aux cost.
Metrics as Phase A. G3, on the **median of three seeds**:

- `core` and `deep-win` `expansions` ≥ **2×** cut vs the product hand;
- `easy-win` no regression vs bare > 10 %, each seed still proves or matches
  bare's censored count;
- no refutation; `unique_keys_capped` reported per cell.

### Phase D — only if G3 passes

`--guide hand` becomes NUMBERS-V1 (R2 banded included); greedy playout and
signature prior stay on (item 27). Then the 120 s matrix (3 cells × seeds
0–2) and D14 per proved cell; `T_3` only if 3/3 with both checkers. Ledger row
plus the item-26 status; a `T_3` miss is recorded, not converted.

### Phase E — if G3 fails

Close item 26 per the initiative stop rule: record the stack, both-direction
single-mechanism ablations at 20 M, no per-class re-tuning, no split. Items
27–29 do not start. Report the numbers as a negative result.

## Gates

- **G1 data.** `env.json`, `probe.json`, `gate.json`, `matrix.json`, parsed
  only, one process at a time, drivers + README under `measurements/plan19/`.
- **G2 Phase A reproduction.** Product net loss and numbers ≥ 2× `core`;
  otherwise stop before writing code.
- **G3 tree size.** Phase C as stated; judge `expansions` first.
- **G4 soundness.** `make test`, `make lint`, `make test-full` (search change
  ⇒ `make veto`), `--guide none` identity, no new GPL dependency, no provari
  proof committed.
- **G5 docs.** `report19.md`; item 26 and D32 status; a `bench.md` note that
  the product hand guide is a net loss.

## Risks

Aux work dominates wall (`core` 17.8 s aux-off vs 106 s aux-on): the gate is
tree size. The band edge is our eval's scale, not provari's. `--stats` is
measurement-only and must not move the trajectory.

## Deliverables

Data: `measurements/plan19/{README.md,env.json,p1.txt,probe.json,gate.json,
matrix.json}`. Docs: this plan, `report19.md`, `initiative.md` item 26 and
D32. Engine: CLI order fix + six mechanisms behind flags + tests; default
change only at Phase D.

## Follow-up options

1. **Kickoff (recommended):** "Read `docs/plans/prover/plan19.md`. Execute
   item 26 Phase 0 (CLI order fix) and Phase A (reproduce the analysis §4
   stack at 20 M on three cells, seeds 0–2); stop at G2. If G2 passes,
   implement NUMBERS-V1 (R1–R6) behind flags and run Phase C. Do not start
   items 27–29, do not change defaults before G3, do not touch the search
   base."
2. G2 fails: close the premise, write a goal re-pin plan under D30.
3. G3 passes: Phase D (default + 120 s matrix + D14); then item 27.
4. G3 fails: Phase E; the plan20 kill review picks the next lever.
