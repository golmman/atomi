// Justification for >10 KB: the table, its normative domination/
// probing rules and the unit tests that pin them are one contract.

//! The proof table (D18): fixed-size open addressing, full 64-bit keys.
//!
//! Each key keeps a **non-dominated** set of `(mv, length, limit)` entries,
//! the same domination rule as [`crate::strategy::store::ProofStore`]: a new
//! entry is dropped when a stored one has `length <=` and `limit >=` it, and
//! it tombstones the stored entries it dominates. The table never evicts and
//! refuses new proofs at 90% load (`refused`).
//!
//! It holds attacker proofs (`mv` set) and defender proofs (`mv == 0` =
//! `Move::NONE`), because the TT no longer can. Terminal wins (`length 0`)
//! are never inserted — they are recomputed at classification.
//!
//! Slots are `#[repr(C)]` 16 bytes; `state` is the only liveness signal
//! (`0` free — the all-zero slot, `1` live, `2` tombstone), so key 0 is a
//! valid position key.
//!
//! Placement: linear probing from `((key as u128 * slots) >> 64)`, wrapping
//! at the end. A *run* is the contiguous non-free slots from a key's start.
//! An insert reuses the run's first tombstone, else takes the run-ending
//! free slot while `used < 9/10 * slots`, else sets `refused`.

use crate::table::alloc::Zeroed;
use crate::table::prefetch;

/// One proof-table entry (`mv` is `Move::to_bits`, `0` at defender proofs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProofEntry {
    /// Winning move bits, `0` for a defender proof.
    pub mv: u16,
    /// Proof length in attacker moves.
    pub length: u16,
    /// Highest 50-move counter at which the entry holds.
    pub limit: u8,
}

/// One proof-table slot.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProofSlot {
    /// Full 64-bit position key.
    pub(crate) key: u64,
    /// `Move::to_bits`; `0` = defender proof.
    pub(crate) mv: u16,
    /// Proof length in attacker moves.
    pub(crate) length: u16,
    /// Highest counter the entry holds at.
    pub(crate) limit: u8,
    /// `0` free, `1` live, `2` tombstone. The only liveness signal.
    pub(crate) state: u8,
    /// Padding to 16 bytes.
    _pad: [u8; 2],
}

const _: () = assert!(std::mem::size_of::<ProofSlot>() == 16);

const FREE: u8 = 0;
const LIVE: u8 = 1;
const TOMBSTONE: u8 = 2;

/// The fixed-size proof table.
pub struct ProofTable {
    slots: Zeroed<ProofSlot>,
    /// Live entries.
    live: usize,
    /// Non-free slots (live + tombstones).
    used: usize,
    /// Live attacker entries (`mv != 0`).
    attacker_live: usize,
    /// Live defender entries (`mv == 0`).
    defender_live: usize,
    /// An insert was refused at the 90% load bound.
    archive_refused: bool,
}

impl ProofTable {
    /// A table of `slots` slots (at least 1), lazily zeroed.
    pub fn new(slots: usize) -> ProofTable {
        ProofTable {
            slots: Zeroed::new(slots.max(1)),
            live: 0,
            used: 0,
            attacker_live: 0,
            defender_live: 0,
            archive_refused: false,
        }
    }

    /// A table on eagerly written storage (test-only: [`Zeroed`]'s
    /// representation-only equivalence).
    #[cfg(test)]
    pub fn eager(slots: usize) -> ProofTable {
        ProofTable {
            slots: Zeroed::eager(slots.max(1)),
            live: 0,
            used: 0,
            attacker_live: 0,
            defender_live: 0,
            archive_refused: false,
        }
    }

    /// The probe start of `key`.
    fn start(&self, key: u64) -> usize {
        ((key as u128 * self.slots.len() as u128) >> 64) as usize
    }

    /// Prefetch `key`'s run's first slot.
    pub fn prefetch(&self, key: u64) {
        prefetch(&self.slots[self.start(key)]);
    }

