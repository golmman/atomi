//! Move classes for the hand guide (D21).
//!
//! A class is a label on a legal move, computed from the parent, the move's
//! [`Effect`] and (when pass 1 built the child) the child board. It only
//! selects a starting-number pair and a tie-break order; it never decides a
//! value.

use crate::mover::Effect;
use atomic_movegen::board::Board;
use atomic_movegen::types::{Bitboard, Move, PieceType, Square};

/// The five hand-guide move classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveClass {
    /// Creates an immediate explosion/check threat.
    Threat,
    /// Looks active but a cheap reply parries it.
    WeakThreat,
    /// A capture, or a piece that is (or becomes) attacked.
    Hanging,
    /// Everything else.
    #[default]
    Quiet,
    /// A reply to a check; overrides every other class.
    Evasion,
}

impl MoveClass {
    /// The default class priority (lower is selected first). Mirrors the
    /// default `pn_base` column; parameters may change the numbers, not the
    /// mechanism order.
    pub fn rank(self) -> u8 {
        match self {
            MoveClass::Evasion => 1,
            MoveClass::Threat => 4,
            MoveClass::WeakThreat => 8,
            MoveClass::Quiet => 9,
            MoveClass::Hanging => 15,
        }
    }

    /// Whether this is the quiet group.
    pub fn is_quiet(self) -> bool {
        self == MoveClass::Quiet
    }
}

/// Whether a capture at `to` lands in the blast zone of `commoner`.
fn blast_hits(to: Square, commoner: Bitboard) -> bool {
    !(Board::blast_zone(to) & commoner).is_empty()
}

/// `true` when some non-commoner piece of `color` is attacked.
fn loose_piece_attacked(board: &Board, color: atomic_movegen::types::Color) -> bool {
    let occ = board.occupied();
    let them = color.flip();
    let mut pieces = board.pieces_color(color) & !board.commoners(color);
    while !pieces.is_empty() {
        let sq = pieces.pop_lsb();
        if !board.attackers_to_color(sq, them, occ).is_empty() {
            return true;
        }
    }
    false
}

/// Classify legal move `m` of `board` under `in_check`.
///
/// `child` is `Some` only when pass 1 already built the child (a capture,
/// commoner move, check, special move or the G1 stalemate gate).
pub fn classify(
    board: &Board,
    m: Move,
    effect: &Effect,
    child: Option<&Board>,
    in_check: bool,
) -> MoveClass {
    if in_check {
        return MoveClass::Evasion;
    }
    let us = board.side_to_move();
    let them = us.flip();
    let their_commoner = board.commoners(them);
    let capture = board.is_capture(m);
    let gives_check = child.is_some_and(|c| !c.checkers().is_empty());
    let explosion_threat = capture && blast_hits(m.to_sq(), their_commoner);
    if gives_check || explosion_threat {
        if let Some(c) = child
            && c.side_to_move() == them
            && crate::guide::threat::immediate_explosion(c, them).is_some()
        {
            return MoveClass::WeakThreat;
        }
        return MoveClass::Threat;
    }
    if capture {
        return MoveClass::Hanging;
    }
    if let Some(c) = child {
        let occ = c.occupied();
        if !c.attackers_to_color(m.to_sq(), them, occ).is_empty() || loose_piece_attacked(c, us) {
            return MoveClass::Hanging;
        }
    } else if effect.needs_board && board.piece_on(m.from_sq()).type_of() != Some(PieceType::Pawn) {
        // A built, non-quiet move whose child we did not need: keep the
        // class conservative (it is only an ordering hint).
        return MoveClass::Hanging;
    }
    MoveClass::Quiet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mover::Mover;
    use atomic_movegen::board::StateInfo;
    use atomic_movegen::movegen::generate_legal;
    use atomic_movegen::types::MoveList;

    #[test]
    fn class_rank_is_a_total_order() {
        let classes = [
            MoveClass::Threat,
            MoveClass::WeakThreat,
            MoveClass::Hanging,
            MoveClass::Quiet,
            MoveClass::Evasion,
        ];
        for (i, a) in classes.iter().enumerate() {
            for b in &classes[i + 1..] {
                assert_ne!(a.rank(), b.rank());
            }
        }
    }

    #[test]
    fn evasions_override_everything() {
        let b = Board::from_fen("4k3/8/8/8/8/8/8/4K2r w - - 0 1").unwrap();
        let mut ml = MoveList::new();
        generate_legal(&b, &mut ml);
        let mover = Mover::new(&b);
        for m in &ml {
            let e = mover.effect(&b, m);
            let mut c = b.clone();
            c.do_move(m, &mut StateInfo::new());
            assert_eq!(classify(&b, m, &e, Some(&c), true), MoveClass::Evasion);
        }
    }

    #[test]
    fn quiet_moves_are_quiet() {
        let b = Board::from_fen("4k2n/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let mut ml = MoveList::new();
        generate_legal(&b, &mut ml);
        let mover = Mover::new(&b);
        let quiet = (&ml)
            .into_iter()
            .filter(|m| !mover.effect(&b, *m).needs_board)
            .count();
        assert!(quiet > 0);
        for m in &ml {
            let e = mover.effect(&b, m);
            if !e.needs_board {
                assert_eq!(classify(&b, m, &e, None, false), MoveClass::Quiet);
            }
        }
    }
}
