//! Frozen `atomic-movegen` rules surface: the golden differential of
//! `docs/plans/movegen/plan1.md` gate 2 (D-MG1, D-MG3, D-MG6).
//!
//! Corpus: the FEN columns of `tests/fixtures/veto.tsv` and
//! `tests/fixtures/terminal.tsv`, every cell of `bench/corpus.json`, and the
//! hand-built castling / en-passant / promotion rows of `src/testutil.rs`.
//! Per position the fixture records side to move, castling rights, the stored
//! and the legal en-passant square, both clocks, both keys, `outcome`,
//! `is_terminal`, and the ordered pseudo-legal and legal move lists; per legal
//! move the legal-en-passant child key, `rule50_after`, the blast casualties
//! and the child `outcome`; plus perft nodes to depth 4 on a four-position
//! subset.
//!
//! The fixture is **frozen**: any diff is a rules change and a veto
//! (`plan1.md` §7), not a note. `write_golden` only ever writes to
//! `$ATOMI_WRITE_GOLDEN`, so `make test-full` cannot repair the fixture
//! silently; rewrite it only for an announced rules fix:
//!
//!     ATOMI_WRITE_GOLDEN=tests/fixtures/movegen_2.4.0.tsv \
//!         cargo test --release --test movegen_golden -- --ignored

use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::{generate_legal, generate_pseudo_legal};
use atomic_movegen::types::{Bitboard, EnPassantMode, Move, MoveList, Square};
use std::path::{Path, PathBuf};

const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Hand-built edge positions: the castling, en-passant and promotion rows of
/// `src/testutil.rs::fixture_fens` (the bench and veto rows are read from
/// their fixtures below).
const EDGE: &[&str] = &[
    // castling: both colours, both wings, blast-lost rights
    "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
    "r3k2r/p6p/8/8/8/8/P6P/R3K2R w KQkq - 0 1",
    "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
    "r3k2r/1P4P1/8/8/8/8/1p4p1/R3K2R b KQkq - 0 1",
    // en passant: capturable, uncapturable, pinned pawn, blast-adjacent
    "4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2",
    "4k3/8/8/2pP4/8/8/8/4K3 w - - 0 2",
    "4k3/8/8/K1pP3r/8/8/8/8 w - c6 0 2",
    "4k3/8/1b6/2pP4/8/8/8/4K3 w - c6 0 2",
    "4k3/8/8/1pP5/8/8/8/4K3 w - b6 0 2",
    "k7/8/8/1pP5/8/8/8/4K3 w - b6 0 2",
    // promotions: quiet, capture, checking
    "4k3/1P6/8/8/8/8/8/4K3 w - - 0 1",
    "3rk3/1P6/8/8/8/8/8/4K3 w - - 0 1",
    "4k3/8/8/8/8/8/1p6/4K3 b - - 0 1",
    "r3k3/1P6/8/8/8/8/8/4K3 w q - 0 1",
    "2r1k3/1P6/8/8/8/8/8/4K3 w - - 0 1",
    "7k/1P6/8/8/8/8/8/4K3 w - - 0 1",
];

/// Perft rows: `(label, fen, depth)`. Depth 4 over the start position and
/// three edge rows (castling, a legal en-passant capture, a promotion with a
/// check); deeper perft stays in the crate's own harness.
fn perft_rows() -> Vec<(&'static str, &'static str, u32)> {
    vec![
        ("start", START, 4),
        ("castling", EDGE[0], 4),
        ("en_passant", EDGE[5], 4),
        ("promotion_check", EDGE[11], 4),
    ]
}

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/movegen_2.4.0.tsv")
}

/// The ordered corpus: fixture FEN columns, bench cells, then `EDGE`, first
/// occurrence wins.
fn corpus() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |fen: String| {
        if !out.contains(&fen) {
            out.push(fen);
        }
    };
    for file in ["veto.tsv", "terminal.tsv"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(file);
        let text = std::fs::read_to_string(&path).expect("fixture missing");
        for line in text.lines() {
            if line.is_empty() || line.starts_with('#') || line.starts_with("name\t") {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            push(fields[1].to_string());
        }
    }
    let bench =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("bench/corpus.json"))
            .expect("bench/corpus.json missing");
    let json: serde_json::Value = serde_json::from_str(&bench).expect("bench corpus JSON");
    push(json["core"]["fen"].as_str().expect("core fen").to_string());
    for cell in json["sample20"].as_array().expect("sample20") {
        push(cell["fen"].as_str().expect("cell fen").to_string());
    }
    push(START.to_string());
    for fen in EDGE {
        push(fen.to_string());
    }
    out
}

fn sq_name(sq: Square) -> String {
    let idx = sq as u8;
    format!("{}{}", (b'a' + idx % 8) as char, (b'1' + idx / 8) as char)
}

fn opt_sq(sq: Option<Square>) -> String {
    sq.map_or_else(|| "-".to_string(), sq_name)
}

fn squares(bb: Bitboard) -> String {
    let names: Vec<String> = (0..64u8)
        .filter(|&i| bb.0 >> i & 1 == 1)
        .map(|i| sq_name(Square::from_u8(i)))
        .collect();
    if names.is_empty() {
        "-".to_string()
    } else {
        names.join(",")
    }
}

