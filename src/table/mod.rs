//! The in-search tables (M1): TT v2, the "possibly stored" bit filter and
//! the proof table, inside one `--memory` label ([`split`]), plus the
//! [`Tables`] facade that routes lookups and stores.
//!
//! Routing:
//!
//! - `lookup` tries `proofs.get(key, counter)` first (a usable proof is
//!   exact), then a usable TT entry; `None` means fresh `(1, 1)`. A usable
//!   proof wins over a usable refutation.
//! - `store`: a `Proved` value goes to the proof table **and** the key's TT
//!   slot is cleared if present — after a proof the TT only holds a value
//!   from a later re-search, mirroring M0's in-place proof overwrite. A
//!   terminal `Proved { length: 0, .. }` is never inserted: no proof-table
//!   write, no TT clear, no filter bit (it is recomputed at
//!   classification). An Open value or a path-independent `Refuted` goes to
//!   the TT. Both stored routes set the filter bit; a path-dependent
//!   refutation does nothing.
//! - The filter (when active) turns most misses away before the tables are
//!   touched. It is exact in the negative direction, so it changes no
//!   result.

mod alloc;
mod filter;
pub mod proofs;
mod tt;

use crate::search::value::{ANY, Value};
use atomic_movegen::types::Move;
pub use filter::Filter;
pub use proofs::{ProofEntry, ProofSlot, ProofTable};
pub use tt::{Cluster, Tt};

/// The `--memory` budget for one `prove` run, in bytes over `mb * 2^20`:
/// three quarters to the proof table, a 128 KiB–4 MiB filter, the rest to
/// the TT.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// TT clusters (64 B each).
    pub tt_clusters: usize,
    /// Proof-table slots (16 B each).
    pub proof_slots: usize,
    /// Filter bits (a power of two in `[2^20, 2^25]`).
    pub filter_bits: usize,
}

/// Split `mb` MiB into the TT / proof-table / filter budgets with the
/// default proof share (3/4). Plan23 selected it: at 192 MB the deep-win
/// archive (up to 5.7 M live entries) fits, while the 1/4 control fills and
/// refuses the export.
pub fn split(mb: usize) -> Budget {
    split_share(mb, 75)
}

/// Split `mb` MiB with `proof_percent` of the label for the proof table
/// (1..=90; the CLI and `Tables` validate). Invariant:
/// `64*tt_clusters + 16*proof_slots + filter_bits/8` bytes fit the label.
pub fn split_share(mb: usize, proof_percent: u8) -> Budget {
    let total = mb.saturating_mul(1 << 20);
    let pct = usize::from(proof_percent.clamp(1, 90));
    // The filter has a 2^20-bit floor (128 KiB); reserve it plus one TT
    // cluster, so every share keeps the byte invariant even at 1 MiB.
    let proof_cap = total.saturating_sub(64 + (1 << 17));
    let proof_bytes = (total * pct / 100).min(proof_cap);
    let proof_slots = proof_bytes / 16;
    let tt_slot_estimate = (total - proof_bytes) / 16;
    // prev_power_of_two(8 * tt_slot_estimate), clamped to [2^20, 2^25].
    let x = (8 * tt_slot_estimate).max(1);
    let filter_bits = (1usize << (usize::BITS - 1 - x.leading_zeros())).clamp(1 << 20, 1 << 25);
    let tt_clusters = total.saturating_sub(proof_bytes + filter_bits / 8) / 64;
    Budget {
        tt_clusters: tt_clusters.clamp(1, u32::MAX as usize),
        proof_slots,
        filter_bits,
    }
}

/// Prefetch one object into cache (x86-64 `_mm_prefetch` T0; a no-op
/// elsewhere). The only `unsafe` outside [`alloc`].
#[inline(always)]
pub(crate) fn prefetch<T>(r: &T) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch is a hint only — it cannot fault, takes no lock and
    // observes nothing; the address need not be dereferenceable.
    unsafe {
        core::arch::x86_64::_mm_prefetch(
            r as *const T as *const i8,
            core::arch::x86_64::_MM_HINT_T0,
        );
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = r;
}

/// Which table served a [`Tables::lookup_with_source`] hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntrySource {
    /// The full-key proof table (an exact proof).
    Proof,
    /// The TT (an `Open` pair or a path-independent refutation).
    Tt,
}

/// The TT + proof table + filter of one `prove` run.
pub struct Tables {
    tt: Tt,
    proofs: ProofTable,
    filter: Filter,
}

impl Tables {
    /// Allocate the tables for the `memory_mb` label. `filter = false` is a
    /// test hook for the filter-neutrality test; the CLI always passes
    /// `true`.
    pub fn new(memory_mb: usize, filter: bool) -> Tables {
        Tables::with_share(memory_mb, filter, 75)
    }

    /// Allocate with an explicit proof-table share (percent of `--memory`).
    pub fn with_share(memory_mb: usize, filter: bool, proof_percent: u8) -> Tables {
        let b = split_share(memory_mb, proof_percent);
        Tables {
            tt: Tt::new(b.tt_clusters),
            proofs: ProofTable::new(b.proof_slots),
            filter: if filter {
                Filter::new(b.filter_bits)
            } else {
                Filter::disabled()
            },
        }
    }

    /// `Tables` over caller-built storage (test-only: the lazy-vs-eager
    /// equivalence test).
    #[cfg(test)]
    pub fn from_parts(tt: Tt, proofs: ProofTable, filter: Filter) -> Tables {
        Tables { tt, proofs, filter }
    }

    /// Whether the filter is still answering (enabled and not dropped).
    pub fn filtering(&self) -> bool {
        self.filter.is_active()
    }

    /// Prefetch the filter word of `key`.
    pub fn prefetch_filter(&self, key: u64) {
        self.filter.prefetch_word(key);
    }

