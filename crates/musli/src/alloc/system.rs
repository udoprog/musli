use core::alloc::Layout;
use core::cmp;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr;
use core::ptr::NonNull;

use rust_alloc::alloc;
use rust_alloc::boxed::Box;

use crate::loom::cell::UnsafeCell;
use crate::loom::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::loom::sync::with_mut_usize;

use super::{Allocator, Buf};

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
/// use musli::{Allocator, Buf};
/// use musli::alloc::System;
/// use musli::alloc::Vec;
///
/// let alloc = System::new();
///
/// let mut buf1 = Vec::new_in(&alloc);
/// let mut buf2 = Vec::new_in(&alloc);
//
/// assert!(buf1.write(b"Hello, "));
/// assert!(buf2.write(b"world!"));
///
/// assert_eq!(buf1.as_slice(), b"Hello, ");
/// assert_eq!(buf2.as_slice(), b"world!");
///
/// buf1.extend(buf2);
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
    type Buf<'this, T> = SystemBuf<'this, T> where Self: 'this, T: 'this;

    #[inline]
    fn alloc<'a, T>(&'a self) -> Self::Buf<'a, T>
    where
        T: 'a,
    {
        let region = if size_of::<T>() == 0 {
            None
        } else {
            self.root.alloc::<T>()
        };

        SystemBuf {
            root: &self.root,
            region,
            _marker: PhantomData,
        }
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
    region: Option<&'a mut Region>,
    _marker: PhantomData<T>,
}

impl<'a, T> Buf<T> for SystemBuf<'a, T> {
    #[inline]
    fn resize(&mut self, len: usize, additional: usize) -> bool {
        if additional == 0 || size_of::<T>() == 0 {
            return true;
        }

        let Some(region) = &mut self.region else {
            return false;
        };

        // SAFETY: The region is always valid.
        unsafe { region.reserve::<T>(len, additional) }
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        let Some(region) = &self.region else {
            return ptr::NonNull::dangling().as_ptr();
        };

        region.data.as_ptr().cast_const().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        let Some(region) = &mut self.region else {
            return ptr::NonNull::dangling().as_ptr();
        };

        region.data.as_ptr().cast()
    }

    #[inline]
    fn try_merge<B>(&mut self, _: usize, other: B, _: usize) -> Result<(), B>
    where
        B: Buf<T>,
    {
        Err(other)
    }
}

impl<'a, T> Drop for SystemBuf<'a, T> {
    fn drop(&mut self) {
        if let Some(region) = self.region.take() {
            self.root.free(region);
        }
    }
}

/// An allocated region.
struct Region {
    /// Data pointer to the allocated region.
    data: NonNull<()>,
    /// The size of the allocated region.
    layout: Layout,
    /// Pointer to the next free region.
    next: Option<NonNull<Region>>,
}

impl Region {
    const fn dangling<T>() -> Self {
        Self {
            data: NonNull::<T>::dangling().cast(),
            // SAFETY: A zero-sized layout is always valid, and the alignment stems from T.
            layout: unsafe { Layout::from_size_align_unchecked(0, align_of::<T>()) },
            next: None,
        }
    }

    // Ensure the region is aligned.
    fn is_aligned<T>(&mut self) -> bool {
        self.layout.align() % align_of::<T>() == 0
    }

    /// Allocate with the specified layout.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the alignment and capacity are valid per
    /// [`Layout`] constraints.
    unsafe fn alloc(layout: Layout) -> Option<Self> {
        let data = alloc::alloc(layout);

        if data.is_null() {
            return None;
        }

        Some(Region {
            data: NonNull::new_unchecked(data).cast(),
            layout,
            next: None,
        })
    }

