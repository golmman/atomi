// Justification for >10 KB: the exact replay, the null-guide entry point and their differential tests are one contract.

//! Exact sibling proof replay (plan 13 Phase C, `--guide-replay-exact`).
//!
//! Kawano-style simulation: replay a proved sibling's proof on another
//! sibling position. The replay is **exact**: every defender reply is
//! verified against a stored proof, an immediate win, or the same reply in
//! the guide position. It only returns a proof it has verified; the caller
//! stores the attacker proofs it collected.
//!
//! The guide is the proved sibling position (attacker to move). At an
//! attacker node the guide's stored proof supplies the move; at a defender
//! node every legal reply of the new position is checked, and the same reply
//! in the guide supplies the next guide position. A position on the search
//! line or the replay line is never used (a cycle is not a proof).
//!
//! This module never writes a table and never emits an event; it returns the
//! proofs for the caller to store. It is default off and measurement only.

use crate::guide::threat;
use crate::rules;
use crate::search::value::Value;
use crate::table::{ProofEntry, Tables};
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, Move, MoveList};

/// The default replay budget (attacker nodes visited per sibling).
pub const REPLAY_BUDGET: u32 = 1000;

/// A defender-side replay result: `(length, limit, proofs)`.
type Reached = (u16, u8, Vec<(u64, ProofEntry)>);

/// A verified replay proof for the sibling position.
pub struct ReplayProof {
    /// Proof length in attacker moves from the sibling position.
    pub length: u16,
    /// Highest 50-move counter at which the proof holds.
    pub limit: u8,
    /// The sibling position's winning move.
    pub mv: Move,
    /// Attacker proofs to store, in discovery order (the sibling's own proof
    /// is last).
    pub proofs: Vec<(u64, ProofEntry)>,
}

/// Replay `guide`'s proof on `pos` (both attacker to move). `remaining` is
/// the maximum proof length in attacker moves; `search_line` is the current
/// search path (a cycle there is not a proof).
#[allow(clippy::too_many_arguments)]
pub fn simulate(
    pos: &Board,
    guide: &Board,
    attacker: Color,
    remaining: u16,
    budget: &mut u32,
    line: &mut Vec<u64>,
    search_line: &[u64],
    tables: &Tables,
) -> Option<ReplayProof> {
    simulate_inner(
        pos,
        guide,
        None,
        attacker,
        remaining,
        budget,
        line,
        search_line,
        tables,
    )
}

/// Replay a proof that is not in the tables (the null position's proof) on
/// `pos`. `guide_mv` is the guide's winning move and `guide_length` its proof
/// length; deeper guide positions are looked up normally. The null guide
/// itself is never keyed or stored.
#[allow(clippy::too_many_arguments)]
pub fn simulate_with_guide_proof(
    pos: &Board,
    guide: &Board,
    guide_mv: Move,
    guide_length: u16,
    attacker: Color,
    remaining: u16,
    budget: &mut u32,
    line: &mut Vec<u64>,
    search_line: &[u64],
    tables: &Tables,
) -> Option<ReplayProof> {
    simulate_inner(
        pos,
        guide,
        Some((guide_mv, guide_length)),
        attacker,
        remaining,
        budget,
        line,
        search_line,
        tables,
    )
}

