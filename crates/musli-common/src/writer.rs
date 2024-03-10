//! Trait for governing how a particular sink of bytes is written to.
//!
//! To adapt [std::io::Write] types, see the [wrap][crate::wrap::wrap] function.

#[cfg(feature = "alloc")]
use core::convert::Infallible;
use core::fmt;
use core::marker;
use core::mem::take;

use musli::Context;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use musli::context::Buffer;

/// Maximum size used by a fixed length [Buffer].
pub const MAX_FIXED_BYTES_LEN: usize = 128;

/// Overflow when trying to write to a slice.
#[derive(Debug)]
pub struct SliceOverflow {
    n: usize,
    capacity: usize,
}

impl fmt::Display for SliceOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SliceOverflow { n, capacity } = self;

        write!(
            f,
            "Tried to write {n} bytes to slice, with a remaining capacity of {capacity}"
        )
    }
}

/// The trait governing how a writer works.
pub trait Writer {
    /// The error type raised by the writer.
    type Error;

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

    /// Write a buffer to the current writer.
    fn write_buffer<C, B>(&mut self, cx: &C, buffer: B) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
        B: Buffer;

    /// Write bytes to the current writer.
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Write a single byte.
    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.write_bytes(cx, &[b])
    }
}

impl<W> Writer for &mut W
where
    W: ?Sized + Writer,
{
    type Error = W::Error;
    type Mut<'this> = &'this mut W where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_buffer<C, B>(&mut self, cx: &C, buffer: B) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
        B: Buffer,
    {
        (*self).write_buffer(cx, buffer)
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        (*self).write_bytes(cx, bytes)
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        (*self).write_byte(cx, b)
    }
}

#[cfg(feature = "alloc")]
impl Writer for Vec<u8> {
    type Error = Infallible;
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_buffer<C, B>(&mut self, cx: &C, buffer: B) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
        B: Buffer,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, unsafe { buffer.as_slice() })
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.extend_from_slice(bytes);
        cx.advance(bytes.len());
        Ok(())
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        self.push(b);
        cx.advance(1);
        Ok(())
    }
}

impl Writer for &mut [u8] {
    type Error = SliceOverflow;
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn write_buffer<C, B>(&mut self, cx: &C, buffer: B) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
        B: Buffer,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, unsafe { buffer.as_slice() })
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.len() < bytes.len() {
            return Err(cx.report(SliceOverflow {
                n: bytes.len(),
                capacity: self.len(),
            }));
        }

        let next = take(self);
        let (this, next) = next.split_at_mut(bytes.len());
        this.copy_from_slice(bytes);
        *self = next;
        Ok(())
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if self.is_empty() {
            return Err(cx.message(format_args!(
                "Buffer overflow, remaining is {} while tried to write 1",
                self.len()
            )));
        }

        self[0] = b;
        *self = &mut take(self)[1..];
        Ok(())
    }
}

/// A writer that writes against an underlying [`Buffer`].
pub struct BufferWriter<T, E> {
    buffer: T,
    _marker: marker::PhantomData<E>,
}

impl<T, E> BufferWriter<T, E> {
    /// Construct a new buffer writer.
    pub fn new(buffer: T) -> Self {
        Self {
            buffer,
            _marker: marker::PhantomData,
        }
    }

    /// Coerce into inner buffer.
    pub fn into_inner(self) -> T {
        self.buffer
    }
}

impl<T, E> Writer for BufferWriter<T, E>
where
    T: Buffer,
{
    type Error = E;

    type Mut<'this> = &'this mut Self
    where
        Self: 'this;

    #[inline(always)]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline(always)]
    fn write_buffer<C, B>(&mut self, cx: &C, buffer: B) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
        B: Buffer,
    {
        if !self.buffer.copy_back(buffer) {
            return Err(cx.message("Buffer overflow"));
        }

        Ok(())
    }

    #[inline(always)]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        if !self.buffer.write(bytes) {
            return Err(cx.message("Buffer overflow"));
        }

        Ok(())
    }
}
