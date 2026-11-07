// Justification for >10 KB: the pass structure, its D19 rationale and the
// classifier-vs-M0 differential test are one contract.

//! Search frame v2: a node's child records while the node is searched.
//!
//! Entry runs in passes:
//!
//! 0. **Node status.** `populate_state` → `generate_legal_with_state` →
//!    [`rules::status_with`]: the node's own exact terminality, decided by
//!    the movegen it runs anyway (this makes every expanded node's
//!    terminality exact, so no frame is ever built at a position that is
//!    over). Non-`Open` → [`Entered::Terminal`], no children initialized.
//! 1. **Effects.** Per legal move a [`crate::mover::Effect`] (child key,
//!    counter, `needs_board`). A `needs_board` move's child is built here
//!    (clone + `do_move`), its key comes from `hash_with(Legal)` and its
//!    [`rules::status`] is kept for pass 3 — cheaper than building twice.
//!    Non-terminal children get their first-level prefetch.
//! 2. **Filter pass** (only while filtering): `seen = may_hold(key)` per
//!    child; the tables are prefetched for seen keys.
//! 3. **Classify**, M0's order: a board child's `Won` → terminal proof,
//!    `Failed { from }` → `Refuted { cycle: ANY, from }`; a non-board child
//!    with `counter >= 100` → `Refuted { ANY, from: 100 }` (it gives no
//!    check, so the clock decides, the same as `status`); the child key on
//!    the current line at index `i` → `Refuted { cycle: i, from: 0 }`
//!    (repetition is never a win); a stored lookup — only when `seen` (or
//!    not filtering); else fresh `(1, 1)`.
//!
//! **D19 lazy stalemate.** Child classification is exact for explosion,
//! mate, material draw and the 50-move limit: an explosion is a capture,
//! a mate needs a checking move, and material changes need a capture — all
//! `needs_board` classes. A *stalemate reached by a quiet, non-checking
//! move* is detected lazily only when it cannot be cheap to miss: when the
//! child's side to move has material beyond commoner + pawns the child
//! starts `Open(1, 1)` and its node-entry status refutes it on expansion;
//! when it has only commoner + pawns the G1 gate builds every child, so the
//! stalemate is exact at the parent (a lazy AND-node refutation forced
//! whole sibling proofs — the rem01 regression). Only
//! completeness-ordering and speed are affected.
//!
//! The records persist for the frame's lifetime so path-dependent results
//! (which the tables never store) cannot make the MID loop spin.

use crate::mover::Mover;
use crate::rules::{self, Status};
use crate::search::value::{ANY, Child, Value};
use crate::table::Tables;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal_with_state;
use atomic_movegen::types::{Color, EnPassantMode, MoveList};

/// One node's search state while it is on the MID stack.
pub struct Frame {
    /// Child records in movegen order.
    pub children: Vec<Child>,
    /// Child position keys, parallel to `children` (`keys[i]` is the key of
    /// `children[i].mv`'s position).
    pub keys: Vec<u64>,
    /// `expansions` when the frame was entered (for the table `work` field).
    pub work_start: u64,
    /// Whether the node is an attacker (OR) node.
    pub attacker_node: bool,
}

/// What `Frame::enter` decided.
pub enum Entered {
    /// The node is terminal; the value to store and return.
    Terminal(Value),
    /// An open node: the frame and the child evals spent (one per move).
    Open(Frame, u64),
}

