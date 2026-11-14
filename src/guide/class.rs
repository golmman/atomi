// Justification for >10 KB: the product classes, the plan-19 exact-number
// predicates (R4/R5/R6) and their differential tests are one contract; the
// tests are the bulk.

//! Move classes for the hand guide (D21).
//!
//! A class is a label on a legal move, computed from the parent, the move's
//! [`Effect`] and (when pass 1 built the child) the child board. It only
//! selects a starting-number pair and a tie-break order; it never decides a
//! value.

use crate::mover::{Effect, Mover};
use atomic_movegen::attacks::{attacks_from, king_attacks};
use atomic_movegen::board::Board;
use atomic_movegen::types::{Bitboard, Color, Move, PieceType, Square};

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

    /// The stable diagnostic name (`classify` output).
    pub fn as_str(self) -> &'static str {
        match self {
            MoveClass::Threat => "Threat",
            MoveClass::WeakThreat => "WeakThreat",
            MoveClass::Hanging => "Hanging",
            MoveClass::Quiet => "Quiet",
            MoveClass::Evasion => "Evasion",
        }
    }
}

/// The plan-19 exact-number switches (R4/R5).
///
/// With `threat_count` the class and the starting proof number of a fresh OR
/// child come from the child's explosion-target count; with `hanging_quiet`
/// the hanging class is the cheap-blast test instead of "any capture or
/// attacked piece". Numbers are our own documented choices: `1` for a double
/// threat, `4`/`8` for one threat (8 when the threat is cheap to parry), `15`
/// for a plain move the defender blasts cheaply, `9` otherwise.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Exact {
    /// R4: number the OR child by its explosion-target count.
    pub threat_count: bool,
    /// R5: hanging means "the defender blasts the moved piece cheaply".
    pub hanging_quiet: bool,
}

/// A fresh child's class and, when the exact numbers decide it, its OR
/// starting proof number (plan-19 R4/R5).
pub type ClassAndNumber = (MoveClass, Option<u32>);

/// `true` when a capture at `to` lands in the blast zone of `commoner`.
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
            && crate::guide::threat::has_explosion(c, them)
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

/// The number of enemy pieces whose capture would explode the enemy commoner
/// after `m` (plan-19 R4).
///
/// A built child hands its own signature; a non-built child is quiet (a
/// capture, promotion, special or checking move is always built), so the
/// count is read from the parent with the move's occupancy `occ` and the
/// moved piece's new attack set.
fn threat_count(board: &Board, m: Move, child: Option<&Board>) -> u32 {
    let us = board.side_to_move();
    if let Some(c) = child {
        return crate::guide::threat::signature_for(c, us).count();
    }
    let commoner = board.commoners(us.flip());
    if commoner.is_empty() {
        return 0;
    }
    let from_bb = Bitboard::square_bb(m.from_sq());
    let to_bb = Bitboard::square_bb(m.to_sq());
    let occ = (board.occupied() & !from_bb) | to_bb;
    let moved = board
        .piece_on(m.from_sq())
        .type_of()
        .map_or(Bitboard::EMPTY, |pt| attacks_from(pt, us, m.to_sq(), occ));
    let mut targets = Board::blast_zone(commoner.lsb()) & board.pieces_color(us.flip());
    let mut n = 0;
    while !targets.is_empty() {
        let sq = targets.pop_lsb();
        // The moved piece's own attacks replace its stale square; every other
        // attacker is read with the post-move occupancy.
        let others = board.attackers_to_color(sq, us, occ) & !from_bb;
        if !(moved & Bitboard::square_bb(sq)).is_empty() || !others.is_empty() {
            n += 1;
        }
    }
    n
}

/// Material of one piece in our own documented table (plan-19 R5); the
/// commoner and the pawn-pair have no exchange value here.
fn material(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        _ => 0,
    }
}

