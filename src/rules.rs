// Justification for >10 KB: the attacker-rules layer is one contract
// (status, keys, the checker material rule and their tests).

//! Attacker-only rules layer over `atomic_movegen`.
//!
//! The attacker is the root's side to move for the whole search. A position's
//! [`Status`] is `Won` (the attacker wins from here), `Failed` (the attacker
//! cannot win under the format's rules — repetition and the 50-move limit are
//! never wins), or `Open`.
//!
//! Position identity is `Board::hash_with(EnPassantMode::Legal)` with no
//! clock (decision D4): the checker treats positions as repeated under the
//! legal-en-passant identity.

use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::types::{Bitboard, Color, EnPassantMode, Move, Outcome, PieceType};

/// Attacker-perspective status of a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The attacker wins from this position.
    Won,
    /// The attacker cannot win. `from` is the lowest 50-move counter at which
    /// the refutation holds (`0` = any counter; `100` for a failure at
    /// `rule50 >= 100`).
    Failed { from: u8 },
    /// Undecided.
    Open,
}

/// The status of `board` for `attacker`.
///
/// `Board::outcome()` decides first (one rules precedence, 2.4.0):
/// - `Some(Loss)`: the side to move lost — `Won` when it is the defender,
///   `Failed` when it is the attacker;
/// - `Some(Win)`: the opponent has no commoner — `Won` when the opponent is
///   the defender, `Failed` otherwise;
/// - `Some(Draw)`: `Failed` (stalemate, K vs K, the clock);
/// - `None`: `Failed` when [`material_draw`] holds, else `Open`.
///
/// `from` is `100` whenever `board.rule50() >= 100` and `0` otherwise, for
/// every `Failed` — including a stalemate or an attacker-mated position at
/// the clock, which `outcome()` does not distinguish from the clock; that is
/// conservative because the entry is only reused at counter 100 or more.
pub fn status(board: &Board, attacker: Color) -> Status {
    let from = if board.rule50() >= 100 { 100 } else { 0 };
    let stm = board.side_to_move();
    match board.outcome() {
        Some(Outcome::Loss) => {
            if stm == attacker {
                Status::Failed { from }
            } else {
                Status::Won
            }
        }
        Some(Outcome::Win) => {
            if stm.flip() == attacker {
                Status::Failed { from }
            } else {
                Status::Won
            }
        }
        Some(_) => Status::Failed { from },
        None if material_draw(board) => Status::Failed { from },
        None => Status::Open,
    }
}

/// The status of `board` for `attacker`, reusing a populated `state` and the
/// caller's legal-move existence bit (the movegen it ran anyway). Identical
/// mapping, precedence and `from` to [`status`].
pub fn status_with(board: &Board, attacker: Color, state: &StateInfo, moves_exist: bool) -> Status {
    let from = if board.rule50() >= 100 { 100 } else { 0 };
    let stm = board.side_to_move();
    match board.outcome_with(state, moves_exist) {
        Some(Outcome::Loss) => {
            if stm == attacker {
                Status::Failed { from }
            } else {
                Status::Won
            }
        }
        Some(Outcome::Win) => {
            if stm.flip() == attacker {
                Status::Failed { from }
            } else {
                Status::Won
            }
        }
        Some(_) => Status::Failed { from },
        None if material_draw(board) => Status::Failed { from },
        None => Status::Open,
    }
}

/// Position key: board, side to move, castling rights and the en passant
/// square only when an en passant capture is legal; no clock.
pub fn key(board: &Board) -> u64 {
    board.hash_with(EnPassantMode::Legal)
}

/// The position key after `m`, same identity as [`key`].
pub fn child_key(board: &Board, m: Move) -> u64 {
    board.hash_after_with(m, EnPassantMode::Legal)
}

/// Whether `m` resets the 50-move counter (pawn move or capture).
pub fn is_zeroing(board: &Board, m: Move) -> bool {
    board.is_zeroing(m)
}

/// The per-file en passant Zobrist key: the value `hash_with(Always)` XORs
/// in for an en passant square on `file` that [`EnPassantMode::Legal`] drops
/// again. `hash_after_with(m, Legal)` needs it only to turn an `Always`
/// child key into a `Legal` one on a double push no enemy pawn can capture.
///
/// Upstream feedback (D2): `hash_after_with(m, Legal)` could skip its clone
/// when no enemy pawn attacks the passed square, or expose the per-file en
/// passant key. Either would retire this derivation.
///
/// The keys are derived, not hard-coded: for each file a board is built
/// with a white pawn on rank 2 of that file and nothing else but the two
/// commoners, the double push is made, and the key is the made child's
/// `hash_with(Always) ^ hash_with(Legal)` (the child's en passant square is
/// present and uncapturable — no black pawn attacks it).
pub fn ep_key(file: usize) -> u64 {
    debug_assert!(file < 8);
    static KEYS: std::sync::OnceLock<[u64; 8]> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        let mut keys = [0u64; 8];
        for (f, k) in keys.iter_mut().enumerate() {
            // White pawn on {file}2 (rank-2 row: f empty files, `P`, the
            // rest); commoners stay on e1/e8, off every pawn file.
            let rank2 = format!(
                "{}P{}",
                if f > 0 { f.to_string() } else { String::new() },
                if 7 - f > 0 {
                    (7 - f).to_string()
                } else {
                    String::new()
                }
            );
            let b = Board::from_fen(&format!("4k3/8/8/8/8/8/{rank2}/4K3 w - - 0 1")).unwrap();
            let m = b
                .move_from_uci(&format!("{}2{}4", FILES[f], FILES[f]))
                .unwrap();
            let mut child = b.clone();
            child.do_move(m, &mut StateInfo::new());
            debug_assert!(child.ep_square().is_some());
            debug_assert!(child.legal_ep_square().is_none());
            *k = child.hash_with(EnPassantMode::Always) ^ child.hash_with(EnPassantMode::Legal);
        }
        keys
    })[file]
}

