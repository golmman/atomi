# atomic-movegen upstream handoff — hot-path API prototypes and wishlist

**Status:** self-contained handoff for the upstream `atomic-movegen` repository.
It records two measured, additive API prototypes (MG1, MG2), the evidence for
them, and the remaining performance wishlist (MG3–MG7). Nothing here changes
atomic rules. The prototypes are **not released** and no upstream PR has been
opened.

**Baseline:** `atomic-movegen 2.4.0` (MIT). **Date:** 2026-11-12.
**Origin:** a downstream atomic-win prover (`atomi`) profiled its hot path and
prototyped the two upstream changes in a local checkout. The consumer did not
adopt the full stack because its own 1.5× gate was missed; that is a consumer
decision, not a correctness failure. The upstream value is the API and the
removed work, not a claimed 1.5×.

---

## 1. What is ready for upstream review

### MG1 — `Board::null_for(attacker)`

**Problem.** A null-move replay needs the same position with the other side to
move and no en-passant target. The consumer did this by serialising with
`fen_with(EnPassantMode::Legal)`, splitting/joining the FEN fields, and calling
`from_fen`. That allocates and reparses on every null replay.

**Prototype.** Add to `src/board.rs`, next to the clock accessors:

```rust
/// Return the null counterpart of this position: the same pieces and
/// castling rights with `attacker` to move and no en-passant target.
///
/// This is the cheap in-place equivalent of serialising the position with
/// [`fen_with(EnPassantMode::Legal)`](Self::fen_with), replacing the side
/// and en-passant fields, and parsing it again. The half-move clock is
/// preserved; the fullmove number is preserved, so the derived game ply
/// follows the new side to move exactly as the FEN round-trip would.
///
/// The result is an ordinary [`Board`]: callers that need an ephemeral
/// null position must still keep it out of any keyed or stored state.
#[must_use]
pub fn null_for(&self, attacker: Color) -> Board {
    let mut board = self.clone();
    board.side_to_move = attacker;
    board.ep_square = None;
    board.game_ply = 2 * (board.fullmove_number() - 1)
        + if attacker == Color::White { 1 } else { 2 };
    board.recompute_hash();
    board
}
```

`recompute_hash` is already crate-private. The method preserves pieces,
castling rights, `rule50`, and the fullmove number; it clears the stored ep
target and recomputes the Zobrist hash. It does not touch any keyed or stored
state.

**Differential test.** The prototype adds `tests/null_for.rs` (and a `[[test]]`
entry because the crate sets `autotests = false`). It compares `null_for`
against the old FEN round-trip for both attackers on six fixtures, including a
legal ep target, a stale ep target, a 99-ply clock, and castling rights:

```rust
fn fen_null(board: &Board, attacker: Color) -> Board {
    let fen = board.fen_with(EnPassantMode::Legal);
    let mut parts: Vec<&str> = fen.split_whitespace().collect();
    parts[1] = if attacker == Color::White { "w" } else { "b" };
    parts[3] = "-";
    Board::from_fen(&parts.join(" ")).unwrap()
}

#[test]
fn null_for_matches_the_fen_round_trip() {
    let fens = [
        "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
        "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
        "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
        "4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2",
        "4k3/8/8/8/8/8/8/4K3 b - - 99 40",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 7 12",
    ];
    for fen in fens {
        let board = Board::from_fen(fen).unwrap();
        for attacker in [Color::White, Color::Black] {
            let got = board.null_for(attacker);
            let want = fen_null(&board, attacker);
            assert_eq!(got.fen(), want.fen(), "{fen} {attacker:?}");
            assert_eq!(got.hash(), want.hash(), "{fen} {attacker:?}");
            assert_eq!(
                got.hash_with(EnPassantMode::Legal),
                want.hash_with(EnPassantMode::Legal)
            );
            assert_eq!(got.side_to_move(), want.side_to_move());
            assert_eq!(got.ep_square(), want.ep_square());
            assert_eq!(got.rule50(), want.rule50());
            assert_eq!(got.game_ply(), want.game_ply());
            assert_eq!(got.castling_rights(), want.castling_rights());
        }
    }
}
```