fn uci_list(moves: &[Move]) -> String {
    if moves.is_empty() {
        return "-".to_string();
    }
    moves
        .iter()
        .map(|m| m.to_uci())
        .collect::<Vec<_>>()
        .join(",")
}

fn lists(board: &Board) -> (String, String) {
    let mut pseudo = MoveList::new();
    generate_pseudo_legal(board, &mut pseudo);
    let mut legal = MoveList::new();
    generate_legal(board, &mut legal);
    (uci_list(pseudo.as_slice()), uci_list(legal.as_slice()))
}

fn record(idx: usize, fen: &str, board: &Board, out: &mut String) {
    let (pseudo, legal) = lists(board);
    out.push_str(&format!(
        "P\t{idx}\t{fen}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{:016x}\t{:016x}\t{:?}\t{}\t{pseudo}\t{legal}\n",
        board.side_to_move(),
        board.castling_rights(),
        opt_sq(board.ep_square()),
        opt_sq(board.legal_ep_square()),
        board.rule50(),
        board.game_ply(),
        board.hash(),
        board.hash_with(EnPassantMode::Legal),
        board.outcome(),
        board.is_terminal(),
    ));
    let mut moves = MoveList::new();
    generate_legal(board, &mut moves);
    for &m in moves.as_slice() {
        let casualties = board.blast_casualties(m);
        let child_key = board.hash_after_with(m, EnPassantMode::Legal);
        let mut child = board.clone();
        child.do_move(m, &mut StateInfo::new());
        // The single-move accessor and the moved board must agree; a stored
        // key comes from the former, the search from the latter.
        assert_eq!(
            child_key,
            child.hash_with(EnPassantMode::Legal),
            "{fen} {}",
            m.to_uci()
        );
        assert_eq!(
            child.rule50(),
            board.rule50_after(m),
            "{fen} {}",
            m.to_uci()
        );
        out.push_str(&format!(
            "M\t{idx}\t{}\t{child_key:016x}\t{}\t{}\t{}\t{}\t{:?}\n",
            m.to_uci(),
            board.rule50_after(m),
            squares(casualties.white),
            squares(casualties.black),
            opt_sq(casualties.en_passant_victim),
            child.outcome(),
        ));
    }
}

fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut moves = MoveList::new();
    generate_legal(board, &mut moves);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut total = 0;
    let mut state = StateInfo::new();
    for &m in moves.as_slice() {
        let mut child = board.clone();
        child.do_move(m, &mut state);
        total += perft(&child, depth - 1);
    }
    total
}

fn surface() -> String {
    let fens = corpus();
    let mut out = String::new();
    out.push_str("# Frozen atomic-movegen rules surface (plan1 gate 2). Generated by\n");
    out.push_str(
        "# `cargo test --release --test movegen_golden -- --ignored`; a diff is a veto.\n",
    );
    out.push_str("# FEN column: tests/fixtures/{veto,terminal}.tsv, bench/corpus.json,\n");
    out.push_str("# src/testutil.rs (provenance of those inputs: PROVENANCE.md).\n");
    out.push_str("# P\tidx\tfen\tside\tcastling\tep_stored\tep_legal\trule50\tgame_ply\thash\thash_legal\toutcome\tis_terminal\tpseudo_legal\tlegal\n");
    out.push_str("# M\tidx\tuci\tchild_key_legal\trule50_after\twhite_casualties\tblack_casualties\tep_victim\tchild_outcome\n");
    out.push_str("# N\tlabel\tdepth\tnodes\n");
    for (idx, fen) in fens.iter().enumerate() {
        let board = Board::from_fen(fen).unwrap_or_else(|_| panic!("bad FEN {fen}"));
        record(idx, fen, &board, &mut out);
    }
    for (label, fen, depth) in perft_rows() {
        let board = Board::from_fen(fen).unwrap_or_else(|_| panic!("bad FEN {fen}"));
        out.push_str(&format!("N\t{label}\t{depth}\t{}\n", perft(&board, depth)));
    }
    out
}

#[test]
fn frozen_surface_matches() {
    let want = std::fs::read_to_string(golden_path()).expect("golden fixture missing");
    let got = surface();
    assert_eq!(
        want, got,
        "the atomic-movegen rules surface changed (plan1 gate 2: a diff is a veto)"
    );
    let positions = want.lines().filter(|l| l.starts_with("P\t")).count();
    assert!(
        positions >= 90,
        "fixture suspiciously small: {positions} positions"
    );
}

/// The writer, never a repair path: `make test-full` runs ignored tests, so
/// this one writes to `$ATOMI_WRITE_GOLDEN` (default `/tmp/movegen_2.4.0.tsv`)
/// and only rewrites the committed fixture when that variable names it.
#[test]
#[ignore = "writer: regenerates the frozen surface into $ATOMI_WRITE_GOLDEN"]
fn write_golden() {
    let path = std::env::var("ATOMI_WRITE_GOLDEN")
        .unwrap_or_else(|_| "/tmp/movegen_2.4.0.tsv".to_string());
    std::fs::write(&path, surface()).unwrap();
}
