use core::mem::{replace, size_of, size_of_val};

use crate::buf::BufMut;
use crate::buf::Padder;
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
    unsafe fn store_bytes<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        unsafe {
            self.start
                .copy_from_nonoverlapping(values.as_ptr().cast(), size_of_val(values));
            self.start = self.start.wrapping_add(size_of_val(values));
        }
    }

    unsafe fn store_bits<T>(&mut self, value: *const T)
    where
        T: ZeroCopy,
    {
        unsafe {
            self.start
                .copy_from_nonoverlapping(value.cast(), size_of::<T>());
            self.start = self.start.wrapping_add(size_of::<T>());
        }
    }

    unsafe fn store<T>(&mut self, value: &T)
    where
        T: ZeroCopy,
    {
        T::store_to(value, self);
    }

    unsafe fn store_struct<T>(&mut self, value: *const T) -> Padder<'_, T>
    where
        T: ZeroCopy,
    {
        let end = self.start.wrapping_add(size_of::<T>());

        self.start
            .copy_from_nonoverlapping(value.cast(), size_of::<T>());

        let start = replace(&mut self.start, end);
        Padder::new(start)
    }

    unsafe fn store_array<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        if T::PADDED {
            for value in values {
                T::store_to(value, self);
            }
        } else {
            let size = size_of_val(values);
            self.start
                .copy_from_nonoverlapping(values.as_ptr().cast::<u8>(), size);
            self.start = self.start.wrapping_add(size);
        }
    }
}
