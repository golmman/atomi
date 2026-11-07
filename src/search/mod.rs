// Justification for >10 KB: the driver, Limits/Output contract and MID
// loop are one unit; the soundness argument belongs with the code.

//! Attacker-only depth-first proof-number search (df-pn) with the 1+ε
//! threshold rule and uniform starting numbers (proof = disproof = 1 for
//! every open child). No move ordering beyond movegen order, no heuristics.
//!
//! The attacker is the root's side to move. The counter is the board's
//! 50-move clock. The line is the path of position keys from the root
//! (index = ply); repetition and the clock are never wins.
//!
//! `prove` runs on a spawned thread with a 256 MiB stack so recursion depth
//! is never the limit below the 4096-ply cap. `ms` measures search time only
//! (the tables are allocated before the clock starts). Deterministic: no
//! randomness and no hash-map iteration anywhere in the search path.
//!
//! Soundness (M1): a proof needs only proved children, exact terminal wins
//! (an explosion via a made child's `status`, or a mate via a made child —
//! every checking move is built at the parent) and usable proof-table
//! entries (full-key match, `counter <= limit`) — none of these changed
//! meaning. What D19's lazy stalemate and partial TT keys can cause is a
//! missed or wrong refutation or a different visiting order, which affects
//! completeness and speed only. The node-entry status makes every expanded
//! node's terminality exact, so no frame is ever built at a position that
//! is over. The validator and `provari check` remain the veto on every
//! claimed win.

mod frame;
pub mod value;

use crate::events::{ProofEvent, ProofSink};
use crate::rules;
use crate::table::Tables;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::types::Color;
use frame::{Entered, Frame};
use std::time::{Duration, Instant};
use value::{Child, INFINITY, Value, and_value, exit_value, or_value, pn_dn};

/// The search budgets.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Wall-clock deadline in seconds (`None` = none).
    pub seconds: Option<u64>,
    /// Deterministic cap on `child_evals` (`None` = none).
    pub evals: Option<u64>,
    /// The `--memory` label in MiB (TT + proof table + filter).
    pub memory_mb: usize,
    /// The "possibly stored" bit filter. A test hook for the
    /// filter-neutrality test; the CLI always passes `true`.
    pub filter: bool,
}

/// How the search ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// The root was proved.
    Proved,
    /// The root was refuted (no win under the format's rules, as far as this
    /// search can tell — not a draw or loss claim).
    Refuted,
    /// The wall-clock deadline fired.
    Timeout,
    /// The `child_evals` cap fired.
    Evals,
    /// The 4096-ply cap fired (censors the run).
    Depth,
}

impl Exit {
    /// The CLI string.
    pub fn as_str(self) -> &'static str {
        match self {
            Exit::Proved => "proved",
            Exit::Refuted => "refuted",
            Exit::Timeout => "timeout",
            Exit::Evals => "evals",
            Exit::Depth => "depth",
        }
    }
}

/// The outcome of one `prove` call.
#[derive(Debug)]
pub struct SearchResult {
    /// How the search ended.
    pub exit: Exit,
    /// The root proof length in attacker moves (proved only).
    pub length: Option<u16>,
    /// Children classified during expansions.
    pub child_evals: u64,
    /// Nodes expanded (a child list generated).
    pub expansions: u64,
    /// Search time in milliseconds (allocation excluded).
    pub ms: u64,
    /// Live entries in the proof table at exit.
    pub proof_entries: u64,
    /// The proof table refused an insert at its 90% load bound.
    pub proof_full: bool,
}

/// What `prove` hands back: the result, the proof table and the sink.
pub struct Output<S> {
    /// The search result.
    pub result: SearchResult,
    /// The search's proof memory (the writer's source, D18).
    pub proofs: crate::table::ProofTable,
    /// The event sink, returned so a store can be drained afterwards.
    pub sink: S,
}

const MAX_PLY: u16 = 4096;
const DEADLINE_CHECK_INTERVAL: u64 = 1024;

