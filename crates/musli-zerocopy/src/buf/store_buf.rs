mod sealed {
    #[cfg(feature = "alloc")]
    use crate::buf::OwnedBuf;
    use crate::buf::SliceMut;
    use crate::{ByteOrder, Size};

    pub trait Sealed {}

    impl<'a, E: ByteOrder, O: Size> Sealed for SliceMut<'a, E, O> {}

    #[cfg(feature = "alloc")]
    impl<E: ByteOrder, O: Size> Sealed for OwnedBuf<E, O> {}
}

/// A buffer that we can store things into.
pub trait StoreBuf: self::sealed::Sealed {}
