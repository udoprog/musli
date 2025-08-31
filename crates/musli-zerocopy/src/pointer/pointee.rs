use core::fmt;
use core::mem::{align_of, size_of};

use crate::ByteOrder;
use crate::error::IntoRepr;
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
    type Metadata: Copy + fmt::Debug + IntoRepr;

    /// The stored representation of the pointee metadata.
    #[doc(hidden)]
    type Stored<O>: Copy + ZeroCopy
    where
        O: Size;

    /// Get the size of the stored type from its metadata.
    #[doc(hidden)]
    fn size(metadata: &Self::Metadata) -> Option<usize>;

    /// Get the alignment of the pointee.
    #[doc(hidden)]
    fn align(metadata: &Self::Metadata) -> usize;

    /// Try to construct packed value from its metadata.
    #[doc(hidden)]
    fn try_from_metadata<O>(metadata: Self::Metadata) -> Option<Self::Stored<O>>
    where
        O: Size;

    /// Try to convert stored representation back to metadata.
    #[doc(hidden)]
    fn to_metadata<O, E>(stored: Self::Stored<O>) -> Self::Metadata
    where
        O: Size,
        E: ByteOrder;
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
    fn size((): &Self::Metadata) -> Option<usize> {
        Some(size_of::<T>())
    }

    #[inline(always)]
    fn align((): &Self::Metadata) -> usize {
        align_of::<T>()
    }

    #[inline(always)]
    fn try_from_metadata<O>((): ()) -> Option<Self::Stored<O>>
    where
        O: Size,
    {
        Some(())
    }

    #[inline(always)]
    fn to_metadata<O, E>((): ()) -> Self::Metadata
    where
        O: Size,
        E: ByteOrder,
    {
        ()
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
    fn size(metadata: &Self::Metadata) -> Option<usize> {
        // NB: This multiplication cannot overflow since Rust limits the max
        // size of a slice so that its extend can fit within the bounds of a
        // `usize`.
        size_of::<T>().checked_mul(*metadata)
    }

    #[inline(always)]
    fn align(_: &Self::Metadata) -> usize {
        align_of::<T>()
    }

    #[inline(always)]
    fn try_from_metadata<O>(metadata: usize) -> Option<O>
    where
        O: Size,
    {
        O::try_from_usize(metadata)
    }

    #[inline(always)]
    fn to_metadata<O, E>(metadata: O) -> usize
    where
        O: Size,
        E: ByteOrder,
    {
        O::as_usize::<E>(metadata)
    }
}

impl Pointee for str {
    type Metadata = usize;
    type Stored<O>
        = O
    where
        O: Size;

    #[inline(always)]
    fn size(metadata: &Self::Metadata) -> Option<usize> {
        Some(*metadata)
    }

    #[inline(always)]
    fn align(_: &Self::Metadata) -> usize {
        align_of::<u8>()
    }

    #[inline(always)]
    fn try_from_metadata<O>(metadata: usize) -> Option<O>
    where
        O: Size,
    {
        O::try_from_usize(metadata)
    }

    #[inline(always)]
    fn to_metadata<O, E>(metadata: O) -> usize
    where
        O: Size,
        E: ByteOrder,
    {
        O::as_usize::<E>(metadata)
    }
}
