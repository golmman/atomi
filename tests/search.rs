// Justification for >10 KB: the search/integration test suite is one gate
// file; splitting would scatter the shared FEN constants.

//! Search integration tests (library API + CLI).
//!
//! Fixture FENs (constructed 2026-11-06; all roots have checker-sufficient
//! material):
//! - `MATE_IN_1_EXPLOSION`: Qxg8 captures the rook next to the black
//!   commoner → explosion win in 1.
//! - `MATE_IN_1_CHECKMATE`: Rh8+ — a non-capture move that mates (all escapes
//!   covered by Qb6 and the rook itself).
//! - `MATE_IN_2`: no immediate mate; `Qd7` forces `Qb7#` on the next turn.

use atomi::events::{NullSink, ProofEvent, ProofSink};
use atomi::rules;
use atomi::search::{self, Exit, Limits};
use atomic_movegen::board::Board;
use std::process::Command;

const MATE_IN_1_EXPLOSION: &str = "6rk/8/8/8/8/8/8/6QK w - - 0 1";
const MATE_IN_1_CHECKMATE: &str = "k7/8/1Q6/8/8/8/8/K6R w - - 0 1";
const MATE_IN_2: &str = "k7/8/8/8/8/7Q/8/K6R w - - 0 1";
const ROOK_ALONE: &str = "8/8/8/8/2k5/8/8/4KR2 w - - 0 1";

fn board(fen: &str) -> Board {
    Board::from_fen(fen).unwrap()
}

fn limits() -> Limits {
    Limits {
        seconds: Some(60),
        evals: None,
        memory_mb: 16,
    }
}

struct VecSink(Vec<ProofEvent>);
impl ProofSink for VecSink {
    fn proven(&mut self, event: ProofEvent) {
        self.0.push(event);
    }
}

#[test]
fn mate_in_1_by_explosion() {
    let (r, _sink) = search::prove(&board(MATE_IN_1_EXPLOSION), &limits(), NullSink);
    assert_eq!(r.exit, Exit::Proved);
    assert_eq!(r.length, Some(1));
}

#[test]
fn mate_in_1_by_checkmate() {
    let (r, _sink) = search::prove(&board(MATE_IN_1_CHECKMATE), &limits(), NullSink);
    assert_eq!(r.exit, Exit::Proved);
    assert_eq!(r.length, Some(1));
}

#[test]
fn mate_in_2() {
    let (r, _sink) = search::prove(&board(MATE_IN_2), &limits(), NullSink);
    assert_eq!(r.exit, Exit::Proved);
    assert_eq!(r.length, Some(2));
}

#[test]
fn rook_alone_is_refuted_at_root() {
    // R1: K+R vs K is a checker material draw → refuted before any expansion.
    let (r, _sink) = search::prove(&board(ROOK_ALONE), &limits(), NullSink);
    assert_eq!(r.exit, Exit::Refuted);
    assert_eq!(r.expansions, 0);
}

#[test]
fn never_prove_rows_are_never_found() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/never_prove.tsv"
    ))
    .unwrap();
    let lim = Limits {
        seconds: Some(60),
        evals: Some(20_000),
        memory_mb: 16,
    };
    let mut rows = 0;
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("name\t") {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let (name, fen) = (fields[0], fields[1]);
        let board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(_) => continue, // unparseable rows are not this test's concern
        };
        let (r, _sink) = search::prove(&board, &lim, NullSink);
        assert_ne!(r.exit, Exit::Proved, "{name} must not be proved");
        rows += 1;
    }
    assert!(rows >= 30, "only {rows} rows");
}

