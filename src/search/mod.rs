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
//! (the table is allocated before the clock starts). Deterministic: no
//! randomness and no hash-map iteration anywhere in the search path.

mod frame;
pub mod value;

use crate::events::{ProofEvent, ProofSink};
use crate::rules;
use crate::table::Table;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::types::Color;
use frame::Frame;
use std::time::{Duration, Instant};
use value::{Child, INFINITY, Value, and_value, exit_value, or_value, pn_dn};

/// The search budgets.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Wall-clock deadline in seconds (`None` = none).
    pub seconds: Option<u64>,
    /// Deterministic cap on `child_evals` (`None` = none).
    pub evals: Option<u64>,
    /// The transposition table's size label in MiB.
    pub memory_mb: usize,
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
}

const MAX_PLY: u16 = 4096;
const DEADLINE_CHECK_INTERVAL: u64 = 1024;

/// Solve `root` for the side to move (the attacker), returning the result
/// and the sink back so a proof store can be drained afterwards.
pub fn prove<S: ProofSink + Send + 'static>(
    root: &Board,
    limits: &Limits,
    sink: S,
) -> (SearchResult, S) {
    let root = root.clone();
    let limits = limits.clone();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let table = Table::new(limits.memory_mb);
            let start = Instant::now();
            let deadline = limits.seconds.map(|s| start + Duration::from_secs(s));
            let mut engine = Engine {
                attacker: root.side_to_move(),
                table,
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
            (result, engine.sink)
        })
        .expect("spawn search thread")
        .join()
        .expect("join search thread")
}

struct Engine<S> {
    attacker: Color,
    table: Table,
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
            };
        }
        let mut value = self.mid(root, 0, INFINITY, INFINITY);
        // A saturated Open value is still undecided: keep searching. Only a
        // Refuted root is a refutation; Open can never become one (the bug
        // this loop fixes was Open saturation reaching INFINITY).
        while self.stop.is_none() && matches!(value, Value::Open { .. }) {
            value = self.mid(root, 0, INFINITY, INFINITY);
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
        }
    }

    /// One MID pass over a node: loop selecting the best child until the
    /// node value crosses a threshold, then store and return it.
    fn mid(&mut self, board: &Board, ply: u16, tpn: u32, tdn: u32) -> Value {
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
        self.line.push(rules::key(board));
        let (mut frame, spent) = Frame::enter(
            board,
            self.attacker,
            &self.line,
            &self.table,
            self.expansions,
        );
        self.child_evals += spent;
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
            let cv = self.mid(&next, ply + 1, ctpn, ctdn);
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
        self.table.store(key, &v, work);
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
