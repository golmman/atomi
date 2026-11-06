//! The M0 proof store and JSONL writer (D15).
//!
//! The store is a [`crate::events::ProofSink`] fed by the search; the writer
//! walks real positions from the root, chooses per (position, counter) the
//! shortest entry that holds at that counter, records position edges as the
//! checker does and rejects any cross-line cycle. It fails closed: on any
//! error nothing is written.

pub mod store;
pub mod write;

pub use store::{ProofStore, StoreEntry};
pub use write::{ExportError, WriteStats, write_proof};
