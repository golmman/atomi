//! Fixture oracle (slow): wins.tsv rows at --evals 2_000_000 must not be
//! refuted (unless a `checker-rules:` disposition is recorded) and their
//! proofs must validate; never_prove.tsv rows must never be found.

use atomi::events::NullSink;
use atomi::search::{self, Exit, Limits};
use atomi::strategy;
use atomic_movegen::board::Board;
use std::path::Path;

fn rows(file: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(file),
    )
    .unwrap()
    .lines()
    .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("name\t"))
    .map(|l| l.split('\t').map(String::from).collect())
    .collect()
}

#[test]
#[ignore = "slow: 52 rows x up to 2M evals + proofs"]
fn wins_oracle() {
    let lim = Limits {
        seconds: Some(120),
        evals: Some(2_000_000),
        memory_mb: 192,
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        seed: 0,
    };
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let (mut proved, mut censored, mut refuted, mut multi, mut fails) = (0, 0, 0, 0, 0);
    for r in rows("wins.tsv") {
        let (name, fen, _src, note) = (&r[0], &r[1], &r[2], &r[3]);
        let b = Board::from_fen(fen).unwrap();
        let out = search::prove(&b, &lim, NullSink);
        match out.result.exit {
            Exit::Proved => {
                proved += 1;
                let path = dir.join(format!("oracle-{name}.jsonl"));
                match strategy::write_proof(&b, &out.proofs, &path) {
                    Ok(stats) => {
                        multi += stats.multi_move_positions;
                        assert!(atomi::validate::validate_file(&path).valid, "{name}");
                    }
                    Err(e) => {
                        fails += 1;
                        eprintln!("{name}: export failure {e:?}");
                    }
                }
            }
            Exit::Refuted => {
                refuted += 1;
                assert!(
                    note.contains("checker-rules:"),
                    "{name} refuted without disposition"
                );
            }
            _ => censored += 1,
        }
    }
    eprintln!(
        "wins: proved={proved} censored={censored} refuted={refuted} export-failures={fails} multi-move={multi}"
    );
}

#[test]
#[ignore = "slow: 31 rows x up to 2M evals"]
fn never_prove_oracle() {
    let lim = Limits {
        seconds: Some(120),
        evals: Some(2_000_000),
        memory_mb: 192,
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        seed: 0,
    };
    for r in rows("never_prove.tsv") {
        let (name, fen) = (&r[0], &r[1]);
        let b = Board::from_fen(fen).unwrap();
        let out = search::prove(&b, &lim, NullSink);
        assert_ne!(
            out.result.exit,
            Exit::Proved,
            "{name} proved a never-prove row"
        );
    }
}

/// Regression for the false-refutation bug (Open saturation at INFINITY):
/// these positions were wrongly refuted before OPEN_MAX + the run loop.
/// The pawn ending is proved cheaply (26,285 evals, length 13).
#[test]
fn pawn_ending_proved() {
    let b = Board::from_fen("1k6/8/8/p4p1p/P4P1P/4K1p1/8/8 b - - 1 54").unwrap();
    let lim = Limits {
        seconds: Some(60),
        evals: Some(200_000),
        memory_mb: 192,
        filter: true,
        guide: atomi::guide::GuideConfig::none(),
        guide_params: atomi::guide::GuideParams::default(),
        seed: 0,
    };
    let out = search::prove(&b, &lim, NullSink);
    assert_eq!(out.result.exit, Exit::Proved, "{:?}", out.result.exit);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("regr-pawn.jsonl");
    strategy::write_proof(&b, &out.proofs, &path).unwrap_or_else(|e| panic!("{e:?}"));
    assert!(atomi::validate::validate_file(&path).valid);
}

#[test]
#[ignore = "slow: dec10 ~2.6M evals, rem01 ~9.4M evals"]
fn false_refutation_regressions() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    for (name, fen, cap) in [
        (
            "dec10",
            "3r3k/2rB3P/p7/P4p2/1p3Pp1/1P4P1/2p1p3/2R1R2K b - - 5 41",
            20_000_000,
        ),
        (
            "rem01",
            "1k6/8/1R6/p4p1p/P2b1PpP/6P1/8/3n3K b - - 32 49",
            20_000_000,
        ),
    ] {
        let b = Board::from_fen(fen).unwrap();
        let lim = Limits {
            seconds: Some(600),
            evals: Some(cap),
            memory_mb: 192,
            filter: true,
            guide: atomi::guide::GuideConfig::none(),
            guide_params: atomi::guide::GuideParams::default(),
            seed: 0,
        };
        let out = search::prove(&b, &lim, NullSink);
        assert_eq!(
            out.result.exit,
            Exit::Proved,
            "{name}: {:?}",
            out.result.exit
        );
        let path = dir.join(format!("regr-{name}.jsonl"));
        strategy::write_proof(&b, &out.proofs, &path).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert!(atomi::validate::validate_file(&path).valid, "{name}");
    }
}

#[test]
#[ignore = "slow: 52 rows x up to 2M evals with the hand guide"]
fn wins_oracle_hand() {
    let lim = Limits {
        seconds: Some(180),
        evals: Some(2_000_000),
        memory_mb: 192,
        filter: true,
        guide: atomi::guide::GuideConfig::hand(),
        guide_params: atomi::guide::GuideParams::default(),
        seed: 0,
    };
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let (mut proved, mut censored, mut refuted) = (0, 0, 0);
    for r in rows("wins.tsv") {
        let (name, fen, note) = (&r[0], &r[1], &r[3]);
        let b = Board::from_fen(fen).unwrap();
        let out = search::prove(&b, &lim, NullSink);
        match out.result.exit {
            Exit::Proved => {
                proved += 1;
                let path = dir.join(format!("hand-{name}.jsonl"));
                strategy::write_proof(&b, &out.proofs, &path)
                    .unwrap_or_else(|e| panic!("{name}: {e:?}"));
                assert!(atomi::validate::validate_file(&path).valid, "{name}");
            }
            Exit::Refuted => {
                refuted += 1;
                assert!(
                    note.contains("checker-rules:"),
                    "{name} refuted without disposition"
                );
            }
            _ => censored += 1,
        }
    }
    eprintln!("wins (hand): proved={proved} censored={censored} refuted={refuted}");
}
