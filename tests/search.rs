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
use atomi::strategy::{ProofSource, ProofStore};
use atomic_movegen::board::Board;
use std::process::Command;

const MATE_IN_1_EXPLOSION: &str = "6rk/8/8/8/8/8/8/6QK w - - 0 1";
const MATE_IN_1_CHECKMATE: &str = "k7/8/1Q6/8/8/8/8/K6R w - - 0 1";
const MATE_IN_2: &str = "k7/8/8/8/8/7Q/8/K6R w - - 0 1";
const ROOK_ALONE: &str = "8/8/8/8/2k5/8/8/4KR2 w - - 0 1";
const VETO_CONTROL: &str = "rnb1k2r/p2p3p/4ppp1/qp6/3PPQ2/8/PPP2PPP/R3KB1R w KQkq - 2 11";
const PAWN_ENDING: &str = "1k6/8/8/p4p1p/P4P1P/4K1p1/8/8 b - - 1 54";

fn board(fen: &str) -> Board {
    Board::from_fen(fen).unwrap()
}

fn limits() -> Limits {
    Limits {
        seconds: Some(60),
        evals: None,
        memory_mb: 16,
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        proof_share_percent: 25,
        null_cache_entries: 0,
        seed: 0,
        stats: false,
        max_depth: None,
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
    let out = search::prove(&board(MATE_IN_1_EXPLOSION), &limits(), NullSink);
    assert_eq!(out.result.exit, Exit::Proved);
    assert_eq!(out.result.length, Some(1));
}

#[test]
fn mate_in_1_by_checkmate() {
    let out = search::prove(&board(MATE_IN_1_CHECKMATE), &limits(), NullSink);
    assert_eq!(out.result.exit, Exit::Proved);
    assert_eq!(out.result.length, Some(1));
}

#[test]
fn mate_in_2() {
    let out = search::prove(&board(MATE_IN_2), &limits(), NullSink);
    assert_eq!(out.result.exit, Exit::Proved);
    assert_eq!(out.result.length, Some(2));
}

#[test]
fn rook_alone_is_refuted_at_root() {
    // R1: K+R vs K is a checker material draw → refuted before any expansion.
    let out = search::prove(&board(ROOK_ALONE), &limits(), NullSink);
    assert_eq!(out.result.exit, Exit::Refuted);
    assert_eq!(out.result.expansions, 0);
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
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        proof_share_percent: 25,
        null_cache_entries: 0,
        seed: 0,
        stats: false,
        max_depth: None,
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
        let out = search::prove(&board, &lim, NullSink);
        assert_ne!(out.result.exit, Exit::Proved, "{name} must not be proved");
        rows += 1;
    }
    assert!(rows >= 30, "only {rows} rows");
}

