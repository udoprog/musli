// Note: This was ported from `hashbrown`, and still contains some of the
// comments assuming that it performs internal allocations. These are most
// likely wrong and need to be rewritten to take into account the safety
// requirements towards `OwnedBuf`.

use core::convert::{identity as likely, identity as unlikely};
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

use crate::buf::Buf;
use crate::buf::OwnedBuf;
use crate::endian::ByteOrder;
use crate::error::{Error, ErrorKind};
use crate::pointer::Size;
use crate::swiss::raw::{h2, is_full, probe_seq, special_is_empty, Group, ProbeSeq};
use crate::traits::ZeroCopy;

#[inline(always)]
fn invalid_mut<T>(addr: usize) -> *mut T {
    // Strict provenance "magic".
    addr as *mut T
}

/// Construction of a raw swiss table.
pub struct Constructor<'a, T, O: Size, E: ByteOrder> {
    buf: &'a mut OwnedBuf<O, E>,

    // Mask to get an index from a hash value. The value is one less than the
    // number of buckets in the table.
    bucket_mask: usize,

    // Control offset, where the control vectors are stored in `buf`.
    ctrl_ptr: usize,

    // Base offset where items are written in `buf`.
    base_ptr: usize,

    // Number of elements that can be inserted before we need to grow the table.
    // Since we can't grow the table, reaching this point results in an error.
    growth_left: usize,

    // Hold onto T to make sure the API stays coherent with the types the
    // constructor can write.
    _marker: PhantomData<T>,
}

impl<'a, T, O: Size, E: ByteOrder> Constructor<'a, T, O, E> {
    /// Wrap the given buffer for table construction.
    ///
    /// # Safety
    ///
    /// The caller must ensure that buffer contains allocated and correctly
    /// initialized memory at `ctrl_ptr` and `base_ptr`.
    ///
    /// * `ctrl_ptr` must point to a memory region that is `buckets + 1` length
    ///   sized for `Group` which has been bitwise initialized to [`EMPTY`].
    /// * `base_ptr` must point to a memory region that is `buckets` length
    ///   sized for `T`.
    /// * `buckets` must be a power of two.
    ///
    /// [`EMPTY`]: crate::swiss::raw::EMPTY
    pub(crate) unsafe fn with_buf(
        buf: &'a mut OwnedBuf<O, E>,
        ctrl_ptr: usize,
        base_ptr: usize,
        buckets: usize,
    ) -> Self {
        debug_assert!(buckets.is_power_of_two());

        Self {
            buf,
            bucket_mask: buckets - 1,
            ctrl_ptr,
            base_ptr,
            growth_left: bucket_mask_to_capacity(buckets - 1),
            _marker: PhantomData,
        }
    }

    /// Access the underlying buffer.
    pub(crate) fn buf(&mut self) -> &Buf {
        self.buf
    }

    /// Export bucket mask.
    pub(crate) fn bucket_mask(&self) -> usize {
        self.bucket_mask
    }

    /// Returns the number of buckets in the table.
    #[inline]
    pub(crate) fn buckets(&self) -> usize {
        self.bucket_mask + 1
    }

