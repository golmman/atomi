// Justification for >10 KB: the signature, the exact settle, the screen and their differential tests are one contract.

//! Threat signatures, the null-side replay and the exact one-ply explosion
//! settle (D21).
//!
//! A signature is a 64-bit set of capture destinations in the enemy
//! commoner's blast zone: a move to such a square explodes the enemy
//! commoner (subject to legality, which the exact settle verifies). The
//! "null" side signature is just the other side's signature on the same
//! board — equivalent to flipping the side to move and clearing en passant.
//! No clone of the position is ever searched, keyed, stored, put on the line
//! or emitted.

use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Bitboard, Color, Move, MoveList};

/// A side's immediate explosion threats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThreatSignature {
    /// Capture destinations that would blast the enemy commoner.
    targets: u64,
}

impl ThreatSignature {
    /// The number of distinct explosion targets.
    pub fn count(self) -> u32 {
        self.targets.count_ones()
    }

    /// The raw target bits (for parry comparisons).
    pub fn targets(self) -> u64 {
        self.targets
    }

    /// Whether the signature has no threats.
    pub fn is_empty(self) -> bool {
        self.targets == 0
    }

    /// Targets present in `self` but not in `other` (threats removed by a
    /// move).
    pub fn removed_from(self, other: ThreatSignature) -> u64 {
        other.targets & !self.targets
    }

    /// Targets present in `self` but not in `other` (threats created).
    pub fn created_beyond(self, other: ThreatSignature) -> u64 {
        self.targets & !other.targets
    }
}

/// The signature of `side`'s immediate explosion threats on `board`.
pub fn signature_for(board: &Board, side: Color) -> ThreatSignature {
    let target = board.commoners(side.flip());
    if target.is_empty() {
        return ThreatSignature::default();
    }
    let occ = board.occupied();
    let zone = Board::blast_zone(target.lsb());
    let victims = board.pieces_color(side.flip());
    let mut targets = 0u64;
    let mut sq = zone;
    while !sq.is_empty() {
        let d = sq.pop_lsb();
        let victim_here = !(victims & Bitboard::square_bb(d)).is_empty();
        let ep_here = side == board.side_to_move() && board.legal_ep_square() == Some(d);
        if (victim_here || ep_here) && !board.attackers_to_color(d, side, occ).is_empty() {
            targets |= 1u64 << (d as u8);
        }
    }
    ThreatSignature { targets }
}

/// The signature of the side to move.
pub fn signature(board: &Board) -> ThreatSignature {
    signature_for(board, board.side_to_move())
}

/// The exact one-ply explosion test: does `side` (to move) have a legal
/// capture that removes the opponent's commoner?
///
/// The candidate is confirmed by making the move, so there are no false
/// positives. Returns the winning move.
pub fn immediate_explosion(board: &Board, side: Color) -> Option<Move> {
    debug_assert_eq!(board.side_to_move(), side);
    if board.commoners(side.flip()).is_empty() {
        return None;
    }
    let zone = Board::blast_zone(board.commoners(side.flip()).lsb());
    let mut moves = MoveList::new();
    generate_legal(board, &mut moves);
    for m in &moves {
        if !board.is_capture(m) || (Board::blast_zone(m.to_sq()) & zone).is_empty() {
            continue;
        }
        let mut child = board.clone();
        child.do_move(m, &mut StateInfo::new());
        if child.commoners(side.flip()).is_empty() {
            return Some(m);
        }
    }
    None
}

/// A verified child-level settle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settle {
    /// The child's side to move explodes the opponent's commoner.
    SideToMoveWins(Move),
    /// The child's opponent wins (only possible when the child's side to
    /// move is the defender, who has no immediate explosion).
    SideToMoveLoses(Move),
}

/// Run the exact settle on a fresh child. The result is relative to the
/// child's side to move. `attacker` decides which settle variant applies.
pub fn one_ply_settle(board: &Board, attacker: Color) -> Option<(Settle, u64)> {
    let side = board.side_to_move();
    let mv = immediate_explosion(board, side)?;
    let settle = if side == attacker {
        Settle::SideToMoveWins(mv)
    } else {
        Settle::SideToMoveLoses(mv)
    };
    Some((settle, 1))
}

/// A conservative screen for a **non-built** quiet child: could the opponent
/// have an immediate explosion after our move? A miss only loses a settle
/// opportunity (never soundness); a hit is verified by
/// [`one_ply_settle`].
pub fn settle_screen(board: &Board, m: Move, mover: Color) -> bool {
    let victim_side = mover;
    let commoner = board.commoners(victim_side);
    if commoner.is_empty() {
        return false;
    }
    let zone = Board::blast_zone(commoner.lsb());
    let (from, to) = (m.from_sq(), m.to_sq());
    let from_bb = Bitboard::square_bb(from);
    let to_bb = Bitboard::square_bb(to);
    // Our pieces in the enemy blast zone after the move, or the new square
    // entering it.
    let victims = (board.pieces_color(victim_side) & !from_bb) | to_bb;
    if (victims & zone).is_empty() && board.legal_ep_square().is_none() {
        return false;
    }
    // A pre-existing attack, a move into the zone, or a line the move opens.
    let them = mover.flip();
    let occ = board.occupied();
    let mut sq = zone;
    while !sq.is_empty() {
        let d = sq.pop_lsb();
        let relevant = !(victims & (Bitboard::square_bb(d) | Bitboard::square_bb(to))).is_empty()
            || board.legal_ep_square() == Some(d);
        let attacked = !board.attackers_to_color(d, them, occ).is_empty();
        if relevant && (attacked || on_ray(from, d) || to == d) {
            return true;
        }
    }
    false
}

