use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::effects::EffectContext;
use atomic_movegen::movegen::generate_legal_with_state;
use atomic_movegen::types::{EnPassantMode, MoveList, PieceType};
use std::hint::black_box;

const FENS: &[&str] = &[
    "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
];

fn corpus() -> Vec<(Board, MoveList)> {
    let mut out = Vec::new();
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for fen in FENS {
        for _w in 0..12 {
            let mut b = Board::from_fen(fen).unwrap();
            for _ply in 0..40 {
                let mut st = StateInfo::new();
                b.populate_state(&mut st);
                let mut ml = MoveList::new();
                generate_legal_with_state(&b, &st, &mut ml);
                if ml.is_empty() { break; }
                out.push((b.clone(), ml.clone()));
                x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                let m = ml.as_slice()[(x as usize) % ml.len()];
                b.do_move(m, &mut StateInfo::new());
            }
        }
    }
    out
}

fn bench(f: impl Fn() -> u64) -> f64 {
    black_box(f());
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        black_box(f());
        best = best.min(t.elapsed().as_secs_f64());
    }
    best
}

/// atomi's `needs_board` without the quiet-class check prediction (captures,
/// specials, commoner moves and commoner-adjacent pawn pushes only).
fn needs_board_like(b: &Board, m: atomic_movegen::types::Move) -> bool {
    b.is_capture(m)
        || m.is_promotion()
        || m.is_en_passant()
        || m.is_castling()
        || b.piece_on(m.from_sq()).type_of() == Some(PieceType::Commoner)
}

fn main() {
    let c = corpus();
    let mut rows: Vec<(&str, f64, f64, f64, f64)> = Vec::new();
    for (label, sel) in [
        ("built-ish (capture/special/commoner)", 0u8),
        ("  of which captures", 1),
        ("rest (quiet pawn/piece moves)", 2),
    ] {
        let count: u64 = c.iter().map(|(b, ml)| ml.as_slice().iter().filter(|&&m| match sel {
            0 => needs_board_like(b, m),
            1 => b.is_capture(m),
            _ => !needs_board_like(b, m),
        }).count() as u64).sum();
        // today: clone + do_move + outcome() (+ Legal hash)
        let t_build = bench(|| {
            let mut acc = 0u64;
            for (b, ml) in &c { for &m in ml.as_slice() {
                if !match sel { 0 => needs_board_like(b, m), 1 => b.is_capture(m), _ => !needs_board_like(b, m) } { continue; }
                let mut child = b.clone(); child.do_move(m, &mut StateInfo::new());
                acc = acc.wrapping_add(u64::from(child.outcome().is_some()));
                acc ^= child.hash_with(EnPassantMode::Legal);
            }} acc
        });
        // MG15: our Mover hash + ctx.terminal_after
        let t_mg15 = bench(|| {
            let mut acc = 0u64;
            for (b, ml) in &c {
                let mut st = StateInfo::new(); b.populate_state(&mut st);
                let ctx = EffectContext::new(b, &st);
                for &m in ml.as_slice() {
                    if !match sel { 0 => needs_board_like(b, m), 1 => b.is_capture(m), _ => !needs_board_like(b, m) } { continue; }
                    let (h, _) = b.hash_and_rule50_after(m);
                    acc ^= h;
                    acc = acc.wrapping_add(u64::from(ctx.terminal_after(b, m).is_some()));
                }
            } acc
        });
        // build only
        let t_only = bench(|| {
            let mut acc = 0u64;
            for (b, ml) in &c { for &m in ml.as_slice() {
                if !match sel { 0 => needs_board_like(b, m), 1 => b.is_capture(m), _ => !needs_board_like(b, m) } { continue; }
                let mut child = b.clone(); child.do_move(m, &mut StateInfo::new());
                acc ^= child.hash();
            }} acc
        });
        let n = count as f64;
        rows.push((label, count as f64, t_build * 1e9 / n, t_mg15 * 1e9 / n, t_only * 1e9 / n));
    }
    for (k, cnt, build, mg15, only) in rows {
        println!("{k:<38} n={cnt:>7.0}  today(build+outcome) {build:>6.1}  MG15(hash+terminal) {mg15:>6.1}  build-only {only:>6.1}  ratio {:.3}", mg15 / build);
    }
}
