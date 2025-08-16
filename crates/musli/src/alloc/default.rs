use core::marker::PhantomData;

use super::{Alloc, AllocError, Allocator};
#[cfg(feature = "alloc")]
use super::{Global, GlobalAlloc};
#[cfg(not(feature = "alloc"))]
use super::{Slice, SliceAlloc};

/// The default stack buffer size for the default allocator provided through
/// [`default()`].
///
/// [`default()`]: super::default()
pub const DEFAULT_ARRAY_BUFFER: usize = 4096;

macro_rules! implement {
    ($id:ident, $ty:ty, $raw_vec:ty, $raw:ty) => {
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

        /// The default raw allocation.
        ///
        /// The exact implementation of this depends on if the `alloc` feature
        /// is enabled.
        ///
        /// For more information, see [`default()`].
        ///
        /// [`default()`]: super::default()
        pub struct DefaultAlloc<'a, T, const BUF: usize> {
            inner: $raw,
            _marker: PhantomData<&'a ()>,
        }
    };
}

#[cfg(feature = "alloc")]
implement!(DefaultAllocator, Global, SystemAlloc<T>, GlobalAlloc<T>);

#[cfg(not(feature = "alloc"))]
implement!(
    DefaultAllocator,
    Slice<'buf>,
    SliceAlloc<'a, T>,
    SliceAlloc<'a, T>
);

unsafe impl<'a, const BUF: usize> Allocator for &'a DefaultAllocator<'_, BUF> {
    #[cfg(feature = "alloc")]
    const IS_GLOBAL: bool = true;

    #[cfg(not(feature = "alloc"))]
    const IS_GLOBAL: bool = false;

    type Alloc<T> = DefaultAlloc<'a, T, BUF>;

    #[inline]
    fn alloc<T>(self, value: T) -> Result<Self::Alloc<T>, AllocError> {
        Ok(DefaultAlloc {
            inner: self.inner.alloc(value)?,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn alloc_empty<T>(self) -> Self::Alloc<T> {
        DefaultAlloc {
            inner: self.inner.alloc_empty(),
            _marker: PhantomData,
        }
    }
}

impl<T, const BUF: usize> Alloc<T> for DefaultAlloc<'_, T, BUF> {
    #[inline]
    fn as_ptr(&self) -> *const T {
        Alloc::as_ptr(&self.inner)
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        Alloc::as_mut_ptr(&mut self.inner)
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    fn resize(&mut self, len: usize, additional: usize) -> Result<(), AllocError> {
        self.inner.resize(len, additional)
    }

    #[inline]
    fn try_merge<B>(&mut self, this_len: usize, other: B, other_len: usize) -> Result<(), B>
    where
        B: Alloc<T>,
    {
        self.inner.try_merge(this_len, other, other_len)
    }
}
