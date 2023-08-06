use core::array;
use core::fmt;

use crate::error::{Error, ErrorKind};
use crate::ptr::Ptr;
use crate::traits::Bind;

/// A raw slice buffer.
#[repr(transparent)]
pub struct Buf {
    data: [u8],
}

impl Buf {
    /// Wrap the given slice as a buffer.
    pub fn new<T>(data: &T) -> &Buf
    where
        T: ?Sized + AsRef<[u8]>,
    {
        // SAFETY: The struct is repr(transparent) over [u8].
        unsafe { &*(data.as_ref() as *const _ as *const Buf) }
    }

    /// Get a slice of the given length from the underlying buffer.
    pub fn get_slice(&self, ptr: Ptr, len: usize) -> Result<&[u8], Error> {
        let start = ptr.as_usize();
        let end = start.wrapping_add(len);

        let Some(data) = self.data.get(start..end) else {
            return Err(Error::new(ErrorKind::OutOfBounds {
                start,
                end,
            }));
        };

        Ok(data)
    }

    /// Get an array.
    pub fn get_array<const N: usize>(&self, ptr: Ptr, len: usize) -> Result<[u8; N], Error> {
        let data = self.get_slice(ptr, len)?;
        Ok(array::from_fn(|n| data[n]))
    }

    /// Read the given pointer as `T`.
    pub fn read<'a, T>(&'a self, value: T) -> Result<T::Output, Error>
    where
        T: Bind<'a>,
    {
        value.bind(self)
    }
}

impl fmt::Debug for Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Buf").field(&self.data.len()).finish()
    }
}