/// Whether the defender can blast the piece a **quiet** move just moved
/// without losing material (plan-19 R5).
///
/// Our own definition: after the quiet move, an enemy capture on the landing
/// square, its blast included, must not cost the defender material in our
/// table — the value of the moved piece plus what the blast destroys (theirs
/// lost, ours won, pawns survive an atomic blast and the commoner is
/// priceless), minus the cheapest enemy capturer (0 when the blast takes it).
/// A commoner in the blast zone is an immediate win, not a cheap blast.
pub fn blasted_cheaply(board: &Board, m: Move, child: Option<&Board>) -> bool {
    let from = m.from_sq();
    let to = m.to_sq();
    if board.is_capture(m)
        || m.is_promotion()
        || m.is_en_passant()
        || m.is_castling()
        || matches!(
            board.piece_on(from).type_of(),
            None | Some(PieceType::Commoner) | Some(PieceType::Pawn)
        )
    {
        return false;
    }
    let us = board.side_to_move();
    let them = us.flip();
    let from_bb = Bitboard::square_bb(from);
    let to_bb = Bitboard::square_bb(to);
    let occ = child.map_or((board.occupied() & !from_bb) | to_bb, Board::occupied);
    let ring = Board::blast_zone(to) & occ;
    if !(ring & board.commoners(us)).is_empty() {
        return false;
    }
    let capturers =
        board.attackers_to_color(to, them, occ) & board.pieces_color(them) & !board.commoners(them);
    if capturers.is_empty() {
        return false;
    }
    let mut gain = board.piece_on(from).type_of().map_or(0, material);
    let mut destroyed = ring & !board.pieces_pt(PieceType::Pawn);
    while !destroyed.is_empty() {
        let sq = destroyed.pop_lsb();
        let Some(pt) = board.piece_on(sq).type_of() else {
            continue;
        };
        gain += if !(board.pieces_color(us) & Bitboard::square_bb(sq)).is_empty() {
            -material(pt)
        } else {
            material(pt)
        };
    }
    let mut capturers = capturers;
    let mut cost = i32::MAX;
    while !capturers.is_empty() {
        let sq = capturers.pop_lsb();
        if (ring & Bitboard::square_bb(sq)).is_empty() {
            cost = cost.min(board.piece_on(sq).type_of().map_or(0, material));
        } else {
            cost = 0;
        }
    }
    gain - cost >= 0
}

/// Whether a defence is one a defender would try first (plan-19 R1's AND
/// class 2 and R6's `UNLIKELY`): the inverse of "not a check, no blast
/// capture of an attacker piece, no king contact".
pub fn defence_likely_to_refute(
    board: &Board,
    m: Move,
    child: Option<&Board>,
    attacker: Color,
) -> bool {
    let us = board.side_to_move();
    let them = us.flip();
    let from_bb = Bitboard::square_bb(m.from_sq());
    let to_bb = Bitboard::square_bb(m.to_sq());
    let occ = child.map_or((board.occupied() & !from_bb) | to_bb, Board::occupied);
    let check = child.is_some_and(|c| !c.checkers().is_empty());
    // A capture whose blast removes an attacker piece other than a pawn.
    let attackers =
        board.pieces_color(attacker) & !board.pieces_color_pt(attacker, PieceType::Pawn);
    let blast_capture = board.is_capture(m) && !(attackers & !occ).is_empty();
    // King contact: our commoner touches theirs after the move, or attacks an
    // empty square beside theirs.
    let ours = if from_bb & board.commoners(us) != Bitboard::EMPTY {
        to_bb
    } else {
        board.commoners(us)
    };
    let theirs = board.commoners(them);
    let contact = (!ours.is_empty() && !theirs.is_empty())
        && (!(king_attacks(theirs.lsb()) & ours).is_empty()
            || !(king_attacks(theirs.lsb()) & king_attacks(ours.lsb()) & !occ).is_empty());
    check || blast_capture || contact
}

/// Classify `m` for the plan-19 stack: the class for ordering and, when R4
/// decides it, the exact OR starting proof number.
#[allow(clippy::too_many_arguments)]
pub fn classify_exact(
    exact: &Exact,
    mover: &Mover,
    board: &Board,
    m: Move,
    effect: &Effect,
    child: Option<&Board>,
    in_check: bool,
) -> ClassAndNumber {
    if exact.threat_count && !in_check {
        let count = threat_count(board, m, child);
        let cheap = exact.hanging_quiet && blasted_cheaply(board, m, child);
        return match count {
            0 if cheap && mover.is_plain(board, m) => (MoveClass::Hanging, Some(15)),
            0 => (MoveClass::Quiet, Some(9)),
            1 if cheap => (MoveClass::WeakThreat, Some(8)),
            1 => (MoveClass::Threat, Some(4)),
            _ => (MoveClass::Threat, Some(1)),
        };
    }
    let mut class = classify(board, m, effect, child, in_check);
    // R5 alone: only the quiet/hanging decision changes (a threat keeps its
    // class and its table number).
    if exact.hanging_quiet && matches!(class, MoveClass::Quiet | MoveClass::Hanging) {
        class = if blasted_cheaply(board, m, child) && mover.is_plain(board, m) {
            MoveClass::Hanging
        } else {
            MoveClass::Quiet
        };
    }
    (class, None)
}

