// Justification for >10 KB: the safety invariant (one normative
// description), the allocator, and the zeroed-value/equivalence tests that
// pin the invariant all live together; splitting them would duplicate the
// invariant statement.

// from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:src/search/tt/alloc.rs

//! Zero-initialized allocation for the in-search tables.
//!
//! The tables are by far the largest allocations in the program (the whole
//! `--memory` label) and the vast majority of their slots are never touched:
//! a mate-in-1 needs a handful, a deep search touches most of them. Writing
//! the memory up front (`vec![zero; n]`) forces the allocator to hand back
//! *committed* anonymous memory and to fault in every page before the search
//! starts — at 192 MB that is tens of milliseconds of process wall for a
//! search that itself runs in microseconds.
//!
//! [`Zeroed`] uses [`std::alloc::alloc_zeroed`] instead. For large requests
//! that call is served by fresh `mmap`ed zero pages on the shipped targets,
//! so construction costs microseconds and pages become resident only on
//! first touch: RSS is bounded by the label rather than equal to it.
//!
//! # The over-allocation (why not `alloc_zeroed(Layout::array::<T>(n))`)
//!
//! On the unix `System` allocator, `alloc_zeroed` reaches `calloc` (lazy
//! zero pages) only when the layout's alignment is at most `MIN_ALIGN`
//! (16); for over-aligned types — [`super::tt::Cluster`] is `align(64)` —
//! it falls back to `posix_memalign` plus an explicit `memset`, eagerly
//! committing the whole table. [`Zeroed::new`] therefore always requests a
//! **16-aligned** layout of `n * size_of::<T>() + align_of::<T>()` bytes,
//! rounds the returned base pointer up to `align_of::<T>()`, and stores the
//! base and layout for `Drop`. The slack guarantees the rounded pointer is
//! inside the allocation and the slice never runs past its end.
//!
//! # Huge pages
//!
//! On the Linux targets the freshly `mmap`ed range is advised as
//! transparent huge pages (`madvise(MADV_HUGEPAGE)` on the 2 MiB-aligned
//! interior; the measured host runs THP in `madvise` mode). The lazily
//! faulted table then uses 2 MiB pages, cutting TLB misses on the
//! TT/proof-table sweep — measured in plan2 M4
//! (`docs/plans/prover/measurements/plan2/ab.json`). RSS stays bounded by
//! the label but rounds up per touched 2 MiB page. The advice is a hint:
//! it fails harmlessly on kernels without THP and is a no-op elsewhere.
//!
//! # Why this is safe (the representation invariant)
//!
//! An all-zero byte pattern *is* a valid value of every allocated type, and
//! it reads as *not live* on the single liveness signal each type defines:
//!
//! - [`super::tt::Cluster`]: four all-zero slots, each `check == 0` — the
//!   empty-slot signal, because a stored check is `max(1, key >> 32)`, never
//!   zero;
//! - [`super::proofs::ProofSlot`]: `state == 0` — the free-slot signal;
//!   liveness is `state == 1` only;
//! - `u64` (a filter word): no bits set.
//!
//! Every field is an integer, which accepts any bit pattern, so an all-zero
//! value is always representable. Non-live slots' remaining fields are
//! unspecified and must never be observed; every read path filters on the
//! liveness signal alone. With that contract the lazy allocation is a
//! representation-only change: live-slot transitions are untouched, so
//! tables built lazily behave identically to eager `vec![zero; n]` storage
//! (`lazy_matches_eager` below).

use crate::table::proofs::ProofSlot;
use crate::table::tt::Cluster;
use std::alloc::Layout;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Advise the 2 MiB-aligned interior of `[base, base + size)` as
/// transparent huge pages (see the module docs). A no-op off the Linux
/// targets.
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
fn advise_huge_pages(base: *mut u8, size: usize) {
    unsafe extern "C" {
        fn madvise(addr: *mut u8, len: usize, advice: i32) -> i32;
    }
    const MADV_HUGEPAGE: i32 = 14;
    const HUGE: usize = 2 << 20;
    let start = base.align_offset(HUGE);
    if start < size {
        let len = (size - start) & !(HUGE - 1);
        if len > 0 {
            // SAFETY: `[base + start, base + start + len)` lies inside our
            // own live allocation (`base .. base + size`);
            // `madvise(MADV_HUGEPAGE)` only changes the kernel's paging
            // policy for it, never its contents or validity — the zero
            // pages stay zero. The return value is ignored: the advice is
            // a hint and fails harmlessly on kernels without THP.
            unsafe {
                madvise(base.add(start), len, MADV_HUGEPAGE);
            }
        }
    }
}

