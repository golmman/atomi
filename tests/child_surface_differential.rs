//! Justification for >10 KB: one contract (MG15–MG18) pinned against one
//! reference (the built child); splitting it would duplicate the derivation.
//!
//! # The `atomic-movegen` 2.7.0 *child* surface against our own built child
//! (movegen [`plan2`](../../docs/plans/movegen/plan2.md) step 3, handoff2
//! MG15–MG18).
//!
//! 2.7.0 answers "what does the child look like?" for every **pseudo-legal**
//! move without writing a board:
//!
//! - MG15 `EffectContext::{terminal_after, is_terminal_after,
//!   has_legal_move_after}` / `Board::terminal_after`;
//! - MG16 `EffectContext::child_outcome` → `AfterState`
//!   (`Illegal`/`Undecided`/`Terminal`), `outcome_after` = its `.outcome()`;
//! - MG17 `post_move` → `PostMove` (child placement, casualties, ep, rights,
//!   clock; facts only);
//! - MG18 `classification` → `MoveClassification` (the effect's rules fields
//!   without the legality verdict).
//!
//! atomi calls none of them yet, so the frozen fixture cannot guard them. This
//! differential pins the crate against **our own built child** — `clone` +
//! `do_move`, the path atomi's search uses today — over the frozen corpus
//! (`movegen_2.4.0.tsv`: 94 positions incl. every bench cell) and over seeded
//! random play-outs into positions the corpus never reaches. A disagreement is
//! a veto, not a note: a lying terminality is a lost win
//! ([`plan2`](../../docs/plans/movegen/plan2.md) §6).
//!
//! For every pseudo-legal move, `post_move` is field for field the built
//! child; on legal moves the fixture's frozen casualties/clock/outcome columns
//! agree too. `terminal_after` is the built child's `terminal()` (reason
//! included), `is_terminal_after` its `is_terminal()`, `has_legal_move_after`
//! its `has_legal_move()`, `outcome_after` its `outcome()` for legal moves.
//!
//! The second test is plan2 §9 question 1: `Terminal::MaterialDraw` is the
//! crate's *commoner-only* draw while atomi's `rules::material_draw` is the
//! broader Lichess clause, so a prover call site must map the **reason**. The
//! test pins both directions: every crate `MaterialDraw` satisfies atomi's
//! clause, and the atomi-only draws (legal, crate-undecided children) exist and
//! are `Failed` for atomi.

mod common;

use atomi::rules::{Status, material_draw, status};
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::effects::AfterState;
use atomic_movegen::movegen::{
    generate_legal, generate_legal_with_state, generate_pseudo_legal, has_legal_move,
};
use atomic_movegen::types::{Color, MoveList, Outcome, PieceType, Square, TerminalReason};

use common::{Children, corpus, opt_sq, squares};

const PIECE_TYPES: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::Commoner,
];

/// What the differential saw, for the report line.
#[derive(Default)]
struct Tally {
    moves: usize,
    legal: usize,
    illegal: usize,
    terminal: usize,
    undecided: usize,
    /// Illegal pseudo-legal moves whose child is already decided: the exact
    /// case MG16 exists for (a self-explosion has an extinct child).
    illegal_decided: usize,
    reasons: std::collections::BTreeMap<String, usize>,
}

/// A deterministic 64-bit LCG (no dependency, no `rand`): play-outs must be
/// the same on every run, like everything else in this repo.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) % n as u64) as usize
    }
}