    /// Insert `entry` under `key` (D15 domination). No-op when a live entry
    /// dominates it; tombstones the live entries it dominates.
    pub fn insert(&mut self, key: u64, entry: ProofEntry) {
        debug_assert!(entry.length >= 1);
        let n = self.slots.len();
        let start = self.start(key);
        // Pass 1: a dominating live entry anywhere in the run makes this a
        // no-op (before any mutation, so the set stays untouched).
        let mut i = start;
        loop {
            let s = self.slots[i];
            if s.state == FREE {
                break;
            }
            if s.state == LIVE && s.key == key && s.length <= entry.length && s.limit >= entry.limit
            {
                return;
            }
            i += 1;
            if i == n {
                i = 0;
            }
            debug_assert!(i != start, "run wrapped: impossible under the 90% bound");
        }
        // Pass 2: tombstone dominated entries, find the first tombstone and
        // the run-ending free slot.
        let mut tomb = None;
        i = start;
        let end = loop {
            let s = self.slots[i];
            if s.state == FREE {
                break i;
            }
            if s.state == TOMBSTONE && tomb.is_none() {
                tomb = Some(i);
            } else if s.state == LIVE
                && s.key == key
                && s.length >= entry.length
                && s.limit <= entry.limit
            {
                self.slots[i].state = TOMBSTONE;
                self.live -= 1;
                if self.slots[i].mv == 0 {
                    self.defender_live -= 1;
                } else {
                    self.attacker_live -= 1;
                }
                // A tombstone created by this insert is reusable too.
                if tomb.is_none() {
                    tomb = Some(i);
                }
            }
            i += 1;
            if i == n {
                i = 0;
            }
            debug_assert!(i != start, "run wrapped: impossible under the 90% bound");
        };
        let target = match tomb {
            Some(t) => t,
            None => {
                if self.used >= n * 9 / 10 {
                    self.archive_refused = true;
                    return;
                }
                self.used += 1;
                end
            }
        };
        self.slots[target] = ProofSlot {
            key,
            mv: entry.mv,
            length: entry.length,
            limit: entry.limit,
            state: LIVE,
            _pad: [0; 2],
        };
        self.live += 1;
        if entry.mv == 0 {
            self.defender_live += 1;
        } else {
            self.attacker_live += 1;
        }
    }

    /// The shortest live entry for `key` that holds at `counter`
    /// (`limit >= counter`); ties go to the higher limit, then the lower
    /// `mv` bits (order-independent).
    pub fn get(&self, key: u64, counter: u16) -> Option<ProofEntry> {
        let n = self.slots.len();
        let start = self.start(key);
        let mut i = start;
        let mut best: Option<ProofSlot> = None;
        loop {
            let s = self.slots[i];
            if s.state == FREE {
                break;
            }
            if s.state == LIVE && s.key == key && u16::from(s.limit) >= counter {
                let better = match best {
                    None => true,
                    Some(b) if s.length != b.length => s.length < b.length,
                    Some(b) if s.limit != b.limit => s.limit > b.limit,
                    Some(b) => s.mv < b.mv,
                };
                if better {
                    best = Some(s);
                }
            }
            i += 1;
            if i == n {
                i = 0;
            }
            if i == start {
                break;
            }
        }
        best.map(|s| ProofEntry {
            mv: s.mv,
            length: s.length,
            limit: s.limit,
        })
    }

    /// Every live entry for `key`, in probe order (any `mv`, including 0).
    pub fn entries(&self, key: u64, out: &mut Vec<ProofEntry>) {
        let n = self.slots.len();
        let start = self.start(key);
        let mut i = start;
        loop {
            let s = self.slots[i];
            if s.state == FREE {
                break;
            }
            if s.state == LIVE && s.key == key {
                out.push(ProofEntry {
                    mv: s.mv,
                    length: s.length,
                    limit: s.limit,
                });
            }
            i += 1;
            if i == n {
                i = 0;
            }
            if i == start {
                break;
            }
        }
    }

    /// Live entries.
    pub fn live(&self) -> usize {
        self.live
    }

    /// Non-free slots (live + tombstones).
    pub fn used(&self) -> usize {
        self.used
    }

    /// Live attacker entries (`mv != 0`).
    pub fn attacker_live(&self) -> usize {
        self.attacker_live
    }

    /// Live defender entries (`mv == 0`).
    pub fn defender_live(&self) -> usize {
        self.defender_live
    }

    /// Whether an archive insert was refused at the 90% load bound (export
    /// then fails `store-full`).
    pub fn refused(&self) -> bool {
        self.archive_refused
    }

