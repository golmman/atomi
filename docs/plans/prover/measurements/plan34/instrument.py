#!/usr/bin/env python3
"""Plan-34 item 42 — build the scratch instrumented census binary.

Copies this repo's source tree to a directory **outside** the repo, adds
`src/probe.rs` (rdtsc interval counters, one leaf per measured operation, an
inclusive aggregate per guide site) and wraps the call sites with `timed!`.
Nothing here is ever built inside the repo; the engine is unchanged
(`docs/plans/prover/measurements/plan34/env.json` names the scratch path and the binary sha).

The leaf ids partition the run: one rdtsc pair per measured call, no leaf
nested inside another. The aggregate ids (`A_*`) are inclusive and are *not*
part of the partition — they exist so a site (frame entry, null pass 1/2, the
replay, the playout) can be named with its own total.

    python3 docs/plans/prover/measurements/plan34/instrument.py --dest /tmp/atomi-plan34-probe
    (cd /tmp/atomi-plan34-probe && cargo build --release)

Then `census.py` runs it and parses the `{"probe": …}` line it prints on
stderr.
"""

import argparse
import pathlib
import shutil
import sys

PROBE_RS = r'''//! Scratch-only rdtsc census instrumentation (prover plan 34, item 42).
//!
//! Written into a copy of the tree outside the repo by
//! `docs/plans/prover/measurements/plan34/instrument.py`; **never** part of atomi. One rdtsc
//! pair per measured call, accumulated per (leaf, site). The leaves partition
//! the run; the `A_*` aggregates are inclusive and excluded from the sums.

use std::cell::UnsafeCell;

// Leaves (disjoint intervals, in bucket order).
pub const L_MG_LEGAL: usize = 0;
pub const L_MG_PSEUDO: usize = 1;
pub const L_STATE: usize = 2;
pub const L_STATUS: usize = 3;
pub const L_LEGALITY: usize = 4;
pub const L_DO: usize = 5;
pub const L_UNDO: usize = 6;
pub const L_CLONE: usize = 7;
pub const L_KEY: usize = 8;
pub const L_MOVER_NEW: usize = 9;
pub const L_MOVER_CHECK: usize = 10;
pub const L_TT: usize = 11;
pub const L_PROOF: usize = 12;
pub const L_FILTER: usize = 13;
pub const L_PREFETCH: usize = 14;
pub const L_CLOSURE_FRAME: usize = 15;
pub const L_CLOSURE_NULLP2: usize = 16;
pub const L_CLOSURE_REPLAY: usize = 17;
pub const L_RETEST: usize = 18;
pub const L_GUIDE_CLASS: usize = 19;
pub const L_EVAL: usize = 20;
pub const L_STORE: usize = 21;
pub const L_SELECT: usize = 22;
pub const L_CACHE: usize = 23;
pub const L_NULLBOARD: usize = 24;
pub const L_ALLOC: usize = 25;
pub const L_DEALLOC: usize = 26;
/// The empty interval: an rdtsc pair with no body, sampled in situ at each
/// site — the instrumentation's own tax on one interval.
pub const L_TAX: usize = 27;
// Inclusive aggregates (NOT part of the partition).
pub const A_ENTER: usize = 28;
pub const A_NULLWIN: usize = 29;
pub const A_NULLP2: usize = 30;
pub const A_REPLAY: usize = 31;
pub const A_SIM: usize = 32;
pub const A_SIBLINGS: usize = 33;
pub const A_NULLREPLAY: usize = 34;
pub const A_P1: usize = 35;
pub const A_P3: usize = 36;
pub const A_START: usize = 37;
pub const A_NLOOP: usize = 38;
pub const A_FIRST: usize = 28;
pub const NLEAF: usize = 39;

pub const NSITE: usize = 8;
pub const S_OTHER: u8 = 0;
pub const S_ENTER: u8 = 1;
pub const S_MID: u8 = 2;
pub const S_NULLP1: u8 = 3;
pub const S_NULLP2: u8 = 4;
pub const S_REPLAY: u8 = 5;
pub const S_SIM: u8 = 6;
pub const S_THREAT: u8 = 7;

pub const SITE_NAMES: [&str; NSITE] = [
    "other", "enter", "mid", "null_p1", "null_p2", "replay", "sim", "threat",
];

pub const LEAF_NAMES: [&str; NLEAF] = [
    "mg_legal",
    "mg_pseudo",
    "state",
    "status",
    "legality",
    "do_move",
    "undo_move",
    "clone",
    "key",
    "mover_new",
    "mover_check",
    "tt_probe",
    "proof_probe",
    "filter",
    "prefetch",
    "closure_frame",
    "closure_null_p2",
    "closure_replay",
    "repetition",
    "guide_class",
    "guide_eval",
    "store",
    "select",
    "null_cache",
    "null_board",
    "alloc",
    "dealloc",
    "tax",
    "A_enter",
    "A_null_win",
    "A_null_p2",
    "A_replay",
    "A_sim",
    "A_siblings",
    "A_null_replay",
    "A_pass1_loop",
    "A_pass3",
    "A_start_numbers",
    "A_null_pass2_loop",
];

// Counts (per run; the driver divides by expansions).
pub const NCNT: usize = 17;
pub const C_MOVES: usize = 0;
pub const C_LISTS: usize = 1;
pub const C_PSEUDO_LISTS: usize = 2;
pub const C_PSEUDO_MOVES: usize = 3;
pub const C_CHILDREN_BUILT: usize = 4;
pub const C_TT_PROBES: usize = 5;
pub const C_PROOF_PROBES: usize = 6;
pub const C_FILTER_PROBES: usize = 7;
pub const C_NULL_CALLS: usize = 8;
pub const C_NULL_MOVES: usize = 9;
pub const C_EVALS: usize = 10;
pub const C_REPLAY_CALLS: usize = 11;
pub const C_STORES: usize = 12;
pub const C_PREFETCHES: usize = 13;
pub const C_ALLOCS: usize = 14;
pub const C_DEALLOCS: usize = 15;
pub const C_SAMPLED_EXPS: usize = 16;

pub const CNT_NAMES: [&str; NCNT] = [
    "moves_legal",
    "lists_legal",
    "lists_pseudo",
    "moves_pseudo",
    "children_built",
    "tt_probes",
    "proof_probes",
    "filter_probes",
    "null_calls",
    "null_moves",
    "hand_evals",
    "replay_calls",
    "stores",
    "prefetches",
    "allocs",
    "deallocs",
    "sampled_expansions",
];

/// A `Sync` cell for the single search thread (the census never runs two).
pub struct Raw<T>(UnsafeCell<T>);
unsafe impl<T> Sync for Raw<T> {}
impl<T> Raw<T> {
    /// The raw pointer to the cell.
    #[inline(always)]
    pub fn ptr(&self) -> *mut T {
        self.0.get()
    }
}

static TICS: Raw<[[u64; NSITE]; NLEAF]> = Raw(UnsafeCell::new([[0; NSITE]; NLEAF]));
static IVS: Raw<[[u64; NSITE]; NLEAF]> = Raw(UnsafeCell::new([[0; NSITE]; NLEAF]));
static SITE: Raw<u8> = Raw(UnsafeCell::new(S_OTHER));
static CNT: Raw<[u64; NCNT]> = Raw(UnsafeCell::new([0; NCNT]));
static ON: Raw<bool> = Raw(UnsafeCell::new(false));
static ON_AGG: Raw<bool> = Raw(UnsafeCell::new(false));

/// The sampling stride: one expansion in `STRIDE` is timed (patched by
/// `instrument.py --stride`).
pub const STRIDE: u64 = __STRIDE__;

/// Whether the leaf buckets are timed at all (`--leaves`, the aggregates-only
/// ground-truth build).
pub const LEAVES: bool = __LEAVES__;

/// Which leaves are timed (`--only`; all of them by default). The aggregate
/// ids are always timed.
pub const ON_LEAF: [bool; NLEAF] = __ON_LEAF__;

/// Start the aggregate timers (the whole search window).
pub fn census_on() {
    // SAFETY: single-threaded; see `Raw`.
    unsafe { *ON_AGG.ptr() = true }
}

/// Whether `leaf` is being timed right now: aggregates always, a leaf only in
/// a sampled expansion.
#[inline(always)]
pub fn on(leaf: usize) -> bool {
    // SAFETY: single-threaded; see `Raw`.
    unsafe {
        if leaf >= A_FIRST {
            *ON_AGG.ptr()
        } else {
            LEAVES && ON_LEAF[leaf] && *ON.ptr()
        }
    }
}

/// Whether leaf buckets are timed in this expansion.
#[inline(always)]
pub fn leaves_on() -> bool {
    LEAVES && unsafe { *ON.ptr() }
}

/// Set (and restore) the sampling flag for one expansion (RAII, so a nested
/// `mid` restores its parent's decision on return).
pub struct Sampled(bool);

impl Sampled {
    /// Enter with flag `on`.
    #[inline(always)]
    pub fn new(on: bool) -> Sampled {
        // SAFETY: single-threaded; see `Raw`.
        unsafe {
            let p = ON.ptr();
            let old = *p;
            *p = on;
            Sampled(old)
        }
    }
}

impl Drop for Sampled {
    fn drop(&mut self) {
        // SAFETY: single-threaded; see `Raw`.
        unsafe { *ON.ptr() = self.0 }
    }
}

/// One serialising-free timestamp (the cost is given back by the driver).
#[inline(always)]
pub fn tsc() -> u64 {
    // SAFETY: `_rdtsc` is available on every x86-64 target this runs on.
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Accumulate one interval of `leaf` at the current site.
#[inline(always)]
pub fn add(leaf: usize, start: u64) {
    // SAFETY: single-threaded (one search thread); see `Raw`.
    unsafe {
        let d = tsc().wrapping_sub(start);
        let s = *SITE.ptr() as usize;
        let i = leaf * NSITE + s;
        let p = TICS.ptr().cast::<u64>();
        *p.add(i) = (*p.add(i)).wrapping_add(d);
        let q = IVS.ptr().cast::<u64>();
        *q.add(i) = (*q.add(i)).wrapping_add(1);
    }
}

/// Add to count `idx`.
#[inline(always)]
pub fn count(idx: usize, n: u64) {
    // SAFETY: single-threaded; see `Raw`.
    unsafe {
        let p = CNT.ptr().cast::<u64>().add(idx);
        *p = (*p).wrapping_add(n);
    }
}

/// Time `$body` as one interval of leaf `$leaf` (no early returns inside),
/// only while the current expansion is being sampled.
#[macro_export]
macro_rules! timed {
    ($leaf:expr, $body:expr) => {{
        let __t = if $crate::probe::on($leaf) {
            Some($crate::probe::tsc())
        } else {
            None
        };
        let __v = $body;
        if let Some(__t) = __t {
            $crate::probe::add($leaf, __t);
        }
        __v
    }};
}

/// One empty interval (an rdtsc pair with no body) at the current site: the
/// instrumentation tax on one measured interval.
#[inline(always)]
pub fn tax() {
    if on(L_TAX) {
        let t = tsc();
        add(L_TAX, t);
    }
}

/// Set the site id for the guard's lifetime (RAII: every early return
/// restores it).
pub struct At(u8);

impl At {
    /// Enter site `s`.
    #[inline(always)]
    pub fn new(s: u8) -> At {
        // SAFETY: single-threaded; see `Raw`.
        unsafe {
            let p = SITE.ptr();
            let old = *p;
            *p = s;
            At(old)
        }
    }
}

impl Drop for At {
    fn drop(&mut self) {
        // SAFETY: single-threaded; see `Raw`.
        unsafe { *SITE.ptr() = self.0 }
    }
}

/// The counting allocator: the `alloc`/`dealloc` buckets.
struct CountAlloc;

// SAFETY: every method forwards to `System`, which is a correct allocator.
unsafe impl std::alloc::GlobalAlloc for CountAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        if on(L_ALLOC) {
            let t = tsc();
            // SAFETY: forwarded unchanged; `layout` is the caller's.
            let p = unsafe { std::alloc::System.alloc(layout) };
            count(C_ALLOCS, 1);
            add(L_ALLOC, t);
            return p;
        }
        unsafe { std::alloc::System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        if on(L_DEALLOC) {
            let t = tsc();
            unsafe { std::alloc::System.dealloc(ptr, layout) };
            count(C_DEALLOCS, 1);
            add(L_DEALLOC, t);
            return;
        }
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: std::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        if on(L_ALLOC) {
            let t = tsc();
            let p = unsafe { std::alloc::System.realloc(ptr, layout, new_size) };
            count(C_ALLOCS, 1);
            add(L_ALLOC, t);
            return p;
        }
        unsafe { std::alloc::System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static COUNT_ALLOC: CountAlloc = CountAlloc;

/// The census as one JSON object (`{"probe": …}`), tics per (leaf, site).
pub fn dump() -> String {
    let mut leaves: Vec<String> = Vec::new();
    let mut aggs: Vec<String> = Vec::new();
    let mut total_ivs = 0u64;
    // SAFETY: read after the search thread joined (single-threaded here).
    unsafe {
        let tics = &*TICS.ptr();
        let ivs = &*IVS.ptr();
        for leaf in 0..NLEAF {
            for site in 0..NSITE {
                let t = tics[leaf][site];
                if t == 0 {
                    continue;
                }
                total_ivs += ivs[leaf][site];
                let row = format!(
                    "\"{}@{}\":{{\"tics\":{},\"intervals\":{}}}",
                    LEAF_NAMES[leaf], SITE_NAMES[site], t, ivs[leaf][site]
                );
                if leaf < A_FIRST {
                    leaves.push(row);
                } else {
                    aggs.push(row);
                }
            }
        }
    }
    let mut counts: Vec<String> = Vec::new();
    // SAFETY: as above.
    unsafe {
        let c = &*CNT.ptr();
        for i in 0..NCNT {
            counts.push(format!("\"{}\":{}", CNT_NAMES[i], c[i]));
        }
    }
    format!(
        "{{\"probe\":{{\"leaves\":{{{}}},\"aggregates\":{{{}}},\"counts\":{{{}}},\
         \"total_intervals\":{}}}}}",
        leaves.join(","),
        aggs.join(","),
        counts.join(","),
        total_ivs
    )
}
'''

