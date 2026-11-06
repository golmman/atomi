//! M0 transposition table: deliberately simple.
//!
//! `--memory MB` is the table's size label: `floor(mb * 2^20 /
//! size_of::<Bucket>())` buckets of 4 entries, rounded down to a power of two
//! (at least 1), allocated eagerly (lazy zeroed allocation is M1). The bucket
//! index is the key's low bits.
//!
//! Store rules: proofs and open values are always stored; refutations only
//! when `cycle == ANY` (path-independent). A path-dependent refutation is a
//! no-op: an existing entry for the key stays as it was. Replacement prefers
//! the same key, then an empty slot, then the lowest-`work` non-proof entry,
//! then the lowest-`work` entry. Dropping a table proof is sound because the
//! output lives in the strategy store.

use crate::search::value::{ANY, Value};
use atomic_movegen::types::Move;

const ENTRIES_PER_BUCKET: usize = 4;

/// The stored payload of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Payload {
    /// A stored attacker proof.
    Proved {
        mv: Option<Move>,
        length: u16,
        limit: u8,
    },
    /// A stored path-independent refutation.
    Refuted { from: u8 },
    /// An open node (numbers live on the entry).
    Open,
}

/// One table entry.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// Full 64-bit position key.
    pub key: u64,
    /// Proof number (open entries).
    pub pn: u32,
    /// Disproof number (open entries).
    pub dn: u32,
    /// Expansions spent below the node during its last frame.
    pub work: u32,
    /// The stored value.
    pub payload: Payload,
}

type Bucket = [Option<Entry>; ENTRIES_PER_BUCKET];

/// The M0 transposition table.
pub struct Table {
    buckets: Vec<Bucket>,
    mask: usize,
}

impl Table {
    /// Allocate a table whose size label is `mb` MiB.
    pub fn new(mb: usize) -> Table {
        let raw = mb.saturating_mul(1 << 20) / std::mem::size_of::<Bucket>();
        // floor to a power of two (>= 1)
        let buckets = if raw >= 1 {
            1usize << (usize::BITS - 1 - raw.leading_zeros())
        } else {
            1
        };
        Table {
            buckets: vec![[None; ENTRIES_PER_BUCKET]; buckets],
            mask: buckets - 1,
        }
    }

    /// The number of buckets (a power of two).
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Look up `key`.
    pub fn probe(&self, key: u64) -> Option<&Entry> {
        let bucket = &self.buckets[key as usize & self.mask];
        bucket.iter().flatten().find(|e| e.key == key)
    }

    /// Store `value` under `key` with `work`, honouring the store and
    /// replacement rules in the module docs.
    pub fn store(&mut self, key: u64, value: &Value, work: u32) {
        let (payload, pn, dn) = match *value {
            Value::Proved { length, limit, mv } => (Payload::Proved { mv, length, limit }, 0, 0),
            Value::Refuted { cycle, from } => {
                if cycle != ANY {
                    return; // path-dependent: never stored
                }
                (Payload::Refuted { from }, 0, 0)
            }
            Value::Open { pn, dn } => (Payload::Open, pn, dn),
        };
        let bucket = &mut self.buckets[key as usize & self.mask];

        if let Some(slot) = bucket.iter_mut().flatten().find(|e| e.key == key) {
            *slot = Entry {
                key,
                pn,
                dn,
                work,
                payload,
            };
            return;
        }
        if let Some(slot) = bucket.iter_mut().find(|s| s.is_none()) {
            *slot = Some(Entry {
                key,
                pn,
                dn,
                work,
                payload,
            });
            return;
        }
        let is_proof = |e: &Entry| matches!(e.payload, Payload::Proved { .. });
        let idx = bucket
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, e)| !is_proof(e))
            .min_by_key(|(_, e)| e.work)
            .map(|(i, _)| i)
            .or_else(|| {
                bucket
                    .iter()
                    .flatten()
                    .enumerate()
                    .min_by_key(|(_, e)| e.work)
                    .map(|(i, _)| i)
            });
        let slot = idx.map(|i| bucket[i].as_mut().unwrap());
        if let Some(slot) = slot {
            *slot = Entry {
                key,
                pn,
                dn,
                work,
                payload,
            };
        }
    }
}