#[test]
fn group_plain_child_proves_and_is_deterministic() {
    // MATE_IN_2's key move Qd7 is plain (quiet, non-capturing, non-checking);
    // the GROUP child must still find the same mate. Seeds are deterministic.
    let root = board(MATE_IN_2);
    for seed in 0..3u64 {
        let mut lim = limits();
        lim.evals = Some(2_000_000);
        lim.seed = seed;
        lim.guide = atomi::guide::GuideConfig::hand();
        lim.guide.group = true;
        let a = search::prove(&root, &lim, NullSink);
        let b = search::prove(&root, &lim, NullSink);
        assert_eq!(a.result.exit, Exit::Proved, "seed {seed}");
        assert!(a.result.length.is_some_and(|l| l >= 2));
        // Plan-24 Phase A counters: the group is installed and carries at
        // least one plain member.
        assert!(a.result.group_children > 0, "seed {seed}");
        assert!(
            a.result.plain_children >= a.result.group_children,
            "seed {seed}"
        );
        assert_eq!(
            (
                a.result.exit,
                a.result.length,
                a.result.child_evals,
                a.result.expansions
            ),
            (
                b.result.exit,
                b.result.length,
                b.result.child_evals,
                b.result.expansions
            ),
            "seed {seed}"
        );
        // The proof through a GROUP child still resolves to real positions.
        let path =
            std::env::temp_dir().join(format!("atomi-group-{}-{seed}.jsonl", std::process::id()));
        let stats = atomi::strategy::write_proof(&root, &a.proofs, &path).unwrap();
        assert!(stats.positions >= 2);
        assert!(atomi::validate::validate_file(&path).valid, "seed {seed}");
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn group_never_proves_a_lost_position() {
    // dec40 is a legacy side-to-move loss: every plain move loses too.
    let root = board("2kr4/p1pb1p2/3p3p/3Pp3/8/7P/P2QK3/6R1 b - - 0 24");
    let mut lim = limits();
    lim.evals = Some(200_000);
    lim.guide = atomi::guide::GuideConfig::hand();
    lim.guide.group = true;
    let out = search::prove(&root, &lim, NullSink);
    assert_ne!(out.result.exit, Exit::Proved);
}

#[test]
fn determinism_and_sink_events() {
    let lim = limits();
    let root = board(MATE_IN_2);
    let a = search::prove(&root, &lim, NullSink).result;
    let b = search::prove(&root, &lim, NullSink).result;
    assert_eq!(a.exit, b.exit);
    assert_eq!(a.length, b.length);
    assert_eq!(a.child_evals, b.child_evals);
    assert_eq!(a.expansions, b.expansions);

    // A collecting sink must not change the trajectory.
    let c = search::prove(&root, &lim, VecSink(Vec::new()));
    assert_eq!(a.exit, c.result.exit);
    assert_eq!(a.child_evals, c.result.child_evals);
    assert_eq!(a.expansions, c.result.expansions);

    // The root's attacker proof is emitted with the root length/limit.
    let root_key = rules::key(&root);
    let ev = c
        .sink
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
    assert_eq!(ev.0, c.result.length.unwrap());
    assert!(ev.1 >= 90, "root limit should be high, got {}", ev.1);
}

#[test]
fn filter_on_off_identical() {
    // The filter changes no result: identical counters with and without it.
    for fen in [MATE_IN_2, VETO_CONTROL, PAWN_ENDING] {
        let root = board(fen);
        let mut lim = Limits {
            seconds: Some(120),
            evals: Some(2_000_000),
            memory_mb: 192,
            filter: true,
            guide: atomi::guide::GuideConfig::none(),
            guide_params: atomi::guide::GuideParams::default(),
            proof_share_percent: 25,
            null_cache_entries: 0,
            seed: 0,
            stats: false,
            max_depth: None,
        };
        let on = search::prove(&root, &lim, NullSink).result;
        lim.filter = false;
        let off = search::prove(&root, &lim, NullSink).result;
        assert_eq!(
            (on.exit, on.length, on.child_evals, on.expansions),
            (off.exit, off.length, off.child_evals, off.expansions),
            "{fen}"
        );
    }
}

#[test]
fn evals_cap_is_honoured() {
    // A non-trivial row that needs more than 5_000 child evals.
    let lim = Limits {
        seconds: Some(120),
        evals: Some(5_000),
        memory_mb: 16,
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        proof_share_percent: 25,
        null_cache_entries: 0,
        seed: 0,
        stats: false,
        max_depth: None,
    };
    let out = search::prove(
        &board("r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1"),
        &lim,
        NullSink,
    );
    assert_eq!(out.result.exit, Exit::Evals);
    assert!(
        out.result.child_evals >= 5_000,
        "child_evals={}",
        out.result.child_evals
    );
    // Overshoot is smaller than one expansion's child count (< 256 is a
    // generous bound for any single position).
    assert!(
        out.result.child_evals - 5_000 < 256,
        "overshoot {}",
        out.result.child_evals - 5_000
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

    // --proof-memory is gone (D18): proof memory lives inside --memory.
    let out = atomi(&["prove", MATE_IN_1_CHECKMATE, "--proof-memory", "1"]);
    assert_eq!(out.status.code(), Some(2));
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert!(
        json["error"].as_str().unwrap().contains("unknown option"),
        "{json}"
    );

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
    assert!(json["proof_entries"].is_number());
    assert!(json["proof_full"].is_boolean());
    assert!(json["proof_attacker_entries"].is_number());
    assert!(json["proof_defender_entries"].is_number());
    assert!(json["proof_archive_refused"].is_boolean());
    // All 22 keys in the contract order.
    let keys = [
        "\"found\"",
        "\"length\"",
        "\"exit\"",
        "\"child_evals\"",
        "\"expansions\"",
        "\"seed\"",
        "\"sim_nodes\"",
        "\"settle_nodes\"",
        "\"aux_nodes\"",
        "\"aux_skipped\"",
        "\"ms\"",
        "\"seconds\"",
        "\"memory_mb\"",
        "\"evals_limit\"",
        "\"proof\"",
        "\"proof_positions\"",
        "\"proof_length\"",
        "\"proof_entries\"",
        "\"proof_full\"",
        "\"proof_attacker_entries\"",
        "\"proof_defender_entries\"",
        "\"proof_archive_refused\"",
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

#[test]
fn cli_proof_out_does_not_change_counters() {
    // D14: collecting a proof cannot change the trajectory.
    for fen in [MATE_IN_2, VETO_CONTROL] {
        let plain = atomi(&["prove", fen, "--seconds", "60", "--memory", "192"]);
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
        let path = dir.join("cli-counters.jsonl").to_string_lossy().to_string();
        let with_proof = atomi(&[
            "prove",
            fen,
            "--seconds",
            "60",
            "--memory",
            "192",
            "--proof-out",
            path.as_str(),
        ]);
        let a: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&plain.stdout)).unwrap();
        let b: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&with_proof.stdout)).unwrap();
        for k in ["found", "length", "exit", "child_evals", "expansions"] {
            assert_eq!(a[k], b[k], "{k} differs on {fen}");
        }
        assert_eq!(b["proof"], "written");
    }
}

// ---- proof round trip + soundness ----

fn prove(fen: &str, evals: Option<u64>) -> search::Output<NullSink> {
    let lim = Limits {
        seconds: Some(120),
        evals,
        memory_mb: 192,
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        proof_share_percent: 25,
        null_cache_entries: 0,
        seed: 0,
        stats: false,
        max_depth: None,
    };
    search::prove(&board(fen), &lim, NullSink)
}

fn roundtrip(fen: &str, dir: &std::path::Path, name: &str) {
    let out = prove(fen, None);
    assert_eq!(out.result.exit, Exit::Proved, "{name}");
    let path = dir.join(format!("{name}.jsonl"));
    let stats = atomi::strategy::write_proof(&board(fen), &out.proofs, &path).unwrap();
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
    assert!(stats.length <= out.result.length.unwrap(), "{name}");
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
fn writer_differential_veto_rows() {
    // The proof table and the event-fed store must write byte-identical
    // proofs: same search, two sources.
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/veto.tsv"
    ))
    .unwrap();
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let lim = Limits {
        seconds: Some(600),
        evals: None,
        memory_mb: 192,
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        proof_share_percent: 25,
        null_cache_entries: 0,
        seed: 0,
        stats: false,
        max_depth: None,
    };
    let mut rows = 0;
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("name\t") {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let (name, fen) = (f[0], f[1]);
        let root = board(fen);
        let out = search::prove(&root, &lim, ProofStore::new(1024));
        assert_eq!(out.result.exit, Exit::Proved, "{name}");
        let p_table = dir.join(format!("diff-{name}-table.jsonl"));
        let p_store = dir.join(format!("diff-{name}-store.jsonl"));
        atomi::strategy::write_proof(&root, &out.proofs, &p_table)
            .unwrap_or_else(|e| panic!("{name} table: {e:?}"));
        atomi::strategy::write_proof(&root, &out.sink, &p_store)
            .unwrap_or_else(|e| panic!("{name} store: {e:?}"));
        assert_eq!(
            std::fs::read(&p_table).unwrap(),
            std::fs::read(&p_store).unwrap(),
            "{name}: table vs store proofs differ"
        );
        assert!(
            atomi::validate::validate_file(&p_table).valid,
            "{name}: proof table's proof invalid"
        );
        rows += 1;
    }
    assert_eq!(rows, 8);
}

