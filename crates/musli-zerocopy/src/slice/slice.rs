use crate::buf::Load;
use crate::endian::ByteOrder;
use crate::error::CoerceError;
use crate::pointer::{Ref, Size};
use crate::traits::ZeroCopy;

mod sealed {
    use crate::endian::ByteOrder;
    use crate::pointer::{Ref, Size};
    use crate::slice::Packed;
    use crate::traits::ZeroCopy;

    pub trait Sealed {}

    impl<T, E, O> Sealed for Ref<[T], E, O>
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
    }

    impl<T, O, L, E> Sealed for Packed<[T], O, L, E>
    where
        O: Size,
        L: Size,
        E: ByteOrder,
    {
    }
}

/// A trait implemented by slice-like types.
pub trait Slice: self::sealed::Sealed + Copy + ZeroCopy + Load<Target = [Self::Item]> {
    /// The item in an unsized slice, or the `T` in `[T]`.
    type Item;

    /// A returned reference to an item in a slice.
    type ItemRef: Load<Target = Self::Item>;

    /// Construct a slice from a [`Ref<[Self::Item]>`].
    ///
    /// # Panics
    ///
    /// This method panics if construction of the slice would overflow any of
    /// its parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Ref, ZeroCopy};
    /// use musli_zerocopy::slice::Slice;
    ///
    /// fn generic<S>(r: Ref<[S::Item]>) -> S
    /// where
    ///     S: Slice,
    ///     S::Item: ZeroCopy
    /// {
    ///     S::from_ref(r)
    /// }
    /// ```
    fn from_ref<E, O>(slice: Ref<[Self::Item], E, O>) -> Self
    where
        Self::Item: ZeroCopy,
        E: ByteOrder,
        O: Size;

    /// Try to construct a slice from a [`Ref<[Self::Item]>`].
    ///
    /// # Errors
    ///
    /// This method errors if construction of the slice would overflow any of
    /// its parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Error, Ref, ZeroCopy};
    /// use musli_zerocopy::slice::Slice;
    ///
    /// fn generic<S>(r: Ref<[S::Item]>) -> Result<S, Error>
    /// where
    ///     S: Slice<Item: ZeroCopy>
    /// {
    ///     Ok(S::try_from_ref(r)?)
    /// }
    /// ```
    fn try_from_ref<E, O>(slice: Ref<[Self::Item], E, O>) -> Result<Self, CoerceError>
    where
        Self::Item: ZeroCopy,
        E: ByteOrder,
        O: Size;

    /// Construct a slice from its `offset` and `len`.
    ///
    /// # Panics
    ///
    /// This method panics if construction of the slice would overflow any of
    /// its parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    /// use musli_zerocopy::slice::Slice;
    ///
    /// fn generic<S>() -> S where S: Slice {
    ///     S::with_metadata(0usize, 10)
    /// }
    /// ```
    fn with_metadata(offset: usize, len: usize) -> Self;

    /// Construct a slice from its `offset` and `len`.
    ///
    /// # Errors
    ///
    /// This method errors if construction of the slice would overflow any of
    /// its parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    /// use musli_zerocopy::slice::Slice;
    ///
    /// fn generic<S>() -> S where S: Slice {
    ///     S::with_metadata(0usize, 10)
    /// }
    /// ```
    fn try_with_metadata(offset: usize, len: usize) -> Result<Self, CoerceError>;

    /// Try to get a reference directly out of the slice without validation.
    ///
    /// This avoids having to validate every element in a slice in order to
    /// address them.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Buf, Error, OwnedBuf};
    /// use musli_zerocopy::slice::Slice;
    ///
    /// fn generic<S>(buf: &Buf, slice: S) -> Result<(), Error>
    /// where
    ///     S: Slice<Item = i32>
    /// {
    ///     let two = slice.get(2).expect("Missing element 2");
    ///     assert_eq!(buf.load(two)?, &3);
    ///
    ///     assert!(slice.get(4).is_none());
    ///     Ok(())
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
    ///
    /// generic(&buf, slice)?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    fn get(self, index: usize) -> Option<Self::ItemRef>;

    /// Split the slice at the given position `at`.
    ///
    /// # Panics
    ///
    /// This panics if the given range is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Buf, Error, OwnedBuf};
    /// use musli_zerocopy::slice::Slice;
    ///
    /// fn generic<S>(buf: &Buf, slice: S) -> Result<(), Error>
    /// where
    ///     S: Slice<Item = i32>
    /// {
    ///     let (a, b) = slice.split_at(3);
    ///     let (c, d) = slice.split_at(4);
    ///
    ///     assert_eq!(buf.load(a)?, &[1, 2, 3]);
    ///     assert_eq!(buf.load(b)?, &[4]);
    ///     assert_eq!(buf.load(c)?, &[1, 2, 3, 4]);
    ///     assert_eq!(buf.load(d)?, &[]);
    ///     Ok(())
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
    ///
    /// buf.align_in_place();
    ///
    /// generic(&buf, slice)?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    fn split_at(self, at: usize) -> (Self, Self);

