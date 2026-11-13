//! Shared reader for the frozen `atomic-movegen` rules surface.
//!
//! `tests/fixtures/movegen_2.4.0.tsv` is written by `tests/movegen_golden.rs`
//! (plan1 gate 2) and read by the two dependant differentials:
//! `effects_differential.rs` (the parent's move surface, MG3) and
//! `child_surface_differential.rs` (the child view, MG15–MG18). Only the
//! columns those two read are parsed; the fixture itself stays the evidence
//! and this module holds no rules logic.
#![allow(dead_code)]

use atomic_movegen::types::{Bitboard, Outcome, Square};
use std::collections::HashMap;
use std::path::Path;

/// The frozen fixture, relative to the crate root.
pub const FIXTURE: &str = "tests/fixtures/movegen_2.4.0.tsv";

/// One `P` row: the position and its move lists.
pub struct Position {
    pub fen: String,
    pub legal: Vec<String>,
    pub pseudo: Vec<String>,
}

/// The `M` row fields the differentials read.
pub struct Child {
    pub outcome: Option<Outcome>,
    pub rule50_after: u16,
    pub white_casualties: String,
    pub black_casualties: String,
    pub ep_victim: String,
}

/// The fixture's `P` rows, keyed by row index.
pub type Positions = Vec<(usize, Position)>;
/// The fixture's `M` rows, keyed by `(row index, uci)`.
pub type Children = HashMap<(usize, String), Child>;

pub fn parse_outcome(s: &str) -> Option<Outcome> {
    match s {
        "Some(Win)" => Some(Outcome::Win),
        "Some(Loss)" => Some(Outcome::Loss),
        "Some(Draw)" => Some(Outcome::Draw),
        "None" => None,
        other => panic!("unparsed child_outcome {other:?}"),
    }
}

/// Parse the frozen tsv into `(positions, children)`.
pub fn corpus() -> (Positions, Children) {
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
                        white_casualties: f[5].to_string(),
                        black_casualties: f[6].to_string(),
                        ep_victim: f[7].to_string(),
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

pub fn sq_name(sq: Square) -> String {
    let idx = sq as u8;
    format!("{}{}", (b'a' + idx % 8) as char, (b'1' + idx / 8) as char)
}

/// A square column as the fixture writes it: `-` or ascending names.
pub fn opt_sq(sq: Option<Square>) -> String {
    sq.map_or_else(|| "-".to_string(), sq_name)
}

/// A bitboard column as the fixture writes it: `-` or ascending names.
pub fn squares(bb: Bitboard) -> String {
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
