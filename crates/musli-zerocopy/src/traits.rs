use core::mem;
use core::str;

use crate::buf::Buf;
use crate::error::Error;
#[cfg(feature = "build")]
use crate::owned_buf::OwnedBuf;
use crate::ptr::Ptr;
use crate::UnsizedRef;

/// Trait for value which can be read.
pub trait Bind<'a> {
    /// The output of the value being read.
    type Output;

    /// Read at the given buffer.
    fn bind(self, buf: &'a Buf) -> Result<Self::Output, Error>;
}

pub trait Size {
    /// The size of an item.
    fn size() -> usize;
}

/// Trait to coerce a buffer from a pointer.
pub trait Read<'a>: Sized {
    /// Coerce a slice into a reference of the current archived type.
    fn read(buf: &'a Buf, ptr: Ptr) -> Result<Self, Error>;
}

/// The trait for an archived type.
#[cfg(feature = "build")]
pub trait Write {
    /// Write to the given buffer.
    fn write(&self, buf: &mut OwnedBuf);
}

/// The trait for an archived type.
#[cfg(feature = "build")]
pub trait UnsizedToBuf: Write {
    /// Get the size of the unsized value.
    fn len(&self) -> usize;
}

macro_rules! impl_tuple {
    ($($ty:ident),*) => {
        impl<$($ty,)*> Size for ($($ty,)*)
        where
            $($ty: Size),*
        {
            #[inline]
            fn size() -> usize {
                0 $(+ $ty::size())*
            }
        }

        impl<'a, $($ty,)*> Read<'a> for ($($ty,)*)
        where
            $($ty: Size + Read<'a>),*
        {
            #[inline]
            #[allow(unused_assignments, non_snake_case)]
            fn read(buf: &'a Buf, ptr: Ptr) -> Result<Self, Error> {
                let mut ptr = ptr;

                $(
                    let $ty = $ty::read(buf, ptr)?;
                    ptr = ptr.wrapping_add($ty::size());
                )*

                Ok(($($ty,)*))
            }
        }

        #[cfg(feature = "build")]
        impl<$($ty,)*> Write for ($($ty,)*)
        where
            $($ty: Write),*
        {
            #[inline]
            #[allow(non_snake_case)]
            fn write(&self, buf: &mut OwnedBuf) {
                let ($($ty,)*) = self;
                $($ty.write(buf);)*
            }
        }

        impl<'a, $($ty,)*> Bind<'a> for ($($ty,)*) where $($ty: Bind<'a>,)* {
            type Output = ($($ty::Output,)*);

            #[inline]
            #[allow(non_snake_case)]
            fn bind(self, buf: &'a Buf) -> Result<Self::Output, Error> {
                let ($($ty,)*) = self;
                Ok(($($ty.bind(buf)?,)*))
            }
        }
    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);

macro_rules! impl_integer {
    ($ty:ty) => {
        impl Size for $ty {
            #[inline]
            fn size() -> usize {
                mem::size_of::<$ty>()
            }
        }

        impl Read<'_> for $ty {
            #[inline]
            fn read(buf: &Buf, ptr: Ptr) -> Result<Self, Error> {
                Ok(<$ty>::from_ne_bytes(
                    buf.get_array(ptr, mem::size_of::<$ty>())?,
                ))
            }
        }

        #[cfg(feature = "build")]
        impl Write for $ty {
            #[inline]
            fn write(&self, buf: &mut OwnedBuf) {
                let bytes = self.to_ne_bytes();
                buf.extend_from_slice(&bytes);
            }
        }

        impl<'a> Bind<'a> for $ty {
            type Output = $ty;

            #[inline]
            fn bind(self, _: &'a Buf) -> Result<Self::Output, Error> {
                Ok(self)
            }
        }
    };
}

impl_integer!(usize);
impl_integer!(u8);
impl_integer!(u16);
impl_integer!(u32);
impl_integer!(u64);
impl_integer!(u128);

impl_integer!(isize);
impl_integer!(i8);
impl_integer!(i16);
impl_integer!(i32);
impl_integer!(i64);
impl_integer!(i128);

impl Size for &str {
    #[inline]
    fn size() -> usize {
        UnsizedRef::<str>::size()
    }
}

impl<'a> Read<'a> for &'a str {
    #[inline]
    fn read(buf: &'a Buf, ptr: Ptr) -> Result<Self, Error> {
        UnsizedRef::<str>::read(buf, ptr)?.bind(buf)
    }
}

impl Size for &[u8] {
    #[inline]
    fn size() -> usize {
        UnsizedRef::<[u8]>::size()
    }
}

impl<'a> Read<'a> for &'a [u8] {
    #[inline]
    fn read(buf: &'a Buf, ptr: Ptr) -> Result<Self, Error> {
        let value = UnsizedRef::<[u8]>::read(buf, ptr)?;
        value.bind(buf)
    }
}

#[cfg(feature = "build")]
impl Write for str {
    #[inline]
    fn write(&self, buf: &mut OwnedBuf) {
        self.as_bytes().write(buf);
    }
}

#[cfg(feature = "build")]
impl UnsizedToBuf for str {
    #[inline]
    fn len(&self) -> usize {
        <str>::len(self)
    }
}

#[cfg(feature = "build")]
impl Write for [u8] {
    #[inline]
    fn write(&self, buf: &mut OwnedBuf) {
        buf.extend_from_slice(self);
    }
}

#[cfg(feature = "build")]
impl UnsizedToBuf for [u8] {
    #[inline]
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}