/// Check every pseudo-legal move of `board` against its built child.
fn check_child_surface(board: &Board, tally: &mut Tally) {
    let fen = board.fen();
    let mut state = StateInfo::new();
    board.populate_state(&mut state);
    let ctx = board.effect_context_with_state(&state);
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(board, &mut pseudo);
    for &m in pseudo.as_slice() {
        let uci = m.to_uci();
        tally.moves += 1;
        let effect = ctx.effect(board, m);
        // MG18: one derivation, so classification cannot disagree with effect.
        assert_eq!(
            ctx.classification(board, m),
            effect.classification(),
            "{fen} {uci}: classification vs effect"
        );
        // MG17: the child summary is exactly the child `do_move` builds.
        let post = ctx
            .post_move(board, m)
            .unwrap_or_else(|| panic!("{fen} {uci}: no summary for a pseudo-legal move"));
        let mut child = board.clone();
        child.do_move(m, &mut StateInfo::new());
        assert_eq!(
            post.side_to_move(),
            child.side_to_move(),
            "{fen} {uci}: side"
        );
        assert_eq!(post.occupied(), child.occupied(), "{fen} {uci}: occupied");
        for c in [Color::White, Color::Black] {
            assert_eq!(
                post.pieces_color(c),
                child.pieces_color(c),
                "{fen} {uci}: pieces of {c:?}"
            );
        }
        for pt in PIECE_TYPES {
            assert_eq!(
                post.pieces_pt(pt),
                child.pieces_pt(pt),
                "{fen} {uci}: {pt:?} bitboard"
            );
        }
        for i in 0..64u8 {
            let sq = Square::from_u8(i);
            assert_eq!(
                post.piece_on(sq),
                child.piece_on(sq),
                "{fen} {uci}: piece on {sq:?}"
            );
        }
        assert_eq!(post.ep_square(), child.ep_square(), "{fen} {uci}: ep");
        assert_eq!(
            post.castling_rights(),
            child.castling_rights(),
            "{fen} {uci}: castling rights"
        );
        assert_eq!(post.rule50(), child.rule50(), "{fen} {uci}: rule50");
        assert_eq!(
            post.casualties(),
            board.blast_casualties(m),
            "{fen} {uci}: casualties vs blast_casualties"
        );
        assert_eq!(
            post.is_capture(),
            board.is_capture(m),
            "{fen} {uci}: is_capture"
        );
        assert_eq!(
            ctx.effect_and_post(board, m),
            (effect, Some(post)),
            "{fen} {uci}: effect_and_post is the two separate queries"
        );
        // MG15: child terminality is the built child's own answer.
        let terminal = ctx.terminal_after(board, m);
        assert_eq!(terminal, child.terminal(), "{fen} {uci}: terminal_after");
        assert_eq!(
            terminal.map(|t| t.outcome),
            child.outcome(),
            "{fen} {uci}: terminal outcome vs outcome()"
        );
        assert_eq!(
            ctx.is_terminal_after(board, m),
            Some(child.is_terminal()),
            "{fen} {uci}: is_terminal_after"
        );
        assert_eq!(
            ctx.has_legal_move_after(board, m),
            Some(has_legal_move(&child)),
            "{fen} {uci}: has_legal_move_after"
        );
        // MG16: the two-level answer separates what `outcome_after` folds.
        let after = ctx.child_outcome(board, m);
        if effect.is_legal {
            assert_eq!(
                ctx.outcome_after(board, m),
                child.outcome(),
                "{fen} {uci}: outcome_after of a legal move"
            );
            assert_eq!(after.terminal(), terminal, "{fen} {uci}: child_outcome");
            assert_eq!(
                after.is_undecided(),
                terminal.is_none(),
                "{fen} {uci}: undecided arm"
            );
            tally.legal += 1;
            match terminal {
                Some(t) => {
                    tally.terminal += 1;
                    *tally.reasons.entry(format!("{:?}", t.reason)).or_default() += 1;
                }
                None => tally.undecided += 1,
            }
        } else {
            assert_eq!(after, AfterState::Illegal, "{fen} {uci}: illegal arm");
            tally.illegal_decided += usize::from(terminal.is_some());
            assert_eq!(
                ctx.outcome_after(board, m),
                None,
                "{fen} {uci}: an illegal move has no outcome"
            );
            tally.illegal += 1;
        }
    }
}

/// The frozen fixture's own columns: casualties, clock and outcome.
fn check_frozen_columns(children: &Children, idx: usize, board: &Board, legal: &MoveList) {
    let fen = board.fen();
    let mut state = StateInfo::new();
    board.populate_state(&mut state);
    let ctx = board.effect_context_with_state(&state);
    for &m in legal.as_slice() {
        let uci = m.to_uci();
        let row = children
            .get(&(idx, uci.clone()))
            .unwrap_or_else(|| panic!("{fen} {uci}: no M row"));
        let post = ctx.post_move(board, m).expect("summary of a legal move");
        assert_eq!(
            squares(post.casualties().white),
            row.white_casualties,
            "{fen} {uci}: white casualties"
        );
        assert_eq!(
            squares(post.casualties().black),
            row.black_casualties,
            "{fen} {uci}: black casualties"
        );
        assert_eq!(
            opt_sq(post.casualties().en_passant_victim),
            row.ep_victim,
            "{fen} {uci}: en-passant victim"
        );
        assert_eq!(post.rule50(), row.rule50_after, "{fen} {uci}: rule50_after");
        assert_eq!(
            ctx.terminal_after(board, m).map(|t| t.outcome),
            row.outcome,
            "{fen} {uci}: frozen child_outcome"
        );
    }
}

