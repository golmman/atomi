//! MG3 (`EffectContext`) differential against our own frozen rules surface.
//!
//! `atomic-movegen` 2.6.0's `Board::effect_context` / `EffectContext::{effect,
//! is_legal, outcome_after}` classify a pseudo-legal candidate without writing
//! a board. The prover does not search on that API today
//! ([`report29`](../../docs/plans/prover/report29.md) closed
//! EFFECT-CHILD-V1 as `no effect`), but the pinned crate ships it, so this
//! test pins the dependency's claim against *our* evidence: the frozen
//! `movegen_2.4.0.tsv` rules surface (94 positions, 1,605 legal moves, 58
//! illegal candidates), not the crate's own corpus.
//!
//! Per legal move of every `P` row:
//!
//! - `is_legal` is true on the parent's populated state;
//! - `outcome_after` equals the `M` row's `child_outcome` (the moved child's
//!   `Board::outcome()`);
//! - `explodes_enemy_commoner` ⇒ that child outcome is `Some(Loss)`, and the
//!   move's `blast_casualties` do contain the enemy commoner;
//! - `checks_enemy` ⇒ the moved child's `checkers()` is non-empty;
//! - `is_capture` equals `Board::is_capture`, and a legal move always leaves
//!   the mover its commoner.
//!
//! For a pseudo-legal but illegal candidate: `is_legal` is false and
//! `outcome_after` is `None`. (`leaves_own_commoner` is **not** a legality
//! test: a pinned quiet move keeps its commoner and is still illegal.)

use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::{generate_legal_with_state, generate_pseudo_legal};
use atomic_movegen::types::{Move, MoveList, Outcome};
use std::collections::HashMap;
use std::path::Path;

const FIXTURE: &str = "tests/fixtures/movegen_2.4.0.tsv";

/// One `P` row: the position and its move lists.
struct Position {
    fen: String,
    legal: Vec<String>,
    pseudo: Vec<String>,
}

/// The `M` row fields the differential reads.
struct Child {
    outcome: Option<Outcome>,
    rule50_after: u16,
}

fn parse_outcome(s: &str) -> Option<Outcome> {
    match s {
        "Some(Win)" => Some(Outcome::Win),
        "Some(Loss)" => Some(Outcome::Loss),
        "Some(Draw)" => Some(Outcome::Draw),
        "None" => None,
        other => panic!("unparsed child_outcome {other:?}"),
    }
}

/// The fixture's `P` rows, keyed by row index.
type Positions = Vec<(usize, Position)>;
/// The fixture's `M` rows, keyed by `(row index, uci)`.
type Children = HashMap<(usize, String), Child>;

/// Parse the frozen tsv into `(positions, children)`.
fn corpus() -> (Positions, Children) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let text = std::fs::read_to_string(&path).expect("fixture missing");
    let mut positions = Vec::new();
    let mut children = HashMap::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        match f[0] {
            "P" => {
                assert_eq!(f.len(), 15, "P row width");
                let list = |col: &str| -> Vec<String> {
                    if col == "-" {
                        Vec::new()
                    } else {
                        col.split(',').map(str::to_string).collect()
                    }
                };
                positions.push((
                    f[1].parse().expect("P idx"),
                    Position {
                        fen: f[2].to_string(),
                        pseudo: list(f[13]),
                        legal: list(f[14]),
                    },
                ));
            }
            "M" => {
                assert_eq!(f.len(), 9, "M row width");
                children.insert(
                    (f[1].parse().expect("M idx"), f[2].to_string()),
                    Child {
                        rule50_after: f[4].parse().expect("M rule50_after"),
                        outcome: parse_outcome(f[8]),
                    },
                );
            }
            "N" => {}
            other => panic!("unparsed row kind {other:?}"),
        }
    }
    assert!(!positions.is_empty(), "empty fixture");
    (positions, children)
}

