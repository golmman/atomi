# Analysis 1 — where provari's tree reduction comes from, and what atomi is missing

Read-only analysis session, 2026-11-11. Not a plan, not a report: it fixes the
comparison, the mechanism table and the adopt order. No engine change, no
default change, no ledger row.

provari read: `/tmp/atomi-oracle/provari` (`48686bc`, v0.1.0, the veto and
bench reference) and `provari-v0.4.0` (`bceb1ee`), scratch clones, ideas only
(D12); atomi read: working tree `6038292`. Host: Ryzen 9 5950X, 4 vCPU, Linux
7.2.13. Method: source differential + single-seed probes at `--memory 192`,
one process at a time; ratios are same-session. The probes are **probes** — no
`--proof-out`, no repeats, no D14. `expansions` is compared with provari's
`nodes`.

## 1. The gap, measured three ways

| Cell (120 s, seed 0) | provari 0.1.0 hand | atomi `--guide none` | atomi `--guide hand` |
| --- | --- | --- | --- |
| `core` | 0.49 s · 342,289 nodes (670 k with `--guide none`) | **proved 18.4 s** · 9.27 M exp | censored, 2.02 M exp |
| `easy-win` | 0.11 s · 72,523 nodes | **proved 1.9 s** · 1.05 M exp | proved 21.6 s · 468 k exp |
| `deep-win` | 6.2 s · 4,574,912 nodes | censored (50.8 M exp) | censored (2.10 M exp) |

1. **Tree size: 27× on `core`, 14–19× on `easy-win`, ~9× on `deep-win`.** The
   proof *sets* are close (report16: 10.8 k vs 7.2 k positions), so atomi
   spends ~850 expansions per proof position against provari's ~47: the search
   is not tree-directed.
2. **The product guide is a net loss.** `--guide hand` is worse than
   `--guide none` on all three cells; with the auxiliary passes off (tree
   only, §4) `core` 12.79 M vs 9.27 M (0.72×), `easy-win` 1.42 M vs 1.05 M
   (0.74×). provari's hand guide is **2× better** than its `--guide none` on
   `core` (342 k vs 670 k nodes, re-measured here).
3. **Per-node cost of the guide.** `core` at 5 M child evals: bare 0.34 s,
   hand 10.2 s (30×); `aux_nodes` 34.1 M (settle 8.9 M, simulation 25.2 M).
   provari runs its full stack at ~700 k nodes/s, atomi's hand at ~15 k
   expansions/s.

## 2. Where the secret sauce is

Not one lever — **a coupled set of starting-number semantics**. provari's own
ablation (history.md, re-read against the source):

| provari stack on `core` | nodes | vs full |
| --- | ---: | ---: |
| full | 342 k | 1.00 |
| remove EF-PN / move-class numbers | — | 1.96× / 2.30× worse |
| remove history, null replay, closures, simulation, evasions, priors | — | 1.0–1.4× each |
| remove **all** of the above | 8.65 M | **25× worse** |
| product of the single-lever costs | — | 7.3× |

The levers are supermultiplicative (25× vs 7.3×), i.e. one system; and
**8.65 M ≈ atomi's bare tree (9.27 M)** — atomi's unguided search *is* provari
with the whole guide removed. The entire 25–27× gap is therefore the guide
stack, and atomi's port of it yields ~1.0×. Three coupled conventions are the
core, and atomi gets all three wrong (§3 R1–R4):

1. **A fresh child's deciding number is its class; the other number is 1**
   (`pn = class, dn = 1` at an OR child; `pn = 1, dn ∈ {1,2}` at an AND child).
   atomi writes the class pair into both numbers — quiet `(9,9)`, threat
   `(4,16)` — inflating every OR node's `dn = Σ dn` ~9× and every AND node's
   per-defence `pn` 9–16×.
2. **EF-PN is banded and asymmetric**: `step ∈ {0,1,2}` from the *parent's*
   hand eval with a ±500 cp decisive band, `pn ×{11,6,1}`, `dn ×{1,11,21}`,
   applied at *both* node kinds. atomi uses a symmetric
   `f = clamp(16+e, 8, 32)`, `pn × 16/f`, `dn × f/16` (4× range instead of
   231×), from the *child's* eval, OR nodes only, saturating at `|e| ≥ 16` —
   a near-binary ±2× mark from an eval that is itself on the wrong scale.
3. **The class predicates are cheap, exact and board-free**: check = number of
   *surviving* evasions (cap 8), threat = number of explosion targets,
   "hanging" = a quiet move whose piece the defender blasts cheaply,
   `UNLIKELY` = not (check, blast-capture of an attacker piece, king
   contact). atomi's classes are a coarse re-invention.

The rest of the stack (exact null-threat replay, exact sibling simulation,
per-child closures, plain-move grouping) is ~1.4× each on top. atomi
implements all of them, but both replays are *approximations* by default and
the closures cost a board build + movegen each — why they dominate wall
instead of tree.

## 3. The complete mechanism table

