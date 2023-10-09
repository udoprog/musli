use core::alloc::Layout;
use core::mem;
use core::str;

use crate::buf::{AnyValue, Buf, BufMut};
use crate::error::{Error, ErrorKind};

/// Trait governing how to write an unsized buffer.
pub unsafe trait UnsizedZeroCopy {
    /// Alignment of the pointed to data. We can only support unsized types
    /// which have a known alignment.
    const ALIGN: usize;

    /// The length of the unsized value.
    fn len(&self) -> usize;

    /// Write to the owned buffer.
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut;

    /// Validate the buffer as this type.
    fn read_from(buf: &Buf) -> Result<&Self, Error>;
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

    /// Indicates if the type can inhabit all possible bit patterns within its
    /// `SIZE`.
    const ANY_BITS: bool = false;

    /// Write to the owned buffer.
    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut;

    /// Validate the buffer as this type.
    fn read_from(buf: &Buf) -> Result<&Self, Error>;

    /// Just validate the current buffer under the assumption that it is
    /// correctly aligned for the current type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is correctly sized and aligned
    /// per the requirements of this type.
    unsafe fn validate_aligned(buf: &Buf) -> Result<(), Error>;
}

unsafe impl UnsizedZeroCopy for str {
    const ALIGN: usize = mem::align_of::<u8>();

    fn len(&self) -> usize {
        <str>::len(self)
    }

    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.extend_from_slice(self.as_bytes())
    }

    fn read_from(buf: &Buf) -> Result<&Self, Error> {
        str::from_utf8(buf.as_bytes()).map_err(|error| Error::new(ErrorKind::Utf8Error { error }))
    }
}

unsafe impl UnsizedZeroCopy for [u8] {
    const ALIGN: usize = mem::align_of::<u8>();

    fn len(&self) -> usize {
        <[_]>::len(self)
    }

    fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
    where
        B: BufMut,
    {
        buf.extend_from_slice(self)
    }

    fn read_from(buf: &Buf) -> Result<&Self, Error> {
        Ok(buf.as_bytes())
    }
}

macro_rules! impl_number {
    ($ty:ty) => {
        unsafe impl ZeroCopy for $ty {
            const ANY_BITS: bool = true;

            fn write_to<B: ?Sized>(&self, buf: &mut B) -> Result<(), Error>
            where
                B: BufMut,
            {
                buf.extend_from_slice(&<$ty>::to_ne_bytes(*self)[..])
            }

            fn read_from(buf: &Buf) -> Result<&Self, Error> {
                if !buf.is_compatible(core::alloc::Layout::new::<$ty>()) {
                    return Err(Error::new(ErrorKind::LayoutMismatch {
                        layout: Layout::new::<$ty>(),
                        buf: buf.range(),
                    }));
                }

                Ok(unsafe { buf.cast() })
            }

            // NB: Numerical types can inhabit any bit pattern.
            unsafe fn validate_aligned(_: &Buf) -> Result<(), Error> {
                Ok(())
            }
        }

        unsafe impl AnyValue for $ty {
            type Target = $ty;

            fn visit<V, O>(&self, _: &Buf, visitor: V) -> Result<O, Error>
            where
                V: FnOnce(&Self::Target) -> O,
            {
                Ok(visitor(self))
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
