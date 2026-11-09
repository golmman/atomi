// Justification for >10 KB: the pass structure, its D19/D21 rationale and
// the classifier differential tests are one contract.

//! Search frame v3: a node's child records while the node is searched, with
//! the hand-guide passes (D21).
//!
//! Entry runs in passes:
//!
//! 0. **Node status.** `populate_state` → `generate_legal_with_state` →
//!    [`rules::status_with`]: the node's own exact terminality, decided by
//!    the movegen it runs anyway. Non-`Open` → [`Entered::Terminal`].
//! 1. **Effects + guide.** Per legal move a [`crate::mover::Effect`] (child
//!    key, counter, `needs_board`). A `needs_board` move's child is built
//!    here. The guide class, starting numbers, the verified one-ply
//!    explosion settle on non-built children (screen first) and the bounded
//!    sibling simulation run here, while a child board is available.
//! 2. **Filter pass** (only while filtering): `seen = may_hold(key)` per
//!    child; the tables are prefetched for seen keys.
//! 3. **Classify**, M0's order: a board child's `Won` → terminal proof,
//!    `Failed { from }` → `Refuted { cycle: ANY, from }`; a non-board child
//!    with `counter >= 100` → `Refuted { ANY, from: 100 }`; the child key on
//!    the current line at index `i` → `Refuted { cycle: i, from: 0 }`; a
//!    stored lookup; else the guide's fresh `(pn, dn)` pair (identity
//!    `(1, 1)` when the guide is off). `UNLIKELY` priors are applied last.
//!
//! **D19 lazy stalemate.** Child classification is exact for explosion,
//! mate, material draw and the 50-move limit. A stalemate reached by a
//! quiet, non-checking move is detected lazily; the G1 gate builds every
//! child when the child's side to move has only commoner + pawns.
//!
//! **D21 one-ply settle.** A non-built child can still allow an immediate
//! explosion by its side to move (the parent move was quiet). A conservative
//! parent screen builds such a child and verifies the explosion with a real
//! legal move. A settled `Proved` child is returned in `pending`, so `mid`
//! stores its own proof entry and emits its event; a settled `Refuted` child
//! is stored as a path-independent TT refutation. Built children need no
//! settle: their `rules::status` already decides every immediate explosion.

use crate::guide::threat::{self, Settle};
use crate::guide::{self, Guide, History, MoveClass};
use crate::mover::Mover;
use crate::rules::{self, Status};
use crate::search::value::{ANY, Child, INFINITY, OPEN_MAX, Value, pn_dn};
use crate::table::Tables;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal_with_state;
use atomic_movegen::types::{Color, EnPassantMode, MoveList};

/// One deferred plain move of a [`Group`].
pub struct GroupMember {
    /// The member's child record (its own class, zeroing flag and value).
    pub child: Child,
    /// The member child's position key.
    pub key: u64,
}

/// The item-15 attacker plain-move `GROUP` pseudo-child.
///
/// The parent's OR frame gets one extra child representing all of its plain
/// moves. The members are not positions of the parent; they are the real
/// children of the plain moves. While a member is unsearched the group is a
/// deferred OR over the members (the group's `(9, #plain)` start until the
/// first member is installed). Once a member is proved the group *is* that
/// member's child value, so the parent's `or_value` adds the plain move
/// exactly once. When every member is refuted the group presents a refutation
/// that, after the parent's one adjustment, equals the OR of the members.
pub struct Group {
    /// The plain members, in the parent's movegen order.
    pub members: Vec<GroupMember>,
    /// The synthetic child's slot in `Frame::children` / `Frame::keys`.
    pub index: usize,
    /// Whether any member has been installed/searched.
    pub touched: bool,
    /// The group's starting pair `(9, max(1, #plain))`.
    pub start: (u32, u32),
}

