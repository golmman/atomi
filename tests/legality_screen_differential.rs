//! The `atomic-movegen` 2.8.0 existence screen against the generator (movegen
//! [`plan3`](../../docs/plans/movegen/plan3.md) step 3, handoff3 MG20).
//!
//! 2.8.0 answers "does the side to move have a legal move?" with a **positive
//! screen** (a provable quiet witness) in front of the exact ladder. It backs
//! `has_legal_move`, `has_legal_move_with_state` — and through them
//! `Board::outcome()`, which [`atomi::rules::status`] reads on every expanded
//! node — and the child-side `has_legal_move_after`.
//!
//! A screen that lies turns a `legal` into a `Stalemate`: the plan's branch
//! table reverts on a single disagreement. So this test never compares the
//! screen with itself; it compares it with **`generate_legal`**, the
//! enumeration it must agree with:
//!
//! - parent: `has_legal_move(board)` and `has_legal_move_with_state(&board,
//!   &state)` equal `!generate_legal(..).is_empty()`, and the two enumeration
//!   entry points agree move for move;
//! - child: for every **pseudo-legal** move, `has_legal_move_after` equals
//!   `!generate_legal(built child).is_empty()` — the illegal candidates
//!   included, where the query describes the unreachable child;
//! - the premise atomi's lazy stalemate rests on: no legal move ⇒ the crate's
//!   `is_terminal()` is true (never a `Stalemate` with a legal move hidden).
//!
//! Corpus: the frozen 94 positions plus seeded play-outs into positions it
//! never reaches (the same shape as `child_surface_differential.rs`).

mod common;

use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::{
    generate_legal, generate_legal_with_state, generate_pseudo_legal, has_legal_move,
    has_legal_move_with_state,
};
use atomic_movegen::types::{Bitboard, MoveList, MoveType, PieceType};

use common::corpus;

/// What the differential saw, for the report line.
#[derive(Default)]
struct Tally {
    nodes: usize,
    /// Parents where the screen's own predicate could fire (a quiet
    /// non-commoner move from an unpinned square, no checker, both commoners
    /// present) — the hot path is exercised, not assumed.
    screen_eligible: usize,
    in_check: usize,
    extinct: usize,
    /// Parents with no legal move (the ladder's `false` arm).
    no_legal: usize,
    /// Parents with a legal move but a decided `outcome()` (clock or material
    /// draw): the case where "terminal" and "no legal move" differ.
    decided_with_moves: usize,
    children: usize,
    child_no_legal: usize,
}

/// The crate's own screen predicate, re-derived from public queries (a
/// superset is enough: it only has to prove the arm was reachable).
fn screen_eligible(board: &Board) -> bool {
    let us = board.side_to_move();
    let commoner = PieceType::Commoner;
    if board.commoners(us).is_empty() || board.commoners(us.flip()).is_empty() {
        return false;
    }
    if !board.checkers().is_empty() {
        return false;
    }
    let pinned = board.pinned(us);
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(board, &mut pseudo);
    pseudo.as_slice().iter().any(|m| {
        !board.is_capture(*m)
            && m.move_type() != MoveType::Castling
            && board.piece_on(m.from_sq()).type_of() != Some(commoner)
            && (pinned & Bitboard::square_bb(m.from_sq())).is_empty()
    })
}

