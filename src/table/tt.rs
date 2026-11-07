// Justification for >10 KB: the slot encoding, replacement rules and
// their unit tests are one contract; splitting would scatter the rules.

//! TT v2: 64-byte clusters of four 16-byte slots with a 32-bit partial key.
//!
//! A slot is `{check, pn, dn, meta}`, all `u32`:
//!
//! - `check = max(1, (key >> 32) as u32)` — the partial key; `0` marks an
//!   empty slot, so an all-zero cluster is empty (the [`super::alloc`]
//!   invariant);
//! - `pn`/`dn`: an [`Value::Open`] pair (`>= 1`, `<= OPEN_MAX`), or
//!   `pn = INFINITY, dn = 0` for a path-independent refutation;
//! - `meta = min(work, 2^24 - 1) << 8 | from`.
//!
//! The cluster index is `((key as u32 as u64) * n_clusters) >> 32`, an exact
//! fixed-point scale (`n <= 2^32` so the product cannot overflow), so any
//! cluster count works — the table is not a power of two.
//!
//! The TT holds only [`Value::Open`] values and path-independent
//! refutations. Proofs live in the proof table ([`super::proofs`]); a
//! path-dependent refutation (`cycle != ANY`) is never stored and leaves an
//! existing entry untouched.
//!
//! Store rules: the slot with the same `check` is overwritten; else an empty
//! slot; else the lowest-`work` slot (the first on ties).
//!
//! # Partial keys are sound here
//!
//! A `check` collision can hand a node a wrong Open pair or a wrong
//! path-independent refutation. In attacker-only semantics both are
//! completeness-only hazards: an Open pair only steers the search order and
//! thresholds, and a wrong refutation can at most make the search give up on
//! a line it could have won. Neither can manufacture a proof — a proved
//! value requires proved children, and proofs never live in the TT (they
//! sit in the full-key proof table). So a collision can lose a win, never
//! invent one.

use crate::search::value::{ANY, INFINITY, OPEN_MAX, Value};
use crate::table::alloc::Zeroed;
use crate::table::prefetch;

/// The 32-bit partial key of `key`; never 0 (`0` is the empty-slot mark).
pub(crate) fn check(key: u64) -> u32 {
    ((key >> 32) as u32).max(1)
}

/// One TT slot.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Slot {
    /// Partial key; `0` = empty.
    pub(crate) check: u32,
    /// Proof number, or [`INFINITY`] on a stored refutation.
    pub(crate) pn: u32,
    /// Disproof number (`0` on a stored refutation).
    pub(crate) dn: u32,
    /// `work << 8 | from`.
    pub(crate) meta: u32,
}

/// Four slots on one 64-byte cache line.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Cluster(pub [Slot; 4]);

const _: () = assert!(std::mem::size_of::<Slot>() == 16);
const _: () = assert!(std::mem::size_of::<Cluster>() == 64);
const _: () = assert!(std::mem::align_of::<Cluster>() == 64);

impl Slot {
    fn work(&self) -> u32 {
        self.meta >> 8
    }

    fn from(&self) -> u8 {
        (self.meta & 0xff) as u8
    }
}

/// The transposition table.
pub struct Tt {
    clusters: Zeroed<Cluster>,
}

impl Tt {
    /// A table of `n` clusters (at least 1), lazily zeroed.
    pub fn new(n: usize) -> Tt {
        Tt {
            clusters: Zeroed::new(n.max(1)),
        }
    }

    /// A table of `n` clusters on eagerly written storage (test-only:
    /// [`Zeroed`]'s representation-only equivalence).
    #[cfg(test)]
    pub fn eager(n: usize) -> Tt {
        Tt {
            clusters: Zeroed::eager(n.max(1)),
        }
    }

    /// The number of clusters.
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    fn index(&self, key: u64) -> usize {
        (((key as u32 as u64) * self.clusters.len() as u64) >> 32) as usize
    }

    /// Prefetch the cluster `key` maps to.
    pub fn prefetch(&self, key: u64) {
        prefetch(&self.clusters[self.index(key)]);
    }

    /// The usable stored value for `key` at `counter`, if the cluster holds
    /// a slot with a matching `check`. A refutation is usable iff
    /// `counter >= from`; an Open value always.
    pub fn probe(&self, key: u64, counter: u16) -> Option<Value> {
        let c = check(key);
        let slot = self.clusters[self.index(key)]
            .0
            .iter()
            .find(|s| s.check == c)?;
        if slot.pn == INFINITY {
            debug_assert_eq!(slot.dn, 0);
            (counter >= u16::from(slot.from())).then_some(Value::Refuted {
                cycle: ANY,
                from: slot.from(),
            })
        } else {
            debug_assert!(slot.pn >= 1 && slot.dn >= 1);
            Some(Value::Open {
                pn: slot.pn,
                dn: slot.dn,
            })
        }
    }

