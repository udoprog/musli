//! Trait governing how to write bytes.
//!
//! To adapt [`std::io::Write`] types, see the [`wrap`] function.
//!
//! [`wrap`]: crate::wrap::wrap

use core::fmt;
use core::mem::take;

use crate::alloc::{Allocator, Vec};
use crate::Context;

/// The trait governing how a writer works.
pub trait Writer {
    /// Reborrowed type.
    ///
    /// Why oh why would we want to do this over having a simple `&'this mut T`?
    ///
    /// We want to avoid recursive types, which will blow up the compiler. And
    /// the above is a typical example of when that can go wrong. This ensures
    /// that each call to `borrow_mut` dereferences the [`Reader`] at each step to
    /// avoid constructing a large muted type, like `&mut &mut &mut VecWriter`.
    ///
    /// [`Reader`]: crate::reader::Reader
    type Mut<'this>: Writer
    where
        Self: 'this;

    /// Reborrow the current type.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Write a buffer to the current writer.
    fn extend<C>(&mut self, cx: &C, buffer: Vec<'_, u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: ?Sized + Context;

    /// Write bytes to the current writer.
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context;

    /// Write a single byte.
    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.write_bytes(cx, &[b])
    }
}

impl<W> Writer for &mut W
where
    W: ?Sized + Writer,
{
    type Mut<'this>
        = &'this mut W
    where
        Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: &C, buffer: Vec<'_, u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        (*self).extend(cx, buffer)
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        (*self).write_bytes(cx, bytes)
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        (*self).write_byte(cx, b)
    }
}

#[cfg(feature = "alloc")]
impl Writer for rust_alloc::vec::Vec<u8> {
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: &C, buffer: Vec<'_, u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, buffer.as_slice())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.extend_from_slice(bytes);
        cx.advance(bytes.len());
        Ok(())
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: &C, b: u8) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.push(b);
        cx.advance(1);
        Ok(())
    }
}

impl Writer for &mut [u8] {
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: &C, buffer: Vec<'_, u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, buffer.as_slice())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        if self.len() < bytes.len() {
            return Err(cx.message(SliceOverflow {
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
        C: ?Sized + Context,
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

/// A writer that writes against an underlying [`Vec`].
pub struct BufWriter<'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    buf: Vec<'a, u8, A>,
}

impl<'a, A> BufWriter<'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    /// Construct a new buffer writer.
    pub fn new(alloc: &'a A) -> Self {
        Self {
            buf: Vec::new_in(alloc),
        }
    }

    /// Coerce into inner buffer.
    pub fn into_inner(self) -> Vec<'a, u8, A> {
        self.buf
    }
}

impl<'a, A> Writer for BufWriter<'a, A>
where
    A: 'a + ?Sized + Allocator,
{
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: &C, buffer: Vec<'_, u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        if !self.buf.write(buffer.as_slice()) {
            return Err(cx.message("Buffer overflow"));
        }

        Ok(())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: &C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        if !self.buf.write(bytes) {
            return Err(cx.message("Buffer overflow"));
        }

        Ok(())
    }
}

/// Overflow when trying to write to a slice.
#[derive(Debug)]
struct SliceOverflow {
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