#[allow(clippy::too_many_arguments)]
fn simulate_inner(
    pos: &Board,
    guide: &Board,
    guide_proof: Option<(Move, u16)>,
    attacker: Color,
    remaining: u16,
    budget: &mut u32,
    line: &mut Vec<u64>,
    search_line: &[u64],
    tables: &Tables,
) -> Option<ReplayProof> {
    match rules::status(pos, attacker) {
        // A terminal win has no move; the caller's parent already sees it.
        rules::Status::Won => return None,
        rules::Status::Failed { .. } => return None,
        rules::Status::Open => {}
    }
    let key = rules::key(pos);
    let counter = pos.rule50();
    if let Some(Value::Proved { length, limit, .. }) = tables.lookup(key, counter)
        && length <= remaining
    {
        // Already proved: no new proof, but the caller needs a move.
        if let Some(Value::Proved { mv: Some(mv), .. }) = tables.lookup(key, counter) {
            return Some(ReplayProof {
                length,
                limit,
                mv,
                proofs: Vec::new(),
            });
        }
        return None;
    }
    if *budget == 0 || remaining == 0 || line.contains(&key) || search_line.contains(&key) {
        return None;
    }
    *budget -= 1;
    let gmv = match guide_proof {
        Some((mv, length)) => {
            if length > remaining {
                return None;
            }
            mv
        }
        None => {
            let gkey = rules::key(guide);
            let gcounter = guide.rule50();
            match tables.lookup(gkey, gcounter) {
                Some(Value::Proved { mv: Some(mv), .. }) => mv,
                _ => return None,
            }
        }
    };
    if !pos.is_legal(gmv) {
        return None;
    }
    let mut after = pos.clone();
    after.do_move(gmv, &mut StateInfo::new());
    let mut gafter = guide.clone();
    gafter.do_move(gmv, &mut StateInfo::new());
    line.push(key);
    let reached = simulate_defences(
        &after,
        &gafter,
        attacker,
        remaining - 1,
        budget,
        line,
        search_line,
        tables,
    );
    line.pop();
    let (clen, clim, mut proofs) = reached?;
    let length = clen.saturating_add(1);
    let limit = if after.rule50() == 0 {
        99
    } else {
        clim.saturating_sub(1).min(99)
    };
    proofs.push((
        key,
        ProofEntry {
            mv: gmv.to_bits(),
            length,
            limit,
        },
    ));
    Some(ReplayProof {
        length,
        limit,
        mv: gmv,
        proofs,
    })
}

