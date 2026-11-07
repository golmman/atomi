//! The proof store and JSONL writer (D15/D18).
//!
//! The search's own proof memory is the proof table
//! ([`crate::table::ProofTable`]); the event-fed [`ProofStore`] remains as a
//! test-only differential source. The writer walks real positions from the
//! root over any [`ProofSource`], chooses per (position, counter) the
//! shortest entry that holds at that counter, records position edges as the
//! checker does and rejects any cross-line cycle. It fails closed: on any
//! error nothing is written.

pub mod store;
pub mod write;

use crate::table::ProofTable;
use atomic_movegen::types::Move;
pub use store::{ProofStore, StoreEntry};
pub use write::{ExportError, WriteStats, write_proof};

/// A source of attacker proof entries for the writer (the proof table, or
/// the event-fed store in the differential tests). `entries` may return
/// the set in any order — `choose()` is order-independent because a
/// non-dominated set is unique on `(length, limit)` and ties break on move
/// bits.
pub trait ProofSource {
    /// Append the attacker's stored entries for `key` to `out`.
    fn entries(&self, key: u64, out: &mut Vec<StoreEntry>);
    /// Whether the source ran out of room (export fails `store-full`).
    fn is_full(&self) -> bool;
}

impl ProofSource for ProofStore {
    fn entries(&self, key: u64, out: &mut Vec<StoreEntry>) {
        out.extend_from_slice(self.get(key));
    }

    fn is_full(&self) -> bool {
        self.is_full()
    }
}

impl ProofSource for ProofTable {
    fn entries(&self, key: u64, out: &mut Vec<StoreEntry>) {
        let mut raw = Vec::new();
        self.entries(key, &mut raw);
        out.extend(raw.into_iter().filter(|e| e.mv != 0).map(|e| StoreEntry {
            mv: Move::from_bits(e.mv).expect("stored move bits decode"),
            length: e.length,
            limit: e.limit,
        }));
    }

    fn is_full(&self) -> bool {
        self.refused()
    }
}