/// One parent: the screen vs the generator, and every child through
/// `has_legal_move_after` vs the built child's generator.
fn check_node(board: &Board, tally: &mut Tally) {
    let fen = board.fen();
    let mut state = StateInfo::new();
    board.populate_state(&mut state);
    let mut with_state = MoveList::new();
    generate_legal_with_state(board, &state, &mut with_state);
    let mut plain = MoveList::new();
    generate_legal(board, &mut plain);
    assert_eq!(
        with_state.as_slice(),
        plain.as_slice(),
        "{fen}: generate_legal vs generate_legal_with_state"
    );
    let exists = !plain.is_empty();
    assert_eq!(
        has_legal_move(board),
        exists,
        "{fen}: has_legal_move vs the generator"
    );
    assert_eq!(
        has_legal_move_with_state(board, &state),
        exists,
        "{fen}: has_legal_move_with_state vs the generator"
    );
    // atomi's lazy stalemate: no legal move is always a decided child.
    if !exists {
        tally.no_legal += 1;
        assert!(board.is_terminal(), "{fen}: no legal move but not terminal");
    } else if board.outcome().is_some() {
        tally.decided_with_moves += 1;
    }
    if board.checkers().is_empty() {
        tally.screen_eligible += usize::from(screen_eligible(board));
    } else {
        tally.in_check += 1;
    }
    if board.commoners(board.side_to_move()).is_empty()
        || board.commoners(board.side_to_move().flip()).is_empty()
    {
        tally.extinct += 1;
    }
    tally.nodes += 1;

    let ctx = board.effect_context_with_state(&state);
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(board, &mut pseudo);
    for &m in pseudo.as_slice() {
        let uci = m.to_uci();
        let mut child = board.clone();
        child.do_move(m, &mut StateInfo::new());
        let mut child_legal = MoveList::new();
        generate_legal(&child, &mut child_legal);
        let child_exists = !child_legal.is_empty();
        assert_eq!(
            has_legal_move(&child),
            child_exists,
            "{fen} {uci}: child has_legal_move vs the generator"
        );
        assert_eq!(
            ctx.has_legal_move_after(board, m),
            Some(child_exists),
            "{fen} {uci}: has_legal_move_after vs the child generator"
        );
        if !child_exists {
            tally.child_no_legal += 1;
            assert!(child.is_terminal(), "{fen} {uci}: no move but not terminal");
        }
        tally.children += 1;
    }
}

#[test]
fn existence_screen_matches_the_generator() {
    let (positions, _children) = corpus();
    let mut tally = Tally::default();
    for (_idx, pos) in &positions {
        let board = Board::from_fen(&pos.fen).expect("fixture FEN");
        check_node(&board, &mut tally);
    }
    // Seeded play-outs: positions the corpus never reaches.
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut playout_nodes = 0usize;
    for (_idx, pos) in &positions {
        for _ in 0..2 {
            let mut board = Board::from_fen(&pos.fen).expect("fixture FEN");
            for _ in 0..16 {
                check_node(&board, &mut tally);
                playout_nodes += 1;
                if board.is_terminal() {
                    break;
                }
                let mut legal = MoveList::new();
                generate_legal(&board, &mut legal);
                if legal.is_empty() {
                    break;
                }
                seed = seed
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let pick = ((seed >> 33) % legal.len() as u64) as usize;
                let m = legal.as_slice()[pick];
                board.do_move(m, &mut StateInfo::new());
            }
        }
    }
    eprintln!(
        "existence screen: positions={} playout_nodes={playout_nodes} nodes={} \
         children={} no_legal={} child_no_legal={} screen_eligible={} in_check={} \
         extinct={} decided_with_moves={}",
        positions.len(),
        tally.nodes,
        tally.children,
        tally.no_legal,
        tally.child_no_legal,
        tally.screen_eligible,
        tally.in_check,
        tally.extinct,
        tally.decided_with_moves
    );
    assert!(positions.len() >= 94, "only {} positions", positions.len());
    assert!(playout_nodes >= 2000, "only {playout_nodes} play-out nodes");
    assert!(tally.children > 20_000, "only {} children", tally.children);
    // Both arms of the screen and both answers exist in the corpus.
    assert!(tally.screen_eligible > 0, "the screen never fired");
    assert!(tally.in_check > 0, "no in-check parent");
    assert!(tally.extinct > 0, "no extinct parent");
    assert!(tally.no_legal > 0, "no parent without a legal move");
    assert!(tally.child_no_legal > 0, "no child without a legal move");
    assert!(
        tally.decided_with_moves > 0,
        "no terminal-with-moves parent: the clock/material arms are not covered"
    );
}