v0.4 keeps the search core unchanged (adds threads, exact tablebase play-out,
progress). atomi state: ✓ parity, ≈ same effect at higher cost, ✗
wrong/absent, ○ opt-in. Verdict: **fix** = adopt now, **align** = same idea,
wrong spec, **add** = absent, **skip** = out of scope by decision.

| # | Idea (our words) | provari spec | atomi state | Verdict |
| --- | --- | --- | --- | --- |
| R1 | Fresh-child numbers: class on the deciding side only | OR `(class,1)`; AND `(1,class)`, class ∈ {1,2} | ✗ class pair on both sides; `--guide-fresh-dn` patches only the OR side | **fix** — 1.5× on `core` from the AND side alone |
| R2 | EF-PN banded, asymmetric, both node kinds, parent eval | ±500 cp band, `pn ×{11,6,1}`, `dn ×{1,11,21}`, a=10 b=5 | ✗ symmetric ×0.5..2, child eval, OR only, saturates at \|e\|≥16 | **fix** — with atomi's numbers it costs 1.38× (`core`); provari's wins 1.96× |
| R3 | Check number = surviving evasions (cap 8) | evasions that do not lose at once | ✗ the parent's total legal count, uncapped | align |
| R4 | Explosion-target count as the threat number | 1 → 4, ≥2 → 1; weak threat (blasted cheaply) 8 | ✗ one "Threat" class, no count, no `blasted_cheaply` material test | align |
| R5 | Hanging = *quiet* move whose piece is blasted free | `blasted_cheaply` + Atomic-Stockfish values | ✗ any capture or any attacked piece | align |
| R6 | `UNLIKELY` prior | not check, not blast-capture of an attacker piece, no king contact → `dn=2` | ✗ predicate is a 64-bit null-threat signature | align |
| R7 | History: tie-break only, +1 per prove/refute, index role×64+to | two tables | ≈ also scales start numbers, `u16::MAX−score`, seed salt | align |
| R8 | Tie order: deciding number, then history | no class rank | ≈ class rank precedes history | align |
| R9 | Plain-move grouping (delayed expansion) | exact bit-test `plain()`; plain moves never scored, one GROUP child `(9,#plain)` under a salted key | ✗ `--guide-group` off; when on, members are fully scored first and installed one at a time | **add** (default on, score-free) |
| R10 | Explosion settles, either side | `explodes_them(effect)` bit test | ≈ built child + `status` | keep |
| R11 | "Exposing" one-move shortcut | `exposes(m,e)`: our king explodes after our move → `DRAWN`; after a defence the attacker explodes → `WON`; board-free | ≈ `settle_screen` + build + legal movegen + verify | **fix** (kills most `settle_nodes`) |
| R12 | Double threat with exposure wins at once | `threats ≥ 2 && exposed(after)` → `WON` | ✗ absent | add |
| R13 | "Exposed" win in two | every defence lets the king be exploded → `WON`, not kept | ✗ absent | add |
| R14 | Mate shortcut on a check child / at a defender node | `mated(next)` | ✓ built-child `status` | keep |
| R15 | Material-draw cut after a capture/promotion only | `might_end_material` + `cannot_win` | ≈ full `status` per built capture | keep |
| R16 | Repetition/50-move never wins, with provenance | `(cycle depth, from counter)` | ✓ same algebra | keep |
| R17 | Don't store recomputable proofs; mate cache in the TT; drop mates when proofs fill | `recomputed`, `Known::Mate`, `drop_mates` | ✗ kept as proofs; no mate cache; table just refuses at 90 % | add (memory headroom) |
| R18 | Null-threat replay, exact, proofs gained | passed position + exact winning move, replayed on every open defence; parrying fresh defence gets `pn/2` | ○ `--guide-null-exact` exact (report18: `core` proved at 18.3 M evals, `easy-win` 32.7× cut); default is a signature prior | **fix** (default on) |
| R19 | Sibling simulation, exact, budget 16 | Kawano replay; keeps the intermediate proofs | ✗ default is a greedy 6-ply playout that only scores; exact is `--guide-sim-exact` (2.35× `easy-win`) | **fix** (default on) |
| R20 | 1+ε widening | `second + 1 + second/8` | ✓ | keep |
| R21 | Child thresholds from the parent's sum | `ctdn = tdn−dn+cdn`, `ctpn = min(tpn, widen(second))` | ✓ | keep |
| R22 | Node-entry early return on stored numbers | `proof_within` + `lookup` at entry | ✗ `--stats` only; measured 45 of 153 k expansions would return | skip |
| R23 | Proof store: full keys, shortest/higher-limit domination, no eviction, refuse at 90 % | 12-byte entries, 64 MB at 10 s | ✓ 16-byte, fixed 25 % share (48 MB) | keep |
| R24 | Table: 4-slot clusters, least-work replacement, sparse filter | | ✓ | keep |
| R25 | Length-limited passes (`--most`, key salted by moves left), `--shorten` | off by default | ✗ absent (plan13's ply cap was a different thing) | skip for now |
| R26 | Network/model guide, tablebases, threads, virtual numbers (v0.4) | | ✗ | skip (D6/D7/D8) |

## 4. The decisive probe: provari's number algebra, flags only

Two existing flags express R1+R2 approximately: `--guide-params` with every
class `*_dn=1`, `--guide-on fresh-dn`, `--guide-ef-weight 0`
(`p_provari.txt`: `threat_pn=4 threat_dn=1 weak_pn=8 weak_dn=1 quiet_pn=9
quiet_dn=1 hanging_pn=15 hanging_dn=1 evasion_pn=1 evasion_dn=1`). Auxiliary
passes off, so this is **tree only**:

| Stack (`core`, seed 0) | expansions | vs bare | vs product hand |
| --- | ---: | ---: | ---: |
| bare | 9,266,026 | 1.00 | 1.38× |
| product hand | 12,785,169 | 0.72 | 1.00 |
| + `fresh-dn` only | 15,210,356 | 0.61 | 1.19 |
| + `ef-weight 0` only | 9,376,325 | 0.99 | 1.36 |
| + params `dn=1` only (AND side) | 6,178,223 | 1.50 | 2.07 |
| + params + `fresh-dn` (both sides, EF-PN 16) | 4,952,893 | 1.87 | 2.58 |
| **params + `fresh-dn` + `ef-weight 0`** | **3,588,914** | **2.58** | **3.56** |

Other cells, same stack: `easy-win` 1.05 M → 670 k (1.56×); `deep-win`
censored at 88.6 M (bare) → **proved** at 23.4 M (length 24, 107 s) against
53.7 M for the product hand (2.29×).

With the auxiliary passes on (the real default), 120 s, seeds 0–2:

| Cell | product `--guide hand` | provari-numbers stack |
| --- | --- | --- |
| `core` | censored 0/3 (2.0 M exp) | **proved 3/3**, 16.5 / 18.0 / 17.8 s |
| `easy-win` | proved 3/3, 21.6 s | **proved 3/3**, 3.0–7.5 s (139 k–773 k exp) |
| `deep-win` | censored 0/3 | 1/3 (seed 0, 106 s); seeds 1–2 censored at 25.3 M |

A **numbers-only** change turns the product guide from a net loss into a
2.6–3.6× tree win, proves `core` 3/3 at 120 s where it was censored, and makes
`deep-win` prove on one seed. It is not `T_3` (`deep-win` 1/3), but it is the
first tree lever in 18 plans with a > 2× case — and it exposes the two
remaining deficits: EF-PN is still atomi's mapping (provari's is 1.96× where
atomi's costs 1.38×), and the auxiliary per-node cost now dominates wall
(aux off: `core` 17.8 s; aux on: 106 s).

## 5. Adopt order, each with its cheapest decisive experiment

1. **R1–R6 as one stack: the starting-number algebra.** Gate: `core` +
   `deep-win` unique expansions at 20 M, ≥ 2× vs the product, no `easy-win`
   regression. Cheapest first step: make the §4 flag stack the *default* (no
   new code) and re-measure.
2. **R18+R19: the exact replays by default**, dropping the greedy playout and
   the signature prior (both implemented, both D14-accepted in report18).
   Cheapest: both flags on top of step 1, three cells at 20 M.
3. **R11–R13: board-free closures** (bit-test exposing shortcut, double
   threat, exposed win). Gate: `aux_nodes`/expansion ≤ 1, no tree regression.
4. **R9: plain-move group on by default**, members never scored. Gate:
   expansions at 20 M and per-node wall.
5. **R17: memory headroom** only if `proof_full` appears in a real run.

Not adopted: network/model (D8), tablebases (D7), threads (D6), `--shorten`
and `--most` (no gate; plan13's length probes were inert).

## 6. Problems and caveats

- §4 is single-seed, no repeats, no D14: evidence, not a claim. The `deep-win`
  3/3 and `T_3` questions need the full matrix.
- **`--guide hand` resets the guide configuration** (`cli.rs` assigns a fresh
  `GuideConfig`), so a `--guide-off` placed *before* it is silently discarded.
  Every run above puts the guide flags after `--guide hand`; a future plan
  should fix the CLI or document the order, since earlier drivers may share
  the trap.
- `--stats` counts distinct keys in a fixed 2^21 set that stops inserting at
  70 % (1,468,006) and then reports a capped lower bound (`unique_keys_capped`).
  All counts above are below that bound and comparable; a count *at* it is not
  (earlier reports' unique counts above 1.47 M cannot come from this set and
  should be re-derived before being compared).
- Everything here is derivative *ideas* read from provari's source; no code,
  comment or doc text was copied (D12). The §4 stack uses atomi code only.
- Next: a plan session adopting §5.1–§5.2 as a declared stack with a
  `deep-win` gate, plus a `bench.md` note that the product hand guide is
  currently a net loss.

SESSION COMPLETE
