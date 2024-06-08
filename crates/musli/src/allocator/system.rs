use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::align_of;
use core::ptr;
use core::ptr::NonNull;

use ::alloc::alloc;
use ::alloc::boxed::Box;

use crate::loom::cell::UnsafeCell;
use crate::loom::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::loom::sync::with_mut_usize;
use crate::{Allocator, Buf};

/// The initial capacity of an allocated region.
#[cfg(not(loom))]
const INITIAL_CAPACITY: usize = 128;
#[cfg(loom)]
const INITIAL_CAPACITY: usize = 4;
/// The max capacity of an allocated region as it's being handed back.
#[cfg(not(loom))]
const MAX_CAPACITY: usize = 4096;
#[cfg(loom)]
const MAX_CAPACITY: usize = 12;
/// The maximum number of regions we hold onto to avoid leaking too much memory.
#[cfg(not(loom))]
const MAX_REGIONS: usize = 64;
#[cfg(loom)]
const MAX_REGIONS: usize = 2;

/// System buffer that can be used in combination with an [`Allocator`].
///
/// This uses the [`System`] allocator.
///
/// [`System` allocator]: https://doc.rust-lang.org/std/alloc/struct.System.html
///
/// # Examples
///
/// ```
/// use musli::allocator::System;
/// use musli::{Allocator, Buf};
///
/// let allocator = System::new();
///
/// let mut buf1 = allocator.alloc().expect("allocation failed");
/// let mut buf2 = allocator.alloc().expect("allocation failed");
///
/// assert!(buf1.write(b"Hello, "));
/// assert!(buf2.write(b"world!"));
///
/// assert_eq!(buf1.as_slice(), b"Hello, ");
/// assert_eq!(buf2.as_slice(), b"world!");
///
/// buf1.write_buffer(buf2);
/// assert_eq!(buf1.as_slice(), b"Hello, world!");
/// ```
pub struct System {
    root: Root,
}

impl System {
    /// Construct a new allocator.
    #[inline]
    #[cfg(not(loom))]
    pub const fn new() -> Self {
        Self {
            root: Root {
                locked: AtomicBool::new(false),
                head: UnsafeCell::new(None),
                regions: AtomicUsize::new(0),
            },
        }
    }

    /// Construct a new allocator.
    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            root: Root {
                locked: AtomicBool::new(false),
                head: UnsafeCell::new(None),
                regions: AtomicUsize::new(0),
            },
        }
    }
}

impl Root {
    fn spin(&self) {
        while self.locked.load(Ordering::Relaxed) {
            crate::loom::spin_loop();
        }
    }

    fn lock(&self) -> Guard<'_> {
        loop {
            self.spin();

            match self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(_) => continue,
            }
        }

        Guard { root: self }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::SeqCst);
    }
}

struct Guard<'a> {
    root: &'a Root,
}

impl Guard<'_> {
    #[inline]
    fn with_mut<'this, O>(
        &'this mut self,
        f: impl FnOnce(&'this mut Option<NonNull<Region>>) -> O,
    ) -> O {
        // SAFETY: We have access to the inner root under a lock guard.
        self.root.head.with_mut(|inner| f(unsafe { &mut *inner }))
    }
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.root.unlock();
    }
}

impl Default for System {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator for System {
    type Buf<'this, T> = SystemBuf<'this, T> where Self: 'this, T: 'static;

    #[inline(always)]
    fn alloc_aligned<T>(&self) -> Option<Self::Buf<'_, T>>
    where
        T: 'static,
    {
        // SAFETY: The alignment of `T` is always valid.
        let region = unsafe { self.root.alloc(align_of::<T>())? };

        Some(SystemBuf {
            root: &self.root,
            region,
            _marker: PhantomData,
        })
    }
}

impl Drop for System {
    fn drop(&mut self) {
        let mut current = self.root.lock().with_mut(|current| current.take());

        with_mut_usize(&mut self.root.regions, |v| *v = 0);

        while let Some(this) = current.take() {
            // SAFETY: While the system allocator is being dropped we hold a
            // mutable reference to it, which ensures exclusive access to all
            // allocated regions.
            let b = unsafe { Box::from_raw(this.as_ptr()) };
            current = b.next;
        }
    }
}

/// A vector-backed allocation.
pub struct SystemBuf<'a, T> {
    root: &'a Root,
    region: &'a mut Region,
    _marker: PhantomData<T>,
}

impl<'a, T> Buf for SystemBuf<'a, T>
where
    T: 'static,
{
    type Item = T;

    #[inline]
    fn resize(&mut self, old: usize, new: usize) -> bool {
        if new < old {
            return true;
        }

        unsafe { self.region.reserve(new - old) }
    }

    #[inline]
    fn as_ptr(&self) -> *const Self::Item {
        self.region.data.as_ptr().cast_const().cast()
    }

    #[inline]
    fn as_ptr_mut(&mut self) -> *mut Self::Item {
        self.region.data.as_ptr().cast()
    }

    #[inline]
    fn try_merge<B>(&mut self, _: usize, other: B, _: usize) -> Result<(), B>
    where
        B: Buf<Item = Self::Item>,
    {
        Err(other)
    }
}

impl<'a, T> Drop for SystemBuf<'a, T> {
    fn drop(&mut self) {
        self.root.free(self.region);
    }
}

/// An allocated region.
struct Region {
    /// Data pointer to the allocated region.
    data: NonNull<u8>,
    /// Initialized length of the region.
    len: usize,
    /// The capacity of the region.
    cap: usize,
    /// The alignment of the allocated region.
    align: usize,
    /// Pointer to the next free region.
    next: Option<NonNull<Region>>,
}

