use core::fmt;

use crate::error::IntoRepr;
use crate::pointer::Size;
use crate::traits::ZeroCopy;
use crate::ByteOrder;

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
    type Stored<O>: Copy + ZeroCopy + IntoRepr
    where
        O: Size;

    /// Try to construct packed value from its metadata.
    #[doc(hidden)]
    fn try_from_metadata<O>(metadata: Self::Metadata) -> Option<Self::Stored<O>>
    where
        O: Size;

    /// Will return `usize::MAX` as an invalid size.
    #[doc(hidden)]
    fn pointee_size<E: ByteOrder, O: Size>(metadata: Self::Stored<O>) -> usize;
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
    fn try_from_metadata<O>((): ()) -> Option<Self::Stored<O>>
    where
        O: Size,
    {
        Some(())
    }

    #[inline(always)]
    fn pointee_size<E: ByteOrder, O: Size>((): Self::Stored<O>) -> usize {
        size_of::<T>()
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
    fn try_from_metadata<O>(metadata: usize) -> Option<O>
    where
        O: Size,
    {
        O::try_from_usize(metadata)
    }

    #[inline(always)]
    fn pointee_size<E: ByteOrder, O: Size>(metadata: Self::Stored<O>) -> usize {
        let len = metadata.as_usize::<E>();
        size_of::<T>().saturating_mul(len)
    }
}

impl Pointee for str {
    type Metadata = usize;
    type Stored<O>
        = O
    where
        O: Size;

    #[inline(always)]
    fn try_from_metadata<O>(metadata: usize) -> Option<O>
    where
        O: Size,
    {
        O::try_from_usize(metadata)
    }

    #[inline(always)]
    fn pointee_size<E: ByteOrder, O: Size>(metadata: Self::Stored<O>) -> usize {
        metadata.as_usize::<E>()
    }
}
