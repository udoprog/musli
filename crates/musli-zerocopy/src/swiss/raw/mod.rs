#![allow(clippy::manual_map)]

#[macro_use]
mod macros;

cfg_if! {
    // Use the SSE2 implementation if possible: it allows us to scan 16 buckets
    // at once instead of 8. We don't bother with AVX since it would require
    // runtime dispatch and wouldn't gain us much anyways: the probability of
    // finding a match drops off drastically after the first few buckets.
    //
    // I attempted an implementation on ARM using NEON instructions, but it
    // turns out that most NEON instructions have multi-cycle latency, which in
    // the end outweighs any gains over the generic implementation.
    if #[cfg(all(
        target_feature = "sse2",
        any(target_arch = "x86", target_arch = "x86_64"),
        not(miri)
    ))] {
        mod sse2;
        use sse2 as imp;
    } else if #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] {
        mod neon;
        use neon as imp;
    } else {
        mod generic;
        use generic as imp;
    }
}

mod bitmask;

use self::bitmask::BitMaskIter;
pub(crate) use self::imp::Group;

use core::alloc::Layout;
use core::convert::{identity as likely, identity as unlikely};
use core::hint;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;
use core::slice;

use alloc::alloc;

#[inline(always)]
fn invalid_mut<T>(addr: usize) -> *mut T {
    // Strict provenance "magic".
    addr as *mut T
}

/// A raw swizz table.
pub struct RawTable<T> {
    table: RawTableInner,
    // Tell dropck that we own instances of T.
    _marker: PhantomData<T>,
}

impl<T> RawTable<T> {
    const TABLE_LAYOUT: TableLayout = TableLayout::new::<T>();

    /// Allocates a new hash table using the given allocator, with at least enough capacity for
    /// inserting the given number of elements without reallocating.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            table: RawTableInner::with_capacity(Self::TABLE_LAYOUT, capacity),
            _marker: PhantomData,
        }
    }

    /// Return section of control bytes.
    pub fn control_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.table.ctrl.as_ptr(), self.table.num_ctrl_bytes()) }
    }

    /// Export bucket mask.
    pub fn bucket_mask(&self) -> usize {
        self.table.bucket_mask
    }

    /// Returns the number of buckets in the table.
    #[inline]
    pub fn buckets(&self) -> usize {
        self.table.bucket_mask + 1
    }

    /// Insert the given value into the table.
    pub fn insert(&mut self, hash: u64, value: T) -> Bucket<T> {
        unsafe {
            // SAFETY:
            // 1. The [`RawTableInner`] must already have properly initialized control bytes since
            //    we will never expose `RawTable::new_uninitialized` in a public API.
            let slot = self.table.find_insert_slot(hash);

            // We can avoid growing the table once we have reached our load factor if we are replacing
            // a tombstone. This works since the number of EMPTY slots does not change in this case.
            //
            // SAFETY: The function is guaranteed to return [`InsertSlot`] that contains an index
            // in the range `0..=self.buckets()`.
            let old_ctrl = *self.table.ctrl(slot.index);

            if unlikely(self.table.growth_left == 0 && special_is_empty(old_ctrl)) {
                panic!("Table out of allocated capacity");
            }

            self.insert_in_slot(hash, slot, value)
        }
    }

    /// Inserts a new element into the table in the given slot, and returns its
    /// raw bucket.
    ///
    /// # Safety
    ///
    /// `slot` must point to a slot previously returned by
    /// `find_or_find_insert_slot`, and no mutation of the table must have
    /// occurred since that call.
    #[inline]
    pub unsafe fn insert_in_slot(&mut self, hash: u64, slot: InsertSlot, value: T) -> Bucket<T> {
        let old_ctrl = *self.table.ctrl(slot.index);
        self.table.record_item_insert_at(slot.index, old_ctrl, hash);

        let bucket = self.bucket(slot.index);
        bucket.write(value);
        bucket
    }

    /// Returns a pointer to an element in the table.
    ///
    /// The caller must ensure that the `RawTable` outlives the returned [`Bucket<T>`],
    /// otherwise using it may result in [`undefined behavior`].
    ///
    /// # Safety
    ///
    /// If `mem::size_of::<T>() != 0`, then the caller of this function must observe the
    /// following safety rules:
    ///
    /// * The table must already be allocated;
    ///
    /// * The `index` must not be greater than the number returned by the [`RawTable::buckets`]
    ///   function, i.e. `(index + 1) <= self.buckets()`.
    ///
    /// It is safe to call this function with index of zero (`index == 0`) on a table that has
    /// not been allocated, but using the returned [`Bucket`] results in [`undefined behavior`].
    ///
    /// If `mem::size_of::<T>() == 0`, then the only requirement is that the `index` must
    /// not be greater than the number returned by the [`RawTable::buckets`] function, i.e.
    /// `(index + 1) <= self.buckets()`.
    ///
    /// [`RawTable::buckets`]: RawTable::buckets
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn bucket(&self, index: usize) -> Bucket<T> {
        // If mem::size_of::<T>() != 0 then return a pointer to the `element` in the `data part` of the table
        // (we start counting from "0", so that in the expression T[n], the "n" index actually one less than
        // the "buckets" number of our `RawTable`, i.e. "n = RawTable::buckets() - 1"):
        //
        //           `table.bucket(3).as_ptr()` returns a pointer that points here in the `data`
        //           part of the `RawTable`, i.e. to the start of T3 (see `Bucket::as_ptr`)
        //                  |
        //                  |               `base = self.data_end()` points here
        //                  |               (to the start of CT0 or to the end of T0)
        //                  v                 v
        // [Pad], T_n, ..., |T3|, T2, T1, T0, |CT0, CT1, CT2, CT3, ..., CT_n, CTa_0, CTa_1, ..., CTa_m
        //                     ^                                              \__________  __________/
        //        `table.bucket(3)` returns a pointer that points                        \/
        //         here in the `data` part of the `RawTable` (to              additional control bytes
        //         the end of T3)                                              `m = Group::WIDTH - 1`
        //
        // where: T0...T_n  - our stored data;
        //        CT0...CT_n - control bytes or metadata for `data`;
        //        CTa_0...CTa_m - additional control bytes (so that the search with loading `Group` bytes from
        //                        the heap works properly, even if the result of `h1(hash) & self.table.bucket_mask`
        //                        is equal to `self.table.bucket_mask`). See also `RawTableInner::set_ctrl` function.
        //
        // P.S. `h1(hash) & self.table.bucket_mask` is the same as `hash as usize % self.buckets()` because the number
        // of buckets is a power of two, and `self.table.bucket_mask = self.buckets() - 1`.
        debug_assert_ne!(self.table.bucket_mask, 0);
        debug_assert!(index < self.table.buckets());
        Bucket::from_base_index(self.table.data_end(), index)
    }

    /// Iterate over buckets in the collection.
    pub(crate) unsafe fn iter(&self) -> RawIter<T> {
        self.table.iter()
    }
}

// SAFETY: RawTable<T> holds T, so is Sync if T is Sync.
unsafe impl<T> Send for RawTable<T> where T: Send {}
// SAFETY: RawTable<T> holds T, so is Sync if T is Sync.
unsafe impl<T> Sync for RawTable<T> where T: Sync {}

/// A reference to a hash table bucket containing a `T`.
///
/// This is usually just a pointer to the element itself. However if the element
/// is a ZST, then we instead track the index of the element in the table so
/// that `erase` works properly.
pub struct Bucket<T> {
    /// The index of the bucket.
    index: usize,
    // Actually it is pointer to next element than element itself
    // this is needed to maintain pointer arithmetic invariants
    // keeping direct pointer to element introduces difficulty.
    // Using `NonNull` for variance and niche layout
    ptr: NonNull<T>,
}

