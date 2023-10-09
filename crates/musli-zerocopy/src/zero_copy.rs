use core::alloc::Layout;
use core::mem;
use core::str;

use crate::buf::Buf;
use crate::error::{Error, ErrorKind};
use crate::owned_buf::OwnedBuf;

/// Trait governing how to write an unsized buffer.
pub unsafe trait UnsizedZeroCopy {
    /// Alignment of the pointed to data. We can only support unsized types
    /// which have a known alignment.
    const ALIGN: usize;

    /// The length of the unsized value.
    fn len(&self) -> usize;

    /// Write to the owned buffer.
    fn write_to(&self, buf: &mut OwnedBuf) -> Result<(), Error>;

    /// Validate the buffer as this type.
    fn validate(buf: &Buf) -> Result<&Self, Error>;
}

/// Trait governing how to write a sized buffer.
///
/// # Safety
///
/// Caller must ensure that the pointed to data is `repr(C)`.
pub unsafe trait ZeroCopy: Sized {
    /// Size of the pointed to data.
    const SIZE: usize = mem::size_of::<Self>();

    /// Alignment of the pointed to data.
    const ALIGN: usize = mem::align_of::<Self>();

    /// Write to the owned buffer.
    fn write_to(&self, buf: &mut OwnedBuf) -> Result<(), Error>;

    /// Validate the buffer as this type.
    fn validate(buf: &Buf) -> Result<&Self, Error>;
}

unsafe impl UnsizedZeroCopy for str {
    const ALIGN: usize = mem::align_of::<u8>();

    fn len(&self) -> usize {
        <str>::len(self)
    }

    fn write_to(&self, buf: &mut OwnedBuf) -> Result<(), Error> {
        buf.extend_from_slice(self.as_bytes())
    }

    fn validate(buf: &Buf) -> Result<&Self, Error> {
        str::from_utf8(buf.as_bytes()).map_err(|error| Error::new(ErrorKind::Utf8Error { error }))
    }
}

unsafe impl UnsizedZeroCopy for [u8] {
    const ALIGN: usize = mem::align_of::<u8>();

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn write_to(&self, buf: &mut OwnedBuf) -> Result<(), Error> {
        buf.extend_from_slice(self)
    }

    fn validate(buf: &Buf) -> Result<&Self, Error> {
        Ok(buf.as_bytes())
    }
}

macro_rules! impl_number {
    ($ty:ty) => {
        unsafe impl ZeroCopy for $ty {
            fn write_to(&self, buf: &mut OwnedBuf) -> Result<(), Error> {
                buf.extend_from_slice(&<$ty>::to_ne_bytes(*self)[..])
            }

            fn validate(buf: &Buf) -> Result<&Self, Error> {
                if !buf.is_compatible(core::alloc::Layout::new::<$ty>()) {
                    return Err(Error::new(ErrorKind::LayoutMismatch {
                        layout: Layout::new::<$ty>(),
                        buf: buf.range(),
                    }));
                }

                Ok(unsafe { buf.cast() })
            }
        }
    };
}

impl_number!(usize);
impl_number!(isize);
impl_number!(u8);
impl_number!(u16);
impl_number!(u32);
impl_number!(u64);
impl_number!(u128);
impl_number!(i8);
impl_number!(i16);
impl_number!(i32);
impl_number!(i64);
impl_number!(i128);
