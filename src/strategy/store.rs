//! Event-fed in-memory proof store (D15), active only with `--proof-out`.
//!
//! Each key keeps a **non-dominated** set of `(move, length, limit)` entries:
//! a new entry is dropped when an existing one has `length <=` and `limit >=`
//! it, and it removes the entries it dominates (shorter-or-holds-further).
//! Lookups only, never iterated, so the store stays deterministic.
//!
//! The `--proof-memory MB` byte cap is accounted conservatively: a new key is
//! charged the key plus a Vec header plus 32 bytes of hash-map overhead, and
//! each entry `2 × size_of::<StoreEntry>()` (Vec growth slack). When a charge
//! would exceed the cap the store sets `full`, stops taking entries, and the
//! search continues unchanged; export then fails with `store-full`.

use crate::events::{ProofEvent, ProofSink};
use atomic_movegen::types::Move;
use std::collections::HashMap;

/// One stored attacker move with its proof bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreEntry {
    /// The winning move.
    pub mv: Move,
    /// Proof length in attacker moves below this position.
    pub length: u16,
    /// Highest 50-move counter at which the entry holds.
    pub limit: u8,
}

/// The event-fed proof store.
pub struct ProofStore {
    map: HashMap<u64, Vec<StoreEntry>>,
    bytes: usize,
    cap: usize,
    full: bool,
}

const KEY_CHARGE: usize = std::mem::size_of::<u64>() + std::mem::size_of::<Vec<StoreEntry>>() + 32;
const ENTRY_CHARGE: usize = 2 * std::mem::size_of::<StoreEntry>();

impl ProofStore {
    /// A store with the `--proof-memory` cap in MiB.
    pub fn new(proof_memory_mb: usize) -> ProofStore {
        Self::with_cap_bytes(proof_memory_mb.saturating_mul(1 << 20))
    }

    /// A store with an explicit byte cap (tests).
    pub fn with_cap_bytes(cap: usize) -> ProofStore {
        ProofStore {
            map: HashMap::new(),
            bytes: 0,
            cap,
            full: false,
        }
    }

    /// The non-dominated entries for `key` (empty when absent).
    pub fn get(&self, key: u64) -> &[StoreEntry] {
        self.map.get(&key).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Whether the cap was hit (export then fails `store-full`).
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// The accounted bytes and the cap (diagnostics).
    pub fn accounting(&self) -> (usize, usize) {
        (self.bytes, self.cap)
    }

    fn insert(&mut self, key: u64, entry: StoreEntry) {
        if self.full {
            return;
        }
        let new_key = !self.map.contains_key(&key);
        let charge = (if new_key { KEY_CHARGE } else { 0 }) + ENTRY_CHARGE;
        if self.bytes + charge > self.cap {
            self.full = true;
            return;
        }
        let entries = self.map.entry(key).or_default();
        if entries
            .iter()
            .any(|e| e.length <= entry.length && e.limit >= entry.limit)
        {
            return;
        }
        entries.retain(|e| !(e.length >= entry.length && e.limit <= entry.limit));
        self.bytes += charge;
        entries.push(entry);
    }
}

impl ProofSink for ProofStore {
    fn proven(&mut self, event: ProofEvent) {
        let ProofEvent::AttackerProven {
            key,
            mv,
            length,
            limit,
        } = event;
        self.insert(key, StoreEntry { mv, length, limit });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_movegen::types::Square;

    fn mv(from: Square, to: Square) -> Move {
        Move::make_move(from, to)
    }

    fn entry(from: Square, to: Square, length: u16, limit: u8) -> StoreEntry {
        StoreEntry {
            mv: mv(from, to),
            length,
            limit,
        }
    }

    #[test]
    fn non_dominated_insert() {
        let mut s = ProofStore::with_cap_bytes(1 << 20);
        let a = entry(Square::A1, Square::A2, 5, 50);
        s.insert(1, a);
        // Strictly dominated by a: dropped.
        s.insert(1, entry(Square::B1, Square::B2, 6, 40));
        assert_eq!(s.get(1), &[a]);
        // Dominates a: replaces it.
        let b = entry(Square::B1, Square::B2, 4, 60);
        s.insert(1, b);
        assert_eq!(s.get(1), &[b]);
        // Incomparable: shorter but lower limit, and longer but higher limit.
        let c = entry(Square::C1, Square::C2, 3, 20);
        s.insert(1, c);
        let d = entry(Square::D1, Square::D2, 8, 90);
        s.insert(1, d);
        assert_eq!(s.get(1).len(), 3);
        // A duplicate is dominated by itself: no-op.
        s.insert(1, c);
        assert_eq!(s.get(1).len(), 3);
    }

    #[test]
    fn cap_sets_full() {
        let mut s = ProofStore::with_cap_bytes(KEY_CHARGE + ENTRY_CHARGE - 1);
        s.insert(1, entry(Square::A1, Square::A2, 1, 99));
        assert!(s.is_full());
        assert!(s.get(1).is_empty());
        // Later events are ignored too.
        s.insert(2, entry(Square::B1, Square::B2, 1, 99));
        assert!(s.get(2).is_empty());
    }

    #[test]
    fn accounting_never_exceeds_cap() {
        let cap = KEY_CHARGE + 3 * ENTRY_CHARGE;
        let mut s = ProofStore::with_cap_bytes(cap);
        for i in 0..10u8 {
            s.insert(1, entry(Square::A1, Square::A2, i.into(), 10 + i * 10));
        }
        assert!(s.accounting().0 <= cap);
    }
}
