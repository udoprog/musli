//! Trait for governing how a particular sink of bytes is written to.
//!
//! To adapt [std::io::Write] types, see the [wrap][crate::io::wrap] function.

use musli::error::Error;

use crate::error::BufferError;

/// Maximum size used by a fixed length [Buffer].
pub const MAX_FIXED_BYTES_LEN: usize = 128;

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
    ///
    /// [Reader]: crate::reader::Reader
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

/// A buffer that roughly corresponds to a vector. For no-std environments this
/// has a fixed size and will error in case the size overflows.
#[derive(Default)]
pub struct Buffer {
    #[cfg(feature = "std")]
    buf: Vec<u8>,
    #[cfg(not(feature = "std"))]
    buf: FixedBytes<MAX_FIXED_BYTES_LEN, BufferError>,
}

impl Buffer {
    /// Constructs a new, empty `Buffer` with the specified capacity.
    ///
    /// Only available for `std` environment.
    #[cfg(feature = "std")]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    /// Construct a new empty buffer.
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "std")]
            buf: Vec::new(),
            #[cfg(not(feature = "std"))]
            buf: FixedBytes::new(),
        }
    }

    /// Get the buffer as a slice.
    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_slice()
    }

    /// Coerce into the backing vector in a std environment.
    #[cfg(feature = "std")]
    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }
}

impl Writer for Buffer {
    type Error = BufferError;
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    #[inline]
    fn write_byte(&mut self, b: u8) -> Result<(), Self::Error> {
        self.buf.push(b);
        Ok(())
    }

    #[inline]
    fn write_array<const N: usize>(&mut self, array: [u8; N]) -> Result<(), Self::Error> {
        self.buf.extend_from_slice(&array[..]);
        Ok(())
    }
}

#[cfg(feature = "std")]
impl Writer for Vec<u8> {
    type Error = BufferError;
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
