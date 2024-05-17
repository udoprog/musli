//! Trait governing how to read bytes.

use core::array;
use core::fmt;
use core::marker;
use core::ops::Range;
use core::ptr;
use core::slice;

use crate::de::UnsizedVisitor;
use crate::Context;

mod sealed {
    use super::{Limit, Reader};

    pub trait Sealed {}

    impl Sealed for &[u8] {}
    impl Sealed for super::SliceReader<'_> {}
    impl<'de, R> Sealed for Limit<R> where R: Reader<'de> {}
    impl<'de, R> Sealed for &mut R where R: ?Sized + Reader<'de> {}
}

/// Trait governing how a source of bytes is read.
///
/// This requires the reader to be able to hand out contiguous references to the
/// byte source through [`Reader::read_bytes`].
pub trait Reader<'de>: self::sealed::Sealed {
    /// Type borrowed from self.
    ///
    /// Why oh why would we want to do this over having a simple `&'this mut T`?
    ///
    /// We want to avoid recursive types, which will blow up the compiler. And
    /// the above is a typical example of when that can go wrong. This ensures
    /// that each call to `borrow_mut` dereferences the [`Reader`] at each step
    /// to avoid constructing a large muted type, like `&mut &mut &mut
    /// SliceReader<'de>`.
    type Mut<'this>: Reader<'de>
    where
        Self: 'this;

    /// Borrow the current reader.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Skip over the given number of bytes.
    fn skip<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context;

    /// Peek the next value.
    fn peek(&mut self) -> Option<u8>;

    /// Read a slice into the given buffer.
    #[inline]
    fn read<C>(&mut self, cx: &C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        struct Visitor<'a>(&'a mut [u8]);

        impl<'a, 'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor<'a>
        where
            C: ?Sized + Context,
        {
            type Ok = ();

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, bytes)
            }

            #[inline]
            fn visit_ref(self, _: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                self.0.copy_from_slice(bytes);
                Ok(())
            }
        }

        self.read_bytes(cx, buf.len(), Visitor(buf))
    }

    /// Read a slice out of the current reader.
    fn read_bytes<C, V>(&mut self, cx: &C, n: usize, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: ?Sized + Context,
        V: UnsizedVisitor<'de, C, [u8]>;

    /// Read a single byte.
    #[inline]
    fn read_byte<C>(&mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: ?Sized + Context,
    {
        let [byte] = self.read_array::<C, 1>(cx)?;
        Ok(byte)
    }

    /// Read an array out of the current reader.
    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: &C) -> Result<[u8; N], C::Error>
    where
        C: ?Sized + Context,
    {
        struct Visitor<const N: usize>([u8; N]);

        impl<'de, const N: usize, C> UnsizedVisitor<'de, C, [u8]> for Visitor<N>
        where
            C: ?Sized + Context,
        {
            type Ok = [u8; N];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, cx: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                self.visit_ref(cx, bytes)
            }

            #[inline]
            fn visit_ref(mut self, cx: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
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

impl<'de> IntoReader<'de> for &'de [u8] {
    type Reader = &'de [u8];

    #[inline]
    fn into_reader(self) -> Self::Reader {
        self
    }
}

impl<'a, 'de, R> IntoReader<'de> for &'a mut R
where
    R: ?Sized + Reader<'de>,
{
    type Reader = &'a mut R;

    #[inline]
    fn into_reader(self) -> Self::Reader {
        self
    }
}

impl<'de> Reader<'de> for &'de [u8] {
    type Mut<'this> = &'this mut &'de [u8] where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        if self.len() < n {
            return Err(cx.message(SliceUnderflow {
                n,
                remaining: self.len(),
            }));
        }

        let (_, tail) = self.split_at(n);
        *self = tail;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read<C>(&mut self, cx: &C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        if self.len() < buf.len() {
            return Err(cx.custom(SliceUnderflow::new(buf.len(), self.len())));
        }

        let (head, tail) = self.split_at(buf.len());
        buf.copy_from_slice(head);
        *self = tail;
        cx.advance(buf.len());
        Ok(())
    }

    #[inline]
    fn read_bytes<C, V>(&mut self, cx: &C, n: usize, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: ?Sized + Context,
        V: UnsizedVisitor<'de, C, [u8]>,
    {
        if self.len() < n {
            return Err(cx.custom(SliceUnderflow::new(n, self.len())));
        }

        let (head, tail) = self.split_at(n);
        *self = tail;
        let ok = visitor.visit_borrowed(cx, head)?;
        cx.advance(n);
        Ok(ok)
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: ?Sized + Context,
    {
        let &[first, ref tail @ ..] = *self else {
            return Err(cx.custom(SliceUnderflow::new(1, self.len())));
        };

        *self = tail;
        cx.advance(1);
        Ok(first)
    }

    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: &C) -> Result<[u8; N], C::Error>
    where
        C: ?Sized + Context,
    {
        if self.len() < N {
            return Err(cx.custom(SliceUnderflow::new(N, self.len())));
        }

        let (head, tail) = self.split_at(N);
        *self = tail;
        cx.advance(N);
        Ok(array::from_fn(|n| head[n]))
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        self.first().copied()
    }
}

/// Coerce a type into a [`Reader`].
pub trait IntoReader<'de>: self::sealed::Sealed {
    /// The reader type.
    type Reader: Reader<'de>;

    /// Convert the type into a reader.
    fn into_reader(self) -> Self::Reader;
}

