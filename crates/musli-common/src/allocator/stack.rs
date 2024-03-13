//! This is a no-std allocator that can be used with the `musli` crate.
//!
//! It is geared towards handling few allocations, but they can be arbitrarily
//! large. It is optimized to work best when allocations are short lived and
//! "merged back" into one previously allocated region through
//! `Buffer::write_buffer`.
//!
//! Further more its optimized to write to one allocation "at a time". So once
//! an allocation has been grown once, it will be put in a region where it is
//! unlikely to need to be moved again, usually the last region which has access
//! to the remainder of the provided buffer.
//!
//! For the moment, this allocator only supports 255 allocations to keep the
//! metadata small.
//!
//! # Design
//!
//! The allocator takes a buffer of contiguous memory. This is dynamically
//! diviced into two parts:
//!
//! * One part which grows upwards from the base, constituting the memory being
//!   allocated.
//! * Its metadata growing downward from the end of the buffer, containing
//!   headers for all allocated region.
//!
//! By designing the allocator so that the memory allocated and its metadata is
//! separate, neighbouring regions can efficiently be merged as they are written
//! or freed.
//!
//! Each allocation is sparse, meaning it does not try to over-allocate memory.
//! This ensures that subsequent regions with initialized memory can be merged
//! efficiently, but degrades performance for many small writes performed across
//! multiple allocations concurrently.
//!
//! Below is an illustration of this, where `a` and `b` are two allocations
//! where we write one byte at a time to each. Here `x` below indicates an
//! occupied `gap` in memory regions.
//!
//! ```text
//! a
//! ab
//! # a moved to end
//! xbaa
//! # b moved to 0
//! bbaa
//! # aa not moved
//! bbaaa
//! # bb moved to end
//! xxaaabbb
//! # aaa moved to 0
//! aaaaxbbb
//! # bbb not moved
//! aaaaxbbbb
//! # aaaa not moved
//! aaaaabbbb
//! # bbbbb not moved
//! aaaaabbbbb
//! # aaaaa moved to end
//! xxxxxbbbbbaaaaaa
//! # bbbbb moved to 0
//! bbbbbbxxxxaaaaaa
//! ```

#[cfg(test)]
mod tests;

use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;
use core::mem::{forget, size_of, MaybeUninit};
use core::num::NonZeroU8;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

use musli::context::Buf;

use crate::allocator::Allocator;
use crate::fixed::FixedVec;

const HEADER_U32: u32 = size_of::<Header>() as u32;

/// A buffer that can be used to store data on the stack.
pub struct StackBuffer<const C: usize> {
    data: FixedVec<u8, C>,
}

impl<const C: usize> StackBuffer<C> {
    /// Construct a new buffer.
    pub const fn new() -> Self {
        Self {
            data: FixedVec::new(),
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
        self.data.as_uninit_slice()
    }
}

impl<const C: usize> DerefMut for StackBuffer<C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut_uninit_slice()
    }
}

/// Stack-based buffer that can be used in combination with a `Context`.
///
/// This type of allocator has a fixed capacity specified by the slice passed
/// in.
///
/// To conveniently construct a buffer you can use the [`StackBuffer`] type.
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
    pub fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        assert!(
            buffer.len() <= u32::MAX as usize,
            "Buffer too large 0-{}",
            u32::MAX
        );

        Self {
            internal: UnsafeCell::new(Internal {
                free: None,
                head: None,
                tail: None,
                bytes: 0,
                headers: 0,
                occupied: 0,
                size: buffer.len() as u32,
                data: buffer.as_mut_ptr(),
            }),
            _marker: PhantomData,
        }
    }
}

impl Allocator for Stack<'_> {
    type Buf<'this> = NoStdBuf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        // SAFETY: We have exclusive access to the internal state, and it's only
        // held for the duration of this call.
        let region = unsafe { (*self.internal.get()).alloc(0)? };

        Some(NoStdBuf {
            region: Cell::new(region.id),
            internal: &self.internal,
        })
    }
}

/// A no-std allocated buffer.
pub struct NoStdBuf<'a> {
    region: Cell<HeaderId>,
    internal: &'a UnsafeCell<Internal>,
}

