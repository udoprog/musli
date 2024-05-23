#[cfg(test)]
mod tests;

use core::cell::UnsafeCell;
use core::fmt::{self, Arguments};
use core::marker::PhantomData;
use core::mem::{align_of, forget, replace, MaybeUninit};
use core::num::NonZeroU16;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

use crate::buf::Error;
use crate::{Allocator, Buf};

#[cfg(test)]
macro_rules! let_mut {
    (let mut $decl:ident = $expr:expr;) => {
        let mut $decl = $expr;
    };
}

#[cfg(not(test))]
macro_rules! let_mut {
    (let mut $decl:ident = $expr:expr;) => {
        let $decl = $expr;
    };
}

#[cfg(test)]
macro_rules! if_test {
    ($($tt:tt)*) => { $($tt)* };
}

#[cfg(not(test))]
macro_rules! if_test {
    ($($tt:tt)*) => {};
}

use super::DEFAULT_STACK_BUFFER;

// We keep max bytes to 2^31, since that ensures that addition between two
// magnitutes never overflow.
const MAX_BYTES: usize = i32::MAX as usize;

/// A buffer that can be used to store data on the stack.
///
/// See the [module level documentation][super] for more information.
#[repr(align(8))]
pub struct StackBuffer<const N: usize = DEFAULT_STACK_BUFFER> {
    data: [MaybeUninit<u8>; N],
}

impl<const C: usize> StackBuffer<C> {
    /// Construct a new buffer with the default size of
    /// [`DEFAULT_STACK_BUFFER`].
    pub const fn new() -> Self {
        Self {
            // SAFETY: This is safe to initialize, since it's just an array of
            // contiguous uninitialized memory.
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl<const C: usize> Default for StackBuffer<C> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const C: usize> Deref for StackBuffer<C> {
    type Target = [MaybeUninit<u8>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const C: usize> DerefMut for StackBuffer<C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A no-std compatible fixed-memory allocator that can be used with the `musli`
/// crate.
///
/// It is geared towards handling few allocations, but they can be arbitrarily
/// large. It is optimized to work best when allocations are short lived and
/// "merged back" into one previously allocated region through
/// `Buffer::write_buffer`.
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
/// # Design
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
pub struct Stack<'a> {
    // This must be an unsafe cell, since it's mutably accessed through an
    // immutable pointers. We simply make sure that those accesses do not
    // clobber each other, which we can do since the API is restricted through
    // the `Buf` trait.
    internal: UnsafeCell<Internal>,
    // The underlying vector being borrowed.
    _marker: PhantomData<&'a mut [MaybeUninit<u8>]>,
}

impl<'a> Stack<'a> {
    /// Build a new no-std allocator.
    ///
    /// The buffer must be aligned by 8 bytes, and should be a multiple of 8 bytes.
    ///
    /// See [type-level documentation][Stack] for more information.
    ///
    /// # Panics
    ///
    /// This method panics if called with a buffer larger than 2**31 or is
    /// provided a buffer which is not aligned by 8.
    ///
    /// An easy way to align a buffer is to use [`StackBuffer`] when
    /// constructing it.
    pub fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        let size = buffer.len();

        let mut data = buffer.as_mut_ptr_range();

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
                free: None,
                #[cfg(test)]
                head: None,
                tail: None,
                occupied: None,
                start: data.start,
                end: data.end,
                free_start: data.start,
                free_end: data.end,
            }),
            _marker: PhantomData,
        }
    }
}

impl Allocator for Stack<'_> {
    type Buf<'this> = StackBuf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc_aligned<T>(&self) -> Option<Self::Buf<'_>> {
        // SAFETY: We have exclusive access to the internal state, and it's only
        // held for the duration of this call.
        let region = unsafe { (*self.internal.get()).alloc(0, align_of::<T>())? };

        Some(StackBuf {
            region: region.id,
            len: 0,
            align: align_of::<T>(),
            internal: &self.internal,
        })
    }
}

/// A no-std allocated buffer.
pub struct StackBuf<'a> {
    region: HeaderId,
    len: usize,
    align: usize,
    internal: &'a UnsafeCell<Internal>,
}

