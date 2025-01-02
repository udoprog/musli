//! Trait governing how to write bytes.
//!
//! To adapt [`std::io::Write`] types, see the [`wrap`] function.
//!
//! [`wrap`]: crate::wrap::wrap

mod slice_mut_writer;
pub use self::slice_mut_writer::SliceMutWriter;

use core::fmt;

use crate::alloc::Vec;
use crate::{Allocator, Context};

mod sealed {
    use super::Writer;

    pub trait Sealed {}
    impl<W> Sealed for &mut W where W: ?Sized + Writer {}
    #[cfg(feature = "std")]
    impl<W> Sealed for crate::wrap::Wrap<W> where W: std::io::Write {}
    impl Sealed for &mut [u8] {}
}

/// Coerce a type into a [`Writer`].
pub trait IntoWriter: self::sealed::Sealed {
    /// The output of the writer which will be returned after writing.
    type Ok;

    /// The writer type.
    type Writer: Writer<Ok = Self::Ok>;

    /// Convert the type into a writer.
    fn into_writer(self) -> Self::Writer;
}

/// The trait governing how a writer works.
pub trait Writer {
    /// The value returned from writing the value.
    type Ok;

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

    /// Finalize the writer and return the output.
    fn finish<C>(&mut self, cx: C) -> Result<Self::Ok, C::Error>
    where
        C: Context;

    /// Reborrow the current type.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Write a buffer to the current writer.
    fn extend<C>(&mut self, cx: C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: Context;

    /// Write bytes to the current writer.
    fn write_bytes<C>(&mut self, cx: C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context;

    /// Write a single byte.
    #[inline]
    fn write_byte<C>(&mut self, cx: C, b: u8) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.write_bytes(cx, &[b])
    }
}

impl<'a, W> IntoWriter for &'a mut W
where
    W: ?Sized + Writer,
{
    type Ok = W::Ok;
    type Writer = &'a mut W;

    #[inline]
    fn into_writer(self) -> Self::Writer {
        self
    }
}

impl<W> Writer for &mut W
where
    W: ?Sized + Writer,
{
    type Ok = W::Ok;
    type Mut<'this>
        = &'this mut W
    where
        Self: 'this;

    #[inline]
    fn finish<C>(&mut self, cx: C) -> Result<Self::Ok, C::Error>
    where
        C: Context,
    {
        (*self).finish(cx)
    }

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: Context,
    {
        (*self).extend(cx, buffer)
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        (*self).write_bytes(cx, bytes)
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: C, b: u8) -> Result<(), C::Error>
    where
        C: Context,
    {
        (*self).write_byte(cx, b)
    }
}

#[cfg(feature = "alloc")]
impl Writer for rust_alloc::vec::Vec<u8> {
    type Ok = ();
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn finish<C>(&mut self, _: C) -> Result<Self::Ok, C::Error>
    where
        C: Context,
    {
        Ok(())
    }

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: Context,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, buffer.as_slice())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.extend_from_slice(bytes);
        cx.advance(bytes.len());
        Ok(())
    }

    #[inline]
    fn write_byte<C>(&mut self, cx: C, b: u8) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.push(b);
        cx.advance(1);
        Ok(())
    }
}

impl<'a> IntoWriter for &'a mut [u8] {
    type Ok = usize;
    type Writer = SliceMutWriter<'a>;

    #[inline]
    fn into_writer(self) -> Self::Writer {
        SliceMutWriter::new(self)
    }
}

/// A writer that writes against an underlying [`Vec`].
pub struct BufWriter<A>
where
    A: Allocator,
{
    buf: Vec<u8, A>,
}

impl<A> BufWriter<A>
where
    A: Allocator,
{
    /// Construct a new buffer writer.
    pub fn new(alloc: A) -> Self {
        Self {
            buf: Vec::new_in(alloc),
        }
    }

    /// Coerce into inner buffer.
    pub fn into_inner(self) -> Vec<u8, A> {
        self.buf
    }
}

impl<A> Writer for BufWriter<A>
where
    A: Allocator,
{
    type Ok = ();
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn finish<C>(&mut self, _: C) -> Result<Self::Ok, C::Error>
    where
        C: Context,
    {
        Ok(())
    }

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.buf
            .extend_from_slice(buffer.as_slice())
            .map_err(cx.map())?;
        Ok(())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.buf.extend_from_slice(bytes).map_err(cx.map())?;
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
