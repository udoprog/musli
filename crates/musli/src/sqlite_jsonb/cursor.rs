//! Traits governing how a JSONB document is read.
//!
//! JSONB is a random access format where every container knows the exact size
//! of its payload, so unlike the streaming formats it is decoded from a byte
//! slice rather than through [`Reader`].
//!
//! [`Reader`]: crate::Reader

use core::fmt;

use crate::Context;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::SliceCursor<'_> {}
    impl Sealed for super::MutSliceCursor<'_, '_> {}
    impl<'de, C> Sealed for &mut C where C: ?Sized + super::Cursor<'de> {}
}

/// Trait governing how a JSONB document is read.
///
/// This is implemented for [`SliceCursor`] and [`MutSliceCursor`], which are
/// constructed through [`IntoCursor`].
pub trait Cursor<'de>: self::sealed::Sealed {
    /// Reborrowed type.
    ///
    /// Just like for [`Reader`] and [`Parser`] this ensures that each call to
    /// [`Cursor::borrow_mut`] dereferences the cursor instead of constructing a
    /// deeply nested `&mut &mut &mut SliceCursor<'de>`, which would blow up the
    /// compiler.
    ///
    /// [`Reader`]: crate::Reader
    /// [`Parser`]: crate::json::Parser
    type Mut<'this>: Cursor<'de>
    where
        Self: 'this;

    /// The type this cursor can be cloned into.
    type TryClone: Cursor<'de>;

    /// Reborrow the current cursor.
    fn borrow_mut(&mut self) -> Self::Mut<'_>;

    /// Try to clone the cursor so that the same input can be decoded twice.
    fn try_clone(&self) -> Option<Self::TryClone>;

    /// The number of bytes remaining in the input.
    #[doc(hidden)]
    fn remaining(&self) -> usize;

    /// Peek at the next byte without consuming it.
    #[doc(hidden)]
    fn peek(&self) -> Option<u8>;

    /// Read a single byte.
    #[doc(hidden)]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context;

    /// Read exactly `n` bytes, which are borrowed out of the underlying input.
    #[doc(hidden)]
    fn read_slice<C>(&mut self, cx: C, n: usize) -> Result<&'de [u8], C::Error>
    where
        C: Context;

    /// Skip over `n` bytes.
    #[doc(hidden)]
    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        self.read_slice(cx, n)?;
        Ok(())
    }

    /// Test if the cursor has been fully consumed.
    #[doc(hidden)]
    #[inline]
    fn is_exhausted<C>(&mut self, _: C) -> bool
    where
        C: Context,
    {
        self.remaining() == 0
    }
}

/// A cursor over a borrowed byte slice.
#[derive(Clone)]
pub struct SliceCursor<'de> {
    data: &'de [u8],
}

impl<'de> SliceCursor<'de> {
    /// Construct a new cursor over `data`.
    #[inline]
    pub(crate) fn new(data: &'de [u8]) -> Self {
        Self { data }
    }
}

impl<'de> Cursor<'de> for SliceCursor<'de> {
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    type TryClone = SliceCursor<'de>;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(SliceCursor { data: self.data })
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.data.first().copied()
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        let [first, tail @ ..] = self.data else {
            return Err(cx.message(Underflow { n: 1, remaining: 0 }));
        };

        self.data = tail;
        cx.advance(1);
        Ok(*first)
    }

    #[inline]
    fn read_slice<C>(&mut self, cx: C, n: usize) -> Result<&'de [u8], C::Error>
    where
        C: Context,
    {
        if self.data.len() < n {
            return Err(cx.message(Underflow {
                n,
                remaining: self.data.len(),
            }));
        }

        let (head, tail) = self.data.split_at(n);
        self.data = tail;
        cx.advance(n);
        Ok(head)
    }
}

/// A cursor which advances the slice it is constructed over, so that any input
/// trailing the decoded element remains available to the caller.
pub struct MutSliceCursor<'a, 'de> {
    data: &'a mut &'de [u8],
}

impl<'a, 'de> MutSliceCursor<'a, 'de> {
    #[inline]
    pub(crate) fn new(data: &'a mut &'de [u8]) -> Self {
        Self { data }
    }
}

impl<'de> Cursor<'de> for MutSliceCursor<'_, 'de> {
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    type TryClone = SliceCursor<'de>;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        Some(SliceCursor { data: self.data })
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.data.first().copied()
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        let [first, tail @ ..] = *self.data else {
            return Err(cx.message(Underflow { n: 1, remaining: 0 }));
        };

        *self.data = tail;
        cx.advance(1);
        Ok(*first)
    }

    #[inline]
    fn read_slice<C>(&mut self, cx: C, n: usize) -> Result<&'de [u8], C::Error>
    where
        C: Context,
    {
        if self.data.len() < n {
            return Err(cx.message(Underflow {
                n,
                remaining: self.data.len(),
            }));
        }

        let (head, tail) = self.data.split_at(n);
        *self.data = tail;
        cx.advance(n);
        Ok(head)
    }
}

impl<'de, P> Cursor<'de> for &mut P
where
    P: ?Sized + Cursor<'de>,
{
    type Mut<'this>
        = P::Mut<'this>
    where
        Self: 'this;

    type TryClone = P::TryClone;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        (**self).borrow_mut()
    }

    #[inline]
    fn try_clone(&self) -> Option<Self::TryClone> {
        (**self).try_clone()
    }

    #[inline]
    fn remaining(&self) -> usize {
        (**self).remaining()
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        (**self).peek()
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: C) -> Result<u8, C::Error>
    where
        C: Context,
    {
        (**self).read_byte(cx)
    }

    #[inline]
    fn read_slice<C>(&mut self, cx: C, n: usize) -> Result<&'de [u8], C::Error>
    where
        C: Context,
    {
        (**self).read_slice(cx, n)
    }

    #[inline]
    fn skip<C>(&mut self, cx: C, n: usize) -> Result<(), C::Error>
    where
        C: Context,
    {
        (**self).skip(cx, n)
    }

    #[inline]
    fn is_exhausted<C>(&mut self, cx: C) -> bool
    where
        C: Context,
    {
        (**self).is_exhausted(cx)
    }
}

mod into_sealed {
    pub trait Sealed {}
    impl Sealed for &[u8] {}
    impl Sealed for &str {}
    impl Sealed for &mut &[u8] {}
}

/// Trait for types which can be converted into a [`Cursor`].
pub trait IntoCursor<'de>: self::into_sealed::Sealed {
    /// The cursor type being converted into.
    type Cursor: Cursor<'de>;

    /// Convert into a cursor.
    fn into_cursor(self) -> Self::Cursor;
}

impl<'de> IntoCursor<'de> for &'de [u8] {
    type Cursor = SliceCursor<'de>;

    #[inline]
    fn into_cursor(self) -> Self::Cursor {
        SliceCursor::new(self)
    }
}

impl<'de> IntoCursor<'de> for &'de str {
    type Cursor = SliceCursor<'de>;

    #[inline]
    fn into_cursor(self) -> Self::Cursor {
        SliceCursor::new(self.as_bytes())
    }
}

impl<'a, 'de> IntoCursor<'de> for &'a mut &'de [u8] {
    type Cursor = MutSliceCursor<'a, 'de>;

    #[inline]
    fn into_cursor(self) -> Self::Cursor {
        MutSliceCursor::new(self)
    }
}

struct Underflow {
    n: usize,
    remaining: usize,
}

impl fmt::Display for Underflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Underflow { n, remaining } = self;

        write!(
            f,
            "Tried to read {n} bytes from JSONB input, with {remaining} bytes remaining"
        )
    }
}
