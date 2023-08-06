#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::ptr;

use crate::buf::Buf;
use crate::ptr::Ptr;
use crate::traits::{UnsizedToBuf, Write};
use crate::unsized_ref::UnsizedRef;
use crate::value_ref::ValueRef;

/// An owned buffer.
pub struct OwnedBuf {
    data: Vec<u8>,
}

impl OwnedBuf {
    /// Construct a new empty buffer.
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Get the buffer as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Write a value to the buffer.
    pub fn insert<T>(&mut self, value: T) -> ValueRef<T>
    where
        T: Write,
    {
        let ptr = self.ptr();
        value.write(self);
        ValueRef::new(ptr)
    }

    /// Write a value to the buffer.
    pub fn insert_unsized<T>(&mut self, value: &T) -> UnsizedRef<T>
    where
        T: ?Sized + UnsizedToBuf,
    {
        let ptr = self.ptr();
        value.write(self);
        UnsizedRef::new(ptr, value.len())
    }

    /// Extend the buffer from a slice.
    #[inline]
    pub(crate) fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Access the current buffer for reading.
    pub fn as_buf(&self) -> &Buf {
        Buf::new(&self.data)
    }

    /// Get the current pointer.
    pub(crate) fn ptr(&mut self) -> Ptr {
        Ptr::new(self.data.len())
    }

    /// Swap two pointer positions.
    ///
    /// The signature and calculation performed to swap guarantees that the
    /// elements do not overlap.
    pub(crate) fn swap(&mut self, base: usize, a: usize, b: usize, len: usize) {
        if a == b {
            return;
        }

        macro_rules! bounds_check {
            ($var:ident) => {
                assert! {
                    $var.wrapping_add(len) <= self.data.len(),
                    "range {}-{} out of bounds 0-{}",
                    $var,
                    $var.wrapping_add(len),
                    self.data.len()
                };
            };
        }

        let a = base.wrapping_add(a.wrapping_mul(len));
        let b = base.wrapping_add(b.wrapping_mul(len));

        bounds_check!(a);
        bounds_check!(b);

        let d = self.data.as_mut_ptr();

        // SAFETY: We've checked that the pointers are in bounds. The signature
        // of the function guarantees that the slices are non-overlapping.
        unsafe {
            ptr::swap_nonoverlapping(d.wrapping_add(a), d.wrapping_add(b), len);
        }
    }
}