impl Group {
    /// Recompute the synthetic child (and its key) from the members.
    pub fn sync(&mut self, child: &mut Child, key: &mut u64) {
        // Proved: the group is the best proved member's child value. No extra
        // ply is added here; the parent's OR adds the plain move.
        let mut best: Option<usize> = None;
        for (i, m) in self.members.iter().enumerate() {
            let Value::Proved { length, limit, .. } = m.child.value else {
                continue;
            };
            let better = match best {
                None => true,
                Some(j) => {
                    let Value::Proved {
                        length: bl,
                        limit: bli,
                        ..
                    } = self.members[j].child.value
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
        if let Some(i) = best {
            let m = &self.members[i];
            child.mv = m.child.mv;
            child.zeroing = m.child.zeroing;
            child.class = m.child.class;
            child.value = m.child.value;
            *key = m.key;
            return;
        }
        // Refuted: every member refuted. `from` is pre-adjusted by one so the
        // parent's OR subtraction lands on the members' own combination.
        if self
            .members
            .iter()
            .all(|m| matches!(m.child.value, Value::Refuted { .. }))
        {
            let mut cycle = ANY;
            let mut from = 0u8;
            for m in &self.members {
                let Value::Refuted { cycle: c, from: f } = m.child.value else {
                    unreachable!()
                };
                cycle = cycle.min(c);
                from = from.max(if m.child.zeroing { 1 } else { f });
            }
            child.zeroing = false;
            child.mv = self.members[0].child.mv;
            child.class = self.members[0].child.class;
            child.value = Value::Refuted { cycle, from };
            *key = self.members[0].key;
            return;
        }
        // Open: the deferred OR over the member values, or the group start
        // until the first member is installed.
        let any_decided = self
            .members
            .iter()
            .any(|m| !matches!(m.child.value, Value::Open { .. }));
        let (pn, dn) = if !self.touched && !any_decided {
            self.start
        } else {
            let mut pn = INFINITY;
            let mut dn = 0u64;
            for m in &self.members {
                let (p, d) = pn_dn(&m.child.value);
                pn = pn.min(p);
                dn += u64::from(d);
            }
            (pn, dn.min(u64::from(OPEN_MAX)) as u32)
        };
        // A representative open member supplies the tie-break move; the
        // parent's proof choice replaces it when the group is proved.
        let rep = self
            .members
            .iter()
            .find(|m| matches!(m.child.value, Value::Open { .. }))
            .unwrap_or(&self.members[0]);
        child.mv = rep.child.mv;
        child.zeroing = rep.child.zeroing;
        child.class = rep.child.class;
        child.value = Value::Open { pn, dn };
        *key = rep.key;
    }
}

/// One node's search state while it is on the MID stack.
pub struct Frame {
    /// Child records in movegen order.
    pub children: Vec<Child>,
    /// Child position keys, parallel to `children`.
    pub keys: Vec<u64>,
    /// `expansions` when the frame was entered (for the table `work` field).
    pub work_start: u64,
    /// Whether the node is an attacker (OR) node.
    pub attacker_node: bool,
    /// The node's position key (seeded tie-break).
    pub key: u64,
    /// The side to move at this node.
    pub mover: Color,
    /// Whether the move-class tie-break is active.
    pub order_classes: bool,
    /// Whether the history tie-break is active.
    pub order_history: bool,
    /// The item-15 plain-move `GROUP` child, when active.
    pub group: Option<Group>,
}

/// What `Frame::enter` decided.
pub enum Entered {
    /// The node is terminal; the value to store and return.
    Terminal(Value),
    /// An open node: the frame, the child evals spent, the settle work, the
    /// simulation work, the optional operations skipped by the global
    /// auxiliary budget, and the exact settled children `(key, value)` that
    /// `mid` must store.
    Open(Frame, u64, u64, u64, u64, Vec<(u64, Value)>),
}

impl Frame {
    /// Enter node `board` (its own `key`, already on `line`).
    #[allow(clippy::too_many_arguments)]
    pub fn enter(
        board: &Board,
        key: u64,
        attacker: Color,
        line: &[u64],
        tables: &Tables,
        expansions: u64,
        guide: &Guide,
        history: &History,
        _seed: u64,
        aux_left: u64,
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
        let mover = board.side_to_move();
        let attacker_node = mover == attacker;
        let ctx = guide.context(
            board,
            &state,
            u32::try_from(moves.len()).unwrap_or(u32::MAX),
            !attacker_node,
        );
        let mover_ctx = Mover::new(board);
        let guide_on = guide.config.enabled;
        let max_settle = u64::from(guide.params.max_settle);
        let max_sim = u64::from(guide.params.max_sim);
        let group_enabled = guide_on && guide.config.group && attacker_node;

        // Pass 1 (effects + guide).
        let mut keys = Vec::with_capacity(moves.len());
        let mut children = Vec::with_capacity(moves.len());
        let mut plain_flags: Vec<bool> = if group_enabled {
            Vec::with_capacity(moves.len())
        } else {
            Vec::new()
        };
        let mut starts: Vec<(u32, u32)> = if guide_on {
            Vec::with_capacity(moves.len())
        } else {
            Vec::new()
        };
        let mut pending: Vec<(u64, Value)> = Vec::new();
        let mut settle_nodes = 0u64;
        let mut sim_nodes = 0u64;
        let mut aux_skipped = 0u64;
        let aux_available = |settle: u64, sim: u64| aux_left == u64::MAX || settle + sim < aux_left;
        for m in &moves {
            let e = mover_ctx.effect(board, m);
            let mut built: Option<Board> = None;
            let (child_key, mut value) = if e.needs_board {
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
                let k = child.hash_with(EnPassantMode::Legal);
                built = Some(child);
                (k, value)
            } else {
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

            // A fresh child may allow an immediate explosion by its side to
            // move. The screen avoids a build for most non-built children;
            // every hit (and every already-built child) is verified exactly.
            let settle_wanted = matches!(value, Value::Open { .. })
                && guide_on
                && guide.config.static_closures
                && (max_settle == 0 || settle_nodes < max_settle)
                && (built.is_some() || threat::settle_screen(board, m, mover));
            if settle_wanted && !aux_available(settle_nodes, sim_nodes) {
                aux_skipped += 1;
            }
            if settle_wanted && aux_available(settle_nodes, sim_nodes) {
                if built.is_none() {
                    let mut child = board.clone();
                    child.do_move(m, &mut StateInfo::new());
                    settle_nodes += 1;
                    built = Some(child);
                }
                let child = built.as_ref().expect("child built above");
                settle_nodes += 1;
                if let Some((settle, work)) = threat::one_ply_settle(child, attacker) {
                    settle_nodes += work;
                    value = match settle {
                        Settle::SideToMoveWins(mv) => Value::Proved {
                            length: 1,
                            limit: 99,
                            mv: Some(mv),
                        },
                        Settle::SideToMoveLoses(_) => Value::Refuted {
                            cycle: ANY,
                            from: 0,
                        },
                    };
                    pending.push((child_key, value));
                }
            }

            let class = if guide_on {
                guide::class::classify(
                    board,
                    m,
                    &e,
                    built.as_ref(),
                    ctx.in_check && guide.config.check_evasions,
                )
            } else {
                MoveClass::Quiet
            };
            if group_enabled {
                plain_flags.push(
                    !ctx.in_check
                        && !matches!(class, MoveClass::Threat | MoveClass::WeakThreat)
                        && mover_ctx.is_plain(board, m),
                );
            }
            let child_ref = built.as_ref();
            let child_eval = if !guide_on {
                0
            } else if let Some(child) = child_ref {
                let e = guide::eval::hand_eval(child);
                if child.side_to_move() == attacker {
                    e
                } else {
                    -e
                }
            } else if attacker_node {
                ctx.eval
            } else {
                -ctx.eval
            };
            let mut start = if matches!(value, Value::Open { .. }) {
                guide.start_numbers(&ctx, class, attacker_node, mover, m, history, child_eval)
            } else {
                (0, 0)
            };
            if let Some(child) = child_ref {
                let sim_wanted = matches!(value, Value::Open { .. })
                    && guide_on
                    && guide.config.sibling_sim
                    && (max_sim == 0 || sim_nodes < max_sim);
                if sim_wanted && !aux_available(settle_nodes, sim_nodes) {
                    aux_skipped += 1;
                }
                if sim_wanted && aux_available(settle_nodes, sim_nodes) {
                    let plies = guide.params.sim_plies.clamp(1, 255) as u8;
                    let (score, moves_made) = guide::simulate::playout_with(child, attacker, plies);
                    sim_nodes += moves_made;
                    start = guide.sim_adjust(start.0, start.1, score);
                }
                if matches!(value, Value::Open { .. })
                    && !attacker_node
                    && guide.config.defender_priors
                    && matches!(guide.replay(&ctx, child, attacker), Some((false, false)))
                {
                    start = guide.unlikely(start.0, start.1);
                }
            }
            children.push(Child {
                mv: m,
                zeroing: e.zeroing,
                class,
                value,
            });
            keys.push(child_key);
            if guide_on {
                starts.push(start);
            }
            if matches!(value, Value::Open { .. }) {
                if filtering {
                    tables.prefetch_filter(child_key);
                } else {
                    tables.prefetch(child_key);
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

        // Pass 3: repetition + lookup + fresh guide numbers.
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
            } else {
                let (pn, dn) = if guide_on { starts[i] } else { (1, 1) };
                c.value = Value::Open { pn, dn };
            }
        }
        // Item-15 GROUP: partition the plain moves out of the OR children and
        // install one synthetic child for them. The group is appended last so
        // the non-group movegen order is untouched.
        let mut group = None;
        if group_enabled && plain_flags.iter().any(|&p| p) {
            let mut members = Vec::new();
            let old_children = std::mem::take(&mut children);
            let old_keys = std::mem::take(&mut keys);
            let old_starts = std::mem::take(&mut starts);
            for (i, child) in old_children.into_iter().enumerate() {
                if plain_flags[i] {
                    members.push(GroupMember {
                        child,
                        key: old_keys[i],
                    });
                } else {
                    children.push(child);
                    keys.push(old_keys[i]);
                    if guide_on {
                        starts.push(old_starts[i]);
                    }
                }
            }
            let n = members.len() as u32;
            let start = (9u32, n.max(1));
            let mut gchild = Child {
                mv: members[0].child.mv,
                zeroing: members[0].child.zeroing,
                class: members[0].child.class,
                value: Value::Open {
                    pn: start.0,
                    dn: start.1,
                },
            };
            let mut gkey = members[0].key;
            let index = children.len();
            let mut g = Group {
                members,
                index,
                touched: false,
                start,
            };
            g.sync(&mut gchild, &mut gkey);
            children.push(gchild);
            keys.push(gkey);
            if guide_on {
                starts.push(start);
            }
            group = Some(g);
        }
        let frame = Frame {
            children,
            keys,
            work_start: expansions,
            attacker_node,
            key,
            mover,
            order_classes: guide.config.enabled
                && (guide.config.move_classes
                    || guide.config.quiet_grouping
                    || guide.config.check_evasions),
            order_history: guide.config.enabled && guide.config.history_ties,
            group,
        };
        let spent = moves.len() as u64;
        Entered::Open(frame, spent, settle_nodes, sim_nodes, aux_skipped, pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::GuideConfig;
    use crate::testutil;
    use atomic_movegen::types::Square;

    fn enter(board: &Board, key: u64, attacker: Color, line: &[u64], tables: &Tables) -> Entered {
        Frame::enter(
            board,
            key,
            attacker,
            line,
            tables,
            0,
            &Guide::new(GuideConfig::none()),
            &History::new(),
            0,
            u64::MAX,
        )
    }

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
    fn classifier_matches_m0_with_the_guide_off() {
        let tables = Tables::new(1, true);
        let mut lazy = 0usize;
        let mut children = 0usize;
        for (board, line, attacker) in testutil::positions() {
            let mut state = StateInfo::new();
            board.populate_state(&mut state);
            let mut moves = MoveList::new();
            generate_legal_with_state(&board, &state, &mut moves);
            let st = rules::status_with(&board, attacker, &state, !moves.is_empty());
            if st != Status::Open {
                if let Status::Failed { from } = st {
                    let key = *line.last().unwrap();
                    match enter(&board, key, attacker, &line, &tables) {
                        Entered::Terminal(v) => {
                            assert_eq!(v, Value::Refuted { cycle: ANY, from });
                        }
                        Entered::Open(..) => panic!("terminal node opened a frame"),
                    }
                }
                continue;
            }
            let key = *line.last().unwrap();
            let Entered::Open(frame, _, _, _, _, _) = enter(&board, key, attacker, &line, &tables)
            else {
                panic!("open node was terminal");
            };
            for (i, m) in (&moves).into_iter().enumerate() {
                children += 1;
                let want = m0_classify(&board, m, attacker, &line);
                let got = frame.children[i].value;
                if want != got {
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
        eprintln!("classifier: children={children} lazy_stalemates={lazy}");
    }

    #[test]
    fn disabled_guide_keeps_m1_numbers() {
        let tables = Tables::new(4, true);
        let b =
            Board::from_fen("r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();
        let key = rules::key(&b);
        let line = vec![key];
        let Entered::Open(frame, _, _, _, _, pending) =
            enter(&b, key, Color::White, &line, &tables)
        else {
            panic!("expected an open frame");
        };
        assert!(pending.is_empty());
        for c in &frame.children {
            if matches!(c.value, Value::Open { .. }) {
                assert_eq!(c.value, Value::Open { pn: 1, dn: 1 });
            }
        }
    }

    fn mv_of(from: Square, to: Square) -> atomic_movegen::types::Move {
        atomic_movegen::types::Move::make_move(from, to)
    }

    fn member(mv: atomic_movegen::types::Move, zeroing: bool, value: Value) -> GroupMember {
        GroupMember {
            child: Child {
                mv,
                zeroing,
                class: MoveClass::Quiet,
                value,
            },
            key: u64::from(mv.to_bits()),
        }
    }

    /// The invariant: once every member is classified, the synthetic group
    /// child is exactly equivalent to the real member children under the
    /// parent's OR value (the group adds no ply and no limit step).
    #[test]
    fn group_value_equals_member_or_when_classified() {
        let m1 = mv_of(Square::A1, Square::A2);
        let m2 = mv_of(Square::B1, Square::B2);
        let m3 = mv_of(Square::C1, Square::C2);
        let cases: Vec<Vec<Value>> = vec![
            vec![Value::Proved {
                length: 3,
                limit: 40,
                mv: Some(m1),
            }],
            vec![
                Value::Proved {
                    length: 4,
                    limit: 90,
                    mv: Some(m1),
                },
                Value::Proved {
                    length: 2,
                    limit: 30,
                    mv: Some(m2),
                },
            ],
            vec![
                Value::Refuted { cycle: 3, from: 40 },
                Value::Refuted { cycle: 7, from: 60 },
            ],
            vec![
                Value::Refuted { cycle: 2, from: 10 },
                Value::Refuted {
                    cycle: ANY,
                    from: 9,
                },
                Value::Refuted { cycle: 5, from: 80 },
            ],
        ];
        for values in cases {
            let zeroing: Vec<bool> = values.iter().enumerate().map(|(i, _)| i % 2 == 1).collect();
            let moves = [m1, m2, m3];
            let members: Vec<GroupMember> = values
                .iter()
                .enumerate()
                .map(|(i, &v)| member(moves[i], zeroing[i], v))
                .collect();
            let expected_children: Vec<Child> = members
                .iter()
                .map(|m| Child {
                    mv: m.child.mv,
                    zeroing: m.child.zeroing,
                    class: m.child.class,
                    value: m.child.value,
                })
                .collect();
            let expected = crate::search::value::or_value(&expected_children);
            let mut g = Group {
                members,
                index: 0,
                touched: true,
                start: (9, 3),
            };
            let mut child = Child {
                mv: m1,
                zeroing: false,
                class: MoveClass::Quiet,
                value: Value::Open { pn: 9, dn: 3 },
            };
            let mut key = 0;
            g.sync(&mut child, &mut key);
            let got = crate::search::value::or_value(&[child]);
            assert_eq!(got, expected, "values {values:?}");
        }
    }

    #[test]
    fn group_starts_at_nine_by_plain_count() {
        let m1 = mv_of(Square::A1, Square::A2);
        let m2 = mv_of(Square::B1, Square::B2);
        let mut g = Group {
            members: vec![
                member(m1, false, Value::Open { pn: 9, dn: 9 }),
                member(m2, false, Value::Open { pn: 9, dn: 9 }),
            ],
            index: 0,
            touched: false,
            start: (9, 2),
        };
        let mut child = Child {
            mv: m1,
            zeroing: false,
            class: MoveClass::Quiet,
            value: Value::Open { pn: 9, dn: 2 },
        };
        let mut key = 0;
        g.sync(&mut child, &mut key);
        assert_eq!(child.value, Value::Open { pn: 9, dn: 2 });
        g.touched = true;
        g.sync(&mut child, &mut key);
        assert_eq!(child.value, Value::Open { pn: 9, dn: 18 });
    }

    #[test]
    fn group_is_installed_only_for_plain_moves() {
        // A quiet position with several plain moves: the frame carries one
        // extra synthetic child and every member is plain.
        let b = Board::from_fen("4k2n/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let key = rules::key(&b);
        let line = vec![key];
        let mut config = GuideConfig::hand();
        config.group = true;
        let tables = Tables::new(4, true);
        let Entered::Open(frame, _, _, _, _, _) = Frame::enter(
            &b,
            key,
            Color::White,
            &line,
            &tables,
            0,
            &Guide::new(config),
            &History::new(),
            0,
            u64::MAX,
        ) else {
            panic!("expected an open frame");
        };
        let g = frame.group.as_ref().expect("group installed");
        assert_eq!(g.index + 1, frame.children.len());
        assert!(!g.members.is_empty());
        let mover = Mover::new(&b);
        for m in &g.members {
            assert!(mover.is_plain(&b, m.child.mv), "{}", m.child.mv);
        }
        assert_eq!(g.start, (9, g.members.len() as u32));
    }

    #[test]
    fn settled_child_is_pending_and_exact() {
        // White to move; the quiet move Kb7? Construct a position where a
        // quiet move allows Black an immediate explosion. White commoner b1,
        // black rook b8 with the b-file blocked by a white pawn b2; a white
        // quiet move that does not move b2 leaves the threat. Simpler: use a
        // playout search below for real hits.
        let tables = Tables::new(4, true);
        let mut hits = 0usize;
        let guide = Guide::new(GuideConfig::hand());
        for (board, line, attacker) in testutil::positions().into_iter().take(400) {
            let key = *line.last().unwrap();
            let mut state = StateInfo::new();
            board.populate_state(&mut state);
            let mut moves = MoveList::new();
            generate_legal_with_state(&board, &state, &mut moves);
            if rules::status_with(&board, attacker, &state, !moves.is_empty()) != Status::Open {
                continue;
            }
            let Entered::Open(frame, _, _, _, _, pending) = Frame::enter(
                &board,
                key,
                attacker,
                &line,
                &tables,
                0,
                &guide,
                &History::new(),
                0,
                u64::MAX,
            ) else {
                continue;
            };
            for &(k, v) in &pending {
                hits += 1;
                let i = frame
                    .keys
                    .iter()
                    .position(|&x| x == k)
                    .expect("pending key");
                assert!(matches!(v, Value::Proved { .. } | Value::Refuted { .. }));
                assert_eq!(frame.children[i].value, v);
            }
        }
        eprintln!("settled children: {hits}");
    }
}
