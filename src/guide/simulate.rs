//! Bounded deterministic sibling simulation (D21).
//!
//! One greedy playout from a fresh child: at most `SIM_PLIES` plies, at most
//! `SIM_BRANCH` candidate moves per ply, ordered by a cheap static hint. No
//! tables, no events, no line mutation, no proof. Its expansions are counted
//! in `sim_nodes` only.

use crate::guide::eval;
use crate::rules;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, Move, MoveList};

/// Maximum plies of one playout.
pub const SIM_PLIES: u8 = 6;
/// Maximum candidate moves examined per ply (diagnostic bound; the hint
/// scans the legal list).
pub const SIM_BRANCH: usize = 8;

/// A greedy playout score from the attacker's perspective in `[-100, 100]`
/// and the number of made moves (`sim_nodes`).
pub fn playout(board: &Board, attacker: Color) -> (i32, u64) {
    playout_with(board, attacker, SIM_PLIES)
}

/// A playout bounded by `plies` (the plan7 leave-one-out knob).
pub fn playout_with(board: &Board, attacker: Color, plies: u8) -> (i32, u64) {
    let mut board = board.clone();
    let mut nodes = 0u64;
    for ply in 0..plies {
        match rules::status(&board, attacker) {
            rules::Status::Won => return (100 - i32::from(ply), nodes),
            rules::Status::Failed { .. } => return (-100 + i32::from(ply), nodes),
            rules::Status::Open => {}
        }
        let mut moves = MoveList::new();
        generate_legal(&board, &mut moves);
        if moves.is_empty() {
            // The status above decides every moveless position; defensive.
            return (0, nodes);
        }
        let m = choose(&board, &moves, attacker);
        board.do_move(m, &mut StateInfo::new());
        nodes += 1;
    }
    let side_score = eval::hand_eval(&board);
    let score = if board.side_to_move() == attacker {
        side_score
    } else {
        -side_score
    };
    (score / 4, nodes)
}

/// The cheap static hint: captures that blast the enemy commoner first,
/// captures next, then movegen order.
fn hint(board: &Board, m: Move) -> i32 {
    let mut h = 0;
    if board.is_capture(m) {
        h += 10;
        let commoner = board.commoners(board.side_to_move().flip());
        if !commoner.is_empty() && !(Board::blast_zone(m.to_sq()) & commoner).is_empty() {
            h += 100;
        }
    }
    if m.is_promotion() {
        h += 5;
    }
    h
}

/// Pick the highest-hint move, ties in movegen order.
fn choose(board: &Board, moves: &MoveList, _attacker: Color) -> Move {
    let mut best = moves.as_slice()[0];
    let mut best_h = hint(board, best);
    for &m in moves.as_slice().iter() {
        let h = hint(board, m);
        if h > best_h {
            best = m;
            best_h = h;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;

    #[test]
    fn short_playouts_are_bounded() {
        let b = Board::from_fen("6rk/8/8/8/8/8/8/6QK w - - 0 1").unwrap();
        for plies in [0, 1, 2, 3, 6] {
            let (_, n) = playout_with(&b, Color::White, plies);
            assert!(n <= u64::from(plies), "plies={plies} nodes={n}");
        }
    }

    #[test]
    fn playout_is_bounded_and_deterministic() {
        for (b, _line, attacker) in testutil::positions().into_iter().take(50) {
            let (a, na) = playout(&b, attacker);
            let (c, nc) = playout(&b, attacker);
            assert_eq!((a, na), (c, nc));
            assert!(na <= u64::from(SIM_PLIES));
            assert!((-100..=100).contains(&a));
        }
    }

    #[test]
    fn immediate_win_scores_high() {
        let b = Board::from_fen("6rk/8/8/8/8/8/8/6QK w - - 0 1").unwrap();
        let (score, nodes) = playout(&b, Color::White);
        assert!(score > 0, "{score}");
        assert!(nodes <= 1);
    }
}