The upstream `cargo test --release` suite, including the golden move-order and
perft tests, stayed green with this change.

### MG2 — public `for_each_pseudo_legal`

**Problem.** `generate_pseudo_legal` always materialises a `MoveList`. A
consumer that only needs the first accepted candidate (for example, a threat
scan or a “has any legal move” check) pays for the whole list.

**Prototype.** The existing internal callback was made public; its body and
order are unchanged:

```rust
/// Walk the pseudo-legal candidate stream for the side to move, invoking `f`
/// on each candidate in generation order and stopping early if `f` returns
/// [`ControlFlow::Break`].
///
/// The walk visits the same stages in the same order as
/// [`generate_pseudo_legal`], which is a thin wrapper collecting the whole
/// stream into a [`MoveList`]: pawn moves (single/double pushes, captures,
/// promotions, en passant, per pawn) → knights → bishops → rooks → queens →
/// commoners → castling.
pub fn for_each_pseudo_legal<B>(
    board: &Board,
    f: &mut impl FnMut(Move) -> ControlFlow<B>,
) -> ControlFlow<B> {
    // existing body
}
```

`generate_pseudo_legal` remains the collecting wrapper. The callback is
read-only with respect to the board; it must not mutate the board while the
stream is being walked.

**Tests.** The existing golden move-order and perft suites are the order gate.
Add a direct differential and an early-exit test:

```rust
let mut list = MoveList::new();
generate_pseudo_legal(&board, &mut list);

let mut stream = Vec::new();
for_each_pseudo_legal::<()>(&board, &mut |m| {
    stream.push(m);
    ControlFlow::Continue(())
});
assert_eq!(list.as_slice(), stream.as_slice());

let mut seen = 0;
let flow = for_each_pseudo_legal::<()>(&board, &mut |_| {
    seen += 1;
    ControlFlow::Break(())
});
assert!(flow.is_break());
assert_eq!(seen, 1);
```

The upstream suite stayed green with the visibility change.

---

## 2. Evidence

The prototypes were measured in the downstream prover on a 5,000,000-child-eval
probe of one `deep-win` position. The full candidate stack was:

- **S1** — consumer uses `Board::null_for` (MG1).
- **S2** — consumer routes two guide scans through `for_each_pseudo_legal`
  (MG2), with early exit.
- **S3** — consumer-side lean proof/TT lookup; **not an upstream change**.

Three repeats per build, alternating, one process at a time. The tree was
identical in every run.

| Build | `child_evals` | Expansions | Median wall | Ratio to control | Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| control, published 2.4.0 | 5,000,034 | 168,910 | 2.558 s | 1.0000 | 1.000× |
| S3 only | 5,000,034 | 168,910 | 2.560 s | 1.0008 | 0.999× |
| S1+S2+S3 candidate | 5,000,034 | 168,910 | 2.099 s | **0.8206** | **1.219×** |

The consumer’s gate was ≤ 0.667× (≥ 1.5×), so it did not adopt the stack. S3
alone was neutral; **MG1+MG2 alone were not separately timed**, so the 1.219×
is the full-stack number, not an upstream-only claim.

A 4 s `perf` self-time profile showed the expected movement:

| Symbol | Control | Candidate |
| --- | ---: | ---: |
| `generate_pseudo_legal` | 20.7 % | folded into the caller |
| `from_fen` | 1.1 % | absent |
| `Board::legal` | 9.3 % | 9.9 % |
| `Board::do_move` | 6.5 % | 6.5 % |
| `Board::compute_checkers` | 4.7 % | 6.1 % |
| `immediate_explosion_bits` | 6.8 % | 14.0 % |

The FEN round-trip and the materialised guide move lists disappear. What
remains is the per-child board/legality/terminality path — the subject of
MG3–MG5, not of MG1/MG2.

**Interpretation.** MG1 and MG2 are low-risk, additive, and remove real work
from a hot path. They are not a complete answer to per-node cost. Upstream
should review them on API merit and on its own benchmarks, not on the
consumer’s missed 1.5× gate.

