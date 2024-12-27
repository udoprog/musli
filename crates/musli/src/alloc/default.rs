use core::marker::PhantomData;

#[cfg(not(feature = "alloc"))]
use super::Slice;
#[cfg(feature = "alloc")]
use super::System;
use super::{Allocator, RawVec};

/// The default stack buffer size for the default allocator provided through
/// [`default!`].
pub const DEFAULT_ARRAY_BUFFER: usize = 4096;

macro_rules! implement {
    ($id:ident, $ty:ty) => {
        /// Alias for the default allocator implementation.
        ///
        /// The exact type of this differs depending on whether the `alloc` feature is
        /// enabled.
        #[repr(transparent)]
        pub struct $id<'buf, const BUF: usize> {
            inner: $ty,
            _marker: PhantomData<&'buf mut [u8]>,
        }

        /// Alias for the default raw vector implementation.
        ///
        /// The exact type of this differs depending on whether the `alloc` feature is
        /// enabled.
        pub struct DefaultRawVec<'a, 'buf: 'a, T, const BUF: usize>
        where
            T: 'a,
        {
            inner: <$ty as Allocator>::RawVec<'a, T>,
            _marker: PhantomData<&'buf mut [u8]>,
        }

        impl<'buf, const BUF: usize> $id<'buf, BUF> {
            #[inline]
            #[cfg_attr(feature = "alloc", allow(clippy::needless_lifetimes))]
            pub(super) fn new<'a>(alloc: &'a $ty) -> &'a Self {
                // SAFETY: The type is repr(transparent) over the interior value.
                unsafe { &*(alloc as *const $ty).cast::<Self>() }
            }
        }
    };
}

#[cfg(feature = "alloc")]
implement!(DefaultAllocator, System);

#[cfg(not(feature = "alloc"))]
implement!(DefaultAllocator, Slice<'buf>);

impl<'buf, const BUF: usize> Allocator for DefaultAllocator<'buf, BUF> {
    type RawVec<'this, T>
        = DefaultRawVec<'this, 'buf, T, BUF>
    where
        Self: 'this,
        T: 'this;

    #[inline]
    fn new_raw_vec<'a, T>(&'a self) -> Self::RawVec<'a, T>
    where
        T: 'a,
    {
        DefaultRawVec {
            inner: self.inner.new_raw_vec(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, const BUF: usize> RawVec<T> for DefaultRawVec<'a, '_, T, BUF>
where
    T: 'a,
{
    #[inline]
    fn resize(&mut self, len: usize, additional: usize) -> bool {
        self.inner.resize(len, additional)
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    #[inline]
    fn try_merge<B>(&mut self, this_len: usize, other: B, other_len: usize) -> Result<(), B>
    where
        B: RawVec<T>,
    {
        self.inner.try_merge(this_len, other, other_len)
    }
}
