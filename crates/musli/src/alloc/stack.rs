#[cfg(test)]
mod tests;

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::{align_of, forget, replace, size_of, MaybeUninit};
use core::num::NonZeroU16;
use core::ops::{Deref, DerefMut};
use core::ptr;

use super::{Allocator, RawVec};

// We keep max bytes to 2^31, since that ensures that addition between two
// magnitutes never overflow.
const MAX_BYTES: usize = i32::MAX as usize;

/// A no-std compatible slice-based allocator that can be used with the `musli`
/// crate.
///
/// It is geared towards handling few allocations, but they can be arbitrarily
/// large. It is optimized to work best when allocations are short lived and
/// "merged back" into one previously allocated region through
/// `Buffer::extend`.
///
/// It's also optimized to write to one allocation "at a time". So once an
/// allocation has been grown once, it will be put in a region where it is
/// unlikely to need to be moved again, usually the last region which has access
/// to the remainder of the provided buffer.
///
/// For the moment, this allocator only supports 65535 unique allocations, which
/// is fine for use with the `musli` crate, but might be a limitation for other
/// use-cases.
///
/// ## Examples
///
/// ```
/// use musli::alloc::{ArrayBuffer, Slice, Vec};
///
/// let mut buf = ArrayBuffer::new();
/// let alloc = Slice::new(&mut buf);
///
/// let mut a = Vec::new_in(&alloc);
/// let mut b = Vec::new_in(&alloc);
///
/// b.write(b"He11o");
/// a.write(b.as_slice());
///
/// assert_eq!(a.as_slice(), b"He11o");
/// assert_eq!(a.len(), 5);
///
/// a.write(b" W0rld");
///
/// assert_eq!(a.as_slice(), b"He11o W0rld");
/// assert_eq!(a.len(), 11);
///
/// let mut c = Vec::new_in(&alloc);
/// c.write(b"!");
/// a.write(c.as_slice());
///
/// assert_eq!(a.as_slice(), b"He11o W0rld!");
/// assert_eq!(a.len(), 12);
/// ```
///
/// ## Design
///
/// The allocator takes a buffer of contiguous memory. This is dynamically
/// diviced into two parts:
///
/// * One part which grows upwards from the base, constituting the memory being
///   allocated.
/// * Its metadata growing downward from the end of the buffer, containing
///   headers for all allocated region.
///
/// By designing the allocator so that the memory allocated and its metadata is
/// separate, neighbouring regions can efficiently be merged as they are written
/// or freed.
///
/// Each allocation is sparse, meaning it does not try to over-allocate memory.
/// This ensures that subsequent regions with initialized memory can be merged
/// efficiently, but degrades performance for many small writes performed across
/// multiple allocations concurrently.
///
/// Below is an illustration of this, where `a` and `b` are two allocations
/// where we write one byte at a time to each. Here `x` below indicates an
/// occupied `gap` in memory regions.
///
/// ```text
/// a
/// ab
/// # a moved to end
/// xbaa
/// # b moved to 0
/// bbaa
/// # aa not moved
/// bbaaa
/// # bb moved to end
/// xxaaabbb
/// # aaa moved to 0
/// aaaaxbbb
/// # bbb not moved
/// aaaaxbbbb
/// # aaaa not moved
/// aaaaabbbb
/// # bbbbb not moved
/// aaaaabbbbb
/// # aaaaa moved to end
/// xxxxxbbbbbaaaaaa
/// # bbbbb moved to 0
/// bbbbbbxxxxaaaaaa
/// ```
pub struct Slice<'a> {
    // This must be an unsafe cell, since it's mutably accessed through an
    // immutable pointers. We simply make sure that those accesses do not
    // clobber each other, which we can do since the API is restricted through
    // the `RawVec` trait.
    internal: UnsafeCell<Internal>,
    // The underlying vector being borrowed.
    _marker: PhantomData<&'a mut [MaybeUninit<u8>]>,
}