impl<T> Bucket<T> {
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    /// Creates a [`Bucket`] that contain pointer to the data.
    /// The pointer calculation is performed by calculating the
    /// offset from given `base` pointer (convenience for
    /// `base.as_ptr().sub(index)`).
    ///
    /// `index` is in units of `T`; e.g., an `index` of 3 represents a pointer
    /// offset of `3 * size_of::<T>()` bytes.
    ///
    /// If the `T` is a ZST, then we instead track the index of the element
    /// in the table so that `erase` works properly (return
    /// `NonNull::new_unchecked((index + 1) as *mut T)`)
    ///
    /// # Safety
    ///
    /// If `mem::size_of::<T>() != 0`, then the safety rules are directly derived
    /// from the safety rules for [`<*mut T>::sub`] method of `*mut T` and the safety
    /// rules of [`NonNull::new_unchecked`] function.
    ///
    /// Thus, in order to uphold the safety contracts for the [`<*mut T>::sub`] method
    /// and [`NonNull::new_unchecked`] function, as well as for the correct
    /// logic of the work of this crate, the following rules are necessary and
    /// sufficient:
    ///
    /// * the `base` pointer must not be `dangling` and must points to the
    ///   end of the first `value element` from the `data part` of the table, i.e.
    ///   must be the pointer that returned by [`RawTable::data_end`] or by
    ///   [`RawTableInner::data_end<T>`];
    ///
    /// * `index` must not be greater than `RawTableInner.bucket_mask`, i.e.
    ///   `index <= RawTableInner.bucket_mask` or, in other words, `(index + 1)`
    ///   must be no greater than the number returned by the function
    ///   [`RawTable::buckets`] or [`RawTableInner::buckets`].
    ///
    /// If `mem::size_of::<T>() == 0`, then the only requirement is that the
    /// `index` must not be greater than `RawTableInner.bucket_mask`, i.e.
    /// `index <= RawTableInner.bucket_mask` or, in other words, `(index + 1)`
    /// must be no greater than the number returned by the function
    /// [`RawTable::buckets`] or [`RawTableInner::buckets`].
    ///
    /// [`Bucket`]: crate::raw::Bucket
    /// [`<*mut T>::sub`]: https://doc.rust-lang.org/core/primitive.pointer.html#method.sub-1
    /// [`NonNull::new_unchecked`]: https://doc.rust-lang.org/stable/std/ptr/struct.NonNull.html#method.new_unchecked
    /// [`RawTable::data_end`]: crate::raw::RawTable::data_end
    /// [`RawTableInner::data_end<T>`]: RawTableInner::data_end<T>
    /// [`RawTable::buckets`]: crate::raw::RawTable::buckets
    /// [`RawTableInner::buckets`]: RawTableInner::buckets
    #[inline]
    unsafe fn from_base_index(base: NonNull<T>, index: usize) -> Self {
        // If mem::size_of::<T>() != 0 then return a pointer to an `element` in
        // the data part of the table (we start counting from "0", so that
        // in the expression T[last], the "last" index actually one less than the
        // "buckets" number in the table, i.e. "last = RawTableInner.bucket_mask"):
        //
        //                   `from_base_index(base, 1).as_ptr()` returns a pointer that
        //                   points here in the data part of the table
        //                   (to the start of T1)
        //                        |
        //                        |        `base: NonNull<T>` must point here
        //                        |         (to the end of T0 or to the start of C0)
        //                        v         v
        // [Padding], Tlast, ..., |T1|, T0, |C0, C1, ..., Clast
        //                           ^
        //                           `from_base_index(base, 1)` returns a pointer
        //                           that points here in the data part of the table
        //                           (to the end of T1)
        //
        // where: T0...Tlast - our stored data; C0...Clast - control bytes
        // or metadata for data.
        let ptr = if mem::size_of::<T>() == 0 {
            // won't overflow because index must be less than length (bucket_mask)
            // and bucket_mask is guaranteed to be less than `isize::MAX`
            // (see TableLayout::calculate_layout_for method)
            invalid_mut(index + 1)
        } else {
            base.as_ptr().sub(index)
        };

        Self {
            index,
            ptr: NonNull::new_unchecked(ptr),
        }
    }

    /// Overwrites a memory location with the given `value` without reading
    /// or dropping the old value (like [`ptr::write`] function).
    ///
    /// # Safety
    ///
    /// See [`ptr::write`] for safety concerns.
    ///
    /// # Note
    ///
    /// [`Hash`] and [`Eq`] on the new `T` value and its borrowed form *must* match
    /// those for the old `T` value, as the map will not re-evaluate where the new
    /// value should go, meaning the value may become "lost" if their location
    /// does not reflect their state.
    ///
    /// [`ptr::write`]: https://doc.rust-lang.org/core/ptr/fn.write.html
    /// [`Hash`]: https://doc.rust-lang.org/core/hash/trait.Hash.html
    /// [`Eq`]: https://doc.rust-lang.org/core/cmp/trait.Eq.html
    #[inline]
    pub(crate) unsafe fn write(&self, val: T) {
        self.as_ptr().write(val);
    }

