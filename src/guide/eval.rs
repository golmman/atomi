//! Cheap deterministic hand evaluation and EF-PN scaling (D21).
//!
//! `hand_eval` returns a bounded integer in `[-256, 256]` from the side to
//! move's perspective: material, commoner safety, immediate explosion
//! threats and a mobility proxy. It allocates nothing and runs no movegen.
//!
//! At an OR node a child with `e > 0` (attacker-favourable) gets a lower
//! proof number and a higher disproof number:
//!
//! ```text
//! f = clamp(16 + e * w / 16, 8, 32)
//! pn = max(1, pn_base * 16 / f)
//! dn = max(1, dn_base * f / 16)
//! ```
//!
//! `w` is `GuideParams.ef_weight` (`16` = the plan3 formula, `0` = neutral).
//! Every result is bounded by `OPEN_MAX` by the caller. The eval never
//! turns an `Open` value into `Proved`/`Refuted`.

use crate::guide::threat;
use crate::search::value::OPEN_MAX;
use atomic_movegen::board::Board;
use atomic_movegen::types::{Bitboard, Color, PieceType};

/// Material score of one piece type in centipawns-ish units (commoners are
/// priceless and excluded: their absence is terminal).
fn material(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::Commoner => 0,
        _ => 0,
    }
}

/// The square bitboard of the commoner of `c`, or `Bitboard::EMPTY`.
fn commoner(board: &Board, c: Color) -> Bitboard {
    board.commoners(c)
}

/// A mobility proxy: the number of distinct squares the side's pieces attack
/// (no legality, no movegen).
fn attacked_squares(board: &Board, c: Color) -> Bitboard {
    let occ = board.occupied();
    let mut pieces = board.pieces_color(c);
    let mut attacked = Bitboard::EMPTY;
    while !pieces.is_empty() {
        let sq = pieces.pop_lsb();
        let Some(pt) = board.piece_on(sq).type_of() else {
            continue;
        };
        attacked = attacked | atomic_movegen::attacks::attacks_from(pt, c, sq, occ);
    }
    attacked
}

/// How many of our pieces attack a square in the enemy commoner's blast
/// zone (commoner safety pressure).
fn commoner_pressure(board: &Board, us: Color) -> i32 {
    let them = us.flip();
    let targets = commoner(board, them);
    if targets.is_empty() {
        return 0;
    }
    let zone = Board::blast_zone(targets.lsb());
    let occ = board.occupied();
    let mut count = 0i32;
    let mut sq = zone;
    while !sq.is_empty() {
        let s = sq.pop_lsb();
        count += board.attackers_to_color(s, us, occ).count().min(3) as i32;
    }
    count
}

/// The hand eval in `[-256, 256]` from `board.side_to_move()`'s perspective.
pub fn hand_eval(board: &Board) -> i32 {
    let us = board.side_to_move();
    let them = us.flip();
    let mut material_score = 0i32;
    for pt in [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
    ] {
        let ours = board.pieces_color_pt(us, pt).count() as i32;
        let theirs = board.pieces_color_pt(them, pt).count() as i32;
        material_score += (ours - theirs) * material(pt);
    }
    let safety = commoner_pressure(board, us) - commoner_pressure(board, them);
    let threats = threat::signature(board).count() as i32;
    let mobility = (attacked_squares(board, us).count() as i32
        - attacked_squares(board, them).count() as i32)
        / 4;
    let raw = material_score / 32 + safety * 12 + threats * 20 + mobility;
    raw.clamp(-256, 256)
}

/// `f = clamp(16 + e, 8, 32)`.
pub fn factor(eval: i32) -> i32 {
    (16 + eval).clamp(8, 32)
}

/// The weighted factor: `f = clamp(16 + e * w / 16, 8, 32)`.
pub fn factor_weighted(eval: i32, weight: u32) -> i32 {
    factor((i64::from(eval) * i64::from(weight) / 16) as i32)
}

/// The EF-PN scaling of a mover-perspective `(pn, dn)` pair (weight 16).
pub fn scale(pn: u32, dn: u32, eval: i32) -> (u32, u32) {
    scale_weighted(pn, dn, eval, 16)
}

