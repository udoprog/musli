//! Trait for governing how a particular source of bytes is read.
//!
//! `musli` requires all sources to reference the complete data being read from
//! it which allows it to make the assumption the bytes are always returned with
//! the `'de` lifetime.

use core::fmt;

use musli::error::Error;

/// Trait governing how a source of bytes is read.
///
/// This requires the reader to be able to hand out contiguous references to the
/// byte source through [Reader::read_bytes].
pub trait Reader<'de> {
    /// Error type raised by the current reader.
    type Error: Error;

    /// The position of the reader.
    fn pos(&self) -> Option<usize>;

    /// Skip over the given number of bytes.
    fn skip(&mut self, n: usize) -> Result<(), Self::Error>;

    /// Read a slice into the given buffer.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let source = self.read_bytes(buf.len())?;
        buf.copy_from_slice(source);
        Ok(())
    }

    /// Read a slice out of the current reader.
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error>;

    /// Read a single byte.
    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let [byte] = self.read_array::<1>()?;
        Ok(byte)
    }

    /// Read an array out of the current reader.
    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        let mut output = [0u8; N];
        output.copy_from_slice(self.read_bytes(N)?);
        Ok(output)
    }

    /// Keep an accurate record of the position within the reader.
    fn with_position(self) -> WithPosition<Self>
    where
        Self: Sized,
    {
        WithPosition {
            pos: 0,
            reader: self,
        }
    }
}

decl_message_repr!(SliceReaderErrorRepr, "error reading from slice");

/// An error raised while decoding a slice.
#[derive(Debug)]
pub struct SliceReaderError(SliceReaderErrorRepr);

impl fmt::Display for SliceReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for SliceReaderError {
    #[inline]
    fn custom<T>(message: T) -> Self
    where
        T: 'static + Send + Sync + fmt::Display + fmt::Debug,
    {
        Self(SliceReaderErrorRepr::collect(message))
    }

    #[inline]
    fn collect_from_display<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self(SliceReaderErrorRepr::collect(message))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SliceReaderError {}

impl<'de> Reader<'de> for &'de [u8] {
    type Error = SliceReaderError;

    #[inline]
    fn pos(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        if self.len() < n {
            return Err(SliceReaderError::custom("buffer underflow"));
        }

        let (_, tail) = self.split_at(n);
        *self = tail;
        Ok(())
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        if self.len() < n {
            return Err(SliceReaderError::custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(n);
        *self = tail;
        Ok(head)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        if self.len() < buf.len() {
            return Err(SliceReaderError::custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(buf.len());
        buf.copy_from_slice(head);
        *self = tail;
        Ok(())
    }
}

impl<'de, R> Reader<'de> for &mut R
where
    R: ?Sized + Reader<'de>,
{
    type Error = R::Error;

    #[inline]
    fn pos(&self) -> Option<usize> {
        (**self).pos()
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        (**self).skip(n)
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        (**self).read_bytes(n)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        (**self).read(buf)
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        (**self).read_byte()
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        (**self).read_array()
    }
}

/// Keep a record of the current position.
///
/// Constructed through [Reader::with_position].
pub struct WithPosition<R> {
    pos: usize,
    reader: R,
}

impl<'de, R> Reader<'de> for WithPosition<R>
where
    R: Reader<'de>,
{
    type Error = R::Error;

    #[inline]
    fn pos(&self) -> Option<usize> {
        Some(self.pos)
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.reader.skip(n)?;
        self.pos += n;
        Ok(())
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], Self::Error> {
        let bytes = self.reader.read_bytes(n)?;
        self.pos += bytes.len();
        Ok(bytes)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.reader.read(buf)?;
        self.pos += buf.len();
        Ok(())
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let b = self.reader.read_byte()?;
        self.pos += 1;
        Ok(b)
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        let array = self.reader.read_array()?;
        self.pos += N;
        Ok(array)
    }
}