/// An efficient [`Reader`] wrapper around a slice.
pub struct SliceReader<'de> {
    range: Range<*const u8>,
    _marker: marker::PhantomData<&'de [u8]>,
}

// SAFETY: `SliceReader` is effectively equivalent to `&'de [u8]`.
unsafe impl Send for SliceReader<'_> {}
// SAFETY: `SliceReader` is effectively equivalent to `&'de [u8]`.
unsafe impl Sync for SliceReader<'_> {}

impl<'de> SliceReader<'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            range: slice.as_ptr_range(),
            _marker: marker::PhantomData,
        }
    }

    /// Get the remaining contents of the reader as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::reader::{Reader, SliceReader};
    ///
    /// fn process<C>(cx: &C) -> Result<(), C::Error>
    /// where
    ///     C: ?Sized + Context
    /// {
    ///     let mut reader = SliceReader::new(&[1, 2, 3, 4]);
    ///     assert_eq!(reader.as_slice(), &[1, 2, 3, 4]);
    ///     reader.skip(cx, 2)?;
    ///     assert_eq!(reader.as_slice(), &[3, 4]);
    ///     Ok(())
    /// }
    /// ```
    pub fn as_slice(&self) -> &'de [u8] {
        unsafe { slice::from_raw_parts(self.range.start, self.remaining()) }
    }

    /// Get remaining bytes in the reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::reader::{Reader, SliceReader};
    ///
    /// fn process<C>(cx: &C) -> Result<(), C::Error>
    /// where
    ///     C: ?Sized + Context
    /// {
    ///     let mut reader = SliceReader::new(&[1, 2, 3, 4]);
    ///     assert_eq!(reader.remaining(), 4);
    ///     reader.skip(cx, 2);
    ///     assert_eq!(reader.remaining(), 2);
    ///     Ok(())
    /// }
    /// ```
    pub fn remaining(&self) -> usize {
        self.range.end as usize - self.range.start as usize
    }
}

impl<'de> Reader<'de> for SliceReader<'de> {
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.range.start = bounds_check_add(cx, &self.range, n)?;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read_bytes<C, V>(&mut self, cx: &C, n: usize, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: ?Sized + Context,
        V: UnsizedVisitor<'de, C, [u8]>,
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
    fn peek(&mut self) -> Option<u8> {
        if self.range.start == self.range.end {
            return None;
        }

        // SAFETY: we've checked that the elements are in bound above.
        unsafe { Some(ptr::read(self.range.start)) }
    }

    #[inline]
    fn read<C>(&mut self, cx: &C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
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
fn bounds_check_add<C>(cx: &C, range: &Range<*const u8>, len: usize) -> Result<*const u8, C::Error>
where
    C: ?Sized + Context,
{
    let outcome = range.start.wrapping_add(len);

    if outcome > range.end || outcome < range.start {
        Err(cx.message(SliceUnderflow {
            n: len,
            remaining: (range.end as usize).wrapping_sub(range.start as usize),
        }))
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

impl<R> Limit<R> {
    /// Get the remaining data in the limited reader.
    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

impl<'de, R> Limit<R>
where
    R: Reader<'de>,
{
    fn bounds_check<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        match self.remaining.checked_sub(n) {
            Some(remaining) => {
                self.remaining = remaining;
                Ok(())
            }
            None => Err(cx.message("Reader out of bounds")),
        }
    }
}

impl<'de, R> Reader<'de> for Limit<R>
where
    R: Reader<'de>,
{
    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.bounds_check(cx, n)?;
        self.reader.skip(cx, n)
    }

    #[inline]
    fn read_bytes<C, V>(&mut self, cx: &C, n: usize, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: ?Sized + Context,
        V: UnsizedVisitor<'de, C, [u8]>,
    {
        self.bounds_check(cx, n)?;
        self.reader.read_bytes(cx, n, visitor)
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        if self.remaining > 0 {
            self.reader.peek()
        } else {
            None
        }
    }

    #[inline]
    fn read<C>(&mut self, cx: &C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        self.bounds_check(cx, buf.len())?;
        self.reader.read(cx, buf)
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: ?Sized + Context,
    {
        self.bounds_check(cx, 1)?;
        self.reader.read_byte(cx)
    }

    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: &C) -> Result<[u8; N], C::Error>
    where
        C: ?Sized + Context,
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
    type Mut<'this> = &'this mut R where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        (**self).skip(cx, n)
    }

    #[inline]
    fn read_bytes<C, V>(&mut self, cx: &C, n: usize, visitor: V) -> Result<V::Ok, C::Error>
    where
        C: ?Sized + Context,
        V: UnsizedVisitor<'de, C, [u8]>,
    {
        (**self).read_bytes(cx, n, visitor)
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        (**self).peek()
    }

    #[inline]
    fn read<C>(&mut self, cx: &C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        (**self).read(cx, buf)
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: ?Sized + Context,
    {
        (**self).read_byte(cx)
    }

    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: &C) -> Result<[u8; N], C::Error>
    where
        C: ?Sized + Context,
    {
        (**self).read_array(cx)
    }
}

/// Underflow when trying to read from a slice.
#[derive(Debug)]
pub(crate) struct SliceUnderflow {
    n: usize,
    remaining: usize,
}

impl SliceUnderflow {
    pub(crate) fn new(n: usize, remaining: usize) -> Self {
        Self { n, remaining }
    }
}

impl fmt::Display for SliceUnderflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SliceUnderflow { n, remaining } = self;

        write!(
            f,
            "Tried to read {n} bytes from slice, with {remaining} byte remaining"
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SliceUnderflow {}
