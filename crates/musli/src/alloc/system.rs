use core::alloc::Layout;
use core::cmp;
use core::mem::{align_of, size_of};
use core::ptr::NonNull;

use rust_alloc::alloc;

use super::{Alloc, AllocError, AllocSlice, Allocator};

/// System buffer that can be used in combination with an [`Allocator`].
///
/// This uses the [`System`] allocator.
///
/// [`System` allocator]: https://doc.rust-lang.org/std/alloc/struct.System.html
///
/// # Examples
///
/// ```
/// use musli::alloc::{System, Vec};
///
/// let alloc = System::new();
///
/// let mut buf1 = Vec::new_in(alloc);
/// let mut buf2 = Vec::new_in(alloc);
//
/// buf1.extend_from_slice(b"Hello, ")?;
/// buf2.extend_from_slice(b"world!")?;
///
/// assert_eq!(buf1.as_slice(), b"Hello, ");
/// assert_eq!(buf2.as_slice(), b"world!");
///
/// buf1.extend(buf2);
/// assert_eq!(buf1.as_slice(), b"Hello, world!");
/// # Ok::<_, musli::alloc::AllocError>(())
/// ```
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct System;

impl System {
    /// Construct a new system allocator.
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Construct an allocation directly from raw parts.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the allocation comes from the same system
    /// allocator and is correctly initialized per its parameters.
    #[inline]
    pub(crate) unsafe fn slice_from_raw_parts<T>(data: NonNull<T>, size: usize) -> SystemBuf<T> {
        SystemBuf { data, size }
    }
}

impl Default for System {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator for System {
    type Alloc<T> = SystemAlloc<T>;
    type AllocSlice<T> = SystemBuf<T>;

    #[inline]
    fn alloc<T>(self, value: T) -> Result<Self::Alloc<T>, AllocError> {
        let mut raw = SystemAlloc::<T>::alloc()?;

        if size_of::<T>() != 0 {
            // SAFETY: The above ensures the data has been allocated.
            unsafe {
                raw.as_mut_ptr().write(value);
            }
        }

        Ok(raw)
    }

    #[inline]
    fn alloc_slice<T>(self) -> Self::AllocSlice<T> {
        SystemBuf::DANGLING
    }
}

/// A vector-backed allocation.
pub struct SystemAlloc<T> {
    /// Pointer to the allocated region.
    data: NonNull<T>,
}

unsafe impl<T> Send for SystemAlloc<T> where T: Send {}
unsafe impl<T> Sync for SystemAlloc<T> where T: Sync {}

impl<T> SystemAlloc<T> {
    /// Reallocate the region to the given capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the new capacity is valid per [`Layout`].
    #[must_use = "allocating is fallible and must be checked"]
    fn alloc() -> Result<Self, AllocError> {
        if size_of::<T>() == 0 {
            return Ok(Self {
                data: NonNull::dangling(),
            });
        }

        unsafe {
            let data = alloc::alloc(Layout::new::<T>());

            if data.is_null() {
                return Err(AllocError);
            }

            Ok(Self {
                data: NonNull::new_unchecked(data).cast(),
            })
        }
    }
}

impl<T> Alloc<T> for SystemAlloc<T> {
    #[inline]
    fn as_ptr(&self) -> *const T {
        self.data.as_ptr().cast_const().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_ptr().cast()
    }
}

impl<T> Drop for SystemAlloc<T> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: Layout assumptions are correctly encoded in the type as it
        // was being allocated or grown.
        unsafe {
            if size_of::<T>() != 0 {
                alloc::dealloc(self.data.as_ptr().cast(), Layout::new::<T>());
            }
        }
    }
}

/// A vector-backed allocation.
pub struct SystemBuf<T> {
    /// Pointer to the allocated region.
    data: NonNull<T>,
    /// The size in number of `T` elements in the region.
    size: usize,
}

unsafe impl<T> Send for SystemBuf<T> where T: Send {}
unsafe impl<T> Sync for SystemBuf<T> where T: Sync {}

impl<T> AllocSlice<T> for SystemBuf<T> {
    #[inline]
    fn as_ptr(&self) -> *const T {
        self.data.as_ptr().cast_const().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_ptr().cast()
    }

    #[inline]
    fn resize(&mut self, len: usize, additional: usize) -> Result<(), AllocError> {
        if additional == 0 || size_of::<T>() == 0 {
            return Ok(());
        }

        if !self.reserve(len, additional) {
            return Err(AllocError);
        }

        Ok(())
    }

    #[inline]
    fn try_merge<B>(&mut self, _: usize, other: B, _: usize) -> Result<(), B>
    where
        B: AllocSlice<T>,
    {
        Err(other)
    }
}

impl<T> SystemBuf<T> {
    const MIN_NON_ZERO_CAP: usize = if size_of::<T>() == 1 {
        8
    } else if size_of::<T>() <= 1024 {
        4
    } else {
        1
    };

    const DANGLING: Self = Self {
        data: NonNull::dangling(),
        size: 0,
    };

    /// Reallocate the region to the given capacity.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the new capacity is valid per [`Layout`].
    #[must_use = "allocating is fallible and must be checked"]
    fn realloc(&mut self, new_layout: Layout) -> bool {
        unsafe {
            let data = {
                if self.size > 0 {
                    let old_layout = Layout::from_size_align_unchecked(
                        self.size.wrapping_mul(size_of::<T>()),
                        align_of::<T>(),
                    );

                    alloc::realloc(self.data.as_ptr().cast(), old_layout, new_layout.size())
                } else {
                    alloc::alloc(new_layout)
                }
            };

            if data.is_null() {
                return false;
            }

            self.data = NonNull::new_unchecked(data).cast();
        }

        true
    }

    #[must_use = "allocating is fallible and must be checked"]
    fn reserve(&mut self, len: usize, additional: usize) -> bool {
        debug_assert_ne!(size_of::<T>(), 0, "ZSTs should not get here");

        let Some(required_cap) = len.checked_add(additional) else {
            return false;
        };

        if self.size >= required_cap {
            return true;
        }

        let cap = cmp::max(self.size * 2, required_cap);
        let cap = cmp::max(Self::MIN_NON_ZERO_CAP, cap);

        let Ok(new_layout) = Layout::array::<T>(cap) else {
            return false;
        };

        if !self.realloc(new_layout) {
            return false;
        }

        self.size = cap;
        true
    }

    fn free(&mut self) {
        if self.size > 0 {
            // SAFETY: Layout assumptions are correctly encoded in the type as
            // it was being allocated or grown.
            unsafe {
                let layout =
                    Layout::from_size_align_unchecked(self.size * size_of::<T>(), align_of::<T>());
                alloc::dealloc(self.data.as_ptr().cast(), layout);
                self.data = NonNull::dangling();
                self.size = 0;
            }
        }
    }
}

impl<T> Drop for SystemBuf<T> {
    #[inline]
    fn drop(&mut self) {
        self.free();
    }
}
