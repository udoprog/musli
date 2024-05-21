#[cfg(test)]
mod tests;

use core::cell::{Cell, UnsafeCell};
use core::fmt::{self, Arguments};
use core::marker::PhantomData;
use core::mem::{align_of, forget, replace, size_of, MaybeUninit};
use core::num::NonZeroU8;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

use crate::buf::Error;
use crate::{Allocator, Buf};

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
/// For the moment, this allocator only supports 255 unique allocations, which
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
                head: None,
                tail: None,
                headers: 0,
                occupied: 0,
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
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        // SAFETY: We have exclusive access to the internal state, and it's only
        // held for the duration of this call.
        let region = unsafe { (*self.internal.get()).alloc(0)? };

        Some(StackBuf {
            region: Cell::new(region.id),
            internal: &self.internal,
        })
    }
}

/// A no-std allocated buffer.
pub struct StackBuf<'a> {
    region: Cell<HeaderId>,
    internal: &'a UnsafeCell<Internal>,
}

impl<'a> Buf for StackBuf<'a> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return true;
        }

        if bytes.len() > MAX_BYTES {
            return false;
        }

        let bytes_len = bytes.len();

        // SAFETY: Due to invariants in the Buffer trait we know that these
        // cannot be used incorrectly.
        unsafe {
            let i = &mut *self.internal.get();

            let region = i.region(self.region.get());
            let len = region.len();

            // Region can fit the bytes available.
            let mut region = 'out: {
                // Region can already fit in the requested bytes.
                if region.capacity() - len >= bytes_len {
                    break 'out region;
                };

                let requested = len + bytes_len;

                let Some(region) = i.realloc(self.region.get(), len, requested) else {
                    return false;
                };

                self.region.set(region.id);
                region
            };

            let dst = region.start.add(len).cast();

            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
            region.len += bytes.len() as u32;
            true
        }
    }

    #[inline]
    fn write_buffer<B>(&mut self, buf: B) -> bool
    where
        B: Buf,
    {
        'out: {
            // NB: Placing this here to make miri happy, since accessing the
            // slice will mean mutably accessing the internal state.
            let other_ptr = buf.as_slice().as_ptr().cast();

            unsafe {
                let i = &mut *self.internal.get();
                let mut this = i.region(self.region.get());

                debug_assert!(this.capacity() >= this.len());

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

                let diff = this.capacity() - this.len();

                // Data needs to be shuffle back to the end of the initialized
                // region.
                if diff > 0 {
                    let to_ptr = this.end.wrapping_sub(diff);
                    ptr::copy(this.end, to_ptr, next.len());
                }

                let old = i.free_region(next);
                this.end = old.end;
                this.len += old.len;
                return true;
            }
        }

        self.write(buf.as_slice())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        unsafe {
            let i = &*self.internal.get();
            i.header(self.region.get()).len()
        }
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        unsafe {
            let i = &*self.internal.get();
            let this = i.header(self.region.get());
            let ptr = this.start.cast();
            slice::from_raw_parts(ptr, this.len())
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
            (*self.internal.get()).free(self.region.get());
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
struct HeaderId(NonZeroU8);

impl HeaderId {
    /// Create a new region identifier.
    ///
    /// # Safety
    ///
    /// The given value must be non-zero.
    #[inline]
    const unsafe fn new_unchecked(value: u8) -> Self {
        Self(NonZeroU8::new_unchecked(value))
    }

    /// Get the value of the region identifier.
    #[inline]
    fn get(self) -> u8 {
        self.0.get()
    }
}

struct Internal {
    // The first free region.
    free: Option<HeaderId>,
    // Pointer to the head region.
    head: Option<HeaderId>,
    // Pointer to the tail region.
    tail: Option<HeaderId>,
    // The number of headers in use.
    headers: u8,
    /// The number of occupied regions.
    occupied: u8,
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
            self.head = header.next;
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

        if self.head == Some(region.id) {
            self.head = next;
        }

        self.push_back(region);
    }

    unsafe fn push_back(&mut self, region: &mut Region) {
        if self.head.is_none() {
            self.head = Some(region.id);
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
            len: 0,
            state: State::Free,
            next_free: self.free.replace(region.id),
            prev: None,
            next: None,
        });

        self.unlink(&old);
        old
    }

    /// Allocate a region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `this` is exclusively available.
    unsafe fn alloc(&mut self, requested: usize) -> Option<Region> {
        if self.occupied > 0 {
            if let Some(mut region) =
                self.find_region(|h| h.state == State::Occupy && h.capacity() >= requested)
            {
                self.occupied -= 1;
                region.state = State::Used;
                return Some(region);
            }
        }

        let mut region = 'out: {
            if let Some(mut region) = self.pop_free() {
                let free_start = self.reserve(requested)?;
                region.start = replace(&mut self.free_start, free_start);
                region.end = free_start;
                region.state = State::Used;
                break 'out region;
            }

            let headers = self.headers.checked_add(1)?;

            if requested + size_of::<Header>() > self.remaining() {
                return None;
            }

            let free_start = self.free_start.wrapping_add(requested);
            let free_end = self.free_end.wrapping_sub(size_of::<Header>());

            let start = replace(&mut self.free_start, free_start);
            self.free_end = free_end;
            self.headers = headers;

            // NB: We've modified the relevant pointers just above, make sure
            // this is not moved.
            let region = self.region(HeaderId::new_unchecked(headers));

            // We need to write a full header, since we're allocating a new one.
            region.ptr.write(Header {
                start,
                end: free_start,
                len: 0,
                state: State::Used,
                next_free: None,
                prev: None,
                next: None,
            });

            region
        };

        self.push_back(&mut region);
        Some(region)
    }

    unsafe fn free(&mut self, region: HeaderId) {
        let mut region = self.region(region);

        debug_assert_eq!(region.state, State::Used);
        debug_assert_eq!(region.next_free, None);

        // Just free up the last region in the slab.
        if region.next.is_none() {
            self.free_tail(region);
            return;
        }

        // If there is no previous region, then mark this region as occupy.
        let Some(prev) = region.prev else {
            self.occupied += 1;
            region.state = State::Occupy;
            region.len = 0;
            return;
        };

        let mut prev = self.region(prev);
        debug_assert!(matches!(prev.state, State::Occupy | State::Used));

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
        self.free_start = self.free_start.wrapping_sub(current.capacity());

        let Some(prev) = current.prev else {
            return;
        };

        let prev = self.region(prev);

        // The prior region is occupied, so we can free that as well.
        if prev.state == State::Occupy {
            let prev = self.free_region(prev);
            self.free_start = self.free_start.wrapping_sub(prev.capacity());
            self.occupied -= 1;
        }
    }

    fn reserve(&mut self, additional: usize) -> Option<*mut MaybeUninit<u8>> {
        let free_start = self.free_start.wrapping_add(additional);

        if free_start > self.free_end || free_start < self.free_start {
            return None;
        }

        Some(free_start)
    }

    unsafe fn realloc(&mut self, from: HeaderId, len: usize, requested: usize) -> Option<Region> {
        let mut from = self.region(from);

        // This is the last region in the slab, so we can just expand it.
        if from.next.is_none() {
            let additional = requested - from.capacity();
            self.free_start = self.reserve(additional)?;
            from.end = from.end.add(additional);
            return Some(from);
        }

        // Try to merge with a preceeding region, if the requested memory can
        // fit in it.
        'bail: {
            // Check if the immediate prior region can fit the requested allocation.
            let Some(prev) = from.prev else {
                break 'bail;
            };

            let mut prev = self.region(prev);

            if prev.state != State::Occupy || prev.capacity() + len < requested {
                break 'bail;
            }

            let from_ptr = from.start.cast_const();
            let prev_ptr = prev.start;

            let from = self.free_region(from);

            ptr::copy(from_ptr, prev_ptr, from.len());

            prev.state = State::Used;
            prev.end = from.end;
            prev.len = from.len;
            return Some(prev);
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

        let mut to = self.alloc(requested)?;

        let from_data = from.start.cast::<u8>().cast_const();
        let to_data = to.start.cast::<u8>();

        ptr::copy_nonoverlapping(from_data, to_data, len);
        to.len = len as u32;
        self.free(from.id);
        Some(to)
    }

    unsafe fn find_region<T>(&mut self, mut condition: T) -> Option<Region>
    where
        T: FnMut(&Header) -> bool,
    {
        let mut next = self.head;

        while let Some(id) = next {
            let ptr = self.header_mut(id);

            if condition(&*ptr) {
                return Some(Region { id, ptr });
            }

            next = (*ptr).next;
        }

        None
    }

    unsafe fn pop_free(&mut self) -> Option<Region> {
        let id = self.free.take()?;
        let ptr = self.header_mut(id);
        self.free = (*ptr).next_free.take();
        Some(Region { id, ptr })
    }
}

/// The state of an allocated region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum State {
    /// The region is fully free and doesn't occupy any memory.
    ///
    /// # Requirements
    ///
    /// - The range must be zero-sized at offset 0.
    /// - The region must not be linked.
    /// - The region must be in the free list.
    Free = 0,
    /// The region is occupied.
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
#[repr(align(8))]
struct Header {
    // The start pointer to the region.
    start: *mut MaybeUninit<u8>,
    // The end pointer of the region.
    end: *mut MaybeUninit<u8>,
    // Number of initialized bytes in the region.
    len: u32,
    // The state of the region.
    state: State,
    // Link to the next free region.
    next_free: Option<HeaderId>,
    // The previous neighbouring region.
    prev: Option<HeaderId>,
    // The next neighbouring region.
    next: Option<HeaderId>,
}

impl Header {
    #[inline]
    fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    fn capacity(&self) -> usize {
        // SAFETY: Both pointers are defined within the region.
        unsafe { self.end.byte_offset_from(self.start) as usize }
    }
}
