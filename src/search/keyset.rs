//! A bounded, deterministic set of position keys for the plan-12
//! attribution counters (`--stats`).
//!
//! The set is an open-addressing table with a fixed power-of-two capacity.
//! It never iterates, so it cannot affect the search trajectory; it only
//! answers "was this key expanded before?". When the load reaches 70 % the
//! set stops inserting and reports `capped`, so `len()` is a lower bound on
//! the true number of distinct keys. That is enough for the plan-12
//! re-expansion ratio at the 20 M-eval attribution point; the 120 s runs are
//! measured without `--stats`.

/// A fixed-capacity set of `u64` keys.
pub struct KeySet {
    slots: Vec<u64>,
    used: Vec<u8>,
    len: u64,
    cap: usize,
    capped: bool,
}

impl KeySet {
    /// A set with `cap` rounded up to a power of two (at least 2).
    pub fn new(cap: usize) -> KeySet {
        let cap = cap.max(2).next_power_of_two();
        KeySet {
            slots: vec![0; cap],
            used: vec![0; cap],
            len: 0,
            cap,
            capped: false,
        }
    }

    /// Insert `key`; returns `true` when it was not already present.
    pub fn insert(&mut self, key: u64) -> bool {
        let mask = self.cap - 1;
        let mut i = (hash(key) as usize) & mask;
        loop {
            if self.used[i] == 0 {
                if self.capped || self.len as usize >= self.cap * 7 / 10 {
                    self.capped = true;
                    return false;
                }
                self.slots[i] = key;
                self.used[i] = 1;
                self.len += 1;
                return true;
            }
            if self.slots[i] == key {
                return false;
            }
            i = (i + 1) & mask;
        }
    }

    /// The number of distinct keys inserted (a lower bound when `capped`).
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether the set stopped inserting at its load bound.
    pub fn capped(&self) -> bool {
        self.capped
    }
}

/// SplitMix64 finalizer: a cheap, well-distributed key hash.
fn hash(key: u64) -> u64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_distinct_keys() {
        let mut s = KeySet::new(16);
        assert!(s.insert(1));
        assert!(!s.insert(1));
        assert!(s.insert(2));
        assert_eq!(s.len(), 2);
        assert!(!s.capped());
    }

    #[test]
    fn caps_at_the_load_bound() {
        let mut s = KeySet::new(8);
        for k in 0..100u64 {
            s.insert(k);
        }
        assert!(s.capped());
        assert!(s.len() <= 8 * 7 / 10);
    }

    #[test]
    fn zero_key_is_a_key() {
        let mut s = KeySet::new(4);
        assert!(s.insert(0));
        assert!(!s.insert(0));
        assert_eq!(s.len(), 1);
    }
}
