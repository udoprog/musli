//! Trait for governing how a particular source of bytes is read.
//!
//! `musli` requires all sources to reference the complete data being read from
//! it which allows it to make the assumption the bytes are always returned with
//! the `'de` lifetime.

use core::array;
use core::fmt;
use core::marker;
use core::ops::Range;
use core::ptr;
use core::slice;

use musli::de::ValueVisitor;
use musli::error::Error;
use musli::Context;

use crate::error::BufferError;

/// Trait governing how a source of bytes is read.
///
/// This requires the reader to be able to hand out contiguous references to the
/// byte source through [Reader::read_bytes].
pub trait Reader<'de> {
    /// Error type raised by the current reader.
    type Error: Error;

    /// Type borrowed from self.
    ///
    /// Why oh why would we want to do this over having a simple `&'this mut T`?
    ///
    /// We want to avoid recursive types, which will blow up the compiler. And
    /// the above is a typical example of when that can go wrong. This ensures
    /// that each call to `borrow_mut` dereferences the [Reader] at each step to
    /// avoid constructing a large muted type, like `&mut &mut &mut
    /// SliceReader<'de>`.
    type Mut<'this>: Reader<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Borrow the current reader.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Skip over the given number of bytes.
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>;

    /// Peek the next value.
    fn peek<'buf, C>(&mut self, cx: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>;

    /// Read a slice into the given buffer.
    #[inline]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        struct Visitor<'a>(&'a mut [u8]);

        impl<'a, 'de, 'buf, C> ValueVisitor<'de, 'buf, C, [u8]> for Visitor<'a>
        where
            C: Context<'buf>,
        {
            type Ok = ();

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, cx: &mut C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, bytes)
            }

            #[inline]
            fn visit_ref(self, _: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                self.0.copy_from_slice(bytes);
                Ok(())
            }
        }

        self.read_bytes(cx, buf.len(), Visitor(buf))
    }

    /// Read a slice out of the current reader.
    fn read_bytes<'buf, C, V>(
        &mut self,
        cx: &mut C,
        n: usize,
        visitor: V,
    ) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, [u8]>;

    /// Read a single byte.
    #[inline]
    fn read_byte<'buf, C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let [byte] = self.read_array::<C, 1>(cx)?;
        Ok(byte)
    }

    /// Read an array out of the current reader.
    #[inline]
    fn read_array<'buf, C, const N: usize>(&mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        struct Visitor<const N: usize>([u8; N]);

        impl<'de, 'buf, const N: usize, C> ValueVisitor<'de, 'buf, C, [u8]> for Visitor<N>
        where
            C: Context<'buf>,
        {
            type Ok = [u8; N];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, cx: &mut C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, bytes)
            }

            #[inline]
            fn visit_ref(mut self, cx: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                self.0.copy_from_slice(bytes);
                cx.advance(bytes.len());
                Ok(self.0)
            }
        }

        self.read_bytes(cx, N, Visitor([0u8; N]))
    }

    /// Keep an accurate record of the position within the reader.
    fn limit(self, limit: usize) -> Limit<Self>
    where
        Self: Sized,
    {
        Limit {
            remaining: limit,
            reader: self,
        }
    }
}

impl<'de> Reader<'de> for &'de [u8] {
    type Error = BufferError;

    type Mut<'this> = &'this mut &'de [u8] where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.len() < n {
            return Err(cx.custom("buffer underflow"));
        }

        let (_, tail) = self.split_at(n);
        *self = tail;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.len() < buf.len() {
            return Err(cx.custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(buf.len());
        buf.copy_from_slice(head);
        *self = tail;
        cx.advance(buf.len());
        Ok(())
    }

    #[inline]
    fn read_bytes<'buf, C, V>(
        &mut self,
        cx: &mut C,
        n: usize,
        visitor: V,
    ) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, [u8]>,
    {
        if self.len() < n {
            return Err(cx.custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(n);
        *self = tail;
        let ok = visitor.visit_borrowed(cx, head)?;
        cx.advance(n);
        Ok(ok)
    }

    #[inline]
    fn read_byte<'buf, C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let &[first, ref tail @ ..] = *self else {
            return Err(cx.custom("buffer underflow"));
        };

        *self = tail;
        cx.advance(1);
        Ok(first)
    }

    #[inline]
    fn read_array<'buf, C, const N: usize>(&mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.len() < N {
            return Err(cx.custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(N);
        *self = tail;
        cx.advance(N);
        Ok(array::from_fn(|n| head[n]))
    }

    #[inline]
    fn peek<'buf, C>(&mut self, _: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(self.first().copied())
    }
}

