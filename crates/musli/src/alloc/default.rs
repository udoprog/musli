use core::marker::PhantomData;

use super::{Allocator, RawVec};
#[cfg(not(feature = "alloc"))]
use super::{Slice, SliceBuf};
#[cfg(feature = "alloc")]
use super::{System, SystemBuf};

/// The default stack buffer size for the default allocator provided through
/// [`default()`].
///
/// [`default()`]: super::default()
pub const DEFAULT_ARRAY_BUFFER: usize = 4096;

macro_rules! implement {
    ($id:ident, $ty:ty, $raw_vec:ty) => {
        /// The default allocator implementation.
        ///
        /// The exact implementation of this depends on if the `alloc` feature
        /// is enabled.
        ///
        /// For more information, see [`default()`].
        ///
        /// [`default()`]: super::default()
        #[repr(transparent)]
        pub struct $id<'buf, const BUF: usize> {
            inner: $ty,
            _marker: PhantomData<&'buf mut [u8]>,
        }

        impl<'buf, const BUF: usize> $id<'buf, BUF> {
            #[inline]
            pub(super) fn new(inner: $ty) -> Self {
                Self {
                    inner,
                    _marker: PhantomData,
                }
            }
        }

        /// The default raw vector allocation.
        ///
        /// The exact implementation of this depends on if the `alloc` feature
        /// is enabled.
        ///
        /// For more information, see [`default()`].
        ///
        /// [`default()`]: super::default()
        pub struct DefaultRawVec<'a, T, const BUF: usize> {
            inner: $raw_vec,
            _marker: PhantomData<&'a ()>,
        }
    };
}

#[cfg(feature = "alloc")]
implement!(DefaultAllocator, System, SystemBuf<T>);

#[cfg(not(feature = "alloc"))]
implement!(DefaultAllocator, Slice<'buf>, SliceBuf<'a, T>);

impl<'a, const BUF: usize> Allocator for &'a DefaultAllocator<'_, BUF> {
    type RawVec<T> = DefaultRawVec<'a, T, BUF>;

    #[inline]
    fn new_raw_vec<T>(self) -> Self::RawVec<T> {
        DefaultRawVec {
            inner: self.inner.new_raw_vec(),
            _marker: PhantomData,
        }
    }
}

impl<T, const BUF: usize> RawVec<T> for DefaultRawVec<'_, T, BUF> {
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
