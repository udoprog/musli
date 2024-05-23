use core::fmt::{self, Arguments};
use core::mem::align_of;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::buf::Error;
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
    type Buf<'this> = SystemBuf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc_aligned<T>(&self) -> Option<Self::Buf<'_>> {
        assert_eq!(
            align_of::<T>(),
            1,
            "The system allocator currently only supported an alignment of 1"
        );

        let region = self.root.alloc();

        Some(SystemBuf {
            root: &self.root,
            region,
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
pub struct SystemBuf<'a> {
    root: &'a Root,
    region: &'a mut Region,
}

impl<'a> Buf for SystemBuf<'a> {
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

    #[inline(always)]
    fn write_fmt(&mut self, arguments: Arguments<'_>) -> Result<(), Error> {
        fmt::write(self, arguments).map_err(|_| Error)
    }
}

impl fmt::Write for SystemBuf<'_> {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.region.data.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl<'a> Drop for SystemBuf<'a> {
    fn drop(&mut self) {
        self.root.free(self.region);
    }
}

/// An allocated region.
struct Region {
    data: Vec<u8>,
    // Pointer to the next free region.
    next: Option<NonNull<Region>>,
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
    #[allow(clippy::mut_from_ref)]
    fn alloc(&self) -> &mut Region {
        self.regions.fetch_add(1, Ordering::SeqCst);

        // SAFETY: We have exclusive access to all regions.
        let this = self.lock().with_mut(|current| unsafe {
            let mut this = current.take()?;
            let this = this.as_mut();
            *current = this.next.take();
            Some(this)
        });

        if let Some(this) = this {
            return this;
        }

        Box::leak(Box::new(Region {
            data: Vec::with_capacity(INITIAL_CAPACITY),
            next: None,
        }))
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

        region.data.clear();
        region.data.shrink_to(MAX_CAPACITY);

        let mut current = self.lock();

        current.with_mut(|current| {
            region.next = current.take();
            *current = Some(NonNull::from(region));
        });
    }
}
