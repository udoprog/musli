//! Trait governing how to read bytes.

use core::fmt;
use core::marker;
use core::mem::MaybeUninit;
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

/// Coerce a type into a [`Reader`].
pub trait IntoReader<'de>: self::sealed::Sealed {
    /// The reader type.
    type Reader: Reader<'de>;

    /// Convert the type into a reader.
    fn into_reader(self) -> Self::Reader;
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

    /// Type that can be cloned from the reader.
    type TryClone: Reader<'de>;

    /// Borrow the current reader.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Try to clone the reader.
    fn try_clone(&self) -> Option<Self::TryClone>;

    /// Test if the reader is at end of input.
    fn is_eof(&mut self) -> bool;

    /// Skip over the given number of bytes.
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context;

    /// Peek the next value.
    fn peek(&mut self) -> Option<u8>;

    /// Read a slice into the given buffer.
    #[inline]
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        struct Visitor<'a>(&'a mut [u8]);

        #[crate::de::unsized_visitor(crate)]
        impl<C> UnsizedVisitor<'_, C, [u8]> for Visitor<'_>
        where
            C: Context,
        {
            type Ok = ();

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_ref(self, _: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                self.0.copy_from_slice(bytes);
                Ok(())
            }
        }

        self.read_bytes(cx, buf.len(), Visitor(buf))
    }

    /// Read a slice out of the current reader.
    fn read_bytes<C, V>(&mut self, cx: C, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, [u8], Error = C::Error, Allocator = C::Allocator>;

    /// Read into the given buffer which might not have been initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer points to valid memory of length
    /// `len`.
    unsafe fn read_bytes_uninit<C>(
        &mut self,
        cx: C,
        ptr: *mut u8,
        len: usize,
    ) -> Result<(), C::Error>
    where
        C: Context;

    /// Read a single byte.
    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        let [byte] = self.read_array::<C, 1>(cx)?;
        Ok(byte)
    }

    /// Read an array out of the current reader.
    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: C) -> Result<[u8; N], C::Error>
    where
        C: Context,
    {
        struct Visitor<const N: usize>([u8; N]);

        #[crate::de::unsized_visitor(crate)]
        impl<const N: usize, C> UnsizedVisitor<'_, C, [u8]> for Visitor<N>
        where
            C: Context,
        {
            type Ok = [u8; N];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_ref(mut self, cx: C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
                self.0.copy_from_slice(bytes);
                cx.advance(bytes.len());
                Ok(self.0)
            }
        }

        self.read_bytes(cx, N, Visitor([0u8; N]))
    }

    /// Keep an accurate record of the position within the reader.
    #[inline]
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

