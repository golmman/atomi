//! Plan 31 (item 38) Phase A — the existence differential for the one-ply
//! explosion test and the null-board identity.
//!
//! Two tree-neutral rewrites are checked against the paths they replace, never
//! against themselves:
//!
//! - **F1 `threat::has_explosion`** (no move list, no child) against
//!   `threat::immediate_explosion` (the shipped list path) — *existence only*.
//!   The witness functions are untouched; a boolean disagreement here is a
//!   tree change, and a false `true` would be a false win.
//! - **F2 `guide::null::null_board`** (`Board::null_for`) against the FEN
//!   round-trip (`fen_with(Legal)` → swap the side → clear ep → `from_fen`):
//!   equal FEN text, key, half-move clock, fullmove number and game ply.
//!
//! Corpus: the frozen fixture positions, the three bench cells, the veto
//! fixture positions, hand-built cases for every shape the fast path must get
//! right (promotion captures, en passant, a capture outside the blast zone, a
//! move that would leave our commoner in the blast, a commoner capture), and
//! ≥ 50,000 positions from seeded random play-outs of the three bench FENs —
//! positions the fixtures never reach, both sides to move.

mod common;

use atomi::guide::threat::{has_explosion, immediate_explosion};
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, EnPassantMode, MoveList};

/// xorshift64* — deterministic, no dependencies.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// The three bench cells (`bench/corpus.json`).
const BENCH: &[&str] = &[
    "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
];

/// Hand cases: `(fen, side to move, expected existence, what it pins)`.
const HAND: &[(&str, bool, &str)] = &[
    (
        // The pawn on b7 captures on c8 (a promotion capture) and the blast
        // reaches the black commoner on b8: the promotion fallback decides.
        "1kn5/1P6/8/8/8/8/8/7K w - - 0 1",
        true,
        "promotion capture that explodes the commoner",
    ),
    (
        // Same shape, no commoner in the blast ring of c8: the promoted piece
        // is irrelevant because the blast is the same for every choice.
        "2n5/1P5k/8/8/8/8/8/7K w - - 0 1",
        false,
        "promotion capture outside the blast zone",
    ),
    (
        // d5xc6 e.p.: the victim is not on the destination square, so the
        // en-passant fallback decides; the blast at c6 reaches b7.
        "8/1k6/8/2pP4/8/8/8/7K w - c6 0 2",
        true,
        "en passant explosion",
    ),
    (
        // The same geometry with the black commoner far away.
        "6k1/8/8/2pP4/8/8/8/7K w - c6 0 2",
        false,
        "en passant without a commoner in the blast",
    ),
    (
        // Target a7 is in the commoner's zone but the blast at a7 leaves the
        // commoner alive: a knight on a7, commoner on a8, capture from a1.
        "k7/r7/8/8/8/8/8/R6K w - - 0 1",
        true,
        "rook capture beside the commoner",
    ),
    (
        // No enemy piece inside the commoner's zone: nothing to capture.
        "k7/8/8/8/8/8/2r5/K6R w - - 0 1",
        false,
        "capture outside the zone",
    ),
    (
        // The only candidate (a3xa7) would leave our own commoner on b6 in
        // the blast: `Board::legal` must reject it.
        "k7/r7/1K6/8/8/R7/8/8 w - - 0 1",
        false,
        "capture that self-explodes our commoner",
    ),
    (
        // The only geometric candidate is our own commoner (a6xa7, whose
        // blast would reach the black commoner on a8). A commoner capture is
        // never legal — the capturer is destroyed at ground zero — so the
        // existence test masks commoner attackers and answers `false`, exactly
        // like the legal move list (which does not contain a6a7).
        "k7/p7/K7/8/8/8/8/8 w - - 0 1",
        false,
        "commoner capture is never a legal explosion",
    ),
    (
        // A pinned attacker: the bishop on d3 is pinned along the d-file by
        // the rook on d8, and d3xe4 would blast the commoner on f5. This
        // movegen's legality does **not** prune it (the move list contains
        // it), so the existence test must not invent a pin rule either: the
        // exact `Board::legal` is the only legality source.
        "3r4/8/8/5k2/4p3/3B4/8/3K4 w - - 0 1",
        true,
        "pinned attacker is decided by the blast, not by the pin",
    ),
    (
        // A piece shielding the commoner: b1xb7 blasts b8.
        "1k6/1p6/8/8/8/8/8/1RK5 w - - 0 1",
        true,
        "capture of the piece shielding the commoner",
    ),
    (
        // The attacker's own commoner is gone: the movegen emits nothing, so
        // no capture is a legal win even though the geometry is there.
        "k7/8/8/8/8/8/8/R7 w - - 0 1",
        false,
        "extinction: the attacker has no commoner",
    ),
];

