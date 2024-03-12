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

/// TODO: Make sure allocator passes miri.
///
/// It currently cannot, since projecting from a pointer through a reference
/// inherits its provinance, which means that each reference holds onto the
/// entirety of the slice.

/// Buffer used in combination with a `Context`.
///
/// This type of allocator has a fixed capacity specified by `C` and can be
/// constructed statically.
pub struct NoStd<'a> {
    // This must be an unsafe cell, since it's mutably accessed through an
    // immutable pointers. We simply make sure that those accesses do not
    // clobber each other, which we can do since the API is restricted through
    // the `Buf` trait.
    internal: UnsafeCell<Internal>,
    // The underlying vector being borrowed.
    _marker: PhantomData<&'a mut [MaybeUninit<u8>]>,
}

impl<'a> NoStd<'a> {
    /// Build a new no-std allocator.
    pub fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            internal: UnsafeCell::new(Internal {
                free: None,
                head: None,
                tail: None,
                bytes: 0,
                regions: 0,
                size: buffer.len(),
                data: buffer.as_mut_ptr(),
            }),
            _marker: PhantomData,
        }
    }
}

impl Allocator for NoStd<'_> {
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
        unsafe {
            let i = &mut *self.internal.get();

            let region = i.region_mut(self.region.get());
            let len = (*region.ptr).len();

            // Region can fit the bytes available.
            let region = 'out: {
                // Region can fit the requested bytes.
                if (*region.ptr).cap() - len >= bytes.len() {
                    break 'out region;
                };

                let to_len = len + bytes.len();

                let Some(region) = i.realloc(self.region.get(), len, to_len) else {
                    return false;
                };

                self.region.set(region.id);
                region
            };

            let dst = i.data.wrapping_add((*region.ptr).start() + len).cast();
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
            (*region.ptr).len += bytes.len() as u32;
            true
        }
    }

    #[inline]
    fn write_buffer<B>(&mut self, other: B) -> bool
    where
        B: Buf,
    {
        let range = self.as_slice().as_ptr_range();

        'out: {
            // If this region immediately follows the other region, we can
            // optimize the write by simply growing the current region and
            // de-allocating the second since they share the same data.
            if !ptr::eq(range.end, other.as_slice().as_ptr()) {
                break 'out;
            }

            let this = self.region.get();

            unsafe {
                let i = &mut *self.internal.get();
                let this = i.region_mut(this);

                let Some(other_next) = (*this.ptr).next else {
                    break 'out;
                };

                // Prevent the other buffer from being dropped.
                forget(other);

                let other = i.region_mut(other_next);

                let next = (*other.ptr).next.take();

                (*this.ptr).cap += (*other.ptr).cap;
                (*this.ptr).len += (*other.ptr).len;
                (*this.ptr).next = next;

                if let Some(next) = next {
                    (*i.header_mut(next)).prev = Some(this.id);
                } else {
                    i.tail = Some(this.id);
                }

                *other.ptr = Header {
                    start: 0,
                    len: 0,
                    cap: 0,
                    state: State::Free,
                    next_free: i.free.replace(other.id),
                    prev: None,
                    next: None,
                };

                return true;
            }
        }

        // NB: Another optimization would be to merge the two regions if they
        // are adjacent, but this would require a copy. Which I am currently too
        // lazy to do, so just fall back to the default behavior.

        self.write(other.as_slice())
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