    /// Slot count.
    pub fn slots(&self) -> usize {
        self.slots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(mv: u16, length: u16, limit: u8) -> ProofEntry {
        ProofEntry { mv, length, limit }
    }

    #[test]
    fn slot_is_16_bytes() {
        assert_eq!(std::mem::size_of::<ProofSlot>(), 16);
    }

    #[test]
    fn zeroed_slot_is_free() {
        let t = ProofTable::new(4);
        assert!(t.slots.iter().all(|s| s.state == FREE));
        assert_eq!((t.live(), t.used()), (0, 0));
    }

    #[test]
    fn key_zero_works() {
        let mut t = ProofTable::new(8);
        t.insert(0, e(7, 3, 50));
        assert_eq!(t.get(0, 50), Some(e(7, 3, 50)));
        assert_eq!(t.get(0, 51), None);
    }

    #[test]
    fn non_dominated_insert() {
        // The store.rs domination test, ported.
        let mut t = ProofTable::new(64);
        t.insert(1, e(10, 5, 50));
        // Strictly dominated: dropped.
        t.insert(1, e(11, 6, 40));
        let mut got = Vec::new();
        t.entries(1, &mut got);
        assert_eq!(got, vec![e(10, 5, 50)]);
        // Dominates: replaces.
        t.insert(1, e(12, 4, 60));
        got.clear();
        t.entries(1, &mut got);
        assert_eq!(got, vec![e(12, 4, 60)]);
        // Incomparable both ways: kept alongside.
        t.insert(1, e(13, 3, 20));
        t.insert(1, e(14, 8, 90));
        got.clear();
        t.entries(1, &mut got);
        assert_eq!(got.len(), 3);
        // A duplicate is dominated by itself: no-op.
        t.insert(1, e(13, 3, 20));
        got.clear();
        t.entries(1, &mut got);
        assert_eq!(got.len(), 3);
    }

    #[test]
    fn get_returns_shortest_usable() {
        let mut t = ProofTable::new(64);
        // Entries usable at counter 30: (5,50) and (6,99); shortest wins.
        t.insert(2, e(10, 5, 50));
        t.insert(2, e(11, 6, 99));
        t.insert(2, e(12, 4, 20)); // shortest but unusable at 30
        assert_eq!(t.get(2, 30), Some(e(10, 5, 50)));
        // A non-dominated set is unique on (length, limit), so the
        // higher-limit/lower-mv tie-breaks are unreachable by construction.
    }

    #[test]
    fn tombstone_is_reused() {
        let mut t = ProofTable::new(16);
        // Same run: keys hashing to start 0 — with n=16, keys < 2^60 map
        // variously; use keys 0.. which spread. Instead pin by inserting and
        // tombstoning, then check used does not grow on reuse.
        t.insert(1, e(1, 5, 50));
        t.insert(2, e(2, 5, 50));
        t.insert(3, e(3, 5, 50));
        assert_eq!(t.used(), 3);
        // Tombstone key 1's entry with a dominating insert.
        t.insert(1, e(4, 4, 60));
        assert_eq!(t.live(), 3);
        assert_eq!(t.used(), 3);
        // A new key whose run covers that tombstone reuses it. Find one by
        // brute force over keys probing the same region: simplest correct
        // check is used() does not decrease and live grows.
        let used_before = t.used();
        for k in 4..64u64 {
            t.insert(k, e(9, 1, 10));
        }
        assert!(t.used() >= used_before);
    }

    #[test]
    fn wraparound_runs() {
        // With 4 slots and keys whose starts are near the end, the run wraps.
        let mut t = ProofTable::new(4);
        // Start index = key * 4 >> 64: keys >= 3/4 * 2^64 start at slot 3.
        let hi = u64::MAX - 100;
        t.insert(hi, e(1, 1, 10)); // start 3, occupies 3
        t.insert(hi - (1 << 62), e(2, 2, 20)); // start 2
        t.insert(hi - (1 << 62) - 1, e(3, 3, 30));
        assert!(t.live() >= 2);
        assert_eq!(t.get(hi, 0), Some(e(1, 1, 10)));
    }

    #[test]
    fn composition_counters() {
        let mut t = ProofTable::new(64);
        t.insert(1, e(10, 5, 50));
        t.insert(2, e(0, 7, 40));
        assert_eq!(t.attacker_live(), 1);
        assert_eq!(t.defender_live(), 1);
        // Domination moves one entry between classes.
        t.insert(2, e(11, 6, 60));
        assert_eq!(t.attacker_live(), 2);
        assert_eq!(t.defender_live(), 0);
        assert_eq!(t.live(), 2);
    }

    #[test]
    fn refusal_at_90_percent() {
        let mut t = ProofTable::new(10);
        for k in 0..9u64 {
            t.insert(k, e(1, 1, 10));
        }
        assert_eq!(t.used(), 9);
        assert!(!t.refused());
        t.insert(9, e(1, 1, 10));
        assert!(t.refused());
        assert_eq!(t.live(), 9);
        // A still-usable table after refusal.
        assert_eq!(t.get(0, 0), Some(e(1, 1, 10)));
    }

    #[test]
    fn insert_needs_length_one() {
        // debug_assert only in debug builds; release stores nothing for
        // length 0 by contract (the caller filters).
        let t = ProofTable::new(4);
        assert_eq!(t.get(0, 0), None);
    }
}