/// Solve `root` for the side to move (the attacker), returning the result,
/// the proof table and the sink back so a store can be drained afterwards.
pub fn prove<S: ProofSink + Send + 'static>(root: &Board, limits: &Limits, sink: S) -> Output<S> {
    let root = root.clone();
    let limits = limits.clone();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let tables = Tables::new(limits.memory_mb, limits.filter);
            let start = Instant::now();
            let deadline = limits.seconds.map(|s| start + Duration::from_secs(s));
            let mut engine = Engine {
                attacker: root.side_to_move(),
                tables,
                line: Vec::new(),
                deadline,
                evals_cap: limits.evals,
                child_evals: 0,
                expansions: 0,
                stop: None,
                sink,
            };
            let mut result = engine.run(&root);
            result.ms = start.elapsed().as_millis() as u64;
            result.proof_entries = engine.tables.proofs().live() as u64;
            result.proof_full = engine.tables.proofs().refused();
            Output {
                result,
                proofs: engine.tables.into_proofs(),
                sink: engine.sink,
            }
        })
        .expect("spawn search thread")
        .join()
        .expect("join search thread")
}

struct Engine<S> {
    attacker: Color,
    tables: Tables,
    line: Vec<u64>,
    deadline: Option<Instant>,
    evals_cap: Option<u64>,
    child_evals: u64,
    expansions: u64,
    stop: Option<Exit>,
    sink: S,
}

impl<S: ProofSink> Engine<S> {
    fn run(&mut self, root: &Board) -> SearchResult {
        if rules::status(root, self.attacker) != rules::Status::Open {
            // An already-over root (mate, stalemate, material draw, the
            // clock) is refuted with no work done.
            return SearchResult {
                exit: Exit::Refuted,
                length: None,
                child_evals: 0,
                expansions: 0,
                ms: 0,
                proof_entries: 0,
                proof_full: false,
            };
        }
        let root_key = rules::key(root);
        let mut value = self.mid(root, root_key, 0, INFINITY, INFINITY);
        // A saturated Open value is still undecided: keep searching. Only a
        // Refuted root is a refutation; Open can never become one (the bug
        // this loop fixes was Open saturation reaching INFINITY).
        while self.stop.is_none() && matches!(value, Value::Open { .. }) {
            value = self.mid(root, root_key, 0, INFINITY, INFINITY);
        }
        let (exit, length) = match (self.stop, value) {
            (Some(stop), _) => (stop, None),
            (None, Value::Proved { length, .. }) => (Exit::Proved, Some(length)),
            (None, Value::Refuted { .. }) => (Exit::Refuted, None),
            (None, Value::Open { .. }) => unreachable!("Open root without a stop"),
        };
        SearchResult {
            exit,
            length,
            child_evals: self.child_evals,
            expansions: self.expansions,
            ms: 0,
            proof_entries: 0,
            proof_full: false,
        }
    }