/// The defender's side of [`simulate`]: every legal reply must be answered.
#[allow(clippy::too_many_arguments)]
fn simulate_defences(
    pos: &Board,
    guide: &Board,
    attacker: Color,
    remaining: u16,
    budget: &mut u32,
    line: &mut Vec<u64>,
    search_line: &[u64],
    tables: &Tables,
) -> Option<Reached> {
    match rules::status(pos, attacker) {
        rules::Status::Won => return Some((0, 99, Vec::new())),
        rules::Status::Failed { .. } => return None,
        rules::Status::Open => {}
    }
    let key = rules::key(pos);
    let counter = pos.rule50();
    if let Some(Value::Proved { length, limit, .. }) = tables.lookup(key, counter)
        && length <= remaining
    {
        return Some((length, limit, Vec::new()));
    }
    if counter >= 100 || line.contains(&key) || search_line.contains(&key) {
        return None;
    }
    let mut moves = MoveList::new();
    generate_legal(pos, &mut moves);
    if moves.is_empty() {
        return None;
    }
    line.push(key);
    let mut length = 0u16;
    let mut limit = 99u8;
    let mut proofs = Vec::new();
    for &m in moves.as_slice() {
        let mut next = pos.clone();
        next.do_move(m, &mut StateInfo::new());
        let (rlen, rlim, rproofs) = match rules::status(&next, attacker) {
            rules::Status::Won => (0u16, 99u8, Vec::new()),
            rules::Status::Failed { .. } => {
                line.pop();
                return None;
            }
            rules::Status::Open => {
                let nkey = rules::key(&next);
                let ncounter = next.rule50();
                if let Some(Value::Proved { length, limit, .. }) = tables.lookup(nkey, ncounter) {
                    if length <= remaining {
                        (length, limit, Vec::new())
                    } else {
                        line.pop();
                        return None;
                    }
                } else if next.rule50() < 100
                    && let Some(explosion) = threat::immediate_explosion(&next, attacker)
                {
                    // The defence lets the attacker explode at once: a
                    // one-move proof that needs no guide. Provari's
                    // `exposing` shortcut. The child's own proof is returned
                    // so the caller stores and emits it; the writer needs it.
                    (
                        1,
                        99,
                        vec![(
                            nkey,
                            ProofEntry {
                                mv: explosion.to_bits(),
                                length: 1,
                                limit: 99,
                            },
                        )],
                    )
                } else if guide.is_legal(m) {
                    let mut gnext = guide.clone();
                    gnext.do_move(m, &mut StateInfo::new());
                    match simulate(
                        &next,
                        &gnext,
                        attacker,
                        remaining,
                        budget,
                        line,
                        search_line,
                        tables,
                    ) {
                        Some(p) => (p.length, p.limit, p.proofs),
                        None => {
                            line.pop();
                            return None;
                        }
                    }
                } else {
                    line.pop();
                    return None;
                }
            }
        };
        length = length.max(rlen);
        let l = if next.rule50() == 0 {
            99
        } else {
            rlim.saturating_sub(1).min(99)
        };
        limit = limit.min(l);
        proofs.extend(rproofs);
    }
    line.pop();
    Some((length, limit, proofs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Tables;

    fn board(fen: &str) -> Board {
        Board::from_fen(fen).unwrap()
    }

    #[test]
    fn a_proved_position_replays_itself() {
        // A mate-in-1: the guide's proof move is legal in the same position.
        let b = board("6rk/8/8/8/8/8/8/6QK w - - 0 1");
        let mut tables = Tables::new(16, true);
        let key = rules::key(&b);
        let mv = b.move_from_uci("g1g8").unwrap();
        tables.store(
            key,
            &Value::Proved {
                length: 1,
                limit: 99,
                mv: Some(mv),
            },
            0,
        );
        let mut budget = REPLAY_BUDGET;
        let mut line = Vec::new();
        let p = simulate(
            &b,
            &b,
            Color::White,
            1,
            &mut budget,
            &mut line,
            &[],
            &tables,
        )
        .expect("self replay");
        assert_eq!((p.length, p.limit, p.mv), (1, 99, mv));
    }

    #[test]
    fn a_missing_guide_proof_gives_up() {
        let b = board("6rk/8/8/8/8/8/8/6QK w - - 0 1");
        let tables = Tables::new(16, true);
        let mut budget = REPLAY_BUDGET;
        let mut line = Vec::new();
        assert!(
            simulate(
                &b,
                &b,
                Color::White,
                1,
                &mut budget,
                &mut line,
                &[],
                &tables
            )
            .is_none()
        );
    }

    #[test]
    fn a_null_guide_proof_replays_on_a_child() {
        // Black to move; if Black passes, White's Rxa7 blasts the commoner
        // on a8. After Kb8 the same move still blasts it.
        let b = board("k7/p7/8/8/8/8/8/R3K3 b - - 0 1");
        let null = crate::guide::null::null_board(&b, Color::White).unwrap();
        let mv = null.move_from_uci("a1a7").unwrap();
        let mut child = b.clone();
        child.do_move(b.move_from_uci("a8b8").unwrap(), &mut StateInfo::new());
        let tables = Tables::new(16, true);
        let mut budget = REPLAY_BUDGET;
        let mut line = Vec::new();
        let p = simulate_with_guide_proof(
            &child,
            &null,
            mv,
            1,
            Color::White,
            u16::MAX,
            &mut budget,
            &mut line,
            &[],
            &tables,
        )
        .expect("null replay");
        assert_eq!((p.length, p.limit, p.mv), (1, 99, mv));
    }

    #[test]
    fn a_parrying_defence_refuses_the_null_guide_proof() {
        let b = board("k7/p7/8/8/8/8/8/R3K3 b - - 0 1");
        let null = crate::guide::null::null_board(&b, Color::White).unwrap();
        let mv = null.move_from_uci("a1a7").unwrap();
        let mut child = b.clone();
        child.do_move(b.move_from_uci("a7a6").unwrap(), &mut StateInfo::new());
        let tables = Tables::new(16, true);
        let mut budget = REPLAY_BUDGET;
        let mut line = Vec::new();
        assert!(
            simulate_with_guide_proof(
                &child,
                &null,
                mv,
                1,
                Color::White,
                u16::MAX,
                &mut budget,
                &mut line,
                &[],
                &tables,
            )
            .is_none()
        );
    }

    #[test]
    fn a_stored_proof_is_exact_even_on_the_line() {
        // A stored proof is a complete proof tree, so it is usable even when
        // the position is on the current line; only open positions are
        // cycle-checked.
        let b = board("6rk/8/8/8/8/8/8/6QK w - - 0 1");
        let mut tables = Tables::new(16, true);
        let key = rules::key(&b);
        let mv = b.move_from_uci("g1g8").unwrap();
        tables.store(
            key,
            &Value::Proved {
                length: 1,
                limit: 99,
                mv: Some(mv),
            },
            0,
        );
        let mut budget = REPLAY_BUDGET;
        let mut line = vec![key];
        assert!(
            simulate(
                &b,
                &b,
                Color::White,
                1,
                &mut budget,
                &mut line,
                &[],
                &tables
            )
            .is_some()
        );
    }
}
