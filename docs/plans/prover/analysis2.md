# Analysis 2 — provari's ideas compiled, with the measured gap to atomi

Read-only analysis session, 2026-11-12. Not a plan, not a report: it compiles
the mechanism table of [`analysis1.md`](analysis1.md) §3 with the gap each
mechanism has to atomi, as measured by plans 18–28. No engine change, no
default change, no ledger row. Sources: `analysis1.md` §2–§3, `report18`–
`report28`, `decisions.md` D31–D40. provari is read in a scratch clone as
**ideas only** (D12); no code, comment or doc text is copied.

## 0. Gap terminology

"Gap" is the measured distance on **one mechanism**, in the unit that
mechanism acts on: tree = expansions or child evals, wall = seconds,
per-node = µs. Gaps are **not additive** — the levers are supermultiplicative
(single-lever costs multiply to 7.3×, removing all of them costs 25×,
analysis1 §2), so a row's gap is a local reading, not a share of the total.

| Term | Meaning |
| --- | --- |
| **parity** | atomi implements the same idea with the same effect; no measured gap. Not necessarily identical code. |
| **closed** | the idea was absent or wrong, a plan ported and **adopted** it under a gate, and the measured gap is gone. "Closed" is a history word: a plan ran and the gain is recorded. |
| **aligned** | same idea, different spec; atomi's variant is close but not identical, and no gate has separated them. Not measured as a gap. |
| **open** | a measured gap remains; the idea is not captured, or only partly. |
| **rejected with data** | the idea was implemented and measured, and it **lost** on atomi's tree. The gap is not "we lack it" but "it does not transfer as specified". |
| **skipped** | out of scope by decision (D6–D8, R25), not a gap. |
| **small, skipped** | measured, but below the threshold to act on. |

## 1. Search base

| # | Idea | provari spec | atomi state | Gap |
| --- | --- | --- | --- | --- |
| B1 | DF-PN with proof/disproof numbers | `combine`/`share`/`widen` | same base | **parity** |
| B2 | 1+ε widening | `second + 1 + second/8` | same | **parity** |
| B3 | Child thresholds from parent sums | `ctdn = tdn−dn+cdn`, `ctpn = min(tpn, widen(second))` | same | **parity** |
| B4 | Node-entry early return on stored numbers | `proof_within` + `lookup` at entry | `--stats` only; 45 of 153 k expansions would return | **small, skipped** (R22) |

## 2. Starting-number algebra (the big lever)

| # | Idea | provari spec | atomi state | Gap |
| --- | --- | --- | --- | --- |
| N1 | Fresh child: class on the deciding side only | OR `(class,1)`, AND `(1,class)` | **adopted** (item 26) | **closed** — part of 4.74×/3.34×/2.80× |
| N2 | EF-PN banded, asymmetric, both node kinds, parent eval | ±500 cp band, `pn ×{11,6,1}`, `dn ×{1,11,21}`, a=10 b=5 | symmetric, child-eval, OR-only, 4× range | **open** — atomi's mapping *costs* 1.38× on `core`; provari's spec wins 1.96×. Atomi's port of the spec cost 1.2–1.35× at every probed edge → **rejected with data** |
| N3 | Check number = surviving evasions (cap 8) | evasions that don't lose at once | **adopted** (item 26) | **closed** |
| N4 | Explosion-target count as threat number | 1→4, ≥2→1, weak 8 | atomi's attempt: +92 % on `core` (1.9×) | **rejected with data** |
| N5 | Hanging = quiet move whose piece is blasted cheaply | `blasted_cheaply` + Atomic-Stockfish values | same +92 % (1.9×) | **rejected with data** |
| N6 | `UNLIKELY` prior | not check, not blast-capture, no king contact → `dn=2` | **adopted** (item 26); the fix alone cut 1.31× on `core` | **closed** |
| N7 | History: tie-break only, +1 per prove/refute | two tables | ≈ also scales start numbers | **aligned** |
| N8 | Tie order: deciding number, then history | no class rank | ≈ class rank precedes history | **aligned** |

## 3. Board-free closures and exact replays

