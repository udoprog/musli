use core::marker::PhantomData;

use crate::owned_buf::OwnedBuf;
use crate::ptr::Ptr;
use crate::traits::{Size, Write};
use crate::{Slice, SliceRef, UnsizedRef};

/// A builder for a zero-copy slice.
pub struct SliceBuilder<T> {
    buf: OwnedBuf,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> SliceBuilder<T>
where
    T: Size + Write,
{
    /// Construct a new slice builder.
    pub fn new() -> Self {
        Self {
            buf: OwnedBuf::new(),
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Push a value onto the slice.
    pub fn push(&mut self, data: T) {
        data.write(&mut self.buf);
        self.len += 1;
    }

    /// Swap the position of elements in the slice being build.
    pub fn swap(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }

        self.buf.swap(0, a, b, T::size());
    }

    /// Write a slice onto the slice builder.
    pub fn build(self, buf: &mut OwnedBuf) -> SliceRef<T> {
        let ptr = buf.ptr().wrapping_add(UnsizedRef::<T>::size());
        let repr = UnsizedRef::new(ptr, self.len);
        repr.write(buf);
        buf.extend_from_slice(self.buf.as_slice());
        SliceRef::new(repr)
    }

    pub(crate) fn as_slice(&self) -> Slice<'_, T> {
        Slice::new(UnsizedRef::new(Ptr::new(0), self.len), self.buf.as_buf())
    }
}