/// The value a probed entry gives at `counter`, if usable.
///
/// A proof is usable iff `counter <= limit`; a refutation iff
/// `counter >= from`; an open entry always. An unusable proof or refutation
/// gives `None` and the caller falls back to fresh numbers.
pub fn usable(entry: &Entry, counter: u16) -> Option<Value> {
    match entry.payload {
        Payload::Proved { mv, length, limit } => {
            (counter <= u16::from(limit)).then_some(Value::Proved { length, limit, mv })
        }
        Payload::Refuted { from } => {
            (counter >= u16::from(from)).then_some(Value::Refuted { cycle: ANY, from })
        }
        Payload::Open => Some(Value::Open {
            pn: entry.pn,
            dn: entry.dn,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::value::INFINITY;
    use atomic_movegen::types::Square;

    fn entry(key: u64, work: u32, payload: Payload) -> Entry {
        Entry {
            key,
            pn: 0,
            dn: 0,
            work,
            payload,
        }
    }

    #[test]
    fn probe_store_roundtrip() {
        let mut t = Table::new(1);
        let v = Value::Open { pn: 3, dn: 5 };
        t.store(42, &v, 10);
        let e = t.probe(42).unwrap();
        assert_eq!(e.payload, Payload::Open);
        assert_eq!((e.pn, e.dn, e.work), (3, 5, 10));
        assert!(t.probe(41).is_none());
        // Same key replaces.
        t.store(42, &Value::Open { pn: 1, dn: 2 }, 7);
        let e = t.probe(42).unwrap();
        assert_eq!((e.pn, e.dn, e.work), (1, 2, 7));
    }

    #[test]
    fn bucket_sizing() {
        let t = Table::new(16);
        assert!(t.bucket_count().is_power_of_two());
        assert!(t.bucket_count() * std::mem::size_of::<Bucket>() <= 16 << 20);
        assert!(t.bucket_count() * 2 * std::mem::size_of::<Bucket>() > 16 << 20);
        assert_eq!(Table::new(0).bucket_count(), 1);
    }

    #[test]
    fn usable_entry_rules() {
        let proved = entry(
            1,
            0,
            Payload::Proved {
                mv: None,
                length: 2,
                limit: 50,
            },
        );
        assert_eq!(
            usable(&proved, 50),
            Some(Value::Proved {
                length: 2,
                limit: 50,
                mv: None
            })
        );
        assert_eq!(usable(&proved, 51), None);
        let refuted = entry(1, 0, Payload::Refuted { from: 40 });
        assert_eq!(
            usable(&refuted, 40),
            Some(Value::Refuted {
                cycle: ANY,
                from: 40
            })
        );
        assert_eq!(usable(&refuted, 39), None);
        let open = Entry {
            pn: 3,
            dn: 5,
            ..entry(1, 0, Payload::Open)
        };
        assert_eq!(usable(&open, 0), Some(Value::Open { pn: 3, dn: 5 }));
    }

    #[test]
    fn path_dependent_refutation_not_stored() {
        let mut t = Table::new(1);
        t.store(7, &Value::Refuted { cycle: 3, from: 0 }, 5);
        assert!(t.probe(7).is_none());
        // An existing entry stays.
        t.store(7, &Value::Open { pn: 2, dn: 2 }, 1);
        t.store(7, &Value::Refuted { cycle: 3, from: 0 }, 9);
        assert!(matches!(t.probe(7).unwrap().payload, Payload::Open));
        // cycle == ANY is stored.
        t.store(
            8,
            &Value::Refuted {
                cycle: ANY,
                from: 30,
            },
            5,
        );
        assert_eq!(t.probe(8).unwrap().payload, Payload::Refuted { from: 30 });
    }

    #[test]
    fn replacement_keeps_proofs() {
        // Force every key into one bucket: keys share low bits.
        let mut t = Table::new(1);
        let mask = t.mask;
        let key = |i: u64| (i << 20) | mask as u64; // same bucket index
        let proof = |l: u16| Value::Proved {
            length: l,
            limit: 99,
            mv: None,
        };
        let mv = Move::make_move(Square::A1, Square::A2);
        for i in 0..4 {
            t.store(key(i), &proof(1), 10 + i as u32);
        }
        // Bucket full of proofs: a new open entry replaces the lowest-work proof.
        t.store(key(9), &Value::Open { pn: 1, dn: 1 }, 0);
        assert!(t.probe(key(0)).is_none());
        assert!(t.probe(key(9)).is_some());
        // Refill with opens and one proof; a new entry drops a non-proof first.
        let mut t = Table::new(1);
        let mask = t.mask;
        let key = |i: u64| (i << 20) | mask as u64;
        t.store(
            key(0),
            &Value::Proved {
                length: 1,
                limit: 99,
                mv: Some(mv),
            },
            1,
        );
        t.store(key(1), &Value::Open { pn: 1, dn: 1 }, 5);
        t.store(key(2), &Value::Open { pn: 1, dn: 1 }, 3);
        t.store(key(3), &Value::Open { pn: 1, dn: 1 }, 8);
        t.store(
            key(9),
            &Value::Refuted {
                cycle: ANY,
                from: 0,
            },
            2,
        );
        assert!(t.probe(key(0)).is_some(), "proof kept");
        assert!(t.probe(key(2)).is_none(), "lowest-work non-proof dropped");
        let e = t.probe(key(0)).unwrap();
        assert_eq!(
            e.payload,
            Payload::Proved {
                mv: Some(mv),
                length: 1,
                limit: 99
            }
        );
        let _ = INFINITY;
    }
}