---

## 3. Remaining wishlist (not prototyped)

| ID | Proposal | Why | Risk | Status |
| --- | --- | --- | --- | --- |
| MG3 | `Board::outcome_after(m, state)` / effect-based terminality | Per-child `outcome` runs `has_legal_move`; ~15–20 % of the consumer’s wall | **high** | proposed; not started |
| MG4 | Expand `TrivialLegality` to captures/checks | `Board::legal` is ~9 % of the consumer’s wall; the fast path covers only quiet non-capture, non-check, non-pinned moves | **high** | proposed |
| MG5 | `Board::legal` fast path / precomputed attack info | Reuse node-constant attack data | high | proposed |
| MG6 | `generate_pseudo_legal` micro-optimisation | Still the largest single movegen symbol | medium | proposed after MG2 |
| MG7 | `from_fen` / `fen_with` performance | Fallback only if MG1 does not land | low | fallback |

MG3 is the highest-leverage next item, but it is a correctness surface: decide
explosion/check/mate from the move effect and fall back to a full legal search
only where stalemate is possible. It needs a differential against `outcome` on
a fixture corpus and on the consumer’s bench cells before it can be trusted.
MG4/MG5 are similarly correctness-sensitive. MG6 is performance-only and
should keep the existing move order.

---

## 4. Upstream integration checklist

- [ ] Apply MG1 and MG2 as additive API; no rules or perft change.
- [ ] Add `tests/null_for.rs` and register it with `[[test]]` (the crate sets
      `autotests = false`).
- [ ] Add the movegen stream differential and early-exit test.
- [ ] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo doc`,
      `cargo test --release`.
- [ ] `cargo run --release --example verify_perft 3`; run depth 6 if time
      allows.
- [ ] Changelog under **Added**; additive API suggests a minor release
      (`2.5.0`), not a patch.
- [ ] README feature bullets for `Board::null_for` and
      `for_each_pseudo_legal`.
- [ ] No `unsafe`, no new dependency, no path dependency in the published
      crate.

Suggested PR title: `Add Board::null_for and public for_each_pseudo_legal`.

---

## 5. Risks and open questions

- **Naming.** `null_for` is domain-specific but short. Alternatives:
  `with_side_to_move_and_no_ep`, `null_move_board`. Decide before release;
  renaming later is a breaking change.
- **`null_for` invariants.** It preserves `rule50` and the fullmove number and
  clears ep, matching the FEN round-trip. If `Board` gains more cached state,
  this method must update it; keep the differential test as the guard.
- **Order contract.** `for_each_pseudo_legal` exposes the existing generation
  order. Document that order and keep the golden test. The callback must not
  mutate the board.
- **Callback shape.** `FnMut(Move) -> ControlFlow<B>` is allocation-free and
  supports early exit. An iterator would be more ergonomic but adds a type and
  may change codegen; the callback is the measured form.
- **Re-export.** The function is reachable as
  `atomic_movegen::movegen::for_each_pseudo_legal`. A crate-root re-export is
  optional; decide based on the existing public-module style.
- **No upstream benchmark.** The 1.219× is a downstream workload. If upstream
  wants a performance claim, add a small benchmark or ask the consumer to
  re-run after release.

---

## 6. Measurement provenance

- Host: AMD Ryzen 9 5950X, 4 vCPU container, Linux; rustc 1.99.1.
- Baseline: published `atomic-movegen 2.4.0`; candidate: local checkout with
  MG1+MG2 reached through a measurement-only path dependency. The path
  dependency was never committed.
- Workload: downstream prover, `deep-win` FEN
  `rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1`,
  `--evals 5000000 --seed 0 --guide hand --memory 192`, release build with
  LTO, `codegen-units = 1`, `panic = "abort"`.
- Three repeats per build, alternating, one process at a time. Counters:
  `child_evals` 5,000,034; `expansions` 168,910; `null_calls` 105,120;
  `null_replies` 2,782,739 — identical for control and candidate.
- No third-party code was copied. The prototypes are original changes to the
  MIT-licensed crate.