    /// Insert the given zero copy value into the table.
    pub(crate) fn insert(&mut self, hash: u64, value: &T) -> Result<Bucket<'_, T>, Error>
    where
        T: ZeroCopy,
    {
        unsafe {
            // SAFETY:
            // 1. The [`RawTableInner`] must already have properly initialized control bytes since
            //    we will never expose `Constructor::new_uninitialized` in a public API.
            let slot = self.find_insert_slot(hash)?;

            // We can avoid growing the table once we have reached our load factor if we are replacing
            // a tombstone. This works since the number of EMPTY slots does not change in this case.
            //
            // SAFETY: The function is guaranteed to return [`InsertSlot`] that contains an index
            // in the range `0..=self.buckets()`.
            let old_ctrl = *self.ctrl(slot.index);

            if unlikely(self.growth_left == 0 && special_is_empty(old_ctrl)) {
                return Err(Error::new(ErrorKind::CapacityError));
            }

            Ok(self.insert_in_slot(hash, slot, value))
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
    pub unsafe fn insert_in_slot(&mut self, hash: u64, slot: InsertSlot, value: &T) -> Bucket<T>
    where
        T: ZeroCopy,
    {
        let old_ctrl = *self.ctrl(slot.index);
        self.record_item_insert_at(slot.index, old_ctrl, hash);

        let bucket = self.bucket(slot.index);
        bucket.write(value);
        bucket
    }

    /// Returns a pointer to an element in the table.
    ///
    /// The caller must ensure that the `Constructor` outlives the returned [`Bucket<T>`],
    /// otherwise using it may result in [`undefined behavior`].
    ///
    /// # Safety
    ///
    /// If `mem::size_of::<T>() != 0`, then the caller of this function must observe the
    /// following safety rules:
    ///
    /// * The table must already be allocated;
    ///
    /// * The `index` must not be greater than the number returned by the [`Constructor::buckets`]
    ///   function, i.e. `(index + 1) <= self.buckets()`.
    ///
    /// It is safe to call this function with index of zero (`index == 0`) on a table that has
    /// not been allocated, but using the returned [`Bucket`] results in [`undefined behavior`].
    ///
    /// If `mem::size_of::<T>() == 0`, then the only requirement is that the `index` must
    /// not be greater than the number returned by the [`Constructor::buckets`] function, i.e.
    /// `(index + 1) <= self.buckets()`.
    ///
    /// [`Constructor::buckets`]: Constructor::buckets
    /// [`undefined behavior`]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    pub unsafe fn bucket(&mut self, index: usize) -> Bucket<'a, T> {
        // If mem::size_of::<T>() != 0 then return a pointer to the `element` in the `data part` of the table
        // (we start counting from "0", so that in the expression T[n], the "n" index actually one less than
        // the "buckets" number of our `Constructor`, i.e. "n = Constructor::buckets() - 1"):
        //
        //           `table.bucket(3).as_ptr()` returns a pointer that points here in the `data`
        //           part of the `Constructor`, i.e. to the start of T3 (see `Bucket::as_ptr`)
        //                  |
        //                  |               `base = self.data_end()` points here
        //                  |               (to the start of CT0 or to the end of T0)
        //                  v                 v
        // [Pad], T_n, ..., |T3|, T2, T1, T0, |CT0, CT1, CT2, CT3, ..., CT_n, CTa_0, CTa_1, ..., CTa_m
        //                     ^                                              \__________  __________/
        //        `table.bucket(3)` returns a pointer that points                        \/
        //         here in the `data` part of the `Constructor` (to              additional control bytes
        //         the end of T3)                                              `m = Group::WIDTH - 1`
        //
        // where: T0...T_n  - our stored data;
        //        CT0...CT_n - control bytes or metadata for `data`;
        //        CTa_0...CTa_m - additional control bytes (so that the search with loading `Group` bytes from
        //                        the heap works properly, even if the result of `h1(hash) & self.bucket_mask`
        //                        is equal to `self.bucket_mask`). See also `RawTableInner::set_ctrl` function.
        //
        // P.S. `h1(hash) & self.bucket_mask` is the same as `hash as usize % self.buckets()` because the number
        // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
        debug_assert_ne!(self.bucket_mask, 0);
        debug_assert!(index < self.buckets());
        Bucket::from_base_index(self.data_start(), index)
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
    }

    /// Searches for an empty or deleted bucket which is suitable for inserting
    /// a new element, returning the `index` for the new [`Bucket`].
    ///
    /// This function does not make any changes to the `data` part of the table,
    /// or any changes to the `items` or `growth_left` field of the table.
    ///
    /// The table must have at least 1 empty or deleted `bucket`, otherwise this
    /// function will never return (will go into an infinite loop) for tables
    /// larger than the group width, or return an index outside of the table
    /// indices range if the table is less than the group width.
    ///
    /// If there is at least 1 empty or deleted `bucket` in the table, the
    /// function is guaranteed to return [`InsertSlot`] with an index in the
    /// range `0..self.buckets()`, but in any case, if this function returns
    /// [`InsertSlot`], it will contain an index in the range
    /// `0..=self.buckets()`.
    ///
    /// # Safety
    ///
    /// The [`RawTableInner`] must have properly initialized control bytes
    /// otherwise calling this function results in [`undefined behavior`].
    ///
    /// Attempt to write data at the [`InsertSlot`] returned by this function
    /// when the table is less than the group width and if there was not at
    /// least one empty or deleted bucket in the table will cause immediate
    /// [`undefined behavior`]. This is because in this case the function will
    /// return `self.bucket_mask + 1` as an index due to the trailing [`EMPTY]
    /// control bytes outside the table range.
    ///
    /// [`undefined behavior`]:
    ///     https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn find_insert_slot(&mut self, hash: u64) -> Result<InsertSlot, Error> {
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
                    return Ok(self.fix_insert_slot(index.unwrap_unchecked()));
                }
            }

            probe_seq.move_next(self.bucket_mask)?;
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
    unsafe fn fix_insert_slot(&mut self, mut index: usize) -> InsertSlot {
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
    /// The caller must ensure that the `index` is not out of bounds of the control allocation.
    #[inline]
    unsafe fn set_ctrl_h2(&mut self, index: usize, hash: u64) {
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
    /// The caller must ensure that `index` is not out of bounds of the control
    /// allocation.
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

    /// Returns pointer to the first `data` element in the the table in `buf`.
    ///
    /// The caller must ensure that the `RawTableInner` outlives the returned
    /// [`NonNull<T>`], otherwise using it may result in [`undefined behavior`].
    ///
    /// [`undefined behavior`]:
    ///     https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    fn data_start(&mut self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.buf.as_ptr_mut().wrapping_add(self.base_ptr)).cast() }
    }

    /// Checks whether the bucket at `index` is full.
    ///
    /// # Safety
    ///
    /// The caller must ensure `index` is less than the number of buckets.
    #[inline]
    unsafe fn is_bucket_full(&mut self, index: usize) -> bool {
        debug_assert!(index < self.buckets());
        is_full(*self.ctrl(index))
    }

    /// Returns a pointer to a control byte.
    ///
    /// # Safety
    ///
    /// For the allocated [`buf`], the result is [`Undefined
    /// Behavior`], if the `index` is greater than the `self.bucket_mask + 1 +
    /// Group::WIDTH`. In that case, calling this function with `index ==
    /// self.bucket_mask + 1 + Group::WIDTH` will return a pointer to the end of
    /// the allocated table and it is useless on its own.
    ///
    /// Calling this function with `index >= self.bucket_mask + 1 +
    /// Group::WIDTH` on a table that has not been allocated results in
    /// [`Undefined Behavior`].
    ///
    /// So to satisfy both requirements you should always follow the rule that
    /// `index < self.bucket_mask + 1 + Group::WIDTH`
    ///
    /// Calling this function on [`buf`] that are not already allocated is safe
    /// for read-only purpose.
    ///
    /// See also [`Bucket::as_ptr()`] method, for more information about of
    /// properly removing or saving `data element` from / into the
    /// [`Constructor`] / [`buf`].
    ///
    /// [`Bucket::as_ptr()`]: Bucket::as_ptr()
    /// [`Undefined Behavior`]:
    ///     https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn ctrl(&mut self, index: usize) -> *mut u8 {
        debug_assert!(index < self.num_ctrl_bytes());

        // SAFETY: The caller must uphold the safety rules for the [`RawTableInner::ctrl`]
        self.buf
            .as_ptr_mut()
            .wrapping_add(self.ctrl_ptr)
            .wrapping_add(index)
    }

    #[inline]
    fn num_ctrl_bytes(&self) -> usize {
        self.bucket_mask + 1 + Group::WIDTH
    }
}

