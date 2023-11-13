use crate::pointer::Size;
use crate::traits::ZeroCopy;

mod sealed {
    pub trait Sealed {}
    impl Sealed for () {}
    impl Sealed for usize {}
}

/// A type that can inhabit a packed representation.
pub trait Packable: self::sealed::Sealed {
    /// The packed representation of the item.
    #[doc(hidden)]
    type Packed<O>: Copy + ZeroCopy
    where
        O: Size;

    /// Try to construct packed value from its metadata.
    fn try_from_metadata<O>(metadata: Self) -> Option<Self::Packed<O>>
    where
        O: Size;
}

impl Packable for () {
    type Packed<O> = () where O: Size;

    #[inline(always)]
    fn try_from_metadata<O>((): ()) -> Option<Self::Packed<O>>
    where
        O: Size,
    {
        Some(())
    }
}

impl Packable for usize {
    type Packed<O> = O where O: Size;

    #[inline(always)]
    fn try_from_metadata<O>(metadata: Self) -> Option<Self::Packed<O>>
    where
        O: Size,
    {
        O::try_from_usize(metadata)
    }
}
