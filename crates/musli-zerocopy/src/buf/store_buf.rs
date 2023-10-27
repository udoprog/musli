use core::slice::SliceIndex;

use crate::pointer::Pointee;
use crate::traits::{UnsizedZeroCopy, ZeroCopy};
use crate::{Buf, ByteOrder, Error, Ref, Size};

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
#[allow(clippy::len_without_is_empty)]
pub trait StoreBuf: self::sealed::Sealed {
    /// The sticky endianness associated with the buffer.
    type ByteOrder: ByteOrder;

    /// The sticky size associated with the buffer.
    type Size: Size;

    /// The current initialized length of the buffer.
    #[doc(hidden)]
    fn len(&self) -> usize;

    /// Store an unsigned value.
    #[doc(hidden)]
    fn store_unsized<P: ?Sized>(&mut self, value: &P) -> Ref<P, Self::ByteOrder, Self::Size>
    where
        P: Pointee<Self::Size, Packed = Self::Size, Metadata = usize>,
        P: UnsizedZeroCopy<P, Self::Size>;

    /// Store a [`ZeroCopy`] value.
    #[doc(hidden)]
    fn store<P>(&mut self, value: &P) -> Ref<P, Self::ByteOrder, Self::Size>
    where
        P: ZeroCopy;

    /// Swap the location of two references.
    #[doc(hidden)]
    fn swap<P>(
        &mut self,
        a: Ref<P, Self::ByteOrder, Self::Size>,
        b: Ref<P, Self::ByteOrder, Self::Size>,
    ) -> Result<(), Error>
    where
        P: ZeroCopy;

    /// Ensure that the store buffer is aligned.
    ///
    /// For buffers which cannot be re-aligned, this will simply panic.
    #[doc(hidden)]
    fn align_in_place(&mut self);

    /// Construct an offset aligned for `T` into the current buffer which points
    /// to the next location that will be written.
    #[doc(hidden)]
    fn next_offset<T>(&mut self) -> usize;

    /// Align by `align` and `reserve` additional space in the buffer or panic.
    #[doc(hidden)]
    fn next_offset_with_and_reserve(&mut self, align: usize, reserve: usize);

    /// Fill the buffer with `len` repetitions of `byte`.
    #[doc(hidden)]
    fn fill(&mut self, byte: u8, len: usize);

    /// Get an immutable slice.
    #[doc(hidden)]
    fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[u8]>;

    /// Get a mutable slice.
    #[doc(hidden)]
    fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[u8]>;

    /// Get the underlying buffer.
    #[doc(hidden)]
    fn as_buf(&self) -> &Buf;

    /// Get the underlying buffer mutably.
    #[doc(hidden)]
    fn as_mut_buf(&mut self) -> &mut Buf;
}