const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

/// Checker-compatible insufficient material for side `s`: `s` cannot win
/// because the opponent `o` still has its commoner and one of these holds:
///
/// - `s` has only its commoner;
/// - `o` has other pieces, the board holds only commoners and bishops, all
///   of `s`'s bishops stand on one square colour and all of `o`'s on the
///   other;
/// - `o` has only its commoner, the board has no queen and no pawn, and
///   either exactly one knight, bishop or rook is on the board, or every
///   non-commoner piece is a knight and there are at most two of them.
///
/// This is the Lichess atomic rule the external checker applies; the board
/// crate's own `is_material_draw` covers only bare K vs K. Both sides
/// insufficient ends the game as a draw there, so [`status`] treats it as
/// `Failed` — a position that is mated or extinct never reaches this test.
pub fn insufficient_material(board: &Board, s: Color) -> bool {
    let o = s.flip();
    let s_commoner = board.commoners(s);
    let o_commoner = board.commoners(o);
    if o_commoner.is_empty() {
        return false;
    }
    let s_others = board.pieces_color(s) & !s_commoner;
    if s_others.is_empty() {
        return true;
    }
    let o_others = board.pieces_color(o) & !o_commoner;
    let noncommoner = board.occupied() & !board.pieces_pt(PieceType::Commoner);
    if !o_others.is_empty() {
        let bishops = board.pieces_pt(PieceType::Bishop);
        if noncommoner != bishops {
            return false;
        }
        let s_bishops = bishops & board.pieces_color(s);
        let o_bishops = bishops & board.pieces_color(o);
        let light = light_squares();
        let dark = !light;
        return (s_bishops & dark).is_empty() && (o_bishops & light).is_empty()
            || (s_bishops & light).is_empty() && (o_bishops & dark).is_empty();
    }
    if !(board.pieces_pt(PieceType::Queen) | board.pieces_pt(PieceType::Pawn)).is_empty() {
        return false;
    }
    let count = noncommoner.count();
    if count == 1 {
        let pt = board.piece_on(noncommoner.lsb()).type_of();
        return matches!(
            pt,
            Some(PieceType::Knight | PieceType::Bishop | PieceType::Rook)
        );
    }
    noncommoner == board.pieces_pt(PieceType::Knight) && count <= 2
}

/// Whether neither side can win on material (checker draw).
pub fn material_draw(board: &Board) -> bool {
    insufficient_material(board, Color::White) && insufficient_material(board, Color::Black)
}

fn light_squares() -> Bitboard {
    // Square index is `rank * 8 + file` with a1 = 0 (dark). Light squares
    // have odd `file + rank`.
    let mut bits = 0u64;
    for sq in 0..64u64 {
        if (sq % 8 + sq / 8) % 2 == 1 {
            bits |= 1 << sq;
        }
    }
    Bitboard(bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_movegen::types::Square;

    fn board(fen: &str) -> Board {
        Board::from_fen(fen).unwrap()
    }

    #[test]
    fn insufficient_material_clauses() {
        // The full clause matrix (both colours and sides to move) is covered
        // by tests/rules.rs over tests/fixtures/terminal.tsv; these spot-check
        // each clause direction.
        let insufficient = [
            "4k3/8/8/8/8/8/8/R3K3 w - - 0 1",  // K+R vs K
            "3bk3/8/8/8/8/8/8/4KB2 w - - 0 1", // opposite-colour bishops
        ];
        for fen in insufficient {
            assert!(material_draw(&board(fen)), "{fen}");
        }
        let sufficient = [
            "4k3/8/8/8/8/8/8/3QK3 w - - 0 1",   // K+Q vs K
            "3nk3/8/8/8/8/8/8/2N1K3 w - - 0 1", // K+N vs K+N
            "2b1k3/8/8/8/8/8/8/4KB2 w - - 0 1", // same-colour bishops
        ];
        for fen in sufficient {
            assert!(!material_draw(&board(fen)), "{fen}");
        }
    }

    #[test]
    fn status_of_terminal_positions() {
        // Side to move mated at counter 100: won for the white attacker,
        // failed (rule50 refutation) for the black attacker.
        let b = board("7K/8/8/8/8/8/1Q6/k7 b - - 100 1");
        assert_eq!(status(&b, Color::White), Status::Won);
        assert_eq!(status(&b, Color::Black), Status::Failed { from: 100 });
        // Extinction both ways.
        let b = board("8/8/8/8/8/8/8/k7 w - - 0 1");
        assert_eq!(status(&b, Color::White), Status::Failed { from: 0 });
        assert_eq!(status(&b, Color::Black), Status::Won);
        // A K+R vs K material draw is Failed, not Open.
        let b = board("4k3/8/8/8/8/8/8/R3K3 w - - 0 1");
        assert_eq!(status(&b, Color::White), Status::Failed { from: 0 });
        assert_eq!(status(&b, Color::Black), Status::Failed { from: 0 });
    }

    #[test]
    fn square_colour_sanity() {
        // a1 dark, h1 light, h8 dark.
        let light = light_squares();
        assert!((light & Bitboard::square_bb(Square::A1)).is_empty());
        assert!(!(light & Bitboard::square_bb(Square::H1)).is_empty());
        assert!((light & Bitboard::square_bb(Square::H8)).is_empty());
    }
}