    /// Acquires the underlying raw pointer `*mut T` to `data`.
    ///
    /// # Note
    ///
    /// If `T` is not [`Copy`], do not use `*mut T` methods that can cause calling the
    /// destructor of `T` (for example the [`<*mut T>::drop_in_place`] method), because
    /// for properly dropping the data we also need to clear `data` control bytes. If we
    /// drop data, but do not clear `data control byte` it leads to double drop when
    /// [`RawTable`] goes out of scope.
    ///
    /// If you modify an already initialized `value`, so [`Hash`] and [`Eq`] on the new
    /// `T` value and its borrowed form *must* match those for the old `T` value, as the map
    /// will not re-evaluate where the new value should go, meaning the value may become
    /// "lost" if their location does not reflect their state.
    ///
    /// [`RawTable`]: crate::raw::RawTable
    /// [`<*mut T>::drop_in_place`]: https://doc.rust-lang.org/core/primitive.pointer.html#method.drop_in_place
    /// [`Hash`]: https://doc.rust-lang.org/core/hash/trait.Hash.html
    /// [`Eq`]: https://doc.rust-lang.org/core/cmp/trait.Eq.html
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "raw")]
    /// # fn test() {
    /// use core::hash::{BuildHasher, Hash};
    /// use hashbrown::raw::{Bucket, RawTable};
    ///
    /// type NewHashBuilder = core::hash::BuildHasherDefault<ahash::AHasher>;
    ///
    /// fn make_hash<K: Hash + ?Sized, S: BuildHasher>(hash_builder: &S, key: &K) -> u64 {
    ///     use core::hash::Hasher;
    ///     let mut state = hash_builder.build_hasher();
    ///     key.hash(&mut state);
    ///     state.finish()
    /// }
    ///
    /// let hash_builder = NewHashBuilder::default();
    /// let mut table = RawTable::new();
    ///
    /// let value = ("a", 100);
    /// let hash = make_hash(&hash_builder, &value.0);
    ///
    /// table.insert(hash, value.clone(), |val| make_hash(&hash_builder, &val.0));
    ///
    /// let bucket: Bucket<(&str, i32)> = table.find(hash, |(k1, _)| k1 == &value.0).unwrap();
    ///
    /// assert_eq!(unsafe { &*bucket.as_ptr() }, &("a", 100));
    /// # }
    /// # fn main() {
    /// #     #[cfg(feature = "raw")]
    /// #     test()
    /// # }
    /// ```
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        if mem::size_of::<T>() == 0 {
            // Just return an arbitrary ZST pointer which is properly aligned
            // invalid pointer is good enough for ZST
            invalid_mut(mem::align_of::<T>())
        } else {
            unsafe { self.ptr.as_ptr().sub(1) }
        }
    }

    /// Executes the destructor (if any) of the pointed-to `data`.
    ///
    /// # Safety
    ///
    /// See [`ptr::drop_in_place`] for safety concerns.
    ///
    /// You should use [`RawTable::erase`] instead of this function,
    /// or be careful with calling this function directly, because for
    /// properly dropping the data we need also clear `data` control bytes.
    /// If we drop data, but do not erase `data control byte` it leads to
    /// double drop when [`RawTable`] goes out of scope.
    ///
    /// [`ptr::drop_in_place`]: https://doc.rust-lang.org/core/ptr/fn.drop_in_place.html
    /// [`RawTable`]: crate::raw::RawTable
    /// [`RawTable::erase`]: crate::raw::RawTable::erase
    pub(crate) unsafe fn drop(&self) {
        self.as_ptr().drop_in_place();
    }

    /// Create a new [`Bucket`] that is offset from the `self` by the given
    /// `offset`. The pointer calculation is performed by calculating the
    /// offset from `self` pointer (convenience for `self.ptr.as_ptr().sub(offset)`).
    /// This function is used for iterators.
    ///
    /// `offset` is in units of `T`; e.g., a `offset` of 3 represents a pointer
    /// offset of `3 * size_of::<T>()` bytes.
    ///
    /// # Safety
    ///
    /// If `mem::size_of::<T>() != 0`, then the safety rules are directly derived
    /// from the safety rules for [`<*mut T>::sub`] method of `*mut T` and safety
    /// rules of [`NonNull::new_unchecked`] function.
    ///
    /// Thus, in order to uphold the safety contracts for [`<*mut T>::sub`] method
    /// and [`NonNull::new_unchecked`] function, as well as for the correct
    /// logic of the work of this crate, the following rules are necessary and
    /// sufficient:
    ///
    /// * `self` contained pointer must not be `dangling`;
    ///
    /// * `self.to_base_index() + ofset` must not be greater than `RawTableInner.bucket_mask`,
    ///   i.e. `(self.to_base_index() + ofset) <= RawTableInner.bucket_mask` or, in other
    ///   words, `self.to_base_index() + ofset + 1` must be no greater than the number returned
    ///   by the function [`RawTable::buckets`] or [`RawTableInner::buckets`].
    ///
    /// If `mem::size_of::<T>() == 0`, then the only requirement is that the
    /// `self.to_base_index() + ofset` must not be greater than `RawTableInner.bucket_mask`,
    /// i.e. `(self.to_base_index() + ofset) <= RawTableInner.bucket_mask` or, in other words,
    /// `self.to_base_index() + ofset + 1` must be no greater than the number returned by the
    /// function [`RawTable::buckets`] or [`RawTableInner::buckets`].
    ///
    /// [`Bucket`]: crate::raw::Bucket
    /// [`<*mut T>::sub`]: https://doc.rust-lang.org/core/primitive.pointer.html#method.sub-1
    /// [`NonNull::new_unchecked`]: https://doc.rust-lang.org/stable/std/ptr/struct.NonNull.html#method.new_unchecked
    /// [`RawTable::buckets`]: crate::raw::RawTable::buckets
    /// [`RawTableInner::buckets`]: RawTableInner::buckets
    #[inline]
    unsafe fn next_n(&self, offset: usize) -> Self {
        let ptr = if mem::size_of::<T>() == 0 {
            // invalid pointer is good enough for ZST
            invalid_mut(self.ptr.as_ptr() as usize + offset)
        } else {
            self.ptr.as_ptr().sub(offset)
        };

        Self {
            index: self.index.wrapping_add(offset),
            ptr: NonNull::new_unchecked(ptr),
        }
    }
}

/// A reference to an empty bucket into which an can be inserted.
pub struct InsertSlot {
    index: usize,
}

/// Probe sequence based on triangular numbers, which is guaranteed (since our
/// table size is a power of two) to visit every group of elements exactly once.
///
/// A triangular probe has us jump by 1 more group every time. So first we
/// jump by 1 group (meaning we just continue our linear scan), then 2 groups
/// (skipping over 1 group), then 3 groups (skipping over 2 groups), and so on.
///
/// Proof that the probe will visit every group in the table:
/// <https://fgiesen.wordpress.com/2015/02/22/triangular-numbers-mod-2n/>
pub(crate) struct ProbeSeq {
    pub(crate) pos: usize,
    stride: usize,
}

impl ProbeSeq {
    #[inline]
    pub(crate) fn move_next(&mut self, bucket_mask: usize) {
        // We should have found an empty bucket by now and ended the probe.
        debug_assert!(
            self.stride <= bucket_mask,
            "Went past end of probe sequence"
        );

        self.stride += Group::WIDTH;
        self.pos += self.stride;
        self.pos &= bucket_mask;
    }
}

/// Non-generic part of `RawTable` which allows functions to be instantiated only once regardless
/// of how many different key-value types are used.
struct RawTableInner {
    // Mask to get an index from a hash value. The value is one less than the
    // number of buckets in the table.
    bucket_mask: usize,

    // [Padding], T1, T2, ..., Tlast, C1, C2, ...
    //                                ^ points here
    ctrl: NonNull<u8>,

    // Number of elements in the table, only really used by len()
    items: usize,

    // Number of elements that can be inserted before we need to grow the table
    growth_left: usize,
}

impl RawTableInner {
    const NEW: Self = RawTableInner::new();

    /// Creates a new empty hash table without allocating any memory.
    ///
    /// In effect this returns a table with exactly 1 bucket. However we can
    /// leave the data pointer dangling since that bucket is never accessed
    /// due to our load factor forcing us to always have at least 1 free bucket.
    #[inline]
    const fn new() -> Self {
        Self {
            // Be careful to cast the entire slice to a raw pointer.
            ctrl: unsafe { NonNull::new_unchecked(Group::static_empty() as *const _ as *mut u8) },
            bucket_mask: 0,
            items: 0,
            growth_left: 0,
        }
    }

    /// Allocates a new [`RawTableInner`] with the given number of buckets.
    /// The control bytes and buckets are left uninitialized.
    ///
    /// # Safety
    ///
    /// The caller of this function must ensure that the `buckets` is power of two
    /// and also initialize all control bytes of the length `self.bucket_mask + 1 +
    /// Group::WIDTH` with the [`EMPTY`] bytes.
    ///
    /// See also [`Allocator`] API for other safety concerns.
    ///
    /// [`Allocator`]: https://doc.rust-lang.org/alloc/alloc/trait.Allocator.html
    unsafe fn new_uninitialized(table_layout: TableLayout, buckets: usize) -> Self {
        debug_assert!(buckets.is_power_of_two());

        // Avoid `Option::ok_or_else` because it bloats LLVM IR.
        let Some((layout, ctrl_offset)) = table_layout.calculate_layout_for(buckets) else {
            panic!("Capacity overflow");
        };

        let ptr = alloc::alloc(layout);

        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        // SAFETY: null pointer will be caught in above check
        let ctrl = NonNull::new_unchecked(ptr.add(ctrl_offset));

        Self {
            ctrl,
            bucket_mask: buckets - 1,
            items: 0,
            growth_left: bucket_mask_to_capacity(buckets - 1),
        }
    }

