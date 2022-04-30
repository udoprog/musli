//! Trait for governing how a particular sink of bytes is written to.
//!
//! To adapt [std::io::Write] types, see the [wrap][crate::io::wrap] function.

use core::fmt;

use musli::error::Error;

/// The trait governing how a writer works.
pub trait Writer {
    /// The error type raised by the writer.
    type Error: Error;

    /// Reborrowed type.
    ///
    /// Why oh why would we want to do this over having a simple `&'this mut T`?
    ///
    /// We want to avoid recursive types, which will blow up the compiler. And
    /// the above is a typical example of when that can go wrong. This ensures
    /// that each call to `borrow_mut` dereferences the [Reader] at each step to
    /// avoid constructing a large muted type, like `&mut &mut &mut VecWriter`.
    type Mut<'this>: Writer<Error = Self::Error>
    where
        Self: 'this;

    /// Reborrow the current type.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Write bytes to the current writer.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Write a single byte.
    #[inline]
    fn write_byte(&mut self, b: u8) -> Result<(), Self::Error> {
        self.write_bytes(&[b])
    }

    /// Write an array to the current writer.
    #[inline]
    fn write_array<const N: usize>(&mut self, array: [u8; N]) -> Result<(), Self::Error> {
        self.write_bytes(&array)
    }
}

impl<W> Writer for &mut W
where
    W: ?Sized + Writer,
{
    type Error = W::Error;
    type Mut<'this> = W::Mut<'this> where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        (**self).borrow_mut()
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        (*self).write_bytes(bytes)
    }

    #[inline]
    fn write_byte(&mut self, b: u8) -> Result<(), Self::Error> {
        (*self).write_byte(b)
    }

    #[inline]
    fn write_array<const N: usize>(&mut self, array: [u8; N]) -> Result<(), Self::Error> {
        (*self).write_array(array)
    }
}

decl_message_repr!(VecWriterErrorRepr, "failed to write to vector");

/// An error raised while decoding a slice.
#[derive(Debug)]
pub struct VecWriterError(VecWriterErrorRepr);

impl fmt::Display for VecWriterError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for VecWriterError {
    #[inline]
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self(VecWriterErrorRepr::collect(message))
    }

    #[inline]
    fn message<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(VecWriterErrorRepr::collect(message))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VecWriterError {}

#[cfg(feature = "std")]
impl Writer for Vec<u8> {
    type Error = VecWriterError;
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.extend_from_slice(bytes);
        Ok(())
    }

    #[inline]
    fn write_byte(&mut self, b: u8) -> Result<(), Self::Error> {
        self.push(b);
        Ok(())
    }

    #[inline]
    fn write_array<const N: usize>(&mut self, array: [u8; N]) -> Result<(), Self::Error> {
        self.extend_from_slice(&array[..]);
        Ok(())
    }
}