/// The plan-19 R3 check number: the evasions whose child is still `Open`
/// after pass 1 — a decided or settled evasion does not survive — capped at
/// 8 and never zero.
pub fn surviving_evasions(children: &[crate::search::value::Child]) -> u32 {
    children
        .iter()
        .filter(|c| {
            c.class == MoveClass::Evasion
                && matches!(c.value, crate::search::value::Value::Open { .. })
        })
        .count()
        .clamp(1, 8) as u32
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

    fn mv_of(from: Square, to: Square) -> Move {
        Move::make_move(from, to)
    }

    fn child_value(
        mv: Move,
        class: MoveClass,
        value: crate::search::value::Value,
    ) -> crate::search::value::Child {
        crate::search::value::Child {
            mv,
            zeroing: false,
            class,
            value,
        }
    }

    /// R3: only evasions whose child is still `Open` survive, and the count
    /// is capped at 8.
    #[test]
    fn surviving_evasions_counts_open_evasions_only() {
        use crate::search::value::Value;
        let mv = mv_of(Square::E1, Square::E2);
        let open = Value::Open { pn: 1, dn: 1 };
        let refuted = Value::Refuted { cycle: 0, from: 0 };
        let one = vec![
            child_value(mv, MoveClass::Evasion, open),
            child_value(mv, MoveClass::Evasion, refuted),
            child_value(mv, MoveClass::Evasion, refuted),
            child_value(mv, MoveClass::Evasion, refuted),
            child_value(mv, MoveClass::Quiet, open),
        ];
        assert_eq!(surviving_evasions(&one), 1);
        let many: Vec<_> = (0..12)
            .map(|_| child_value(mv, MoveClass::Evasion, open))
            .collect();
        assert_eq!(surviving_evasions(&many), 8);
        // No surviving evasion is still one (the parent has a legal move).
        let none = vec![child_value(mv, MoveClass::Evasion, refuted)];
        assert_eq!(surviving_evasions(&none), 1);
    }

    fn exact_cfg(threat_count: bool, hanging_quiet: bool) -> Exact {
        Exact {
            threat_count,
            hanging_quiet,
        }
    }

    fn class_of(board: &Board, uci: &str, exact: &Exact) -> ClassAndNumber {
        let mut ml = MoveList::new();
        generate_legal(board, &mut ml);
        let m = (&ml)
            .into_iter()
            .find(|m| m.to_uci() == uci)
            .unwrap_or_else(|| panic!("{uci} is not legal in {board:?}"));
        let mover = Mover::new(board);
        let e = mover.effect(board, m);
        let mut child = board.clone();
        child.do_move(m, &mut StateInfo::new());
        classify_exact(exact, &mover, board, m, &e, Some(&child), false)
    }

    /// R4: the explosion-target count decides the number — two targets start
    /// at 1, one at 4, none at 9.
    #[test]
    fn threat_count_sets_the_or_number() {
        let exact = exact_cfg(true, true);
        // Ne6 attacks two pieces beside the black commoner.
        let double = Board::from_fen("3rkn2/8/8/2N5/8/8/8/4K3 w - - 0 1").unwrap();
        assert_eq!(
            class_of(&double, "c5e6", &exact),
            (MoveClass::Threat, Some(1))
        );
        // Be7 attacks one piece beside the black commoner.
        let single = Board::from_fen("3nk3/8/8/8/7B/8/8/4K3 w - - 0 1").unwrap();
        assert_eq!(
            class_of(&single, "h4e7", &exact),
            (MoveClass::Threat, Some(4))
        );
        // A quiet move with no target starts at 9.
        let quiet = Board::from_fen("3nk3/8/8/8/7B/8/8/4K3 w - - 0 1").unwrap();
        assert_eq!(
            class_of(&quiet, "h4g3", &exact),
            (MoveClass::Quiet, Some(9))
        );
        // R4 off keeps the product table numbers.
        assert_eq!(class_of(&double, "c5e6", &Exact::default()).1, None);
    }

    /// R4's board-free path (a quiet, unbuilt child) agrees with the built
    /// child's signature over the fixture playouts.
    #[test]
    fn threat_count_matches_the_built_child() {
        let mut checked = 0usize;
        for (board, _line, _attacker) in crate::testutil::positions() {
            let mover = Mover::new(&board);
            let mut ml = MoveList::new();
            generate_legal(&board, &mut ml);
            for m in &ml {
                let e = mover.effect(&board, m);
                if e.needs_board {
                    continue;
                }
                let mut child = board.clone();
                child.do_move(m, &mut StateInfo::new());
                let want =
                    crate::guide::threat::signature_for(&child, board.side_to_move()).count();
                assert_eq!(threat_count(&board, m, None), want, "{m} on {board:?}");
                checked += 1;
            }
        }
        assert!(checked > 1000, "only {checked} quiet children checked");
    }

    /// R5: a quiet move the defender blasts cheaply is hanging; one it cannot
    /// take profitably is not.
    #[test]
    fn blasted_cheaply_is_the_material_test() {
        // Nc6 attacks d4: a queen landing there is taken for a knight.
        let b = Board::from_fen("r3k3/8/2n5/8/8/8/8/3QK3 w - - 0 1").unwrap();
        assert!(blasted_cheaply(&b, mv_of(Square::D1, Square::D4), None));
        // Nobody attacks e4: no cheap blast.
        let b = Board::from_fen("r3k3/8/2n5/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        assert!(!blasted_cheaply(&b, mv_of(Square::E2, Square::E4), None));
        // A knight landing where only the queen attacks it loses material.
        let b = Board::from_fen("r3k3/8/2n5/8/8/8/8/3qKN2 w - - 0 1").unwrap();
        assert!(!blasted_cheaply(&b, mv_of(Square::F1, Square::D2), None));
        // A commoner move is never a cheap blast (it is not "quiet" here).
        let b = Board::from_fen("3nk3/8/8/8/7B/8/8/4K3 w - - 0 1").unwrap();
        assert!(!blasted_cheaply(&b, mv_of(Square::E1, Square::D1), None));
    }

    /// R6: a defence is likely to refute when it gives check, removes an
    /// attacker piece with the blast, or brings the commoners into contact.
    #[test]
    fn defence_likely_to_refute_predicate() {
        use atomic_movegen::types::Color;
        let w = Color::White;
        // A quiet pawn push is not a likely refutation.
        let b = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 b - - 0 1").unwrap();
        assert!(!defence_likely_to_refute(
            &b,
            mv_of(Square::E7, Square::E5),
            None,
            w
        ));
        // A checking move is: Rh8-h1 attacks the white commoner on e1.
        let b = Board::from_fen("4k2r/8/8/8/8/8/8/4K3 b - - 0 1").unwrap();
        let m = mv_of(Square::H8, Square::H1);
        let mut child = b.clone();
        child.do_move(m, &mut StateInfo::new());
        assert!(!child.checkers().is_empty());
        assert!(defence_likely_to_refute(&b, m, Some(&child), Color::White));
        // A capture whose blast removes a white piece (here the queen).
        let b = Board::from_fen("4k3/8/8/8/8/2n5/3Q4/4K3 b - - 0 1").unwrap();
        let m = mv_of(Square::C3, Square::D2);
        let mut child = b.clone();
        child.do_move(m, &mut StateInfo::new());
        assert!(defence_likely_to_refute(&b, m, Some(&child), w));
        // Commoner contact: the black commoner steps beside the white one.
        let b = Board::from_fen("8/8/8/8/8/8/8/4k1K1 b - - 0 1").unwrap();
        let m = mv_of(Square::E1, Square::F1);
        let mut child = b.clone();
        child.do_move(m, &mut StateInfo::new());
        assert!(defence_likely_to_refute(&b, m, Some(&child), w));
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