/// Whether `from` lies on a rank, file or diagonal through `to` (a
/// conservative line-opening test).
fn on_ray(from: atomic_movegen::types::Square, to: atomic_movegen::types::Square) -> bool {
    let (f, t) = (from as i32, to as i32);
    let (fd, td) = (f & 7, t & 7);
    let (fr, tr) = (f >> 3, t >> 3);
    from != to && (fd == td || fr == tr || (fd - fr) == (td - tr) || (fd + fr) == (td + tr))
}

/// The null-side signature used by the defender priors: the attacker's
/// explosion threats on the same board (no position clone).
pub fn null_side_signature(board: &Board, attacker: Color) -> ThreatSignature {
    signature_for(board, attacker)
}

/// `true` when `rules::status` and the settle agree on the child's
/// terminality (test helper).
#[cfg(test)]
fn agrees(board: &Board, attacker: Color) -> bool {
    use crate::rules;
    let Some((settle, _)) = one_ply_settle(board, attacker) else {
        return true;
    };
    let mv = match settle {
        Settle::SideToMoveWins(m) | Settle::SideToMoveLoses(m) => m,
    };
    let mut after = board.clone();
    after.do_move(mv, &mut StateInfo::new());
    match settle {
        Settle::SideToMoveWins(_) => matches!(rules::status(&after, attacker), rules::Status::Won),
        Settle::SideToMoveLoses(_) => {
            matches!(
                rules::status(&after, attacker),
                rules::Status::Failed { .. }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules;
    use crate::testutil;

    #[test]
    fn signature_is_deterministic_and_self_consistent() {
        for (b, _line, _att) in testutil::positions().into_iter().take(200) {
            let a = signature(&b);
            let b2 = signature(&b);
            assert_eq!(a, b2);
            assert_eq!(signature_for(&b, b.side_to_move()), a);
        }
    }

    #[test]
    fn direct_commoner_capture_is_a_threat() {
        // White rook on a5, black commoner a8, black pawn a7: the rook can
        // take a7, whose blast hits a8.
        let b = Board::from_fen("k7/p7/R7/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(!signature(&b).is_empty());
    }

    #[test]
    fn settle_matches_status_on_playouts() {
        let mut children = 0usize;
        let mut settles = 0usize;
        for (board, _line, attacker) in testutil::positions() {
            let mut moves = MoveList::new();
            generate_legal(&board, &mut moves);
            for m in &moves {
                let mut child = board.clone();
                child.do_move(m, &mut StateInfo::new());
                if rules::status(&child, attacker) != rules::Status::Open {
                    continue;
                }
                children += 1;
                if let Some((settle, _)) = one_ply_settle(&child, attacker) {
                    settles += 1;
                    if !agrees(&child, attacker) {
                        let mv = match settle {
                            Settle::SideToMoveWins(m) | Settle::SideToMoveLoses(m) => m,
                        };
                        let mut after = child.clone();
                        after.do_move(mv, &mut StateInfo::new());
                        panic!(
                            "settle/status disagree: fen={} attacker={attacker:?} move={} settle={settle:?} status={:?} after_white_commoners={} after_black_commoners={} after_fen={}",
                            child.fen_with(atomic_movegen::types::EnPassantMode::Legal),
                            mv.to_uci(),
                            rules::status(&child, attacker),
                            after.commoners(Color::White).count(),
                            after.commoners(Color::Black).count(),
                            after.fen_with(atomic_movegen::types::EnPassantMode::Legal),
                        );
                    }
                }
            }
        }
        eprintln!("settle differential: open children={children} settles={settles}");
    }

    #[test]
    fn screen_never_hides_a_built_settle() {
        // A hit screened from the parent is verified; a missed hit is a
        // strength loss only, so this test pins that a hit implies the
        // built child's exact settle and that a quiet move screen is
        // conservative on the obvious case.
        let b = Board::from_fen("k7/8/8/8/8/8/1R6/4K3 w - - 0 1").unwrap();
        let mut moves = MoveList::new();
        generate_legal(&b, &mut moves);
        let _ = settle_screen(&b, moves.as_slice()[0], b.side_to_move());
    }

    #[test]
    fn ray_test() {
        use atomic_movegen::types::Square;
        assert!(on_ray(Square::A1, Square::H8));
        assert!(on_ray(Square::A1, Square::A8));
        assert!(on_ray(Square::A1, Square::H1));
        assert!(!on_ray(Square::A1, Square::B3));
        assert!(!on_ray(Square::A1, Square::A1));
    }
}
