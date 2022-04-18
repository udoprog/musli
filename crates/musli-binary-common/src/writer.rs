//! Trait for governing how a particular sink of bytes is written to.
//!
//! To adapt [std::io::Write] types, see the [wrap][crate::io::wrap] function.

use core::fmt;

use musli::error::Error;

/// The trait governing how a writer works.
pub trait Writer {
    /// The error type raised by the writer.
    type Error: Error;

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
    fn collect_from_display<T>(message: T) -> Self
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
