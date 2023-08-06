use crate::ptr::Ptr;
use crate::traits::{Bind, Read, Size, Write};
use crate::{Buf, Error, OwnedBuf, UnsizedRef};

/// A zero-copy accessor for a slice inside of a buffer.
#[derive(Debug)]
pub struct Slice<'a, T> {
    repr: UnsizedRef<[T]>,
    buf: &'a Buf,
}

impl<'a, T> Slice<'a, T> {
    pub(crate) fn new(repr: UnsizedRef<[T]>, buf: &'a Buf) -> Self {
        Self { repr, buf }
    }

    /// Get the length of the slice.
    #[inline]
    pub fn len(&self) -> usize {
        self.repr.len
    }

    /// Check if the slice is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.repr.len == 0
    }

    /// Get a value at the given index.
    pub fn get(&self, index: usize) -> Result<Option<T>, Error>
    where
        T: Size + Read<'a>,
    {
        if index >= self.repr.len {
            return Ok(None);
        }

        let ptr = self.repr.ptr.wrapping_add(index.wrapping_mul(T::size()));
        Ok(Some(T::read(self.buf, ptr)?))
    }
}

impl<'a, T> Read<'a> for Slice<'a, T> {
    #[inline]
    fn read(buf: &'a Buf, ptr: Ptr) -> Result<Self, Error> {
        Ok(Self {
            repr: UnsizedRef::read(buf, ptr)?,
            buf,
        })
    }
}

impl<T> Size for Slice<'_, T> {
    #[inline]
    fn size() -> usize {
        UnsizedRef::<[T]>::size()
    }
}

/// A typed reference to a slice.
pub struct SliceRef<T> {
    repr: UnsizedRef<[T]>,
}

impl<T> SliceRef<T> {
    #[inline]
    pub(crate) fn new(repr: UnsizedRef<[T]>) -> Self {
        Self { repr }
    }
}

impl<T> Size for SliceRef<T> {
    #[inline]
    fn size() -> usize {
        UnsizedRef::<[T]>::size()
    }
}

impl<T> Write for SliceRef<T> {
    #[inline]
    fn write(&self, buf: &mut OwnedBuf) {
        self.repr.write(buf);
    }
}

impl<'a, T: 'a> Bind<'a> for SliceRef<T>
where
    T: Bind<'a>,
{
    type Output = Slice<'a, T::Output>;

    #[inline]
    fn bind(self, buf: &'a Buf) -> Result<Self::Output, Error> {
        Ok(Slice {
            repr: UnsizedRef::new(self.repr.ptr, self.repr.len),
            buf,
        })
    }
}