impl Region {
    const DANGLING: Region = Region {
        data: NonNull::dangling(),
        len: 0,
        cap: 0,
        align: 0,
        next: None,
    };

    /// Allocate with the specifiec alignment and default capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the alignment and capacity are valid per
    /// [`Layout`] constraints.
    unsafe fn alloc(align: usize) -> Option<Self> {
        Self::alloc_capacity(align, INITIAL_CAPACITY)
    }

    /// Allocate with the specifiec alignment and capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the alignment and capacity are valid per
    /// [`Layout`] constraints.
    unsafe fn alloc_capacity(align: usize, cap: usize) -> Option<Self> {
        let layout = Layout::from_size_align_unchecked(cap, align);
        let data = alloc::alloc(layout);

        if data.is_null() {
            return None;
        }

        Some(Region {
            data: NonNull::new_unchecked(data),
            len: 0,
            cap,
            align,
            next: None,
        })
    }

    /// Align the region to the given alignment.
    #[must_use = "allocation is fallible and must be checked"]
    unsafe fn initialize(&mut self, align: usize) -> bool {
        // Region is not allocated.
        if self.cap == 0 {
            if let Some(new) = Self::alloc(align) {
                *self = new;
                return true;
            }

            return false;
        }

        // Region is already aligned.
        if self.align % align == 0 {
            return true;
        }

        if let Some(new) = Self::alloc_capacity(align, self.cap) {
            *self = new;
            return true;
        }

        false
    }

    fn shrink_to(&mut self, cap: usize) -> bool {
        if self.cap <= cap {
            return true;
        }

        // SAFETY: A smaller capacity is always valid.
        unsafe { self.realloc_to(cap) }
    }

    /// Reallocate the region to the given capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the new capacity is valid per [`Layout`].
    #[must_use = "allocating is fallible and must be checked"]
    unsafe fn realloc_to(&mut self, cap: usize) -> bool {
        let old_layout = Layout::from_size_align_unchecked(self.cap, self.align);
        let data = alloc::realloc(self.data.as_ptr(), old_layout, cap);

        if data.is_null() {
            return false;
        }

        self.data = NonNull::new_unchecked(data);
        self.cap = cap;
        true
    }

    #[must_use = "allocating is fallible and must be checked"]
    unsafe fn reserve(&mut self, additional: usize) -> bool {
        let min_cap = self.len + additional;

        if self.cap >= min_cap {
            return true;
        }

        let new_cap = min_cap.next_power_of_two();

        // Ensure we don't overflow isize which guarantees that the computed
        // layout is valid.
        if new_cap > max_size_for_align(self.align) {
            return false;
        }

        if new_cap < min_cap {
            return false;
        }

        if self.cap == 0 {
            if let Some(new) = Self::alloc_capacity(self.align, new_cap) {
                *self = new;
                return true;
            }

            return false;
        }

        if new_cap > self.cap {
            return self.realloc_to(new_cap);
        }

        true
    }

    fn free(&mut self) {
        if self.cap != 0 {
            // SAFETY: Layout assumptions are correctly encoded in the type as it was being allocated or grown.
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.cap, self.align);
                alloc::dealloc(self.data.as_ptr(), layout);
                ptr::write(self, Region::DANGLING);
            }
        }
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        self.free();
    }
}

/// Internals of the allocator.
struct Root {
    /// Spin lock used for root.
    locked: AtomicBool,
    /// Regions of re-usable allocations we can hand out.
    head: UnsafeCell<Option<NonNull<Region>>>,
    /// The number of allocated regions.
    regions: AtomicUsize,
}

unsafe impl Send for Root {}
unsafe impl Sync for Root {}

impl Root {
    /// Allocate a new region.
    ///
    /// Note that this will return a leaked memory region, so the unbound
    /// lifetime is intentional.
    ///
    /// Clippy note: We know that we are correctly returning mutable references
    /// to different mutable regions.
    ///
    /// # Safety
    ///
    /// The specified alignment must be a power of 2.
    #[allow(clippy::mut_from_ref)]
    unsafe fn alloc(&self, align: usize) -> Option<&mut Region> {
        // SAFETY: We have exclusive access to all regions.
        let this = self.lock().with_mut(|current| unsafe {
            let mut this = current.take()?;
            let this = this.as_mut();
            *current = this.next.take();
            Some(this)
        });

        let region = if let Some(this) = this {
            if !this.initialize(align) {
                return None;
            }

            this
        } else {
            Box::leak(Box::new(Region::alloc(align)?))
        };

        self.regions.fetch_add(1, Ordering::SeqCst);
        Some(region)
    }

    fn free<'a>(&'a self, region: &'a mut Region) {
        let regions = self.regions.fetch_sub(1, Ordering::SeqCst);

        if regions >= MAX_REGIONS {
            // SAFETY: We have exclusive access to the region, which we
            // previously leaked.
            unsafe {
                drop(Box::from_raw(region));
            }

            return;
        }

        if region.shrink_to(MAX_CAPACITY) {
            region.len = 0;
        } else {
            // If we fail to shrink the region, the only option we have left is
            // to free it. Shrinking should only fail if there is insufficient
            // memory to allocate a new smaller region at the samt time as
            // maintaining the old one.
            region.free();
        }

        let mut current = self.lock();

        current.with_mut(|current| {
            region.next = current.take();
            *current = Some(NonNull::from(region));
        });
    }
}

#[inline(always)]
const fn max_size_for_align(align: usize) -> usize {
    isize::MAX as usize - (align - 1)
}
