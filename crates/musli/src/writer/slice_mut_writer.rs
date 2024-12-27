use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::alloc::Vec;
use crate::Context;

use super::{SliceOverflow, Writer};

/// A writer into a slice.
pub struct SliceMutWriter<'a> {
    start: NonNull<u8>,
    end: NonNull<u8>,
    _marker: PhantomData<&'a mut [u8]>,
}

impl<'a> SliceMutWriter<'a> {
    pub(super) fn new(slice: &'a mut [u8]) -> Self {
        // SAFETY: The slice is valid for the duration of the writer.
        unsafe {
            let range = slice.as_mut_ptr_range();
            let start = NonNull::new_unchecked(range.start);
            let end = NonNull::new_unchecked(range.end);
            Self {
                start,
                end,
                _marker: PhantomData,
            }
        }
    }

    #[inline]
    pub(crate) fn remaining(&self) -> usize {
        self.end.as_ptr() as usize - self.start.as_ptr() as usize
    }
}

impl<'a> Writer for SliceMutWriter<'a> {
    type Ok = usize;

    type Mut<'this>
        = &'this mut SliceMutWriter<'a>
    where
        Self: 'this;

    #[inline]
    fn finish<C>(&mut self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: ?Sized + Context,
    {
        Ok(self.remaining())
    }

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: &C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, buffer.as_slice())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        let end = self.start.as_ptr().wrapping_add(bytes.len());

        if end > self.end.as_ptr() || end < self.start.as_ptr() {
            return Err(cx.message(SliceOverflow {
                n: bytes.len(),
                capacity: self.remaining(),
            }));
        }

        // SAFETY: Construction of the writer ensures the range is valid.
        unsafe {
            self.start
                .as_ptr()
                .copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
            self.start = NonNull::new_unchecked(end);
        }

        Ok(())
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        if self.start == self.end {
            return Err(cx.message(format_args!(
                "Buffer overflow, remaining is 0 while tried to write 1"
            )));
        }

        // SAFETY: Construction of the writer ensures the range is valid.
        unsafe {
            self.start.write(b);
            self.start = self.start.add(1);
        }

        Ok(())
    }
}