/// The EF-PN scaling of a mover-perspective `(pn, dn)` pair at `weight`.
pub fn scale_weighted(pn: u32, dn: u32, eval: i32, weight: u32) -> (u32, u32) {
    let f = factor_weighted(eval, weight) as u64;
    let pn = (u64::from(pn) * 16 / f).max(1) as u32;
    let dn = (u64::from(dn) * f / 16).max(1) as u32;
    (pn, dn)
}

/// The EF-PN band step (plan-19 R2): `2` when the attacker is clearly ahead,
/// `0` when clearly behind, `1` between. `edge` is in our own `hand_eval`
/// units (`[-256, 256]`), not centipawns.
pub fn band_step(attacker_eval: i32, edge: u32) -> u8 {
    let edge = edge as i32;
    if attacker_eval >= edge {
        2
    } else if attacker_eval <= -edge {
        0
    } else {
        1
    }
}

/// The banded, asymmetric EF-PN scaling of an attacker-oriented `(pn, dn)`
/// pair (plan-19 R2).
///
/// The factors are EF-PN's at its own weighting (a = 10, b = 5): at an OR
/// node the proof number — the deciding one there — takes `x{11, 6, 1}` and
/// the disproof number `x{1, 11, 21}`; at an AND node the deciding disproof
/// number takes `x{1, 6, 11}` and the summed proof number `x{21, 11, 1}`.
/// A favourable attacker eval therefore makes the mover's deciding number
/// cheaper at the OR node and dearer at the AND node. Rows are `step`
/// 0 (defender ahead), 1 (even), 2 (attacker ahead).
pub fn scale_banded(pn: u32, dn: u32, step: u8, attacker_node: bool) -> (u32, u32) {
    const OR_PN: [u64; 3] = [11, 6, 1];
    const OR_DN: [u64; 3] = [1, 11, 21];
    const AND_PN: [u64; 3] = [21, 11, 1];
    const AND_DN: [u64; 3] = [1, 6, 11];
    let i = usize::from(step.min(2));
    let (fp, fd) = if attacker_node {
        (OR_PN[i], OR_DN[i])
    } else {
        (AND_PN[i], AND_DN[i])
    };
    let pn = (u64::from(pn) * fp).min(u64::from(OPEN_MAX)) as u32;
    let dn = (u64::from(dn) * fd).min(u64::from(OPEN_MAX)) as u32;
    (pn.max(1), dn.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_factor_is_16_and_scaling_is_identity() {
        assert_eq!(factor(0), 16);
        assert_eq!(scale(4, 16, 0), (4, 16));
    }

    #[test]
    fn weight_16_is_the_plan3_scale() {
        for e in [-256, -37, 0, 9, 64, 256] {
            assert_eq!(scale_weighted(9, 11, e, 16), scale(9, 11, e));
        }
        assert_eq!(scale_weighted(4, 16, 100, 0), (4, 16));
        assert_eq!(factor_weighted(256, 0), 16);
        assert_eq!(factor_weighted(256, 32), 32);
    }

    #[test]
    fn factor_is_clamped() {
        assert_eq!(factor(-1000), 8);
        assert_eq!(factor(1000), 32);
        assert_eq!(factor(-8), 8);
        assert_eq!(factor(16), 32);
    }

    #[test]
    fn favourable_eval_lowers_proof_number() {
        let (pn_low, dn_high) = scale(9, 9, 8);
        let (pn_mid, dn_mid) = scale(9, 9, 0);
        let (pn_high, dn_low) = scale(9, 9, -8);
        assert!(pn_low < pn_mid && pn_mid < pn_high);
        assert!(dn_high > dn_mid && dn_mid > dn_low);
        assert!(pn_low >= 1 && dn_low >= 1);
    }

    #[test]
    fn eval_is_bounded_and_side_relative() {
        let b = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 w - - 0 1").unwrap();
        let e = hand_eval(&b);
        assert!((-256..=256).contains(&e));
        assert!(e > 0, "white has a queen: {e}");
        let flipped = Board::from_fen("4k3/8/8/8/8/8/8/Q3K3 b - - 0 1").unwrap();
        let ef = hand_eval(&flipped);
        assert!(ef < 0, "black is down a queen: {ef}");
    }

    #[test]
    fn material_direction() {
        let white = Board::from_fen("4k3/8/8/8/8/8/8/3QK3 w - - 0 1").unwrap();
        let plain = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(hand_eval(&white) > hand_eval(&plain));
    }
}