impl Frame {
    /// Enter node `board` (its own `key`, already on `line`): decide its
    /// status and, when `Open`, initialize the child records.
    pub fn enter(
        board: &Board,
        key: u64,
        attacker: Color,
        line: &[u64],
        tables: &Tables,
        expansions: u64,
    ) -> Entered {
        debug_assert_eq!(key, *line.last().unwrap());
        // Pass 0: the node's own exact status from the movegen it runs anyway.
        let mut state = StateInfo::new();
        board.populate_state(&mut state);
        let mut moves = MoveList::new();
        generate_legal_with_state(board, &state, &mut moves);
        match rules::status_with(board, attacker, &state, !moves.is_empty()) {
            Status::Failed { from } => {
                return Entered::Terminal(Value::Refuted { cycle: ANY, from });
            }
            Status::Won => {
                debug_assert!(false, "entry Won: unreachable under D19");
                return Entered::Terminal(Value::Proved {
                    length: 0,
                    limit: 99,
                    mv: None,
                });
            }
            Status::Open => {}
        }
        let filtering = tables.filtering();
        let mover = Mover::new(board);

        // Pass 1 (effects): keys, board children, first-level prefetch.
        // Children decided here get their terminal value directly; the rest
        // start as `Open(1, 1)` placeholders for pass 3.
        let mut keys = Vec::with_capacity(moves.len());
        let mut children = Vec::with_capacity(moves.len());
        for m in &moves {
            let e = mover.effect(board, m);
            let (key, value) = if e.needs_board {
                let mut child = board.clone();
                child.do_move(m, &mut StateInfo::new());
                let value = match rules::status(&child, attacker) {
                    Status::Won => Value::Proved {
                        length: 0,
                        limit: 99,
                        mv: None,
                    },
                    Status::Failed { from } => Value::Refuted { cycle: ANY, from },
                    Status::Open => Value::Open { pn: 1, dn: 1 },
                };
                (child.hash_with(EnPassantMode::Legal), value)
            } else {
                // A non-board child gives no check, so `counter >= 100`
                // decides it the same way `status` does.
                let value = if e.counter >= 100 {
                    Value::Refuted {
                        cycle: ANY,
                        from: 100,
                    }
                } else {
                    Value::Open { pn: 1, dn: 1 }
                };
                (e.key, value)
            };
            children.push(Child {
                mv: m,
                zeroing: e.zeroing,
                value,
            });
            keys.push(key);
            if matches!(value, Value::Open { .. }) {
                if filtering {
                    tables.prefetch_filter(key);
                } else {
                    tables.prefetch(key);
                }
            }
        }

        // Pass 2 (filter): which keys may be stored, plus their prefetch.
        let mut seen: Vec<bool> = Vec::new();
        if filtering {
            seen.reserve(moves.len());
            for &k in &keys {
                let s = tables.may_hold(k);
                seen.push(s);
                if s {
                    tables.prefetch(k);
                }
            }
        }

        // Pass 3: repetition + lookup for children pass 1 left open.
        for (i, c) in children.iter_mut().enumerate() {
            if !matches!(c.value, Value::Open { .. }) {
                continue;
            }
            let counter = if c.zeroing { 0 } else { board.rule50() + 1 };
            if let Some(j) = line.iter().position(|&k| k == keys[i]) {
                c.value = Value::Refuted {
                    cycle: j as u16,
                    from: 0,
                };
            } else if seen.get(i).copied().unwrap_or(true)
                && let Some(v) = tables.lookup(keys[i], counter)
            {
                c.value = v;
            }
        }
        let frame = Frame {
            children,
            keys,
            work_start: expansions,
            attacker_node: board.side_to_move() == attacker,
        };
        Entered::Open(frame, moves.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;

    /// M0's classifier (clone + `status` + repetition + fresh), no tables.
    fn m0_classify(
        board: &Board,
        m: atomic_movegen::types::Move,
        attacker: Color,
        line: &[u64],
    ) -> Value {
        let mut next = board.clone();
        next.do_move(m, &mut StateInfo::new());
        match rules::status(&next, attacker) {
            Status::Won => {
                return Value::Proved {
                    length: 0,
                    limit: 99,
                    mv: None,
                };
            }
            Status::Failed { from } => {
                return Value::Refuted { cycle: ANY, from };
            }
            Status::Open => {}
        }
        let ck = rules::child_key(board, m);
        if let Some(i) = line.iter().position(|&k| k == ck) {
            return Value::Refuted {
                cycle: i as u16,
                from: 0,
            };
        }
        Value::Open { pn: 1, dn: 1 }
    }

    #[test]
    fn classifier_matches_m0() {
        // For every non-terminal playout position, pass-3 values equal M0's.
        // The only allowed difference is a lazy stalemate: M0 calls `Failed`
        // on a child with no legal move and no check, while the frame gives
        // `Open(1, 1)` — counted and printed.
        let tables = Tables::new(1, true);
        let mut lazy = 0usize;
        let mut children = 0usize;
        let mut terminals = 0usize;
        for (board, line, attacker) in testutil::positions() {
            let mut state = StateInfo::new();
            board.populate_state(&mut state);
            let mut moves = MoveList::new();
            generate_legal_with_state(&board, &state, &mut moves);
            let st = rules::status_with(&board, attacker, &state, !moves.is_empty());
            if st != Status::Open {
                terminals += 1;
                // Terminal nodes must produce Entered::Terminal — exercised
                // on `Failed` positions only: a `Won` node is unreachable in
                // the descent (a Won child is never expanded), which is what
                // `enter`'s `debug_assert` pins.
                if let Status::Failed { from } = st {
                    let key = *line.last().unwrap();
                    match Frame::enter(&board, key, attacker, &line, &tables, 0) {
                        Entered::Terminal(v) => {
                            assert_eq!(v, Value::Refuted { cycle: ANY, from });
                        }
                        Entered::Open(..) => panic!("terminal node opened a frame"),
                    }
                }
                continue;
            }
            let key = *line.last().unwrap();
            let Entered::Open(frame, _) = Frame::enter(&board, key, attacker, &line, &tables, 0)
            else {
                panic!("open node was terminal");
            };
            for (i, m) in (&moves).into_iter().enumerate() {
                children += 1;
                let want = m0_classify(&board, m, attacker, &line);
                let got = frame.children[i].value;
                if want != got {
                    // Allowed only with the G1 gate off: M0's Failed is a
                    // non-check stalemate — the child has no legal move and
                    // is not in check.
                    let mut next = board.clone();
                    next.do_move(m, &mut StateInfo::new());
                    let mut st = StateInfo::new();
                    next.populate_state(&mut st);
                    let mut ml = MoveList::new();
                    generate_legal_with_state(&next, &st, &mut ml);
                    assert!(
                        !crate::mover::Mover::new(&board).stalemate_plausible(),
                        "lazy diff with the G1 gate on at {board:?}"
                    );
                    assert!(
                        matches!(want, Value::Refuted { .. })
                            && matches!(got, Value::Open { pn: 1, dn: 1 })
                            && ml.is_empty()
                            && st.checkers.is_empty(),
                        "child {i} mismatch: want {want:?} got {got:?}"
                    );
                    lazy += 1;
                }
            }
        }
        eprintln!(
            "classifier_matches_m0: children={children} lazy_stalemates={lazy} terminals={terminals}"
        );
    }
}
