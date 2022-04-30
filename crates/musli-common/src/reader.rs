//! Trait for governing how a particular source of bytes is read.
//!
//! `musli` requires all sources to reference the complete data being read from
//! it which allows it to make the assumption the bytes are always returned with
//! the `'de` lifetime.

use core::fmt;
use core::marker;
use core::ops::Range;
use core::ptr;
use core::slice;

use musli::de::ValueVisitor;
use musli::error::Error;

/// A reader where the current position is exactly known.
pub trait PosReader<'de>: Reader<'de> {
    /// The exact position of a reader.
    fn pos(&self) -> usize;

    /// Reborrowed type.
    ///
    /// Why oh why would we want to do this over having a simple `&'this mut T`?
    ///
    /// We want to avoid recursive types, which will blow up the compiler. And
    /// the above is a typical example of when that can go wrong. This ensures
    /// that each call to `borrow_mut` dereferences the [Reader] at each step to
    /// avoid constructing a large muted type, like `&mut &mut &mut
    /// SliceReader<'de>`.
    type PosMut<'this>: PosReader<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Reborrow the current reader.
    fn pos_borrow_mut(&mut self) -> Self::PosMut<'_>;
}

/// Trait governing how a source of bytes is read.
///
/// This requires the reader to be able to hand out contiguous references to the
/// byte source through [Reader::read_bytes].
pub trait Reader<'de> {
    /// Error type raised by the current reader.
    type Error: Error;

    /// Reborrowed type.
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

    /// Reborrow the current reader.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Skip over the given number of bytes.
    fn skip(&mut self, n: usize) -> Result<(), Self::Error>;

    /// Read a slice into the given buffer.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        return self.read_bytes(buf.len(), Visitor::<Self::Error>(buf, marker::PhantomData));

        struct Visitor<'a, E>(&'a mut [u8], marker::PhantomData<E>);

        impl<'a, 'de, E> ValueVisitor<'de> for Visitor<'a, E>
        where
            E: Error,
        {
            type Target = [u8];
            type Ok = ();
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, bytes: &'de Self::Target) -> Result<Self::Ok, Self::Error> {
                self.visit_any(bytes)
            }

            #[inline]
            fn visit_any(self, bytes: &Self::Target) -> Result<Self::Ok, Self::Error> {
                self.0.copy_from_slice(bytes);
                Ok(())
            }
        }
    }

    /// Read a slice out of the current reader.
    fn read_bytes<V>(&mut self, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>;

    /// Read a single byte.
    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let [byte] = self.read_array::<1>()?;
        Ok(byte)
    }

    /// Read an array out of the current reader.
    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        return self.read_bytes(N, Visitor::<N, Self::Error>([0u8; N], marker::PhantomData));

        struct Visitor<const N: usize, E>([u8; N], marker::PhantomData<E>);

        impl<'de, const N: usize, E> ValueVisitor<'de> for Visitor<N, E>
        where
            E: Error,
        {
            type Target = [u8];
            type Ok = [u8; N];
            type Error = E;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "bytes")
            }

            #[inline]
            fn visit_borrowed(self, bytes: &'de Self::Target) -> Result<Self::Ok, Self::Error> {
                self.visit_any(bytes)
            }

            #[inline]
            fn visit_any(mut self, bytes: &Self::Target) -> Result<Self::Ok, Self::Error> {
                self.0.copy_from_slice(bytes);
                Ok(self.0)
            }
        }
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
    fn message<T>(message: T) -> Self
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
    type Mut<'this> = &'this mut &'de [u8] where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
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
    fn read_bytes<V>(&mut self, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        if self.len() < n {
            return Err(SliceReaderError::custom("buffer underflow"));
        }

        let (head, tail) = self.split_at(n);
        *self = tail;
        visitor.visit_borrowed(head)
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

/// An efficient [Reader] wrapper around a slice.
pub struct SliceReader<'de> {
    range: Range<*const u8>,
    _marker: marker::PhantomData<&'de [u8]>,
}

