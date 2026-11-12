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
use atomic_movegen::movegen::generate_pseudo_legal;
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
/// This is the shipped path: candidates are confirmed by making the move, so
/// there are no false positives. [`immediate_explosion_bits`] is the
/// board-free closure (plan-20 R11) and must agree with this on every legal
/// position (test oracle).
pub fn immediate_explosion(board: &Board, side: Color) -> Option<Move> {
    debug_assert_eq!(board.side_to_move(), side);
    if board.commoners(side.flip()).is_empty() {
        return None;
    }
    let zone = Board::blast_zone(board.commoners(side.flip()).lsb());
    let mut moves = MoveList::new();
    atomic_movegen::movegen::generate_legal(board, &mut moves);
    for &m in moves.as_slice() {
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

/// The board-free one-ply explosion closure (plan-20 R11, item 28):
/// candidates come from the pseudo-legal stream, their casualties are read
/// with [`Board::blast_casualties`], and only the candidates are legality
/// checked. No child board is built and no legal move list is generated.
///
/// The move chosen can differ from [`immediate_explosion`]'s (the pseudo-legal
/// order is not the legal order); the settled value is the same.
pub fn immediate_explosion_bits(board: &Board, side: Color) -> Option<Move> {
    debug_assert_eq!(board.side_to_move(), side);
    let enemy_commoners = board.commoners(side.flip());
    if enemy_commoners.is_empty() {
        return None;
    }
    let mut state = StateInfo::new();
    board.populate_state(&mut state);
    let mut moves = MoveList::new();
    generate_pseudo_legal(board, &mut moves);
    for &m in moves.as_slice() {
        if !board.is_capture(m) {
            continue;
        }
        // A captured piece can only blast the enemy commoner when that
        // commoner is at or beside the capture square.
        if (Board::blast_zone(m.to_sq()) & enemy_commoners).is_empty() {
            continue;
        }
        if (board.blast_casualties(m).all() & enemy_commoners).is_empty() {
            continue;
        }
        if board.legal(m, &state) {
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

/// The board-free settle variant (plan-20 R11): same value as
/// [`one_ply_settle`], no child build and no legal move list. The returned
/// work is `0` — the bit tests are pass-1 effect work, not auxiliary work.
pub fn one_ply_settle_bits(board: &Board, attacker: Color) -> Option<(Settle, u64)> {
    let side = board.side_to_move();
    let mv = immediate_explosion_bits(board, side)?;
    let settle = if side == attacker {
        Settle::SideToMoveWins(mv)
    } else {
        Settle::SideToMoveLoses(mv)
    };
    Some((settle, 0))
}

/// The shipped conservative screen for a **non-built** quiet child: could the
/// opponent have an immediate explosion after our move? A miss only loses a
/// settle opportunity (never soundness); a hit is verified by
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

/// The board-free screen (plan-20 R11): the quiet move's occupancy change is
/// applied to the bitboards, the pieces left beside our commoner (and away
/// from theirs, where a capture would also destroy their commoner) are read
/// off, and one attack lookup per such square decides. A miss loses a settle
/// opportunity only; a hit is verified by [`one_ply_settle_bits`].
pub fn settle_screen_bits(board: &Board, m: Move, mover: Color) -> bool {
    let commoners = board.commoners(mover);
    if commoners.is_empty() {
        return false;
    }
    let k = commoners.lsb();
    let (from, to) = (m.from_sq(), m.to_sq());
    let from_bb = Bitboard::square_bb(from);
    let to_bb = Bitboard::square_bb(to);
    // Squares where one of our pieces is beside our commoner and a capture
    // there would blast it. Their commoner's own blast ring is excluded:
    // such a capture would destroy their commoner too, so it is never legal.
    let mut beside = Board::blast_zone(k);
    let their_commoners = board.commoners(mover.flip());
    if !their_commoners.is_empty() {
        beside = beside & !Board::blast_zone(their_commoners.lsb());
    }
    // The quiet move's own occupancy change: the moved piece leaves `from`
    // and lands on `to` (non-built moves are never captures or specials).
    let ours = (board.pieces_color(mover) & !from_bb) | to_bb;
    let exposed = ours & beside;
    if exposed.is_empty() {
        return false;
    }
    let occ = (board.occupied() & !from_bb) | to_bb;
    let them = mover.flip();
    let capturers = board.pieces_color(them) & !board.commoners(them);
    let mut sq = exposed;
    while !sq.is_empty() {
        let d = sq.pop_lsb();
        if !(board.attackers_to_color(d, them, occ) & capturers).is_empty() {
            return true;
        }
    }
    false
}

/// Whether `from` lies on a rank, file or diagonal through `to` (the shipped
/// screen's conservative line-opening test).
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
    use atomic_movegen::movegen::generate_legal;

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
    fn board_free_explosion_matches_the_build_path() {
        // Plan-20 R11: the pseudo-legal + blast-casualties closure must equal
        // the old build-and-make oracle on every fixture position.
        let mut checked = 0usize;
        let mut hits = 0usize;
        for (board, _line, _att) in testutil::positions() {
            let side = board.side_to_move();
            let fast = immediate_explosion_bits(&board, side);
            let full = immediate_explosion(&board, side);
            assert_eq!(fast, full, "{board:?}");
            checked += 1;
            if fast.is_some() {
                hits += 1;
            }
        }
        assert!(checked >= 1_000, "only {checked} positions");
        eprintln!("board-free explosion: positions={checked} hits={hits}");
    }

    #[test]
    fn board_free_explosion_never_claims_a_false_win() {
        // Every move the closure returns is legal and really removes the
        // enemy commoner after the move (no false win).
        let mut hits = 0usize;
        for (board, _line, _att) in testutil::positions().into_iter().take(400) {
            let side = board.side_to_move();
            let Some(mv) = immediate_explosion_bits(&board, side) else {
                continue;
            };
            hits += 1;
            assert!(board.is_legal(mv), "{mv} on {board:?}");
            let mut child = board.clone();
            child.do_move(mv, &mut StateInfo::new());
            assert!(
                child.commoners(side.flip()).is_empty(),
                "{mv} did not explode the commoner on {board:?}"
            );
        }
        assert!(hits >= 1, "no explosion closure hit in the sample");
    }

    #[test]
    fn screen_never_misses_a_settle() {
        // The screen is a conservative hint: a false hit only costs a build,
        // but a miss would lose the settle. For every fixture move whose
        // child really has an immediate explosion, the parent screen must
        // fire.
        let mut settles = 0usize;
        let mut missed = 0usize;
        for (board, _line, _att) in testutil::positions().into_iter().take(400) {
            let mover = board.side_to_move();
            let ctx = crate::mover::Mover::new(&board);
            let mut moves = MoveList::new();
            generate_legal(&board, &mut moves);
            for &m in moves.as_slice() {
                if ctx.effect(&board, m).needs_board {
                    continue;
                }
                let mut child = board.clone();
                child.do_move(m, &mut StateInfo::new());
                if immediate_explosion_bits(&child, child.side_to_move()).is_none() {
                    continue;
                }
                settles += 1;
                if !settle_screen_bits(&board, m, mover) {
                    missed += 1;
                    eprintln!(
                        "missed settle: fen={} move={}",
                        board.fen_with(atomic_movegen::types::EnPassantMode::Legal),
                        m.to_uci()
                    );
                }
            }
        }
        assert!(settles >= 1, "only {settles} child settles");
        assert_eq!(missed, 0, "{missed} of {settles} settles missed");
    }
}