#[test]
fn byte_deterministic_output() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let out = prove(MATE_IN_2, None);
    assert_eq!(out.result.exit, Exit::Proved);
    let b = board(MATE_IN_2);
    let p1 = dir.join("det1.jsonl");
    let p2 = dir.join("det2.jsonl");
    atomi::strategy::write_proof(&b, &out.proofs, &p1).unwrap();
    atomi::strategy::write_proof(&b, &out.proofs, &p2).unwrap();
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
        let out = prove(fen, Some(cap));
        match want {
            "prove" => {
                assert_eq!(out.result.exit, Exit::Proved, "{name}");
                let path = dir.join(format!("{name}.jsonl"));
                atomi::strategy::write_proof(&board(fen), &out.proofs, &path)
                    .unwrap_or_else(|e| panic!("{name}: {e:?}"));
                assert!(atomi::validate::validate_file(&path).valid, "{name}");
            }
            "never" => assert_ne!(out.result.exit, Exit::Proved, "{name}"),
            "refute" => assert_eq!(out.result.exit, Exit::Refuted, "{name}"),
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
fn classify_subcommand_is_read_only_and_default_off() {
    let out = atomi(&["classify", MATE_IN_1_CHECKMATE]);
    assert_eq!(out.status.code(), Some(0));
    let j: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    let moves = j["moves"].as_array().unwrap();
    assert!(!moves.is_empty());
    for (i, m) in moves.iter().enumerate() {
        assert_eq!(m["start_pn"], 1);
        assert_eq!(m["start_dn"], 1);
        assert_eq!(m["class"], "Quiet");
        assert_eq!(m["order"], i);
        assert!(m["child"].as_str().unwrap().contains(' '));
    }
    // The hand guide changes the fresh numbers but never the legal set.
    let out = atomi(&["classify", MATE_IN_1_CHECKMATE, "--guide", "hand"]);
    assert_eq!(out.status.code(), Some(0));
    let h: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(h["legal"], j["legal"]);
    assert_eq!(h["moves"].as_array().unwrap().len(), moves.len());
    let out = atomi(&["classify", "nonsense"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn proof_source_for_store_is_complete() {
    // The ProofSource impl for ProofStore returns the same entries get()
    // exposes (the impl is a thin forwarder; kept so the differential test
    // exercises both sources).
    let mut store = ProofStore::with_cap_bytes(1 << 20);
    store.proven(ProofEvent::AttackerProven {
        key: 5,
        mv: atomic_movegen::types::Move::make_move(
            atomic_movegen::types::Square::A1,
            atomic_movegen::types::Square::A2,
        ),
        length: 2,
        limit: 40,
    });
    let mut v = Vec::new();
    ProofSource::entries(&store, 5, &mut v);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].length, 2);
    assert!(!ProofSource::is_full(&store));
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

#[test]
fn cli_seed_and_guide_contract() {
    let out = atomi(&[
        "prove",
        MATE_IN_2,
        "--seconds",
        "30",
        "--memory",
        "16",
        "--guide",
        "hand",
        "--seed",
        "3",
    ]);
    assert_eq!(out.status.code(), Some(0));
    let line = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(json["seed"], 3);
    assert!(json["sim_nodes"].is_number());
    assert!(json["settle_nodes"].is_number());
    assert_eq!(json["found"], true);

    // Unknown guide mechanisms and bad guides are errors.
    for args in [
        vec!["prove", MATE_IN_2, "--guide", "bogus"],
        vec!["prove", MATE_IN_2, "--guide-off", "bogus"],
        vec!["prove", MATE_IN_2, "--guide-on", "bogus"],
        vec!["prove", MATE_IN_2, "--seed", "-1"],
    ] {
        let out = atomi(&args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn no_guide_seed_zero_is_m1_identical() {
    // The m1 counters on `core` at 1 M child evals (rebuilt baseline
    // `git archive HEAD` in /tmp, same session): 1000032 / 30786.
    let fen = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1";
    let out = atomi(&[
        "prove",
        fen,
        "--seconds",
        "60",
        "--memory",
        "192",
        "--evals",
        "1000000",
        "--guide",
        "none",
        "--seed",
        "0",
    ]);
    let json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(json["exit"], "evals");
    assert_eq!(json["child_evals"], 1_000_032);
    assert_eq!(json["expansions"], 30_786);
    assert_eq!(json["settle_nodes"], 0);
    assert_eq!(json["sim_nodes"], 0);
}

#[test]
fn stats_are_observation_only() {
    // Plan 12 Phase A: the attribution counters may not change the search.
    let root = board(MATE_IN_2);
    let mut off = limits();
    let mut on = limits();
    on.stats = true;
    let a = search::prove(&root, &off, NullSink).result;
    let b = search::prove(&root, &on, NullSink).result;
    assert_eq!(
        (a.exit, a.length, a.child_evals, a.expansions),
        (b.exit, b.length, b.child_evals, b.expansions)
    );
    // Off: every counter is zero.
    assert_eq!(a.unique_keys, 0);
    assert_eq!(a.re_expansions, 0);
    assert_eq!((a.plain_children, a.group_children), (0, 0));
    assert_eq!((a.null_calls, a.null_replies), (0, 0));
    assert_eq!(a.entry_would_return, 0);
    // On: the counters are internally consistent.
    assert!(b.unique_keys <= b.expansions);
    assert_eq!(b.re_expansions, b.expansions - b.unique_keys);
    assert!(b.entry_proof_hits + b.entry_tt_hits >= b.entry_would_return);
    assert_eq!(b.attacker_expansions + b.defender_expansions, b.expansions);
    assert_eq!(b.depth_histogram.iter().sum::<u64>(), b.expansions);
    assert!(b.selected_class.iter().sum::<u64>() > 0);
    assert!(b.null_replies >= b.null_calls);
    off.stats = true;
    assert_eq!(
        search::prove(&root, &off, NullSink).result.unique_keys,
        b.unique_keys,
        "the distinct-key count must be deterministic"
    );
}

#[test]
fn max_depth_default_off_is_trajectory_neutral() {
    // Plan 13 Phase B: `--max-depth` is opt-in and measurement only. Off
    // (`None`) must equal the old 4096-ply cap exactly.
    let root = board(MATE_IN_2);
    let off = limits();
    let mut explicit = limits();
    explicit.max_depth = Some(4096);
    let a = search::prove(&root, &off, NullSink).result;
    let b = search::prove(&root, &explicit, NullSink).result;
    assert_eq!(
        (a.exit, a.length, a.child_evals, a.expansions),
        (b.exit, b.length, b.child_evals, b.expansions)
    );
    // The CLI default is the same trajectory as the explicit 4096 cap.
    let plain = atomi(&["prove", MATE_IN_2, "--seconds", "30", "--memory", "16"]);
    let capped = atomi(&[
        "prove",
        MATE_IN_2,
        "--seconds",
        "30",
        "--memory",
        "16",
        "--max-depth",
        "4096",
    ]);
    let plain: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&plain.stdout).trim()).unwrap();
    let capped: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&capped.stdout).trim()).unwrap();
    assert_eq!(plain["child_evals"], capped["child_evals"]);
    assert_eq!(plain["expansions"], capped["expansions"]);
}

#[test]
fn max_depth_censors_at_the_cap() {
    // A cap below the proof depth censors with `Exit::Depth`; it never
    // invents a win or a refutation.
    let root = board(MATE_IN_2);
    let mut lim = limits();
    lim.max_depth = Some(1);
    let out = search::prove(&root, &lim, NullSink).result;
    assert_eq!(out.exit, Exit::Depth);
    assert_eq!(out.length, None);
    // A cap above the proof depth still proves.
    lim.max_depth = Some(8);
    let out = search::prove(&root, &lim, NullSink).result;
    assert_eq!(out.exit, Exit::Proved);
    assert_eq!(out.length, Some(2));
}

#[test]
fn guide_conformance_flags_are_default_off() {
    // Plan 13 Phase C: `--guide-fresh-dn` and `--guide-replay-exact` are
    // opt-in. With `--guide none` they are inert and the trajectory is the
    // bare tree.
    let plain = atomi(&["prove", MATE_IN_2, "--seconds", "30", "--memory", "16"]);
    let flagged = atomi(&[
        "prove",
        MATE_IN_2,
        "--seconds",
        "30",
        "--memory",
        "16",
        "--guide",
        "none",
        "--guide-fresh-dn",
        "--guide-replay-exact",
    ]);
    let plain: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&plain.stdout).trim()).unwrap();
    let flagged: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&flagged.stdout).trim()).unwrap();
    assert_eq!(plain["child_evals"], flagged["child_evals"]);
    assert_eq!(plain["expansions"], flagged["expansions"]);
    assert_eq!(plain["exit"], flagged["exit"]);
}