impl<'a> Buf for NoStdBuf<'a> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return true;
        }

        let Ok(bytes_len) = u32::try_from(bytes.len()) else {
            return false;
        };

        // SAFETY: Due to invariants in the Buffer trait we know that these
        // cannot be used incorrectly.
        unsafe {
            let i = &mut *self.internal.get();

            let region = i.region_mut(self.region.get());
            let len = (*region.ptr).len;

            // Region can fit the bytes available.
            let region = 'out: {
                // Region can fit the requested bytes.
                if (*region.ptr).cap - len >= bytes_len {
                    break 'out region;
                };

                let Ok(to_len) = u32::try_from(len + bytes_len) else {
                    return false;
                };

                let Some(region) = i.realloc(self.region.get(), len, to_len) else {
                    return false;
                };

                self.region.set(region.id);
                region
            };

            let dst = i
                .data
                .wrapping_add((*region.ptr).start())
                .wrapping_add(len as usize)
                .cast();
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
            (*region.ptr).len += bytes.len() as u32;
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
                let this = i.region_mut(self.region.get());

                debug_assert!((*this.ptr).cap >= (*this.ptr).len);

                let data_cap_ptr = this.data_cap_ptr(i.data);

                // If this region immediately follows the other region, we can
                // optimize the write by simply growing the current region and
                // de-allocating the second since the only conclusion is that
                // they share the same allocator.
                if !ptr::eq(data_cap_ptr.cast_const(), other_ptr) {
                    break 'out;
                }

                let Some(next) = (*this.ptr).next else {
                    break 'out;
                };

                // Prevent the other buffer from being dropped, since we're
                // taking care of the allocation in here directly instead.
                forget(buf);

                let next = i.region_mut(next);

                let diff = (*this.ptr).cap - (*this.ptr).len;

                // Data needs to be shuffle back to the end of the initialized
                // region.
                if diff > 0 {
                    let to_ptr = data_cap_ptr.wrapping_sub(diff as usize);
                    ptr::copy(data_cap_ptr, to_ptr, (*next.ptr).len as usize);
                }

                let next_next = (*next.ptr).next.take();

                (*this.ptr).cap += (*next.ptr).cap;
                (*this.ptr).len += (*next.ptr).len;
                (*this.ptr).next = next_next;

                if let Some(next_next) = next_next {
                    (*i.header_mut(next_next)).prev = Some(this.id);
                } else {
                    i.tail = Some(this.id);
                }

                *next.ptr = Header {
                    start: 0,
                    len: 0,
                    cap: 0,
                    state: State::Free,
                    next_free: i.free.replace(next.id),
                    prev: None,
                    next: None,
                };

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
            let header = i.header(self.region.get());
            let start = header.start();
            let ptr = i.data.wrapping_add(start).cast();
            slice::from_raw_parts(ptr, header.len())
        }
    }
}

impl Drop for NoStdBuf<'_> {
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

impl Region {
    #[inline]
    unsafe fn data_cap_ptr(&self, data: *mut MaybeUninit<u8>) -> *mut MaybeUninit<u8> {
        data.wrapping_add((*self.ptr).start())
            .wrapping_add((*self.ptr).cap())
    }

    #[inline]
    unsafe fn data_base_ptr(&self, data: *mut MaybeUninit<u8>) -> *mut MaybeUninit<u8> {
        data.wrapping_add((*self.ptr).start())
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
    // Bytes allocated.
    bytes: u32,
    // Bytes used by region headers.
    headers: u32,
    /// The number of occupied regions.
    occupied: usize,
    /// The size of the buffer being wrapped.
    size: u32,
    // The slab of regions and allocations.
    //
    // Allocated memory grows from the bottom upwards, because this allows
    // copying writes to be optimized.
    //
    // Region metadata is written to the end growing downwards.
    data: *mut MaybeUninit<u8>,
}

impl Internal {
    /// Get the header pointer corresponding to the given id.
    #[inline]
    fn header(&self, at: HeaderId) -> &Header {
        // SAFETY: Once we've coerced to `&self`, then we guarantee that we can
        // get a header immutably.
        unsafe {
            &*self
                .data
                .wrapping_add(self.region_to_addr(at) as usize)
                .cast::<Header>()
        }
    }