impl<'a> Slice<'a> {
    /// Construct a new slice allocator.
    ///
    /// See [type-level documentation][Slice] for more information.
    ///
    /// # Panics
    ///
    /// This panics if called with a buffer larger than `2^31` bytes.
    pub fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        let size = buffer.len();
        assert!(
            size <= MAX_BYTES,
            "Buffer of {size} bytes is larger than the maximum {MAX_BYTES}"
        );

        let mut data = Range::new(buffer.as_mut_ptr_range());

        // The region of allocated headers grows downwards from the end of the
        // buffer, so in order to ensure headers are aligned, we simply align
        // the end pointer of the buffer preemptively here. Then we don't have
        // to worry about it.
        let align = data.end.align_offset(align_of::<Header>());

        if align != 0 {
            let sub = align_of::<Header>() - align;

            if sub <= size {
                // SAFETY: We've ensured that the adjustment is less than the
                // size of the buffer.
                unsafe {
                    data.end = data.end.sub(sub);
                }
            } else {
                data.end = data.start;
            }
        }

        Self {
            internal: UnsafeCell::new(Internal {
                free_head: None,
                tail: None,
                occupied: None,
                full: data,
                free: data,
            }),
            _marker: PhantomData,
        }
    }
}

impl Allocator for Slice<'_> {
    type RawVec<'this, T> = SliceBuf<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn new_raw_vec<'a, T>(&'a self) -> Self::RawVec<'a, T>
    where
        T: 'a,
    {
        // SAFETY: We have exclusive access to the internal state, and it's only
        // held for the duration of this call.
        let region = if size_of::<T>() == 0 {
            None
        } else {
            unsafe {
                (*self.internal.get())
                    .alloc(0, align_of::<T>())
                    .map(|r| r.id)
            }
        };

        SliceBuf {
            region,
            internal: &self.internal,
            _marker: PhantomData,
        }
    }
}

/// A slice allocated buffer.
///
/// See [`Slice`].
pub struct SliceBuf<'a, T> {
    region: Option<HeaderId>,
    internal: &'a UnsafeCell<Internal>,
    _marker: PhantomData<T>,
}

impl<'a, T> RawVec<T> for SliceBuf<'a, T> {
    #[inline]
    fn resize(&mut self, len: usize, additional: usize) -> bool {
        if additional == 0 || size_of::<T>() == 0 {
            return true;
        }

        let Some(region_id) = self.region else {
            return false;
        };

        let Some(len) = len.checked_mul(size_of::<T>()) else {
            return false;
        };

        let Some(additional) = additional.checked_mul(size_of::<T>()) else {
            return false;
        };

        let Some(requested) = len.checked_add(additional) else {
            return false;
        };

        if requested > MAX_BYTES {
            return false;
        }

        // SAFETY: Due to invariants in the Buffer trait we know that these
        // cannot be used incorrectly.
        unsafe {
            let i = &mut *self.internal.get();

            let region = i.region(region_id);

            // Region can already fit in the requested bytes.
            if region.capacity() >= requested {
                return true;
            };

            let Some(region) = i.realloc(region_id, len, requested, align_of::<T>()) else {
                return false;
            };

            self.region = Some(region.id);
            true
        }
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        let Some(region) = self.region else {
            return ptr::NonNull::dangling().as_ptr();
        };

        unsafe {
            let i = &*self.internal.get();
            let this = i.header(region);
            this.range.start.cast_const().cast()
        }
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        let Some(region) = self.region else {
            return ptr::NonNull::dangling().as_ptr();
        };

        unsafe {
            let i = &*self.internal.get();
            let this = i.header(region);
            this.range.start.cast()
        }
    }