#[test]
fn guide_conformance_flags_prove_a_small_position() {
    // The assembled probe must stay sound on a small win: the replay only
    // stores proofs it verified, and the result is still a proof.
    let out = atomi(&[
        "prove",
        MATE_IN_2,
        "--seconds",
        "30",
        "--memory",
        "16",
        "--guide",
        "hand",
        "--guide-fresh-dn",
        "--guide-replay-exact",
    ]);
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(json["exit"], "proved");
    assert_eq!(json["length"], 2);
}

#[test]
fn guide_exactness_flags_are_default_off() {
    // Plan 18: `--guide-sim-exact` and `--guide-null-exact` are opt-in. With
    // `--guide none` they are inert and the trajectory is the bare tree.
    let plain = atomi(&["prove", MATE_IN_2, "--seconds", "30", "--memory", "16"]);
    let flagged = atomi(&[
        "prove",
        MATE_IN_2,
        "--seconds",
        "30",
        "--memory",
        "16",
        "--guide",
        "none",
        "--guide-sim-exact",
        "--guide-null-exact",
    ]);
    let plain: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&plain.stdout).trim()).unwrap();
    let flagged: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&flagged.stdout).trim()).unwrap();
    assert_eq!(plain["child_evals"], flagged["child_evals"]);
    assert_eq!(plain["expansions"], flagged["expansions"]);
    assert_eq!(plain["exit"], flagged["exit"]);
}

