//! Per-call cost of the movegen 2.7.0 "board-free child" entry points on the
//! `built-ish` class of the probe30 corpus, in two loop shapes.
//!
//! Execution-session addition (plan30 §3.1): `src/main.rs` is the plan-session
//! probe verbatim and times the three rows of plan30 §1 that decide the stop
//! rule (`today`, `MG15 = hash + terminal_after`, `build only`). This binary
//! times the *other* rows of that table — the MG17 hash / `post_move`
//! (`analyze`) pieces, `child_outcome` and `has_legal_move_after` — and
//! repeats every row in a flattened shape (a pre-collected `(board, state,
//! move)` list), because per-call cost on this host depends on the loop shape
//! far more than on the API. Only the ratio `MG15 / today` is stable across
//! shapes; absolute ns/child are not (see the plan30 README).
//!
//! `StateInfo` is built per position inside the timed pass in the node shape
//! (as atomi holds one context per expanded node); the flattened shape keeps
//! it next to the move.

use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::effects::EffectContext;
use atomic_movegen::movegen::generate_legal_with_state;
use atomic_movegen::types::{EnPassantMode, Move, MoveList, PieceType};
use std::hint::black_box;

const FENS: &[&str] = &[
    "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
];

/// `(board, populated state, legal moves)` from 12 play-out streams of 40
/// plies per bench FEN, xorshift-chosen replies — the probe30 corpus.
fn corpus() -> Vec<(Board, StateInfo, Vec<Move>)> {
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
                if ml.is_empty() {
                    break;
                }
                out.push((b.clone(), st, ml.as_slice().to_vec()));
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                let m = ml.as_slice()[(x as usize) % ml.len()];
                b.do_move(m, &mut StateInfo::new());
            }
        }
    }
    out
}

/// Best of 5 timed passes, result black-boxed.
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

/// atomi's `needs_board` without the quiet-class check prediction.
fn built(b: &Board, m: Move) -> bool {
    b.is_capture(m)
        || m.is_promotion()
        || m.is_en_passant()
        || m.is_castling()
        || b.piece_on(m.from_sq()).type_of() == Some(PieceType::Commoner)
}

