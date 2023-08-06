use core::marker::PhantomData;
use core::str;

use crate::buf::Buf;
use crate::error::{Error, ErrorKind};
use crate::owned_buf::OwnedBuf;
use crate::ptr::Ptr;
use crate::traits::{Bind, Read, Size, Write};

/// Reference to an unsized value.
#[derive(Debug)]
pub struct UnsizedRef<T>
where
    T: ?Sized,
{
    pub(crate) ptr: Ptr,
    pub(crate) len: usize,
    _marker: PhantomData<T>,
}

impl<T> UnsizedRef<T>
where
    T: ?Sized,
{
    pub(crate) const fn new(ptr: Ptr, len: usize) -> Self {
        Self {
            ptr,
            len,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for UnsizedRef<T>
where
    T: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for UnsizedRef<T> where T: ?Sized {}

impl<'a, T> Read<'a> for UnsizedRef<T>
where
    T: ?Sized,
{
    #[inline]
    fn read(buf: &'a Buf, base: Ptr) -> Result<Self, Error> {
        let ptr = Ptr::read(buf, base)?;
        let len = base.wrapping_add(Ptr::size());
        let len = usize::read(buf, len)?;

        Ok(UnsizedRef {
            ptr,
            len,
            _marker: PhantomData,
        })
    }
}

impl<T> Size for UnsizedRef<T>
where
    T: ?Sized,
{
    #[inline]
    fn size() -> usize {
        Ptr::size() + usize::size()
    }
}

impl<T> Write for UnsizedRef<T>
where
    T: ?Sized,
{
    fn write(&self, buf: &mut OwnedBuf) {
        self.ptr.write(buf);
        self.len.write(buf);
    }
}

impl<'a> Bind<'a> for UnsizedRef<[u8]> {
    type Output = &'a [u8];

    #[inline]
    fn bind(self, buf: &'a Buf) -> Result<Self::Output, Error> {
        buf.get_slice(self.ptr, self.len)
    }
}

impl<'a> Bind<'a> for UnsizedRef<str> {
    type Output = &'a str;

    #[inline]
    fn bind(self, buf: &'a Buf) -> Result<Self::Output, Error> {
        let data = buf.get_slice(self.ptr, self.len)?;

        let Ok(string) = str::from_utf8(data) else {
            return Err(Error::new(ErrorKind::BadUtf8));
        };

        Ok(string)
    }
}