impl<'de> SliceReader<'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            range: slice.as_ptr_range(),
            _marker: marker::PhantomData,
        }
    }
}

impl<'de> Reader<'de> for SliceReader<'de> {
    type Error = SliceReaderError;

    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.range.start = bounds_check_add(&self.range, n)?;
        Ok(())
    }

    #[inline]
    fn read_bytes<V>(&mut self, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let outcome = bounds_check_add(&self.range, n)?;

        unsafe {
            let bytes = slice::from_raw_parts(self.range.start, n);
            self.range.start = outcome;
            visitor.visit_borrowed(bytes)
        }
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let outcome = bounds_check_add(&self.range, buf.len())?;

        unsafe {
            ptr::copy_nonoverlapping(self.range.start, buf.as_mut_ptr(), buf.len());
            self.range.start = outcome;
        }

        Ok(())
    }
}

#[inline]
fn bounds_check_add(range: &Range<*const u8>, len: usize) -> Result<*const u8, SliceReaderError> {
    let outcome = range.start.wrapping_add(len);

    if outcome > range.end || outcome < range.start {
        Err(SliceReaderError::custom("buffer underflow"))
    } else {
        Ok(outcome)
    }
}

/// Keep a record of the current position.
///
/// Constructed through [Reader::with_position].
pub struct WithPosition<R> {
    pos: usize,
    reader: R,
}

impl<'de, R> PosReader<'de> for WithPosition<R>
where
    R: Reader<'de>,
{
    type PosMut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn pos_borrow_mut(&mut self) -> Self::PosMut<'_> {
        self
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }
}

impl<'de, R> Reader<'de> for WithPosition<R>
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
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.reader.skip(n)?;
        self.pos += n;
        Ok(())
    }

    #[inline]
    fn read_bytes<V>(&mut self, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        let ok = self.reader.read_bytes(n, visitor)?;
        self.pos += n;
        Ok(ok)
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
    fn bounds_check(&mut self, n: usize) -> Result<(), R::Error> {
        match self.remaining.checked_sub(n) {
            Some(remaining) => {
                self.remaining = remaining;
                Ok(())
            }
            None => Err(R::Error::custom("out of bounds")),
        }
    }
}

impl<'de, R> PosReader<'de> for Limit<R>
where
    R: PosReader<'de>,
{
    type PosMut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn pos_borrow_mut(&mut self) -> Self::PosMut<'_> {
        self
    }

    #[inline]
    fn pos(&self) -> usize {
        self.reader.pos()
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
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        self.bounds_check(n)?;
        self.reader.skip(n)
    }

    #[inline]
    fn read_bytes<V>(&mut self, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        self.bounds_check(n)?;
        self.reader.read_bytes(n, visitor)
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.bounds_check(buf.len())?;
        self.reader.read(buf)
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        self.bounds_check(1)?;
        self.reader.read_byte()
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Self::Error> {
        self.bounds_check(N)?;
        self.reader.read_array()
    }
}

// Forward implementations.

impl<'de, R> PosReader<'de> for &mut R
where
    R: ?Sized + PosReader<'de>,
{
    type PosMut<'this> = R::PosMut<'this> where Self: 'this;

    #[inline]
    fn pos_borrow_mut(&mut self) -> Self::PosMut<'_> {
        (**self).pos_borrow_mut()
    }

    #[inline]
    fn pos(&self) -> usize {
        (**self).pos()
    }
}

impl<'de, R> Reader<'de> for &mut R
where
    R: ?Sized + Reader<'de>,
{
    type Error = R::Error;

    type Mut<'this> = &'this mut Self where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn skip(&mut self, n: usize) -> Result<(), Self::Error> {
        (**self).skip(n)
    }

    #[inline]
    fn read_bytes<V>(&mut self, n: usize, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        (**self).read_bytes(n, visitor)
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