#[test]
fn determinism_and_sink_independence() {
    let lim = limits();
    let root = board(MATE_IN_2);
    let (a, _a) = search::prove(&root, &lim, NullSink);
    let (b, _b) = search::prove(&root, &lim, NullSink);
    assert_eq!(a.exit, b.exit);
    assert_eq!(a.length, b.length);
    assert_eq!(a.child_evals, b.child_evals);
    assert_eq!(a.expansions, b.expansions);

    // A collecting sink must not change the trajectory.
    let (c, sink) = search::prove(&root, &lim, VecSink(Vec::new()));
    assert_eq!(a.exit, c.exit);
    assert_eq!(a.child_evals, c.child_evals);
    assert_eq!(a.expansions, c.expansions);

    // The root's attacker proof is emitted with the root length/limit.
    let root_key = rules::key(&root);
    let ev = sink
        .0
        .iter()
        .find_map(|e| match e {
            ProofEvent::AttackerProven {
                key,
                mv: _,
                length,
                limit,
            } if *key == root_key => Some((*length, *limit)),
            _ => None,
        })
        .expect("root AttackerProven event missing");
    assert_eq!(ev.0, c.length.unwrap());
    assert!(ev.1 >= 90, "root limit should be high, got {}", ev.1);
}

#[test]
fn evals_cap_is_honoured() {
    // A non-trivial row that needs more than 5_000 child evals.
    let lim = Limits {
        seconds: Some(120),
        evals: Some(5_000),
        memory_mb: 16,
    };
    let (r, _sink) = search::prove(
        &board("r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1"),
        &lim,
        NullSink,
    );
    assert_eq!(r.exit, Exit::Evals);
    assert!(r.child_evals >= 5_000, "child_evals={}", r.child_evals);
    // Overshoot is smaller than one expansion's child count (< 256 is a
    // generous bound for any single position).
    assert!(
        r.child_evals - 5_000 < 256,
        "overshoot {}",
        r.child_evals - 5_000
    );
}

fn atomi(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_atomi"))
        .args(args)
        .output()
        .expect("spawn atomi")
}

