use core::alloc::{Layout, LayoutError};
use core::mem::size_of;

use crate::error::CoerceError;
use crate::pointer::Size;
use crate::traits::ZeroCopy;

mod sealed {
    use crate::mem::MaybeUninit;
    use crate::pointer::Pointee;
    use crate::traits::ZeroCopy;

    pub trait Sealed {}

    impl<T> Sealed for MaybeUninit<T> where T: Pointee {}
    impl<T> Sealed for T where T: ZeroCopy {}
    impl<T> Sealed for [T] where T: ZeroCopy {}
    impl Sealed for str {}
}

/// The trait for a value that can be pointed to by a [`Ref<T>`].
///
/// This ultimately determines the layout of [`Ref<T>`] as for unsized types it
/// needs to accommodate the size of the pointed-to type as well.
///
/// ```
/// use std::mem::size_of;
///
/// use musli_zerocopy::Ref;
///
/// assert_eq!(size_of::<Ref::<u32>>(), 4);
/// assert_eq!(size_of::<Ref::<[u32]>>(), 8);
/// ```
///
/// [`Ref<T>`]: crate::Ref
pub trait Pointee: self::sealed::Sealed {
    /// Metadata associated with a pointee.
    type Metadata: Copy;

    /// The stored representation of the pointee metadata.
    #[doc(hidden)]
    type Stored<O>: Copy + ZeroCopy
    where
        O: Size;

    /// Try to construct packed value from its metadata.
    #[doc(hidden)]
    fn try_from_metadata<O>(metadata: Self::Metadata) -> Result<Self::Stored<O>, CoerceError>
    where
        O: Size;

    /// The size of `T` with the given stored metadata.
    #[doc(hidden)]
    fn size<O>(metadata: Self::Stored<O>) -> Option<usize>
    where
        O: Size;

    /// The alignment of `T` with the given stored metadata.
    #[doc(hidden)]
    fn align<O>(metadata: Self::Stored<O>) -> usize
    where
        O: Size;

    /// The layout of `T` with the given stored metadata.
    #[doc(hidden)]
    fn pointee_layout<O>(metadata: Self::Stored<O>) -> Result<Layout, LayoutError>
    where
        O: Size;
}

impl<T> Pointee for T
where
    T: ZeroCopy,
{
    type Metadata = ();
    type Stored<O>
        = ()
    where
        O: Size;

    #[inline(always)]
    fn try_from_metadata<O>((): ()) -> Result<Self::Stored<O>, CoerceError>
    where
        O: Size,
    {
        Ok(())
    }

    #[inline(always)]
    fn size<O>((): Self::Stored<O>) -> Option<usize>
    where
        O: Size,
    {
        Some(size_of::<T>())
    }

    #[inline(always)]
    fn align<O>((): Self::Stored<O>) -> usize
    where
        O: Size,
    {
        align_of::<T>()
    }

    #[inline(always)]
    fn pointee_layout<O>((): Self::Stored<O>) -> Result<Layout, LayoutError>
    where
        O: Size,
    {
        Ok(Layout::new::<T>())
    }
}

impl<T> Pointee for [T]
where
    T: ZeroCopy,
{
    type Metadata = usize;
    type Stored<O>
        = O
    where
        O: Size;

    #[inline(always)]
    fn try_from_metadata<O>(metadata: usize) -> Result<O, CoerceError>
    where
        O: Size,
    {
        O::try_from_usize(metadata)
    }

    #[inline(always)]
    fn size<O>(metadata: Self::Stored<O>) -> Option<usize>
    where
        O: Size,
    {
        let len = metadata.as_usize();
        size_of::<T>().checked_mul(len)
    }

    #[inline(always)]
    fn align<O>(_: Self::Stored<O>) -> usize
    where
        O: Size,
    {
        align_of::<T>()
    }

    #[inline(always)]
    fn pointee_layout<O>(metadata: Self::Stored<O>) -> Result<Layout, LayoutError>
    where
        O: Size,
    {
        let len = metadata.as_usize();
        Layout::array::<T>(len)
    }
}

impl Pointee for str {
    type Metadata = usize;
    type Stored<O>
        = O
    where
        O: Size;

    #[inline(always)]
    fn try_from_metadata<O>(metadata: usize) -> Result<O, CoerceError>
    where
        O: Size,
    {
        O::try_from_usize(metadata)
    }

    #[inline(always)]
    fn size<O>(metadata: Self::Stored<O>) -> Option<usize>
    where
        O: Size,
    {
        Some(metadata.as_usize())
    }

    #[inline(always)]
    fn align<O>(_: Self::Stored<O>) -> usize
    where
        O: Size,
    {
        align_of::<u8>()
    }

    #[inline(always)]
    fn pointee_layout<O>(metadata: Self::Stored<O>) -> Result<Layout, LayoutError>
    where
        O: Size,
    {
        Layout::array::<u8>(metadata.as_usize())
    }
}