# (path, old, new) — every `old` must occur exactly once.
PATCHES = [
    (
        "src/lib.rs",
        "pub mod mover;\n",
        "pub mod mover;\npub mod probe;\n",
    ),
    (
        "src/main.rs",
        "        stats,\n    );\n    if opts.proof_out.is_some() && found && !proof_written {",
        "        stats,\n    );\n    eprintln!(\"{}\", atomi::probe::dump());\n"
        "    if opts.proof_out.is_some() && found && !proof_written {",
    ),
    # --- rules: status / status_with / key / child_key -----------------------
    (
        "src/rules.rs",
        "pub fn status(board: &Board, attacker: Color) -> Status {\n"
        "    let from = if board.rule50() >= 100 { 100 } else { 0 };\n"
        "    let stm = board.side_to_move();\n"
        "    match board.outcome() {",
        "pub fn status(board: &Board, attacker: Color) -> Status {\n"
        "    crate::timed!(crate::probe::L_STATUS, status_inner(board, attacker))\n"
        "}\n\n"
        "fn status_inner(board: &Board, attacker: Color) -> Status {\n"
        "    let from = if board.rule50() >= 100 { 100 } else { 0 };\n"
        "    let stm = board.side_to_move();\n"
        "    match board.outcome() {",
    ),
    (
        "src/rules.rs",
        "pub fn status_with(board: &Board, attacker: Color, state: &StateInfo, moves_exist: bool) -> Status {\n"
        "    let from = if board.rule50() >= 100 { 100 } else { 0 };",
        "pub fn status_with(board: &Board, attacker: Color, state: &StateInfo, moves_exist: bool) -> Status {\n"
        "    crate::timed!(\n"
        "        crate::probe::L_STATUS,\n"
        "        status_with_inner(board, attacker, state, moves_exist)\n"
        "    )\n"
        "}\n\n"
        "fn status_with_inner(\n"
        "    board: &Board,\n"
        "    attacker: Color,\n"
        "    state: &StateInfo,\n"
        "    moves_exist: bool,\n"
        ") -> Status {\n"
        "    let from = if board.rule50() >= 100 { 100 } else { 0 };",
    ),
    (
        "src/rules.rs",
        "pub fn key(board: &Board) -> u64 {\n    board.hash_with(EnPassantMode::Legal)\n}",
        "pub fn key(board: &Board) -> u64 {\n"
        "    crate::timed!(crate::probe::L_KEY, board.hash_with(EnPassantMode::Legal))\n"
        "}",
    ),
    # --- mover: the hash/counter read and the needs-board test ---------------
    (
        "src/mover.rs",
        "        let (h, counter) = board.hash_and_rule50_after(m);\n"
        "        let double_push = board.is_double_push(m);\n"
        "        let needs_board = self.stalemate_plausible\n"
        "            || m.is_promotion()\n"
        "            || m.is_en_passant()\n"
        "            || m.is_castling()\n"
        "            || board.is_capture(m)\n"
        "            || board.piece_on(m.from_sq()).type_of() == Some(PieceType::Commoner)\n"
        "            || self.gives_check(board, m)\n"
        "            || (double_push && self.enemy_pawn_attacks_passed(m));",
        "        let (h, counter) =\n"
        "            crate::timed!(crate::probe::L_KEY, board.hash_and_rule50_after(m));\n"
        "        let (double_push, needs_board) = crate::timed!(crate::probe::L_MOVER_CHECK, {\n"
        "            let double_push = board.is_double_push(m);\n"
        "            let needs_board = self.stalemate_plausible\n"
        "                || m.is_promotion()\n"
        "                || m.is_en_passant()\n"
        "                || m.is_castling()\n"
        "                || board.is_capture(m)\n"
        "                || board.piece_on(m.from_sq()).type_of() == Some(PieceType::Commoner)\n"
        "                || self.gives_check(board, m)\n"
        "                || (double_push && self.enemy_pawn_attacks_passed(m));\n"
        "            (double_push, needs_board)\n"
        "        });",
    ),
    # --- frame: the enter site, pass 0, pass 1, pass 2, pass 3 ---------------
    (
        "src/search/frame.rs",
        "    ) -> Entered {\n        debug_assert_eq!(key, *line.last().unwrap());",
        "    ) -> Entered {\n"
        "        let _at = crate::probe::At::new(crate::probe::S_ENTER);\n"
        "        debug_assert_eq!(key, *line.last().unwrap());",
    ),
    (
        "src/search/frame.rs",
        "        let mut state = StateInfo::new();\n"
        "        board.populate_state(&mut state);\n"
        "        let mut moves = MoveList::new();\n"
        "        generate_legal_with_state(board, &state, &mut moves);\n"
        "        match rules::status_with(board, attacker, &state, !moves.is_empty()) {",
        "        let mut state = StateInfo::new();\n"
        "        crate::timed!(crate::probe::L_STATE, board.populate_state(&mut state));\n"
        "        let mut moves = MoveList::new();\n"
        "        crate::timed!(\n"
        "            crate::probe::L_MG_LEGAL,\n"
        "            generate_legal_with_state(board, &state, &mut moves)\n"
        "        );\n"
        "        crate::probe::count(crate::probe::C_LISTS, 1);\n"
        "        crate::probe::count(crate::probe::C_MOVES, moves.len() as u64);\n"
        "        match rules::status_with(board, attacker, &state, !moves.is_empty()) {",
    ),
    (
        "src/search/frame.rs",
        "        let ctx = guide.context(\n"
        "            board,\n"
        "            &state,\n"
        "            u32::try_from(moves.len()).unwrap_or(u32::MAX),\n"
        "            attacker_node,\n"
        "            !attacker_node,\n"
        "        );\n"
        "        let mover_ctx = Mover::new(board);",
        "        let ctx = crate::timed!(\n"
        "            crate::probe::L_GUIDE_CLASS,\n"
        "            guide.context(\n"
        "                board,\n"
        "                &state,\n"
        "                u32::try_from(moves.len()).unwrap_or(u32::MAX),\n"
        "                attacker_node,\n"
        "                !attacker_node,\n"
        "            )\n"
        "        );\n"
        "        let mover_ctx = crate::timed!(crate::probe::L_MOVER_NEW, Mover::new(board));",
    ),
    (
        "src/search/frame.rs",
        "            let (child_key, mut value) = if e.needs_board {\n"
        "                let mut child = board.clone();\n"
        "                child.do_move(m, &mut StateInfo::new());\n"
        "                let value = match rules::status(&child, attacker) {",
        "            let (child_key, mut value) = if e.needs_board {\n"
        "                crate::probe::count(crate::probe::C_CHILDREN_BUILT, 1);\n"
        "                let mut child = crate::timed!(crate::probe::L_CLONE, board.clone());\n"
        "                crate::timed!(crate::probe::L_DO, child.do_move(m, &mut StateInfo::new()));\n"
        "                let value = match rules::status(&child, attacker) {",
    ),
    (
        "src/search/frame.rs",
        "                let k = child.hash_with(EnPassantMode::Legal);",
        "                let k =\n"
        "                    crate::timed!(crate::probe::L_KEY, child.hash_with(EnPassantMode::Legal));",
    ),
    (
        "src/search/frame.rs",
        "                && (built.is_some()\n"
        "                    || if closure_bits {\n"
        "                        threat::settle_screen_bits(board, m, mover)\n"
        "                    } else {\n"
        "                        threat::settle_screen(board, m, mover)\n"
        "                    });",
        "                && (built.is_some()\n"
        "                    || crate::timed!(crate::probe::L_CLOSURE_FRAME, if closure_bits {\n"
        "                        threat::settle_screen_bits(board, m, mover)\n"
        "                    } else {\n"
        "                        threat::settle_screen(board, m, mover)\n"
        "                    }));",
    ),
    (
        "src/search/frame.rs",
        "                if built.is_none() {\n"
        "                    let mut child = board.clone();\n"
        "                    child.do_move(m, &mut StateInfo::new());\n"
        "                    settle_nodes += 1;\n"
        "                    built = Some(child);\n"
        "                }\n"
        "                let child = built.as_ref().expect(\"child built above\");\n"
        "                let settled = if closure_bits {\n"
        "                    threat::one_ply_settle_bits(child, attacker)\n"
        "                } else {\n"
        "                    settle_nodes += 1;\n"
        "                    threat::one_ply_settle(child, attacker)\n"
        "                };",
        "                if built.is_none() {\n"
        "                    crate::probe::count(crate::probe::C_CHILDREN_BUILT, 1);\n"
        "                    let mut child = crate::timed!(crate::probe::L_CLONE, board.clone());\n"
        "                    crate::timed!(\n"
        "                        crate::probe::L_DO,\n"
        "                        child.do_move(m, &mut StateInfo::new())\n"
        "                    );\n"
        "                    settle_nodes += 1;\n"
        "                    built = Some(child);\n"
        "                }\n"
        "                let child = built.as_ref().expect(\"child built above\");\n"
        "                let settled = crate::timed!(crate::probe::L_CLOSURE_FRAME, if closure_bits {\n"
        "                    threat::one_ply_settle_bits(child, attacker)\n"
        "                } else {\n"
        "                    settle_nodes += 1;\n"
        "                    threat::one_ply_settle(child, attacker)\n"
        "                });",
    ),
    (
        "src/search/frame.rs",
        "            let (class, or_number) = if guide_on {\n"
        "                guide::class::classify_exact(\n"
        "                    &exact,\n"
        "                    &mover_ctx,\n"
        "                    board,\n"
        "                    m,\n"
        "                    &e,\n"
        "                    built.as_ref(),\n"
        "                    ctx.in_check && guide.config.check_evasions,\n"
        "                )\n"
        "            } else {\n"
        "                (MoveClass::Quiet, None)\n"
        "            };",
        "            let (class, or_number) = crate::timed!(crate::probe::L_GUIDE_CLASS, if guide_on {\n"
        "                guide::class::classify_exact(\n"
        "                    &exact,\n"
        "                    &mover_ctx,\n"
        "                    board,\n"
        "                    m,\n"
        "                    &e,\n"
        "                    built.as_ref(),\n"
        "                    ctx.in_check && guide.config.check_evasions,\n"
        "                )\n"
        "            } else {\n"
        "                (MoveClass::Quiet, None)\n"
        "            });",
    ),
    (
        "src/search/frame.rs",
        "            let and_number = if guide_on\n"
        "                && !attacker_node\n"
        "                && !guide::class::defence_likely_to_refute(board, m, built.as_ref(), attacker)\n"
        "            {",
        "            let and_number = if guide_on\n"
        "                && !attacker_node\n"
        "                && !crate::timed!(\n"
        "                    crate::probe::L_GUIDE_CLASS,\n"
        "                    guide::class::defence_likely_to_refute(board, m, built.as_ref(), attacker)\n"
        "                )\n"
        "            {",
    ),
    (
        "src/search/frame.rs",
        "                } else if let Some(child) = child_ref {\n"
        "                    let e = guide::eval::hand_eval(child);",
        "                } else if let Some(child) = child_ref {\n"
        "                    crate::probe::count(crate::probe::C_EVALS, 1);\n"
        "                    let e = crate::timed!(crate::probe::L_EVAL, guide::eval::hand_eval(child));",
    ),
    (
        "src/search/frame.rs",
        "            if let Some(j) = line.iter().position(|&k| k == keys[i]) {",
        "            if let Some(j) = crate::timed!(\n"
        "                crate::probe::L_RETEST,\n"
        "                line.iter().position(|&k| k == keys[i])\n"
        "            ) {",
    ),
    (
        "src/search/frame.rs",
        "                if fresh[i] {\n"
        "                    let (pn, dn) =\n"
        "                        guide.start_numbers(&ctx, &inputs[i], attacker_node, mover, c.mv, history);",
        "                if fresh[i] {\n"
        "                    let (pn, dn) = crate::timed!(\n"
        "                        crate::probe::L_GUIDE_CLASS,\n"
        "                        guide.start_numbers(&ctx, &inputs[i], attacker_node, mover, c.mv, history)\n"
        "                    );",
    ),
    # --- the in-situ tax samples (one empty interval per call, per site) ----
    (
        "src/search/frame.rs",
        "        let _at = crate::probe::At::new(crate::probe::S_ENTER);\n"
        "        debug_assert_eq!(key, *line.last().unwrap());",
        "        let _at = crate::probe::At::new(crate::probe::S_ENTER);\n"
        "        crate::probe::tax();\n"
        "        debug_assert_eq!(key, *line.last().unwrap());",
    ),
    # --- search driver ------------------------------------------------------
    (
        "src/search/mod.rs",
        "            let start = Instant::now();",
        "            crate::probe::census_on();\n            let start = Instant::now();",
    ),
    (
        "src/search/mod.rs",
        "    fn mid(&mut self, board: &Board, key: u64, ply: u16, tpn: u32, tdn: u32) -> Value {\n"
        "        if self.stop.is_some() {",
        "    fn mid(&mut self, board: &Board, key: u64, ply: u16, tpn: u32, tdn: u32) -> Value {\n"
        "        let _at = crate::probe::At::new(crate::probe::S_MID);\n"
        "        let _sm = crate::probe::Sampled::new(\n"
        "            self.expansions % crate::probe::STRIDE == 0,\n"
        "        );\n"
        "        if crate::probe::leaves_on() {\n"
        "            crate::probe::count(crate::probe::C_SAMPLED_EXPS, 1);\n"
        "        }\n"
        "        if self.stop.is_some() {",
    ),
    (
        "src/search/mod.rs",
        "        let mut frame = match Frame::enter(",
        "        let mut frame = match crate::timed!(crate::probe::A_ENTER, Frame::enter(",
    ),
    (
        "src/search/mod.rs",
        "            self.seed,\n            aux_left,\n        ) {\n            Entered::Terminal(v) => {",
        "            self.seed,\n            aux_left,\n        )) {\n            Entered::Terminal(v) => {",
    ),
    (
        "src/search/mod.rs",
        "        if self.guide.config.enabled && self.guide.config.null_exact && !frame.attacker_node {\n"
        "            self.null_replay(&mut frame, board);\n"
        "        }",
        "        if self.guide.config.enabled && self.guide.config.null_exact && !frame.attacker_node {\n"
        "            crate::timed!(crate::probe::A_NULLREPLAY, self.null_replay(&mut frame, board));\n"
        "        }",
    ),
    (
        "src/search/mod.rs",
        "        loop {\n            let v = node_value(&frame);",
        "        loop {\n            let v = crate::timed!(crate::probe::L_SELECT, node_value(&frame));",
    ),
    (
        "src/search/mod.rs",
        "            let (best, second) = select(&frame, &self.history, self.seed);",
        "            let (best, second) =\n"
        "                crate::timed!(crate::probe::L_SELECT, select(&frame, &self.history, self.seed));",
    ),
    (
        "src/search/mod.rs",
        "            let (ctpn, ctdn) = child_thresholds(&frame, best, second, tpn, tdn, pn, dn);",
        "            let (ctpn, ctdn) = crate::timed!(\n"
        "                crate::probe::L_SELECT,\n"
        "                child_thresholds(&frame, best, second, tpn, tdn, pn, dn)\n"
        "            );",
    ),
    (
        "src/search/mod.rs",
        "            let mut next = board.clone();\n"
        "            next.do_move(frame.children[best].mv, &mut StateInfo::new());\n"
        "            let cv = self.mid(&next, frame.keys[best], ply + 1, ctpn, ctdn);",
        "            let mut next = crate::timed!(crate::probe::L_CLONE, board.clone());\n"
        "            crate::timed!(\n"
        "                crate::probe::L_DO,\n"
        "                next.do_move(frame.children[best].mv, &mut StateInfo::new())\n"
        "            );\n"
        "            let cv = self.mid(&next, frame.keys[best], ply + 1, ctpn, ctdn);",
    ),
    (
        "src/search/mod.rs",
        "                self.replay_siblings(&mut frame, best, board, &next, length);",
        "                crate::timed!(\n"
        "                    crate::probe::A_SIBLINGS,\n"
        "                    self.replay_siblings(&mut frame, best, board, &next, length)\n"
        "                );",
    ),
    (
        "src/search/mod.rs",
        "            let mut child = board.clone();\n"
        "            child.do_move(frame.children[j].mv, &mut StateInfo::new());\n"
        "            let mut budget = crate::guide::replay::REPLAY_BUDGET",
        "            crate::probe::count(crate::probe::C_CHILDREN_BUILT, 1);\n"
        "            let mut child = crate::timed!(crate::probe::L_CLONE, board.clone());\n"
        "            crate::timed!(\n"
        "                crate::probe::L_DO,\n"
        "                child.do_move(frame.children[j].mv, &mut StateInfo::new())\n"
        "            );\n"
        "            let mut budget = crate::guide::replay::REPLAY_BUDGET",
    ),
    # --- null: the site, the two passes, the null board ----------------------
    (
        "src/guide/null.rs",
        ") -> Option<(Board, NullWin)> {\n"
        "    null_win_pipelined::<PASS1_CHUNK>(board, attacker, tables, remaining, screen, work)\n}",
        ") -> Option<(Board, NullWin)> {\n"
        "    crate::probe::count(crate::probe::C_NULL_CALLS, 1);\n"
        "    crate::timed!(\n"
        "        crate::probe::A_NULLWIN,\n"
        "        null_win_pipelined::<PASS1_CHUNK>(board, attacker, tables, remaining, screen, work)\n"
        "    )\n"
        "}",
    ),
    (
        "src/guide/null.rs",
        "    let null = null_board(board, attacker);\n"
        "    let mut moves = MoveList::new();\n"
        "    generate_legal(&null, &mut moves);\n"
        "    work.moves = u64::from(moves.len() as u32);",
        "    let null = crate::timed!(crate::probe::L_NULLBOARD, null_board(board, attacker));\n"
        "    let mut moves = MoveList::new();\n"
        "    crate::timed!(crate::probe::L_MG_LEGAL, generate_legal(&null, &mut moves));\n"
        "    crate::probe::count(crate::probe::C_LISTS, 1);\n"
        "    crate::probe::count(crate::probe::C_MOVES, moves.len() as u64);\n"
        "    crate::probe::count(crate::probe::C_NULL_MOVES, moves.len() as u64);\n"
        "    work.moves = u64::from(moves.len() as u32);",
    ),
    (
        "src/guide/null.rs",
        "    let mut scratch = null.clone();\n"
        "    let mut state = StateInfo::new();\n"
        "    // Pass 1: a direct win or a stored proof (shorter proofs first).\n"
        "    if let Some(win) = pass1::<K>(\n"
        "        &mut scratch,\n"
        "        &mut state,\n"
        "        moves.as_slice(),\n"
        "        attacker,\n"
        "        tables,\n"
        "        remaining,\n"
        "    ) {\n"
        "        return Some((null, win));\n"
        "    }",
        "    let mut scratch = crate::timed!(crate::probe::L_CLONE, null.clone());\n"
        "    let mut state = StateInfo::new();\n"
        "    // Pass 1: a direct win or a stored proof (shorter proofs first).\n"
        "    let pass1_win = {\n"
        "        let _at = crate::probe::At::new(crate::probe::S_NULLP1);\n"
        "        pass1::<K>(\n"
        "            &mut scratch,\n"
        "            &mut state,\n"
        "            moves.as_slice(),\n"
        "            attacker,\n"
        "            tables,\n"
        "            remaining,\n"
        "        )\n"
        "    };\n"
        "    if let Some(win) = pass1_win {\n"
        "        return Some((null, win));\n"
        "    }",
    ),
    (
        "src/guide/null.rs",
        "    let cache = tables.null_cache_mut();\n"
        "    for &m in moves.as_slice() {\n"
        "        scratch.do_move(m, &mut state);",
        "    let _at2 = crate::probe::At::new(crate::probe::S_NULLP2);\n"
        "    let cache = tables.null_cache_mut();\n"
        "    for &m in moves.as_slice() {\n"
        "        crate::timed!(crate::probe::L_DO, scratch.do_move(m, &mut state));",
    ),
    (
        "src/guide/null.rs",
        "        scratch.undo_move(m, &state);\n    }\n    None\n}",
        "        crate::timed!(crate::probe::L_UNDO, scratch.undo_move(m, &state));\n    }\n    None\n}",
    ),
    (
        "src/guide/null.rs",
        "            scratch.do_move(m, state);\n"
        "            let is_won = rules::status(scratch, attacker) == rules::Status::Won;\n"
        "            keys[i] = (rules::key(scratch), scratch.rule50());\n"
        "            scratch.undo_move(m, state);",
        "            crate::timed!(crate::probe::L_DO, scratch.do_move(m, state));\n"
        "            let is_won = rules::status(scratch, attacker) == rules::Status::Won;\n"
        "            keys[i] = (rules::key(scratch), scratch.rule50());\n"
        "            crate::timed!(crate::probe::L_UNDO, scratch.undo_move(m, state));",
    ),
    (
        "src/guide/null.rs",
        "    if let Some(cached) = cache.get(key, rule50, attacker) {\n"
        "        return match cached {\n"
        "            CachedLimit::Negative => None,\n"
        "            CachedLimit::Positive(limit) => Some(limit),\n"
        "        };\n"
        "    }\n"
        "    let result = exposed_limit_uncached(child, attacker, work);\n"
        "    cache.insert(key, rule50, attacker, result);\n"
        "    result",
        "    if let Some(cached) = crate::timed!(\n"
        "        crate::probe::L_CACHE,\n"
        "        cache.get(key, rule50, attacker)\n"
        "    ) {\n"
        "        return match cached {\n"
        "            CachedLimit::Negative => None,\n"
        "            CachedLimit::Positive(limit) => Some(limit),\n"
        "        };\n"
        "    }\n"
        "    let result = crate::timed!(\n"
        "        crate::probe::A_NULLP2,\n"
        "        exposed_limit_uncached(child, attacker, work)\n"
        "    );\n"
        "    crate::timed!(\n"
        "        crate::probe::L_CACHE,\n"
        "        cache.insert(key, rule50, attacker, result)\n"
        "    );\n"
        "    result",
    ),
    (
        "src/guide/null.rs",
        "    let mut child_state = StateInfo::new();\n"
        "    child.populate_state(&mut child_state);\n"
        "    let defender = attacker.flip();\n"
        "    let mut limit = 99u8;\n"
        "    let mut scratch = child.clone();\n"
        "    let mut state = StateInfo::new();\n"
        "    let mut any_legal = false;\n"
        "    let mut rejected = false;\n"
        "    let _ = for_each_legal_with_state(child, &child_state, &mut |reply| {\n"
        "        any_legal = true;\n"
        "        work.replies += 1;\n"
        "        scratch.do_move(reply, &mut state);",
        "    let mut child_state = StateInfo::new();\n"
        "    crate::timed!(\n"
        "        crate::probe::L_STATE,\n"
        "        child.populate_state(&mut child_state)\n"
        "    );\n"
        "    let defender = attacker.flip();\n"
        "    let mut limit = 99u8;\n"
        "    let mut scratch = crate::timed!(crate::probe::L_CLONE, child.clone());\n"
        "    let mut state = StateInfo::new();\n"
        "    let mut any_legal = false;\n"
        "    let mut rejected = false;\n"
        "    let _ = crate::timed!(\n"
        "        crate::probe::L_MG_LEGAL,\n"
        "        for_each_legal_with_state(child, &child_state, &mut |reply| {\n"
        "            any_legal = true;\n"
        "            work.replies += 1;\n"
        "            crate::timed!(crate::probe::L_DO, scratch.do_move(reply, &mut state));",
    ),
    (
        "src/guide/null.rs",
        "        let accepted = if scratch.commoners(defender).is_empty() {\n"
        "            true\n"
        "        } else if scratch.commoners(attacker).is_empty() || rules::material_draw(&scratch) {\n"
        "            false\n"
        "        } else {\n"
        "            threat::has_explosion(&scratch, attacker)\n"
        "        };",
        "        let accepted = if scratch.commoners(defender).is_empty() {\n"
        "            true\n"
        "        } else if scratch.commoners(attacker).is_empty()\n"
        "            || crate::timed!(crate::probe::L_LEGALITY, rules::material_draw(&scratch))\n"
        "        {\n"
        "            false\n"
        "        } else {\n"
        "            crate::timed!(\n"
        "                crate::probe::L_CLOSURE_NULLP2,\n"
        "                threat::has_explosion(&scratch, attacker)\n"
        "            )\n"
        "        };",
    ),
    (
        "src/guide/null.rs",
        "        scratch.undo_move(reply, &state);\n"
        "        ControlFlow::Continue(())\n    });",
        "        crate::timed!(crate::probe::L_UNDO, scratch.undo_move(reply, &state));\n"
        "        ControlFlow::Continue(())\n"
        "        })\n"
        "    );",
    ),
    # --- replay: the site, and its leaves -----------------------------------
    (
        "src/guide/replay.rs",
        "fn simulate_inner(\n"
        "    pos: &Board,\n"
        "    guide: &Board,\n"
        "    guide_proof: Option<(Move, u16)>,\n"
        "    attacker: Color,\n"
        "    remaining: u16,\n"
        "    budget: &mut u32,\n"
        "    line: &mut Vec<u64>,\n"
        "    search_line: &[u64],\n"
        "    tables: &Tables,\n"
        ") -> Option<ReplayProof> {\n"
        "    match rules::status(pos, attacker) {",
        "fn simulate_inner(\n"
        "    pos: &Board,\n"
        "    guide: &Board,\n"
        "    guide_proof: Option<(Move, u16)>,\n"
        "    attacker: Color,\n"
        "    remaining: u16,\n"
        "    budget: &mut u32,\n"
        "    line: &mut Vec<u64>,\n"
        "    search_line: &[u64],\n"
        "    tables: &Tables,\n"
        ") -> Option<ReplayProof> {\n"
        "    let _at = crate::probe::At::new(crate::probe::S_REPLAY);\n"
        "    crate::probe::count(crate::probe::C_REPLAY_CALLS, 1);\n"
        "    crate::timed!(\n"
        "        crate::probe::A_REPLAY,\n"
        "        simulate_body(\n"
        "            pos,\n"
        "            guide,\n"
        "            guide_proof,\n"
        "            attacker,\n"
        "            remaining,\n"
        "            budget,\n"
        "            line,\n"
        "            search_line,\n"
        "            tables,\n"
        "        )\n"
        "    )\n"
        "}\n\n"
        "#[allow(clippy::too_many_arguments)]\n"
        "fn simulate_body(\n"
        "    pos: &Board,\n"
        "    guide: &Board,\n"
        "    guide_proof: Option<(Move, u16)>,\n"
        "    attacker: Color,\n"
        "    remaining: u16,\n"
        "    budget: &mut u32,\n"
        "    line: &mut Vec<u64>,\n"
        "    search_line: &[u64],\n"
        "    tables: &Tables,\n"
        ") -> Option<ReplayProof> {\n"
        "    match rules::status(pos, attacker) {",
    ),
    (
        "src/guide/replay.rs",
        "    if !pos.is_legal(gmv) {\n"
        "        return None;\n"
        "    }\n"
        "    let mut after = pos.clone();\n"
        "    after.do_move(gmv, &mut StateInfo::new());\n"
        "    let mut gafter = guide.clone();\n"
        "    gafter.do_move(gmv, &mut StateInfo::new());",
        "    if !crate::timed!(crate::probe::L_LEGALITY, pos.is_legal(gmv)) {\n"
        "        return None;\n"
        "    }\n"
        "    let mut after = crate::timed!(crate::probe::L_CLONE, pos.clone());\n"
        "    crate::timed!(crate::probe::L_DO, after.do_move(gmv, &mut StateInfo::new()));\n"
        "    let mut gafter = crate::timed!(crate::probe::L_CLONE, guide.clone());\n"
        "    crate::timed!(crate::probe::L_DO, gafter.do_move(gmv, &mut StateInfo::new()));",
    ),
    (
        "src/guide/replay.rs",
        "    let mut moves = MoveList::new();\n"
        "    generate_legal(pos, &mut moves);\n"
        "    if moves.is_empty() {",
        "    let mut moves = MoveList::new();\n"
        "    crate::timed!(crate::probe::L_MG_LEGAL, generate_legal(pos, &mut moves));\n"
        "    crate::probe::count(crate::probe::C_LISTS, 1);\n"
        "    crate::probe::count(crate::probe::C_MOVES, moves.len() as u64);\n"
        "    if moves.is_empty() {",
    ),
    (
        "src/guide/replay.rs",
        "    for &m in moves.as_slice() {\n"
        "        let mut next = pos.clone();\n"
        "        next.do_move(m, &mut StateInfo::new());",
        "    for &m in moves.as_slice() {\n"
        "        let mut next = crate::timed!(crate::probe::L_CLONE, pos.clone());\n"
        "        crate::timed!(crate::probe::L_DO, next.do_move(m, &mut StateInfo::new()));",
    ),
    (
        "src/guide/replay.rs",
        "                } else if next.rule50() < 100\n"
        "                    && let Some(explosion) = threat::immediate_explosion(&next, attacker)\n"
        "                {",
        "                } else if next.rule50() < 100\n"
        "                    && let Some(explosion) = crate::timed!(\n"
        "                        crate::probe::L_CLOSURE_REPLAY,\n"
        "                        threat::immediate_explosion(&next, attacker)\n"
        "                    )\n"
        "                {",
    ),
    (
        "src/guide/replay.rs",
        "                } else if guide.is_legal(m) {\n"
        "                    let mut gnext = guide.clone();\n"
        "                    gnext.do_move(m, &mut StateInfo::new());",
        "                } else if crate::timed!(crate::probe::L_LEGALITY, guide.is_legal(m)) {\n"
        "                    let mut gnext = crate::timed!(crate::probe::L_CLONE, guide.clone());\n"
        "                    crate::timed!(\n"
        "                        crate::probe::L_DO,\n"
        "                        gnext.do_move(m, &mut StateInfo::new())\n"
        "                    );",
    ),
    # --- simulate: the site and its leaves (off in the shipped stack) -------
    (
        "src/guide/simulate.rs",
        "pub fn playout_with(board: &Board, attacker: Color, plies: u8) -> (i32, u64) {\n"
        "    let mut board = board.clone();",
        "pub fn playout_with(board: &Board, attacker: Color, plies: u8) -> (i32, u64) {\n"
        "    let _at = crate::probe::At::new(crate::probe::S_SIM);\n"
        "    crate::timed!(crate::probe::A_SIM, playout_body(board, attacker, plies))\n"
        "}\n\n"
        "fn playout_body(board: &Board, attacker: Color, plies: u8) -> (i32, u64) {\n"
        "    let mut board = crate::timed!(crate::probe::L_CLONE, board.clone());",
    ),
    (
        "src/guide/simulate.rs",
        "        let mut moves = MoveList::new();\n"
        "        generate_legal(&board, &mut moves);\n"
        "        if moves.is_empty() {",
        "        let mut moves = MoveList::new();\n"
        "        crate::timed!(crate::probe::L_MG_LEGAL, generate_legal(&board, &mut moves));\n"
        "        crate::probe::count(crate::probe::C_LISTS, 1);\n"
        "        crate::probe::count(crate::probe::C_MOVES, moves.len() as u64);\n"
        "        if moves.is_empty() {",
    ),
    (
        "src/guide/simulate.rs",
        "        board.do_move(m, &mut StateInfo::new());\n        nodes += 1;",
        "        crate::timed!(crate::probe::L_DO, board.do_move(m, &mut StateInfo::new()));\n"
        "        nodes += 1;",
    ),
    # --- table: the probes, the filter, the store ---------------------------
    (
        "src/table/mod.rs",
        "    pub fn may_hold(&self, key: u64) -> bool {\n        self.filter.may_hold(key)\n    }",
        "    pub fn may_hold(&self, key: u64) -> bool {\n"
        "        crate::probe::count(crate::probe::C_FILTER_PROBES, 1);\n"
        "        crate::timed!(crate::probe::L_FILTER, self.filter.may_hold(key))\n"
        "    }",
    ),
    (
        "src/table/mod.rs",
        "    pub fn prefetch(&self, key: u64) {\n"
        "        self.tt.prefetch(key);\n"
        "        self.proofs.prefetch(key);\n"
        "    }",
        "    pub fn prefetch(&self, key: u64) {\n"
        "        crate::probe::count(crate::probe::C_PREFETCHES, 1);\n"
        "        crate::timed!(crate::probe::L_PREFETCH, {\n"
        "            self.tt.prefetch(key);\n"
        "            self.proofs.prefetch(key);\n"
        "        })\n"
        "    }",
    ),
    (
        "src/table/mod.rs",
        "    pub fn prefetch_proved(&self, key: u64) {\n        self.proofs.prefetch(key);\n    }",
        "    pub fn prefetch_proved(&self, key: u64) {\n"
        "        crate::probe::count(crate::probe::C_PREFETCHES, 1);\n"
        "        crate::timed!(crate::probe::L_PREFETCH, self.proofs.prefetch(key))\n"
        "    }",
    ),
    (
        "src/table/mod.rs",
        "    pub fn prefetch_filter(&self, key: u64) {\n        self.filter.prefetch_word(key);\n    }",
        "    pub fn prefetch_filter(&self, key: u64) {\n"
        "        crate::probe::count(crate::probe::C_PREFETCHES, 1);\n"
        "        crate::timed!(crate::probe::L_PREFETCH, self.filter.prefetch_word(key))\n"
        "    }",
    ),
    (
        "src/table/mod.rs",
        "        if let Some(e) = self.proofs.get(key, counter) {\n"
        "            return Some((\n"
        "                Value::Proved {",
        "        if let Some(e) = crate::timed!(crate::probe::L_PROOF, self.proofs.get(key, counter))\n"
        "        {\n"
        "            return Some((\n"
        "                Value::Proved {",
    ),
    (
        "src/table/mod.rs",
        "        self.tt.probe(key, counter).map(|v| (v, EntrySource::Tt))",
        "        crate::probe::count(crate::probe::C_TT_PROBES, 1);\n"
        "        crate::timed!(crate::probe::L_TT, self.tt.probe(key, counter))\n"
        "            .map(|v| (v, EntrySource::Tt))",
    ),
    (
        "src/table/mod.rs",
        "    pub fn proved(&self, key: u64, counter: u16) -> Option<ProofEntry> {\n"
        "        self.proofs.get(key, counter)\n"
        "    }",
        "    pub fn proved(&self, key: u64, counter: u16) -> Option<ProofEntry> {\n"
        "        crate::probe::count(crate::probe::C_PROOF_PROBES, 1);\n"
        "        crate::timed!(crate::probe::L_PROOF, self.proofs.get(key, counter))\n"
        "    }",
    ),
    (
        "src/table/mod.rs",
        "    pub fn store(&mut self, key: u64, value: &Value, work: u32) {\n        match *value {",
        "    pub fn store(&mut self, key: u64, value: &Value, work: u32) {\n"
        "        crate::probe::count(crate::probe::C_STORES, 1);\n"
        "        crate::timed!(crate::probe::L_STORE, self.store_inner(key, value, work));\n"
        "    }\n\n"
        "    fn store_inner(&mut self, key: u64, value: &Value, work: u32) {\n"
        "        match *value {",
    ),
    # --- threat: count the pseudo-legal stream the closures read (untimed:
    # it is inside the closure leaves) --------------------------------------
    (
        "src/guide/threat.rs",
        "    let mut moves = MoveList::new();\n"
        "    generate_pseudo_legal(board, &mut moves);\n"
        "    for &m in moves.as_slice() {",
        "    let mut moves = MoveList::new();\n"
        "    generate_pseudo_legal(board, &mut moves);\n"
        "    crate::probe::count(crate::probe::C_PSEUDO_LISTS, 1);\n"
        "    crate::probe::count(crate::probe::C_PSEUDO_MOVES, moves.len() as u64);\n"
        "    for &m in moves.as_slice() {",
    ),
    (
        "src/guide/null.rs",
        "        let _at = crate::probe::At::new(crate::probe::S_NULLP1);\n",
        "        let _at = crate::probe::At::new(crate::probe::S_NULLP1);\n"
        "        crate::probe::tax();\n",
    ),
    (
        "src/guide/null.rs",
        "    let mut child_state = StateInfo::new();\n"
        "    crate::timed!(\n"
        "        crate::probe::L_STATE,\n"
        "        child.populate_state(&mut child_state)\n"
        "    );",
        "    crate::probe::tax();\n"
        "    let mut child_state = StateInfo::new();\n"
        "    crate::timed!(\n"
        "        crate::probe::L_STATE,\n"
        "        child.populate_state(&mut child_state)\n"
        "    );",
    ),
    (
        "src/guide/replay.rs",
        "    let _at = crate::probe::At::new(crate::probe::S_REPLAY);\n"
        "    crate::probe::count(crate::probe::C_REPLAY_CALLS, 1);\n",
        "    let _at = crate::probe::At::new(crate::probe::S_REPLAY);\n"
        "    crate::probe::tax();\n"
        "    crate::probe::count(crate::probe::C_REPLAY_CALLS, 1);\n",
    ),
    # --- region aggregates: where the frames' unattributed time is ----------
    (
        "src/search/frame.rs",
        "        for m in &moves {\n            let e = mover_ctx.effect(board, m);",
        "        crate::timed!(crate::probe::A_P1, {\n"
        "        for m in &moves {\n            let e = mover_ctx.effect(board, m);",
    ),
    (
        "src/search/frame.rs",
        "        }\n\n        // Pass 2 (filter): which keys may be stored, plus their prefetch.",
        "        }\n"
        "        });\n\n"
        "        // Pass 2 (filter): which keys may be stored, plus their prefetch.",
    ),
    (
        "src/search/frame.rs",
        "        let mut fresh: Vec<bool> = vec![false; children.len()];\n"
        "        for (i, c) in children.iter_mut().enumerate() {",
        "        let mut fresh: Vec<bool> = vec![false; children.len()];\n"
        "        crate::timed!(crate::probe::A_P3, {\n"
        "        for (i, c) in children.iter_mut().enumerate() {",
    ),
    (
        "src/search/frame.rs",
        "        }\n        if guide_on {\n            // Plan-19 R3: the check number is the evasions",
        "        }\n"
        "        });\n"
        "        crate::timed!(crate::probe::A_START, {\n"
        "        if guide_on {\n            // Plan-19 R3: the check number is the evasions",
    ),
    (
        "src/search/frame.rs",
        "        }\n        // Item-15 GROUP: partition the plain moves out of the OR children and",
        "        }\n"
        "        });\n"
        "        // Item-15 GROUP: partition the plain moves out of the OR children and",
    ),
    (
        "src/guide/null.rs",
        "    for &m in moves.as_slice() {\n"
        "        crate::timed!(crate::probe::L_DO, scratch.do_move(m, &mut state));",
        "    crate::timed!(crate::probe::A_NLOOP, {\n"
        "    for &m in moves.as_slice() {\n"
        "        crate::timed!(crate::probe::L_DO, scratch.do_move(m, &mut state));",
    ),
    (
        "src/guide/null.rs",
        "        crate::timed!(crate::probe::L_UNDO, scratch.undo_move(m, &state));\n"
        "    }\n    None\n}",
        "        crate::timed!(crate::probe::L_UNDO, scratch.undo_move(m, &state));\n"
        "    }\n"
        "    });\n    None\n}",
    ),
]

