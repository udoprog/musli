use core::alloc::Layout;
use core::cmp;
use core::mem::{align_of, size_of};
use core::ptr::NonNull;

use rust_alloc::alloc;

use super::{Allocator, RawVec};

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
#[non_exhaustive]
pub struct System;

impl System {
    /// Construct a new system allocator.
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for System {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator for System {
    type RawVec<'this, T>
        = SystemBuf<T>
    where
        Self: 'this,
        T: 'this;

    #[inline]
    fn new_raw_vec<'a, T>(&'a self) -> Self::RawVec<'a, T>
    where
        T: 'a,
    {
        SystemBuf::DANGLING
    }
}

/// A vector-backed allocation.
pub struct SystemBuf<T> {
    /// Pointer to the allocated region.
    data: NonNull<T>,
    /// The size in number of `T` elements in the region.
    size: usize,
}

impl<T> RawVec<T> for SystemBuf<T> {
    #[inline]
    fn resize(&mut self, len: usize, additional: usize) -> bool {
        if additional == 0 || size_of::<T>() == 0 {
            return true;
        }

        self.reserve(len, additional)
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        self.data.as_ptr().cast_const().cast()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_ptr().cast()
    }

    #[inline]
    fn try_merge<B>(&mut self, _: usize, other: B, _: usize) -> Result<(), B>
    where
        B: RawVec<T>,
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
    fn drop(&mut self) {
        self.free();
    }
}
