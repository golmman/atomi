//! One-hash bit filter: "this key may be stored somewhere".
//!
//! A bit array of `filter_bits` (a power of two). The bit index is
//! `check(key) & (bits - 1)` ([`super::tt::check`], the same partial key the
//! TT stores). Every TT store and every proof-table insert sets the key's
//! bit; a lookup skips the tables entirely when the bit is clear.
//!
//! The filter is exact in the negative direction — no false negatives,
//! because evicted keys leave their bits set — so it changes no result.
//!
//! Once more than a quarter of the bits are set the filter is dropped
//! permanently: `may_hold` is then always `true` and `set` is a no-op. There
//! is no rebuild in M1.

use crate::table::alloc::Zeroed;
use crate::table::prefetch;
use crate::table::tt::check;

/// The "positions possibly stored" bit filter.
pub struct Filter {
    words: Zeroed<u64>,
    /// `bits - 1` (bits is a power of two; `usize::MAX` when disabled).
    mask: usize,
    /// Bits set so far (counted as they flip).
    set: usize,
    /// Permanently dropped (> 1/4 full, or disabled at construction).
    dropped: bool,
}

impl Filter {
    /// A filter of `bits` bits (`bits` a power of two), lazily zeroed.
    pub fn new(bits: usize) -> Filter {
        debug_assert!(bits.is_power_of_two());
        Filter {
            words: Zeroed::new(bits / 64),
            mask: bits - 1,
            set: 0,
            dropped: false,
        }
    }

    /// A disabled filter: `may_hold` is always `true` (the `Limits.filter =
    /// false` test hook).
    pub fn disabled() -> Filter {
        Filter {
            words: Zeroed::new(0),
            mask: usize::MAX,
            set: 0,
            dropped: true,
        }
    }

    /// A filter on eagerly written storage (test-only: [`Zeroed`]'s
    /// representation-only equivalence).
    #[cfg(test)]
    pub fn eager(bits: usize) -> Filter {
        debug_assert!(bits.is_power_of_two());
        Filter {
            words: Zeroed::eager(bits / 64),
            mask: bits - 1,
            set: 0,
            dropped: false,
        }
    }

    /// Whether the filter is still answering (enabled and not yet dropped).
    pub fn is_active(&self) -> bool {
        !self.dropped
    }

    fn index(&self, key: u64) -> usize {
        check(key) as usize & self.mask
    }

    /// Set `key`'s bit. Counts bits as they flip and drops the filter
    /// permanently once more than a quarter are set.
    pub fn set(&mut self, key: u64) {
        if self.dropped {
            return;
        }
        let idx = self.index(key);
        let (word, bit) = (idx / 64, idx % 64);
        if self.words[word] >> bit & 1 == 0 {
            self.words[word] |= 1 << bit;
            self.set += 1;
            if self.set > (self.mask + 1) / 4 {
                self.dropped = true;
            }
        }
    }

    /// `true` when the tables may hold an entry for `key` (always `true`
    /// after the filter is dropped or disabled).
    pub fn may_hold(&self, key: u64) -> bool {
        if self.dropped {
            return true;
        }
        let idx = self.index(key);
        self.words[idx / 64] >> (idx % 64) & 1 == 1
    }

    /// Prefetch the word holding `key`'s bit.
    pub fn prefetch_word(&self, key: u64) {
        if self.words.is_empty() {
            return;
        }
        prefetch(&self.words[self.index(key) / 64]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_with_check(c: u64) -> u64 {
        c << 32
    }

    #[test]
    fn no_false_negatives() {
        let mut f = Filter::new(1 << 16);
        // 100k inserted keys over 64k bits: every set key must read
        // may_hold, and (after the drop) may_hold stays true.
        let mut seed = 0xDEAD_BEEFu64;
        for i in 0..100_000u64 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key = seed;
            f.set(key);
            assert!(f.may_hold(key), "key {i}");
        }
        assert!(f.may_hold(u64::MAX));
        assert!(f.may_hold(0));
    }

    #[test]
    fn unset_keys_read_false_before_drop() {
        let mut f = Filter::new(1 << 20);
        f.set(key_with_check(7));
        assert!(f.may_hold(key_with_check(7)));
        assert!(!f.may_hold(key_with_check(8)));
        // Same check, different low bits → same bit (partial-key filter).
        f.set(key_with_check(9) | 0xFFFF);
        assert!(f.may_hold(key_with_check(9)));
    }

    #[test]
    fn drops_at_quarter_fill() {
        let bits = 1 << 12; // 4096 bits, drops at > 1024 set
        let mut f = Filter::new(bits);
        for i in 1..=1024u64 {
            f.set(key_with_check(i));
            assert!(f.is_active(), "dropped early at {i}");
        }
        f.set(key_with_check(1025));
        assert!(!f.is_active());
        assert!(f.may_hold(key_with_check(999_999)));
        // set is a no-op after the drop.
        f.set(key_with_check(2000));
        assert!(!f.is_active());
    }

    #[test]
    fn disabled_always_holds() {
        let mut f = Filter::disabled();
        assert!(f.may_hold(123));
        f.set(123);
        assert!(f.may_hold(123));
        assert!(!f.is_active());
        f.prefetch_word(123); // no-op, must not panic
    }
}