#[test]
fn effect_context_matches_the_frozen_surface() {
    let (positions, children) = corpus();
    let mut moves_total = 0usize;
    let mut illegal_total = 0usize;
    let mut explosions = 0usize;
    let mut checks = 0usize;
    let mut captures = 0usize;
    let mut draws = 0usize;
    for (idx, pos) in &positions {
        let board = Board::from_fen(&pos.fen).expect("fixture FEN");
        let mut state = StateInfo::new();
        board.populate_state(&mut state);
        let ectx = board.effect_context_with_state(&state);
        let mut legal = MoveList::new();
        generate_legal_with_state(&board, &state, &mut legal);
        let mut pseudo = MoveList::new();
        generate_pseudo_legal(&board, &mut pseudo);
        let by_uci = |list: &MoveList, uci: &str| -> Option<Move> {
            list.as_slice().iter().copied().find(|m| m.to_uci() == uci)
        };
        let legal_uci: Vec<String> = legal.as_slice().iter().map(|m| m.to_uci()).collect();
        assert_eq!(legal_uci, pos.legal, "legal list of {}", pos.fen);
        for uci in &pos.legal {
            moves_total += 1;
            let m = by_uci(&legal, uci).expect("generated legal move");
            let row = children.get(&(*idx, uci.clone())).expect("M row");
            let effect = ectx.effect(&board, m);
            assert!(
                effect.is_legal,
                "{uci} of {} not legal per the effect",
                pos.fen
            );
            assert_eq!(
                ectx.outcome_after(&board, m),
                row.outcome,
                "outcome_after({uci}) of {}",
                pos.fen
            );
            assert_eq!(board.rule50_after(m), row.rule50_after, "{uci} clock");
            let mut child = board.clone();
            child.do_move(m, &mut StateInfo::new());
            assert_eq!(
                effect.is_capture,
                board.is_capture(m),
                "is_capture({uci}) of {}",
                pos.fen
            );
            captures += usize::from(effect.is_capture);
            // The mover keeps its commoner on every legal move.
            assert!(effect.leaves_own_commoner, "{uci} of {}", pos.fen);
            if effect.explodes_enemy_commoner {
                explosions += 1;
                assert_eq!(
                    row.outcome,
                    Some(Outcome::Loss),
                    "{uci} of {} explodes the commoner but the child is {:?}",
                    pos.fen,
                    row.outcome
                );
                let their = board.commoners(board.side_to_move().flip());
                assert!(
                    !(board.blast_casualties(m).all() & their).is_empty(),
                    "{uci} of {}: blast misses the commoner",
                    pos.fen
                );
            }
            if effect.checks_enemy {
                checks += 1;
                assert!(
                    !child.checkers().is_empty(),
                    "{uci} of {} checks but the child is not in check",
                    pos.fen
                );
            }
            if row.outcome == Some(Outcome::Draw) {
                draws += 1;
            }
        }
        for uci in &pos.pseudo {
            if pos.legal.contains(uci) {
                continue;
            }
            illegal_total += 1;
            let m = by_uci(&pseudo, uci).expect("generated pseudo-legal move");
            assert!(
                !ectx.is_legal(&board, m),
                "{uci} of {} is illegal per the surface but legal per the effect",
                pos.fen
            );
            assert!(!ectx.effect(&board, m).is_legal, "{uci}: effect disagrees");
            assert_eq!(
                ectx.outcome_after(&board, m),
                None,
                "illegal {uci} of {} had an outcome",
                pos.fen
            );
        }
    }
    eprintln!(
        "effect differential: positions={} legal_moves={moves_total} \
         illegal_candidates={illegal_total} captures={captures} \
         explosions={explosions} checks={checks} child_draws={draws}",
        positions.len()
    );
    assert!(positions.len() >= 94, "only {} positions", positions.len());
    assert!(moves_total >= 1605, "only {moves_total} legal moves");
    assert!(illegal_total > 0, "no illegal candidate in the corpus");
    assert!(captures > 0 && explosions > 0 && checks > 0);
    // The 50-move and material rows of the terminal fixture are in the
    // corpus, so `outcome_after`'s draw path is exercised.
    assert!(draws > 0, "no decided draw child in the corpus");
}
