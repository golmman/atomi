//! Search frame: a node's child records while the node is searched.
//!
//! On entry the frame generates the legal moves and initializes one
//! [`Child`] record per move in movegen order. Initialization order:
//!
//! 1. clone + make the move; `Status::Won` → terminal proof,
//!    `Status::Failed { from }` → `Refuted { cycle: ANY, from }`;
//! 2. the child key on the current line at index `i` →
//!    `Refuted { cycle: i, from: 0 }` (repetition is never a win);
//! 3. table probe: a usable entry gives its value, anything else `(1, 1)`.
//!
//! The records persist for the frame's lifetime so path-dependent results
//! (which the table never stores) cannot make the MID loop spin.

use crate::rules;
use crate::search::value::{ANY, Child, Value};
use crate::table::{Table, usable};
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, MoveList};

/// One node's search state while it is on the MID stack.
pub struct Frame {
    /// Child records in movegen order.
    pub children: Vec<Child>,
    /// `expansions` when the frame was entered (for the table `work` field).
    pub work_start: u64,
    /// Whether the node is an attacker (OR) node.
    pub attacker_node: bool,
}

impl Frame {
    /// Enter a node: generate children and initialize their values.
    ///
    /// Returns the frame and the number of child evaluations spent (one per
    /// legal move). `line` holds the position keys of the current path
    /// (index = ply); `expansions` is the counter at entry.
    pub fn enter(
        board: &Board,
        attacker: Color,
        line: &[u64],
        table: &Table,
        expansions: u64,
    ) -> (Frame, u64) {
        let mut moves = MoveList::new();
        generate_legal(board, &mut moves);
        let counter = board.rule50();
        let mut children = Vec::with_capacity(moves.len());
        for m in &moves {
            let zeroing = rules::is_zeroing(board, m);
            let child_counter = if zeroing { 0 } else { counter + 1 };
            let value = classify(board, m, attacker, line, table, child_counter);
            children.push(Child {
                mv: m,
                zeroing,
                value,
            });
        }
        let frame = Frame {
            children,
            work_start: expansions,
            attacker_node: board.side_to_move() == attacker,
        };
        (frame, moves.len() as u64)
    }
}

fn classify(
    board: &Board,
    m: atomic_movegen::types::Move,
    attacker: Color,
    line: &[u64],
    table: &Table,
    child_counter: u16,
) -> Value {
    let mut next = board.clone();
    next.do_move(m, &mut StateInfo::new());
    match rules::status(&next, attacker) {
        rules::Status::Won => {
            return Value::Proved {
                length: 0,
                limit: 99,
                mv: None,
            };
        }
        rules::Status::Failed { from } => {
            return Value::Refuted { cycle: ANY, from };
        }
        rules::Status::Open => {}
    }
    let ck = rules::child_key(board, m);
    if let Some(i) = line.iter().position(|&k| k == ck) {
        return Value::Refuted {
            cycle: i as u16,
            from: 0,
        };
    }
    if let Some(entry) = table.probe(ck)
        && let Some(v) = usable(entry, child_counter)
    {
        return v;
    }
    Value::Open { pn: 1, dn: 1 }
}