    #[inline]
    fn try_merge<B>(&mut self, this_len: usize, buf: B, other_len: usize) -> Result<(), B>
    where
        B: RawVec<T>,
    {
        let Some(region) = self.region else {
            return Err(buf);
        };

        let this_len = this_len * size_of::<T>();
        let other_len = other_len * size_of::<T>();

        // NB: Placing this here to make miri happy, since accessing the
        // slice will mean mutably accessing the internal state.
        let other_ptr = buf.as_ptr().cast();

        unsafe {
            let i = &mut *self.internal.get();
            let mut this = i.region(region);

            debug_assert!(this.capacity() >= this_len);

            // If this region immediately follows the other region, we can
            // optimize the write by simply growing the current region and
            // de-allocating the second since the only conclusion is that
            // they share the same allocator.
            if !ptr::eq(this.range.end.cast_const(), other_ptr) {
                return Err(buf);
            }

            let Some(next) = this.next else {
                return Err(buf);
            };

            // Prevent the other buffer from being dropped, since we're
            // taking care of the allocation in here directly instead.
            forget(buf);

            let next = i.region(next);

            let to = this.range.start.wrapping_add(this_len);

            // Data needs to be shuffle back to the end of the initialized
            // region.
            if this.range.end != to {
                this.range.end.copy_to(to, other_len);
            }

            let old = i.free_region(next);
            this.range.end = old.range.end;
            Ok(())
        }
    }
}

impl<T> Drop for SliceBuf<'_, T> {
    fn drop(&mut self) {
        if let Some(region) = self.region.take() {
            // SAFETY: We have exclusive access to the internal state.
            unsafe {
                (*self.internal.get()).free(region);
            }
        }
    }
}

struct Region {
    id: HeaderId,
    ptr: *mut Header,
}

impl Deref for Region {
    type Target = Header;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: Construction of the region is unsafe, so the caller must
        // ensure that it's used correctly after that.
        unsafe { &*self.ptr }
    }
}

impl DerefMut for Region {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Construction of the region is unsafe, so the caller must
        // ensure that it's used correctly after that.
        unsafe { &mut *self.ptr }
    }
}

/// The identifier of a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(PartialOrd, Ord, Hash))]
#[repr(transparent)]
struct HeaderId(NonZeroU16);

impl HeaderId {
    #[cfg(test)]
    const unsafe fn new_unchecked(value: u16) -> Self {
        Self(NonZeroU16::new_unchecked(value))
    }

    /// Create a new region identifier.
    ///
    /// # Safety
    ///
    /// The given value must be non-zero.
    #[inline]
    fn new(value: isize) -> Option<Self> {
        Some(Self(NonZeroU16::new(u16::try_from(value).ok()?)?))
    }

    /// Get the value of the region identifier.
    #[inline]
    fn get(self) -> usize {
        self.0.get() as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    start: *mut MaybeUninit<u8>,
    end: *mut MaybeUninit<u8>,
}

impl Range {
    fn new(range: core::ops::Range<*mut MaybeUninit<u8>>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    fn head(self) -> Range {
        Self {
            start: self.start,
            end: self.start,
        }
    }
}

struct Internal {
    // The first free region.
    free_head: Option<HeaderId>,
    // Pointer to the tail region.
    tail: Option<HeaderId>,
    // The occupied header region
    occupied: Option<HeaderId>,
    // The full range used by the allocator.
    full: Range,
    // The free range available to the allocator.
    free: Range,
}

impl Internal {
    // Return the number of allocates bytes.
    #[cfg(test)]
    #[inline]
    fn bytes(&self) -> usize {
        // SAFETY: It is guaranteed that free_end >= free_start inside of the provided region.
        unsafe { self.free.start.byte_offset_from(self.full.start) as usize }
    }

    #[cfg(test)]
    #[inline]
    fn headers(&self) -> usize {
        unsafe {
            self.full
                .end
                .cast::<Header>()
                .offset_from(self.free.end.cast()) as usize
        }
    }

    // Return the number of remaining bytes.
    #[inline]
    fn remaining(&self) -> usize {
        // SAFETY: It is guaranteed that free_end >= free_start inside of the provided region.
        unsafe { self.free.end.byte_offset_from(self.free.start) as usize }
    }

    /// Get the header pointer corresponding to the given id.
    #[inline]
    fn header(&self, at: HeaderId) -> &Header {
        // SAFETY: Once we've coerced to `&self`, then we guarantee that we can
        // get a header immutably.
        unsafe { &*self.full.end.cast::<Header>().wrapping_sub(at.get()) }
    }