    /// Get the mutable header pointer corresponding to the given id.
    #[inline]
    fn header_mut(&mut self, at: HeaderId) -> *mut Header {
        self.data
            .wrapping_add(self.region_to_addr(at) as usize)
            .cast::<Header>()
    }

    /// Get the mutable region corresponding to the given id.
    #[inline]
    fn region_mut(&mut self, id: HeaderId) -> Region {
        Region {
            id,
            ptr: self.header_mut(id),
        }
    }

    /// Free a region.
    #[inline]
    unsafe fn region_free(&mut self, region: Region) -> Header {
        region.ptr.replace(Header {
            start: 0,
            len: 0,
            cap: 0,
            state: State::Free,
            next_free: self.free.replace(region.id),
            prev: None,
            next: None,
        })
    }

    /// Allocate a region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `this` is exclusively available.
    unsafe fn alloc(&mut self, requested: u32) -> Option<Region> {
        if self.occupied > 0 {
            if let Some(region) =
                self.find_region(|h| h.state == State::Occupy && h.cap >= requested)
            {
                self.occupied -= 1;
                (*region.ptr).state = State::Used;
                return Some(region);
            }
        }

        let (region, bytes, regions) = 'out: {
            if let Some(region) = self.pop_free() {
                let bytes = self.bytes.checked_add(requested)?;

                if self.headers.checked_add(bytes)? > self.size {
                    return None;
                }

                break 'out (region, bytes, self.headers);
            }

            let headers = self.headers.checked_add(HEADER_U32)?;
            let bytes = self.bytes.checked_add(requested)?;

            if headers.checked_add(bytes)? > self.size {
                return None;
            }

            let addr = self.size - headers;
            let region = Region {
                id: self.addr_to_region(addr),
                ptr: self.data.wrapping_add(addr as usize).cast::<Header>(),
            };
            (region, bytes, headers)
        };

        let start = u32::try_from(self.bytes).ok()?;
        let cap = u32::try_from(requested).ok()?;

        region.ptr.write(Header {
            start,
            len: 0,
            cap,
            state: State::Used,
            next_free: None,
            prev: None,
            next: None,
        });

        if self.head.is_none() {
            self.head = Some(region.id);
        }

        if let Some(tail) = self.tail.replace(region.id) {
            (*region.ptr).prev = Some(tail);
            (*self.header_mut(tail)).next = Some(region.id);
        }

        self.headers = regions;
        self.bytes = bytes;
        Some(region)
    }

    unsafe fn free(&mut self, region: HeaderId) {
        let region = self.region_mut(region);

        debug_assert_eq!((*region.ptr).state, State::Used);
        debug_assert_eq!((*region.ptr).next_free, None);

        // Just free up the last region in the slab.
        if (*region.ptr).next.is_none() {
            self.free_tail(region);
            return;
        }

        // If there is no previous region, then mark this region as occupy.
        let Some(prev) = (*region.ptr).prev else {
            self.occupied += 1;
            (*region.ptr).state = State::Occupy;
            (*region.ptr).len = 0;
            return;
        };

        let prev = self.region_mut(prev);
        debug_assert!(matches!((*prev.ptr).state, State::Occupy | State::Used));

        // Move allocation to the previous region.
        let old = self.region_free(region);

        (*prev.ptr).cap += old.cap;
        (*prev.ptr).next = old.next;

        if let Some(next) = old.next {
            (*self.header_mut(next)).prev = old.prev;
        } else {
            // The current header being freed is the last in the list.
            self.bytes = old.start;
            self.tail = old.prev;
        }
    }

    /// Free the tail starting at the `current` region.
    unsafe fn free_tail(&mut self, current: Region) {
        debug_assert_eq!(self.tail, Some(current.id));

        let old = self.region_free(current);
        debug_assert_eq!(old.next, None);

        let mut total = old.cap;

        'out: {
            let Some(prev) = old.prev else {
                self.head = None;
                self.tail = None;
                break 'out;
            };

            let prev = self.region_mut(prev);

            if (*prev.ptr).state != State::Occupy {
                (*prev.ptr).next = None;
                self.tail = Some(prev.id);
                break 'out;
            }

            // The prior region is occupied, so we can free that as well.
            let prev = self.region_free(prev);

            total += prev.cap;
            self.occupied -= 1;

            self.tail = prev.prev;

            if prev.prev.is_none() {
                self.head = None;
            }
        };