| # | Idea | provari spec | atomi state | Gap |
| --- | --- | --- | --- | --- |
| C1 | Explosion settles, either side | `explodes_them(effect)` bit test | ≈ built child + `status` | **parity** (R10) |
| C2 | "Exposing" one-move shortcut | `exposes(m,e)` board-free | **adopted** (item 28, `closure-bits`) | **closed** — aux 5–7.5×, `easy-win` tree 2.52× |
| C3 | Double threat with exposure wins at once | `threats ≥ 2 && exposed(after)` | subsumed by C4 | **closed** (R12) |
| C4 | "Exposed" win in two | every defence lets the king be exploded | in the exact null replay | **closed** (R13) |
| C5 | Mate shortcut on a check child / defender node | `mated(next)` | built-child `status` | **parity** (R14) |
| C6 | Material-draw cut after capture/promotion | `might_end_material` + `cannot_win` | ≈ full `status` per built capture | **parity** (R15) |
| C7 | Null-threat replay, exact, proofs gained | passed position + exact winning move, replayed on every open defence | **adopted** (item 27) | **closed** — part of 5.88×/4.65×/1.66× |
| C8 | Sibling simulation, exact, budget 16 | Kawano replay, keeps intermediate proofs | **adopted** (item 27); `sim-exact` alone 2.35× `easy-win` | **closed** |
| C9 | Plain-move grouping, members never scored | exact bit-test `plain()`, one GROUP child under a salted key | opt-in `--guide-group`; default-on **closed negative** (item 29) | **open** — `deep-win` 0/3 at 120 s vs 3/3 control, `core` 1.045×, `easy-win` 0.863× |

## 4. Memory

| # | Idea | provari spec | atomi state | Gap |
| --- | --- | --- | --- | --- |
| M1 | Don't store recomputable proofs; mate cache in TT; drop mates when proofs fill | `recomputed`, `Known::Mate`, `drop_mates` | no mate cache, no `drop_mates`; proof table refuses at 90 % | **open but not wall** — item 30 triggered; `T_3` resolved by the 75 % proof share (D35) |
| M2 | Proof store: full keys, shortest/higher-limit domination, no eviction, refuse at 90 % | 12-byte entries, 64 MB at 10 s | 16-byte, 75 % share | **parity** (R23) |
| M3 | Table: 4-slot clusters, least-work replacement, sparse filter | | same | **parity** (R24) |
| M4 | Length-limited passes (`--most`, `--shorten`) | off by default | absent | **skipped** (R25) |

## 5. Per-node cost (the dominant remaining gap)

| # | Idea | provari spec | atomi state | Gap |
| --- | --- | --- | --- | --- |
| P1 | Decide a child from the move's *effect*, not a built board | ~1.42 µs/node | ~15.5 µs/expansion | **open — 11.5–15.5×** (D36) |
| P2 | Movegen/legality/terminality on the hot path | ~54 % of atomi's wall is movegen + legality + terminality | | **open** — target of the movegen initiative (MG3–MG7) |
| P3 | Per-node micro-optimisation | | NULL-V1 0.796×, NULL-V2 0.8631×, HOTPATH-V1 1.219× | **closed** — per-node line closed for good (D40) |

## 6. The gap in one place

| Measure | provari | atomi | Gap |
| --- | ---: | ---: | ---: |
| Per-node cost | 1.42 µs/node | 15.5 µs/expansion | **~11×** |
| Tree (expansions) | 342 k (`core`) | at parity (D36: 1.01× / 0.59× / 1.18×) | **~1×** |
| Wall factor, Cap 120 | 1.000 | 13.288 / 3.891 / 17.549 | **geomean 9.681** |
| Rung | — | `T_3` | `T_4` needs `deep-win` **7.17×**; `T_5` needs geomean **< 1.0** |

**The shape of it:** the *tree* ideas are essentially all ported (N1/N3/N6,
C2/C7/C8) and the tree is now at parity. The *per-node* ideas are the ones
that did not transfer — and they are exactly the ones that live in hot loops
we cannot transliterate (GPL). The remaining 11× is a coupled implementation
gap, not a missing idea.

## 7. What it implies

- The next lever is **not** another provari idea: the tree is at parity and
  the per-node line is closed (D40). It is the formulation fork (PDS-PN/PN²
  or a goal re-pin under D30).
- The per-node gap is attacked **upstream** in the movegen initiative (MIT
  crate), not by copying GPL code.
- A row marked **rejected with data** (N2, N4, N5) is a closed question: do
  not re-port it without a new spec and a new gate.