    /// Get an unchecked reference directly out of the slice without validation.
    ///
    /// This avoids having to validate every element in a slice in order to
    /// address them.
    ///
    /// In contrast to [`get()`], this does not check that the index is within
    /// the bounds of the current slice, all though it's not unsafe since it
    /// cannot lead to anything inherently unsafe. Only garbled data.
    ///
    /// [`get()`]: Slice::get
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Buf, Error, Ref, OwnedBuf};
    /// use musli_zerocopy::slice::Slice;
    ///
    /// // A method generic over a specific slice implementation.
    /// fn generic<S>(buf: &Buf, slice: S) -> Result<(), Error>
    /// where
    ///     S: Slice<Item = i32>
    /// {
    ///     let two = slice.get_unchecked(2);
    ///     assert_eq!(buf.load(two)?, &3);
    ///
    ///     let oob = slice.get_unchecked(4);
    ///     assert!(buf.load(oob).is_err());
    ///     Ok(())
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
    /// generic(&buf, slice)?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    fn get_unchecked(self, index: usize) -> Self::ItemRef;

    /// Get the offset the slice points to.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    /// use musli_zerocopy::slice::Slice;
    ///
    /// // A method generic over a specific slice implementation.
    /// fn generic<S>(slice: S) where S: Slice {
    ///     assert_eq!(slice.offset(), 42);
    /// }
    ///
    /// let slice = Ref::<[i32]>::with_metadata(42u32, 2);
    /// generic(slice);
    /// ```
    fn offset(self) -> usize;

    /// Return the number of elements in the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    /// use musli_zerocopy::slice::Slice;
    ///
    /// // A method generic over a specific slice implementation.
    /// fn generic<S>(slice: S) where S: Slice {
    ///     assert_eq!(slice.len(), 2);
    /// }
    ///
    /// let slice = Ref::<[i32]>::with_metadata(0u32, 2);
    /// generic(slice);
    /// ```
    fn len(self) -> usize;

    /// Test if the slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    /// use musli_zerocopy::slice::Slice;
    ///
    /// // A method generic over a specific slice implementation.
    /// fn generic<S>(a: S, b: S) where S: Slice {
    ///     assert!(a.is_empty());
    ///     assert!(!b.is_empty());
    /// }
    ///
    /// let a = Ref::<[u32]>::with_metadata(0u32, 0);
    /// let b = Ref::<[u32]>::with_metadata(0u32, 2);
    /// generic(a, b);
    /// ```
    fn is_empty(self) -> bool;
}

impl<T, A: ByteOrder, B: Size> Slice for Ref<[T], A, B>
where
    T: ZeroCopy,
{
    type Item = T;
    type ItemRef = Ref<T, A, B>;

    #[inline]
    fn from_ref<E, O>(slice: Ref<[T], E, O>) -> Self
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
        Ref::with_metadata(slice.offset(), slice.len())
    }

    #[inline]
    fn try_from_ref<E, O>(slice: Ref<[T], E, O>) -> Result<Self, CoerceError>
    where
        T: ZeroCopy,
        E: ByteOrder,
        O: Size,
    {
        Ref::try_with_metadata(slice.offset(), slice.len())
    }

    #[inline]
    fn with_metadata(offset: usize, len: usize) -> Self {
        Ref::with_metadata(offset, len)
    }

    #[inline]
    fn try_with_metadata(offset: usize, len: usize) -> Result<Self, CoerceError> {
        Ref::try_with_metadata(offset, len)
    }

    #[inline]
    fn get(self, index: usize) -> Option<Self::ItemRef> {
        Ref::get(self, index)
    }

    #[inline]
    fn split_at(self, at: usize) -> (Self, Self) {
        Ref::split_at(self, at)
    }

    #[inline]
    fn get_unchecked(self, index: usize) -> Self::ItemRef {
        Ref::get_unchecked(self, index)
    }

    #[inline]
    fn offset(self) -> usize {
        Ref::<[T], _, _>::offset(self)
    }

    #[inline]
    fn len(self) -> usize {
        Ref::<[T], _, _>::len(self)
    }

    #[inline]
    fn is_empty(self) -> bool {
        Ref::<[T], _, _>::is_empty(self)
    }
}
