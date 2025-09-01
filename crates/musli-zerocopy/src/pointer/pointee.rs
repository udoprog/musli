use core::alloc::{Layout, LayoutError};
use core::mem::size_of;

use crate::error::{CoerceError, CoerceErrorKind};
use crate::pointer::Size;
use crate::traits::ZeroCopy;

mod sealed {
    use crate::mem::PackedMaybeUninit;
    use crate::pointer::Pointee;
    use crate::traits::ZeroCopy;

    pub trait Sealed {}

    impl<T> Sealed for PackedMaybeUninit<T> where T: Pointee {}
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
pub trait Pointee
where
    Self: self::sealed::Sealed,
{
    /// Metadata associated with a pointee.
    #[doc(hidden)]
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

    /// Convert packed metadata to metadata.
    #[doc(hidden)]
    fn to_metadata<O>(stored: Self::Stored<O>) -> Self::Metadata
    where
        O: Size;

    /// The size of `T` with the given stored metadata.
    #[doc(hidden)]
    fn size(metadata: Self::Metadata) -> Option<usize>;

    /// The alignment of `T` with the given stored metadata.
    #[doc(hidden)]
    fn align(metadata: Self::Metadata) -> usize;

    /// The layout of `T` with the given stored metadata.
    #[doc(hidden)]
    fn pointee_layout(metadata: Self::Metadata) -> Result<Layout, LayoutError>;

    #[inline(always)]
    #[doc(hidden)]
    fn check_layout(offset: usize, metadata: Self::Metadata) -> Result<(), CoerceError> {
        let Ok(layout) = Self::pointee_layout(metadata) else {
            return Err(CoerceError::new(CoerceErrorKind::InvalidLayout {
                size: Self::size(metadata),
                align: Self::align(metadata),
            }));
        };

        if offset.checked_add(layout.size()).is_none() {
            return Err(CoerceError::new(CoerceErrorKind::InvalidOffsetRange {
                offset,
                end: usize::MAX - layout.size(),
            }));
        };

        Ok(())
    }
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
    fn to_metadata<O>((): ()) -> Self::Metadata
    where
        O: Size,
    {
    }

    #[inline(always)]
    fn size((): Self::Metadata) -> Option<usize> {
        Some(size_of::<T>())
    }

    #[inline(always)]
    fn align((): Self::Metadata) -> usize {
        align_of::<T>()
    }

    #[inline(always)]
    fn pointee_layout((): Self::Metadata) -> Result<Layout, LayoutError> {
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
    fn to_metadata<O>(metadata: Self::Stored<O>) -> Self::Metadata
    where
        O: Size,
    {
        metadata.as_usize()
    }

    #[inline(always)]
    fn size(metadata: Self::Metadata) -> Option<usize> {
        size_of::<T>().checked_mul(metadata)
    }

    #[inline(always)]
    fn align(_: Self::Metadata) -> usize {
        align_of::<T>()
    }

    #[inline(always)]
    fn pointee_layout(metadata: Self::Metadata) -> Result<Layout, LayoutError> {
        Layout::array::<T>(metadata)
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
    fn to_metadata<O>(metadata: Self::Stored<O>) -> Self::Metadata
    where
        O: Size,
    {
        metadata.as_usize()
    }

    #[inline(always)]
    fn size(metadata: Self::Metadata) -> Option<usize> {
        Some(metadata.as_usize())
    }

    #[inline(always)]
    fn align(_: Self::Metadata) -> usize {
        align_of::<u8>()
    }

    #[inline(always)]
    fn pointee_layout(metadata: Self::Metadata) -> Result<Layout, LayoutError> {
        Layout::array::<u8>(metadata)
    }
}