    /// Get the mutable header pointer corresponding to the given id.
    #[inline]
    fn header_mut(&mut self, at: HeaderId) -> *mut Header {
        self.full.end.cast::<Header>().wrapping_sub(at.get())
    }

    /// Get the mutable region corresponding to the given id.
    #[inline]
    fn region(&mut self, id: HeaderId) -> Region {
        Region {
            id,
            ptr: self.header_mut(id),
        }
    }

    unsafe fn unlink(&mut self, header: &Header) {
        if let Some(next) = header.next {
            (*self.header_mut(next)).prev = header.prev;
        } else {
            self.tail = header.prev;
        }

        if let Some(prev) = header.prev {
            (*self.header_mut(prev)).next = header.next;
        }
    }

    unsafe fn replace_back(&mut self, region: &mut Region) {
        let prev = region.prev.take();
        let next = region.next.take();

        if let Some(prev) = prev {
            (*self.header_mut(prev)).next = next;
        }

        if let Some(next) = next {
            (*self.header_mut(next)).prev = prev;
        }

        self.push_back(region);
    }

    unsafe fn push_back(&mut self, region: &mut Region) {
        if let Some(tail) = self.tail.replace(region.id) {
            region.prev = Some(tail);
            (*self.header_mut(tail)).next = Some(region.id);
        }
    }

    /// Free a region.
    unsafe fn free_region(&mut self, region: Region) -> Header {
        self.unlink(&region);

        region.ptr.replace(Header {
            range: self.full.head(),
            next: self.free_head.replace(region.id),
            prev: None,
        })
    }

    /// Allocate a new header.
    ///
    /// # Safety
    ///
    /// The caller msut ensure that there is enough space for the region and
    /// that the end pointer has been aligned to the requirements of `Header`.
    unsafe fn alloc_header(&mut self, end: *mut MaybeUninit<u8>) -> Option<Region> {
        if let Some(region) = self.free_head.take() {
            let mut region = self.region(region);

            region.range.start = self.free.start;
            region.range.end = end;

            return Some(region);
        }

        debug_assert_eq!(
            self.free.end.align_offset(align_of::<Header>()),
            0,
            "End pointer should be aligned to header"
        );

        let ptr = self.free.end.cast::<Header>().wrapping_sub(1);

        if ptr < self.free.start.cast() || ptr >= self.free.end.cast() {
            return None;
        }

        let id = HeaderId::new(self.full.end.cast::<Header>().offset_from(ptr))?;

        ptr.write(Header {
            range: Range::new(self.free.start..end),
            prev: None,
            next: None,
        });

        self.free.end = ptr.cast();
        Some(Region { id, ptr })
    }

    /// Allocate a region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `this` is exclusively available.
    unsafe fn alloc(&mut self, requested: usize, align: usize) -> Option<Region> {
        if let Some(occupied) = self.occupied {
            let region = self.region(occupied);

            if region.capacity() >= requested && region.is_aligned(align) {
                self.occupied = None;
                return Some(region);
            }
        }

        self.align(align)?;

        if self.remaining() < requested {
            return None;
        }

        let end = self.free.start.wrapping_add(requested);

        let mut region = self.alloc_header(end)?;

        self.free.start = end;
        debug_assert!(self.free.start <= self.free.end);
        self.push_back(&mut region);
        Some(region)
    }

    /// Align the free region by the specified alignment.
    ///
    /// This might require either expanding the tail region, or introducing an
    /// occupied region which matches the number of bytes needed to fulfill the
    /// specified alignment.
    unsafe fn align(&mut self, align: usize) -> Option<()> {
        let align = self.free.start.align_offset(align);

        if align == 0 {
            return Some(());
        }

        if self.remaining() < align {
            return None;
        }

        let aligned_start = self.free.start.wrapping_add(align);

        if let Some(tail) = self.tail {
            // Simply expand the tail region to fill the gap created.
            self.region(tail).range.end = aligned_start;
        } else {
            // We need to construct a new occupied header to fill in the gap
            // which we just aligned from since there is no previous region to
            // expand.
            let mut region = self.alloc_header(aligned_start)?;

            self.push_back(&mut region);
        }

        self.free.start = aligned_start;
        Some(())
    }