impl<'a> Buf for StackBuf<'a> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return true;
        }

        let bytes_len = bytes.len();

        if self.len + bytes_len > MAX_BYTES {
            return false;
        }

        // SAFETY: Due to invariants in the Buffer trait we know that these
        // cannot be used incorrectly.
        unsafe {
            let i = &mut *self.internal.get();

            let region = i.region(self.region);

            // Region can fit the bytes available.
            let region = 'out: {
                let requested = self.len + bytes_len;

                // Region can already fit in the requested bytes.
                if region.capacity() >= requested {
                    break 'out region;
                };

                let Some(region) = i.realloc(self.region, self.len, requested, self.align) else {
                    return false;
                };

                self.region = region.id;
                region
            };

            let dest = region.start.add(self.len).cast();
            bytes.as_ptr().copy_to_nonoverlapping(dest, bytes_len);
            self.len += bytes.len();
            true
        }
    }

    #[inline]
    fn write_buffer<B>(&mut self, buf: B) -> bool
    where
        B: Buf,
    {
        'out: {
            let other_len = buf.len();
            // NB: Placing this here to make miri happy, since accessing the
            // slice will mean mutably accessing the internal state.
            let other_ptr = buf.as_slice().as_ptr().cast();

            unsafe {
                let i = &mut *self.internal.get();
                let mut this = i.region(self.region);

                debug_assert!(this.capacity() >= self.len);

                // If this region immediately follows the other region, we can
                // optimize the write by simply growing the current region and
                // de-allocating the second since the only conclusion is that
                // they share the same allocator.
                if !ptr::eq(this.end.cast_const(), other_ptr) {
                    break 'out;
                }

                let Some(next) = this.next else {
                    break 'out;
                };

                // Prevent the other buffer from being dropped, since we're
                // taking care of the allocation in here directly instead.
                forget(buf);

                let next = i.region(next);

                let to = this.start.wrapping_add(self.len);

                // Data needs to be shuffle back to the end of the initialized
                // region.
                if this.end != to {
                    this.end.copy_to(to, other_len);
                }

                let old = i.free_region(next);
                this.end = old.end;
                self.len += other_len;
                return true;
            }
        }

        self.write(buf.as_slice())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        unsafe {
            let i = &*self.internal.get();
            let this = i.header(self.region);
            let ptr = this.start.cast();
            slice::from_raw_parts(ptr, self.len)
        }
    }

    #[inline(always)]
    fn write_fmt(&mut self, arguments: Arguments<'_>) -> Result<(), Error> {
        fmt::write(self, arguments).map_err(|_| Error)
    }
}

impl fmt::Write for StackBuf<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.write(s.as_bytes()) {
            return Err(fmt::Error);
        }

        Ok(())
    }
}

