use core::cell::UnsafeCell;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::vec::Vec;

use musli::context::Buffer;

use crate::allocator::Allocator;

/// A dynamic buffer allocated on the heap.
pub struct HeapBuffer {
    internal: UnsafeCell<Internal>,
}

impl HeapBuffer {
    /// Construct a new heap buffer.
    pub fn new() -> Self {
        Self {
            internal: UnsafeCell::new(Internal { head: None }),
        }
    }
}

impl Default for HeapBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Buffer used in combination with an [`Allocator`].
pub struct Alloc<'a> {
    buf: &'a mut HeapBuffer,
}

impl<'a> Alloc<'a> {
    /// Construct a new allocator.
    pub fn new(buf: &'a mut HeapBuffer) -> Self {
        Self { buf }
    }
}

impl<'a> Allocator for Alloc<'a> {
    type Buf<'this> = Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        Some(Buf {
            region: Internal::alloc(&self.buf.internal),
            internal: &self.buf.internal,
        })
    }
}

impl<'a> Drop for Alloc<'a> {
    fn drop(&mut self) {
        let internal = unsafe { &mut *self.buf.internal.get() };

        while let Some(mut head) = internal.head.take() {
            // SAFETY: This collection has exclusive access to any heads it
            // contain.
            unsafe {
                internal.head = head.as_mut().next.take();
                drop(Box::from_raw(head.as_ptr()));
            }
        }
    }
}

/// A vector-backed allocation.
pub struct Buf<'a> {
    region: &'a mut Region,
    internal: &'a UnsafeCell<Internal>,
}

impl<'a> Buffer for Buf<'a> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> bool {
        self.region.data.extend_from_slice(bytes);
        true
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.region.data.len()
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        &self.region.data
    }
}

impl<'a> Drop for Buf<'a> {
    fn drop(&mut self) {
        Internal::free(self.internal, self.region);
    }
}

/// An allocated region.
#[repr(C)]
struct Region {
    data: Vec<u8>,
    // Pointer to the next free region.
    next: Option<NonNull<Region>>,
}

/// Internals of the allocator.
struct Internal {
    // Regions of re-usable allocations we can hand out.
    head: Option<NonNull<Region>>,
}

impl Internal {
    /// Allocate a new region.
    ///
    /// Note that this will return a leaked memory region, so the unbound
    /// lifetime is intentional.
    fn alloc<'a>(this: &UnsafeCell<Self>) -> &'a mut Region {
        // SAFETY: We take care to only access internals in a single-threaded
        // mutable fashion.
        let internal = unsafe { &mut *this.get() };

        if let Some(mut head) = internal.head.take() {
            // SAFETY: This collection has exclusive access to any heads it contain.
            unsafe {
                let head = head.as_mut();
                internal.head = head.next.take();
                head
            }
        } else {
            Box::leak(Box::new(Region {
                data: Vec::new(),
                next: None,
            }))
        }
    }

    fn free(this: &UnsafeCell<Self>, region: &mut Region) {
        unsafe {
            let this = &mut *this.get();
            region.data.clear();
            region.next = this.head;
            this.head = Some(NonNull::from(region));
        }
    }
}
