//! Exact null-threat replay (plan 18, `--guide-null-exact`).
//!
//! At a defender node the attacker may have a winning move if the defender
//! passed (the "null" position). Provari's `replay_threat` finds such a move,
//! then replays its proof on every still-open defence: a defence that parries
//! the threat starts lower, one that does not is proven outright.
//!
//! The null board is **ephemeral**. It is derived from the current board with
//! the side to move replaced and the en-passant square cleared
//! ([`Board::null_for`], plan 31 F2), used for one exact ply, and dropped. It
//! is never keyed, looked up, stored, put on `line` or emitted; the position
//! keys and proofs that leave this module are the real children of the
//! current node.
//!
//! This is deliberately narrower than provari's null replay: it finds an
//! immediate explosion/mate, a move into a stored proof, or an exposed
//! mate-in-two (every defence lets the attacker explode). Check-every-evasion,
//! double-threat and `recomputed` proofs are out of scope for this flag.

use crate::guide::threat;
use crate::rules;
use crate::table::{CachedLimit, NullCache, Tables};
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::{for_each_legal_with_state, generate_legal};
use atomic_movegen::types::{Color, Move, MoveList};
use std::ops::ControlFlow;

/// A winning move for the attacker in the null position, with its proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullWin {
    /// The attacker's move if the defender passed.
    pub mv: Move,
    /// Proof length in attacker moves (1 for an immediate win, 2 for an
    /// exposed mate-in-two).
    pub length: u16,
    /// Highest 50-move counter at which the proof holds.
    pub limit: u8,
}

/// Build the null counterpart of `board`: the same pieces with `attacker` to
/// move and no en-passant target.
///
/// Plan 31 F2: [`Board::null_for`] replaces the FEN round-trip (write → split
/// → join → parse). The result is the same board the round-trip produced —
/// pieces, castling rights, cleared ep square, half-move clock, fullmove
/// number and the derived game ply — checked in
/// `tests/explosion_differential.rs`. It stays ephemeral either way: never
/// keyed, looked up, stored, put on `line` or emitted.
pub fn null_board(board: &Board, attacker: Color) -> Board {
    board.null_for(attacker)
}

/// The work a null replay spent. `moves` is the existing auxiliary work
/// unit (the null movegen); `replies` is observation-only (plan 25).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NullWork {
    /// Null moves scanned (the auxiliary work unit).
    pub moves: u64,
    /// Defender replies examined in the exposed-limit pass.
    pub replies: u64,
}

/// Find an exact winning move for `attacker` in the null position and return
/// it with the null board. `remaining` bounds the proof length. `screen`
/// enables the plan-25 NULL-S pass-2 signature screen (default off).
pub fn null_win(
    board: &Board,
    attacker: Color,
    tables: &mut Tables,
    remaining: u16,
    screen: bool,
    work: &mut NullWork,
) -> Option<(Board, NullWin)> {
    null_win_pipelined::<PASS1_CHUNK>(board, attacker, tables, remaining, screen, work)
}

/// Pass 1's pipeline depth: the moves per key / prefetch / probe chunk (plan
/// 33, NULL-PIPE).
///
/// It has **no correctness effect**: the pipeline changes when a proof is
/// read, never which index wins (proof reads are pure and pass 1 stores
/// nothing), so any `K >= 1` gives the reference answer. Tuning only.
const PASS1_CHUNK: usize = 16;

/// [`null_win`] with an explicit pass-1 chunk size `K` (tests pin `K = 1`,
/// the unchunked order, and the shipped value).
fn null_win_pipelined<const K: usize>(
    board: &Board,
    attacker: Color,
    tables: &mut Tables,
    remaining: u16,
    screen: bool,
    work: &mut NullWork,
) -> Option<(Board, NullWin)> {
    if board.side_to_move() == attacker {
        return None;
    }
    let null = null_board(board, attacker);
    let mut moves = MoveList::new();
    generate_legal(&null, &mut moves);
    work.moves = u64::from(moves.len() as u32);
    // One scratch board for both passes: make, inspect, unmake. The cached
    // attack fields are not restored by `undo_move`, but every query below
    // populates its own `StateInfo`, so no stale cache is read.
    let mut scratch = null.clone();
    let mut state = StateInfo::new();
    // Pass 1: a direct win or a stored proof (shorter proofs first).
    if let Some(win) = pass1::<K>(
        &mut scratch,
        &mut state,
        moves.as_slice(),
        attacker,
        tables,
        remaining,
    ) {
        return Some((null, win));
    }
    // NULL-S: pass 2 only when the null position has an immediate explosion
    // threat. Pass 1 is untouched; the screen is a heuristic (tree growth is
    // reported, not gated).
    if screen && threat::signature_for(&null, attacker).is_empty() {
        return None;
    }
    // Pass 2: every defence exposes the commoner (a mate in two).
    let cache = tables.null_cache_mut();
    for &m in moves.as_slice() {
        scratch.do_move(m, &mut state);
        if let Some(limit) = exposed_limit(&scratch, attacker, work, cache) {
            return Some((
                null,
                NullWin {
                    mv: m,
                    length: 2,
                    limit,
                },
            ));
        }
        scratch.undo_move(m, &state);
    }
    None
}

