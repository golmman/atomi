// Tests for `guide::null`: the exact-replay differentials (plans 18/25/28) and
// the plan-33 NULL-PIPE pass-1 pipeline. Split out of `null.rs` so the search
// module stays under the source-size convention; `use super::*` is the
// module's own namespace (this file *is* `null::tests`).

use super::*;
use crate::search::value::Value;

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
        // G1: this reference site only accepts a proof.
        if let Some(pe) = tables.proved(key, counter)
            && pe.length <= remaining
        {
            let limit = if child.rule50() == 0 {
                99
            } else {
                pe.limit.saturating_sub(1).min(99)
            };
            return Some((
                null,
                NullWin {
                    mv: m,
                    length: pe.length.saturating_add(1),
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
            (got, reference) => {
                panic!("null win presence mismatch on {fen}: got {got:?}, reference {reference:?}")
            }
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

// ---- plan 33 (NULL-PIPE): the pipelined pass 1 ------------------------

/// The pre-plan-33 byte-serial pass 1, kept as the differential oracle for
/// [`pass1`]: one move at a time, probe immediately after keying, return on
/// the first firing index.
fn pass1_reference(
    null: &Board,
    attacker: Color,
    tables: &Tables,
    remaining: u16,
) -> Option<NullWin> {
    let mut moves = MoveList::new();
    generate_legal(null, &mut moves);
    let mut scratch = null.clone();
    let mut state = StateInfo::new();
    for &m in moves.as_slice() {
        scratch.do_move(m, &mut state);
        if rules::status(&scratch, attacker) == rules::Status::Won {
            return Some(NullWin {
                mv: m,
                length: 1,
                limit: 99,
            });
        }
        let key = rules::key(&scratch);
        let counter = scratch.rule50();
        if let Some(pe) = tables.proved(key, counter)
            && pe.length <= remaining
        {
            let limit = if counter == 0 {
                99
            } else {
                pe.limit.saturating_sub(1).min(99)
            };
            return Some(NullWin {
                mv: m,
                length: pe.length.saturating_add(1),
                limit,
            });
        }
        scratch.undo_move(m, &state);
    }
    None
}

/// [`pass1`] at an explicit `K`, over the null board of `b`.
fn pass1_at<const K: usize>(
    b: &Board,
    attacker: Color,
    tables: &Tables,
    remaining: u16,
) -> Option<NullWin> {
    let null = null_board(b, attacker);
    let mut moves = MoveList::new();
    generate_legal(&null, &mut moves);
    pass1::<K>(
        &mut null.clone(),
        &mut StateInfo::new(),
        moves.as_slice(),
        attacker,
        tables,
        remaining,
    )
}

/// [`null_win`] at an explicit `K`, with the work it spent.
fn null_win_at<const K: usize>(
    b: &Board,
    attacker: Color,
    tables: &mut Tables,
    remaining: u16,
) -> (Option<(Board, NullWin)>, NullWork) {
    let mut work = NullWork::default();
    let got = null_win_pipelined::<K>(b, attacker, tables, remaining, false, &mut work);
    (got, work)
}

/// The G-40a corpus: every `FIXTURES` board and child, then every seeded
/// playout position with the defender to move (`null_win` works there).
/// The second value is how many playout positions there were in total.
fn pass1_corpus() -> (Vec<(Board, Color)>, usize) {
    let mut out = Vec::new();
    for fen in FIXTURES {
        let b = board(fen);
        out.push((b.clone(), b.side_to_move().flip()));
        let mut moves = MoveList::new();
        generate_legal(&b, &mut moves);
        for &m in moves.as_slice() {
            let mut child = b.clone();
            child.do_move(m, &mut StateInfo::new());
            out.push((child, b.side_to_move()));
        }
    }
    let playouts = crate::testutil::positions();
    let total = playouts.len();
    for (b, _line, attacker) in playouts {
        if b.side_to_move() != attacker {
            out.push((b, attacker));
        }
    }
    (out, total)
}

/// Seed a proof on one null child of every 3rd corpus position, cycling
/// usable at the counter, exactly at it, unusable and length 3 (so
/// `remaining = 1` makes it non-firing).
fn seed_proofs(tables: &mut Tables, corpus: &[(Board, Color)]) -> u64 {
    let mut seeded = 0;
    for (i, (b, attacker)) in corpus.iter().enumerate() {
        if !i.is_multiple_of(3) {
            continue;
        }
        let null = null_board(b, *attacker);
        let mut moves = MoveList::new();
        generate_legal(&null, &mut moves);
        if moves.is_empty() {
            continue;
        }
        let j = (i / 3) % moves.len();
        let mut child = null.clone();
        child.do_move(moves.as_slice()[j], &mut StateInfo::new());
        let counter = child.rule50().min(255) as u8;
        let (length, limit) = match (i / 3) % 4 {
            0 => (1u16, 99u8),
            1 => (2, counter),
            2 => (1, counter.saturating_sub(1)),
            _ => (3, 99),
        };
        tables.store(
            rules::key(&child),
            &Value::Proved {
                length,
                limit,
                mv: None,
            },
            u32::from(counter),
        );
        seeded += 1;
    }
    seeded
}

/// G-40a: the pipelined pass 1 and the whole `null_win` answer exactly like
/// the reference for `K = 1` (the byte-serial order), `K = 8` and `K = 16`
/// (shipped), over the fixtures and the seeded playouts — with an empty
/// table and with one seeded on every third position, at full and at
/// `remaining = 1`.
#[test]
fn pipelined_pass1_matches_reference() {
    let (corpus, playouts) = pass1_corpus();
    assert!(playouts >= 5_000, "too few playout positions: {playouts}");
    assert!(corpus.len() > 4_000, "corpus too small: {}", corpus.len());
    let mut proof_fires = 0u64;
    let mut won_fires = 0u64;
    for &remaining in &[u16::MAX, 1] {
        for seeded in [false, true] {
            // The null cache is disabled so every call starts from the same
            // state: a warm cache would cut `work.replies` in whichever
            // side runs second (it is the only table state pass 2 writes).
            let mut tables = Tables::with_share_and_cache(16, true, 75, 0);
            if seeded {
                assert!(seed_proofs(&mut tables, &corpus) > 1_000);
            }
            for (b, attacker) in corpus.iter() {
                let reference = null_win_reference(b, *attacker, &tables, remaining);
                let null = null_board(b, *attacker);
                let want = pass1_reference(&null, *attacker, &tables, remaining);
                match want {
                    Some(NullWin { length: 1, .. }) => won_fires += 1,
                    Some(_) => proof_fires += 1,
                    None => {}
                }
                macro_rules! check {
                        ($k:literal) => {{
                            let (got, work) = null_win_at::<$k>(b, *attacker, &mut tables, remaining);
                            match (&got, &reference) {
                                (None, None) => {}
                                (Some((null, win)), Some((rnull, rwin, rwork))) => {
                                    assert_eq!(win, rwin, "win mismatch (K={}) on {b:?}", $k);
                                    assert_eq!(&work, rwork, "work mismatch (K={}) on {b:?}", $k);
                                    assert_eq!(
                                        rules::key(null),
                                        rules::key(rnull),
                                        "null key mismatch (K={}) on {b:?}",
                                        $k
                                    );
                                }
                                (got, reference) => panic!(
                                    "presence mismatch (K={}, remaining={remaining}, seeded={seeded}) on {b:?}: got {got:?}, reference {reference:?}",
                                    $k
                                ),
                            }
                            // Pass 1 itself, at the same K.
                            assert_eq!(
                                pass1_at::<$k>(b, *attacker, &tables, remaining),
                                want,
                                "pass 1 mismatch (K={}) on {b:?}",
                                $k
                            );
                        }};
                    }
                check!(1);
                check!(8);
                check!(16);
            }
        }
    }
    assert!(
        won_fires > 0 && proof_fires > 0,
        "arms not exercised: {won_fires} won, {proof_fires} proof"
    );
}

/// Corner cases at the chunk boundaries: a proof on child `i` for every
/// `i`, including `K - 1`, `K`, `K + 1` and the tail chunk, with the proof
/// usable, exactly usable and non-firing, and with an immediate `Won`
/// somewhere in the list (the `Won` wins only after a proof on a lower
/// index has been ruled out).
#[test]
fn pass1_chunk_boundaries_match_the_reference() {
    let boards = [
        // `Won` in the null list (the immediate-explosion fixture).
        "k7/p7/8/8/8/8/8/R3K3 b - - 0 1",
        // Castling and a long list; 34 null moves.
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        // The bench `core` cell.
        "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
        // A clock position.
        "4k3/8/8/8/8/8/8/4K3 w - - 99 1",
    ];
    for fen in boards {
        let b = board(fen);
        let attacker = b.side_to_move().flip();
        let null = null_board(&b, attacker);
        let mut moves = MoveList::new();
        generate_legal(&null, &mut moves);
        let n = moves.len();
        for i in 0..n {
            let mut child = null.clone();
            child.do_move(moves.as_slice()[i], &mut StateInfo::new());
            let counter = child.rule50().min(255) as u8;
            for &(length, limit) in &[
                (1u16, 99u8),
                (2, counter),
                (2, counter.saturating_sub(1)),
                (3, 99),
            ] {
                for &remaining in &[u16::MAX, 1] {
                    // Cache off: the differential is over pass 1, which
                    // never reads it (see `pipelined_pass1_matches_reference`).
                    let mut tables = Tables::with_share_and_cache(16, true, 75, 0);
                    tables.store(
                        rules::key(&child),
                        &Value::Proved {
                            length,
                            limit,
                            mv: None,
                        },
                        u32::from(counter),
                    );
                    let want = pass1_reference(&null, attacker, &tables, remaining);
                    assert_eq!(
                        pass1_at::<1>(&b, attacker, &tables, remaining),
                        want,
                        "K=1 on {fen} child {i}/{n} ({length}, {limit}) rem {remaining}"
                    );
                    assert_eq!(
                        pass1_at::<8>(&b, attacker, &tables, remaining),
                        want,
                        "K=8 on {fen} child {i}/{n} ({length}, {limit}) rem {remaining}"
                    );
                    assert_eq!(
                        pass1_at::<16>(&b, attacker, &tables, remaining),
                        want,
                        "K=16 on {fen} child {i}/{n} ({length}, {limit}) rem {remaining}"
                    );
                }
            }
        }
    }
}

/// `K = 0` would be a zero-length chunk; the shipped constant is positive
/// and the pipeline is exercised at `K = 1` (no batching at all).
#[test]
fn shipped_chunk_is_at_least_one() {
    const { assert!(PASS1_CHUNK >= 1) };
}