    /// Reallocate the region to the given capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the new capacity is valid per [`Layout`].
    #[must_use = "allocating is fallible and must be checked"]
    unsafe fn realloc(&mut self, new_layout: Layout) -> bool {
        debug_assert!(self.layout.align() % new_layout.align() == 0);

        let (data, new_layout) = if self.layout.size() > 0 {
            // The new layout inherits the alignment of the existing one, but with a new size.
            let layout = Layout::from_size_align_unchecked(new_layout.size(), self.layout.align());

            (
                alloc::realloc(self.data.as_ptr().cast(), self.layout, new_layout.size()),
                layout,
            )
        } else {
            (alloc::alloc(new_layout), new_layout)
        };

        if data.is_null() {
            return false;
        }

        self.data = NonNull::new_unchecked(data).cast();
        self.layout = new_layout;
        true
    }

    fn shrink_to(&mut self, max_bytes: usize) -> bool {
        if self.layout.size() <= max_bytes {
            return true;
        }

        let Ok(new_layout) = Layout::from_size_align(max_bytes, self.layout.align()) else {
            return false;
        };

        // SAFETY: We're taking care to ensure that elements are layout compatible.
        unsafe { self.realloc(new_layout) }
    }

    #[must_use = "allocating is fallible and must be checked"]
    unsafe fn reserve<T>(&mut self, len: usize, additional: usize) -> bool {
        if size_of::<T>() == 0 {
            // This being a ZST by definition means it is compatible with
            // anything that this region happens to hold as an allocation.
            // All though we should never reach this point.
            debug_assert_eq!(align_of::<T>(), 1);
            return true;
        }

        let Some(required_cap) = len.checked_add(additional) else {
            return false;
        };

        let cap = if self.layout.align() % align_of::<T>() == 0
            && self.layout.size() % size_of::<T>() == 0
        {
            let cap = self.layout.size() / size_of::<T>();

            if cap >= required_cap {
                return true;
            }

            cap
        } else {
            0
        };

        let cap = cmp::max(cap * 2, required_cap);
        let cap = cmp::max(min_non_zero_cap::<T>(), cap);

        let Ok(new_layout) = Layout::array::<T>(cap) else {
            return false;
        };

        if self.layout.align() % new_layout.align() == 0 {
            if self.layout.size() >= new_layout.size() {
                return true;
            }

            self.realloc(new_layout)
        } else {
            let Some(new) = Self::alloc(new_layout) else {
                return false;
            };

            *self = new;
            true
        }
    }

    fn free(&mut self) {
        if self.layout.size() > 0 {
            // SAFETY: Layout assumptions are correctly encoded in the type as
            // it was being allocated or grown.
            unsafe {
                alloc::dealloc(self.data.as_ptr().cast(), self.layout);
                self.data = NonNull::<u8>::dangling().cast();
                self.layout = Layout::from_size_align_unchecked(0, 1);
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
    fn alloc<T>(&self) -> Option<&mut Region> {
        // SAFETY: We have exclusive access to all regions.
        let this = self.lock().with_mut(|current| unsafe {
            let mut this = current.take()?;
            let this = this.as_mut();
            *current = this.next.take();
            Some(this)
        });

        let region = 'out: {
            if let Some(this) = this {
                if this.is_aligned::<T>() {
                    break 'out this;
                }

                this.free();
                this.data = NonNull::<T>::dangling().cast();
                break 'out this;
            };

            Box::leak(Box::new(Region::dangling::<T>()))
        };

        self.regions.fetch_add(1, Ordering::SeqCst);
        Some(region)
    }

    fn free(&self, region: &mut Region) {
        let regions = self.regions.fetch_sub(1, Ordering::SeqCst);

        if regions >= MAX_REGIONS {
            // SAFETY: We have exclusive access to the region, which we
            // previously leaked.
            unsafe {
                drop(Box::from_raw(region));
            }

            return;
        }

        if !region.shrink_to(MAX_CAPACITY) {
            // If we fail to shrink the region, the only option we have left is
            // to free it. Shrinking should only fail if there is insufficient
            // memory to allocate a new smaller region at the same time as
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

fn min_non_zero_cap<T>() -> usize {
    if size_of::<T>() == 1 {
        8
    } else if size_of::<T>() <= 1024 {
        4
    } else {
        1
    }
}