impl Drop for StackBuf<'_> {
    fn drop(&mut self) {
        // SAFETY: We have exclusive access to the internal state.
        unsafe {
            (*self.internal.get()).free(self.region);
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
    fn get(self) -> u16 {
        self.0.get()
    }
}

struct Internal {
    // The first free region.
    free: Option<HeaderId>,
    // Pointer to the head region.
    #[cfg(test)]
    head: Option<HeaderId>,
    // Pointer to the tail region.
    tail: Option<HeaderId>,
    // The occupied header region
    occupied: Option<HeaderId>,
    // The start of the allocation region.
    start: *mut MaybeUninit<u8>,
    // The end of the allocation region.
    end: *mut MaybeUninit<u8>,
    // Pointer to the start of the free region.
    free_start: *mut MaybeUninit<u8>,
    // Pointer to the end of the free region.
    free_end: *mut MaybeUninit<u8>,
}

impl Internal {
    // Return the number of allocates bytes.
    #[cfg(test)]
    #[inline]
    fn bytes(&self) -> usize {
        // SAFETY: It is guaranteed that free_end >= free_start inside of the provided region.
        unsafe { self.free_start.byte_offset_from(self.start) as usize }
    }

    #[cfg(test)]
    #[inline]
    fn headers(&self) -> usize {
        unsafe { self.end.cast::<Header>().offset_from(self.free_end.cast()) as usize }
    }

    // Return the number of remaining bytes.
    #[inline]
    fn remaining(&self) -> usize {
        // SAFETY: It is guaranteed that free_end >= free_start inside of the provided region.
        unsafe { self.free_end.byte_offset_from(self.free_start) as usize }
    }

    /// Get the header pointer corresponding to the given id.
    #[inline]
    fn header(&self, at: HeaderId) -> &Header {
        // SAFETY: Once we've coerced to `&self`, then we guarantee that we can
        // get a header immutably.
        unsafe { &*self.end.cast::<Header>().wrapping_sub(at.get() as usize) }
    }

    /// Get the mutable header pointer corresponding to the given id.
    #[inline]
    fn header_mut(&mut self, at: HeaderId) -> *mut Header {
        self.end.cast::<Header>().wrapping_sub(at.get() as usize)
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
        } else {
            if_test! {
                self.head = header.next;
            };
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

        if_test! {
            if self.head == Some(region.id) {
                self.head = next;
            }
        };

        self.push_back(region);
    }

    unsafe fn push_back(&mut self, region: &mut Region) {
        if_test! {
            if self.head.is_none() {
                self.head = Some(region.id);
            }
        }

        if let Some(tail) = self.tail.replace(region.id) {
            region.prev = Some(tail);
            (*self.region(tail).ptr).next = Some(region.id);
        }
    }

    /// Free a region.
    unsafe fn free_region(&mut self, region: Region) -> Header {
        let old = region.ptr.replace(Header {
            start: self.start,
            end: self.start,
            #[cfg(test)]
            state: State::Free,
            next: self.free.replace(region.id),
            prev: None,
        });

        self.unlink(&old);
        old
    }

    /// Allocate a new header.
    ///
    /// # Safety
    ///
    /// The caller msut ensure that there is enough space for the region and
    /// that the end pointer has been aligned to the requirements of `Header`.
    unsafe fn alloc_header(
        &mut self,
        start: *mut MaybeUninit<u8>,
        end: *mut MaybeUninit<u8>,
        #[cfg(test)] state: State,
    ) -> Option<Region> {
        if let Some(mut region) = self.pop_free() {
            region.start = start;
            region.end = end;

            if_test! {
                region.state = state;
            };

            return Some(region);
        }

        let header_ptr = self.free_end.cast::<Header>().wrapping_sub(1);

        if header_ptr < self.free_start.cast() || header_ptr >= self.free_end.cast() {
            return None;
        }

        let id = HeaderId::new(self.end.cast::<Header>().offset_from(header_ptr.cast()))?;

        let region = self.region(id);

        region.ptr.write(Header {
            start,
            end,
            #[cfg(test)]
            state,
            prev: None,
            next: None,
        });

        self.free_end = header_ptr.cast();
        Some(region)
    }

    /// Allocate a region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `this` is exclusively available.
    unsafe fn alloc(&mut self, requested: usize, align: usize) -> Option<Region> {
        if let Some(occupied) = self.occupied {
            let_mut! {
                let mut region = self.region(occupied);
            };

            if region.capacity() >= requested && region.is_aligned(align) {
                if_test! {
                    region.state = State::Used;
                };

                self.occupied = None;
                return Some(region);
            }
        }

        self.align(align)?;

        if self.remaining() < requested {
            return None;
        }

        let end = self.free_start.wrapping_add(requested);

        let mut region = self.alloc_header(
            self.free_start,
            end,
            #[cfg(test)]
            State::Used,
        )?;

        self.free_start = end;
        debug_assert!(self.free_start <= self.free_end);
        self.push_back(&mut region);
        Some(region)
    }

    /// Align the free region by the specified alignment.
    ///
    /// This might require either expanding the tail region, or introducing an
    /// occupied region which matches the number of bytes needed to fulfill the
    /// specified alignment.
    unsafe fn align(&mut self, align: usize) -> Option<()> {
        let align = self.free_start.align_offset(align);

        if align == 0 {
            return Some(());
        }

        if self.remaining() < align {
            return None;
        }

        let aligned_start = self.free_start.wrapping_add(align);

        if let Some(tail) = self.tail {
            // Simply expand the tail region to fill the gap created.
            self.region(tail).end = aligned_start;
        } else {
            // We need to construct a new occupied header to fill in the gap
            // which we just aligned from since there is no previous region to
            // expand.
            let mut region = self.alloc_header(
                self.free_start,
                aligned_start,
                #[cfg(test)]
                State::Occupy,
            )?;

            self.push_back(&mut region);
        }

        self.free_start = aligned_start;
        Some(())
    }

    unsafe fn free(&mut self, region: HeaderId) {
        let_mut! {
            let mut region = self.region(region);
        }

        #[cfg(test)]
        debug_assert_eq!(region.state, State::Used);

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

            if_test! {
                region.state = State::Occupy;
            };

            self.occupied = Some(region.id);
            return;
        };

        let mut prev = self.region(prev);

        // Move allocation to the previous region.
        let region = self.free_region(region);

        prev.end = region.end;

        // The current header being freed is the last in the list.
        if region.next.is_none() {
            self.free_start = region.start;
        }
    }

    /// Free the tail starting at the `current` region.
    unsafe fn free_tail(&mut self, current: Region) {
        debug_assert_eq!(self.tail, Some(current.id));

        let current = self.free_region(current);
        debug_assert_eq!(current.next, None);

        self.free_start = match current.prev {
            // The prior region is occupied, so we can free that as well.
            Some(prev) if self.occupied == Some(prev) => {
                self.occupied = None;
                let prev = self.region(prev);
                self.free_region(prev).start
            }
            _ => current.start,
        };
    }

    fn reserve(&mut self, additional: usize) -> Option<*mut MaybeUninit<u8>> {
        let free_start = self.free_start.wrapping_add(additional);

        if free_start > self.free_end || free_start < self.free_start {
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
            self.free_start = self.reserve(additional)?;
            from.end = from.end.add(additional);
            return Some(from);
        }

        // There is no data allocated in the current region, so we can simply
        // re-link it to the end of the chain of allocation.
        if from.start == from.end {
            let free_start = self.reserve(requested)?;
            from.start = replace(&mut self.free_start, free_start);
            from.end = free_start;
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

            let from = self.free_region(from);

            from.start.copy_to(prev.start, len);

            if_test! {
                prev.state = State::Used;
            };

            prev.end = from.end;
            self.occupied = None;
            return Some(prev);
        }

        let to = self.alloc(requested, align)?;
        from.start.copy_to_nonoverlapping(to.start, len);
        self.free(from.id);
        Some(to)
    }

    unsafe fn pop_free(&mut self) -> Option<Region> {
        let id = self.free.take()?;
        let ptr = self.header_mut(id);
        self.free = (*ptr).next.take();
        Some(Region { id, ptr })
    }
}