    /// Allocates a new [`RawTableInner`] with at least enough capacity for inserting
    /// the given number of elements without reallocating.
    ///
    /// Panics if the new capacity exceeds [`isize::MAX`] bytes and [`abort`] the program
    /// in case of allocation error. Use [`fallible_with_capacity`] instead if you want to
    /// handle memory allocation failure.
    ///
    /// All the control bytes are initialized with the [`EMPTY`] bytes.
    ///
    /// [`fallible_with_capacity`]: RawTableInner::fallible_with_capacity
    /// [`abort`]: https://doc.rust-lang.org/alloc/alloc/fn.handle_alloc_error.html
    fn with_capacity(table_layout: TableLayout, capacity: usize) -> Self {
        // Avoid `Result::unwrap_or_else` because it bloats LLVM IR.
        Self::fallible_with_capacity(table_layout, capacity)
    }

    /// Attempts to allocate a new [`RawTableInner`] with at least enough
    /// capacity for inserting the given number of elements without reallocating.
    ///
    /// All the control bytes are initialized with the [`EMPTY`] bytes.
    #[inline]
    fn fallible_with_capacity(table_layout: TableLayout, capacity: usize) -> Self {
        if capacity == 0 {
            Self::NEW
        } else {
            // SAFETY: We checked that we could successfully allocate the new table, and then
            // initialized all control bytes with the constant `EMPTY` byte.
            unsafe {
                let Some(buckets) = capacity_to_buckets(capacity) else {
                    panic!("Capacity overflow");
                };

                let result = Self::new_uninitialized(table_layout, buckets);

                // SAFETY: We checked that the table is allocated and therefore the table already has
                // `self.bucket_mask + 1 + Group::WIDTH` number of control bytes (see TableLayout::calculate_layout_for)
                // so writing `self.num_ctrl_bytes() == bucket_mask + 1 + Group::WIDTH` bytes is safe.
                result.ctrl(0).write_bytes(EMPTY, result.num_ctrl_bytes());

                result
            }
        }
    }

    /// Finds the position to insert something in a group.
    ///
    /// **This may have false positives and must be fixed up with `fix_insert_slot`
    /// before it's used.**
    ///
    /// The function is guaranteed to return the index of an empty or deleted [`Bucket`]
    /// in the range `0..self.buckets()` (`0..=self.bucket_mask`).
    #[inline]
    fn find_insert_slot_in_group(&self, group: &Group, probe_seq: &ProbeSeq) -> Option<usize> {
        let bit = group.match_empty_or_deleted().lowest_set_bit();

        if likely(bit.is_some()) {
            // This is the same as `(probe_seq.pos + bit) % self.buckets()` because the number
            // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
            Some((probe_seq.pos + bit.unwrap()) & self.bucket_mask)
        } else {
            None
        }
    }

    #[inline]
    unsafe fn record_item_insert_at(&mut self, index: usize, old_ctrl: u8, hash: u64) {
        self.growth_left -= usize::from(special_is_empty(old_ctrl));
        self.set_ctrl_h2(index, hash);
        self.items += 1;
    }

    /// Searches for an empty or deleted bucket which is suitable for inserting
    /// a new element, returning the `index` for the new [`Bucket`].
    ///
    /// This function does not make any changes to the `data` part of the table, or any
    /// changes to the `items` or `growth_left` field of the table.
    ///
    /// The table must have at least 1 empty or deleted `bucket`, otherwise this function
    /// will never return (will go into an infinite loop) for tables larger than the group
    /// width, or return an index outside of the table indices range if the table is less
    /// than the group width.
    ///
    /// If there is at least 1 empty or deleted `bucket` in the table, the function is
    /// guaranteed to return [`InsertSlot`] with an index in the range `0..self.buckets()`,
    /// but in any case, if this function returns [`InsertSlot`], it will contain an index
    /// in the range `0..=self.buckets()`.
    ///
    /// # Safety
    ///
    /// The [`RawTableInner`] must have properly initialized control bytes otherwise calling
    /// this function results in [`undefined behavior`].
    ///
    /// Attempt to write data at the [`InsertSlot`] returned by this function when the table is
    /// less than the group width and if there was not at least one empty or deleted bucket in
    /// the table will cause immediate [`undefined behavior`]. This is because in this case the
    /// function will return `self.bucket_mask + 1` as an index due to the trailing [`EMPTY]
    /// control bytes outside the table range.
    ///
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn find_insert_slot(&self, hash: u64) -> InsertSlot {
        let mut probe_seq = probe_seq(self.bucket_mask, hash);

        loop {
            // SAFETY:
            // * Caller of this function ensures that the control bytes are
            //   properly initialized.
            //
            // * `ProbeSeq.pos` cannot be greater than `self.bucket_mask =
            //   self.buckets() - 1` of the table due to masking with
            //   `self.bucket_mask` and also because mumber of buckets is a
            //   power of two (see `probe_seq` function).
            //
            // * Even if `ProbeSeq.pos` returns `position == self.bucket_mask`,
            //   it is safe to call `Group::load` due to the extended control
            //  bytes range, which is `self.bucket_mask + 1 + Group::WIDTH` (in
            //   fact, this means that the last control byte will never be read
            //   for the allocated table);
            //
            // * Also, even if `RawTableInner` is not already allocated,
            //   `ProbeSeq.pos` will always return "0" (zero), so Group::load
            //   will read unaligned `Group::static_empty()` bytes, which is
            //   safe (see RawTableInner::new).
            let group = unsafe { Group::load(self.ctrl(probe_seq.pos)) };

            let index = self.find_insert_slot_in_group(&group, &probe_seq);
            if likely(index.is_some()) {
                // SAFETY:
                // * Caller of this function ensures that the control bytes are properly initialized.
                //
                // * We use this function with the slot / index found by `self.find_insert_slot_in_group`
                unsafe {
                    return self.fix_insert_slot(index.unwrap_unchecked());
                }
            }
            probe_seq.move_next(self.bucket_mask);
        }
    }