fn board(fen: &str) -> Board {
    Board::from_fen(fen).expect("test FEN parses")
}

/// One seeded random legal play-out of `fen`: the positions visited.
fn playout(fen: &str, seed: u64, plies: usize, out: &mut Vec<Board>) {
    let mut board = board(fen);
    let mut rng = Rng(seed | 1);
    let mut moves = MoveList::new();
    for _ in 0..plies {
        out.push(board.clone());
        if board.commoners(Color::White).is_empty() || board.commoners(Color::Black).is_empty() {
            break;
        }
        moves.clear();
        generate_legal(&board, &mut moves);
        if moves.is_empty() {
            break;
        }
        let m = moves.as_slice()[(rng.next() as usize) % moves.len()];
        board.do_move(m, &mut StateInfo::new());
    }
}

/// The frozen movegen fixture positions (the shared corpus reader).
fn frozen_fens() -> Vec<String> {
    common::corpus().0.into_iter().map(|(_, p)| p.fen).collect()
}

/// A FEN from `(square, piece)` pairs (`a1 = 0`, `h8 = 63`).
fn fen_from(pieces: &[(u8, char)], side: char) -> String {
    let mut rows = Vec::new();
    for rank in (0..8).rev() {
        let mut row = String::new();
        let mut empty = 0;
        for file in 0..8u8 {
            let sq = rank * 8 + file;
            match pieces.iter().find(|(s, _)| *s == sq) {
                Some((_, p)) => {
                    if empty > 0 {
                        row.push_str(&empty.to_string());
                        empty = 0;
                    }
                    row.push(*p);
                }
                None => empty += 1,
            }
        }
        if empty > 0 {
            row.push_str(&empty.to_string());
        }
        rows.push(row);
    }
    format!("{} {side} - - 0 1", rows.join("/"))
}

/// The squares in `sq`'s blast zone that are on the board.
fn zone_squares(sq: u8) -> Vec<u8> {
    let (f, r) = (sq % 8, sq / 8);
    let mut out = Vec::new();
    for dr in -1i8..=1 {
        for df in -1i8..=1 {
            let (nf, nr) = (f as i8 + df, r as i8 + dr);
            if (0..8).contains(&nf) && (0..8).contains(&nr) {
                out.push(nr as u8 * 8 + nf as u8);
            }
        }
    }
    out
}

/// Promotion-capture sweep: every pawn file, both promotion destinations
/// (the two capture diagonals), both victim types and every placement of the
/// enemy commoner in the blast zone — the case the existence test hands back
/// to the list path. Mirrored for Black.
fn promotion_sweep(out: &mut Vec<String>, count: &mut usize) {
    for white in [true, false] {
        let (pawn, enemy_commoner, own_commoner, victim) = if white {
            ('P', 'k', 'K', 'n')
        } else {
            ('p', 'K', 'k', 'N')
        };
        let (from_rank, to_rank, home) = if white {
            (6u8, 7u8, 7u8)
        } else {
            (1u8, 0u8, 0u8)
        };
        for file in 0..8u8 {
            let from = from_rank * 8 + file;
            for df in [-1i8, 1] {
                let fd = file as i8 + df;
                if !(0..8).contains(&fd) {
                    continue;
                }
                let to = to_rank * 8 + fd as u8;
                for (vi, v) in [victim, 'r'].iter().enumerate() {
                    for commoner in zone_squares(to) {
                        if commoner == to {
                            continue;
                        }
                        let own = home * 8;
                        if commoner == own || commoner == from || commoner == to {
                            continue;
                        }
                        let pieces = [
                            (from, pawn),
                            (to, *v),
                            (commoner, enemy_commoner),
                            (own, own_commoner),
                        ];
                        let fen = fen_from(&pieces, if white { 'w' } else { 'b' });
                        if Board::from_fen(&fen).is_err() {
                            continue;
                        }
                        if vi == 0 {
                            *count += 1;
                        }
                        out.push(fen);
                    }
                }
            }
        }
    }
}