/// A no-op off the Linux targets.
#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
fn advise_huge_pages(_base: *mut u8, _size: usize) {}

/// A type whose all-zero byte pattern is a valid, not-live value (the module
/// docs state the liveness signal per type).
///
/// # Safety
///
/// Implementors must be plain-integer aggregates (any bit pattern valid) for
/// which the all-zero value is a well-formed *empty* element per the module
/// docs.
pub(crate) unsafe trait ZeroValid: Copy {}

// SAFETY: `Cluster` is `#[repr(C, align(64))]` over four `#[repr(C)]` slots
// of plain `u32`s; all-zero reads `check == 0`, the empty slot.
unsafe impl ZeroValid for Cluster {}
// SAFETY: `ProofSlot` is a `#[repr(C)]` plain-integer aggregate; all-zero
// reads `state == 0`, the free slot.
unsafe impl ZeroValid for ProofSlot {}
// SAFETY: a filter word of zeros means no bit set.
unsafe impl ZeroValid for u64 {}

/// An owning `n`-element, zero-initialized, lazily committed array of `T`.
/// Dereferences to `[T]`.
pub(crate) struct Zeroed<T: ZeroValid> {
    /// The `align_of::<T>()`-aligned start of the elements.
    data: NonNull<T>,
    /// The allocation's base pointer (16-aligned, pre-rounding).
    base: NonNull<u8>,
    /// Element count.
    len: usize,
    /// The layout `base` was allocated with; size 0 means no allocation.
    layout: Layout,
}

impl<T: ZeroValid> Zeroed<T> {
    /// Allocate `len` zero-initialized elements. For large sizes this is
    /// served by `mmap`ed zero pages: construction costs microseconds and
    /// pages become resident on first touch (see the module docs).
    ///
    /// # Panics
    ///
    /// Panics if the byte size overflows `usize` (unreachable memory).
    pub(crate) fn new(len: usize) -> Zeroed<T> {
        if len == 0 || std::mem::size_of::<T>() == 0 {
            return Zeroed {
                data: NonNull::dangling(),
                base: NonNull::dangling(),
                len: 0,
                layout: Layout::from_size_align(0, 1).unwrap(),
            };
        }
        let align = std::mem::align_of::<T>();
        let size = len
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|s| s.checked_add(align))
            .expect("table allocation layout overflows usize");
        // A 16-aligned request always takes the calloc path (see the module
        // docs); the +align slack covers the pointer rounding below.
        let layout = Layout::from_size_align(size, 16).unwrap();
        // SAFETY: `alloc_zeroed` returns `layout.size()` zeroed bytes (or
        // null, reported via `handle_alloc_error`). All-zero bytes form a
        // valid `T` that reads as not live (the module-docs invariant).
        // `data` rounds the base up inside the allocation: at most
        // `align - 1` bytes are skipped and `len * size_of::<T>()` bytes
        // remain, so `[T; len]` is in bounds. `Zeroed` is the sole owner and
        // `Drop` deallocates `base` with the same layout.
        unsafe {
            let base = std::alloc::alloc_zeroed(layout);
            if base.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            advise_huge_pages(base, layout.size());
            let offset = base.align_offset(align);
            Zeroed {
                data: NonNull::new(base.add(offset).cast::<T>()).unwrap(),
                base: NonNull::new_unchecked(base),
                len,
                layout,
            }
        }
    }

    /// The same allocation, but with the pages actually written (test-only:
    /// the eager side of the lazy-vs-eager equivalence).
    #[cfg(test)]
    pub(crate) fn eager(len: usize) -> Zeroed<T> {
        let z = Zeroed::new(len);
        if len > 0 {
            // SAFETY: `data` points at `len` valid elements inside the
            // allocation (see `new`).
            unsafe {
                std::ptr::write_bytes(z.data.as_ptr(), 0, len);
            }
        }
        z
    }
}

impl<T: ZeroValid> Deref for Zeroed<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        // SAFETY: `data` points at `len` properly aligned, initialized `T`s
        // owned by `self` for the borrow's lifetime.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.len) }
    }
}

