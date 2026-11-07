// Test-only module: seeded playouts shared by the mover, frame and rules
// differential tests.

//! Shared test fixtures: seeded random legal playouts over the bench,
//! veto and hand-built fixture positions.
//!
//! Each playout starts at a fixture FEN and makes up to 120 random legal
//! moves (xorshift64*, fixed seeds, stop at terminal positions). For every
//! visited position the playout yields `(board, line, attacker)` where
//! `line` is the key history from the playout's start *including* the
//! position's own key (the classifier test's `line`) and `attacker` is the
//! playout's starting side to move.

use crate::rules;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, MoveList};

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

/// The 3 bench FENs (corpus `core`, `1eb80b92f92c`, `db6a741c723f`), the 8
/// veto FENs, and fixtures exercising castling (both colours, both wings,
/// rights lost by a blast), en passant (capturable, uncapturable, pinned,
/// blast-adjacent) and promotions (quiet, capture, checking).
fn fixture_fens() -> Vec<&'static str> {
    vec![
        // bench
        "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
        "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
        "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
        // veto.tsv
        "6rk/8/8/8/8/8/8/6QK w - - 0 1",
        "k7/8/1Q6/8/8/8/8/K6R w - - 0 1",
        "k7/8/8/8/8/7Q/8/K6R w - - 0 1",
        "5R2/3p4/3Bk1p1/6Pp/2p4P/p1N2P2/P1PP4/7K w - - 0 29",
        "k7/8/1Q6/8/8/8/8/K6R w - - 99 1",
        "k7/8/8/8/8/7Q/8/K6R w - - 97 1",
        "r4r1k/3q1P2/pp4pp/2pp4/N4Q2/2P3PP/PP2p3/R4R1K w - - 0 25",
        "rnb1k2r/p2p3p/4ppp1/qp6/3PPQ2/8/PPP2PPP/R3KB1R w KQkq - 2 11",
        // castling: both colours, both wings, blast-lost rights
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
        "r3k2r/p6p/8/8/8/8/P6P/R3K2R w KQkq - 0 1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "r3k2r/1P4P1/8/8/8/8/1p4p1/R3K2R b KQkq - 0 1",
        // en passant: capturable, uncapturable, pinned pawn, blast-adjacent
        "4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2",
        "4k3/8/8/2pP4/8/8/8/4K3 w - - 0 2",
        "4k3/8/8/K1pP3r/8/8/8/8 w - c6 0 2",
        "4k3/8/1b6/2pP4/8/8/8/4K3 w - c6 0 2",
        "4k3/8/8/1pP5/8/8/8/4K3 w - b6 0 2",
        "k7/8/8/1pP5/8/8/8/4K3 w - b6 0 2",
        // promotions: quiet, capture, checking
        "4k3/1P6/8/8/8/8/8/4K3 w - - 0 1",
        "3rk3/1P6/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/1p6/4K3 b - - 0 1",
        "r3k3/1P6/8/8/8/8/8/4K3 w q - 0 1",
        "2r1k3/1P6/8/8/8/8/8/4K3 w - - 0 1",
        "7k/1P6/8/8/8/8/8/4K3 w - - 0 1",
    ]
}

/// One seeded random legal playout: the positions visited, each with its
/// key history and the playout's starting side as the attacker.
fn playout(fen: &str, seed: u64, plies: usize) -> Vec<(Board, Vec<u64>, Color)> {
    let start = Board::from_fen(fen).unwrap();
    let attacker = start.side_to_move();
    let mut rng = Rng(seed | 1);
    let mut board = start;
    let mut line = Vec::new();
    let mut out = Vec::new();
    let mut moves = MoveList::new();
    for _ in 0..plies {
        line.push(rules::key(&board));
        out.push((board.clone(), line.clone(), attacker));
        if rules::status(&board, attacker) != rules::Status::Open {
            break;
        }
        moves.clear();
        generate_legal(&board, &mut moves);
        if moves.is_empty() {
            break;
        }
        let m = moves.as_slice()[(rng.next() as usize) % moves.len()];
        debug_assert!(board.is_legal(m));
        board.do_move(m, &mut StateInfo::new());
    }
    out
}

/// All playout positions over the fixture FENs (≥ 5,000 by construction:
/// ~30 FENs × 5 seeds × up to 120 plies, most non-terminal).
pub(crate) fn positions() -> Vec<(Board, Vec<u64>, Color)> {
    let mut out = Vec::new();
    for (i, fen) in fixture_fens().iter().enumerate() {
        for s in 0..5u64 {
            out.extend(playout(
                fen,
                0x9E37_79B9_7F4A_7C15 ^ (i as u64) << 32 ^ s,
                120,
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playout_moves_are_legal() {
        // Re-run the playouts; generated+selected moves must always be
        // legal for the board they were generated on.
        let fens = fixture_fens();
        for (i, fen) in fens.iter().enumerate() {
            for s in 0..5u64 {
                let start = Board::from_fen(fen).unwrap();
                let attacker = start.side_to_move();
                let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (i as u64) << 32 ^ s | 1);
                let mut board = start;
                let mut moves = MoveList::new();
                for _ in 0..120 {
                    if rules::status(&board, attacker) != rules::Status::Open {
                        break;
                    }
                    moves.clear();
                    generate_legal(&board, &mut moves);
                    if moves.is_empty() {
                        break;
                    }
                    let idx = (rng.next() as usize) % moves.len();
                    let m = moves.as_slice()[idx];
                    assert!(
                        board.is_legal(m),
                        "{fen}: {m} on {}",
                        board.fen_with(atomic_movegen::types::EnPassantMode::Legal)
                    );
                    board.do_move(m, &mut StateInfo::new());
                }
            }
        }
    }
}
