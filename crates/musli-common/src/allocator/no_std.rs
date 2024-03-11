#[cfg(test)]
mod tests;

use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;
use core::mem::{size_of, ManuallyDrop, MaybeUninit};
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
        let (region, _) = unsafe { (*self.internal.get()).alloc(0)? };

        Some(NoStdBuf {
            region: Cell::new(region),
            len: Cell::new(0),
            internal: &self.internal,
        })
    }
}

/// A no-std allocated buffer.
pub struct NoStdBuf<'a> {
    region: Cell<Region>,
    len: Cell<usize>,
    internal: &'a UnsafeCell<Internal>,
}

impl<'a> Buf for NoStdBuf<'a> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> bool {
        unsafe {
            let len = self.len.get();

            let i = &mut *self.internal.get();

            let header_ptr = i.header_mut(self.region.get());

            // Region can fit the bytes available.
            let header_ptr = if (*header_ptr).size() - len < bytes.len() {
                let to_len = len + bytes.len();

                let Some(region) = i.realloc(self.region.get(), len, to_len) else {
                    return false;
                };

                self.region.set(region);
                i.header_mut(region)
            } else {
                header_ptr
            };

            let dst = i.data.wrapping_add((*header_ptr).start() + len).cast();
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
            self.len.set(len + bytes.len());
            true
        }
    }

    #[inline]
    fn write_buffer(&mut self, other: Self) -> bool
    where
        Self: Sized,
    {
        // If not pointing to the same internal allocator, fallback to default
        // behavior.
        if !ptr::eq(self.internal, other.internal) {
            return self.write(other.as_slice());
        }

        // Prevent the other buffer from being dropped.
        let other = ManuallyDrop::new(other);

        unsafe {
            let i = &mut *self.internal.get();

            let this_ptr = &mut *i.header_mut(self.region.get());
            let other_region = other.region.get();
            let other_ptr = i.header_mut(other_region);

            // If this region immediately follows the other region, we can
            // optimize the write by simply growing the current region and
            // de-allocating the second since they share the same data.
            if this_ptr.start + this_ptr.size == (*other_ptr).start {
                this_ptr.size += (*other_ptr).size;
                self.len.set(self.len.get() + other.len.get());

                other_ptr.write(Header {
                    start: 0,
                    size: 0,
                    state: State::Free,
                    next_free: i.free.replace(other_region),
                    prev: None,
                    next: None,
                });

                return true;
            }
        }

        // NB: Another optimization would be to merge the two regions if they
        // are adjacent, but this would require a copy. Which I am currently too
        // lazy to do, so just fall back to the default behavior.

        let other = ManuallyDrop::into_inner(other);
        self.write(other.as_slice())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len.get()
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        unsafe {
            let i = &*self.internal.get();
            let start = i.header(self.region.get()).start();
            let data = i.data.wrapping_add(start).cast();
            slice::from_raw_parts(data, self.len.get())
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

/// The identifier of a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Region(NonZeroU8);

impl Region {
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
    free: Option<Region>,
    // Pointer to the head region.
    head: Option<Region>,
    // Pointer to the tail region.
    tail: Option<Region>,
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
    /// Get the header pointer corresponding to the given index.
    #[inline]
    fn header(&self, at: Region) -> &Header {
        // SAFETY: Once we've coerced to `&self`, then we guarantee that we can
        // get a header immutably.
        unsafe {
            &*self
                .data
                .wrapping_add(self.region_to_addr(at))
                .cast::<Header>()
        }
    }

    /// Get the mutable header pointer corresponding to the given index.
    #[inline]
    fn header_mut(&mut self, at: Region) -> *mut Header {
        self.data
            .wrapping_add(self.region_to_addr(at))
            .cast::<Header>()
    }

    /// Allocate a region.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `this` is exclusively available.
    unsafe fn alloc(&mut self, requested: usize) -> Option<(Region, *mut Header)> {
        if let Some((region, header_ptr)) =
            self.find_region(|h| h.state == State::Occupy && h.size() >= requested)
        {
            (*header_ptr).state = State::Used;
            // TODO: Should we split the allocated region if possible?
            return Some((region, header_ptr));
        }

        let (region, header_ptr, bytes, regions) = 'out: {
            if let Some((region, header_ptr)) = self.pop_free() {
                let bytes = self.bytes.checked_add(requested)?;

                if self.regions.checked_add(bytes)? > self.size {
                    return None;
                }

                break 'out (region, header_ptr, bytes, self.regions);
            }

            let regions = self.regions.checked_add(size_of::<Header>())?;
            let bytes = self.bytes.checked_add(requested)?;

            if regions.checked_add(bytes)? > self.size {
                return None;
            }

            let addr = self.size - regions;
            let region = self.addr_to_region(addr);
            let header_ptr = self.data.wrapping_add(addr).cast::<Header>();
            (region, header_ptr, bytes, regions)
        };

        let start = u32::try_from(self.bytes).ok()?;
        let size = u32::try_from(requested).ok()?;

        header_ptr.write(Header {
            start,
            size,
            state: State::Used,
            next_free: None,
            prev: None,
            next: None,
        });

        if self.head.is_none() {
            self.head = Some(region);
        }

        if let Some(tail) = self.tail.replace(region) {
            (*header_ptr).prev = Some(tail);
            let tail_ptr = self.header_mut(tail);
            (*tail_ptr).next = Some(region);
        }

        self.regions = regions;
        self.bytes = bytes;

        Some((region, header_ptr))
    }

    unsafe fn free(&mut self, at: Region) {
        let header_ptr = self.header_mut(at);

        debug_assert_eq!((*header_ptr).state, State::Used);
        debug_assert_eq!((*header_ptr).next_free, None);

        // Just free up the last region in the slab.
        if (*header_ptr).next.is_none() {
            debug_assert_eq!(self.tail, Some(at));

            let mut at = at;
            let mut current_ptr = header_ptr;
            let mut total = 0;
            let mut prev = (*current_ptr).prev.take();

            loop {
                total += (*current_ptr).size();

                (*current_ptr).next_free = self.free.replace(at);
                (*current_ptr).state = State::Free;
                (*current_ptr).start = 0;
                (*current_ptr).size = 0;

                let Some(next) = prev else {
                    self.head = None;
                    break;
                };

                current_ptr = self.header_mut(next);
                at = next;

                (*current_ptr).next = None;

                if (*current_ptr).state != State::Occupy {
                    break;
                }

                prev = (*current_ptr).prev.take();
            }

            self.tail = prev;
            self.bytes -= total;
            return;
        }

        let Some(prev) = (*header_ptr).prev else {
            (*header_ptr).state = State::Occupy;
            return;
        };

        let prev_ptr = &mut *self.header_mut(prev);

        if prev_ptr.state != State::Occupy {
            (*header_ptr).state = State::Occupy;
            return;
        }

        // Move allocation to the previous region.
        let Header {
            size, next, prev, ..
        } = header_ptr.replace(Header {
            start: 0,
            size: 0,
            state: State::Free,
            next_free: self.free.replace(at),
            prev: None,
            next: None,
        });

        prev_ptr.size += size;
        prev_ptr.next = next;

        if let Some(next) = next {
            (*self.header_mut(next)).prev = prev;
        } else {
            // The current header being freed is the last in the list.
            self.bytes = (*header_ptr).start();
            self.tail = prev;
        }
    }

    unsafe fn realloc(&mut self, from: Region, len: usize, requested: usize) -> Option<Region> {
        let from_header = self.header_mut(from);

        if requested <= (*from_header).size() {
            return Some(from);
        }

        // This is the last region in the slab, so we can just expand it.
        if (*from_header).next.is_none() {
            let additional = requested - (*from_header).size();

            if self.bytes + additional > self.size {
                return None;
            }

            (*from_header).size += additional as u32;
            self.bytes += additional;
            return Some(from);
        }

        let (to, to_header) = self.alloc(requested)?;

        let from_data = self
            .data
            .wrapping_add((*from_header).start())
            .cast::<u8>()
            .cast_const();

        let to_data = self.data.wrapping_add((*to_header).start()).cast::<u8>();

        ptr::copy_nonoverlapping(from_data, to_data, len);
        self.free(from);
        Some(to)
    }

    unsafe fn find_region<T>(&mut self, mut condition: T) -> Option<(Region, *mut Header)>
    where
        T: FnMut(&Header) -> bool,
    {
        let mut current = self.head;

        while let Some(to) = current {
            let header_ptr = self.header_mut(to);

            if condition(&*header_ptr) {
                return Some((to, header_ptr));
            }

            current = (*header_ptr).next;
        }

        None
    }

    unsafe fn pop_free(&mut self) -> Option<(Region, *mut Header)> {
        let region = self.free.take()?;
        let header_ptr = self.header_mut(region);
        self.free = (*header_ptr).next_free.take();
        Some((region, header_ptr))
    }

    #[inline]
    fn region_to_addr(&self, at: Region) -> usize {
        self.size - (at.get() as usize) * size_of::<Header>()
    }

    #[inline]
    unsafe fn addr_to_region(&self, addr: usize) -> Region {
        debug_assert!(addr < self.size);
        Region::new_unchecked(((self.size - addr) / size_of::<Header>()) as u8)
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
    // Size of the region.
    size: u32,
    // The state of the region.
    state: State,
    // Link to the next free region.
    next_free: Option<Region>,
    // The previous neighbouring region.
    prev: Option<Region>,
    // The next neighbouring region.
    next: Option<Region>,
}

impl Header {
    /// Get the start address.
    #[inline]
    fn start(&self) -> usize {
        self.start as usize
    }

    /// Get the size.
    #[inline]
    fn size(&self) -> usize {
        self.size as usize
    }
}
