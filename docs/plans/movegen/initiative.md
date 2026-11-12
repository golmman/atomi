# Initiative: `movegen` — upstream performance and API work for `atomic-movegen`

## Status

**Open (2026-11-12).** Created from the `prover` hot-path profile
([`../prover/plan28.md`](../prover/plan28.md)): atomi spends 15.5 µs per
expansion against provari's 1.42 µs per node, and ~54 % of the wall is
movegen, legality and terminality. The crate is MIT; this initiative is the
upstream wishlist, not a fork.

## Goal

Make `atomic-movegen` the fastest correct atomic rules crate for
prover-style workloads, and expose the minimal API a hot-path prover needs,
**without changing rules semantics**.

Success: the prover's S1–S3 stack reaches ≥1.5× per-expansion at an
identical tree, and the published crate is the dependency the prover ships.

## Non-goals

- No provari, shakmaty or other GPL code, dependency, binary or text. Ideas
  only, in our own words (the prover's D12 boundary).
- No rules change without a differential against the previous version and
  the prover's fixture corpus.
- No atomi-specific API that only one caller can use; every item must be
  generally useful.
- No `unsafe` without a measured win and a `// SAFETY:` argument.
- No network, model or tablebase work.

## Relationship to `prover`

The prover initiative consumes the crate. The wishlist is driven by the
prover's `perf` profile and by the `prover` plan that needs the API. The
prover may use a **local checkout** for measurement, but the committed
dependency stays on a published version; a path dependency is never
committed.

## Wishlist

| # | Item | Why (evidence) | API / behaviour | Risk | Acceptance | Priority |
| --- | --- | --- | --- | --- | --- | --- |
| MG1 | `Board::null_for(attacker)` (or `with_side_to_move` + `clear_ep`) | `null_board` does `fen_with` → split/join → `from_fen` every null replay; ~1.6 % direct + allocator | Cheap field mutation; no FEN | low | Differential: same key, side, ep, clock as the FEN path; no rules change | P0 |
| MG2 | Public `for_each_pseudo_legal` (or iterator) | `generate_pseudo_legal` is 21 % of the prover's wall; guide paths materialise a `MoveList` | Callback/iterator in the same order; early exit | low | Same order as `generate_pseudo_legal`; differential | P0 |
| MG3 | `Board::outcome_after(m, state)` / effect-based terminality | Per-child `outcome` runs `has_legal_move`; ~15–20 % of the prover's wall | Decide explosion/check/mate from the move effect; full legal search only where stalemate is possible | **high** | Differential vs `outcome` on the fixture corpus and the three bench cells; no divergence | P1 |
| MG4 | Expand `TrivialLegality` (captures/checks) | `Board::legal` is 8.4 % of the prover's wall; the fast path only covers quiet non-capture, non-check, non-pinned moves | More trivially-legal classes | **high** | Differential vs `legal` on the corpus; no divergence | P2 |
| MG5 | `Board::legal` fast path / precomputed attack info | Same as MG4 | Reuse node-constant attack data | high | Differential | P2 |
| MG6 | `generate_pseudo_legal` micro-optimisation | 21 % of the prover's wall | Bitboard/loop work, no API change | medium | Perft + movegen differential; same order | P2 |
| MG7 | `from_fen`/`fen_with` performance | Only if MG1 does not land | Parser/serialiser | low | Round-trip differential | P3 |

## Rules

- Every change is performance-only unless a rules bug is found; a rules bug
  is a separate item with its own differential.
- The crate's own test suite, perft, and a differential against the previous
  version are the gate. The prover's `make test-full` + veto is the
  integration gate.
- No GPL source is read while writing the code; the prover's profile and the
  atomic rules are the design inputs.
- Measurements are same-session ratios, one process at a time; drivers and
  parsed results live under `measurements/` (never raw transcripts).

## Backlog

| # | Item | Status |
| --- | --- | --- |
| MG1 | `null_for` | proposed (plan28 Phase 0) |
| MG2 | public `for_each_pseudo_legal` | proposed (plan28 Phase 0) |
| MG3 | `outcome_after` | proposed (plan28 Phase D, conditional) |
| MG4 | `TrivialLegality` expansion | proposed (after MG3) |
| MG5 | `legal` fast path | proposed (after MG4) |
| MG6 | movegen micro-optimisation | proposed (after MG2) |
| MG7 | FEN performance | fallback only |

## Decisions

| # | Decision | Resolution |
| --- | --- | --- |
| D-MG1 | Scope | Performance and API only; no rules change without a differential |
| D-MG2 | GPL boundary | Ideas only; no provari/shakmaty code, dependency or text |
| D-MG3 | Release gate | The crate's tests + perft + differential + the prover's `make test-full` + veto |
| D-MG4 | Dependency | The prover ships a published version; a path dependency is measurement-only |

## Risks

| Risk | Mitigation |
| --- | --- |
| A performance change alters rules | Differential against the previous version and the prover's fixture corpus |
| The API is too atomi-specific | Every item must be generally useful; review before release |
| The prover blocks on a release | The prover plan measures with a local checkout; the committed change waits |
| GPL contamination | Ideas only; the licence gate stays in the prover repo |

## History

- **2026-11-12** — initiative created from the `prover` hot-path profile
  ([`../prover/plan28.md`](../prover/plan28.md)); MG1–MG7 proposed.