    /// Store `value` under `key` with `work`. A path-dependent refutation is
    /// a no-op; proofs do not belong in the TT (a no-op here — the caller
    /// routes them to the proof table and uses [`Tt::clear`]).
    pub fn store(&mut self, key: u64, value: &Value, work: u32) {
        let (pn, dn, from) = match *value {
            Value::Open { pn, dn } => {
                debug_assert!(pn >= 1 && dn >= 1 && pn <= OPEN_MAX && dn <= OPEN_MAX);
                (pn, dn, 0u8)
            }
            Value::Refuted { cycle, from } => {
                if cycle != ANY {
                    return; // path-dependent: never stored, existing stays
                }
                (INFINITY, 0, from)
            }
            Value::Proved { .. } => {
                debug_assert!(false, "proofs are routed to the proof table");
                return;
            }
        };
        let meta = work.min((1 << 24) - 1) << 8 | u32::from(from);
        let c = check(key);
        let idx = self.index(key);
        let cluster = &mut self.clusters[idx];
        if let Some(slot) = cluster.0.iter_mut().find(|s| s.check == c) {
            *slot = Slot {
                check: c,
                pn,
                dn,
                meta,
            };
            return;
        }
        if let Some(slot) = cluster.0.iter_mut().find(|s| s.check == 0) {
            *slot = Slot {
                check: c,
                pn,
                dn,
                meta,
            };
            return;
        }
        let slot = cluster.0.iter_mut().min_by_key(|s| s.work()).unwrap();
        *slot = Slot {
            check: c,
            pn,
            dn,
            meta,
        };
    }

    /// Zero the slot holding `key`'s `check`, if present (used when a proof
    /// replaces a table entry, mirroring M0's in-place overwrite).
    pub fn clear(&mut self, key: u64) {
        let c = check(key);
        let idx = self.index(key);
        let cluster = &mut self.clusters[idx];
        if let Some(slot) = cluster.0.iter_mut().find(|s| s.check == c) {
            *slot = Slot {
                check: 0,
                pn: 0,
                dn: 0,
                meta: 0,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(pn: u32, dn: u32) -> Value {
        Value::Open { pn, dn }
    }

    #[test]
    fn zeroed_cluster_is_empty() {
        let t = Tt::new(3);
        for c in t.clusters.iter() {
            assert!(c.0.iter().all(|s| s.check == 0));
        }
    }

    #[test]
    fn check_is_never_zero() {
        assert_eq!(check(0), 1);
        assert_eq!(check(u64::MAX), u32::MAX);
        assert_eq!(check(0xFFFF_FFFF), 1);
    }

    #[test]
    fn store_probe_roundtrip() {
        let mut t = Tt::new(64);
        // Keys need distinct checks (high 32 bits): 41 and 42 share check 1.
        let (a, b) = (42u64 << 32, 41u64 << 32);
        t.store(a, &open(3, 5), 10);
        assert_eq!(t.probe(a, 0), Some(open(3, 5)));
        assert_eq!(t.probe(b, 0), None);
        t.store(a, &open(1, 2), 7);
        assert_eq!(t.probe(a, 0), Some(open(1, 2)));
    }

    #[test]
    fn refuted_roundtrip_and_usable() {
        let mut t = Tt::new(64);
        t.store(
            9,
            &Value::Refuted {
                cycle: ANY,
                from: 40,
            },
            3,
        );
        assert_eq!(t.probe(9, 39), None);
        assert_eq!(
            t.probe(9, 40),
            Some(Value::Refuted {
                cycle: ANY,
                from: 40
            })
        );
        assert_eq!(
            t.probe(9, 99),
            Some(Value::Refuted {
                cycle: ANY,
                from: 40
            })
        );
    }

    #[test]
    fn path_dependent_refutation_noop() {
        let mut t = Tt::new(64);
        t.store(7, &Value::Refuted { cycle: 3, from: 0 }, 5);
        assert_eq!(t.probe(7, 0), None);
        t.store(7, &open(2, 2), 1);
        t.store(7, &Value::Refuted { cycle: 3, from: 0 }, 9);
        assert_eq!(t.probe(7, 0), Some(open(2, 2)));
    }

    #[test]
    fn lowest_work_replacement() {
        // One cluster: four keys sharing the index and check-distinct.
        let mut t = Tt::new(1);
        let key = |hi: u64| hi << 32;
        for i in 1..=4u64 {
            t.store(key(i), &open(1, 1), 10 * i as u32);
        }
        t.store(key(5), &open(9, 9), 1);
        // Lowest work (key(1), work 10) was evicted.
        assert_eq!(t.probe(key(1), 0), None);
        assert_eq!(t.probe(key(5), 0), Some(open(9, 9)));
        // Same check overwrites even at higher work.
        t.store(key(2), &open(7, 7), 100);
        assert_eq!(t.probe(key(2), 0), Some(open(7, 7)));
    }

    #[test]
    fn clear_zeros_the_slot() {
        let mut t = Tt::new(1);
        t.store(42 << 32 | 1, &open(4, 4), 5);
        assert!(t.probe(42 << 32 | 1, 0).is_some());
        t.clear(42 << 32 | 1);
        assert_eq!(t.probe(42 << 32 | 1, 0), None);
        assert!(t.clusters[0].0.iter().all(|s| s.check == 0));
    }

    #[test]
    fn index_in_range_for_odd_counts() {
        for n in [1usize, 3, 7, 999, 65_537] {
            let t = Tt::new(n);
            let mut seed = 0x243F_6A88_85A3_08D3u64;
            for _ in 0..10_000 {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                assert!(t.index(seed) < n);
            }
        }
    }
}
