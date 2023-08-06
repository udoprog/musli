use core::marker::PhantomData;

use crate::buf::Buf;
use crate::error::Error;
use crate::owned_buf::OwnedBuf;
use crate::ptr::Ptr;
use crate::traits::{Bind, Read, Size, Write};

/// Reference to a value stored in a buffer.
pub struct ValueRef<T> {
    ptr: Ptr,
    _marker: PhantomData<T>,
}

impl<T> ValueRef<T> {
    pub(crate) fn new(ptr: Ptr) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Access the underlying pointer for the value.
    #[inline]
    pub fn as_ptr(&self) -> Ptr {
        self.ptr
    }
}

impl<T> Size for ValueRef<T> {
    #[inline]
    fn size() -> usize {
        Ptr::size()
    }
}

impl<T> Write for ValueRef<T> {
    #[inline]
    fn write(&self, buf: &mut OwnedBuf) {
        self.ptr.write(buf);
    }
}

impl<'a, T> Read<'a> for ValueRef<T> {
    fn read(buf: &'a Buf, ptr: Ptr) -> Result<Self, Error> {
        Ok(ValueRef {
            ptr: Ptr::read(buf, ptr)?,
            _marker: PhantomData,
        })
    }
}

impl<'a, T> Bind<'a> for ValueRef<T>
where
    T: Read<'a>,
{
    type Output = T;

    #[inline]
    fn bind(self, buf: &'a Buf) -> Result<Self::Output, Error> {
        T::read(buf, self.ptr)
    }
}
