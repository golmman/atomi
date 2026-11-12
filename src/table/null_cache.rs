//! The plan-26 exposed-limit cache (NULL-V2).
//!
//! `exposed_limit` is a pure function of the child board: it asks whether
//! every legal defender reply lets the attacker explode the defender's
//! commoner, and returns the highest 50-move counter at which that holds.
//! The null replay calls it once per null move and the same child recurs
//! often, so a small direct-mapped cache removes the repeats.
//!
//! The key is `(child key, rule50, attacker)`. Both a positive answer (the
//! limit) and a negative answer are exact, so both are stored. A hit is
//! accepted only after a full-key and full-meta compare; a collision simply
//! recomputes, exactly like a TT collision. The cache is board-only: it
//! never stores a null position, a proof or a table-dependent value.
//!
//! Storage is three parallel arrays (`u64` key, `u8` meta, `u8` value) so an
//! entry costs 10 bytes rather than the 16 a padded struct would take. The
//! arrays are lazily zeroed like the other tables; `value == 0` is the only
//! liveness signal.

use crate::table::alloc::Zeroed;
use atomic_movegen::types::Color;

/// The default entry count at the 192 MB label (2^17 entries, 1.25 MiB).
///
/// Plan-26 Phase A swept 2^17/2^18/2^20/2^21. The larger caches need TT
/// compensation, which changes the search trajectory; 2^17 fits in the
/// filter's slack, so the TT and proof table keep their sizes and the tree
/// is identical to the control (see report26). The request is only a cap:
/// [`Budget::cache_entries`](crate::table::Budget) is the size actually
/// allocated after the label split.
pub const DEFAULT_ENTRIES: usize = 1 << 17;

/// Bytes per entry: `u64` key + `u8` meta + `u8` value.
pub const ENTRY_BYTES: usize = 10;

const EMPTY: u8 = 0;
const NEGATIVE: u8 = 1;
const POSITIVE_BASE: u8 = 2;

/// A cached `exposed_limit` answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachedLimit {
    /// Not every defence exposes the commoner.
    Negative,
    /// Every defence exposes it; the proof holds up to this counter.
    Positive(u8),
}

/// The direct-mapped exposed-limit cache.
pub struct NullCache {
    keys: Zeroed<u64>,
    meta: Zeroed<u8>,
    values: Zeroed<u8>,
    mask: usize,
    hits: u64,
    misses: u64,
}

impl NullCache {
    /// A cache of `entries` slots (rounded down to a power of two; `0`
    /// disables it).
    pub fn new(entries: usize) -> NullCache {
        let entries = if entries == 0 {
            0
        } else {
            1usize << (usize::BITS - 1 - entries.leading_zeros())
        };
        NullCache {
            keys: Zeroed::new(entries),
            meta: Zeroed::new(entries),
            values: Zeroed::new(entries),
            mask: entries.saturating_sub(1),
            hits: 0,
            misses: 0,
        }
    }

    /// The number of slots.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether the cache is disabled.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Cache hits so far.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Cache misses so far.
    pub fn misses(&self) -> u64 {
        self.misses
    }

    fn index(&self, key: u64, meta: u8) -> usize {
        let h = key.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ u64::from(meta).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        (h >> 32) as usize & self.mask
    }

    fn encode(rule50: u8, attacker: Color) -> u8 {
        (rule50 << 1) | u8::from(attacker == Color::Black)
    }

    /// Look up `(key, rule50, attacker)`. A hit is exact: the full key and
    /// meta must match.
    pub fn get(&mut self, key: u64, rule50: u8, attacker: Color) -> Option<CachedLimit> {
        if self.is_empty() {
            return None;
        }
        let meta = Self::encode(rule50, attacker);
        let idx = self.index(key, meta);
        if self.values[idx] != EMPTY && self.keys[idx] == key && self.meta[idx] == meta {
            self.hits += 1;
            let value = self.values[idx];
            return Some(if value == NEGATIVE {
                CachedLimit::Negative
            } else {
                CachedLimit::Positive(value - POSITIVE_BASE)
            });
        }
        self.misses += 1;
        None
    }

    /// Store an exact answer for `(key, rule50, attacker)`.
    pub fn insert(&mut self, key: u64, rule50: u8, attacker: Color, limit: Option<u8>) {
        if self.is_empty() {
            return;
        }
        let meta = Self::encode(rule50, attacker);
        let idx = self.index(key, meta);
        self.keys[idx] = key;
        self.meta[idx] = meta;
        self.values[idx] = match limit {
            None => NEGATIVE,
            Some(limit) => POSITIVE_BASE + limit,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_and_negative_round_trip() {
        let mut c = NullCache::new(1 << 10);
        assert_eq!(c.get(7, 3, Color::White), None);
        c.insert(7, 3, Color::White, Some(42));
        assert_eq!(c.get(7, 3, Color::White), Some(CachedLimit::Positive(42)));
        c.insert(7, 3, Color::White, None);
        assert_eq!(c.get(7, 3, Color::White), Some(CachedLimit::Negative));
        assert_eq!(c.hits(), 2);
        assert_eq!(c.misses(), 1);
    }

    #[test]
    fn key_meta_and_attacker_are_verified() {
        let mut c = NullCache::new(1 << 10);
        c.insert(7, 3, Color::White, Some(42));
        assert_eq!(c.get(7, 4, Color::White), None);
        assert_eq!(c.get(7, 3, Color::Black), None);
        assert_eq!(c.get(8, 3, Color::White), None);
    }

    #[test]
    fn disabled_cache_is_inert() {
        let mut c = NullCache::new(0);
        assert!(c.is_empty());
        c.insert(1, 0, Color::White, Some(1));
        assert_eq!(c.get(1, 0, Color::White), None);
        assert_eq!((c.hits(), c.misses()), (0, 0));
    }

    #[test]
    fn default_entries_is_the_measured_size() {
        assert_eq!(DEFAULT_ENTRIES, 1 << 17);
        assert_eq!(DEFAULT_ENTRIES * ENTRY_BYTES, 1_310_720);
    }
}
