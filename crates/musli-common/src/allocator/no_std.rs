use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::slice;

use arrayvec::ArrayVec;
use musli::context::Buffer;

use crate::allocator::Allocator;

/// Buffer used in combination with a `Context`.
///
/// This type of allocator has a fixed capacity specified by `C` and can be
/// constructed statically.
pub struct NoStd<const C: usize> {
    // This must be an unsafe cell, since it's mutably accessed through an
    // immutable pointers. We simply make sure that those accesses do not
    // clobber each other, which we can do since the API is restricted through
    // the `Buffer` trait.
    scratch: UnsafeCell<ArrayVec<u8, C>>,
}

impl<const C: usize> NoStd<C> {
    /// Build a new no-std allocator.
    pub const fn new() -> Self {
        Self {
            scratch: UnsafeCell::new(ArrayVec::new_const()),
        }
    }
}

impl<const C: usize> Default for NoStd<C> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const C: usize> Allocator for &'a NoStd<C> {
    type Buf = Buf<'a, C>;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf {
        unsafe {
            let n = (*self.scratch.get()).len();

            Buf {
                base: n,
                len: 0,
                data: &self.scratch,
            }
        }
    }
}

/// A no-std allocated buffer.
pub struct Buf<'a, const C: usize> {
    base: usize,
    len: usize,
    data: &'a UnsafeCell<ArrayVec<u8, C>>,
}

impl<'a, const C: usize> Buffer for Buf<'a, C> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> bool {
        unsafe {
            let data = &mut *self.data.get();
            assert_eq!(data.len(), self.len.wrapping_add(self.base));

            if data.try_extend_from_slice(bytes).is_err() {
                return false;
            }

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

            if to.wrapping_add(len) > data.capacity() {
                return false;
            }

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

impl<'a, const C: usize> Drop for Buf<'a, C> {
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
