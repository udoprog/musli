use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::slice;

use alloc::vec::Vec;

use musli::context::Buffer;

use crate::allocator::Allocator;

/// Buffer used in combination with [`AllocContext`].
///
/// This can be safely re-used.
pub struct Alloc {
    // This must be an unsafe cell, since it's mutably accessed through an
    // immutable pointers. We simply make sure that those accesses do not
    // clobber each other, which we can do since the API is restricted through
    // the `Buffer` trait.
    data: UnsafeCell<Vec<u8>>,
}

impl Alloc {
    /// Construct a new allocator.
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(Vec::new()),
        }
    }
}

impl Default for Alloc {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Allocator for &'a Alloc {
    type Buf = Buf<'a>;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf {
        unsafe {
            let n = (*self.data.get()).len();

            Buf {
                base: n,
                len: 0,
                data: &self.data,
            }
        }
    }
}

/// A vector-backed allocation.
pub struct Buf<'a> {
    base: usize,
    len: usize,
    data: &'a UnsafeCell<Vec<u8>>,
}

impl<'a> Buffer for Buf<'a> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> bool {
        unsafe {
            let data = &mut *self.data.get();
            assert_eq!(data.len(), self.base.wrapping_add(self.len));
            data.extend_from_slice(bytes);
            self.len = self.len.wrapping_add(bytes.len());
        }

        true
    }

    #[inline]
    fn write_at(&mut self, at: usize, bytes: &[u8]) -> bool {
        unsafe {
            if at.wrapping_add(bytes.len()) > self.len {
                return false;
            }

            let data = &mut *self.data.get();

            let Some(data) = data.get_mut(at..at.wrapping_add(bytes.len())) else {
                return false;
            };

            data.copy_from_slice(bytes);
            true
        }
    }

    #[inline]
    fn copy_back<B>(&mut self, other: B) -> bool
    where
        B: Buffer,
    {
        let (ptr, from, len) = other.raw_parts();

        unsafe {
            let data = &mut *self.data.get();
            let same = ptr::eq(ptr, data.as_ptr());
            let to = self.base.wrapping_add(self.len);

            data.reserve(len);

            if same {
                if from != to {
                    assert!(from.wrapping_add(len) <= data.len());
                    let from = data.as_ptr().wrapping_add(from);
                    let to = data.as_mut_ptr().wrapping_add(to);
                    ptr::copy(from, to, len);
                }

                // We forget the other buffer, so that it doesn't clobber the
                // underlying allocator data when dropped.
                mem::forget(other);
            } else {
                let from = ptr.wrapping_add(from);
                let to = data.as_mut_ptr().wrapping_add(to);
                ptr::copy_nonoverlapping(from, to, len);
            }

            self.len = self.len.wrapping_add(len);
            data.set_len(to.wrapping_add(len));
            true
        }
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn raw_parts(&self) -> (*const u8, usize, usize) {
        unsafe {
            let data = &*self.data.get();
            (data.as_ptr(), self.base, self.len)
        }
    }

    #[inline(always)]
    unsafe fn as_slice(&self) -> &[u8] {
        unsafe {
            let data = &*self.data.get();
            slice::from_raw_parts(data.as_ptr().wrapping_add(self.base), self.len)
        }
    }
}

impl<'a> Drop for Buf<'a> {
    fn drop(&mut self) {
        // SAFETY: During construction of the buffer, we fetch the length of the
        // vector which is known to be initialized. Since the only way the
        // vector can be extended is through `Buffer::write`.
        unsafe {
            let data = &mut *self.data.get();
            data.set_len(self.base);
        }
    }
}
