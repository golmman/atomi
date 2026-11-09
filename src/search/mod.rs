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
use crate::guide::{Guide, GuideConfig, GuideParams, History};
use crate::rules;
use crate::table::Tables;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::types::Color;
use frame::{Entered, Frame, Group};
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
    /// The hand-guide configuration (`--guide none|hand` + ablations).
    pub guide: GuideConfig,
    /// The starting-number table (`--guide-params` or built-in).
    pub guide_params: GuideParams,
    /// Share of `--memory` given to the proof table, in percent.
    pub proof_share_percent: u8,
    /// The tie-break seed (0 = movegen order).
    pub seed: u64,
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
    /// Settle movegens/makes (not in `child_evals`/`expansions`).
    pub settle_nodes: u64,
    /// Simulation expansions (not in `child_evals`/`expansions`).
    pub sim_nodes: u64,
    /// Total auxiliary work units (settle + simulation) actually spent.
    pub aux_nodes: u64,
    /// Optional auxiliary operations skipped by the global budget.
    pub aux_skipped: u64,
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
            let tables =
                Tables::with_share(limits.memory_mb, limits.filter, limits.proof_share_percent);
            let start = Instant::now();
            let deadline = limits.seconds.map(|s| start + Duration::from_secs(s));
            let guide = Guide::new(limits.guide).with_params(limits.guide_params);
            let mut engine = Engine {
                attacker: root.side_to_move(),
                tables,
                line: Vec::new(),
                deadline,
                evals_cap: limits.evals,
                child_evals: 0,
                expansions: 0,
                settle_nodes: 0,
                sim_nodes: 0,
                aux_nodes: 0,
                aux_skipped: 0,
                stop: None,
                sink,
                guide,
                history: History::new(),
                seed: limits.seed,
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
    settle_nodes: u64,
    sim_nodes: u64,
    aux_nodes: u64,
    aux_skipped: u64,
    stop: Option<Exit>,
    sink: S,
    guide: Guide,
    history: History,
    seed: u64,
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
                settle_nodes: 0,
                sim_nodes: 0,
                aux_nodes: 0,
                aux_skipped: 0,
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
            settle_nodes: self.settle_nodes,
            sim_nodes: self.sim_nodes,
            aux_nodes: self.aux_nodes,
            aux_skipped: self.aux_skipped,
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
        let mut proved_pending: Vec<(u64, Value)> = Vec::new();
        if self.expansions.is_multiple_of(DEADLINE_CHECK_INTERVAL)
            && let Some(dl) = self.deadline
            && Instant::now() >= dl
        {
            self.stop = Some(Exit::Timeout);
            return Value::Open { pn: 0, dn: 0 };
        }
        debug_assert_eq!(key, rules::key(board));
        self.line.push(key);
        let aux_left = if self.guide.params.aux_budget == 0 {
            u64::MAX
        } else {
            self.guide.params.aux_budget.saturating_sub(self.aux_nodes)
        };
        let mut frame = match Frame::enter(
            board,
            key,
            self.attacker,
            &self.line,
            &self.tables,
            self.expansions,
            &self.guide,
            &self.history,
            self.seed,
            aux_left,
        ) {
            Entered::Terminal(v) => {
                // The node is over: no children were initialized (the
                // expansion above ran the movegen that decided it). A
                // terminal proof is a no-op in `Tables::store`.
                self.tables.store(key, &v, 0);
                self.line.pop();
                return v;
            }
            Entered::Open(frame, spent, settle, sim, skipped, pending) => {
                self.child_evals += spent;
                self.settle_nodes += settle;
                self.sim_nodes += sim;
                self.aux_nodes += settle + sim;
                self.aux_skipped += skipped;
                // Plan4 proof-table pressure: a settled refutation is still a
                // path-independent TT refutation, but a settled proof is only
                // installed when this node exits proved (and, at an OR node,
                // only for the child the OR value selects).
                proved_pending.clear();
                for (child_key, value) in pending {
                    if matches!(value, Value::Proved { .. }) {
                        proved_pending.push((child_key, value));
                    } else {
                        self.tables.store(child_key, &value, 0);
                    }
                }
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
            let (best, second) = select(&frame, &self.history, self.seed);
            let (ctpn, ctdn) = child_thresholds(&frame, best, second, tpn, tdn, pn, dn);
            if frame.group.as_ref().is_some_and(|g| g.index == best) {
                // Item-15 GROUP: install one plain member. The group is a
                // virtual OR node, so its member thresholds use the same
                // algebra with the group's own numbers as the node values.
                let group = frame.group.as_mut().expect("group index");
                let Some((mi, second_member)) =
                    select_member(group, frame.mover, &self.history, self.seed, frame.key)
                else {
                    // Every member is decided; a sync makes the child exact.
                    group.touched = true;
                    let child = &mut frame.children[best];
                    let key = &mut frame.keys[best];
                    group.sync(child, key);
                    continue;
                };
                let (gpn, gdn) = pn_dn(&frame.children[best].value);
                let (mpn, mdn) = pn_dn(&group.members[mi].child.value);
                let (mtpn, mtdn) = thresholds(true, mpn, mdn, second_member, ctpn, ctdn, gpn, gdn);
                let mv = group.members[mi].child.mv;
                let mkey = group.members[mi].key;
                let mut next = board.clone();
                next.do_move(mv, &mut StateInfo::new());
                let cv = self.mid(&next, mkey, ply + 1, mtpn, mtdn);
                if self.stop.is_some() {
                    self.line.pop();
                    return Value::Open { pn: 0, dn: 0 };
                }
                if self.guide.config.enabled {
                    let good = frame.attacker_node && matches!(cv, Value::Proved { .. });
                    self.history.observe(frame.mover, mv, good);
                }
                group.members[mi].child.value = cv;
                group.touched = true;
                let child = &mut frame.children[best];
                let key = &mut frame.keys[best];
                group.sync(child, key);
                continue;
            }
            let mut next = board.clone();
            next.do_move(frame.children[best].mv, &mut StateInfo::new());
            let cv = self.mid(&next, frame.keys[best], ply + 1, ctpn, ctdn);
            if self.stop.is_some() {
                self.line.pop();
                return Value::Open { pn: 0, dn: 0 };
            }
            if self.guide.config.enabled {
                let good = (frame.attacker_node && matches!(cv, Value::Proved { .. }))
                    || (!frame.attacker_node && matches!(cv, Value::Refuted { .. }));
                self.history
                    .observe(frame.mover, frame.children[best].mv, good);
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
        if matches!(v, Value::Proved { .. }) {
            let selected = if frame.attacker_node {
                selected_proved_child(&frame).map(|i| frame.keys[i])
            } else {
                None
            };
            for (child_key, value) in &proved_pending {
                let keep = !frame.attacker_node || selected == Some(*child_key);
                if !keep {
                    continue;
                }
                self.tables.store(*child_key, value, 0);
                if let Value::Proved {
                    length,
                    limit,
                    mv: Some(mv),
                } = *value
                {
                    self.sink.proven(ProofEvent::AttackerProven {
                        key: *child_key,
                        mv,
                        length,
                        limit,
                    });
                }
            }
        }
        debug_assert!(value_invariant(&v, board.rule50()));
        self.line.pop();
        v
    }
}

/// The child `or_value` would select among proved children (same
/// length/higher-limit/movegen-order rule).
fn selected_proved_child(frame: &Frame) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, c) in frame.children.iter().enumerate() {
        let Value::Proved { length, limit, .. } = c.value else {
            continue;
        };
        let better = match best {
            None => true,
            Some(j) => {
                let Value::Proved {
                    length: bl,
                    limit: bli,
                    ..
                } = frame.children[j].value
                else {
                    unreachable!()
                };
                length < bl || (length == bl && limit > bli)
            }
        };
        if better {
            best = Some(i);
        }
    }
    best
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

/// The best child's index and the second-best value of the selecting metric
/// ([`INFINITY`] when there is no second). Ties compare, in order: the guide
/// class rank, the history score, the seeded hash (only for `seed != 0`) and
/// movegen order.
fn select(frame: &Frame, history: &History, seed: u64) -> (usize, u32) {
    let metric = |c: &Child| {
        let (pn, dn) = pn_dn(&c.value);
        if frame.attacker_node { pn } else { dn }
    };
    let tie = |c: &Child| -> (u8, u16, u64) {
        let class = if frame.order_classes {
            c.class.rank()
        } else {
            0
        };
        let hist = if frame.order_history {
            u16::MAX - history.score(frame.mover, c.mv)
        } else {
            u16::MAX
        };
        let salted = if seed != 0 {
            History::seeded(seed, frame.key, c.mv)
        } else {
            0
        };
        (class, hist, salted)
    };
    let mut best = usize::MAX;
    let mut best_m = INFINITY;
    let mut best_tie = (u8::MAX, u16::MAX, u64::MAX);
    let mut second = INFINITY;
    for (i, c) in frame.children.iter().enumerate() {
        let m = metric(c);
        let t = tie(c);
        if m < best_m {
            second = best_m;
            best_m = m;
            best = i;
            best_tie = t;
        } else if m == best_m {
            if t < best_tie {
                // The old best keeps the same metric, so it is still the
                // second-best value.
                best = i;
                best_tie = t;
            }
            if m < second {
                second = m;
            }
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
    thresholds(frame.attacker_node, cpn, cdn, second, tpn, tdn, pn, dn)
}

/// The 1+ε child thresholds at a node kind (used by the frame and by the
/// virtual GROUP OR node).
#[allow(clippy::too_many_arguments)]
fn thresholds(
    attacker_node: bool,
    cpn: u32,
    cdn: u32,
    second: u32,
    tpn: u32,
    tdn: u32,
    pn: u32,
    dn: u32,
) -> (u32, u32) {
    let (ctpn, ctdn) = if attacker_node {
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

/// Select the next open GROUP member and return its index plus the
/// second-best metric (the group is an OR, so the metric is `pn`). Ties use
/// the same class/history/seed path as [`select`].
fn select_member(
    group: &Group,
    mover: Color,
    history: &History,
    seed: u64,
    node_key: u64,
) -> Option<(usize, u32)> {
    let mut best = usize::MAX;
    let mut best_m = INFINITY;
    let mut best_tie = (u8::MAX, u16::MAX, u64::MAX);
    let mut second = INFINITY;
    for (i, m) in group.members.iter().enumerate() {
        if !matches!(m.child.value, Value::Open { .. }) {
            continue;
        }
        let metric = pn_dn(&m.child.value).0;
        let tie = (
            m.child.class.rank(),
            u16::MAX - history.score(mover, m.child.mv),
            if seed != 0 {
                History::seeded(seed, node_key, m.child.mv)
            } else {
                0
            },
        );
        if metric < best_m {
            second = best_m;
            best_m = metric;
            best = i;
            best_tie = tie;
        } else if metric == best_m {
            if tie < best_tie {
                best = i;
                best_tie = tie;
            }
            if metric < second {
                second = metric;
            }
        } else if metric < second {
            second = metric;
        }
    }
    (best != usize::MAX).then_some((best, second))
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
    fn selected_proved_child_matches_or_value() {
        // OR selects the shortest proof, then the higher limit, then the
        // earlier movegen slot. Only that child's settled proof is retained.
        let mv = atomic_movegen::types::Move::make_move(
            atomic_movegen::types::Square::A2,
            atomic_movegen::types::Square::A3,
        );
        let child = |length, limit| Child {
            mv,
            zeroing: false,
            class: crate::guide::MoveClass::Quiet,
            value: Value::Proved {
                length,
                limit,
                mv: Some(mv),
            },
        };
        let make = |children: Vec<Child>| Frame {
            keys: (0..children.len() as u64).collect(),
            children,
            work_start: 0,
            attacker_node: true,
            key: 0,
            mover: Color::White,
            order_classes: false,
            order_history: false,
            group: None,
        };
        assert_eq!(
            selected_proved_child(&make(vec![child(4, 90), child(3, 50), child(3, 80)])),
            Some(2)
        );
        assert_eq!(
            selected_proved_child(&make(vec![child(2, 90), child(3, 99)])),
            Some(0)
        );
        assert_eq!(
            selected_proved_child(&make(vec![child(2, 9), child(2, 9)])),
            Some(0)
        );
    }

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
            settle_nodes: 0,
            sim_nodes: 0,
            aux_nodes: 0,
            aux_skipped: 0,
            stop: None,
            sink: NullSink,
            guide: Guide::new(GuideConfig::none()),
            history: History::new(),
            seed: 0,
        };
        let key = rules::key(&parent);
        engine.line.push(key);
        let Entered::Open(frame, _, _, _, _, _) = Frame::enter(
            &parent,
            key,
            engine.attacker,
            &engine.line,
            &engine.tables,
            0,
            &engine.guide,
            &engine.history,
            engine.seed,
            u64::MAX,
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
            settle_nodes: 0,
            sim_nodes: 0,
            aux_nodes: 0,
            aux_skipped: 0,
            stop: None,
            sink: NullSink,
            guide: Guide::new(GuideConfig::none()),
            history: History::new(),
            seed: 0,
        };
        let r = engine.run(&parent);
        assert_eq!(r.exit, Exit::Proved);
        assert_eq!(r.length, Some(14));
    }
}