    /// `true` when the tables may hold an entry for `key`.
    pub fn may_hold(&self, key: u64) -> bool {
        self.filter.may_hold(key)
    }

    /// Prefetch the TT cluster and the proof run's first slot of `key`.
    pub fn prefetch(&self, key: u64) {
        self.tt.prefetch(key);
        self.proofs.prefetch(key);
    }

    /// The usable stored value for `key` at `counter`: a usable proof-table
    /// entry first, then a usable TT entry; `None` = fresh.
    pub fn lookup(&self, key: u64, counter: u16) -> Option<Value> {
        self.lookup_with_source(key, counter).map(|(v, _)| v)
    }

    /// Like [`Tables::lookup`], but also reports which table served the hit
    /// (the plan-12 attribution counters).
    pub(crate) fn lookup_with_source(
        &self,
        key: u64,
        counter: u16,
    ) -> Option<(Value, EntrySource)> {
        if let Some(e) = self.proofs.get(key, counter) {
            return Some((
                Value::Proved {
                    length: e.length,
                    limit: e.limit,
                    mv: if e.mv == 0 {
                        None
                    } else {
                        Some(Move::from_bits(e.mv).expect("stored move bits decode"))
                    },
                },
                EntrySource::Proof,
            ));
        }
        self.tt.probe(key, counter).map(|v| (v, EntrySource::Tt))
    }

    /// Route `value` under `key` (see the module docs). `work` is the
    /// expansions spent below the node during its last frame.
    pub fn store(&mut self, key: u64, value: &Value, work: u32) {
        match *value {
            Value::Proved { length: 0, .. } => {}
            Value::Proved { length, limit, mv } => {
                self.proofs.insert(
                    key,
                    ProofEntry {
                        mv: mv.map_or(0, |m| m.to_bits()),
                        length,
                        limit,
                    },
                );
                self.tt.clear(key);
                self.filter.set(key);
            }
            Value::Refuted { cycle, .. } if cycle != ANY => {}
            _ => {
                self.tt.store(key, value, work);
                self.filter.set(key);
            }
        }
    }

    /// The proof table.
    pub fn proofs(&self) -> &ProofTable {
        &self.proofs
    }

    /// Consume the tables, returning the proof table.
    pub fn into_proofs(self) -> ProofTable {
        self.proofs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_invariant() {
        for mb in 1usize..=4096 {
            let b = split(mb);
            assert!(
                64 * b.tt_clusters + 16 * b.proof_slots + b.filter_bits / 8 <= mb << 20,
                "mb={mb}: {:?} exceeds the label",
                b
            );
            assert!(b.tt_clusters >= 1);
            assert!(b.filter_bits.is_power_of_two());
        }
    }

    #[test]
    fn split_share_invariant() {
        for mb in [1usize, 7, 16, 64, 192, 1024, 4096] {
            for pct in 1u8..=90 {
                let b = split_share(mb, pct);
                assert!(
                    64 * b.tt_clusters + 16 * b.proof_slots + b.filter_bits / 8 <= mb << 20,
                    "mb={mb} pct={pct}: {b:?}"
                );
                assert!(b.tt_clusters >= 1);
                assert!(b.filter_bits.is_power_of_two());
            }
        }
        assert_eq!(split_share(192, 75).proof_slots, split(192).proof_slots);
    }

    #[test]
    fn split_at_192mb() {
        let b = split(192);
        assert_eq!(b.proof_slots, 9_437_184);
        assert_eq!(b.filter_bits, 1 << 24);
        assert_eq!(b.tt_clusters, 753_664);
    }

    #[test]
    fn proved_store_routing() {
        let mut t = Tables::new(16, true);
        // An open value goes to the TT and sets the filter bit.
        t.store(1, &Value::Open { pn: 2, dn: 3 }, 5);
        assert_eq!(t.lookup(1, 0), Some(Value::Open { pn: 2, dn: 3 }));
        // A proof replaces the TT entry and is served by the proof table.
        t.store(
            1,
            &Value::Proved {
                length: 4,
                limit: 50,
                mv: Some(Move::make_move(
                    atomic_movegen::types::Square::A2,
                    atomic_movegen::types::Square::A3,
                )),
            },
            9,
        );
        let got = t.lookup(1, 0);
        assert!(
            matches!(
                got,
                Some(Value::Proved {
                    length: 4,
                    limit: 50,
                    mv: Some(_)
                })
            ),
            "{got:?}"
        );
        // A terminal win is never inserted.
        t.store(
            2,
            &Value::Proved {
                length: 0,
                limit: 99,
                mv: None,
            },
            1,
        );
        assert_eq!(t.lookup(2, 0), None);
        // A path-dependent refutation stores nothing.
        t.store(3, &Value::Refuted { cycle: 2, from: 0 }, 4);
        assert_eq!(t.lookup(3, 0), None);
        // A path-independent refutation is TT-stored.
        t.store(
            4,
            &Value::Refuted {
                cycle: ANY,
                from: 10,
            },
            4,
        );
        assert_eq!(t.lookup(4, 5), None);
        assert!(matches!(
            t.lookup(4, 10),
            Some(Value::Refuted {
                cycle: ANY,
                from: 10
            })
        ));
    }

    #[test]
    fn proof_wins_over_refutation() {
        let mut t = Tables::new(16, false);
        t.store(
            7,
            &Value::Refuted {
                cycle: ANY,
                from: 0,
            },
            1,
        );
        t.store(
            7,
            &Value::Proved {
                length: 2,
                limit: 90,
                mv: None,
            },
            2,
        );
        assert!(matches!(
            t.lookup(7, 0),
            Some(Value::Proved { length: 2, .. })
        ));
    }
}
