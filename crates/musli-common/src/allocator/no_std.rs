use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;
use core::mem::{size_of, MaybeUninit};
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

/// Minimum length of a region as its being written to.
const MIN_LEN: usize = 8;

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
        let (region, _) = unsafe { (*self.internal.get()).alloc(MIN_LEN)? };

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
                let to_len = align_requested(len + bytes.len());

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
    // The first region to allocate if it's been freed.
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
        if let Some((region, header_ptr, prev)) = self.find_free(|h| h.size() >= requested) {
            let free = (*header_ptr).free.take();

            if let Some(prev) = prev {
                (*self.header_mut(prev)).free = free;
            } else {
                self.free = free;
            }

            (*header_ptr).state = State::Used;

            // TODO: Should we split the allocated region?
            return Some((region, header_ptr));
        }

        let requested = align_requested(requested);
        let start = u32::try_from(self.bytes).ok()?;
        let size = u32::try_from(requested).ok()?;

        let bytes = self.bytes.checked_add(requested)?;
        let regions = self.regions.checked_add(size_of::<Header>())?;
        let total = bytes.checked_add(regions)?;

        if total > self.size {
            return None;
        }

        let addr = self.size - regions;
        let region = self.addr_to_region(addr);
        let header_ptr = self.data.wrapping_add(addr).cast::<Header>();
        self.regions = regions;
        self.bytes = bytes;

        header_ptr.write(Header {
            start,
            size,
            state: State::Used,
            free: None,
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

        Some((region, header_ptr))
    }

    unsafe fn free(&mut self, at: Region) {
        let header_ptr = self.header_mut(at);

        (*header_ptr).state = State::Free;
        (*header_ptr).free = self.free.replace(at);

        if (*header_ptr).next.is_none() {
            self.bytes += (*header_ptr).size();
            self.tail = (*header_ptr).prev.take();
            (*header_ptr).start = 0;
            (*header_ptr).size = 0;
            (*header_ptr).next = None;
            return;
        }

        let Some(prev) = (*header_ptr).prev else {
            return;
        };

        let prev_ptr = &mut *self.header_mut(prev);

        if prev_ptr.state != State::Free {
            return;
        }

        // Move allocation to the previous region.
        let Header {
            size, next, prev, ..
        } = header_ptr.replace(Header {
            start: 0,
            size: 0,
            state: State::Free,
            free: (*header_ptr).free,
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
            let requested = align_requested(requested);
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

    unsafe fn find_free<T>(
        &mut self,
        mut condition: T,
    ) -> Option<(Region, *mut Header, Option<Region>)>
    where
        T: FnMut(&Header) -> bool,
    {
        // First iterate over existing regions to try and find a different one
        // which is suitable.
        let mut current = self.free;
        let mut prev = None;

        while let Some(to) = current {
            let header_ptr = self.header_mut(to);

            if condition(&*header_ptr) {
                return Some((to, header_ptr, prev));
            }

            prev = Some(to);
            current = (*header_ptr).free;
        }

        None
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
    Free = 0,
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
    // The next freed region to be allocated.
    free: Option<Region>,
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

fn align_requested(requested: usize) -> usize {
    if requested < MIN_LEN {
        return MIN_LEN;
    }

    requested.next_multiple_of(MIN_LEN)
}

#[cfg(test)]
mod tests {
    use crate::allocator::{Allocator, Buf};

    use super::{Header, NoStd, Region, State};

    macro_rules! assert_regions {
        (
            $list:expr,
            $($region:expr => {
                $start:expr,
                $size:expr,
                $state:expr,
                free: $free:expr,
                prev: $prev:expr,
                next: $next:expr $(,)?
            },)* $(,)?
        ) => {{
            let i = unsafe { &*$list.internal.get() };

            $(
                assert_eq! {
                    *i.header($region),
                    Header {
                        start: $start,
                        size: $size,
                        state: $state,
                        free: $free,
                        prev: $prev,
                        next: $next,
                    },
                    "Comparing region {}", stringify!($region)
                };
            )*
        }};
    }

    macro_rules! assert_free {
        (
            $list:expr $(, $head:expr $(, $rest:expr)*)? $(,)?
        ) => {{
            $(
                let i = unsafe { &*$list.internal.get() };
                assert_eq!(i.free, Some($head), "Expected head region");
                let free = i.header($head).free;

                $(
                    let Some(free) = free else {
                        panic!("Expected another region before {}", stringify!($rest));
                    };

                    let free = i.header(free).free;
                )*

                assert_eq!(free, None);
            )*
        }};
    }

    const A: Region = unsafe { Region::new_unchecked(1) };
    const B: Region = unsafe { Region::new_unchecked(2) };
    const C: Region = unsafe { Region::new_unchecked(3) };
    const D: Region = unsafe { Region::new_unchecked(4) };

    fn grow_last(alloc: &NoStd<'_>) {
        let a = alloc.alloc().unwrap();

        let mut b = alloc.alloc().unwrap();
        b.write(&[1, 2, 3, 4, 5, 6]);
        b.write(&[7, 8]);

        {
            let i = unsafe { &*alloc.internal.get() };

            assert_eq!(i.free, None);
            assert_eq!(i.head, Some(A));
            assert_eq!(i.tail, Some(B));

            assert_regions! {
                alloc,
                A => { 0, 8, State::Used, free: None, prev: None, next: Some(B) },
                B => { 8, 8, State::Used, free: None, prev: Some(A), next: None },
            };

            assert_free!(alloc);
        }

        b.write(&[9, 10]);

        {
            let i = unsafe { &*alloc.internal.get() };

            assert_eq!(i.free, None);
            assert_eq!(i.head, Some(A));
            assert_eq!(i.tail, Some(B));

            assert_regions! {
                alloc,
                A => { 0, 8, State::Used, free: None, prev: None, next: Some(B) },
                B => { 8, 16, State::Used, free: None, prev: Some(A), next: None },
            };

            assert_free!(alloc);
        }

        drop(a);
        drop(b);

        {
            let i = unsafe { &*alloc.internal.get() };

            assert_eq!(i.head, Some(A));
            assert_eq!(i.tail, Some(A));

            // TODO: Since all regions have been freed, the allocator should be
            // uninitialized again.
            assert_regions! {
                alloc,
                A => { 0, 8, State::Free, free: None, prev: None, next: Some(B) },
                B => { 0, 0, State::Free, free: Some(A), prev: None, next: None },
            };

            assert_free!(alloc, B, A);
        }
    }

    #[test]
    fn nostd_grow_last() {
        let mut buf = crate::allocator::StackBuffer::<4096>::new();
        let alloc = crate::allocator::NoStd::new(&mut buf);
        grow_last(&alloc);
    }

    fn realloc(alloc: &NoStd<'_>) {
        let mut a = alloc.alloc().unwrap();
        a.write(&[1, 2, 3, 4]);
        assert_eq!(a.region.get(), A);

        let mut b = alloc.alloc().unwrap();
        b.write(&[1, 2, 3, 4]);
        assert_eq!(b.region.get(), B);

        let mut c = alloc.alloc().unwrap();
        c.write(&[1, 2, 3, 4]);
        assert_eq!(c.region.get(), C);

        assert_eq!(a.region.get(), A);
        assert_eq!(b.region.get(), B);
        assert_eq!(c.region.get(), C);

        {
            let i = unsafe { &*alloc.internal.get() };

            assert_eq!(i.free, None);
            assert_eq!(i.head, Some(A));
            assert_eq!(i.tail, Some(C));

            assert_regions! {
                alloc,
                A => { 0, 8, State::Used, free: None, prev: None, next: Some(B) },
                B => { 8, 8, State::Used, free: None, prev: Some(A), next: Some(C) },
                C => { 16, 8, State::Used, free: None, prev: Some(B), next: None },
            };
        }

        drop(a);

        {
            let i = unsafe { &*alloc.internal.get() };
            assert_eq!(i.free, Some(A));
            assert_eq!(i.head, Some(A));
            assert_eq!(i.tail, Some(C));

            assert_regions! {
                alloc,
                A => { 0, 8, State::Free, free: None, prev: None, next: Some(B) },
                B => { 8, 8, State::Used, free: None, prev: Some(A), next: Some(C) },
                C => { 16, 8, State::Used, free: None, prev: Some(B), next: None },
            };
        }

        drop(b);

        {
            let i = unsafe { &*alloc.internal.get() };
            assert_eq!(i.free, Some(B));
            assert_eq!(i.head, Some(A));
            assert_eq!(i.tail, Some(C));
        }

        assert_regions! {
            alloc,
            A => { 0, 16, State::Free, free: None, prev: None, next: Some(C) },
            B => { 0, 0, State::Free, free: Some(A), prev: None, next: None },
            C => { 16, 8, State::Used, free: None, prev: Some(A), next: None },
        };

        let mut d = alloc.alloc().unwrap();
        d.write(&[1, 2]);

        assert_eq!(d.region.get(), A);

        assert_regions! {
            alloc,
            A => { 0, 16, State::Used, free: None, prev: None, next: Some(C) },
            B => { 0, 0, State::Free, free: None, prev: None, next: None },
            C => { 16, 8, State::Used, free: None, prev: Some(A), next: None },
        };

        assert_free!(alloc, B);

        d.write(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        assert_eq!(d.region.get(), D);

        assert_regions! {
            alloc,
            A => { 0, 16, State::Free, free: Some(B), prev: None, next: Some(C) },
            B => { 0, 0, State::Free, free: None, prev: None, next: None },
            C => { 16, 8, State::Used, free: None, prev: Some(A), next: Some(D) },
            D => { 24, 24, State::Used, free: None, prev: Some(C), next: None },
        };

        assert_free!(alloc, A, B);
    }

    #[test]
    fn nostd_realloc() {
        let mut buf = crate::allocator::StackBuffer::<4096>::new();
        let alloc = crate::allocator::NoStd::new(&mut buf);
        realloc(&alloc);
    }
}