/// An efficient [Reader] wrapper around a slice.
pub struct SliceReader<'de, E = BufferError> {
    range: Range<*const u8>,
    _marker: marker::PhantomData<(&'de [u8], E)>,
}

impl<'de, E> SliceReader<'de, E> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            range: slice.as_ptr_range(),
            _marker: marker::PhantomData,
        }
    }
}

impl<'de, E> Reader<'de> for SliceReader<'de, E>
where
    E: Error,
{
    type Error = E;

    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.range.start = bounds_check_add(cx, &self.range, n)?;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read_bytes<'buf, C, V>(
        &mut self,
        cx: &mut C,
        n: usize,
        visitor: V,
    ) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, [u8]>,
    {
        let outcome = bounds_check_add(cx, &self.range, n)?;

        let ok = unsafe {
            let bytes = slice::from_raw_parts(self.range.start, n);
            self.range.start = outcome;
            visitor.visit_borrowed(cx, bytes)?
        };

        cx.advance(n);
        Ok(ok)
    }

    #[inline]
    fn peek<'buf, C>(&mut self, _: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        if self.range.start == self.range.end {
            return Ok(None);
        }

        // SAFETY: we've checked that the elements are in bound above.
        unsafe { Ok(Some(ptr::read(self.range.start))) }
    }

    #[inline]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let outcome = bounds_check_add(cx, &self.range, buf.len())?;

        unsafe {
            ptr::copy_nonoverlapping(self.range.start, buf.as_mut_ptr(), buf.len());
            self.range.start = outcome;
        }

        cx.advance(buf.len());
        Ok(())
    }
}

#[inline]
fn bounds_check_add<'buf, C, E>(
    cx: &mut C,
    range: &Range<*const u8>,
    len: usize,
) -> Result<*const u8, C::Error>
where
    C: Context<'buf, Input = E>,
{
    let outcome = range.start.wrapping_add(len);

    if outcome > range.end || outcome < range.start {
        Err(cx.custom("buffer underflow"))
    } else {
        Ok(outcome)
    }
}

/// Limit the number of bytes that can be read out of a reader to the specified limit.
///
/// Constructed through [Reader::limit].
pub struct Limit<R> {
    remaining: usize,
    reader: R,
}

impl<'de, R> Limit<R>
where
    R: Reader<'de>,
{
    fn bounds_check<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = R::Error>,
    {
        match self.remaining.checked_sub(n) {
            Some(remaining) => {
                self.remaining = remaining;
                Ok(())
            }
            None => Err(cx.custom("out of bounds")),
        }
    }
}

impl<'de, R> Reader<'de> for Limit<R>
where
    R: Reader<'de>,
{
    type Error = R::Error;

    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.bounds_check(cx, n)?;
        self.reader.skip(cx, n)
    }

    #[inline]
    fn read_bytes<'buf, C, V>(
        &mut self,
        cx: &mut C,
        n: usize,
        visitor: V,
    ) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, [u8]>,
    {
        self.bounds_check(cx, n)?;
        self.reader.read_bytes(cx, n, visitor)
    }

    #[inline]
    fn peek<'buf, C>(&mut self, cx: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.reader.peek(cx)
    }

    #[inline]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.bounds_check(cx, buf.len())?;
        self.reader.read(cx, buf)
    }

    #[inline]
    fn read_byte<'buf, C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.bounds_check(cx, 1)?;
        self.reader.read_byte(cx)
    }

    #[inline]
    fn read_array<'buf, C, const N: usize>(&mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.bounds_check(cx, N)?;
        self.reader.read_array(cx)
    }
}

// Forward implementations.

impl<'de, R> Reader<'de> for &mut R
where
    R: ?Sized + Reader<'de>,
{
    type Error = R::Error;

    type Mut<'this> = &'this mut R where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<'buf, C>(&mut self, cx: &mut C, n: usize) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (**self).skip(cx, n)
    }

    #[inline]
    fn read_bytes<'buf, C, V>(
        &mut self,
        cx: &mut C,
        n: usize,
        visitor: V,
    ) -> Result<V::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
        V: ValueVisitor<'de, 'buf, C, [u8]>,
    {
        (**self).read_bytes(cx, n, visitor)
    }

    #[inline]
    fn peek<'buf, C>(&mut self, cx: &mut C) -> Result<Option<u8>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (**self).peek(cx)
    }

    #[inline]
    fn read<'buf, C>(&mut self, cx: &mut C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (**self).read(cx, buf)
    }

    #[inline]
    fn read_byte<'buf, C>(&mut self, cx: &mut C) -> Result<u8, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (**self).read_byte(cx)
    }

    #[inline]
    fn read_array<'buf, C, const N: usize>(&mut self, cx: &mut C) -> Result<[u8; N], C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        (**self).read_array(cx)
    }
}