        self.bytes -= total;
    }

    unsafe fn realloc(&mut self, from: HeaderId, len: u32, requested: u32) -> Option<Region> {
        let from = self.region_mut(from);

        // This is the last region in the slab, so we can just expand it.
        if (*from.ptr).next.is_none() {
            let additional = requested - (*from.ptr).cap;

            if self.bytes + additional > self.size {
                return None;
            }

            (*from.ptr).cap += additional as u32;
            self.bytes += additional;
            return Some(from);
        }

        // Try to merge with a preceeding region, if the requested memory can
        // fit in it.
        'bail: {
            // Check if the immediate prior region can fit the requested allocation.
            let Some(prev) = (*from.ptr).prev else {
                break 'bail;
            };

            let prev = self.region_mut(prev);

            if (*prev.ptr).state != State::Occupy || (*prev.ptr).cap + len < requested {
                break 'bail;
            }

            let prev_ptr = prev.data_base_ptr(self.data);
            let from_ptr = from.data_base_ptr(self.data);

            let old = self.region_free(from);

            ptr::copy(from_ptr, prev_ptr, old.len());

            (*prev.ptr).state = State::Used;
            (*prev.ptr).cap += old.cap;
            (*prev.ptr).len = old.len;
            (*prev.ptr).next = old.next;

            if let Some(next) = old.next {
                (*self.header_mut(next)).prev = old.prev;
            } else {
                self.tail = old.prev;
            }

            return Some(prev);
        }

        // There is no data allocated in the current region, so we can simply
        // re-link it to the end of the chain of allocation.
        if (*from.ptr).cap == 0 {
            if self.bytes + requested > self.size {
                return None;
            }

            let start = u32::try_from(self.bytes).ok()?;
            let cap = u32::try_from(requested).ok()?;

            let prev = (*from.ptr).prev.take();
            let next = (*from.ptr).next.take();

            (*from.ptr).start = start;
            (*from.ptr).cap = cap;

            if let Some(prev) = prev {
                (*self.header_mut(prev)).next = next;
            }

            if let Some(next) = next {
                (*self.header_mut(next)).prev = prev;
            }

            if let Some(tail) = self.tail {
                let tail = self.region_mut(tail);
                (*tail.ptr).next = Some(from.id);
                (*from.ptr).prev = Some(tail.id);
            }

            if self.head == Some(from.id) {
                self.head = next;
            }

            self.tail = Some(from.id);
            self.bytes += requested;
            return Some(from);
        }

        let to = self.alloc(requested)?;

        let from_data = self
            .data
            .wrapping_add((*from.ptr).start())
            .cast::<u8>()
            .cast_const();

        let to_data = self.data.wrapping_add((*to.ptr).start()).cast::<u8>();

        ptr::copy_nonoverlapping(from_data, to_data, len as usize);
        (*to.ptr).len = len;
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

    #[inline]
    fn region_to_addr(&self, at: HeaderId) -> u32 {
        self.size - u32::from(at.get()) * HEADER_U32
    }

    #[inline]
    unsafe fn addr_to_region(&self, addr: u32) -> HeaderId {
        debug_assert!(addr < self.size);
        HeaderId::new_unchecked(((self.size - addr) / HEADER_U32) as u8)
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
#[repr(C, packed)]
struct Header {
    // Start of the allocated region as a multiple of 8.
    start: u32,
    // The length of the region.
    len: u32,
    // The capacity of the region.
    cap: u32,
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
    /// Get the start address.
    #[inline]
    fn start(&self) -> usize {
        self.start as usize
    }

    /// Get the length of the allocation.
    #[inline]
    fn len(&self) -> usize {
        self.len as usize
    }

    /// Get the capacity of the allocation.
    #[inline]
    fn cap(&self) -> usize {
        self.cap as usize
    }
}