/// A reference to a hash table bucket containing a `T`.
///
/// This is usually just a pointer to the element itself. However if the element
/// is a ZST, then we instead track the index of the element in the table so
/// that `erase` works properly.
pub struct Bucket<'a, T> {
    // Actually it is pointer to next element than element itself
    // this is needed to maintain pointer arithmetic invariants
    // keeping direct pointer to element introduces difficulty.
    // Using `NonNull` for variance and niche layout
    ptr: NonNull<T>,
    _marker: PhantomData<&'a mut [u8]>,
}

impl<'a, T> Bucket<'a, T> {
    #[inline]
    unsafe fn from_base_index(base: NonNull<T>, index: usize) -> Self {
        let ptr = if mem::size_of::<T>() == 0 {
            invalid_mut(index + 1)
        } else {
            base.as_ptr().add(index)
        };

        Self {
            ptr: NonNull::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }

    /// Overwrites a memory location with the given `value` without reading or
    /// dropping the old value (like [`ptr::write`] function).
    #[inline]
    pub(crate) unsafe fn write(&self, value: &T)
    where
        T: ZeroCopy,
    {
        crate::buf::store_unaligned(self.as_ptr().cast(), value)
    }

    /// Acquires the underlying raw pointer `*mut T` to `data`.
    ///
    /// # Note
    ///
    /// If `T` is not [`Copy`], do not use `*mut T` methods that can cause calling the
    /// destructor of `T` (for example the [`<*mut T>::drop_in_place`] method), because
    /// for properly dropping the data we also need to clear `data` control bytes. If we
    /// drop data, but do not clear `data control byte` it leads to double drop when
    /// [`Constructor`] goes out of scope.
    ///
    /// If you modify an already initialized `value`, so [`Hash`] and [`Eq`] on the new
    /// `T` value and its borrowed form *must* match those for the old `T` value, as the map
    /// will not re-evaluate where the new value should go, meaning the value may become
    /// "lost" if their location does not reflect their state.
    ///
    /// [`Constructor`]: crate::raw::Constructor
    /// [`<*mut T>::drop_in_place`]: https://doc.rust-lang.org/core/primitive.pointer.html#method.drop_in_place
    /// [`Hash`]: https://doc.rust-lang.org/core/hash/trait.Hash.html
    /// [`Eq`]: https://doc.rust-lang.org/core/cmp/trait.Eq.html
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "raw")]
    /// # fn test() {
    /// use std::hash::{BuildHasher, Hash};
    ///
    /// use hashbrown::raw::{Bucket, Constructor};
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
    /// let mut table = Constructor::new();
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
    pub(crate) fn as_ptr(&self) -> *mut T {
        if mem::size_of::<T>() == 0 {
            // Just return an arbitrary ZST pointer which is properly aligned
            // invalid pointer is good enough for ZST
            invalid_mut(mem::align_of::<T>())
        } else {
            self.ptr.as_ptr()
        }
    }
}

/// A reference to an empty bucket into which an can be inserted.
pub struct InsertSlot {
    index: usize,
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
