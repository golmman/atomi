//! Exact null-threat replay (plan 18, `--guide-null-exact`).
//!
//! At a defender node the attacker may have a winning move if the defender
//! passed (the "null" position). Provari's `replay_threat` finds such a move,
//! then replays its proof on every still-open defence: a defence that parries
//! the threat starts lower, one that does not is proven outright.
//!
//! The null board is **ephemeral**. It is derived from the current board with
//! the side to move replaced and the en-passant square cleared
//! ([`Board::null_for`], plan 31 F2), used for one exact ply, and dropped. It
//! is never keyed, looked up, stored, put on `line` or emitted; the position
//! keys and proofs that leave this module are the real children of the
//! current node.
//!
//! This is deliberately narrower than provari's null replay: it finds an
//! immediate explosion/mate, a move into a stored proof, or an exposed
//! mate-in-two (every defence lets the attacker explode). Check-every-evasion,
//! double-threat and `recomputed` proofs are out of scope for this flag.

use crate::guide::threat;
use crate::rules;
use crate::search::value::Value;
use crate::table::{CachedLimit, NullCache, Tables};
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::{for_each_legal_with_state, generate_legal};
use atomic_movegen::types::{Color, Move, MoveList};
use std::ops::ControlFlow;

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
/// move and no en-passant target.
///
/// Plan 31 F2: [`Board::null_for`] replaces the FEN round-trip (write → split
/// → join → parse). The result is the same board the round-trip produced —
/// pieces, castling rights, cleared ep square, half-move clock, fullmove
/// number and the derived game ply — checked in
/// `tests/explosion_differential.rs`. It stays ephemeral either way: never
/// keyed, looked up, stored, put on `line` or emitted.
pub fn null_board(board: &Board, attacker: Color) -> Board {
    board.null_for(attacker)
}

/// The work a null replay spent. `moves` is the existing auxiliary work
/// unit (the null movegen); `replies` is observation-only (plan 25).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NullWork {
    /// Null moves scanned (the auxiliary work unit).
    pub moves: u64,
    /// Defender replies examined in the exposed-limit pass.
    pub replies: u64,
}

/// Find an exact winning move for `attacker` in the null position and return
/// it with the null board. `remaining` bounds the proof length. `screen`
/// enables the plan-25 NULL-S pass-2 signature screen (default off).
pub fn null_win(
    board: &Board,
    attacker: Color,
    tables: &mut Tables,
    remaining: u16,
    screen: bool,
    work: &mut NullWork,
) -> Option<(Board, NullWin)> {
    if board.side_to_move() == attacker {
        return None;
    }
    let null = null_board(board, attacker);
    let mut moves = MoveList::new();
    generate_legal(&null, &mut moves);
    work.moves = u64::from(moves.len() as u32);
    // One scratch board for both passes: make, inspect, unmake. The cached
    // attack fields are not restored by `undo_move`, but every query below
    // populates its own `StateInfo`, so no stale cache is read.
    let mut scratch = null.clone();
    let mut state = StateInfo::new();
    // Pass 1: a direct win or a stored proof (shorter proofs first).
    for &m in moves.as_slice() {
        scratch.do_move(m, &mut state);
        if rules::status(&scratch, attacker) == rules::Status::Won {
            return Some((
                null,
                NullWin {
                    mv: m,
                    length: 1,
                    limit: 99,
                },
            ));
        }
        let key = rules::key(&scratch);
        let counter = scratch.rule50();
        if let Some(Value::Proved { length, limit, .. }) = tables.lookup(key, counter)
            && length <= remaining
        {
            let limit = if scratch.rule50() == 0 {
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
            ));
        }
        scratch.undo_move(m, &state);
    }
    // NULL-S: pass 2 only when the null position has an immediate explosion
    // threat. Pass 1 is untouched; the screen is a heuristic (tree growth is
    // reported, not gated).
    if screen && threat::signature_for(&null, attacker).is_empty() {
        return None;
    }
    // Pass 2: every defence exposes the commoner (a mate in two).
    let cache = tables.null_cache_mut();
    for &m in moves.as_slice() {
        scratch.do_move(m, &mut state);
        if let Some(limit) = exposed_limit(&scratch, attacker, work, cache) {
            return Some((
                null,
                NullWin {
                    mv: m,
                    length: 2,
                    limit,
                },
            ));
        }
        scratch.undo_move(m, &state);
    }
    None
}

/// If every legal defence from `child` (defender to move) lets `attacker`
/// explode the defender's commoner at once, return the proof limit.
///
/// Plan 25 NULL-V1: the reply test is the board-free closure (defender
/// self-explosion, material draw, or [`threat::immediate_explosion_bits`])
/// instead of `rules::status` + `immediate_explosion`, and one scratch board
/// is reused for every reply.
fn exposed_limit(
    child: &Board,
    attacker: Color,
    work: &mut NullWork,
    cache: &mut NullCache,
) -> Option<u8> {
    if child.rule50() >= 100 {
        return None;
    }
    let key = rules::key(child);
    let rule50 = child.rule50() as u8;
    if let Some(cached) = cache.get(key, rule50, attacker) {
        return match cached {
            CachedLimit::Negative => None,
            CachedLimit::Positive(limit) => Some(limit),
        };
    }
    let result = exposed_limit_uncached(child, attacker, work);
    cache.insert(key, rule50, attacker, result);
    result
}

