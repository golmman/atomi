//! `terminal.tsv` round-trip through `rules::status`, including the R1
//! material rows on both sides of every clause (both attackers, both colours
//! to move).

use atomi::rules::{Status, child_key, key, status};
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, EnPassantMode, MoveList};

fn board(fen: &str) -> Board {
    Board::from_fen(fen).unwrap()
}

fn play(board: &Board, uci: &str) -> Board {
    let mut next = board.clone();
    let m = next.move_from_uci(uci).unwrap();
    next.do_move(m, &mut StateInfo::new());
    next
}

#[test]
fn terminal_tsv_matches_status() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/terminal.tsv"
    ))
    .unwrap();
    let mut rows = 0;
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("name\t") {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 6, "bad row: {line}");
        let (name, fen, attacker, expected, refutation, _source) = (
            fields[0], fields[1], fields[2], fields[3], fields[4], fields[5],
        );
        let board = Board::from_fen(fen).unwrap_or_else(|_| panic!("{name}: bad FEN {fen}"));
        let attackers: &[Color] = match attacker {
            "w" => &[Color::White],
            "b" => &[Color::Black],
            "both" => &[Color::White, Color::Black],
            other => panic!("{name}: bad attacker {other}"),
        };
        for &a in attackers {
            let want = match (expected, refutation) {
                ("won", "none") => Status::Won,
                ("failed", "unconditional") => Status::Failed { from: 0 },
                ("failed", "rule50") => Status::Failed { from: 100 },
                ("open", "none") => Status::Open,
                other => panic!("{name}: bad expectation {other:?}"),
            };
            assert_eq!(status(&board, a), want, "{name}: attacker {a:?} fen {fen}");
        }
        rows += 1;
    }
    assert!(rows >= 50, "only {rows} rows");
}

#[test]
fn legal_ep_key_roundtrip() {
    let start = board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let e4 = play(&start, "e2e4");
    let shuffled = play(&play(&play(&play(&e4, "g8f6"), "g1f3"), "f6g8"), "f3g1");
    // After e2e4 no en passant capture is legal, so the shuffle returns
    // to the same legal-ep position.
    assert_eq!(key(&e4), key(&shuffled));
    assert_ne!(
        e4.hash_with(EnPassantMode::Always),
        shuffled.hash_with(EnPassantMode::Always)
    );
}

#[test]
fn capturable_ep_square_changes_key() {
    // White to move with a capturable en passant square on d6.
    let with_ep = board("4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2");
    let without = board("4k3/8/8/2pP4/8/8/8/4K3 w - - 0 2");
    assert_ne!(key(&with_ep), key(&without));
    let mut moves = MoveList::new();
    generate_legal(&with_ep, &mut moves);
    assert!(
        (&moves)
            .into_iter()
            .any(|m| m.move_type() == atomic_movegen::types::MoveType::EnPassant)
    );
}

#[test]
fn key_ignores_counters() {
    let low = board("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
    let high = board("4k3/8/8/8/8/8/8/4K2R w K - 87 40");
    assert_eq!(key(&low), key(&high));
}

#[test]
fn child_key_matches_post_move_key() {
    let fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // double pushes available
        "4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2",                        // en passant
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",                     // castling
    ];
    for fen in fens {
        let b = board(fen);
        let mut moves = MoveList::new();
        generate_legal(&b, &mut moves);
        assert!(!moves.is_empty(), "{fen}");
        for m in &moves {
            let mut next = b.clone();
            next.do_move(m, &mut StateInfo::new());
            assert_eq!(child_key(&b, m), key(&next), "{fen} {}", m.to_uci());
        }
    }
}