    /// One MID pass over a node (`key` is its position key, computed by the
    /// caller — the frame's pass 1 supplies it for every descent): loop
    /// selecting the best child until the node value crosses a threshold,
    /// then store and return it.
    fn mid(&mut self, board: &Board, key: u64, ply: u16, tpn: u32, tdn: u32) -> Value {
        if self.stop.is_some() {
            return Value::Open { pn: 0, dn: 0 };
        }
        if ply >= MAX_PLY {
            self.stop = Some(Exit::Depth);
            return Value::Open { pn: 0, dn: 0 };
        }
        self.expansions += 1;
        if self.expansions.is_multiple_of(DEADLINE_CHECK_INTERVAL)
            && let Some(dl) = self.deadline
            && Instant::now() >= dl
        {
            self.stop = Some(Exit::Timeout);
            return Value::Open { pn: 0, dn: 0 };
        }
        debug_assert_eq!(key, rules::key(board));
        self.line.push(key);
        let mut frame = match Frame::enter(
            board,
            key,
            self.attacker,
            &self.line,
            &self.tables,
            self.expansions,
        ) {
            Entered::Terminal(v) => {
                // The node is over: no children were initialized (the
                // expansion above ran the movegen that decided it). A
                // terminal proof is a no-op in `Tables::store`.
                self.tables.store(key, &v, 0);
                self.line.pop();
                return v;
            }
            Entered::Open(frame, spent) => {
                self.child_evals += spent;
                frame
            }
        };
        if let Some(cap) = self.evals_cap
            && self.child_evals >= cap
        {
            self.stop = Some(Exit::Evals);
            self.line.pop();
            return Value::Open { pn: 0, dn: 0 };
        }
        // MID loop.
        loop {
            let v = node_value(&frame);
            let (pn, dn) = pn_dn(&v);
            if pn >= tpn || dn >= tdn {
                break;
            }
            let (best, second) = select(&frame);
            let (ctpn, ctdn) = child_thresholds(&frame, best, second, tpn, tdn, pn, dn);
            let mut next = board.clone();
            next.do_move(frame.children[best].mv, &mut StateInfo::new());
            let cv = self.mid(&next, frame.keys[best], ply + 1, ctpn, ctdn);
            if self.stop.is_some() {
                self.line.pop();
                return Value::Open { pn: 0, dn: 0 };
            }
            frame.children[best].value = cv;
        }
        // Frame exit.
        let v = exit_value(node_value(&frame), ply);
        let work = (self.expansions - frame.work_start).min(u64::from(u32::MAX)) as u32;
        let key = *self.line.last().unwrap();
        self.tables.store(key, &v, work);
        if frame.attacker_node
            && let Value::Proved {
                length,
                limit,
                mv: Some(mv),
            } = v
        {
            self.sink.proven(ProofEvent::AttackerProven {
                key,
                mv,
                length,
                limit,
            });
        }
        debug_assert!(value_invariant(&v, board.rule50()));
        self.line.pop();
        v
    }
}

fn node_value(frame: &Frame) -> Value {
    if frame.attacker_node {
        or_value(&frame.children)
    } else {
        and_value(&frame.children)
    }
}

fn value_invariant(v: &Value, counter: u16) -> bool {
    match *v {
        Value::Proved { limit, .. } => u16::from(limit) >= counter,
        Value::Refuted { from, .. } => u16::from(from) <= counter,
        Value::Open { .. } => true,
    }
}

/// The best child's index (OR: minimum pn; AND: minimum dn; ties to the
/// first) and the second-best value of the selecting metric
/// ([`INFINITY`] when there is no second).
fn select(frame: &Frame) -> (usize, u32) {
    let metric = |c: &Child| {
        let (pn, dn) = pn_dn(&c.value);
        if frame.attacker_node { pn } else { dn }
    };
    let mut best = usize::MAX;
    let mut best_m = INFINITY;
    let mut second = INFINITY;
    for (i, c) in frame.children.iter().enumerate() {
        let m = metric(c);
        if m < best_m {
            second = best_m;
            best_m = m;
            best = i;
        } else if m < second {
            second = m;
        }
    }
    debug_assert!(best != usize::MAX);
    (best, second)
}

fn widen(x: u32) -> u64 {
    u64::from(x) + 1 + u64::from(x) / 8
}