#[test]
fn guide_exactness_flags_prove_a_small_position() {
    // The exactness stack must stay sound on a small win: the null replay
    // only stores proofs it verified, and the result is still a proof.
    let out = atomi(&[
        "prove",
        MATE_IN_2,
        "--seconds",
        "30",
        "--memory",
        "16",
        "--guide",
        "hand",
        "--guide-sim-exact",
        "--guide-null-exact",
    ]);
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(json["exit"], "proved");
    assert_eq!(json["length"], 2);
}

#[test]
fn cli_stats_object_is_emitted() {
    let out = atomi(&[
        "prove",
        MATE_IN_1_CHECKMATE,
        "--seconds",
        "30",
        "--memory",
        "16",
        "--stats",
    ]);
    assert_eq!(out.status.code(), Some(0));
    let line = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert!(json["stats"].is_object(), "{json}");
    assert!(json["stats"]["unique_keys"].is_number());
    assert!(json["stats"]["depth_histogram"].is_array());
    assert!(json["stats"]["selected_class"].is_array());
    // Without the flag the default contract is unchanged.
    let plain = atomi(&[
        "prove",
        MATE_IN_1_CHECKMATE,
        "--seconds",
        "30",
        "--memory",
        "16",
    ]);
    let plain: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&plain.stdout).trim()).unwrap();
    assert!(plain.get("stats").is_none());
}