/// The uncached body of [`exposed_limit`] (plan-26 EARLY-REPLY, plan-31 F3).
///
/// The replies come from the **legal stream**
/// ([`for_each_legal_with_state`]) instead of a materialised pseudo-legal
/// list: the first reply that is not accepted ends the scan with
/// [`ControlFlow::Break`], and no list is built. The stream visits exactly
/// what `generate_legal` collects, in the same order, so the result and the
/// reply count are the ones the list path computed; `limit` is a max over the
/// accepted replies, i.e. order-free anyway.
fn exposed_limit_uncached(child: &Board, attacker: Color, work: &mut NullWork) -> Option<u8> {
    // Two populated states: one for the child's own counts (the stream reads
    // `commoners_count`), one reused by the scratch make/unmake. Sharing them
    // diverged in the plan-26 probe.
    let mut child_state = StateInfo::new();
    child.populate_state(&mut child_state);
    let defender = attacker.flip();
    let mut limit = 99u8;
    let mut scratch = child.clone();
    let mut state = StateInfo::new();
    let mut any_legal = false;
    let mut rejected = false;
    let _ = for_each_legal_with_state(child, &child_state, &mut |reply| {
        any_legal = true;
        work.replies += 1;
        scratch.do_move(reply, &mut state);
        if scratch.rule50() >= 100 {
            rejected = true;
            return ControlFlow::Break(());
        }
        // The closure must agree with `rules::status` + `immediate_explosion`:
        // a defender self-explosion is `Won`; a missing attacker commoner, a
        // material draw or a failed explosion is `Failed`/`Open` without a
        // win.
        let accepted = if scratch.commoners(defender).is_empty() {
            true
        } else if scratch.commoners(attacker).is_empty() || rules::material_draw(&scratch) {
            false
        } else {
            threat::has_explosion(&scratch, attacker)
        };
        if !accepted {
            rejected = true;
            return ControlFlow::Break(());
        }
        if scratch.rule50() == 0 {
            limit = 98;
        }
        scratch.undo_move(reply, &state);
        ControlFlow::Continue(())
    });
    if rejected || !any_legal {
        return None;
    }
    Some(limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(fen: &str) -> Board {
        Board::from_fen(fen).unwrap()
    }

    /// The pre-plan-25 clone-based reply test, kept as the differential
    /// oracle for the closure.
    fn exposed_limit_reference(child: &Board, attacker: Color, work: &mut NullWork) -> Option<u8> {
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
            work.replies += 1;
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

    /// The pre-plan-25 clone-per-move `null_win`, kept as the differential
    /// oracle for the scratch-board path.
    fn null_win_reference(
        board: &Board,
        attacker: Color,
        tables: &Tables,
        remaining: u16,
    ) -> Option<(Board, NullWin, NullWork)> {
        if board.side_to_move() == attacker {
            return None;
        }
        let null = null_board(board, attacker);
        let mut moves = MoveList::new();
        generate_legal(&null, &mut moves);
        let mut work = NullWork {
            moves: u64::from(moves.len() as u32),
            replies: 0,
        };
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
                    work,
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
                    work,
                ));
            }
        }
        for &m in moves.as_slice() {
            let mut child = null.clone();
            child.do_move(m, &mut StateInfo::new());
            if let Some(limit) = exposed_limit_reference(&child, attacker, &mut work) {
                return Some((
                    null,
                    NullWin {
                        mv: m,
                        length: 2,
                        limit,
                    },
                    work,
                ));
            }
        }
        None
    }

    /// A small fixture corpus: the three bench cells, the module's own
    /// positions, a material draw and a clock position.
    const FIXTURES: &[&str] = &[
        "k7/p7/8/8/8/8/8/R3K3 b - - 0 1",
        "7k/7p/8/8/8/8/8/R3K3 b - - 0 1",
        "4k3/8/8/8/8/8/8/4K3 b - - 0 1",
        "4k3/8/8/8/8/8/8/R3K3 w - - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w - - 99 1",
        "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
        "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
        "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
    ];

    #[test]
    fn null_board_flips_the_mover_and_clears_ep() {
        let b = board("4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2");
        let n = null_board(&b, Color::Black);
        assert_eq!(n.side_to_move(), Color::Black);
        assert_eq!(n.legal_ep_square(), None);
    }

    #[test]
    fn immediate_explosion_is_found() {
        // Black to move; White's rook on a1 can take a7 and blast the black
        // commoner on a8 if White were to move.
        let b = board("k7/p7/8/8/8/8/8/R3K3 b - - 0 1");
        let mut tables = Tables::new(16, true);
        let mut work = NullWork::default();
        let (null, win) =
            null_win(&b, Color::White, &mut tables, u16::MAX, false, &mut work).expect("null win");
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
        let null = null_board(&b, Color::White);
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
        let mut work = NullWork::default();
        let (_, win) =
            null_win(&b, Color::White, &mut tables, u16::MAX, false, &mut work).expect("null win");
        assert_eq!((win.mv, win.length), (mv, 2));
    }

    #[test]
    fn no_null_win_on_a_quiet_position() {
        let b = board("4k3/8/8/8/8/8/8/4K3 b - - 0 1");
        let mut tables = Tables::new(16, true);
        let mut work = NullWork::default();
        assert!(null_win(&b, Color::White, &mut tables, u16::MAX, false, &mut work).is_none());
    }

    #[test]
    fn a_null_board_is_never_keyed_into_the_tables() {
        // The null board is built and dropped: the tables stay empty.
        let b = board("k7/p7/8/8/8/8/8/R3K3 b - - 0 1");
        let tables = Tables::new(16, true);
        let null = null_board(&b, Color::White);
        assert!(tables.lookup(rules::key(&null), null.rule50()).is_none());
    }

    #[test]
    fn closure_matches_the_clone_reference_on_every_fixture_child() {
        for fen in FIXTURES {
            let b = board(fen);
            let mut moves = MoveList::new();
            generate_legal(&b, &mut moves);
            for &m in moves.as_slice() {
                let mut child = b.clone();
                child.do_move(m, &mut StateInfo::new());
                let attacker = child.side_to_move().flip();
                let mut closure_work = NullWork::default();
                let mut reference_work = NullWork::default();
                let mut cache = NullCache::new(1 << 10);
                let closure = exposed_limit(&child, attacker, &mut closure_work, &mut cache);
                let reference = exposed_limit_reference(&child, attacker, &mut reference_work);
                assert_eq!(
                    closure, reference,
                    "closure mismatch on {fen} after {m:?} (attacker {attacker:?})"
                );
                assert_eq!(
                    closure_work.replies, reference_work.replies,
                    "reply count mismatch on {fen} after {m:?}"
                );
            }
        }
    }

    #[test]
    fn scratch_null_win_matches_the_clone_reference() {
        let mut tables = Tables::new(16, true);
        for fen in FIXTURES {
            let b = board(fen);
            let attacker = b.side_to_move().flip();
            let mut work = NullWork::default();
            let got = null_win(&b, attacker, &mut tables, u16::MAX, false, &mut work);
            let reference = null_win_reference(&b, attacker, &tables, u16::MAX);
            match (got, reference) {
                (None, None) => {}
                (Some((null, win)), Some((rnull, rwin, rwork))) => {
                    assert_eq!(win, rwin, "null win mismatch on {fen}");
                    assert_eq!(work, rwork, "null work mismatch on {fen}");
                    assert_eq!(
                        rules::key(&null),
                        rules::key(&rnull),
                        "null board mismatch on {fen}"
                    );
                }
                (got, reference) => panic!(
                    "null win presence mismatch on {fen}: got {got:?}, reference {reference:?}"
                ),
            }
        }
    }

    #[test]
    fn cache_matches_an_uncached_call_on_a_repeated_position() {
        for fen in FIXTURES {
            let b = board(fen);
            let mut moves = MoveList::new();
            generate_legal(&b, &mut moves);
            for &m in moves.as_slice() {
                let mut child = b.clone();
                child.do_move(m, &mut StateInfo::new());
                if child.rule50() >= 100 {
                    continue;
                }
                let attacker = child.side_to_move().flip();
                let mut cache = NullCache::new(1 << 10);
                let mut first_work = NullWork::default();
                let first = exposed_limit(&child, attacker, &mut first_work, &mut cache);
                let mut second_work = NullWork::default();
                let second = exposed_limit(&child, attacker, &mut second_work, &mut cache);
                assert_eq!(first, second, "cache mismatch on {fen} after {m:?}");
                assert_eq!(second_work.replies, 0, "cache hit did work on {fen}");
                assert_eq!(cache.hits(), 1, "cache did not hit on {fen}");
            }
        }
    }

    #[test]
    fn undo_move_restores_every_field_the_null_pass_reads() {
        for fen in FIXTURES {
            let b = board(fen);
            let mut moves = MoveList::new();
            generate_legal(&b, &mut moves);
            for &m in moves.as_slice() {
                let mut scratch = b.clone();
                let before = (
                    scratch.fen_with(atomic_movegen::types::EnPassantMode::Legal),
                    rules::key(&scratch),
                    scratch.rule50(),
                    scratch.side_to_move(),
                    scratch.commoners(Color::White),
                    scratch.commoners(Color::Black),
                );
                let mut state = StateInfo::new();
                scratch.do_move(m, &mut state);
                scratch.undo_move(m, &state);
                let after = (
                    scratch.fen_with(atomic_movegen::types::EnPassantMode::Legal),
                    rules::key(&scratch),
                    scratch.rule50(),
                    scratch.side_to_move(),
                    scratch.commoners(Color::White),
                    scratch.commoners(Color::Black),
                );
                assert_eq!(before, after, "undo mismatch on {fen} after {m:?}");
            }
        }
    }
}
