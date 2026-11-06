//! Neutral proof events decoupling the search from the proof store.
//!
//! The search is generic over a [`ProofSink`] and knows nothing about
//! `strategy`; the store (Phase D) is a `ProofSink` impl. Invariant: every
//! attacker proof the table ever stored was emitted with the same values, so
//! a sink holds an equal or dominating entry for it. Defender proofs are not
//! emitted — the writer re-derives them from their replies' attacker entries.

use atomic_movegen::types::Move;

/// A proof event emitted by the search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofEvent {
    /// An attacker node was proved with move `mv` and the given
    /// length/limit.
    AttackerProven {
        key: u64,
        mv: Move,
        length: u16,
        limit: u8,
    },
}

/// Consumer of proof events.
pub trait ProofSink {
    /// Record one event.
    fn proven(&mut self, event: ProofEvent);
}

/// No-op sink used when no proof is requested.
pub struct NullSink;

impl ProofSink for NullSink {
    fn proven(&mut self, _event: ProofEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_movegen::types::Square;

    #[test]
    fn null_sink_accepts_events() {
        let mut sink = NullSink;
        sink.proven(ProofEvent::AttackerProven {
            key: 1,
            mv: Move::make_move(Square::A1, Square::A2),
            length: 1,
            limit: 99,
        });
    }
}