fn main() {
    let c = corpus();
    let total: u64 = c.iter().map(|(_, _, ms)| ms.len() as u64).sum();
    let mut pairs: Vec<(Board, StateInfo, Move)> = Vec::new();
    for (b, st, ms) in &c {
        for &m in ms {
            if built(b, m) {
                pairs.push((b.clone(), st.clone(), m));
            }
        }
    }
    let n = pairs.len() as f64;

    macro_rules! node {
        ($body:expr) => {
            bench(|| {
                let mut acc = 0u64;
                for (b, ms) in c.iter().map(|(b, _, ms)| (b, ms)) {
                    let mut st = StateInfo::new();
                    b.populate_state(&mut st);
                    let ctx = EffectContext::new(b, &st);
                    for &m in ms {
                        if !built(b, m) {
                            continue;
                        }
                        acc = acc.wrapping_mul(31).wrapping_add(u64::from($body(&ctx, b, &mut st, m)));
                        acc = acc.wrapping_mul(31).wrapping_add(u64::from(m.from_sq() != m.to_sq()));
                    }
                }
                acc
            })
        };
    }
    macro_rules! flat {
        ($body:expr) => {
            bench(|| {
                let mut acc = 0u64;
                for (b, st, m) in &pairs {
                    let ctx = EffectContext::new(b, st);
                    acc = acc.wrapping_mul(31).wrapping_add(u64::from($body(&ctx, b, st, *m)));
                    acc = acc.wrapping_mul(31).wrapping_add(u64::from(m.from_sq() != m.to_sq()));
                }
                acc
            })
        };
    }

    // The deciders of plan30 §1, repeated here so one table holds every row.
    // What atomi spends on one built child today: build, ask the child for its
    // status, and key it (`Legal` ep mode, as `rules::key` does).
    let today = |b: &Board, m: Move| -> u64 {
        let mut child = b.clone();
        child.do_move(m, &mut StateInfo::new());
        u64::from(child.outcome().is_some()) ^ child.hash_with(EnPassantMode::Legal)
    };
    let today_n = bench(|| {
        let mut acc = 0u64;
        for (b, ms) in c.iter().map(|(b, _, ms)| (b, ms)) {
            for &m in ms {
                if built(b, m) {
                    acc = acc.wrapping_mul(31).wrapping_add(today(b, m));
                }
            }
        }
        acc
    });
    let today_f = bench(|| {
        let mut acc = 0u64;
        for (b, _, m) in &pairs {
            acc = acc.wrapping_mul(31).wrapping_add(today(b, *m));
        }
        acc
    });

    let hash_n = bench(|| {
        let mut acc = 0u64;
        for (b, _, ms) in &c {
            for &m in ms {
                if built(b, m) {
                    acc ^= b.hash_and_rule50_after(m).0;
                }
            }
        }
        acc
    });
    let hash_f = bench(|| {
        let mut acc = 0u64;
        for (b, _, m) in &pairs {
            acc ^= b.hash_and_rule50_after(*m).0;
        }
        acc
    });
    let post_n = node!(|ctx: &EffectContext, b: &Board, _st: &mut StateInfo, m: Move| ctx
        .post_move(b, m)
        .is_some());
    let post_f = flat!(|ctx: &EffectContext, b: &Board, _st: &StateInfo, m: Move| ctx
        .post_move(b, m)
        .is_some());
    let term_n = node!(|ctx: &EffectContext, b: &Board, _st: &mut StateInfo, m: Move| ctx
        .terminal_after(b, m)
        .is_some());
    let term_f = flat!(|ctx: &EffectContext, b: &Board, _st: &StateInfo, m: Move| ctx
        .terminal_after(b, m)
        .is_some());
    let mg15_n = bench(|| {
        let mut acc = 0u64;
        for (b, _, ms) in &c {
            let mut st = StateInfo::new();
            b.populate_state(&mut st);
            let ctx = EffectContext::new(b, &st);
            for &m in ms {
                if !built(b, m) {
                    continue;
                }
                acc ^= b.hash_and_rule50_after(m).0;
                acc = acc.wrapping_add(u64::from(ctx.terminal_after(b, m).is_some()));
            }
        }
        acc
    });
    let mg15_f = bench(|| {
        let mut acc = 0u64;
        for (b, st, m) in &pairs {
            let ctx = EffectContext::new(b, st);
            acc ^= b.hash_and_rule50_after(*m).0;
            acc = acc.wrapping_add(u64::from(ctx.terminal_after(b, *m).is_some()));
        }
        acc
    });
    let child_n = node!(|ctx: &EffectContext, b: &Board, _st: &mut StateInfo, m: Move| ctx
        .child_outcome(b, m)
        .is_terminal());
    let child_f = flat!(|ctx: &EffectContext, b: &Board, _st: &StateInfo, m: Move| ctx
        .child_outcome(b, m)
        .is_terminal());
    let him_n = node!(|ctx: &EffectContext, b: &Board, _st: &mut StateInfo, m: Move| ctx
        .has_legal_move_after(b, m)
        .is_some());
    let him_f = flat!(|ctx: &EffectContext, b: &Board, _st: &StateInfo, m: Move| ctx
        .has_legal_move_after(b, m)
        .is_some());

    println!(
        "corpus {} positions, {total} legal moves, {} built-ish ({:.1} %); ns per built child",
        c.len(),
        pairs.len(),
        n * 100.0 / total as f64
    );
    println!("{:<50} {:>8} {:>8}", "variant", "node", "flat");
    let rows: [(&str, f64, f64); 8] = [
        ("today: clone+do_move+outcome()+Legal hash", today_n, today_f),
        ("hash_and_rule50_after (MG17 hash)", hash_n, hash_f),
        ("ctx.post_move (MG17 analyze)", post_n, post_f),
        ("ctx.terminal_after (MG15 body)", term_n, term_f),
        ("MG15: hash + ctx.terminal_after", mg15_n, mg15_f),
        ("ctx.child_outcome", child_n, child_f),
        ("ctx.has_legal_move_after", him_n, him_f),
        ("ratio MG15 / today", mg15_n / today_n, mg15_f / today_f),
    ];
    for (k, a, b) in rows {
        if k.starts_with("ratio") {
            println!("{k:<50} {a:>8.3} {b:>8.3}");
        } else {
            println!("{k:<50} {:>8.1} {:>8.1}", a * 1e9 / n, b * 1e9 / n);
        }
    }
}
