//! Trait for governing how a particular sink of bytes is written to.
//!
//! To adapt [std::io::Write] types, see the [wrap][crate::wrap::wrap] function.

#[cfg(feature = "alloc")]
use core::convert::Infallible;
use core::fmt;
use core::mem::take;

use musli::Context;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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
            "tried to write {n} bytes to slice, with a remaining capacity of {capacity}"
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

    /// Write bytes to the current writer.
    fn write_bytes<'buf, C>(&mut self, cx: &mut C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>;

    /// Write a single byte.
    #[inline]
    fn write_byte<'buf, C>(&mut self, cx: &mut C, b: u8) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.write_bytes(cx, &[b])
    }

    /// Write an array to the current writer.
    #[inline]
    fn write_array<'buf, C, const N: usize>(
        &mut self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.write_bytes(cx, &array)
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
    fn write_bytes<'buf, C>(&mut self, cx: &mut C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (*self).write_bytes(cx, bytes)
    }

    #[inline]
    fn write_byte<'buf, C>(&mut self, cx: &mut C, b: u8) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (*self).write_byte(cx, b)
    }

    #[inline]
    fn write_array<'buf, C, const N: usize>(
        &mut self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (*self).write_array(cx, array)
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
    fn write_bytes<'buf, C>(&mut self, cx: &mut C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.extend_from_slice(bytes);
        cx.advance(bytes.len());
        Ok(())
    }

    #[inline]
    fn write_byte<'buf, C>(&mut self, cx: &mut C, b: u8) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.push(b);
        cx.advance(1);
        Ok(())
    }

    #[inline]
    fn write_array<'buf, C, const N: usize>(
        &mut self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.extend_from_slice(&array[..]);
        cx.advance(N);
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
    fn write_bytes<'buf, C>(&mut self, cx: &mut C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
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
    fn write_byte<'buf, C>(&mut self, cx: &mut C, b: u8) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
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

    #[inline]
    fn write_array<'buf, C, const N: usize>(
        &mut self,
        cx: &mut C,
        array: [u8; N],
    ) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.len() < N {
            return Err(cx.message(format_args!(
                "Buffer overflow, remaining is {} while tried to write {}",
                self.len(),
                N
            )));
        }

        let next = take(self);
        let (this, next) = next.split_at_mut(N);
        this.copy_from_slice(&array[..]);
        *self = next;
        Ok(())
    }
}