impl<'de> Reader<'de> for &'de [u8] {
    type Mut<'this>
        = &'this mut &'de [u8]
    where
        Self: 'this;

    type TryClone = &'de [u8];

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(self)
    }

    #[inline]
    fn is_eof(&mut self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
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
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
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
    fn read_bytes<C, V>(&mut self, cx: C, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, [u8], Error = C::Error, Allocator = C::Allocator>,
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
    unsafe fn read_bytes_uninit<C>(&mut self, cx: C, ptr: *mut u8, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        if self.len() < n {
            return Err(cx.custom(SliceUnderflow::new(n, self.len())));
        }

        ptr.copy_from_nonoverlapping(self.as_ptr(), n);
        *self = &self[n..];
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        let &[first, ref tail @ ..] = *self else {
            return Err(cx.custom(SliceUnderflow::new(1, self.len())));
        };

        *self = tail;
        cx.advance(1);
        Ok(first)
    }

    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: C) -> Result<[u8; N], C::Error>
    where
        C: Context,
    {
        if self.len() < N {
            return Err(cx.custom(SliceUnderflow::new(N, self.len())));
        }

        cx.advance(N);

        let mut array: MaybeUninit<[u8; N]> = MaybeUninit::uninit();

        // SAFETY: We've checked the length of the current buffer just above.
        // PERFORMANCE: This generates better code than `array::from_fn`, and
        // `read_array` is performance sensitive.
        unsafe {
            array
                .as_mut_ptr()
                .cast::<u8>()
                .copy_from_nonoverlapping(self.as_ptr(), N);
            *self = self.get_unchecked(N..);
            Ok(array.assume_init())
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        self.first().copied()
    }
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
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::reader::SliceReader;
    ///
    /// let data = &[1, 2, 3, 4];
    /// let reader = SliceReader::new(data);
    /// assert_eq!(reader.as_slice(), &[1, 2, 3, 4]);
    /// ```
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
    /// fn process<C>(cx: C) -> Result<(), C::Error>
    /// where
    ///     C: Context
    /// {
    ///     let mut reader = SliceReader::new(&[1, 2, 3, 4]);
    ///     assert_eq!(reader.as_slice(), &[1, 2, 3, 4]);
    ///     reader.skip(cx, 2)?;
    ///     assert_eq!(reader.as_slice(), &[3, 4]);
    ///     Ok(())
    /// }
    /// ```
    #[inline]
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
    /// fn process<C>(cx: C) -> Result<(), C::Error>
    /// where
    ///     C: Context
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
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    type TryClone = Self;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(Self {
            range: self.range.clone(),
            _marker: marker::PhantomData,
        })
    }

    #[inline]
    fn is_eof(&mut self) -> bool {
        self.range.start == self.range.end
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.range.start = bounds_check_add(cx, &self.range, n)?;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read_bytes<C, V>(&mut self, cx: C, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, [u8], Error = C::Error, Allocator = C::Allocator>,
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
    unsafe fn read_bytes_uninit<C>(&mut self, cx: C, ptr: *mut u8, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        let outcome = bounds_check_add(cx, &self.range, n)?;
        ptr.copy_from_nonoverlapping(self.range.start, n);
        self.range.start = outcome;
        cx.advance(n);
        Ok(())
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
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        let outcome = bounds_check_add(cx, &self.range, buf.len())?;

        // SAFETY: We've checked that the updated pointer is in bounds.
        unsafe {
            self.range
                .start
                .copy_to_nonoverlapping(buf.as_mut_ptr(), buf.len());
            self.range.start = outcome;
        }

        cx.advance(buf.len());
        Ok(())
    }
}

#[inline]
fn bounds_check_add<C>(cx: C, range: &Range<*const u8>, len: usize) -> Result<*const u8, C::Error>
where
    C: Context,
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
/// This type wraps another reader and ensures that no more than a specified number
/// of bytes can be read from it. This is useful for implementing bounded reads
/// in serialization contexts.
///
/// Constructed through [Reader::limit].
///
/// # Examples
///
/// ```
/// use musli::Context;
/// use musli::reader::{Reader, SliceReader};
///
/// let cx = musli::context::new();
/// let data = &[1, 2, 3, 4, 5];
/// let mut reader = SliceReader::new(data);
/// let mut limited = reader.limit(3);
///
/// // Can read from the limited reader
/// let byte = limited.read_byte(&cx)?;
/// assert_eq!(byte, 1);
/// assert_eq!(limited.remaining(), 2);
///
/// // Read two more bytes
/// limited.read_byte(&cx)?;
/// limited.read_byte(&cx)?;
/// assert_eq!(limited.remaining(), 0);
/// # Ok::<_, musli::context::ErrorMarker>(())
/// ```
pub struct Limit<R> {
    remaining: usize,
    reader: R,
}

impl<R> Limit<R> {
    /// Get the remaining data in the limited reader.
    ///
    /// Returns the number of bytes that can still be read from this limited reader
    /// before the limit is reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::reader::{Reader, SliceReader};
    ///
    /// let data = &[1, 2, 3, 4, 5];
    /// let mut reader = SliceReader::new(data);
    /// let limited = reader.limit(3);
    ///
    /// assert_eq!(limited.remaining(), 3);
    /// ```
    #[inline]
    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

impl<'de, R> Limit<R>
where
    R: Reader<'de>,
{
    #[inline]
    fn bounds_check<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
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
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    type TryClone = Limit<R::TryClone>;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(Limit {
            remaining: self.remaining,
            reader: self.reader.try_clone()?,
        })
    }

    #[inline]
    fn is_eof(&mut self) -> bool {
        self.remaining == 0 || self.reader.is_eof()
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.bounds_check(cx, n)?;
        self.reader.skip(cx, n)
    }

    #[inline]
    fn read_bytes<C, V>(&mut self, cx: C, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, [u8], Error = C::Error, Allocator = C::Allocator>,
    {
        self.bounds_check(cx, n)?;
        self.reader.read_bytes(cx, n, visitor)
    }

    #[inline]
    unsafe fn read_bytes_uninit<C>(&mut self, cx: C, ptr: *mut u8, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.bounds_check(cx, n)?;
        self.reader.read_bytes_uninit(cx, ptr, n)
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
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.bounds_check(cx, buf.len())?;
        self.reader.read(cx, buf)
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        self.bounds_check(cx, 1)?;
        self.reader.read_byte(cx)
    }

    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: C) -> Result<[u8; N], C::Error>
    where
        C: Context,
    {
        self.bounds_check(cx, N)?;
        self.reader.read_array(cx)
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

impl<'de, R> Reader<'de> for &mut R
where
    R: ?Sized + Reader<'de>,
{
    type Mut<'this>
        = &'this mut R
    where
        Self: 'this;

    type TryClone = R::TryClone;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        (**self).try_clone()
    }

    #[inline]
    fn is_eof(&mut self) -> bool {
        (**self).is_eof()
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).skip(cx, n)
    }

    #[inline]
    fn read_bytes<C, V>(&mut self, cx: C, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        C: Context,
        V: UnsizedVisitor<'de, C, [u8], Error = C::Error, Allocator = C::Allocator>,
    {
        (**self).read_bytes(cx, n, visitor)
    }

    #[inline]
    unsafe fn read_bytes_uninit<C>(&mut self, cx: C, ptr: *mut u8, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).read_bytes_uninit(cx, ptr, n)
    }

    #[inline]
    fn peek(&mut self) -> Option<u8> {
        (**self).peek()
    }

    #[inline]
    fn read<C>(&mut self, cx: C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).read(cx, buf)
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        (**self).read_byte(cx)
    }

    #[inline]
    fn read_array<C, const N: usize>(&mut self, cx: C) -> Result<[u8; N], C::Error>
    where
        C: Context,
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
    #[inline]
    pub(crate) fn new(n: usize, remaining: usize) -> Self {
        Self { n, remaining }
    }
}

impl fmt::Display for SliceUnderflow {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SliceUnderflow { n, remaining } = self;

        write!(
            f,
            "Tried to read {n} bytes from slice, with {remaining} byte remaining"
        )
    }
}

impl core::error::Error for SliceUnderflow {}