/// Pass 1 of [`null_win_pipelined`]: the least index whose child is `Won` or
/// holds a stored proof of length `<= remaining`.
///
/// Plan 33 (NULL-PIPE): the moves come in chunks of `K`. Each chunk is keyed
/// first (make → status → key/counter → unmake), then its proof runs are
/// prefetched, then the keys are probed in move order. The order of decisions
/// and every answer are the ones the byte-serial scan produced — only the
/// issue time of the reads moves (E2) — and `work` is untouched here (E1):
/// `work.moves` is the full legal count, set before pass 1. Work past the
/// firing index (the make/unmake pairs of the rest of the chunk, the
/// prefetches) touches no counter, table, `line` or sink (E3), and the scratch
/// board is always unmade.
fn pass1<const K: usize>(
    scratch: &mut Board,
    state: &mut StateInfo,
    moves: &[Move],
    attacker: Color,
    tables: &Tables,
    remaining: u16,
) -> Option<NullWin> {
    // `(key, rule50)` of each move of the chunk; only `..scanned` is read.
    let mut keys = [(0u64, 0u16); K];
    for chunk in moves.chunks(K) {
        // (A) key the chunk, stopping at the first immediate win.
        let mut won = None;
        for (i, &m) in chunk.iter().enumerate() {
            scratch.do_move(m, state);
            let is_won = rules::status(scratch, attacker) == rules::Status::Won;
            keys[i] = (rules::key(scratch), scratch.rule50());
            scratch.undo_move(m, state);
            if is_won {
                won = Some(i);
                break;
            }
        }
        let scanned = won.unwrap_or(chunk.len());
        // (B) prefetch the chunk's runs, then probe in move order. A move
        // that is `Won` is not probed: `Won` wins at its index.
        for &(key, _) in &keys[..scanned] {
            tables.prefetch_proved(key);
        }
        for i in 0..scanned {
            let (key, counter) = keys[i];
            // G1: this site only accepts a proof, so the TT probe is skipped.
            if let Some(pe) = tables.proved(key, counter)
                && pe.length <= remaining
            {
                let limit = if counter == 0 {
                    99
                } else {
                    pe.limit.saturating_sub(1).min(99)
                };
                return Some(NullWin {
                    mv: chunk[i],
                    length: pe.length.saturating_add(1),
                    limit,
                });
            }
        }
        if let Some(i) = won {
            return Some(NullWin {
                mv: chunk[i],
                length: 1,
                limit: 99,
            });
        }
    }
    None
}

/// If every legal defence from `child` (defender to move) lets `attacker`
/// explode the defender's commoner at once, return the proof limit.
///
/// Plan 25 NULL-V1: the reply test is the board-free closure (defender
/// self-explosion, material draw, or [`threat::immediate_explosion_bits`])
/// instead of `rules::status` + `immediate_explosion`, and one scratch board
/// is reused for every reply.
fn exposed_limit(
    child: &Board,
    attacker: Color,
    work: &mut NullWork,
    cache: &mut NullCache,
) -> Option<u8> {
    if child.rule50() >= 100 {
        return None;
    }
    let key = rules::key(child);
    let rule50 = child.rule50() as u8;
    if let Some(cached) = cache.get(key, rule50, attacker) {
        return match cached {
            CachedLimit::Negative => None,
            CachedLimit::Positive(limit) => Some(limit),
        };
    }
    let result = exposed_limit_uncached(child, attacker, work);
    cache.insert(key, rule50, attacker, result);
    result
}

/// The uncached body of [`exposed_limit`] (plan-26 EARLY-REPLY, plan-31 F3).
///
/// The replies come from the **legal stream**
/// ([`for_each_legal_with_state`]) instead of a materialised pseudo-legal
/// list: the first reply that is not accepted ends the scan with
/// [`ControlFlow::Break`], and no list is built. The stream visits exactly
/// what `generate_legal` collects, in the same order, so the result and the
/// reply count are the ones the list path computed; `limit` is a max over the
/// accepted replies, i.e. order-free anyway.
fn exposed_limit_uncached(child: &Board, attacker: Color, work: &mut NullWork) -> Option<u8> {
    // Two populated states: one for the child's own counts (the stream reads
    // `commoners_count`), one reused by the scratch make/unmake. Sharing them
    // diverged in the plan-26 probe.
    let mut child_state = StateInfo::new();
    child.populate_state(&mut child_state);
    let defender = attacker.flip();
    let mut limit = 99u8;
    let mut scratch = child.clone();
    let mut state = StateInfo::new();
    let mut any_legal = false;
    let mut rejected = false;
    let _ = for_each_legal_with_state(child, &child_state, &mut |reply| {
        any_legal = true;
        work.replies += 1;
        scratch.do_move(reply, &mut state);
        if scratch.rule50() >= 100 {
            rejected = true;
            return ControlFlow::Break(());
        }
        // The closure must agree with `rules::status` + `immediate_explosion`:
        // a defender self-explosion is `Won`; a missing attacker commoner, a
        // material draw or a failed explosion is `Failed`/`Open` without a
        // win.
        let accepted = if scratch.commoners(defender).is_empty() {
            true
        } else if scratch.commoners(attacker).is_empty() || rules::material_draw(&scratch) {
            false
        } else {
            threat::has_explosion(&scratch, attacker)
        };
        if !accepted {
            rejected = true;
            return ControlFlow::Break(());
        }
        if scratch.rule50() == 0 {
            limit = 98;
        }
        scratch.undo_move(reply, &state);
        ControlFlow::Continue(())
    });
    if rejected || !any_legal {
        return None;
    }
    Some(limit)
}

#[cfg(test)]
mod tests;