/// The state of an allocated region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
enum State {
    /// The region is fully free and doesn't occupy any memory.
    ///
    /// # Requirements
    ///
    /// - The range must be zero-sized at offset 0.
    /// - The region must not be linked.
    /// - The region must be in the free list.
    Free = 0,
    /// The region is occupied (only used during tests).
    ///
    /// # Requirements
    ///
    /// - The range must point to a non-zero slice of memory.,
    /// - The region must be linked.
    /// - The region must be in the occupied list.
    Occupy,
    /// The region is used by an active allocation.
    Used,
}

/// The header of a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Header {
    // The start pointer to the region.
    start: *mut MaybeUninit<u8>,
    // The end pointer of the region.
    end: *mut MaybeUninit<u8>,
    // The previous region.
    prev: Option<HeaderId>,
    // The next region.
    next: Option<HeaderId>,
    // The state of the region.
    #[cfg(test)]
    state: State,
}

impl Header {
    #[inline]
    fn capacity(&self) -> usize {
        // SAFETY: Both pointers are defined within the region.
        unsafe { self.end.byte_offset_from(self.start) as usize }
    }

    /// Test if region is aligned to `align`.
    #[inline]
    fn is_aligned(&self, align: usize) -> bool {
        self.start.align_offset(align) == 0
    }
}
