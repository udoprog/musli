use core::mem::{replace, size_of, size_of_val};
use core::ptr;

use crate::buf::BufMut;
use crate::buf::StructPadder;
use crate::traits::ZeroCopy;

pub(crate) struct RawBufMut {
    start: *mut u8,
}

impl RawBufMut {
    pub(crate) fn new(start: *mut u8) -> Self {
        Self { start }
    }
}

impl BufMut for RawBufMut {
    unsafe fn store_bytes(&mut self, bytes: &[u8]) {
        unsafe {
            self.start
                .copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
            self.start = self.start.wrapping_add(bytes.len());
        }
    }

    unsafe fn store_bits<T>(&mut self, value: T)
    where
        T: ZeroCopy,
    {
        unsafe {
            ptr::write_unaligned(self.start.cast::<T>(), value);
            self.start = self.start.wrapping_add(size_of::<T>());
        }
    }

    unsafe fn store<T>(&mut self, value: &T)
    where
        T: ZeroCopy,
    {
        value.store_to(self);
    }

    unsafe fn store_struct<T>(&mut self, value: &T) -> StructPadder<'_, T>
    where
        T: ZeroCopy,
    {
        let end = self.start.wrapping_add(size_of::<T>());

        unsafe {
            self.start
                .copy_from_nonoverlapping((value as *const T).cast::<u8>(), size_of::<T>());
        }

        let start = replace(&mut self.start, end);
        StructPadder::new(start)
    }

    unsafe fn store_array<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        if T::PADDED {
            for value in values {
                value.store_to(self);
            }
        } else {
            let size = size_of_val(values);
            self.start
                .copy_from_nonoverlapping(values.as_ptr().cast::<u8>(), size);
            self.start = self.start.wrapping_add(size);
        }
    }
}