impl<T: ZeroValid> DerefMut for Zeroed<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        // SAFETY: as `deref`; `self` is the sole owner, so `&mut self`
        // guarantees exclusivity.
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr(), self.len) }
    }
}

impl<T: ZeroValid> Drop for Zeroed<T> {
    fn drop(&mut self) {
        if self.layout.size() > 0 {
            // SAFETY: `base`/`layout` are the exact pointer and layout from
            // `new`'s `alloc_zeroed`, and `self` is the sole owner.
            unsafe {
                std::alloc::dealloc(self.base.as_ptr(), self.layout);
            }
        }
    }
}

// SAFETY: `Zeroed<T>` owns its buffer exclusively, so it can be sent/shared
// exactly when `[T]` can.
unsafe impl<T: ZeroValid + Send> Send for Zeroed<T> {}
// SAFETY: as above.
unsafe impl<T: ZeroValid + Sync> Sync for Zeroed<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::value::Value;

    #[test]
    fn data_pointer_is_aligned() {
        for n in [1usize, 3, 7, 1024, 1 << 16] {
            let z = Zeroed::<Cluster>::new(n);
            assert_eq!(z.as_ptr() as usize % 64, 0, "n={n}");
        }
        let z = Zeroed::<ProofSlot>::new(5);
        assert_eq!(z.as_ptr() as usize % std::mem::align_of::<ProofSlot>(), 0);
    }

    #[test]
    fn zeroed_cluster_reads_empty() {
        for c in Zeroed::<Cluster>::new(7).iter() {
            for s in &c.0 {
                assert_eq!(s.check, 0, "a zeroed slot must read empty");
            }
        }
    }

    #[test]
    fn zeroed_proof_slot_reads_free() {
        for s in Zeroed::<ProofSlot>::new(7).iter() {
            assert_eq!(s.state, 0, "a zeroed slot must read free");
        }
    }

    #[test]
    fn zero_is_empty() {
        let z = Zeroed::<Cluster>::new(0);
        assert!(z.is_empty());
    }

    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    #[test]
    fn huge_page_advice_keeps_contents() {
        // ≥ 8 MiB so the advised interior is non-empty; the hint must not
        // disturb contents: all-empty slots, 64 B-aligned data.
        let n = (8 << 20) / std::mem::size_of::<Cluster>();
        let z = Zeroed::<Cluster>::new(n);
        assert_eq!(z.as_ptr() as usize % 64, 0);
        assert!(z.iter().all(|c| c.0.iter().all(|s| s.check == 0)));
    }

    #[test]
    fn lengths_match_request() {
        for n in [1usize, 3, 7, 1024] {
            assert_eq!(Zeroed::<Cluster>::new(n).len(), n);
            assert_eq!(Zeroed::<ProofSlot>::new(n).len(), n);
        }
    }

    #[test]
    fn lazy_matches_eager() {
        // Representation-only equivalence: a `Tables` over lazily zeroed
        // storage and one over eagerly written storage give identical
        // lookups after the same store sequence (stores, evictions, a proof
        // clear and filter bits included).
        use crate::table::tt::Tt;
        use crate::table::{Filter, ProofTable, Tables};
        let mut lazy = Tables::from_parts(Tt::new(64), ProofTable::new(64), Filter::new(1 << 10));
        let mut eager =
            Tables::from_parts(Tt::eager(64), ProofTable::eager(64), Filter::eager(1 << 10));
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let keys: Vec<u64> = (0..200).map(|_| next()).collect();
        for (i, k) in keys.iter().enumerate() {
            for t in [&mut lazy, &mut eager] {
                match i % 4 {
                    0 => t.store(
                        *k,
                        &Value::Proved {
                            length: 1 + (i % 5) as u16,
                            limit: 99,
                            mv: None,
                        },
                        i as u32,
                    ),
                    1 => t.store(
                        *k,
                        &Value::Refuted {
                            cycle: crate::search::value::ANY,
                            from: (i % 50) as u8,
                        },
                        i as u32,
                    ),
                    _ => t.store(
                        *k,
                        &Value::Open {
                            pn: (i % 7 + 1) as u32,
                            dn: (i % 5 + 1) as u32,
                        },
                        i as u32,
                    ),
                }
            }
        }
        for k in &keys {
            for c in [0u16, 30, 99] {
                assert_eq!(
                    lazy.lookup(*k, c),
                    eager.lookup(*k, c),
                    "key {k} counter {c}"
                );
            }
        }
    }
}