#[test]
fn same_seed_is_repeatable() {
    let fen = "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1";
    let run = || {
        let out = atomi(&[
            "prove",
            fen,
            "--seconds",
            "60",
            "--memory",
            "192",
            "--evals",
            "200000",
            "--guide",
            "hand",
            "--seed",
            "1",
        ]);
        let j: serde_json::Value =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
        (
            j["exit"].clone(),
            j["child_evals"].clone(),
            j["expansions"].clone(),
        )
    };
    assert_eq!(run(), run());
}

#[test]
fn calibration_defaults_are_the_plan3_tree() {
    let fen = "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1";
    let run = |extra: &[&str]| {
        let mut args = vec![
            "prove",
            fen,
            "--seconds",
            "60",
            "--memory",
            "192",
            "--evals",
            "200000",
            "--guide",
            "hand",
            "--seed",
            "1",
        ];
        args.extend_from_slice(extra);
        let out = atomi(&args);
        let j: serde_json::Value =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
        (
            j["exit"].clone(),
            j["child_evals"].clone(),
            j["expansions"].clone(),
        )
    };
    assert_eq!(
        run(&[]),
        run(&["--guide-scale", "1", "--guide-ef-weight", "16"])
    );
    assert_eq!(
        run(&[]),
        run(&["--guide-max-sim", "0", "--guide-max-settle", "0"])
    );
}

