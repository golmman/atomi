use atomi::guide::threat::signature_for;
use atomi::rules;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Bitboard, Color, MoveList, PieceType};
use std::hint::black_box;
use std::time::Instant;

// The existing eval, imported from the repo under test.
use atomi::guide::eval::hand_eval as current_eval;

fn material(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        _ => 0,
    }
}

fn commoner_pressure(board: &Board, us: Color) -> i32 {
    let them = us.flip();
    let targets = board.commoners(them);
    if targets.is_empty() {
        return 0;
    }
    let zone = Board::blast_zone(targets.lsb());
    let occ = board.occupied();
    let mut count = 0i32;
    let mut sq = zone;
    while !sq.is_empty() {
        let s = sq.pop_lsb();
        count += board.attackers_to_color(s, us, occ).count().min(3) as i32;
    }
    count
}

fn attacked_squares(board: &Board, c: Color) -> Bitboard {
    let occ = board.occupied();
    let mut pieces = board.pieces_color(c);
    let mut attacked = Bitboard::EMPTY;
    while !pieces.is_empty() {
        let sq = pieces.pop_lsb();
        let Some(pt) = board.piece_on(sq).type_of() else {
            continue;
        };
        attacked = attacked | atomic_movegen::attacks::attacks_from(pt, c, sq, occ);
    }
    attacked
}

fn mobility(board: &Board, us: Color, them: Color) -> i32 {
    (attacked_squares(board, us).count() as i32 - attacked_squares(board, them).count() as i32) / 4
}

fn exposure(board: &Board, us: Color) -> i32 {
    let them = us.flip();
    let target = board.commoners(them);
    if target.is_empty() {
        return 0;
    }
    let zone = Board::blast_zone(target.lsb());
    let pieces = board.pieces_color(us) & !board.pieces_color_pt(us, PieceType::Pawn);
    (pieces & zone).count().min(3) as i32
}

fn fuse(board: &Board, us: Color) -> i32 {
    let them = us.flip();
    let occ = board.occupied();
    let enemy = board.commoners(them);
    let ours = board.commoners(us);
    let mut score = 0;
    if !enemy.is_empty() {
        score += board.attackers_to_color(enemy.lsb(), us, occ).count().min(3) as i32;
    }
    if !ours.is_empty() {
        score -= board.attackers_to_color(ours.lsb(), them, occ).count().min(3) as i32;
    }
    score
}

/// TERMS-V1, frozen at Phase 0. `mobility_on` is the trim switch from G-40a.
fn terms_v1(board: &Board, mobility_on: bool) -> i32 {
    let us = board.side_to_move();
    let them = us.flip();
    let mut material_score = 0i32;
    for pt in [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
    ] {
        let ours = board.pieces_color_pt(us, pt).count() as i32;
        let theirs = board.pieces_color_pt(them, pt).count() as i32;
        material_score += (ours - theirs) * material(pt);
    }
    let safety = commoner_pressure(board, us) - commoner_pressure(board, them);
    let targets = (signature_for(board, us).count() as i32 - signature_for(board, them).count() as i32) * 20;
    let exposure = (exposure(board, us) - exposure(board, them)) * 8;
    let fuse = fuse(board, us) * 16;
    let checkers = -(board.checkers().count() as i32) * 16;
    let mob = if mobility_on { mobility(board, us, them) } else { 0 };
    let raw = material_score / 32 + safety * 12 + targets + exposure + fuse + checkers + mob;
    raw.clamp(-256, 256)
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn fixture_fens() -> Vec<&'static str> {
    vec![
        "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
        "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
        "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
        "6rk/8/8/8/8/8/8/6QK w - - 0 1",
        "k7/8/1Q6/8/8/8/8/K6R w - - 0 1",
        "k7/8/8/8/8/7Q/8/K6R w - - 0 1",
        "5R2/3p4/3Bk1p1/6Pp/2p4P/p1N2P2/P1PP4/7K w - - 0 29",
        "k7/8/1Q6/8/8/8/8/K6R w - - 99 1",
        "k7/8/8/8/8/7Q/8/K6R w - - 97 1",
        "r4r1k/3q1P2/pp4pp/2pp4/N4Q2/2P3PP/PP2p3/R4R1K w - - 0 25",
        "rnb1k2r/p2p3p/4ppp1/qp6/3PPQ2/8/PPP2PPP/R3KB1R w KQkq - 2 11",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
        "r3k2r/p6p/8/8/8/8/P6P/R3K2R w KQkq - 0 1",
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
        "r3k2r/1P4P1/8/8/8/8/1p4p1/R3K2R b KQkq - 0 1",
        "4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2",
        "4k3/8/8/2pP4/8/8/8/4K3 w - - 0 2",
        "4k3/8/8/K1pP3r/8/8/8/8 w - c6 0 2",
        "4k3/8/1b6/2pP4/8/8/8/4K3 w - c6 0 2",
        "4k3/8/8/1pP5/8/8/8/4K3 w - b6 0 2",
        "k7/8/8/1pP5/8/8/8/4K3 w - b6 0 2",
        "4k3/1P6/8/8/8/8/8/4K3 w - - 0 1",
        "3rk3/1P6/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/1p6/4K3 b - - 0 1",
        "r3k3/1P6/8/8/8/8/8/4K3 w q - 0 1",
        "2r1k3/1P6/8/8/8/8/8/4K3 w - - 0 1",
        "7k/1P6/8/8/8/8/8/4K3 w - - 0 1",
    ]
}