    /// Fixes up an insertion slot returned by the [`RawTableInner::find_insert_slot_in_group`] method.
    ///
    /// In tables smaller than the group width (`self.buckets() < Group::WIDTH`), trailing control
    /// bytes outside the range of the table are filled with [`EMPTY`] entries. These will unfortunately
    /// trigger a match of [`RawTableInner::find_insert_slot_in_group`] function. This is because
    /// the `Some(bit)` returned by `group.match_empty_or_deleted().lowest_set_bit()` after masking
    /// (`(probe_seq.pos + bit) & self.bucket_mask`) may point to a full bucket that is already occupied.
    /// We detect this situation here and perform a second scan starting at the beginning of the table.
    /// This second scan is guaranteed to find an empty slot (due to the load factor) before hitting the
    /// trailing control bytes (containing [`EMPTY`] bytes).
    ///
    /// If this function is called correctly, it is guaranteed to return [`InsertSlot`] with an
    /// index of an empty or deleted bucket in the range `0..self.buckets()` (see `Warning` and
    /// `Safety`).
    ///
    /// # Warning
    ///
    /// The table must have at least 1 empty or deleted `bucket`, otherwise if the table is less than
    /// the group width (`self.buckets() < Group::WIDTH`) this function returns an index outside of the
    /// table indices range `0..self.buckets()` (`0..=self.bucket_mask`). Attempt to write data at that
    /// index will cause immediate [`undefined behavior`].
    ///
    /// # Safety
    ///
    /// The safety rules are directly derived from the safety rules for [`RawTableInner::ctrl`] method.
    /// Thus, in order to uphold those safety contracts, as well as for the correct logic of the work
    /// of this crate, the following rules are necessary and sufficient:
    ///
    /// * The [`RawTableInner`] must have properly initialized control bytes otherwise calling this
    ///   function results in [`undefined behavior`].
    ///
    /// * This function must only be used on insertion slots found by [`RawTableInner::find_insert_slot_in_group`]
    ///   (after the `find_insert_slot_in_group` function, but before insertion into the table).
    ///
    /// * The `index` must not be greater than the `self.bucket_mask`, i.e. `(index + 1) <= self.buckets()`
    ///   (this one is provided by the [`RawTableInner::find_insert_slot_in_group`] function).
    ///
    /// Calling this function with an index not provided by [`RawTableInner::find_insert_slot_in_group`]
    /// may result in [`undefined behavior`] even if the index satisfies the safety rules of the
    /// [`RawTableInner::ctrl`] function (`index < self.bucket_mask + 1 + Group::WIDTH`).
    ///
    /// [`RawTableInner::ctrl`]: RawTableInner::ctrl
    /// [`RawTableInner::find_insert_slot_in_group`]: RawTableInner::find_insert_slot_in_group
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn fix_insert_slot(&self, mut index: usize) -> InsertSlot {
        // SAFETY: The caller of this function ensures that `index` is in the range `0..=self.bucket_mask`.
        if unlikely(self.is_bucket_full(index)) {
            debug_assert!(self.bucket_mask < Group::WIDTH);
            // SAFETY:
            //
            // * Since the caller of this function ensures that the control bytes are properly
            //   initialized and `ptr = self.ctrl(0)` points to the start of the array of control
            //   bytes, therefore: `ctrl` is valid for reads, properly aligned to `Group::WIDTH`
            //   and points to the properly initialized control bytes (see also
            //   `TableLayout::calculate_layout_for` and `ptr::read`);
            //
            // * Because the caller of this function ensures that the index was provided by the
            //   `self.find_insert_slot_in_group()` function, so for for tables larger than the
            //   group width (self.buckets() >= Group::WIDTH), we will never end up in the given
            //   branch, since `(probe_seq.pos + bit) & self.bucket_mask` in `find_insert_slot_in_group`
            //   cannot return a full bucket index. For tables smaller than the group width, calling
            //   the `unwrap_unchecked` function is also safe, as the trailing control bytes outside
            //   the range of the table are filled with EMPTY bytes (and we know for sure that there
            //   is at least one FULL bucket), so this second scan either finds an empty slot (due to
            //   the load factor) or hits the trailing control bytes (containing EMPTY).
            index = Group::load_aligned(self.ctrl(0))
                .match_empty_or_deleted()
                .lowest_set_bit()
                .unwrap_unchecked();
        }
        InsertSlot { index }
    }

    /// Sets a control byte to the hash, and possibly also the replicated control byte at
    /// the end of the array.
    ///
    /// This function does not make any changes to the `data` parts of the table,
    /// or any changes to the the `items` or `growth_left` field of the table.
    ///
    /// # Safety
    ///
    /// The safety rules are directly derived from the safety rules for [`RawTableInner::set_ctrl`]
    /// method. Thus, in order to uphold the safety contracts for the method, you must observe the
    /// following rules when calling this function:
    ///
    /// * The [`RawTableInner`] has already been allocated;
    ///
    /// * The `index` must not be greater than the `RawTableInner.bucket_mask`, i.e.
    ///   `index <= RawTableInner.bucket_mask` or, in other words, `(index + 1)` must
    ///   be no greater than the number returned by the function [`RawTableInner::buckets`].
    ///
    /// Calling this function on a table that has not been allocated results in [`undefined behavior`].
    ///
    /// See also [`Bucket::as_ptr`] method, for more information about of properly removing
    /// or saving `data element` from / into the [`RawTable`] / [`RawTableInner`].
    ///
    /// [`RawTableInner::set_ctrl`]: RawTableInner::set_ctrl
    /// [`RawTableInner::buckets`]: RawTableInner::buckets
    /// [`Bucket::as_ptr`]: Bucket::as_ptr
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn set_ctrl_h2(&mut self, index: usize, hash: u64) {
        // SAFETY: The caller must uphold the safety rules for the [`RawTableInner::set_ctrl_h2`]
        self.set_ctrl(index, h2(hash));
    }

    /// Sets a control byte, and possibly also the replicated control byte at
    /// the end of the array.
    ///
    /// This function does not make any changes to the `data` parts of the table,
    /// or any changes to the the `items` or `growth_left` field of the table.
    ///
    /// # Safety
    ///
    /// You must observe the following safety rules when calling this function:
    ///
    /// * The [`RawTableInner`] has already been allocated;
    ///
    /// * The `index` must not be greater than the `RawTableInner.bucket_mask`, i.e.
    ///   `index <= RawTableInner.bucket_mask` or, in other words, `(index + 1)` must
    ///   be no greater than the number returned by the function [`RawTableInner::buckets`].
    ///
    /// Calling this function on a table that has not been allocated results in [`undefined behavior`].
    ///
    /// See also [`Bucket::as_ptr`] method, for more information about of properly removing
    /// or saving `data element` from / into the [`RawTable`] / [`RawTableInner`].
    ///
    /// [`RawTableInner::buckets`]: RawTableInner::buckets
    /// [`Bucket::as_ptr`]: Bucket::as_ptr
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn set_ctrl(&mut self, index: usize, ctrl: u8) {
        // Replicate the first Group::WIDTH control bytes at the end of
        // the array without using a branch. If the tables smaller than
        // the group width (self.buckets() < Group::WIDTH),
        // `index2 = Group::WIDTH + index`, otherwise `index2` is:
        //
        // - If index >= Group::WIDTH then index == index2.
        // - Otherwise index2 == self.bucket_mask + 1 + index.
        //
        // The very last replicated control byte is never actually read because
        // we mask the initial index for unaligned loads, but we write it
        // anyways because it makes the set_ctrl implementation simpler.
        //
        // If there are fewer buckets than Group::WIDTH then this code will
        // replicate the buckets at the end of the trailing group. For example
        // with 2 buckets and a group size of 4, the control bytes will look
        // like this:
        //
        //     Real    |             Replicated
        // ---------------------------------------------
        // | [A] | [B] | [EMPTY] | [EMPTY] | [A] | [B] |
        // ---------------------------------------------

        // This is the same as `(index.wrapping_sub(Group::WIDTH)) % self.buckets() + Group::WIDTH`
        // because the number of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
        let index2 = ((index.wrapping_sub(Group::WIDTH)) & self.bucket_mask) + Group::WIDTH;

        // SAFETY: The caller must uphold the safety rules for the [`RawTableInner::set_ctrl`]
        *self.ctrl(index) = ctrl;
        *self.ctrl(index2) = ctrl;
    }

    /// Returns an iterator over every element in the table.
    ///
    /// # Safety
    ///
    /// If any of the following conditions are violated, the result
    /// is [`undefined behavior`]:
    ///
    /// * The caller has to ensure that the `RawTableInner` outlives the
    ///   `RawIter`. Because we cannot make the `next` method unsafe on
    ///   the `RawIter` struct, we have to make the `iter` method unsafe.
    ///
    /// * The [`RawTableInner`] must have properly initialized control bytes.
    ///
    /// The type `T` must be the actual type of the elements stored in the table,
    /// otherwise using the returned [`RawIter`] results in [`undefined behavior`].
    ///
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn iter<T>(&self) -> RawIter<T> {
        // SAFETY:
        // 1. Since the caller of this function ensures that the control bytes
        //    are properly initialized and `self.data_end()` points to the start
        //    of the array of control bytes, therefore: `ctrl` is valid for reads,
        //    properly aligned to `Group::WIDTH` and points to the properly initialized
        //    control bytes.
        // 2. `data` bucket index in the table is equal to the `ctrl` index (i.e.
        //    equal to zero).
        // 3. We pass the exact value of buckets of the table to the function.
        //
        //                         `ctrl` points here (to the start
        //                         of the first control byte `CT0`)
        //                          ∨
        // [Pad], T_n, ..., T1, T0, |CT0, CT1, ..., CT_n|, CTa_0, CTa_1, ..., CTa_m
        //                           \________  ________/
        //                                    \/
        //       `n = buckets - 1`, i.e. `RawTableInner::buckets() - 1`
        //
        // where: T0...T_n  - our stored data;
        //        CT0...CT_n - control bytes or metadata for `data`.
        //        CTa_0...CTa_m - additional control bytes, where `m = Group::WIDTH - 1` (so that the search
        //                        with loading `Group` bytes from the heap works properly, even if the result
        //                        of `h1(hash) & self.bucket_mask` is equal to `self.bucket_mask`). See also
        //                        `RawTableInner::set_ctrl` function.
        //
        // P.S. `h1(hash) & self.bucket_mask` is the same as `hash as usize % self.buckets()` because the number
        // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
        let data = Bucket::from_base_index(self.data_end(), 0);
        RawIter {
            // SAFETY: See explanation above
            iter: RawIterRange::new(self.ctrl.as_ptr(), data, self.buckets()),
            items: self.items,
        }
    }

