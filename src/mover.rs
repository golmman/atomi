// Justification for >10 KB: the needs-board rules, the check test and
// the playout differential that pins them are one contract.

//! `Mover` (per expanded node) and `Effect` (per legal move): a child's
//! position key, 50-move counter and "must be built" flag, computed from
//! the parent without making the move (M1, decision D19).
//!
//! A move needs a built child only when its effect cannot be read off the
//! parent exactly:
//!
//! - promotion, en passant or castling;
//! - a capture (blast bookkeeping);
//! - a commoner move (blast-adjacency rules);
//! - a move giving check (so a mate is always detected at the parent — the
//!   soundness of the lazy stalemate depends on it);
//! - a double push an enemy pawn attacks on the passed square (its
//!   `Legal`-identity key keeps the en passant file, so it must come from
//!   the made child);
//! - **any** move when `them` (the side to move in the children) has only
//!   its commoner and pawns: then a quiet move can stalemate them, and a
//!   lazy `Open` child would deny an AND node a cheap refutation — the
//!   rem01 regression (G1; the node-level gate costs nothing per child).
//!
//! The check test for the remaining quiet class (non-capture, non-commoner,
//! non-special, non-adjacent double push):
//!
//! - no check when the enemy commoner is missing or the commoners touch
//!   (the move cannot change either fact);
//! - else on `occ' = occupied ^ from | to`: **direct** —
//!   `attacks_from(pt, us, to, occ')` hits the enemy commoner; **discovered**
//!   — `rook_attacks(ksq, occ') & our_rook_queens & !from` or the
//!   `bishop_attacks` analogue is non-empty.
//!
//! This is exact for that move class (the differential test pins it against
//! `checkers()`); every other class takes the conservative `needs_board`.
//!
//! `Mover` depends on no `crate::` module but `rules` (for `ep_key`).

use crate::rules;
use atomic_movegen::attacks::{
    attacks_from, bishop_attacks, king_attacks, pawn_attacks, rook_attacks,
};
use atomic_movegen::board::Board;
use atomic_movegen::types::{Bitboard, Color, Move, PieceType, Square, file_of};

/// A move's effect on the parent, computed without building the child.
pub struct Effect {
    /// `Legal`-identity child key (valid unless `needs_board`).
    pub key: u64,
    /// The child's 50-move counter.
    pub counter: u16,
    /// Whether the move resets the counter (pawn move or capture).
    pub zeroing: bool,
    /// Whether the child must be built (see the module docs).
    pub needs_board: bool,
}

/// Per-node move-effect context.
pub struct Mover {
    us: Color,
    occupied: Bitboard,
    /// The enemy commoner's square, `None` when it is gone.
    enemy_commoner: Option<Square>,
    /// Our commoner and the enemy's touch (atomic check immunity).
    commoners_touch: bool,
    /// Our rooks and queens (discovered rook-line checks).
    rq: Bitboard,
    /// Our bishops and queens (discovered bishop-line checks).
    bq: Bitboard,
    /// Enemy pawns (double-push en passant test).
    their_pawns: Bitboard,
    /// Whether `them` (the children) has only its commoner and pawns — then
    /// a quiet move can stalemate them and every child must be built (G1;
    /// the rem01 lazy-stalemate regression).
    stalemate_plausible: bool,
}

impl Mover {
    /// Whether the stalemate gate (G1) fires on this node.
    pub fn stalemate_plausible(&self) -> bool {
        self.stalemate_plausible
    }
}

impl Mover {
    /// Build the context for `board`'s side to move.
    pub fn new(board: &Board) -> Mover {
        let us = board.side_to_move();
        let our_commoners = board.commoners(us);
        let their_commoners = board.commoners(us.flip());
        let enemy_commoner = (!their_commoners.is_empty()).then(|| their_commoners.lsb());
        let commoners_touch =
            enemy_commoner.is_some_and(|ksq| !(our_commoners & king_attacks(ksq)).is_empty());
        let ours = board.pieces_color(us);
        let queens = board.pieces_pt(PieceType::Queen);
        Mover {
            us,
            occupied: board.occupied(),
            enemy_commoner,
            commoners_touch,
            rq: ours & (board.pieces_pt(PieceType::Rook) | queens),
            bq: ours & (board.pieces_pt(PieceType::Bishop) | queens),
            their_pawns: board.pieces_color_pt(us.flip(), PieceType::Pawn),
            stalemate_plausible: (board.pieces_color(us.flip())
                & !(board.commoners(us.flip())
                    | board.pieces_color_pt(us.flip(), PieceType::Pawn)))
            .is_empty(),
        }
    }