    unsafe fn free(&mut self, region: HeaderId) {
        let region = self.region(region);

        // Just free up the last region in the slab.
        if region.next.is_none() {
            self.free_tail(region);
            return;
        }

        // If there is no previous region, then mark this region as occupy.
        let Some(prev) = region.prev else {
            debug_assert!(
                self.occupied.is_none(),
                "There can only be one occupied region"
            );

            self.occupied = Some(region.id);
            return;
        };

        let mut prev = self.region(prev);

        // Move allocation to the previous region.
        let region = self.free_region(region);

        prev.range.end = region.range.end;

        // The current header being freed is the last in the list.
        if region.next.is_none() {
            self.free.start = region.range.start;
        }
    }

    /// Free the tail starting at the `current` region.
    unsafe fn free_tail(&mut self, current: Region) {
        debug_assert_eq!(self.tail, Some(current.id));

        let current = self.free_region(current);
        debug_assert_eq!(current.next, None);

        self.free.start = match current.prev {
            // The prior region is occupied, so we can free that as well.
            Some(prev) if self.occupied == Some(prev) => {
                self.occupied = None;
                let prev = self.region(prev);
                self.free_region(prev).range.start
            }
            _ => current.range.start,
        };
    }

    unsafe fn reserve(&mut self, additional: usize, align: usize) -> Option<*mut MaybeUninit<u8>> {
        self.align(align)?;

        let free_start = self.free.start.wrapping_add(additional);

        if free_start > self.free.end || free_start < self.free.start {
            return None;
        }

        Some(free_start)
    }

    unsafe fn realloc(
        &mut self,
        from: HeaderId,
        len: usize,
        requested: usize,
        align: usize,
    ) -> Option<Region> {
        let mut from = self.region(from);

        // This is the last region in the slab, so we can just expand it.
        if from.next.is_none() {
            // Before we call realloc, we check the capacity of the current
            // region. So we know that it is <= requested.
            let additional = requested - from.capacity();
            self.free.start = self.reserve(additional, align)?;
            from.range.end = from.range.end.add(additional);
            return Some(from);
        }

        // There is no data allocated in the current region, so we can simply
        // re-link it to the end of the chain of allocation.
        if from.range.start == from.range.end {
            let free_start = self.reserve(requested, align)?;
            from.range.start = replace(&mut self.free.start, free_start);
            from.range.end = free_start;
            self.replace_back(&mut from);
            return Some(from);
        }

        // Try to merge with a preceeding region, if the requested memory can
        // fit in it.
        'bail: {
            // Check if the immediate prior region can fit the requested allocation.
            let Some(prev) = from.prev else {
                break 'bail;
            };

            if self.occupied != Some(prev) {
                break 'bail;
            }

            let mut prev = self.region(prev);

            if prev.capacity() + from.capacity() < requested {
                break 'bail;
            }

            if !prev.is_aligned(align) {
                break 'bail;
            }

            let from = self.free_region(from);

            from.range.start.copy_to(prev.range.start, len);
            prev.range.end = from.range.end;
            self.occupied = None;
            return Some(prev);
        }

        let to = self.alloc(requested, align)?;
        from.range.start.copy_to_nonoverlapping(to.range.start, len);
        self.free(from.id);
        Some(to)
    }
}

/// The header of a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Header {
    // The range of the allocated region.
    range: Range,
    // The previous region.
    prev: Option<HeaderId>,
    // The next region.
    next: Option<HeaderId>,
}

impl Header {
    #[inline]
    fn capacity(&self) -> usize {
        // SAFETY: Both pointers are defined within the region.
        unsafe { self.range.end.byte_offset_from(self.range.start) as usize }
    }

    /// Test if region is aligned to `align`.
    #[inline]
    fn is_aligned(&self, align: usize) -> bool {
        self.range.start.align_offset(align) == 0
    }
}
