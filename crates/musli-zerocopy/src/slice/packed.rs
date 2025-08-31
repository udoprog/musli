use core::marker::PhantomData;
use core::mem::size_of;

use crate::buf::{Buf, Load};
use crate::endian::{ByteOrder, Native};
use crate::error::{CoerceError, Error};
use crate::pointer::{Pointee, Ref, Size};
use crate::slice::Slice;
use crate::{DefaultSize, ZeroCopy};

/// A packed slice representation that uses exactly `O` and `L` for offset and
/// length respectively.
///
/// This is functionally equivalent to a [`Ref<[T]>`], but pointer and metadata
/// `L` does not have to be the same size and its representation is packed.
///
/// ```
/// use core::mem::{size_of, align_of};
///
/// use musli_zerocopy::slice::Packed;
/// use musli_zerocopy::{DefaultSize, Ref};
///
/// assert_eq!(size_of::<Packed<[u32], u32, u8>>(), 5);
/// assert_eq!(align_of::<Packed<[u32], u32, u8>>(), 1);
///
/// assert_eq!(size_of::<Ref<[u32]>>(), size_of::<DefaultSize>() * 2);
/// assert_eq!(align_of::<Ref<[u32]>>(), align_of::<DefaultSize>());
/// ```
///
/// Since this implements [`Slice<T>`] it can be used to build collection
/// flavors like [`trie::Flavor`].
///
/// [`trie::Flavor`]: crate::trie::Flavor
#[derive(ZeroCopy)]
#[zero_copy(crate, bounds = {O: ZeroCopy, L: ZeroCopy})]
#[repr(C, packed)]
pub struct Packed<T, O = DefaultSize, L = DefaultSize, E = Native>
where
    T: ?Sized,
    O: Size,
    L: Size,
    E: ByteOrder,
{
    offset: O,
    len: L,
    #[zero_copy(ignore)]
    _marker: PhantomData<(E, T)>,
}

impl<T, O, L, E> Slice for Packed<[T], O, L, E>
where
    T: ZeroCopy,
    O: Size + TryFrom<usize>,
    L: Size + TryFrom<usize>,
    E: ByteOrder,
{
    type Item = T;
    type ItemRef = Ref<T, E, usize>;

    #[inline]
    fn from_ref<A, B>(slice: Ref<[T], A, B>) -> Self
    where
        A: ByteOrder,
        B: Size,
    {
        Self::with_metadata(slice.offset(), slice.len())
    }

    #[inline]
    fn try_from_ref<A, B>(slice: Ref<[T], A, B>) -> Result<Self, CoerceError>
    where
        A: ByteOrder,
        B: Size,
    {
        Self::try_with_metadata(slice.offset(), slice.len())
    }

    #[inline]
    fn with_metadata(offset: usize, len: usize) -> Self {
        Packed::from_raw_parts(offset, len)
    }

    #[inline]
    fn try_with_metadata(offset: usize, len: usize) -> Result<Self, CoerceError> {
        Packed::try_from_raw_parts(offset, len)
    }

    #[inline]
    fn get(self, index: usize) -> Option<Self::ItemRef> {
        Packed::get(self, index)
    }

    #[inline]
    fn split_at(self, at: usize) -> (Self, Self) {
        Packed::split_at(self, at)
    }

    #[inline]
    fn get_unchecked(self, index: usize) -> Self::ItemRef {
        Packed::get_unchecked(self, index)
    }

    #[inline]
    fn offset(self) -> usize {
        Packed::offset(self)
    }

    #[inline]
    fn len(self) -> usize {
        Packed::len(self)
    }

    #[inline]
    fn is_empty(self) -> bool {
        Packed::is_empty(self)
    }
}