fn child_thresholds(
    frame: &Frame,
    best: usize,
    second: u32,
    tpn: u32,
    tdn: u32,
    pn: u32,
    dn: u32,
) -> (u32, u32) {
    let c = &frame.children[best];
    let (cpn, cdn) = pn_dn(&c.value);
    let (ctpn, ctdn) = if frame.attacker_node {
        (
            u64::from(tpn).min(widen(second)),
            u64::from(tdn) - u64::from(dn) + u64::from(cdn),
        )
    } else {
        (
            u64::from(tpn) - u64::from(pn) + u64::from(cpn),
            u64::from(tdn).min(widen(second)),
        )
    };
    (
        ctpn.min(u64::from(INFINITY)) as u32,
        ctdn.min(u64::from(INFINITY)) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::NullSink;
    use crate::search::value::ANY;
    use atomic_movegen::movegen::generate_legal;
    use atomic_movegen::types::MoveList;

    /// D19 end to end. Parent `1K5n/5p2/5Pp1/6P1/8/3N4/N7/kN6 w - - 0 1`
    /// (white K b8, knights a2/b1/d3; black commoner a1, black knight h8
    /// walled in by its own blocked pawns f7/g6): the quiet knight move
    /// `d3b2` completes the box a2/b1/b2 around a1 — a quiet, non-checking
    /// move after which black has no legal move and is not in check (every
    /// "escape" capture self-destructs the black commoner, and h8's only
    /// destinations f7/g6 are its own pawns). Black holds a knight, so the
    /// stalemate gate (G1: lazy only when `them` has more than
    /// commoner + pawns) leaves this move lazy. The child
    /// `1K5n/5p2/5Pp1/6P1/8/8/NN6/kN6 b - - 0 1` is a stalemate: `atomi
    /// moves` prints none and `rules::status` says `Failed`. The frame
    /// starts that child `Open(1, 1)` (lazy); a `mid` descent refutes it at
    /// node entry.
    #[test]
    fn lazy_stalemate_end_to_end() {
        let parent = Board::from_fen("1K5n/5p2/5Pp1/6P1/8/3N4/N7/kN6 w - - 0 1").unwrap();
        let child = Board::from_fen("1K5n/5p2/5Pp1/6P1/8/8/NN6/kN6 b - - 0 1").unwrap();
        // The child is a verified stalemate.
        let mut ml = MoveList::new();
        generate_legal(&child, &mut ml);
        assert!(ml.is_empty());
        assert_eq!(
            rules::status(&child, Color::White),
            rules::Status::Failed { from: 0 }
        );
        // The frame gives the d1b2 child Open(1, 1) — the lazy case.
        let mut engine = Engine {
            attacker: Color::White,
            tables: Tables::new(16, true),
            line: Vec::new(),
            deadline: None,
            evals_cap: None,
            child_evals: 0,
            expansions: 0,
            stop: None,
            sink: NullSink,
        };
        let key = rules::key(&parent);
        engine.line.push(key);
        let Entered::Open(frame, _) = Frame::enter(
            &parent,
            key,
            engine.attacker,
            &engine.line,
            &engine.tables,
            0,
        ) else {
            panic!("parent must be open");
        };
        let b2 = parent.move_from_uci("d3b2").unwrap();
        let i = frame
            .children
            .iter()
            .position(|c| c.mv == b2)
            .expect("d1b2 must be a child");
        assert_eq!(frame.children[i].value, Value::Open { pn: 1, dn: 1 });
        engine.line.pop();
        // A mid descent into the stalemate child refutes it at node entry.
        let ck = rules::key(&child);
        let v = engine.mid(&child, ck, 1, INFINITY, INFINITY);
        assert_eq!(
            v,
            Value::Refuted {
                cycle: ANY,
                from: 0
            }
        );
        // The root outcome equals M0's: `prove` on the parent gives
        // (exit=proved, length=14) — measured once with
        // `/tmp/atomi-m0/target/release/atomi prove <FEN> --memory 192
        // --seconds 600 --evals 2000000` (2026-11-06); m0 spent 646 child
        // evals / 134 expansions there.
        let mut engine = Engine {
            attacker: Color::White,
            tables: Tables::new(192, true),
            line: Vec::new(),
            deadline: None,
            evals_cap: Some(2_000_000),
            child_evals: 0,
            expansions: 0,
            stop: None,
            sink: NullSink,
        };
        let r = engine.run(&parent);
        assert_eq!(r.exit, Exit::Proved);
        assert_eq!(r.length, Some(14));
    }
}