#[test]
fn cli_help_and_version() {
    let out = atomi(&["--help"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("USAGE"));
    let out = atomi(&["--version"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("atomi "));
}

#[test]
fn cli_errors() {
    let out = atomi(&["prove", "not a fen"]);
    assert_eq!(out.status.code(), Some(2));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert!(json.get("error").is_some());

    let out = atomi(&["prove", MATE_IN_1_CHECKMATE, "--bogus-opt", "1"]);
    assert_eq!(out.status.code(), Some(2));

    let out = atomi(&["nonsense"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn cli_prove_output_contract() {
    let out = atomi(&[
        "prove",
        MATE_IN_1_CHECKMATE,
        "--seconds",
        "30",
        "--memory",
        "16",
    ]);
    assert_eq!(out.status.code(), Some(0));
    let line = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(json["found"], true);
    assert_eq!(json["length"], 1);
    assert_eq!(json["exit"], "proved");
    assert_eq!(json["proof"], "not-requested");
    // All 12 keys in the contract order.
    let keys = [
        "\"found\"",
        "\"length\"",
        "\"exit\"",
        "\"child_evals\"",
        "\"expansions\"",
        "\"ms\"",
        "\"seconds\"",
        "\"memory_mb\"",
        "\"evals_limit\"",
        "\"proof\"",
        "\"proof_positions\"",
        "\"proof_length\"",
    ];
    let mut pos = 0usize;
    for k in keys {
        let p = line
            .find(k)
            .unwrap_or_else(|| panic!("{k} missing in {line}"));
        assert!(
            p > pos || (k == "\"found\"" && p == 0) || p >= pos,
            "{k} out of order"
        );
        pos = p;
    }
}

// ---- Phase D: proof round trip + soundness ----

fn prove_with_store(
    fen: &str,
    evals: Option<u64>,
) -> (search::SearchResult, atomi::strategy::ProofStore) {
    let lim = Limits {
        seconds: Some(120),
        evals,
        memory_mb: 192,
    };
    search::prove(&board(fen), &lim, atomi::strategy::ProofStore::new(1024))
}

fn roundtrip(fen: &str, dir: &std::path::Path, name: &str) {
    let (r, store) = prove_with_store(fen, None);
    assert_eq!(r.exit, Exit::Proved, "{name}");
    let path = dir.join(format!("{name}.jsonl"));
    let stats = atomi::strategy::write_proof(&board(fen), &store, &path).unwrap();
    let report = atomi::validate::validate_file(&path);
    assert!(report.valid, "{name}: {}", report.error.unwrap_or_default());
    assert_eq!(report.length, Some(stats.length), "{name}");
    let head: serde_json::Value = serde_json::from_str(
        std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        head["moves"].as_u64().unwrap() as u16,
        stats.length,
        "{name}"
    );
    assert!(stats.length <= r.length.unwrap(), "{name}");
}

#[test]
fn proof_round_trips() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    for (name, fen) in [
        ("m1e", MATE_IN_1_EXPLOSION),
        ("m1c", MATE_IN_1_CHECKMATE),
        ("m2", MATE_IN_2),
        ("m99", "k7/8/1Q6/8/8/8/8/K6R w - - 99 1"),
        ("r97", "k7/8/8/8/8/7Q/8/K6R w - - 97 1"),
        (
            "detour",
            "r4r1k/3q1P2/pp4pp/2pp4/N4Q2/2P3PP/PP2p3/R4R1K w - - 0 25",
        ),
    ] {
        roundtrip(fen, dir, name);
    }
}

#[test]
fn byte_deterministic_output() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let (r, store) = prove_with_store(MATE_IN_2, None);
    assert_eq!(r.exit, Exit::Proved);
    let b = board(MATE_IN_2);
    let p1 = dir.join("det1.jsonl");
    let p2 = dir.join("det2.jsonl");
    atomi::strategy::write_proof(&b, &store, &p1).unwrap();
    atomi::strategy::write_proof(&b, &store, &p2).unwrap();
    assert_eq!(std::fs::read(&p1).unwrap(), std::fs::read(&p2).unwrap());
}

#[test]
fn soundness_rows() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/soundness.tsv"
    ))
    .unwrap();
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("name\t") {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let (name, fen, want, cap) = (f[0], f[1], f[2], f[3].parse::<u64>().unwrap());
        let (r, store) = prove_with_store(fen, Some(cap));
        match want {
            "prove" => {
                assert_eq!(r.exit, Exit::Proved, "{name}");
                let path = dir.join(format!("{name}.jsonl"));
                let stats = atomi::strategy::write_proof(&board(fen), &store, &path)
                    .unwrap_or_else(|e| panic!("{name}: {e:?}"));
                assert!(atomi::validate::validate_file(&path).valid, "{name}");
                let _ = stats;
            }
            "never" => assert_ne!(r.exit, Exit::Proved, "{name}"),
            "refute" => assert_eq!(r.exit, Exit::Refuted, "{name}"),
            other => panic!("bad expectation {other}"),
        }
    }
}

#[test]
fn moves_subcommand() {
    let out = atomi(&["moves", MATE_IN_1_CHECKMATE]);
    assert_eq!(out.status.code(), Some(0));
    let j: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert!(j["moves"].as_array().unwrap().len() > 10);
    let out = atomi(&["moves", "nonsense"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn cycle_line_rejected() {
    // A fabricated proof that completes its walk but shuffles back to the
    // root position at a later counter: the identity edge graph has a
    // position-level cycle (root -> D1 -> A2 -> D2 -> root), rejected.
    let t = "{\"root\":\"k7/3Q4/8/8/8/8/8/K7 w - - 0 1\",\"moves\":3}\n\
             {\"fen\":\"k7/3Q4/8/8/8/8/8/K7 w - - 0 1\",\"move\":\"a1a2\"}\n\
             {\"fen\":\"1k6/3Q4/8/8/8/8/K7/8 w - - 2 1\",\"move\":\"a2a1\"}\n\
             {\"fen\":\"k7/3Q4/8/8/8/8/8/K7 w - - 4 1\",\"move\":\"d7a7\"}\n";
    let r = atomi::validate::validate_text(t);
    assert!(!r.valid);
    assert_eq!(r.error.as_deref(), Some("cycle"));
}
