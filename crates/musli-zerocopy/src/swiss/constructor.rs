// Note: This was ported from `hashbrown`, and still contains some of the
// comments assuming that it performs internal allocations. These are most
// likely wrong and need to be rewritten to take into account the safety
// requirements towards `OwnedBuf`.

use core::convert::{identity as likely, identity as unlikely};
use core::marker::PhantomData;
use core::mem::size_of;

use crate::buf::Buf;
use crate::buf::OwnedBuf;
use crate::endian::ByteOrder;
use crate::error::{Error, ErrorKind};
use crate::pointer::Size;
use crate::swiss::raw::{h2, is_full, probe_seq, special_is_empty, Group, ProbeSeq};
use crate::traits::ZeroCopy;

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
    pub(crate) fn with_buf(
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

    /// Get the length of the table.
    pub(crate) fn len(&self) -> usize {
        self.bucket_mask - self.growth_left
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

    /// Inserts a new element into the table in the given slot, and returns its
    /// raw bucket.
    #[inline]
    pub fn insert_in_slot(&mut self, hash: u64, slot: InsertSlot, value: &T) -> Bucket<'_, T>
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
    #[inline]
    pub fn bucket(&mut self, index: usize) -> Bucket<'_, T> {
        debug_assert_ne!(self.bucket_mask, 0);
        debug_assert!(index < self.buckets());

        let Some(index) = index.checked_mul(size_of::<T>()) else {
            panic!("Index `{index}` out of bounds");
        };

        let Some(start) = self.base_ptr.checked_add(index) else {
            panic!("Start `{index}` out of bounds");
        };

        let end = start.wrapping_add(size_of::<T>());

        let Some(slice) = self.buf.get_mut(start..end) else {
            panic!("Missing bucket at range {start}..{end}");
        };

        // SAFETY: We've ensure that the bucket is appropriately sized just
        // above.
        unsafe { Bucket::from_slice(slice) }
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
        let bit = likely(group.match_empty_or_deleted().lowest_set_bit()?);

        // This is the same as `(probe_seq.pos + bit) % self.buckets()` because the number
        // of buckets is a power of two, and `self.bucket_mask = self.buckets() - 1`.
        Some((probe_seq.pos + bit) & self.bucket_mask)
    }

    #[inline]
    fn record_item_insert_at(&mut self, index: usize, old_ctrl: u8, hash: u64) {
        self.growth_left -= usize::from(special_is_empty(old_ctrl));
        self.set_ctrl_h2(index, hash);
    }

    /// Searches for an empty or deleted bucket which is suitable for inserting
    /// a new element, returning the `index` for the new [`Bucket`].
    #[inline]
    fn find_insert_slot(&mut self, hash: u64) -> Result<InsertSlot, Error> {
        let mut probe_seq = probe_seq(self.bucket_mask, hash);

        loop {
            let group = unsafe { Group::load(self.ctrl_group(probe_seq.pos).as_ptr()) };

            if let Some(index) = self.find_insert_slot_in_group(&group, &probe_seq) {
                return Ok(self.fix_insert_slot(index));
            };

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
    fn fix_insert_slot(&mut self, mut index: usize) -> InsertSlot {
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
            index = unsafe {
                Group::load_aligned(self.ctrl_group(0).as_ptr())
                    .match_empty_or_deleted()
                    .lowest_set_bit()
                    .unwrap_unchecked()
            };
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
    fn set_ctrl_h2(&mut self, index: usize, hash: u64) {
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
    fn set_ctrl(&mut self, index: usize, ctrl: u8) {
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
        *self.ctrl_mut(index) = ctrl;
        *self.ctrl_mut(index2) = ctrl;
    }

    /// Checks whether the bucket at `index` is full.
    ///
    /// # Safety
    ///
    /// The caller must ensure `index` is less than the number of buckets.
    #[inline]
    fn is_bucket_full(&self, index: usize) -> bool {
        debug_assert!(index < self.buckets());
        is_full(*self.ctrl(index))
    }

    #[inline]
    fn ctrl_mut(&mut self, index: usize) -> &mut u8 {
        debug_assert!(index < self.num_ctrl_bytes());

        let offset = self.ctrl_ptr.wrapping_add(index);

        let Some(ctrl) = self.buf.get_mut(offset) else {
            panic!("Missing control byte at {offset}");
        };

        ctrl
    }

    #[inline]
    fn ctrl(&self, index: usize) -> &u8 {
        debug_assert!(index < self.num_ctrl_bytes());

        let offset = self.ctrl_ptr.wrapping_add(index);

        let Some(ctrl) = self.buf.get(offset) else {
            panic!("Missing control byte at {offset}");
        };

        ctrl
    }

    #[inline]
    fn ctrl_group(&self, index: usize) -> &[u8] {
        debug_assert!(index < self.num_ctrl_bytes());

        let start = self.ctrl_ptr.wrapping_add(index);
        let end = start.wrapping_add(Group::WIDTH);

        let Some(ctrl) = self.buf.get(start..end) else {
            panic!("Missing control byte at range {start}..{end}");
        };

        ctrl
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
    data: *mut u8,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Bucket<'a, T> {
    /// Construct a bucket from the given slice.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the slice is sized for `T`.
    #[inline]
    unsafe fn from_slice(data: &'a mut [u8]) -> Self {
        debug_assert!(data.len() == size_of::<T>());

        Self {
            data: data.as_mut_ptr(),
            _marker: PhantomData,
        }
    }

    /// Overwrites a memory location with the given `value` without reading or
    /// dropping the old value (like [`ptr::write`] function).
    #[inline]
    pub(crate) fn write(&self, value: &T)
    where
        T: ZeroCopy,
    {
        // SAFETY: During bucket construction we've asserted that a buffer of
        // the appropriate size (that is not guaranteed to be aligned) is used.
        unsafe { crate::buf::store_unaligned(self.data, value) }
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