    /// Returns pointer to one past last `data` element in the the table as viewed from
    /// the start point of the allocation (convenience for `self.ctrl.cast()`).
    ///
    /// This function actually returns a pointer to the end of the `data element` at
    /// index "0" (zero).
    ///
    /// The caller must ensure that the `RawTableInner` outlives the returned [`NonNull<T>`],
    /// otherwise using it may result in [`undefined behavior`].
    ///
    /// # Note
    ///
    /// The type `T` must be the actual type of the elements stored in the table, otherwise
    /// using the returned [`NonNull<T>`] may result in [`undefined behavior`].
    ///
    /// ```none
    ///                        `table.data_end::<T>()` returns pointer that points here
    ///                        (to the end of `T0`)
    ///                          ∨
    /// [Pad], T_n, ..., T1, T0, |CT0, CT1, ..., CT_n|, CTa_0, CTa_1, ..., CTa_m
    ///                           \________  ________/
    ///                                    \/
    ///       `n = buckets - 1`, i.e. `RawTableInner::buckets() - 1`
    ///
    /// where: T0...T_n  - our stored data;
    ///        CT0...CT_n - control bytes or metadata for `data`.
    ///        CTa_0...CTa_m - additional control bytes, where `m = Group::WIDTH - 1` (so that the search
    ///                        with loading `Group` bytes from the heap works properly, even if the result
    ///                        of `h1(hash) & self.bucket_mask` is equal to `self.bucket_mask`). See also
    ///                        `RawTableInner::set_ctrl` function.
    ///
    /// P.S. `h1(hash) & self.bucket_mask` is the same as `hash as usize % self.buckets()` because the number
    /// of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
    /// ```
    ///
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    fn data_end<T>(&self) -> NonNull<T> {
        self.ctrl.cast()
    }

    /// Checks whether the bucket at `index` is full.
    ///
    /// # Safety
    ///
    /// The caller must ensure `index` is less than the number of buckets.
    #[inline]
    unsafe fn is_bucket_full(&self, index: usize) -> bool {
        debug_assert!(index < self.buckets());
        is_full(*self.ctrl(index))
    }

    /// Returns the number of buckets in the table.
    #[inline]
    pub fn buckets(&self) -> usize {
        self.bucket_mask + 1
    }

