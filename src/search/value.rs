//! Justification for >10 KB: the value algebra and its exhaustive unit tests
//! are one contract; splitting would separate a rule from its tests.

//! Attacker-only value algebra: the proof/refutation/open value of a node as
//! a pure function of its children, in movegen order.
//!
//! (>10 KB note: the bulk is the hand-built child-set unit tests kept at the
//! bottom per the module-test convention; the value logic itself is ~150 lines.)
//!
//! A [`Value::Proved`] entry carries the proof's `length` (in attacker moves)
//! and `limit` (the highest 50-move counter at which it holds). A
//! [`Value::Refuted`] entry carries `cycle` (the ply index on the current
//! line of the shallowest repeated position it relies on; [`ANY`] when it is
//! path-independent) and `from` (the lowest counter at which it holds; `0` =
//! any counter).

use crate::guide::MoveClass;
use atomic_movegen::types::Move;

/// Saturating proof/disproof-number infinity.
pub const INFINITY: u32 = 1 << 30;
/// The largest number an undecided node may carry. Open sums saturate here
/// so an [`Value::Open`] never reaches [`INFINITY`], which marks decided
/// values only.
pub const OPEN_MAX: u32 = INFINITY - 1;
/// `cycle` marker for a path-independent refutation.
pub const ANY: u16 = u16::MAX;

/// The value of a node for the attacker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// Attacker win. `length` counts attacker moves; `limit` is the highest
    /// 50-move counter at which the proof holds. `mv` is `Some` at attacker
    /// nodes and `None` at defender nodes and terminal wins.
    Proved {
        length: u16,
        limit: u8,
        mv: Option<Move>,
    },
    /// No attacker win (as far as the search can tell). `cycle` is a ply
    /// index or [`ANY`]; `from` is the lowest counter the refutation holds at.
    Refuted { cycle: u16, from: u8 },
    /// Undecided, with proof and disproof numbers.
    Open { pn: u32, dn: u32 },
}

/// One legal move of a node and the value computed for it.
pub struct Child {
    /// The move, in movegen order.
    pub mv: Move,
    /// Whether the move resets the 50-move counter (pawn move or capture).
    pub zeroing: bool,
    /// The hand-guide class (only an ordering/starting-number hint).
    pub class: MoveClass,
    /// The child's value.
    pub value: Value,
}

fn is_proved(v: &Value) -> bool {
    matches!(v, Value::Proved { .. })
}

fn is_refuted(v: &Value) -> bool {
    matches!(v, Value::Refuted { .. })
}

/// The value of an attacker (OR) node from its children.
///
/// Proved by the best proved child: `length = 1 + child.length` and
/// `limit = 99` for a terminal win (`child.length == 0`) or a zeroing move,
/// else `child.limit - 1`. Ties: higher limit, then movegen order. Refuted
/// when every child is refuted: `cycle = min`, `from` the maximum adjusted
/// `from` over non-zeroing children (0 when there are none). Else `Open`
/// with `pn = min`, `dn = sum` saturating at [`OPEN_MAX`].
pub fn or_value(children: &[Child]) -> Value {
    if children.iter().any(|c| is_proved(&c.value)) {
        let mut best: Option<(u16, u8, Move)> = None;
        for c in children {
            let Value::Proved { length, limit, .. } = c.value else {
                continue;
            };
            let len = length.saturating_add(1);
            let lim = if length == 0 || c.zeroing {
                99
            } else {
                limit - 1
            };
            if best.is_none_or(|(bl, blim, _)| len < bl || (len == bl && lim > blim)) {
                best = Some((len, lim, c.mv));
            }
        }
        let (length, limit, mv) = best.unwrap();
        return Value::Proved {
            length,
            limit,
            mv: Some(mv),
        };
    }
    if children.iter().all(|c| is_refuted(&c.value)) {
        let mut cycle = ANY;
        let mut from = 0u8;
        for c in children {
            let Value::Refuted {
                cycle: cc,
                from: cf,
            } = c.value
            else {
                continue;
            };
            cycle = cycle.min(cc);
            if !c.zeroing {
                from = from.max(cf.saturating_sub(1));
            }
        }
        return Value::Refuted { cycle, from };
    }
    let mut pn = INFINITY;
    let mut dn = 0u64;
    for c in children {
        let (cpn, cdn) = pn_dn(&c.value);
        pn = pn.min(cpn);
        dn += u64::from(cdn);
    }
    Value::Open {
        pn,
        dn: dn.min(u64::from(OPEN_MAX)) as u32,
    }
}

