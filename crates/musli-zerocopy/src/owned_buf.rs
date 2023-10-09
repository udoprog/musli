#[cfg(test)]
mod tests;

use core::ptr;

use alloc::vec::Vec;

use crate::buf::Buf;
use crate::error::{Error, ErrorKind};
use crate::ptr::Ptr;
use crate::ref_::Ref;
use crate::to_buf::{UnsizedZeroCopy, ZeroCopy};
use crate::unsized_ref::UnsizedRef;

/// An owned buffer.
pub struct OwnedBuf {
    data: Vec<u8>,
    align: usize,
}

impl OwnedBuf {
    /// Construct a new empty buffer.
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            align: 0,
        }
    }

    /// Alignment of the buffer.
    pub fn align(&self) -> usize {
        self.align
    }

    /// Get the buffer as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Write a value to the buffer.
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let first = buf.insert_unsized("first")?;
    /// let second = buf.insert_unsized("second")?;
    ///
    /// let buf = buf.as_buf()?;
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_unsized<T>(&mut self, value: &T) -> Result<UnsizedRef<T>, Error>
    where
        T: ?Sized + UnsizedZeroCopy,
    {
        let ptr = self.ptr(value.align());
        value.write_to(self)?;
        Ok(UnsizedRef::new(ptr, value.len()))
    }

    /// Insert a value with the given size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::ZeroCopy;
    /// use musli_zerocopy::{Error, OwnedBuf, UnsizedRef};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     string: UnsizedRef<str>,
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let string = buf.insert_unsized("string")?;
    /// let custom = buf.insert_sized(Custom { field: 1, string })?;
    /// let custom2 = buf.insert_sized(Custom { field: 2, string })?;
    ///
    /// let buf = buf.as_buf()?;
    ///
    /// let custom = buf.load(custom)?;
    /// assert_eq!(custom.field, 1);
    /// assert_eq!(buf.load(custom.string)?, "string");
    ///
    /// let custom2 = buf.load(custom2)?;
    /// assert_eq!(custom2.field, 2);
    /// assert_eq!(buf.load(custom2.string)?, "string");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_sized<T>(&mut self, value: T) -> Result<Ref<T>, Error>
    where
        T: ZeroCopy,
    {
        let ptr = self.ptr(T::ALIGN);
        value.write_to(self)?;
        Ok(Ref::new(ptr))
    }

    /// Write the bytes of a value.
    pub fn write<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        self.align_buf(T::ALIGN);
        value.write_to(self)?;
        Ok(())
    }

    /// Extend the buffer from a slice.
    #[inline]
    pub(crate) fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    /// Access the current buffer for reading.
    pub fn as_buf(&self) -> Result<&Buf, Error> {
        if (self.data.as_ptr() as usize) % self.align != 0 {
            return Err(Error::new(ErrorKind::BadAlignment {
                ptr: self.data.as_ptr() as usize,
                align: self.align,
            }));
        }

        Ok(Buf::new(&self.data))
    }

    /// Align the current write cursor.
    pub(crate) fn align_buf(&mut self, align: usize) {
        let len = self.data.len().next_multiple_of(align);
        self.align = self.align.max(align);
        self.data.resize(len, 0);
    }

    /// Get an aligned pointer.
    pub(crate) fn ptr(&mut self, align: usize) -> Ptr {
        self.align_buf(align);
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