/// En-passant sweep: a pawn next to a double-pushed pawn, the ep target, and
/// every placement of the enemy commoner in the target's blast zone.
fn en_passant_sweep(out: &mut Vec<String>, count: &mut usize) {
    for white in [true, false] {
        let (pawn, enemy_pawn, enemy_commoner, own_commoner) = if white {
            ('P', 'p', 'k', 'K')
        } else {
            ('p', 'P', 'K', 'k')
        };
        let (pawn_rank, enemy_rank, ep_rank, home, side) = if white {
            (4u8, 4u8, 5u8, 0u8, 'w')
        } else {
            (3u8, 3u8, 2u8, 7u8, 'b')
        };
        for file in 0..8u8 {
            for df in [-1i8, 1] {
                let ef = file as i8 + df;
                if !(0..8).contains(&ef) {
                    continue;
                }
                let from = pawn_rank * 8 + file;
                let victim = enemy_rank * 8 + ef as u8;
                let to = ep_rank * 8 + ef as u8;
                let own = home * 8 + 4;
                for commoner in zone_squares(to) {
                    if commoner == own || commoner == to || commoner == from {
                        continue;
                    }
                    let mut rows = vec![
                        (from, pawn),
                        (victim, enemy_pawn),
                        (commoner, enemy_commoner),
                        (own, own_commoner),
                    ];
                    let fen = {
                        let mut f = fen_from(&rows, side);
                        // Replace the ep field: `-` becomes the target square.
                        let mut parts: Vec<String> =
                            f.split_whitespace().map(str::to_string).collect();
                        let name =
                            format!("{}{}", (b'a' + (to % 8)) as char, (b'1' + to / 8) as char);
                        parts[3] = name;
                        f = parts.join(" ");
                        f
                    };
                    if Board::from_fen(&fen).is_err() {
                        continue;
                    }
                    rows.clear();
                    *count += 1;
                    out.push(fen);
                }
            }
        }
    }
}

/// The whole corpus: fixtures + hand cases + seeded play-outs of the three
/// bench FENs (≥ 50,000 positions by construction).
fn corpus() -> Vec<Board> {
    let mut out: Vec<Board> = Vec::new();
    for fen in frozen_fens() {
        out.push(board(&fen));
    }
    for (fen, _, _) in HAND {
        out.push(board(fen));
    }
    let mut sweep: Vec<String> = Vec::new();
    let mut promotions = 0usize;
    let mut passes = 0usize;
    promotion_sweep(&mut sweep, &mut promotions);
    en_passant_sweep(&mut sweep, &mut passes);
    eprintln!("fallback sweep: {promotions} promotion-capture, {passes} en-passant");
    assert!(promotions >= 50, "promotion sweep too small: {promotions}");
    assert!(passes >= 50, "en-passant sweep too small: {passes}");
    for fen in &sweep {
        out.push(board(fen));
    }
    for (i, fen) in BENCH.iter().enumerate() {
        for seed in 0..400u64 {
            playout(
                fen,
                0xB5AD_4ECE_9E37_79B9 ^ (i as u64) << 40 ^ seed,
                120,
                &mut out,
            );
        }
    }
    out
}

/// What the differential saw, for the report line.
#[derive(Debug, Default)]
struct Tally {
    positions: usize,
    hits: usize,
    ep_positions: usize,
    promotion_capture_positions: usize,
    children: usize,
    child_hits: usize,
}