/// The value of a defender (AND) node from its children.
///
/// Proved when every child is proved: `length = max(child.length)` and
/// `limit` the minimum over children of (`99` if the reply is zeroing else
/// `child.limit - 1`). Refuted by the best refuted child: prefer `cycle ==
/// ANY`, then the lowest adjusted `from` (0 if the reply is zeroing else
/// `child.from - 1`), then the highest `cycle`, then movegen order. Else
/// `Open` with `pn = sum`, `dn = min`.
///
/// A defender node always has at least one child: a moveless position is
/// terminal and never reaches a frame.
pub fn and_value(children: &[Child]) -> Value {
    debug_assert!(!children.is_empty());
    if children.iter().all(|c| is_proved(&c.value)) {
        let mut length = 0u16;
        let mut limit = 99u8;
        for c in children {
            let Value::Proved {
                length: cl,
                limit: clm,
                ..
            } = c.value
            else {
                unreachable!()
            };
            length = length.max(cl);
            if !c.zeroing {
                limit = limit.min(clm - 1);
            }
        }
        return Value::Proved {
            length,
            limit,
            mv: None,
        };
    }
    if children.iter().any(|c| is_refuted(&c.value)) {
        let mut best: Option<(u8, u16, u16)> = None; // (from, cycle, index)
        for (i, c) in children.iter().enumerate() {
            let Value::Refuted {
                cycle: cc,
                from: cf,
            } = c.value
            else {
                continue;
            };
            let from = if c.zeroing { 0 } else { cf.saturating_sub(1) };
            let better = match best {
                None => true,
                Some((bf, bc, _)) => {
                    if (cc == ANY) != (bc == ANY) {
                        cc == ANY
                    } else if from != bf {
                        from < bf
                    } else {
                        cc > bc
                    }
                }
            };
            if better {
                best = Some((from, cc, i as u16));
            }
        }
        let (from, cycle, _) = best.unwrap();
        return Value::Refuted { cycle, from };
    }
    let mut pn = 0u64;
    let mut dn = INFINITY;
    for c in children {
        let (cpn, cdn) = pn_dn(&c.value);
        pn += u64::from(cpn);
        dn = dn.min(cdn);
    }
    Value::Open {
        pn: pn.min(u64::from(OPEN_MAX)) as u32,
        dn,
    }
}

/// Frame-exit `cycle` reset: a refutation whose repeated position is the node
/// itself (`cycle != ANY && cycle >= ply`) becomes path-independent
/// (`cycle = ANY`), because it holds from every path through the node.
pub fn exit_value(v: Value, ply: u16) -> Value {
    if let Value::Refuted { cycle, from } = v
        && cycle != ANY
        && cycle >= ply
    {
        return Value::Refuted { cycle: ANY, from };
    }
    v
}