    /// The effect of legal move `m` on `board`.
    pub fn effect(&self, board: &Board, m: Move) -> Effect {
        let (h, counter) = board.hash_and_rule50_after(m);
        let double_push = board.is_double_push(m);
        let needs_board = self.stalemate_plausible
            || m.is_promotion()
            || m.is_en_passant()
            || m.is_castling()
            || board.is_capture(m)
            || board.piece_on(m.from_sq()).type_of() == Some(PieceType::Commoner)
            || self.gives_check(board, m)
            || (double_push && self.enemy_pawn_attacks_passed(m));
        // A non-board double push's ep file is uncapturable: `Legal` drops
        // the per-file key the `Always` hash kept.
        let key = if double_push && !needs_board {
            h ^ rules::ep_key(file_of(m.to_sq()) as usize)
        } else {
            h
        };
        Effect {
            key,
            counter,
            zeroing: counter == 0,
            needs_board,
        }
    }

    /// Whether `m` gives check, for the quiet move class only (see the
    /// module docs; callers may pass other classes — the test pins the
    /// class boundary itself).
    fn gives_check(&self, board: &Board, m: Move) -> bool {
        let (from, to) = (m.from_sq(), m.to_sq());
        let Some(ksq) = self.enemy_commoner else {
            return false;
        };
        if self.commoners_touch {
            return false;
        }
        let occ = self.occupied ^ Bitboard::square_bb(from) | Bitboard::square_bb(to);
        let pt = board.piece_on(from).type_of().unwrap();
        if !(attacks_from(pt, board.side_to_move(), to, occ) & Bitboard::square_bb(ksq)).is_empty()
        {
            return true;
        }
        let not_from = !Bitboard::square_bb(from);
        !(rook_attacks(ksq, occ) & self.rq & not_from).is_empty()
            || !(bishop_attacks(ksq, occ) & self.bq & not_from).is_empty()
    }