#[test]
fn child_surface_matches_the_built_child() {
    let (positions, children) = corpus();
    let mut tally = Tally::default();
    for (idx, pos) in &positions {
        let board = Board::from_fen(&pos.fen).expect("fixture FEN");
        let mut state = StateInfo::new();
        board.populate_state(&mut state);
        let mut legal = MoveList::new();
        generate_legal_with_state(&board, &state, &mut legal);
        check_frozen_columns(&children, *idx, &board, &legal);
        check_child_surface(&board, &mut tally);
    }
    // Seeded play-outs: positions the corpus never reaches, reached only by
    // legal moves, checked with the same assertions.
    let mut rng = Lcg(0x9E37_79B9_7F4A_7C15);
    let mut nodes = 0usize;
    for (_idx, pos) in &positions {
        for _ in 0..2 {
            let mut board = Board::from_fen(&pos.fen).expect("fixture FEN");
            for _ in 0..16 {
                check_child_surface(&board, &mut tally);
                nodes += 1;
                if board.terminal().is_some() {
                    break;
                }
                let mut legal = MoveList::new();
                generate_legal(&board, &mut legal);
                if legal.is_empty() {
                    break;
                }
                let pick = rng.below(legal.len());
                let m = legal.as_slice()[pick];
                board.do_move(m, &mut StateInfo::new());
            }
        }
    }
    eprintln!(
        "child surface: positions={} playout_nodes={nodes} children={} \
         legal={} illegal={} illegal_decided={} terminal={} undecided={} reasons={:?}",
        positions.len(),
        tally.moves,
        tally.legal,
        tally.illegal,
        tally.illegal_decided,
        tally.terminal,
        tally.undecided,
        tally.reasons
    );
    assert!(positions.len() >= 94, "only {} positions", positions.len());
    assert!(nodes >= 2000, "only {nodes} play-out nodes");
    assert!(tally.moves > 20_000, "only {} children", tally.moves);
    assert!(tally.illegal > 0 && tally.undecided > 0 && tally.terminal > 0);
    // The `Illegal`/`Terminal` split MG16 adds is exercised, not assumed.
    assert!(
        tally.illegal_decided > 0,
        "no illegal move with a decided child"
    );
    // Every `TerminalReason` branch the corpus can reach must actually appear,
    // or the differential is not guarding the reason field.
    for reason in ["Extinction", "NoMovesInCheck", "Stalemate", "MaterialDraw"] {
        assert!(
            tally.reasons.get(reason).is_some_and(|&n| n > 0),
            "no {reason} child: {:?}",
            tally.reasons
        );
    }
}

#[test]
fn terminal_after_pins_the_atomi_mapping() {
    let (positions, _children) = corpus();
    let mut decided = 0usize;
    let mut material = 0usize;
    let mut atomi_only = 0usize;
    let mut material_fens: Vec<String> = Vec::new();
    for (_idx, pos) in &positions {
        let board = Board::from_fen(&pos.fen).expect("fixture FEN");
        let mut state = StateInfo::new();
        board.populate_state(&mut state);
        let ctx = board.effect_context_with_state(&state);
        let mut legal = MoveList::new();
        generate_legal_with_state(&board, &state, &mut legal);
        for &m in legal.as_slice() {
            let mut child = board.clone();
            child.do_move(m, &mut StateInfo::new());
            let stm = child.side_to_move();
            let label = format!("{} {}", pos.fen, m.to_uci());
            match ctx.terminal_after(&board, m) {
                Some(t) => {
                    decided += 1;
                    // The reason, not the outcome, is what a prover call site
                    // must map: the crate's material draw is commoner-only.
                    if t.reason == TerminalReason::MaterialDraw {
                        material += 1;
                        material_fens.push(label.clone());
                        assert_eq!(t.outcome, Outcome::Draw, "{label}");
                        assert!(
                            material_draw(&child),
                            "{label}: crate MaterialDraw, atomi's clause false"
                        );
                    }
                    let ours = status(&child, stm);
                    let theirs = status(&child, stm.flip());
                    match t.outcome {
                        // The child's side to move lost: Failed for it.
                        Outcome::Loss => {
                            assert!(matches!(ours, Status::Failed { .. }), "{label}: {ours:?}");
                            assert_eq!(theirs, Status::Won, "{label}");
                        }
                        // The child's side to move won: its opponent has none.
                        Outcome::Win => {
                            assert_eq!(ours, Status::Won, "{label}");
                            assert!(
                                matches!(theirs, Status::Failed { .. }),
                                "{label}: {theirs:?}"
                            );
                        }
                        Outcome::Draw => {
                            assert!(matches!(ours, Status::Failed { .. }), "{label}");
                            assert!(matches!(theirs, Status::Failed { .. }), "{label}");
                        }
                        other => panic!("{label}: unhandled Outcome {other:?}"),
                    }
                }
                // Undecided for the crate, decided for atomi: its clause is
                // broader than the crate's commoner-only draw.
                None if material_draw(&child) => {
                    atomi_only += 1;
                    assert_eq!(child.outcome(), None, "{label}");
                    assert!(
                        matches!(status(&child, stm), Status::Failed { .. }),
                        "{label}: atomi draws it but status is open"
                    );
                }
                None => {}
            }
        }
    }
    eprintln!(
        "terminal mapping: decided={decided} crate_material_draws={material} \
         atomi_only_draws={atomi_only} first_material={:?}",
        material_fens.first()
    );
    assert!(decided > 0, "no decided child");
    assert!(material > 0, "no crate MaterialDraw child in the corpus");
    assert!(
        atomi_only > 0,
        "no atomi-only draw child: the broader clause is not exercised"
    );
}