/// Proof and disproof numbers of a value.
pub fn pn_dn(v: &Value) -> (u32, u32) {
    match *v {
        Value::Proved { .. } => (0, INFINITY),
        Value::Refuted { .. } => (INFINITY, 0),
        Value::Open { pn, dn } => (pn, dn),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_movegen::types::Square;

    fn mv(from: Square, to: Square) -> Move {
        Move::make_move(from, to)
    }

    fn child(mv: Move, zeroing: bool, value: Value) -> Child {
        Child {
            mv,
            zeroing,
            class: MoveClass::Quiet,
            value,
        }
    }

    fn proved(length: u16, limit: u8) -> Value {
        Value::Proved {
            length,
            limit,
            mv: None,
        }
    }

    fn refuted(cycle: u16, from: u8) -> Value {
        Value::Refuted { cycle, from }
    }

    #[test]
    fn or_proof_choice_and_limit() {
        let m1 = mv(Square::A1, Square::A2);
        let m2 = mv(Square::B1, Square::B2);
        // Terminal win gives limit 99.
        assert_eq!(
            or_value(&[child(m1, false, proved(0, 99))]),
            Value::Proved {
                length: 1,
                limit: 99,
                mv: Some(m1)
            }
        );
        // Zeroing non-terminal gives limit 99; non-zeroing subtracts 1.
        assert_eq!(
            or_value(&[child(m1, true, proved(2, 80))]),
            Value::Proved {
                length: 3,
                limit: 99,
                mv: Some(m1)
            }
        );
        assert_eq!(
            or_value(&[child(m1, false, proved(2, 80))]),
            Value::Proved {
                length: 3,
                limit: 79,
                mv: Some(m1)
            }
        );
        // Shortest wins; ties go to the higher limit, then movegen order.
        let v = or_value(&[
            child(m1, false, proved(4, 90)),
            child(m2, false, proved(1, 50)),
        ]);
        assert_eq!(
            v,
            Value::Proved {
                length: 2,
                limit: 49,
                mv: Some(m2)
            }
        );
        let v = or_value(&[
            child(m1, false, proved(1, 50)),
            child(m2, true, proved(1, 30)),
        ]);
        assert_eq!(
            v,
            Value::Proved {
                length: 2,
                limit: 99,
                mv: Some(m2)
            }
        );
        let v = or_value(&[
            child(m1, false, proved(1, 50)),
            child(m2, false, proved(1, 60)),
        ]);
        assert_eq!(
            v,
            Value::Proved {
                length: 2,
                limit: 59,
                mv: Some(m2)
            }
        );
    }

    #[test]
    fn or_refutation_cycle_and_from() {
        let m1 = mv(Square::A1, Square::A2);
        let m2 = mv(Square::B1, Square::B2);
        let m3 = mv(Square::C1, Square::C2);
        // min cycle, max adjusted from over non-zeroing.
        let v = or_value(&[
            child(m1, false, refuted(3, 40)),
            child(m2, false, refuted(7, 60)),
            child(m3, true, refuted(1, 80)), // zeroing: from contributes 0
        ]);
        assert_eq!(v, Value::Refuted { cycle: 1, from: 59 });
        // All-zeroing children give from = 0.
        let v = or_value(&[
            child(m1, true, refuted(2, 40)),
            child(m2, true, refuted(ANY, 60)),
        ]);
        assert_eq!(v, Value::Refuted { cycle: 2, from: 0 });
        // A mixed set is Open.
        let v = or_value(&[
            child(m1, false, refuted(2, 0)),
            child(m2, false, Value::Open { pn: 3, dn: 4 }),
        ]);
        assert_eq!(v, Value::Open { pn: 3, dn: 4 });
    }

    #[test]
    fn and_proof_length_and_limit() {
        let m1 = mv(Square::A1, Square::A2);
        let m2 = mv(Square::B1, Square::B2);
        // max length, min over children of (99 if zeroing else limit - 1).
        let v = and_value(&[
            child(m1, false, proved(2, 90)),
            child(m2, true, proved(5, 10)),
        ]);
        assert_eq!(
            v,
            Value::Proved {
                length: 5,
                limit: 89,
                mv: None
            }
        );
        // All zeroing replies keep the limit at 99.
        let v = and_value(&[child(m1, true, proved(1, 50))]);
        assert_eq!(
            v,
            Value::Proved {
                length: 1,
                limit: 99,
                mv: None
            }
        );
    }

    #[test]
    fn and_refutation_child_preference() {
        let m1 = mv(Square::A1, Square::A2);
        let m2 = mv(Square::B1, Square::B2);
        let m3 = mv(Square::C1, Square::C2);
        // cycle == ANY wins over a lower from.
        let v = and_value(&[
            child(m1, false, refuted(5, 3)),
            child(m2, false, refuted(ANY, 40)),
        ]);
        assert_eq!(
            v,
            Value::Refuted {
                cycle: ANY,
                from: 39
            }
        );
        // Then the lowest adjusted from; a zeroing reply adjusts to 0.
        let v = and_value(&[
            child(m1, false, refuted(5, 10)),
            child(m2, true, refuted(9, 80)),
        ]);
        assert_eq!(v, Value::Refuted { cycle: 9, from: 0 });
        // Then the highest cycle.
        let v = and_value(&[
            child(m1, false, refuted(4, 10)),
            child(m2, false, refuted(8, 10)),
        ]);
        assert_eq!(v, Value::Refuted { cycle: 8, from: 9 });
        // Then movegen order.
        let v = and_value(&[
            child(m1, false, refuted(8, 10)),
            child(m2, false, refuted(8, 10)),
            child(m3, false, refuted(8, 10)),
        ]);
        assert_eq!(v, Value::Refuted { cycle: 8, from: 9 });
    }

    #[test]
    fn open_numbers() {
        let m1 = mv(Square::A1, Square::A2);
        let m2 = mv(Square::B1, Square::B2);
        let open = Value::Open { pn: 5, dn: 7 };
        assert_eq!(
            or_value(&[
                child(m1, false, open),
                child(m2, false, Value::Open { pn: 2, dn: 9 })
            ]),
            Value::Open { pn: 2, dn: 16 }
        );
        assert_eq!(
            and_value(&[
                child(m1, false, open),
                child(m2, false, Value::Open { pn: 2, dn: 9 })
            ]),
            Value::Open { pn: 7, dn: 7 }
        );
        // Open sums saturate at OPEN_MAX, strictly below INFINITY.
        let big = Value::Open {
            pn: OPEN_MAX,
            dn: OPEN_MAX,
        };
        let or = or_value(&[child(m1, false, big), child(m2, false, big)]);
        assert_eq!(
            or,
            Value::Open {
                pn: OPEN_MAX,
                dn: OPEN_MAX
            }
        );
        let and = and_value(&[child(m1, false, big), child(m2, false, big)]);
        assert_eq!(
            and,
            Value::Open {
                pn: OPEN_MAX,
                dn: OPEN_MAX
            }
        );
        let (pn, dn) = pn_dn(&or);
        assert!(pn < INFINITY && dn < INFINITY);
        let (pn, dn) = pn_dn(&and);
        assert!(pn < INFINITY && dn < INFINITY);
    }

    #[test]
    fn exit_value_cycle_reset() {
        // cycle >= ply and != ANY resets to ANY.
        assert_eq!(exit_value(refuted(5, 10), 5), refuted(ANY, 10));
        assert_eq!(exit_value(refuted(7, 10), 5), refuted(ANY, 10));
        // cycle < ply is kept.
        assert_eq!(exit_value(refuted(3, 10), 5), refuted(3, 10));
        // ANY is kept.
        assert_eq!(exit_value(refuted(ANY, 10), 5), refuted(ANY, 10));
        // Other values pass through.
        assert_eq!(exit_value(proved(1, 99), 3), proved(1, 99));
        assert_eq!(
            exit_value(Value::Open { pn: 1, dn: 1 }, 3),
            Value::Open { pn: 1, dn: 1 }
        );
    }

    #[test]
    fn pn_dn_mapping() {
        assert_eq!(pn_dn(&proved(1, 99)), (0, INFINITY));
        assert_eq!(pn_dn(&refuted(2, 5)), (INFINITY, 0));
        assert_eq!(pn_dn(&Value::Open { pn: 3, dn: 4 }), (3, 4));
    }
}