    /// Returns a pointer to a control byte.
    ///
    /// # Safety
    ///
    /// For the allocated [`RawTableInner`], the result is [`Undefined Behavior`],
    /// if the `index` is greater than the `self.bucket_mask + 1 + Group::WIDTH`.
    /// In that case, calling this function with `index == self.bucket_mask + 1 + Group::WIDTH`
    /// will return a pointer to the end of the allocated table and it is useless on its own.
    ///
    /// Calling this function with `index >= self.bucket_mask + 1 + Group::WIDTH` on a
    /// table that has not been allocated results in [`Undefined Behavior`].
    ///
    /// So to satisfy both requirements you should always follow the rule that
    /// `index < self.bucket_mask + 1 + Group::WIDTH`
    ///
    /// Calling this function on [`RawTableInner`] that are not already allocated is safe
    /// for read-only purpose.
    ///
    /// See also [`Bucket::as_ptr()`] method, for more information about of properly removing
    /// or saving `data element` from / into the [`RawTable`] / [`RawTableInner`].
    ///
    /// [`Bucket::as_ptr()`]: Bucket::as_ptr()
    /// [`Undefined Behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn ctrl(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.num_ctrl_bytes());
        // SAFETY: The caller must uphold the safety rules for the [`RawTableInner::ctrl`]
        self.ctrl.as_ptr().add(index)
    }

    #[inline]
    fn num_ctrl_bytes(&self) -> usize {
        self.bucket_mask + 1 + Group::WIDTH
    }

    #[inline]
    fn is_empty_singleton(&self) -> bool {
        self.bucket_mask == 0
    }

    /// Executes the destructors (if any) of the values stored in the table.
    ///
    /// # Note
    ///
    /// This function does not erase the control bytes of the table and does
    /// not make any changes to the `items` or `growth_left` fields of the
    /// table. If necessary, the caller of this function must manually set
    /// up these table fields, for example using the [`clear_no_drop`] function.
    ///
    /// Be careful during calling this function, because drop function of
    /// the elements can panic, and this can leave table in an inconsistent
    /// state.
    ///
    /// # Safety
    ///
    /// The type `T` must be the actual type of the elements stored in the table,
    /// otherwise calling this function may result in [`undefined behavior`].
    ///
    /// If `T` is a type that should be dropped and **the table is not empty**,
    /// calling this function more than once results in [`undefined behavior`].
    ///
    /// If `T` is not [`Copy`], attempting to use values stored in the table after
    /// calling this function may result in [`undefined behavior`].
    ///
    /// It is safe to call this function on a table that has not been allocated,
    /// on a table with uninitialized control bytes, and on a table with no actual
    /// data but with `Full` control bytes if `self.items == 0`.
    ///
    /// See also [`Bucket::drop`] / [`Bucket::as_ptr`] methods, for more information
    /// about of properly removing or saving `element` from / into the [`RawTable`] /
    /// [`RawTableInner`].
    ///
    /// [`Bucket::drop`]: Bucket::drop
    /// [`Bucket::as_ptr`]: Bucket::as_ptr
    /// [`clear_no_drop`]: RawTableInner::clear_no_drop
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn drop_elements<T>(&mut self) {
        // Check that `self.items != 0`. Protects against the possibility
        // of creating an iterator on an table with uninitialized control bytes.
        if mem::needs_drop::<T>() && self.items != 0 {
            // SAFETY: We know for sure that RawTableInner will outlive the
            // returned `RawIter` iterator, and the caller of this function
            // must uphold the safety contract for `drop_elements` method.
            for item in self.iter::<T>() {
                // SAFETY: The caller must uphold the safety contract for
                // `drop_elements` method.
                item.drop();
            }
        }
    }

    /// Deallocates the table without dropping any entries.
    ///
    /// # Note
    ///
    /// This function must be called only after [`drop_elements`](RawTableInner::drop_elements),
    /// else it can lead to leaking of memory. Also calling this function automatically
    /// makes invalid (dangling) all instances of buckets ([`Bucket`]) and makes invalid
    /// (dangling) the `ctrl` field of the table.
    ///
    /// # Safety
    ///
    /// If any of the following conditions are violated, the result is [`Undefined Behavior`]:
    ///
    /// * The [`RawTableInner`] has already been allocated;
    ///
    /// * The `alloc` must be the same [`Allocator`] as the `Allocator` that was used
    ///   to allocate this table.
    ///
    /// * The `table_layout` must be the same [`TableLayout`] as the `TableLayout` that was used
    ///   to allocate this table.
    ///
    /// See also [`GlobalAlloc::dealloc`] or [`Allocator::deallocate`] for more  information.
    ///
    /// [`Undefined Behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    /// [`GlobalAlloc::dealloc`]: https://doc.rust-lang.org/alloc/alloc/trait.GlobalAlloc.html#tymethod.dealloc
    /// [`Allocator::deallocate`]: https://doc.rust-lang.org/alloc/alloc/trait.Allocator.html#tymethod.deallocate
    #[inline]
    unsafe fn free_buckets(&mut self, table_layout: TableLayout) {
        // SAFETY: The caller must uphold the safety contract for `free_buckets`
        // method.
        let (ptr, layout) = self.allocation_info(table_layout);
        alloc::dealloc(ptr.as_ptr(), layout);
    }

    /// Returns a pointer to the allocated memory and the layout that was used to
    /// allocate the table.
    ///
    /// # Safety
    ///
    /// Caller of this function must observe the following safety rules:
    ///
    /// * The [`RawTableInner`] has already been allocated, otherwise
    ///   calling this function results in [`undefined behavior`]
    ///
    /// * The `table_layout` must be the same [`TableLayout`] as the `TableLayout`
    ///   that was used to allocate this table. Failure to comply with this condition
    ///   may result in [`undefined behavior`].
    ///
    /// See also [`GlobalAlloc::dealloc`] or [`Allocator::deallocate`] for more  information.
    ///
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    /// [`GlobalAlloc::dealloc`]: https://doc.rust-lang.org/alloc/alloc/trait.GlobalAlloc.html#tymethod.dealloc
    /// [`Allocator::deallocate`]: https://doc.rust-lang.org/alloc/alloc/trait.Allocator.html#tymethod.deallocate
    #[inline]
    unsafe fn allocation_info(&self, table_layout: TableLayout) -> (NonNull<u8>, Layout) {
        debug_assert!(
            !self.is_empty_singleton(),
            "this function can only be called on non-empty tables"
        );

        // Avoid `Option::unwrap_or_else` because it bloats LLVM IR.
        let (layout, ctrl_offset) = match table_layout.calculate_layout_for(self.buckets()) {
            Some(lco) => lco,
            None => unsafe { hint::unreachable_unchecked() },
        };

        (
            // SAFETY: The caller must uphold the safety contract for `allocation_info` method.
            unsafe { NonNull::new_unchecked(self.ctrl.as_ptr().sub(ctrl_offset)) },
            layout,
        )
    }

    /// Executes the destructors (if any) of the values stored in the table and than
    /// deallocates the table.
    ///
    /// # Note
    ///
    /// Calling this function automatically makes invalid (dangling) all instances of
    /// buckets ([`Bucket`]) and makes invalid (dangling) the `ctrl` field of the table.
    ///
    /// This function does not make any changes to the `bucket_mask`, `items` or `growth_left`
    /// fields of the table. If necessary, the caller of this function must manually set
    /// up these table fields.
    ///
    /// # Safety
    ///
    /// If any of the following conditions are violated, the result is [`undefined behavior`]:
    ///
    /// * Calling this function more than once;
    ///
    /// * The type `T` must be the actual type of the elements stored in the table.
    ///
    /// * The `alloc` must be the same [`Allocator`] as the `Allocator` that was used
    ///   to allocate this table.
    ///
    /// * The `table_layout` must be the same [`TableLayout`] as the `TableLayout` that
    ///   was used to allocate this table.
    ///
    /// The caller of this function should pay attention to the possibility of the
    /// elements' drop function panicking, because this:
    ///
    ///    * May leave the table in an inconsistent state;
    ///
    ///    * Memory is never deallocated, so a memory leak may occur.
    ///
    /// Attempt to use the `ctrl` field of the table (dereference) after calling this
    /// function results in [`undefined behavior`].
    ///
    /// It is safe to call this function on a table that has not been allocated,
    /// on a table with uninitialized control bytes, and on a table with no actual
    /// data but with `Full` control bytes if `self.items == 0`.
    ///
    /// See also [`RawTableInner::drop_elements`] or [`RawTableInner::free_buckets`]
    /// for more  information.
    ///
    /// [`RawTableInner::drop_elements`]: RawTableInner::drop_elements
    /// [`RawTableInner::free_buckets`]: RawTableInner::free_buckets
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn drop_inner_table<T>(&mut self, table_layout: TableLayout) {
        if !self.is_empty_singleton() {
            unsafe {
                // SAFETY: The caller must uphold the safety contract for `drop_inner_table` method.
                self.drop_elements::<T>();
                // SAFETY:
                // 1. We have checked that our table is allocated.
                // 2. The caller must uphold the safety contract for `drop_inner_table` method.
                self.free_buckets(table_layout);
            }
        }
    }
}

impl<T> Drop for RawTable<T> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY:
            // 1. We call the function only once;
            // 2. We know for sure that `alloc` and `table_layout` matches the [`Allocator`]
            //    and [`TableLayout`] that were used to allocate this table.
            // 3. If the drop function of any elements fails, then only a memory leak will occur,
            //    and we don't care because we are inside the `Drop` function of the `RawTable`,
            //    so there won't be any table left in an inconsistent state.
            self.table.drop_inner_table::<T>(Self::TABLE_LAYOUT);
        }
    }
}

// Constant for h2 function that grabing the top 7 bits of the hash.
const MIN_HASH_LEN: usize = if mem::size_of::<usize>() < mem::size_of::<u64>() {
    mem::size_of::<usize>()
} else {
    mem::size_of::<u64>()
};

/// Returns an iterator-like object for a probe sequence on the table.
///
/// This iterator never terminates, but is guaranteed to visit each bucket
/// group exactly once. The loop using `probe_seq` must terminate upon
/// reaching a group containing an empty bucket.
#[inline]
pub(crate) fn probe_seq(bucket_mask: usize, hash: u64) -> ProbeSeq {
    ProbeSeq {
        // This is the same as `hash as usize % self.buckets()` because the number
        // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
        pos: h1(hash) & bucket_mask,
        stride: 0,
    }
}

/// Primary hash function, used to select the initial bucket to probe from.
#[inline]
#[allow(clippy::cast_possible_truncation)]
fn h1(hash: u64) -> usize {
    // On 32-bit platforms we simply ignore the higher hash bits.
    hash as usize
}

/// Secondary hash function, saved in the low 7 bits of the control byte.
#[inline]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn h2(hash: u64) -> u8 {
    // Grab the top 7 bits of the hash. While the hash is normally a full 64-bit
    // value, some hash functions (such as FxHash) produce a usize result
    // instead, which means that the top 32 bits are 0 on 32-bit platforms.
    // So we use MIN_HASH_LEN constant to handle this.
    let top7 = hash >> (MIN_HASH_LEN * 8 - 7);
    (top7 & 0x7f) as u8 // truncation
}

/// Control byte value for an empty bucket.
const EMPTY: u8 = 0b1111_1111;

/// Checks whether a control byte represents a full bucket (top bit is clear).
#[inline]
fn is_full(ctrl: u8) -> bool {
    ctrl & 0x80 == 0
}

/// Checks whether a control byte represents a special value (top bit is set).
#[inline]
fn is_special(ctrl: u8) -> bool {
    ctrl & 0x80 != 0
}

/// Checks whether a special control value is EMPTY (just check 1 bit).
#[inline]
fn special_is_empty(ctrl: u8) -> bool {
    debug_assert!(is_special(ctrl));
    ctrl & 0x01 != 0
}

/// Helper which allows the max calculation for ctrl_align to be statically computed for each T
/// while keeping the rest of `calculate_layout_for` independent of `T`
#[derive(Copy, Clone)]
struct TableLayout {
    size: usize,
    ctrl_align: usize,
}

