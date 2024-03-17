//! Fixed capacity containers.

// Parts borrowed under the MIT license from
// https://github.com/bluss/arrayvec/tree/2c92a59bed0d1669cede3806000d2e61d5994c4e

use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

/// An error raised when we are at capacity.
#[non_exhaustive]
pub struct CapacityError;

/// A fixed capacity vector allocated on the stack.
pub struct FixedVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    /// Construct a new empty fixed vector.
    pub(crate) const fn new() -> FixedVec<T, N> {
        unsafe {
            FixedVec {
                data: MaybeUninit::uninit().assume_init(),
                len: 0,
            }
        }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    #[inline]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    /// Try to push an element onto the fixed vector.
    pub(crate) fn try_push(&mut self, element: T) -> Result<(), CapacityError> {
        if self.len >= N {
            return Err(CapacityError);
        }

        unsafe {
            ptr::write(self.as_mut_ptr().wrapping_add(self.len), element);
            self.len += 1;
        }

        Ok(())
    }

    /// Pop the last element in the fixed vector.
    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        unsafe {
            let new_len = self.len - 1;
            self.len = new_len;
            Some(ptr::read(self.as_ptr().wrapping_add(new_len)))
        }
    }

    pub(crate) fn clear(&mut self) {
        if self.len == 0 {
            return;
        }

        let len = mem::take(&mut self.len);

        if mem::needs_drop::<T>() {
            unsafe {
                let tail = slice::from_raw_parts_mut(self.as_mut_ptr(), len);
                ptr::drop_in_place(tail);
            }
        }
    }
}

impl<T, const N: usize> Deref for FixedVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for FixedVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Drop for FixedVec<T, N> {
    #[inline]
    fn drop(&mut self) {
        self.clear()
    }
}