#[test]
fn has_explosion_matches_the_list_path() {
    let mut tally = Tally::default();
    for (i, b) in corpus().into_iter().enumerate() {
        let side = b.side_to_move();
        let fast = has_explosion(&b, side);
        let list = immediate_explosion(&b, side).is_some();
        assert_eq!(
            fast,
            list,
            "existence mismatch on fen={} side={side:?} ep={:?}",
            b.fen_with(EnPassantMode::Legal),
            b.legal_ep_square()
        );
        tally.positions += 1;
        tally.hits += usize::from(fast);
        tally.ep_positions += usize::from(b.legal_ep_square().is_some());
        // Every legal child of a sample: the corpus above is node-shaped, the
        // children are where the null replay asks the question.
        if i % 25 == 0 {
            let mut moves = MoveList::new();
            generate_legal(&b, &mut moves);
            for &m in moves.as_slice() {
                let mut child = b.clone();
                child.do_move(m, &mut StateInfo::new());
                let side = child.side_to_move();
                let fast = has_explosion(&child, side);
                let list = immediate_explosion(&child, side).is_some();
                assert_eq!(
                    fast,
                    list,
                    "child existence mismatch: {} on {}",
                    m.to_uci(),
                    b.fen_with(EnPassantMode::Legal)
                );
                tally.children += 1;
                tally.child_hits += usize::from(fast);
            }
            // A promotion capture whose blast reaches the enemy commoner is
            // the case the fast path hands back to the list.
            let enemy = b.commoners(side.flip());
            if !enemy.is_empty() {
                let mut moves = MoveList::new();
                generate_legal(&b, &mut moves);
                for &m in moves.as_slice() {
                    if m.is_promotion()
                        && b.is_capture(m)
                        && !(b.blast_casualties(m).all() & enemy).is_empty()
                    {
                        tally.promotion_capture_positions += 1;
                        break;
                    }
                }
            }
        }
    }
    eprintln!("has_explosion differential: {tally:?}");
    assert!(
        tally.positions >= 50_000,
        "corpus too small: {} positions",
        tally.positions
    );
    assert!(tally.children >= 10_000, "child sample: {}", tally.children);
    assert!(tally.hits >= 1_000, "only {} node hits", tally.hits);
    assert!(tally.child_hits >= 1, "no child hit");
    assert!(
        tally.ep_positions >= 100,
        "only {} ep positions",
        tally.ep_positions
    );
    assert!(
        tally.promotion_capture_positions >= 1,
        "no promotion-capture position in the sample"
    );
}

#[test]
fn hand_cases_pin_every_shape() {
    for (fen, expected, what) in HAND {
        let b = board(fen);
        let side = b.side_to_move();
        assert_eq!(has_explosion(&b, side), *expected, "{what}: {fen}");
        assert_eq!(
            immediate_explosion(&b, side).is_some(),
            *expected,
            "{what} (list path): {fen}"
        );
    }
}

#[test]
fn null_board_matches_the_fen_round_trip() {
    let mut checked = 0usize;
    let mut with_ep = 0usize;
    for b in corpus().into_iter().step_by(3) {
        for attacker in [Color::White, Color::Black] {
            let fast = atomi::guide::null::null_board(&b, attacker);
            let fen = b.fen_with(EnPassantMode::Legal);
            let mut parts: Vec<&str> = fen.split_whitespace().collect();
            parts[1] = if attacker == Color::White { "w" } else { "b" };
            parts[3] = "-";
            let slow = Board::from_fen(&parts.join(" ")).expect("round trip");
            assert_eq!(
                fast.fen_with(EnPassantMode::Legal),
                slow.fen_with(EnPassantMode::Legal),
                "null FEN differs for {fen} (attacker {attacker:?})"
            );
            assert_eq!(
                fast.hash_with(EnPassantMode::Legal),
                slow.hash_with(EnPassantMode::Legal),
                "null key differs for {fen}"
            );
            assert_eq!(fast.rule50(), slow.rule50(), "null clock differs for {fen}");
            assert_eq!(
                fast.fullmove_number(),
                slow.fullmove_number(),
                "null fullmove differs for {fen}"
            );
            assert_eq!(
                fast.game_ply(),
                slow.game_ply(),
                "null ply differs for {fen}"
            );
            assert_eq!(fast.side_to_move(), attacker);
            checked += 1;
            with_ep += usize::from(b.legal_ep_square().is_some());
        }
    }
    eprintln!("null board differential: {checked} pairs, {with_ep} with ep");
    assert!(checked >= 10_000, "only {checked} null boards");
    assert!(with_ep >= 100, "only {with_ep} en-passant boards");
}