/// The identifier of a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    // Bytes used by regions.
    regions: usize,
    // Bytes allocated.
    bytes: usize,
    /// The size of the buffer being wrapped.
    size: usize,
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
                .wrapping_add(self.region_to_addr(at))
                .cast::<Header>()
        }
    }

    /// Get the mutable header pointer corresponding to the given id.
    #[inline]
    fn header_mut(&mut self, at: HeaderId) -> *mut Header {
        self.data
            .wrapping_add(self.region_to_addr(at))
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

    /// Allocate a region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `this` is exclusively available.
    unsafe fn alloc(&mut self, requested: usize) -> Option<Region> {
        if let Some(region) = self.find_region(|h| h.state == State::Occupy && h.cap() >= requested)
        {
            (*region.ptr).state = State::Used;
            // TODO: Should we split the allocated region if possible?
            return Some(region);
        }

        let (region, bytes, regions) = 'out: {
            if let Some(region) = self.pop_free() {
                let bytes = self.bytes.checked_add(requested)?;

                if self.regions.checked_add(bytes)? > self.size {
                    return None;
                }

                break 'out (region, bytes, self.regions);
            }

            let regions = self.regions.checked_add(size_of::<Header>())?;
            let bytes = self.bytes.checked_add(requested)?;

            if regions.checked_add(bytes)? > self.size {
                return None;
            }

            let addr = self.size - regions;
            let region = Region {
                id: self.addr_to_region(addr),
                ptr: self.data.wrapping_add(addr).cast::<Header>(),
            };
            (region, bytes, regions)
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

        self.regions = regions;
        self.bytes = bytes;

        Some(region)
    }

    unsafe fn free(&mut self, region: HeaderId) {
        let region = self.region_mut(region);

        debug_assert_eq!((*region.ptr).state, State::Used);
        debug_assert_eq!((*region.ptr).next_free, None);

        // Just free up the last region in the slab.
        if (*region.ptr).next.is_none() {
            debug_assert_eq!(self.tail, Some(region.id));

            let mut current = region;
            let mut total = 0;
            let mut prev = (*current.ptr).prev.take();

            loop {
                total += (*current.ptr).cap();

                (*current.ptr).state = State::Free;
                (*current.ptr).start = 0;
                (*current.ptr).len = 0;
                (*current.ptr).cap = 0;
                (*current.ptr).next_free = self.free.replace(current.id);

                let Some(next) = prev else {
                    self.head = None;
                    break;
                };

                current = self.region_mut(next);

                (*current.ptr).next = None;

                if (*current.ptr).state != State::Occupy {
                    break;
                }

                prev = (*current.ptr).prev.take();
            }

            self.tail = prev;
            self.bytes -= total;
            return;
        }

        let Some(prev) = (*region.ptr).prev else {
            (*region.ptr).state = State::Occupy;
            (*region.ptr).len = 0;
            return;
        };

        let prev = self.region_mut(prev);
        debug_assert!(matches!((*prev.ptr).state, State::Occupy | State::Used));

        // Move allocation to the previous region.
        let old = region.ptr.replace(Header {
            start: 0,
            len: 0,
            cap: 0,
            state: State::Free,
            next_free: self.free.replace(region.id),
            prev: None,
            next: None,
        });

        (*prev.ptr).cap += old.cap;
        (*prev.ptr).next = old.next;

        if let Some(next) = old.next {
            (*self.header_mut(next)).prev = old.prev;
        } else {
            // The current header being freed is the last in the list.
            self.bytes = (*region.ptr).start();
            self.tail = old.prev;
        }
    }

    unsafe fn realloc(&mut self, from: HeaderId, len: usize, requested: usize) -> Option<Region> {
        let from = self.region_mut(from);

        // This is the last region in the slab, so we can just expand it.
        if (*from.ptr).next.is_none() {
            let additional = requested - (*from.ptr).cap();

            if self.bytes + additional > self.size {
                return None;
            }

            (*from.ptr).cap += additional as u32;
            self.bytes += additional;
            return Some(from);
        }

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

        ptr::copy_nonoverlapping(from_data, to_data, len);
        (*to.ptr).len = len as u32;
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
    fn region_to_addr(&self, at: HeaderId) -> usize {
        self.size - (at.get() as usize) * size_of::<Header>()
    }

    #[inline]
    unsafe fn addr_to_region(&self, addr: usize) -> HeaderId {
        debug_assert!(addr < self.size);
        HeaderId::new_unchecked(((self.size - addr) / size_of::<Header>()) as u8)
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