#[test]
fn aux_budget_is_deterministic_and_caps_work() {
    let fen = "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1";
    let run = |extra: &[&str]| {
        let mut args = vec![
            "prove",
            fen,
            "--seconds",
            "60",
            "--memory",
            "192",
            "--evals",
            "200000",
            "--guide",
            "hand",
            "--seed",
            "1",
        ];
        args.extend_from_slice(extra);
        let out = atomi(&args);
        let j: serde_json::Value =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
        (
            j["child_evals"].as_u64().unwrap(),
            j["sim_nodes"].as_u64().unwrap(),
            j["settle_nodes"].as_u64().unwrap(),
            j["aux_nodes"].as_u64().unwrap(),
            j["aux_skipped"].as_u64().unwrap(),
        )
    };
    let full = run(&[]);
    let capped = run(&["--guide-max-sim", "1", "--guide-max-settle", "1"]);
    assert_eq!(
        capped,
        run(&["--guide-max-sim", "1", "--guide-max-settle", "1"])
    );
    // The auxiliary caps change the guide's starts, so the main tree may
    // differ in either direction (and does once the exact replays are on):
    // the caps are a deterministic work bound, not a monotone tree order.
    assert!(capped.0 > 0 && full.0 > 0);
    assert_eq!(capped.3, capped.1 + capped.2);
    assert_eq!(full.3, full.1 + full.2);
    assert_eq!(full.4, 0, "unlimited work skips nothing");

    // Plan7 D25: the global auxiliary budget is deterministic and counted
    // in the same units as `sim_nodes + settle_nodes`; it changes the tree
    // only (never `child_evals` accounting).
    let budgeted = run(&["--guide-aux-budget", "1000"]);
    assert_eq!(budgeted, run(&["--guide-aux-budget", "1000"]));
    assert_eq!(budgeted.3, budgeted.1 + budgeted.2);
    // The last admitted operation may overshoot the cap by its own work:
    // one exact replay is bounded by the 1000-unit replay budget.
    assert!(budgeted.3 <= 1000 + 1000, "{budgeted:?}");
    assert!(
        budgeted.4 > 0,
        "a small budget must skip work: {budgeted:?}"
    );
    assert_ne!(budgeted.0, full.0, "the budget changes the tree");
}

#[test]
fn settled_child_proof_round_trip() {
    // MATE_IN_2 with the hand guide exercises the one-ply settle (a child is
    // settled Proved without being expanded). The writer must find the
    // child's own proof entry, and the event-fed store must agree.
    let root = board(MATE_IN_2);
    // Both closure implementations must round-trip a settled child's proof:
    // the build-and-verify path (plan-19 default) and the board-free closure
    // (plan-20 `closure-bits`, the shipped default).
    for bits in [false, true] {
        let mut guide = atomi::guide::GuideConfig::hand();
        guide.closure_bits = bits;
        let lim = Limits {
            seconds: Some(60),
            evals: None,
            memory_mb: 192,
            filter: true,
            guide,
            guide_params: atomi::guide::GuideParams::default(),
            proof_share_percent: 25,
            null_cache_entries: 0,
            seed: 0,
            stats: false,
            max_depth: None,
        };
        let out = search::prove(&root, &lim, ProofStore::new(64));
        assert_eq!(out.result.exit, Exit::Proved);
        if !bits {
            assert!(out.result.settle_nodes > 0, "{:?}", out.result);
        }
        let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
        let p_table = dir.join(format!("settle-table-{bits}.jsonl"));
        let p_store = dir.join(format!("settle-store-{bits}.jsonl"));
        let stats = atomi::strategy::write_proof(&root, &out.proofs, &p_table)
            .unwrap_or_else(|e| panic!("{e:?}"));
        atomi::strategy::write_proof(&root, &out.sink, &p_store)
            .unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(
            std::fs::read(&p_table).unwrap(),
            std::fs::read(&p_store).unwrap(),
            "settled-child proof (bits={bits}): table vs store"
        );
        assert!(atomi::validate::validate_file(&p_table).valid);
        assert!(stats.positions > 0);
    }
}
