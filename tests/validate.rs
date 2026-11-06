//! Validator tests: independence, the validator's own material rule over
//! terminal.tsv, and the acceptance/rejection cases.

use std::path::Path;

const M1: &str = "k7/8/1Q6/8/8/8/8/K6R w - - 0 1";

fn proof(lines: &[&str]) -> String {
    let mut s = format!("{{\"root\":\"{M1}\",\"moves\":1}}\n");
    for l in lines {
        s.push_str(l);
        s.push('\n');
    }
    s
}

fn valid(lines: &[&str]) -> bool {
    atomi::validate::validate_text(&proof(lines)).valid
}

#[test]
fn independence() {
    // src/validate may only use std/serde_json/atomic_movegen.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/validate");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap();
        let src = std::fs::read_to_string(entry.path()).unwrap();
        for (i, line) in src.lines().enumerate() {
            let l = line.trim();
            if l.starts_with("//") {
                continue;
            }
            let here = format!("{}:{}", entry.file_name().to_string_lossy(), i);
            assert!(!l.contains("crate::"), "{here}: {l}");
            assert!(!l.contains("atomi::"), "{here}: {l}");
            assert!(!l.contains("super::"), "{here}: {l}");
            if l.starts_with("use ") {
                assert!(
                    l.contains("std::") || l.contains("serde_json") || l.contains("atomic_movegen"),
                    "{here}: forbidden import: {l}"
                );
            }
        }
    }
}

#[test]
fn material_rule_matches_terminal_tsv() {
    // The validator's own copy of the material rule runs over terminal.tsv,
    // read by its own test code (no shared helper).
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/terminal.tsv"
    ))
    .unwrap();
    let mut checked = 0;
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("name\t") {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let (name, fen, expected) = (f[0], f[1], f[3]);
        // Material rows only: write a proof whose root line claims a move in
        // a material-draw position — the validator must call it failed.
        if f[5] != "material:R1" || expected != "failed" {
            continue;
        }
        let text = format!("{{\"root\":\"{fen}\",\"moves\":1}}\n");
        let r = atomi::validate::validate_text(&text);
        assert!(!r.valid, "{name}: material draw accepted as open root");
        checked += 1;
    }
    assert!(checked >= 19, "only {checked} material rows checked");
}

#[test]
fn accepts_and_reports() {
    // A real proof line for the mate-in-1.
    let ok = proof(&[r#"{"fen":"k7/8/1Q6/8/8/8/8/K6R w - - 0 1","move":"h1h8"}"#]);
    let r = atomi::validate::validate_text(&ok);
    assert!(r.valid, "{}", r.error.unwrap_or_default());
    assert_eq!(r.positions, Some(1));
    assert_eq!(r.length, Some(1));
    assert_eq!(r.header_moves, Some(1));
    // header moves differing from the computed length is accepted + reported.
    let bad_moves = ok.replacen(r#""moves":1"#, r#""moves":7"#, 1);
    let r = atomi::validate::validate_text(&bad_moves);
    assert!(r.valid);
    assert_eq!(r.header_moves, Some(7));
    assert_eq!(r.length, Some(1));
}

#[test]
fn rejects() {
    let good = r#"{"fen":"k7/8/1Q6/8/8/8/8/K6R w - - 0 1","move":"h1h8"}"#;
    // missing move
    assert!(!valid(&[]));
    // illegal move
    assert!(!valid(&[
        r#"{"fen":"k7/8/1Q6/8/8/8/8/K6R w - - 0 1","move":"a1a2"}"#
    ]));
    // a move into stalemate (Qd7: black bare king stalemated? h1h8 needed; use a quiet non-mate)
    // a move into a checker material draw: Qb6-b7?? no — use a row where the
    // written move leads to a material draw: write K+R v K mate attempt
    // provari header field
    let text = proof(&[good]).replacen(r#""root""#, r#""provari":1,"root""#, 1);
    assert!(!atomi::validate::validate_text(&text).valid);
    // conflicting duplicate
    let dup = proof(&[
        good,
        r#"{"fen":"k7/8/1Q6/8/8/8/8/K6R w - - 0 1","move":"h1g1"}"#,
    ]);
    assert!(!atomi::validate::validate_text(&dup).valid);
    // a move into a checker material draw: play Qb6-c7 then... craft directly:
    // root K+R v K claiming a rook mate — the move leads to material draw.
    let mat = r#"{"fen":"4k3/8/8/8/8/8/8/R3K3 w - - 0 1","move":"a1a8"}"#;
    let t = format!("{{\"root\":\"4k3/8/8/8/8/8/8/R3K3 w - - 0 1\",\"moves\":1}}\n{mat}\n");
    assert!(!atomi::validate::validate_text(&t).valid);
    // a move into stalemate: black bare king stalemated after Qc7
    let stale = r#"{"fen":"k7/8/1Q6/8/8/8/8/K6R w - - 0 1","move":"b6c7"}"#;
    let t = format!("{{\"root\":\"k7/8/1Q6/8/8/8/8/K6R w - - 0 1\",\"moves\":1}}\n{stale}\n");
    assert!(!atomi::validate::validate_text(&t).valid);
}

#[test]
fn fifty_move_line_rejected() {
    // Same mate at counter 100 with the move written at counter 100: the
    // defender position is at the limit... the move itself still mates, so
    // write a line where the defender sits at counter 100 instead: the
    // 2-move proof written at counter 99 then reaching counter 100
    // non-terminal must fail.
    let t = "{\"root\":\"k7/8/8/8/8/7Q/8/K6R w - - 99 1\",\"moves\":2}\n\
             {\"fen\":\"k7/8/8/8/8/7Q/8/K6R w - - 99 1\",\"move\":\"h3d7\"}\n\
             {\"fen\":\"k7/8/8/3Q4/8/8/8/KR6 b - - 100 1\",\"move\":\"a8a7\"}\n\
             {\"fen\":\"k7/8/8/3Q4/8/8/8/KR6 w - - 0 1\",\"move\":\"d7b7\"}\n";
    assert!(!atomi::validate::validate_text(t).valid);
}

#[test]
fn fen_text_differences_match() {
    // A line whose FEN text differs but parses to the same position:
    // different fullmove number and a non-capturable ep square spelled out.
    let l = r#"{"fen":"k7/8/1Q6/8/8/8/8/K6R w - - 0 55","move":"h1h8"}"#;
    assert!(valid(&[l]));
    // MISSING (recorded): a non-capturable ep square written in a line FEN
    // needs a real ep-bearing winning position; the identity rule is the
    // same re-render used for the counter case above.
}

#[test]
fn capture_wins_omitted() {
    // The writer omits positions a capture wins at once: the validator must
    // still accept. Explosion: Qxg8 explodes the defender commoner.
    let t = "{\"root\":\"6rk/8/8/8/8/8/8/6QK w - - 0 1\",\"moves\":1}\n";
    assert!(atomi::validate::validate_text(t).valid);
    // Mate by capture (no commoner explosion): Bxe8 explodes the blocking
    // rook (and itself), unblocking Rh8 into mate.
    let t = "{\"root\":\"k3r2R/5B2/P1N5/8/8/8/8/K7 w - - 0 1\",\"moves\":1}\n";
    assert!(atomi::validate::validate_text(t).valid);
}