impl<T, O, L, E> Packed<[T], O, L, E>
where
    T: ZeroCopy,
    O: Size,
    L: Size,
    E: ByteOrder,
{
    /// Construct a packed slice from a reference.
    #[inline]
    pub fn from_ref<A, B>(slice: Ref<[T], A, B>) -> Self
    where
        T: ZeroCopy,
        A: ByteOrder,
        B: Size,
    {
        Self::from_raw_parts(slice.offset(), slice.len())
    }

    /// Construct a packed slice from its raw parts.
    ///
    /// # Panics
    ///
    /// This panics in case any components in the path overflow its
    /// representation.
    #[inline]
    pub fn from_raw_parts(offset: usize, len: usize) -> Self {
        match Self::try_from_raw_parts(offset, len) {
            Ok(slice) => slice,
            Err(error) => panic!("{error}"),
        }
    }

    /// Try to construct a packed slice from its raw parts.
    ///
    /// # Errors
    ///
    /// This errors in case any components in the path overflow its
    /// representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::slice::Packed;
    ///
    /// let slice = Packed::<[u32], u32, u8>::try_from_raw_parts(42, 2)?;
    /// assert_eq!(slice.offset(), 42);
    ///
    /// assert!(Packed::<[u32], u32, u8>::try_from_raw_parts(42, usize::MAX).is_err());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn try_from_raw_parts(offset: usize, len: usize) -> Result<Self, CoerceError> {
        <[T]>::check_layout(offset, len)?;

        Ok(Self {
            offset: O::try_from(offset)?.swap_bytes::<E>(),
            len: L::try_from(len)?.swap_bytes::<E>(),
            _marker: PhantomData,
        })
    }

    /// Try to get a reference directly out of the slice without validation.
    ///
    /// This avoids having to validate every element in a slice in order to
    /// address them.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    /// use musli_zerocopy::slice::Packed;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let slice: Packed<[i32]> = Packed::from_ref(buf.store_slice(&[1, 2, 3, 4]));
    ///
    /// let two = slice.get(2).expect("Missing element 2");
    /// assert_eq!(buf.load(two)?, &3);
    ///
    /// assert!(slice.get(4).is_none());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn get(self, index: usize) -> Option<Ref<T, E, usize>> {
        if index >= self.len() {
            return None;
        }

        Some(self.get_unchecked(index))
    }

    /// Split the slice reference at the given position `at`.
    ///
    /// # Panics
    ///
    /// This panics if the given range is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    /// use musli_zerocopy::slice::Packed;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let slice: Packed<[i32]> = Packed::from_ref(buf.store_slice(&[1, 2, 3, 4]));
    ///
    /// buf.align_in_place();
    ///
    /// let (a, b) = slice.split_at(3);
    /// let (c, d) = slice.split_at(4);
    ///
    /// assert_eq!(buf.load(a)?, &[1, 2, 3]);
    /// assert_eq!(buf.load(b)?, &[4]);
    /// assert_eq!(buf.load(c)?, &[1, 2, 3, 4]);
    /// assert_eq!(buf.load(d)?, &[]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn split_at(self, at: usize) -> (Self, Self) {
        let offset = self.offset.swap_bytes::<E>().as_usize();
        let len = self.len.swap_bytes::<E>().as_usize();
        assert!(at <= len, "Split point {at} is out of bounds 0..={len}");
        let a = Self::from_raw_parts(offset, at);
        let b = Self::from_raw_parts(offset + at * size_of::<T>(), len - at);
        (a, b)
    }

    /// Get an unchecked reference directly out of the slice without validation.
    ///
    /// This avoids having to validate every element in a slice in order to
    /// address them.
    ///
    /// In contrast to [`get()`], this does not check that the index is within
    /// the bounds of the current slice, all though it's not unsafe since it
    /// cannot lead to anything inherently unsafe. Only garbled data.
    ///
    /// [`get()`]: Packed::get
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    /// use musli_zerocopy::slice::Packed;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let slice: Packed<[i32]> = Packed::from_ref(buf.store_slice(&[1, 2, 3, 4]));
    ///
    /// let two = slice.get_unchecked(2);
    /// assert_eq!(buf.load(two)?, &3);
    ///
    /// let oob = slice.get_unchecked(4);
    /// assert!(buf.load(oob).is_err());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn get_unchecked(self, index: usize) -> Ref<T, E, usize> {
        let offset = self.offset.swap_bytes::<E>().as_usize() + size_of::<T>() * index;
        Ref::new(offset)
    }

    /// Get the offset the packed slice points to.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::slice::Packed;
    ///
    /// let slice = Packed::<[u32], u32, u8>::from_raw_parts(42, 2);
    /// assert_eq!(slice.offset(), 42);
    /// ```
    pub fn offset(self) -> usize {
        self.offset.swap_bytes::<E>().as_usize()
    }

    /// Return the number of elements in the packed slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::slice::Packed;
    ///
    /// let slice = Packed::<[u32], u32, u8>::from_raw_parts(0, 2);
    /// assert_eq!(slice.len(), 2);
    /// ```
    #[inline]
    pub fn len(self) -> usize {
        self.len.swap_bytes::<E>().as_usize()
    }

    /// Test if the packed slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::slice::Packed;
    ///
    /// let slice = Packed::<[u32], u32, u8>::from_raw_parts(0, 0);
    /// assert!(slice.is_empty());
    ///
    /// let slice = Packed::<[u32], u32, u8>::from_raw_parts(0, 2);
    /// assert!(!slice.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len.is_zero()
    }
}

impl<T, O, L, E> Load for Packed<[T], O, L, E>
where
    T: ZeroCopy,
    O: Size,
    L: Size,
    E: ByteOrder,
{
    type Target = [T];

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        // SAFETY: We ensure the same invariants as Ref<[T]> through
        // construction.
        let r = unsafe {
            Ref::<[T], Native, usize>::new_unchecked(
                self.offset.swap_bytes::<E>().as_usize(),
                self.len.swap_bytes::<E>().as_usize(),
            )
        };

        buf.load(r)
    }
}

impl<T, O, L, E> Clone for Packed<[T], O, L, E>
where
    O: Size,
    L: Size,
    E: ByteOrder,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, O, L, E> Copy for Packed<[T], O, L, E>
where
    O: Size,
    L: Size,
    E: ByteOrder,
{
}
