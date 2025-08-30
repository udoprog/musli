use core::alloc::Layout;
use core::cmp;
use core::mem::{align_of, size_of};
use core::ptr::NonNull;

use rust_alloc::alloc;

use super::{Alloc, AllocError, Allocator, GlobalAllocator};

/// Global buffer that can be used in combination with an [`Allocator`].
///
/// This uses the global allocator.
///
/// # Examples
///
/// ```
/// use musli::alloc::{Global, Vec};
///
/// let alloc = Global::new();
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Global;

impl Global {
    /// Construct a new global allocator.
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for Global {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAllocator for Global {
    #[inline]
    fn __do_not_implement() {}

    #[inline]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn slice_from_raw_parts<T>(ptr: NonNull<T>, len: usize) -> Self::Alloc<T> {
        GlobalAlloc {
            data: ptr,
            size: len,
        }
    }
}

unsafe impl Allocator for Global {
    #[inline]
    fn __do_not_implement() {}

    const IS_GLOBAL: bool = true;

    type Alloc<T> = GlobalAlloc<T>;

    #[inline]
    fn alloc<T>(self, value: T) -> Result<Self::Alloc<T>, AllocError> {
        let mut raw = GlobalAlloc::<T>::alloc()?;

        if size_of::<T>() != 0 {
            // SAFETY: The above ensures the data has been allocated.
            unsafe {
                raw.as_mut_ptr().write(value);
            }
        }

        Ok(raw)
    }

    #[inline]
    fn alloc_empty<T>(self) -> Self::Alloc<T> {
        GlobalAlloc::DANGLING
    }
}

/// A vector-backed allocation.
pub struct GlobalAlloc<T> {
    /// Pointer to the allocated region.
    data: NonNull<T>,
    /// The size in number of `T` elements in the region.
    size: usize,
}

impl<T> GlobalAlloc<T> {
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
                size: 1,
            });
        }

        unsafe {
            let data = alloc::alloc(Layout::new::<T>());

            if data.is_null() {
                return Err(AllocError);
            }

            Ok(Self {
                data: NonNull::new_unchecked(data).cast(),
                size: 1,
            })
        }
    }
}

unsafe impl<T> Send for GlobalAlloc<T> where T: Send {}
unsafe impl<T> Sync for GlobalAlloc<T> where T: Sync {}

impl<T> Alloc<T> for GlobalAlloc<T> {
    #[inline]
    fn as_ptr(&self) -> *const T {
        self.data.as_ptr().cast_const().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_ptr().cast()
    }

    #[inline]
    fn capacity(&self) -> usize {
        if size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.size
        }
    }

    #[inline]
    fn resize(&mut self, len: usize, additional: usize) -> Result<(), AllocError> {
        if size_of::<T>() == 0 {
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
        B: Alloc<T>,
    {
        if size_of::<T>() == 0 {
            return Ok(());
        }

        Err(other)
    }
}

impl<T> GlobalAlloc<T> {
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
        if size_of::<T>() == 0 || self.size == 0 {
            return;
        }

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

impl<T> Drop for GlobalAlloc<T> {
    #[inline]
    fn drop(&mut self) {
        self.free();
    }
}