    /// Whether an enemy pawn attacks the square a double push passes over.
    fn enemy_pawn_attacks_passed(&self, m: Move) -> bool {
        // The passed square sits halfway between from and to on the same
        // file.
        let passed = Square::from_u8((m.from_sq() as u8 + m.to_sq() as u8) / 2);
        !(pawn_attacks(self.us, passed) & self.their_pawns).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;
    use atomic_movegen::board::StateInfo;
    use atomic_movegen::movegen::generate_legal;
    use atomic_movegen::types::{EnPassantMode, MoveList};

    #[test]
    fn effect_differential() {
        // For every legal move of every playout position: counter/zeroing
        // equal the made child's; a non-board move's key equals the made
        // child's `hash_with(Legal)` and gives no check; a board move's
        // predicted key... is supplied by the child itself (here we check
        // the made key equals `rules::child_key`); and the quiet-class
        // check test is exact.
        let mut positions = 0usize;
        let mut moves_total = 0usize;
        let mut board_children = 0usize;
        let mut quiet_class = 0usize;
        for (board, _line, _attacker) in testutil::positions() {
            positions += 1;
            let mover = Mover::new(&board);
            let mut moves = MoveList::new();
            generate_legal(&board, &mut moves);
            for m in &moves {
                moves_total += 1;
                let e = mover.effect(&board, m);
                let mut child = board.clone();
                child.do_move(m, &mut StateInfo::new());
                assert_eq!(e.counter, child.rule50(), "{m}");
                assert_eq!(e.zeroing, rules::is_zeroing(&board, m), "{m}");
                let quiet = !m.is_promotion()
                    && !m.is_en_passant()
                    && !m.is_castling()
                    && !board.is_capture(m)
                    && board.piece_on(m.from_sq()).type_of() != Some(PieceType::Commoner)
                    && !(board.is_double_push(m) && mover.enemy_pawn_attacks_passed(m));
                if e.needs_board {
                    board_children += 1;
                    assert_eq!(
                        child.hash_with(EnPassantMode::Legal),
                        rules::child_key(&board, m),
                        "{m}"
                    );
                } else {
                    assert_eq!(
                        e.key,
                        child.hash_with(EnPassantMode::Legal),
                        "non-board key wrong for {m}"
                    );
                    assert!(child.checkers().is_empty(), "non-board check for {m}");
                }
                if quiet && !mover.stalemate_plausible() {
                    quiet_class += 1;
                    assert_eq!(
                        e.needs_board,
                        !child.checkers().is_empty(),
                        "check prediction wrong for quiet-class {m}"
                    );
                }
            }
        }
        eprintln!(
            "effect_differential: positions={positions} moves={moves_total} \
             board_children={board_children} ({:.1}%) quiet_class={quiet_class}",
            100.0 * board_children as f64 / moves_total as f64
        );
        assert!(positions >= 5_000, "only {positions} positions");
    }

    #[test]
    fn ep_keys_are_distinct_and_derived() {
        let keys: Vec<u64> = (0..8).map(rules::ep_key).collect();
        for (i, k) in keys.iter().enumerate() {
            assert_ne!(*k, 0, "file {i}");
            for (j, other) in keys.iter().enumerate() {
                if i != j {
                    assert_ne!(k, other, "files {i},{j}");
                }
            }
        }
        // On every uncapturable double push in the playouts, the file key
        // equals `hash_after_with(m, Legal) ^ hash_after(m)`.
        let mut seen = 0usize;
        for (board, _line, _attacker) in testutil::positions() {
            let mut moves = MoveList::new();
            generate_legal(&board, &mut moves);
            for m in &moves {
                if !board.is_double_push(m) {
                    continue;
                }
                let mut child = board.clone();
                child.do_move(m, &mut StateInfo::new());
                if child.legal_ep_square().is_some() {
                    continue;
                }
                seen += 1;
                let file = file_of(m.to_sq()) as usize;
                assert_eq!(
                    rules::ep_key(file),
                    board.hash_after_with(m, EnPassantMode::Legal) ^ board.hash_after(m),
                    "{m}"
                );
            }
        }
        assert!(seen >= 8, "only {seen} uncapturable double pushes seen");
    }

    #[test]
    fn status_with_matches_status() {
        for (board, _line, attacker) in testutil::positions() {
            for att in [attacker, attacker.flip()] {
                let mut state = StateInfo::new();
                board.populate_state(&mut state);
                let mut moves = MoveList::new();
                atomic_movegen::movegen::generate_legal_with_state(&board, &state, &mut moves);
                assert_eq!(
                    rules::status_with(&board, att, &state, !moves.is_empty()),
                    rules::status(&board, att),
                );
            }
        }
    }

    #[test]
    fn stalemate_gate_builds_every_child() {
        // `them` = black: only a commoner (a1) → the G1 gate fires and every
        // move needs a board child.
        let gated = Board::from_fen("K7/8/8/8/8/3N4/N7/kN6 w - - 0 1").unwrap();
        let mover = Mover::new(&gated);
        assert!(mover.stalemate_plausible());
        let mut moves = MoveList::new();
        generate_legal(&gated, &mut moves);
        assert!(!moves.is_empty());
        for m in &moves {
            assert!(mover.effect(&gated, m).needs_board, "{m}");
        }
        // `them` = black: commoner + one pawn → still gated.
        let pawn = Board::from_fen("K7/8/8/8/8/3N4/Np6/kN6 w - - 0 1").unwrap();
        assert!(Mover::new(&pawn).stalemate_plausible());
        // `them` = black: commoner + knight → gate off; the quiet class can
        // stay lazy.
        let free = Board::from_fen("1K5n/8/8/8/8/3N4/N7/kN6 w - - 0 1").unwrap();
        let mover = Mover::new(&free);
        assert!(!mover.stalemate_plausible());
        let mut moves = MoveList::new();
        generate_legal(&free, &mut moves);
        assert!(
            (&moves)
                .into_iter()
                .any(|m| !mover.effect(&free, m).needs_board)
        );
    }
}