impl TableLayout {
    #[inline]
    const fn new<T>() -> Self {
        let layout = Layout::new::<T>();
        Self {
            size: layout.size(),
            ctrl_align: if layout.align() > Group::WIDTH {
                layout.align()
            } else {
                Group::WIDTH
            },
        }
    }

    #[inline]
    fn calculate_layout_for(self, buckets: usize) -> Option<(Layout, usize)> {
        debug_assert!(buckets.is_power_of_two());

        let TableLayout { size, ctrl_align } = self;
        // Manual layout calculation since Layout methods are not yet stable.
        let ctrl_offset =
            size.checked_mul(buckets)?.checked_add(ctrl_align - 1)? & !(ctrl_align - 1);
        let len = ctrl_offset.checked_add(buckets + Group::WIDTH)?;

        // We need an additional check to ensure that the allocation doesn't
        // exceed `isize::MAX` (https://github.com/rust-lang/rust/pull/95295).
        if len > isize::MAX as usize - (ctrl_align - 1) {
            return None;
        }

        Some((
            unsafe { Layout::from_size_align_unchecked(len, ctrl_align) },
            ctrl_offset,
        ))
    }
}

/// Returns the number of buckets needed to hold the given number of items,
/// taking the maximum load factor into account.
///
/// Returns `None` if an overflow occurs.
// Workaround for emscripten bug emscripten-core/emscripten-fastcomp#258
#[cfg_attr(target_os = "emscripten", inline(never))]
#[cfg_attr(not(target_os = "emscripten"), inline)]
fn capacity_to_buckets(cap: usize) -> Option<usize> {
    debug_assert_ne!(cap, 0);

    // For small tables we require at least 1 empty bucket so that lookups are
    // guaranteed to terminate if an element doesn't exist in the table.
    if cap < 8 {
        // We don't bother with a table size of 2 buckets since that can only
        // hold a single element. Instead we skip directly to a 4 bucket table
        // which can hold 3 elements.
        return Some(if cap < 4 { 4 } else { 8 });
    }

    // Otherwise require 1/8 buckets to be empty (87.5% load)
    //
    // Be careful when modifying this, calculate_layout relies on the
    // overflow check here.
    let adjusted_cap = cap.checked_mul(8)? / 7;

    // Any overflows will have been caught by the checked_mul. Also, any
    // rounding errors from the division above will be cleaned up by
    // next_power_of_two (which can't overflow because of the previous division).
    Some(adjusted_cap.next_power_of_two())
}

/// Returns the maximum effective capacity for the given bucket mask, taking
/// the maximum load factor into account.
#[inline]
fn bucket_mask_to_capacity(bucket_mask: usize) -> usize {
    if bucket_mask < 8 {
        // For tables with 1/2/4/8 buckets, we always reserve one empty slot.
        // Keep in mind that the bucket mask is one less than the bucket count.
        bucket_mask
    } else {
        // For larger tables we reserve 12.5% of the slots as empty.
        ((bucket_mask + 1) / 8) * 7
    }
}

/// Iterator over a sub-range of a table. Unlike `RawIter` this iterator does
/// not track an item count.
pub(crate) struct RawIterRange<T> {
    // Mask of full buckets in the current group. Bits are cleared from this
    // mask as each element is processed.
    current_group: BitMaskIter,

    // Pointer to the buckets for the current group.
    data: Bucket<T>,

    // Pointer to the next group of control bytes,
    // Must be aligned to the group size.
    next_ctrl: *const u8,

    // Pointer one past the last control byte of this range.
    end: *const u8,
}

impl<T> RawIterRange<T> {
    /// Returns a `RawIterRange` covering a subset of a table.
    ///
    /// # Safety
    ///
    /// If any of the following conditions are violated, the result is
    /// [`undefined behavior`]:
    ///
    /// * `ctrl` must be [valid] for reads, i.e. table outlives the `RawIterRange`;
    ///
    /// * `ctrl` must be properly aligned to the group size (Group::WIDTH);
    ///
    /// * `ctrl` must point to the array of properly initialized control bytes;
    ///
    /// * `data` must be the [`Bucket`] at the `ctrl` index in the table;
    ///
    /// * the value of `len` must be less than or equal to the number of table buckets,
    ///   and the returned value of `ctrl.as_ptr().add(len).offset_from(ctrl.as_ptr())`
    ///   must be positive.
    ///
    /// * The `ctrl.add(len)` pointer must be either in bounds or one
    ///   byte past the end of the same [allocated table].
    ///
    /// * The `len` must be a power of two.
    ///
    /// [valid]: https://doc.rust-lang.org/std/ptr/index.html#safety
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn new(ctrl: *const u8, data: Bucket<T>, len: usize) -> Self {
        debug_assert_ne!(len, 0);
        debug_assert_eq!(ctrl as usize % Group::WIDTH, 0);
        // SAFETY: The caller must uphold the safety rules for the [`RawIterRange::new`]
        let end = ctrl.add(len);

        // Load the first group and advance ctrl to point to the next group
        // SAFETY: The caller must uphold the safety rules for the [`RawIterRange::new`]
        let current_group = Group::load_aligned(ctrl).match_full();
        let next_ctrl = ctrl.add(Group::WIDTH);

        Self {
            current_group: current_group.into_iter(),
            data,
            next_ctrl,
            end,
        }
    }

    /// # Safety
    /// If DO_CHECK_PTR_RANGE is false, caller must ensure that we never try to iterate
    /// after yielding all elements.
    unsafe fn next_impl<const DO_CHECK_PTR_RANGE: bool>(&mut self) -> Option<Bucket<T>> {
        loop {
            if let Some(index) = self.current_group.next() {
                return Some(self.data.next_n(index));
            }

            if DO_CHECK_PTR_RANGE && self.next_ctrl >= self.end {
                return None;
            }

            // We might read past self.end up to the next group boundary,
            // but this is fine because it only occurs on tables smaller
            // than the group size where the trailing control bytes are all
            // EMPTY. On larger tables self.end is guaranteed to be aligned
            // to the group size (since tables are power-of-two sized).
            self.current_group = Group::load_aligned(self.next_ctrl).match_full().into_iter();
            self.data = self.data.next_n(Group::WIDTH);
            self.next_ctrl = self.next_ctrl.add(Group::WIDTH);
        }
    }
}

/// Iterator which returns a raw pointer to every full bucket in the table.
///
/// For maximum flexibility this iterator is not bound by a lifetime, but you
/// must observe several rules when using it:
/// - You must not free the hash table while iterating (including via growing/shrinking).
/// - It is fine to erase a bucket that has been yielded by the iterator.
/// - Erasing a bucket that has not yet been yielded by the iterator may still
///   result in the iterator yielding that bucket (unless `reflect_remove` is called).
/// - It is unspecified whether an element inserted after the iterator was
///   created will be yielded by that iterator (unless `reflect_insert` is called).
/// - The order in which the iterator yields bucket is unspecified and may
///   change in the future.
pub struct RawIter<T> {
    pub(crate) iter: RawIterRange<T>,
    items: usize,
}

impl<T> Iterator for RawIter<T> {
    type Item = Bucket<T>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn next(&mut self) -> Option<Bucket<T>> {
        // Inner iterator iterates over buckets
        // so it can do unnecessary work if we already yielded all items.
        if self.items == 0 {
            return None;
        }

        let nxt = unsafe {
            // SAFETY: We check number of items to yield using `items` field.
            self.iter.next_impl::<false>()
        };

        debug_assert!(nxt.is_some());
        self.items -= 1;

        nxt
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.items, Some(self.items))
    }
}

impl<T> ExactSizeIterator for RawIter<T> {}
impl<T> FusedIterator for RawIter<T> {}