# Files copied to the scratch tree (source, manifest, licence gate input).
COPY = ["Cargo.toml", "Cargo.lock", "build.rs", "LICENSE", "src"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=str(pathlib.Path(__file__).resolve().parents[5]))
    ap.add_argument("--dest", required=True)
    ap.add_argument("--stride", type=int, default=32, help="one expansion in STRIDE is timed")
    ap.add_argument("--leaves", default="on", choices=["on", "off"],
                    help="off = the aggregates-only ground-truth build")
    ap.add_argument("--only", default="",
                    help="comma-separated leaf names to time (default: all)")
    args = ap.parse_args()
    repo = pathlib.Path(args.repo)
    dest = pathlib.Path(args.dest)
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True)
    for item in COPY:
        src = repo / item
        dst = dest / item
        if src.is_dir():
            shutil.copytree(src, dst, ignore=shutil.ignore_patterns("tests.rs"))
        else:
            shutil.copy2(src, dst)
        if item == "src":
            # The unit tests are not part of the census build (they would
            # need every instrumented call site to type-check twice).
            for extra in ("testutil.rs",):
                (dst / extra).unlink(missing_ok=True)
    import re as _re
    names = [ln.strip() for ln in args.only.split(",") if ln.strip()]
    NLEAF = len(_re.findall(r'"([A-Za-z_0-9]+)"',
                           PROBE_RS.split("pub const LEAF_NAMES")[1].split("];")[0]))
    if names:
        import re
        block = PROBE_RS.split("pub const LEAF_NAMES")[1].split("];")[0]
        all_names = re.findall(r'"([A-Za-z_0-9]+)"', block)
        leaves = all_names[: all_names.index("A_enter")]
        unknown = sorted(set(names) - set(leaves))
        if unknown:
            sys.exit(f"unknown leaf name(s): {unknown}; known: {leaves}")
        only = "[" + ", ".join(
            ["true" if ln in names else "false" for ln in leaves]
            + ["true"] * (NLEAF - len(leaves))
        ) + "]"
    else:
        only = "[true; NLEAF]"
    (dest / "src" / "probe.rs").write_text(
        PROBE_RS.replace("__STRIDE__", str(args.stride))
        .replace("__LEAVES__", "true" if args.leaves == "on" else "false")
        .replace("__ON_LEAF__", only)
    )
    failed = 0
    for rel, old, new in PATCHES:
        path = dest / rel
        text = path.read_text()
        n = text.count(old)
        if n != 1:
            print(f"FAIL {rel}: pattern occurs {n}x: {old.splitlines()[0]!r}", file=sys.stderr)
            failed += 1
            continue
        path.write_text(text.replace(old, new))
    if failed:
        sys.exit(f"{failed} patch(es) did not apply")
    print(f"instrumented tree: {dest} ({len(PATCHES)} patches)")


if __name__ == "__main__":
    main()
