//! Exact null-threat replay (plan 18, `--guide-null-exact`).
//!
//! At a defender node the attacker may have a winning move if the defender
//! passed (the "null" position). Provari's `replay_threat` finds such a move,
//! then replays its proof on every still-open defence: a defence that parries
//! the threat starts lower, one that does not is proven outright.
//!
//! The null board is **ephemeral**. It is built from the current FEN with the
//! side to move and the en-passant square replaced, used for one exact ply,
//! and dropped. It is never keyed, looked up, stored, put on `line` or
//! emitted; the position keys and proofs that leave this module are the real
//! children of the current node.
//!
//! This is deliberately narrower than provari's null replay: it finds an
//! immediate explosion/mate, a move into a stored proof, or an exposed
//! mate-in-two (every defence lets the attacker explode). Check-every-evasion,
//! double-threat and `recomputed` proofs are out of scope for this flag.

use crate::guide::threat;
use crate::rules;
use crate::search::value::Value;
use crate::table::Tables;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, EnPassantMode, Move, MoveList};

/// A winning move for the attacker in the null position, with its proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullWin {
    /// The attacker's move if the defender passed.
    pub mv: Move,
    /// Proof length in attacker moves (1 for an immediate win, 2 for an
    /// exposed mate-in-two).
    pub length: u16,
    /// Highest 50-move counter at which the proof holds.
    pub limit: u8,
}

/// Build the null counterpart of `board`: the same pieces with `attacker` to
/// move and no en-passant target. Returns `None` if the FEN cannot be parsed
/// (defensive; the input FEN came from a valid board).
pub fn null_board(board: &Board, attacker: Color) -> Option<Board> {
    let fen = board.fen_with(EnPassantMode::Legal);
    let mut parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }
    parts[1] = if attacker == Color::White { "w" } else { "b" };
    parts[3] = "-";
    Board::from_fen(&parts.join(" ")).ok()
}

/// Find an exact winning move for `attacker` in the null position and return
/// it with the null board and the movegen work spent. `remaining` bounds the
/// proof length.
pub fn null_win(
    board: &Board,
    attacker: Color,
    tables: &Tables,
    remaining: u16,
) -> Option<(Board, NullWin, u64)> {
    if board.side_to_move() == attacker {
        return None;
    }
    let null = null_board(board, attacker)?;
    let mut moves = MoveList::new();
    generate_legal(&null, &mut moves);
    // Pass 1: a direct win or a stored proof (shorter proofs first).
    for &m in moves.as_slice() {
        let mut child = null.clone();
        child.do_move(m, &mut StateInfo::new());
        if rules::status(&child, attacker) == rules::Status::Won {
            return Some((
                null,
                NullWin {
                    mv: m,
                    length: 1,
                    limit: 99,
                },
                u64::from(moves.len() as u32),
            ));
        }
        let key = rules::key(&child);
        let counter = child.rule50();
        if let Some(Value::Proved { length, limit, .. }) = tables.lookup(key, counter)
            && length <= remaining
        {
            let limit = if child.rule50() == 0 {
                99
            } else {
                limit.saturating_sub(1).min(99)
            };
            return Some((
                null,
                NullWin {
                    mv: m,
                    length: length.saturating_add(1),
                    limit,
                },
                u64::from(moves.len() as u32),
            ));
        }
    }
    // Pass 2: every defence exposes the commoner (a mate in two).
    for &m in moves.as_slice() {
        let mut child = null.clone();
        child.do_move(m, &mut StateInfo::new());
        if let Some(limit) = exposed_limit(&child, attacker) {
            return Some((
                null,
                NullWin {
                    mv: m,
                    length: 2,
                    limit,
                },
                u64::from(moves.len() as u32),
            ));
        }
    }
    None
}

/// If every legal defence from `child` (defender to move) lets `attacker`
/// explode the defender's commoner at once, return the proof limit.
fn exposed_limit(child: &Board, attacker: Color) -> Option<u8> {
    if child.rule50() >= 100 {
        return None;
    }
    let mut replies = MoveList::new();
    generate_legal(child, &mut replies);
    if replies.is_empty() {
        return None;
    }
    let mut limit = 99u8;
    for &reply in replies.as_slice() {
        let mut next = child.clone();
        next.do_move(reply, &mut StateInfo::new());
        if next.rule50() >= 100 {
            return None;
        }
        match rules::status(&next, attacker) {
            rules::Status::Won => {}
            rules::Status::Open => {
                threat::immediate_explosion(&next, attacker)?;
            }
            rules::Status::Failed { .. } => return None,
        }
        if next.rule50() == 0 {
            limit = 98;
        }
    }
    Some(limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(fen: &str) -> Board {
        Board::from_fen(fen).unwrap()
    }

    #[test]
    fn null_board_flips_the_mover_and_clears_ep() {
        let b = board("4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2");
        let n = null_board(&b, Color::Black).expect("null board");
        assert_eq!(n.side_to_move(), Color::Black);
        assert_eq!(n.legal_ep_square(), None);
    }

    #[test]
    fn immediate_explosion_is_found() {
        // Black to move; White's rook on a1 can take a7 and blast the black
        // commoner on a8 if White were to move.
        let b = board("k7/p7/8/8/8/8/8/R3K3 b - - 0 1");
        let tables = Tables::new(16, true);
        let (null, win, _) = null_win(&b, Color::White, &tables, u16::MAX).expect("null win");
        assert_eq!(win.length, 1);
        let mut after = null.clone();
        after.do_move(win.mv, &mut StateInfo::new());
        assert_eq!(rules::status(&after, Color::White), rules::Status::Won);
    }

    #[test]
    fn a_stored_null_child_proof_is_a_win() {
        // White to move in the null position; the rook move into the stored
        // child proof is the only win found (the position has no immediate
        // explosion and no exposed mate in two).
        let b = board("7k/7p/8/8/8/8/8/R3K3 b - - 0 1");
        let mut tables = Tables::new(16, true);
        let null = null_board(&b, Color::White).unwrap();
        let mv = null.move_from_uci("a1a8").unwrap();
        let mut child = null.clone();
        child.do_move(mv, &mut StateInfo::new());
        let key = rules::key(&child);
        tables.store(
            key,
            &Value::Proved {
                length: 1,
                limit: 99,
                mv: None,
            },
            0,
        );
        let (_, win, _) = null_win(&b, Color::White, &tables, u16::MAX).expect("null win");
        assert_eq!((win.mv, win.length), (mv, 2));
    }

    #[test]
    fn no_null_win_on_a_quiet_position() {
        let b = board("4k3/8/8/8/8/8/8/4K3 b - - 0 1");
        let tables = Tables::new(16, true);
        assert!(null_win(&b, Color::White, &tables, u16::MAX).is_none());
    }

    #[test]
    fn a_null_board_is_never_keyed_into_the_tables() {
        // The null board is built and dropped: the tables stay empty.
        let b = board("k7/p7/8/8/8/8/8/R3K3 b - - 0 1");
        let tables = Tables::new(16, true);
        let null = null_board(&b, Color::White).unwrap();
        assert!(tables.lookup(rules::key(&null), null.rule50()).is_none());
    }
}
