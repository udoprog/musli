use crate::buf::Load;
use crate::endian::ByteOrder;
use crate::pointer::{Ref, Size};
use crate::traits::ZeroCopy;

/// A trait implemented by slice-like types.
pub trait Slice<T>: ZeroCopy + Load<Target = [T]>
where
    T: ZeroCopy,
{
    /// An item inside of the slice.
    type Item: Load<Target = T>;

    /// Construct a slice from a [`Ref<[T]>`].
    fn from_ref<E: ByteOrder, O: Size>(slice: Ref<[T], E, O>) -> Self;

    /// Construct a slice from its metadata.
    fn with_metadata(offset: usize, len: usize) -> Self;

    /// Access an item in the slice.
    fn get(&self, index: usize) -> Option<Self::Item>;

    /// Split the slice at the given position.
    ///
    /// # Panics
    ///
    /// This panics if the given range is out of bounds.
    fn split_at(&self, at: usize) -> (Self, Self);

    /// Access an item in the slice in an unchecked manner.
    fn get_unchecked(&self, index: usize) -> Self::Item;

    /// The length of a slice.
    fn len(&self) -> usize;

    /// Test if the slice is empty.
    fn is_empty(&self) -> bool;
}

impl<T, A: ByteOrder, B: Size> Slice<T> for Ref<[T], A, B>
where
    T: ZeroCopy,
{
    type Item = Ref<T, A, B>;

    #[inline]
    fn from_ref<E: ByteOrder, O: Size>(slice: Ref<[T], E, O>) -> Self {
        Self::with_metadata(slice.offset(), slice.len())
    }

    #[inline]
    fn with_metadata(offset: usize, len: usize) -> Self
    where
        T: ZeroCopy,
    {
        Self::with_metadata(offset, len)
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Self::Item> {
        (*self).get(index)
    }

    #[inline]
    fn split_at(&self, at: usize) -> (Self, Self) {
        (*self).split_at(at)
    }

    #[inline]
    fn get_unchecked(&self, index: usize) -> Self::Item {
        (*self).get_unchecked(index)
    }

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}