fn playout(fen: &str, seed: u64, plies: usize, out: &mut Vec<Board>) {
    let mut board = Board::from_fen(fen).unwrap();
    let attacker = board.side_to_move();
    let mut rng = Rng(seed | 1);
    let mut moves = MoveList::new();
    for _ in 0..plies {
        out.push(board.clone());
        if rules::status(&board, attacker) != rules::Status::Open {
            break;
        }
        moves.clear();
        generate_legal(&board, &mut moves);
        if moves.is_empty() {
            break;
        }
        let m = moves.as_slice()[(rng.next() as usize) % moves.len()];
        board.do_move(m, &mut StateInfo::new());
    }
}

fn corpus(path: &str) -> Vec<Board> {
    let text = std::fs::read_to_string(path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut fens = vec![v["core"]["fen"].as_str().unwrap().to_string()];
    for p in v["sample20"].as_array().unwrap() {
        fens.push(p["fen"].as_str().unwrap().to_string());
    }
    fens.iter().map(|f| Board::from_fen(f).unwrap()).collect()
}

fn ns_per_call<F: Fn(&Board) -> i32>(boards: &[Board], f: F) -> f64 {
    let passes = 20usize;
    let mut samples = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        let mut acc = 0i64;
        for _ in 0..passes {
            for b in boards {
                acc += i64::from(black_box(f(black_box(b))));
            }
        }
        black_box(acc);
        let ns = start.elapsed().as_nanos() as f64 / (boards.len() * passes) as f64;
        samples.push(ns);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[1]
}

fn term_parts(board: &Board) -> [i32; 7] {
    let us = board.side_to_move();
    let them = us.flip();
    let mut material_score = 0i32;
    for pt in [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
    ] {
        let ours = board.pieces_color_pt(us, pt).count() as i32;
        let theirs = board.pieces_color_pt(them, pt).count() as i32;
        material_score += (ours - theirs) * material(pt);
    }
    let safety = commoner_pressure(board, us) - commoner_pressure(board, them);
    let targets = (signature_for(board, us).count() as i32
        - signature_for(board, them).count() as i32)
        * 20;
    let exposure = (exposure(board, us) - exposure(board, them)) * 8;
    let fuse = fuse(board, us) * 16;
    let checkers = -(board.checkers().count() as i32) * 16;
    let mobility = mobility(board, us, them);
    [
        material_score / 32,
        safety * 12,
        targets,
        exposure,
        fuse,
        checkers,
        mobility,
    ]
}

fn term_stats(boards: &[Board]) {
    let names = [
        "material/32",
        "pressure*12",
        "targets*20",
        "exposure*8",
        "fuse*16",
        "checkers*-16",
        "mobility/4",
    ];
    let mut mins = [i32::MAX; 7];
    let mut maxs = [i32::MIN; 7];
    let mut means = [0i64; 7];
    let mut zeros = [0usize; 7];
    for b in boards {
        let parts = term_parts(b);
        for i in 0..7 {
            mins[i] = mins[i].min(parts[i]);
            maxs[i] = maxs[i].max(parts[i]);
            means[i] += i64::from(parts[i]);
            if parts[i] == 0 {
                zeros[i] += 1;
            }
        }
    }
    for i in 0..7 {
        println!(
            "term {:<15} min {:>5} max {:>5} mean {:>7.2} zero {}/{}",
            names[i],
            mins[i],
            maxs[i],
            means[i] as f64 / boards.len() as f64,
            zeros[i],
            boards.len()
        );
    }
}

fn dist<F: Fn(&Board) -> i32>(boards: &[Board], f: F) -> (i32, i32, i32, f64) {
    let mut mn = i32::MAX;
    let mut mx = i32::MIN;
    let mut sat = 0usize;
    let mut sum = 0i64;
    for b in boards {
        let v = f(b);
        mn = mn.min(v);
        mx = mx.max(v);
        sum += i64::from(v);
        if v == 256 || v == -256 {
            sat += 1;
        }
    }
    (mn, mx, sat as i32, sum as f64 / boards.len() as f64)
}

fn main() {
    let mut boards = corpus("/workspace/atomi/bench/corpus.json");
    println!("corpus positions: {}", boards.len());
    for (i, fen) in fixture_fens().iter().enumerate() {
        for s in 0..5u64 {
            playout(fen, 0x9E37_79B9_7F4A_7C15 ^ (i as u64) << 32 ^ s, 120, &mut boards);
        }
    }
    println!("total positions: {}", boards.len());
    let _ = black_box(current_eval(black_box(&boards[0])));
    let _ = black_box(terms_v1(black_box(&boards[0]), true));

    let cur_ns = ns_per_call(&boards, current_eval);
    let v1_ns = ns_per_call(&boards, |b| terms_v1(b, true));
    let trim_ns = ns_per_call(&boards, |b| terms_v1(b, false));
    println!("current_ns_per_call {cur_ns:.2}");
    println!("terms_v1_ns_per_call {v1_ns:.2}");
    println!("terms_v1_trimmed_ns_per_call {trim_ns:.2}");
    println!("current_dist {:?}", dist(&boards, current_eval));
    println!("v1_dist {:?}", dist(&boards, |b| terms_v1(b, true)));
    println!("trim_dist {:?}", dist(&boards, |b| terms_v1(b, false)));
    term_stats(&boards);
    let mut diff = 0usize;
    for b in &boards {
        if current_eval(b) != terms_v1(b, true) {
            diff += 1;
        }
    }
    println!("current_ne_v1 {diff}/{}", boards.len());
}
