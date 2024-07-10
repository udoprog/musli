use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::marker::PhantomData;
use core::mem::size_of;

use crate::endian::{Big, ByteOrder, Little, Native};
use crate::error::{Error, ErrorKind, IntoRepr};
use crate::mem::MaybeUninit;
use crate::pointer::Coerce;
use crate::pointer::{DefaultSize, Pointee, Size};
use crate::ZeroCopy;

/// A stored reference to a type `T`.
///
/// A reference is made up of two components:
/// * An [`offset()`] indicating the absolute offset into a [`Buf`] where the
///   pointed-to (pointee) data is located.
/// * An optional [`metadata()`] components, which if set indicates that this
///   reference is a wide pointer. This is used when encoding types such as
///   `[T]` or `str` to include additional data necessary to handle the type.
///
/// [`Buf`]: crate::buf::Buf
/// [`offset()`]: Ref::offset
/// [`metadata()`]: Ref::metadata
///
/// # Examples
///
/// ```
/// use std::mem::align_of;
///
/// use musli_zerocopy::{Ref, OwnedBuf};
///
/// let mut buf = OwnedBuf::with_alignment::<u32>();
/// buf.extend_from_slice(&[1, 2, 3, 4]);
///
/// let buf = buf.as_ref();
///
/// let number = Ref::<u32>::new(0);
/// assert_eq!(*buf.load(number)?, u32::from_ne_bytes([1, 2, 3, 4]));
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(ZeroCopy)]
#[repr(C)]
#[zero_copy(crate, swap_bytes_self)]
pub struct Ref<T, E = Native, O = DefaultSize>
where
    T: ?Sized + Pointee,
    E: ByteOrder,
    O: Size,
{
    offset: O,
    metadata: T::Stored<O>,
    #[zero_copy(ignore)]
    _marker: PhantomData<(E, T)>,
}

impl<T, E, O> Ref<T, E, O>
where
    T: ?Sized + Pointee,
    E: ByteOrder,
    O: Size,
{
    /// Convert this reference into a [`Big`]-endian [`ByteOrder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{endian, Ref};
    ///
    /// let r: Ref<u32> = Ref::new(10);
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Little> = Ref::new(10);
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Big> = r.to_be();
    /// assert_eq!(r.offset(), 10);
    /// ```
    #[inline]
    pub fn to_be(self) -> Ref<T, Big, O> {
        self.to_endian()
    }

    /// Convert this reference into a [`Little`]-endian [`ByteOrder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{endian, Ref};
    ///
    /// let r: Ref<u32> = Ref::new(10);
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Big> = Ref::new(10);
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Little> = r.to_le();
    /// assert_eq!(r.offset(), 10);
    /// ```
    #[inline]
    pub fn to_le(self) -> Ref<T, Little, O> {
        self.to_endian()
    }

    /// Convert this reference into a [`Native`]-endian [`ByteOrder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{endian, Ref};
    ///
    /// let r: Ref<u32, endian::Native> = Ref::<u32, endian::Big>::new(10).to_ne();
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Native> = Ref::<u32, endian::Little>::new(10).to_ne();
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Native> = Ref::<u32, endian::Native>::new(10).to_ne();
    /// assert_eq!(r.offset(), 10);
    /// ```
    #[inline]
    pub fn to_ne(self) -> Ref<T, Native, O> {
        self.to_endian()
    }

    /// Convert this reference into a `U`-endian [`ByteOrder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{endian, Ref};
    ///
    /// let r: Ref<u32, endian::Native> = Ref::<u32, endian::Big>::new(10).to_endian();
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Native> = Ref::<u32, endian::Little>::new(10).to_endian();
    /// assert_eq!(r.offset(), 10);
    ///
    /// let r: Ref<u32, endian::Native> = Ref::<u32, endian::Native>::new(10).to_endian();
    /// assert_eq!(r.offset(), 10);
    /// ```
    #[inline]
    pub fn to_endian<U: ByteOrder>(self) -> Ref<T, U, O> {
        Ref {
            offset: self.offset.swap_bytes::<E>().swap_bytes::<U>(),
            metadata: self.metadata.swap_bytes::<E>().swap_bytes::<U>(),
            _marker: PhantomData,
        }
    }
}

impl<T, E, O> Ref<T, E, O>
where
    T: ?Sized + Pointee,
    E: ByteOrder,
    O: Size,
{
    /// Construct a reference with custom metadata.
    ///
    /// # Panics
    ///
    /// This will panic if either:
    /// * The `offset` or `metadata` can't be byte swapped as per
    ///   [`ZeroCopy::CAN_SWAP_BYTES`].
    /// * Packed [`offset()`] cannot be constructed from `U` (out of range).
    /// * Packed [`metadata()`] cannot be constructed from `T::Metadata` (reason
    ///   depends on the exact metadata).
    ///
    /// To guarantee that this constructor will never panic, [`Ref<T, E,
    /// usize>`] can be used. This also ensures that construction is a no-op.
    ///
    /// [`offset()`]: Ref::offset
    /// [`metadata()`]: Ref::metadata
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference = Ref::<[u64]>::with_metadata(42, 10);
    /// assert_eq!(reference.offset(), 42);
    /// assert_eq!(reference.len(), 10);
    /// ```
    #[inline]
    pub fn with_metadata<U>(offset: U, metadata: T::Metadata) -> Self
    where
        U: Copy + fmt::Debug,
        O: TryFrom<U>,
    {
        const {
            assert!(
                O::CAN_SWAP_BYTES,
                "Offset cannot be byte-ordered since it would not inhabit valid types"
            );
        }

        let Some(offset) = O::try_from(offset).ok() else {
            panic!("Offset {offset:?} not in legal range 0-{}", O::MAX);
        };

        let Some(metadata) = T::try_from_metadata(metadata) else {
            panic!("Metadata {metadata:?} not in legal range 0-{}", O::MAX);
        };

        Self {
            offset: O::swap_bytes::<E>(offset),
            metadata: T::Stored::<O>::swap_bytes::<E>(metadata),
            _marker: PhantomData,
        }
    }

    /// Fallibly try to construct a reference with metadata.
    ///
    /// # Errors
    ///
    /// This will not compile through a constant assertion if the `offset` or
    ///   `metadata` can't be byte swapped as per [`ZeroCopy::CAN_SWAP_BYTES`].
    ///
    /// This will error if either:
    /// * Packed [`offset()`] cannot be constructed from `U` (out of range).
    /// * Packed [`metadata()`] cannot be constructed from `T::Metadata` (reason
    ///   depends on the exact metadata).
    ///
    /// To guarantee that this constructor will never error, [`Ref<T, Native,
    /// usize>`] can be used. This also ensures that construction is a no-op.
    ///
    /// [`offset()`]: Ref::offset
    /// [`metadata()`]: Ref::metadata
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference = Ref::<[u64]>::try_with_metadata(42, 10)?;
    /// assert_eq!(reference.offset(), 42);
    /// assert_eq!(reference.len(), 10);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn try_with_metadata<U>(offset: U, metadata: T::Metadata) -> Result<Self, Error>
    where
        U: Copy + IntoRepr + fmt::Debug,
        O: TryFrom<U>,
    {
        const {
            assert!(
                O::CAN_SWAP_BYTES,
                "Offset cannot be byte-ordered since it would not inhabit valid types"
            );

            assert!(
                T::Stored::<O>::CAN_SWAP_BYTES,
                "Packed offset cannot be byte-ordered since it would not inhabit valid types"
            );
        }

        let Some(offset) = O::try_from(offset).ok() else {
            return Err(Error::new(ErrorKind::InvalidOffsetRange {
                offset: U::into_repr(offset),
                max: O::into_repr(O::MAX),
            }));
        };

        let Some(metadata) = T::try_from_metadata(metadata) else {
            return Err(Error::new(ErrorKind::InvalidMetadataRange {
                metadata: T::Metadata::into_repr(metadata),
                max: O::into_repr(O::MAX),
            }));
        };

        Ok(Self {
            offset: O::swap_bytes::<E>(offset),
            metadata: T::Stored::swap_bytes::<E>(metadata),
            _marker: PhantomData,
        })
    }
}

impl<T, E, O> Ref<[T], E, O>
where
    T: ZeroCopy,
    E: ByteOrder,
    O: Size,
{
    /// Return the number of elements in the slice `[T]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let slice = Ref::<[u32]>::with_metadata(0, 2);
    /// assert_eq!(slice.len(), 2);
    /// ```
    #[inline]
    pub fn len(self) -> usize {
        self.metadata.as_usize::<E>()
    }

    /// Test if the slice `[T]` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let slice = Ref::<[u32]>::with_metadata(0, 0);
    /// assert!(slice.is_empty());
    ///
    /// let slice = Ref::<[u32]>::with_metadata(0, 2);
    /// assert!(!slice.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(self) -> bool {
        self.metadata.is_zero()
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
    ///
    /// let mut buf = OwnedBuf::new();
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
    ///
    /// let two = slice.get(2).expect("Missing element 2");
    /// assert_eq!(buf.load(two)?, &3);
    ///
    /// assert!(slice.get(4).is_none());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn get(self, index: usize) -> Option<Ref<T, E, O>> {
        if index >= self.len() {
            return None;
        }

        let offset = self.offset.as_usize::<E>() + size_of::<T>() * index;
        Some(Ref::new(offset))
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
    /// [`get()`]: Ref::get
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
    ///
    /// let two = slice.get_unchecked(2);
    /// assert_eq!(buf.load(two)?, &3);
    ///
    /// let oob = slice.get_unchecked(4);
    /// assert!(buf.load(oob).is_err());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get_unchecked(self, index: usize) -> Ref<T, E, O> {
        let offset = self.offset.as_usize::<E>() + size_of::<T>() * index;
        Ref::new(offset)
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
    ///
    /// let mut buf = OwnedBuf::new();
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
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
        let offset = self.offset();
        let len = self.len();
        assert!(at <= len, "Split point {at} is out of bounds 0..={len}");
        let a = Self::with_metadata(offset, at);
        let b = Self::with_metadata(offset + at * size_of::<T>(), len - at);
        (a, b)
    }

    /// Perform an fetch like `get` which panics with diagnostics in case the
    /// index is out-of-bounds.
    #[inline]
    #[cfg(feature = "alloc")]
    pub(crate) fn at(self, index: usize) -> Ref<T, E, O> {
        let Some(r) = self.get(index) else {
            panic!("Index {index} out of bounds 0-{}", self.len());
        };

        r
    }

    /// Construct an iterator over this reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
    ///
    /// buf.align_in_place();
    ///
    /// let mut out = Vec::new();
    ///
    /// for r in slice.iter() {
    ///     out.push(*buf.load(r)?);
    /// }
    ///
    /// for r in slice.iter().rev() {
    ///     out.push(*buf.load(r)?);
    /// }
    ///
    /// assert_eq!(out, [1, 2, 3, 4, 4, 3, 2, 1]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn iter(self) -> Iter<T, E, O> {
        let start = self.offset.as_usize::<E>();
        let end = start + self.metadata.as_usize::<E>() * size_of::<T>();

        Iter {
            start,
            end,
            _marker: PhantomData,
        }
    }
}

impl<E, O> Ref<str, E, O>
where
    E: ByteOrder,
    O: Size,
{
    /// Return the length of the string.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let slice = Ref::<str>::with_metadata(0, 2);
    /// assert_eq!(slice.len(), 2);
    /// ```
    #[inline]
    pub fn len(self) -> usize {
        self.metadata.as_usize::<E>()
    }

    /// Test if the slice `[T]` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let slice = Ref::<str>::with_metadata(0, 0);
    /// assert!(slice.is_empty());
    ///
    /// let slice = Ref::<str>::with_metadata(0, 2);
    /// assert!(!slice.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(self) -> bool {
        self.metadata.is_zero()
    }
}

/// An iterator over a `Ref<[T]>` which produces `Ref<T>` values.
///
/// See [`Ref::iter`].
pub struct Iter<T, E, O> {
    start: usize,
    end: usize,
    _marker: PhantomData<(T, E, O)>,
}

impl<T, E, O> Iterator for Iter<T, E, O>
where
    T: ZeroCopy,
    E: ByteOrder,
    O: Size,
{
    type Item = Ref<T, E, O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let start = self.start;
        self.start += size_of::<T>();
        Some(Ref::new(start))
    }
}

impl<T, E, O> DoubleEndedIterator for Iter<T, E, O>
where
    T: ZeroCopy,
    E: ByteOrder,
    O: Size,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        self.end -= size_of::<T>();
        Some(Ref::new(self.end))
    }
}

impl<T, E, O> Ref<T, E, O>
where
    T: ?Sized + Pointee,
    E: ByteOrder,
    O: Size,
{
    /// The number of elements in the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let slice = Ref::<str>::with_metadata(0, 10);
    /// assert_eq!(slice.metadata(), 10);
    /// ```
    #[inline]
    pub fn metadata(self) -> T::Stored<O> {
        self.metadata
    }
}

impl<T, E, O> Ref<T, E, O>
where
    T: Pointee<Metadata = (), Stored<O> = ()>,
    E: ByteOrder,
    O: Size,
{
    /// Construct a reference at the given offset.
    ///
    /// # Errors
    ///
    /// This will not compile through a constant assertion if the `offset` or
    /// can't be byte swapped as per [`ZeroCopy::CAN_SWAP_BYTES`].
    ///
    /// # Panics
    ///
    /// This will panic if:
    /// * Packed [`offset()`] cannot be constructed from `U` (out of range).
    ///
    /// [`offset()`]: Self::offset
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference = Ref::<u64>::new(42);
    /// assert_eq!(reference.offset(), 42);
    /// ```
    ///
    /// Characters cannot be used as offsets:
    ///
    /// ```compile_fail
    /// use musli_zerocopy::Ref;
    ///
    /// let reference = Ref::<_, _, char>::new('a');
    /// ```
    #[inline]
    pub fn new<U>(offset: U) -> Self
    where
        U: Copy + fmt::Debug,
        O: TryFrom<U>,
    {
        const {
            assert!(
                O::CAN_SWAP_BYTES,
                "Offset cannot be byte-ordered since it would not inhabit valid types",
            );
        }

        let Some(offset) = O::try_from(offset).ok() else {
            panic!("Offset {offset:?} not in the legal range 0-{}", O::MAX);
        };

        Self {
            offset: O::swap_bytes::<E>(offset),
            metadata: (),
            _marker: PhantomData,
        }
    }

    /// Construct a typed reference to the zeroeth offset in a buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference = Ref::<u64>::zero();
    /// assert_eq!(reference.offset(), 0);
    /// ```
    #[inline]
    pub const fn zero() -> Self {
        Self {
            offset: O::ZERO,
            metadata: (),
            _marker: PhantomData,
        }
    }
}

impl<T, E, O> Ref<T, E, O>
where
    T: ?Sized + Pointee,
    E: ByteOrder,
    O: Size,
{
    /// Get the offset the reference points to.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference = Ref::<u64>::new(42);
    /// assert_eq!(reference.offset(), 42);
    /// ```
    #[inline]
    pub fn offset(self) -> usize {
        self.offset.as_usize::<E>()
    }

    /// Coerce from one kind of reference to another ensuring that the
    /// destination type `U` is size-compatible.
    ///
    /// This performs metadata conversion if the destination metadata for `U`
    /// differs from `T`, such as for `[u32]` to `[u8]` it would multiply the
    /// length by 4 to ensure that the slice points to an appropriately sized
    /// region.
    ///
    /// If the metadata conversion would overflow, this will wrap around the
    /// numerical bounds or panic for debug builds.
    ///
    /// See [`try_coerce()`] for more documentation, which is also a checked
    /// variant of this method.
    ///
    /// [`try_coerce()`]: Self::try_coerce
    pub fn coerce<U>(self) -> Ref<U, E, O>
    where
        T: Coerce<U>,
        U: ?Sized + Pointee,
    {
        Ref {
            offset: self.offset,
            metadata: T::coerce_metadata(self.metadata),
            _marker: PhantomData,
        }
    }

    /// Try to coerce from one kind of reference to another ensuring that the
    /// destination type `U` is size-compatible.
    ///
    /// This performs metadata conversion if the destination metadata for `U`
    /// differs from `T`, such as for `[u32]` to `[u8]` it would multiply the
    /// length by 4 to ensure that the slice points to an appropriately sized
    /// region.
    ///
    /// This returns `None` in case metadata would overflow due to the
    /// conversion.
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference: Ref<u64> = Ref::zero();
    /// let reference2 = reference.coerce::<[u32]>();
    /// assert_eq!(reference2.len(), 2);
    /// ```
    ///
    /// This method ensures that coercions across inappropriate types are
    /// prohibited, such as coercing from a reference to a slice which is too
    /// large.
    ///
    /// ```compile_fail
    /// use musli_zerocopy::Ref;
    ///
    /// let reference: Ref<u32> = Ref::zero();
    /// let reference2 = reference.coerce::<[u64]>();
    /// ```
    ///
    /// If metadata needs to be adjusted for the destination type such as for
    /// slices, it will be:
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference: Ref<[u32]> = Ref::with_metadata(0, 1);
    /// let reference2 = reference.try_coerce::<[u8]>().ok_or("bad coercion")?;
    /// assert_eq!(reference2.len(), 4);
    ///
    /// let reference: Ref<str> = Ref::with_metadata(0, 12);
    /// let reference2 = reference.try_coerce::<[u8]>().ok_or("bad coercion")?;
    /// assert_eq!(reference2.len(), 12);
    /// # Ok::<_, &'static str>(())
    /// ```
    ///
    /// This does mean that numerical overflow might occur if the packed
    /// metadata is too small:
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    /// use musli_zerocopy::endian::Native;
    ///
    /// let reference = Ref::<[u32], Native, u8>::with_metadata(0, 64);
    /// let reference2 = reference.try_coerce::<[u8]>();
    /// assert!(reference2.is_none()); // 64 * 4 would overflow u8 packed metadata.
    /// ```
    ///
    /// Coercion of non-zero types are supported, but do not guarantee that the
    /// destination data is valid.
    pub fn try_coerce<U>(self) -> Option<Ref<U, E, O>>
    where
        T: Coerce<U>,
        U: ?Sized + Pointee,
    {
        Some(Ref {
            offset: self.offset,
            metadata: T::try_coerce_metadata(self.metadata)?,
            _marker: PhantomData,
        })
    }

    #[cfg(test)]
    pub(crate) fn cast<U>(self) -> Ref<U, E, O>
    where
        U: ?Sized + Pointee<Stored<O> = T::Stored<O>>,
    {
        Ref {
            offset: self.offset,
            metadata: self.metadata,
            _marker: PhantomData,
        }
    }
}

impl<T, const N: usize, E, O> Ref<[T; N], E, O>
where
    T: ZeroCopy,
    E: ByteOrder,
    O: Size,
{
    /// Coerce a reference to an array into a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = buf.store(&[1, 2, 3, 4]);
    /// let slice = values.array_into_slice();
    ///
    /// assert_eq!(buf.load(slice)?, &[1, 2, 3, 4]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn array_into_slice(self) -> Ref<[T], E, O> {
        Ref::with_metadata(self.offset, N)
    }
}

impl<T, E, O> Ref<MaybeUninit<T>, E, O>
where
    T: Pointee,
    E: ByteOrder,
    O: Size,
{
    /// Assume that the reference is initialized.
    ///
    /// Unlike the counterpart in Rust, this isn't actually unsafe. Because in
    /// order to load the reference again we'd have to validate it anyways.
    #[inline]
    pub const fn assume_init(self) -> Ref<T, E, O> {
        Ref {
            offset: self.offset,
            metadata: self.metadata,
            _marker: PhantomData,
        }
    }
}

impl<T, E, O> fmt::Debug for Ref<T, E, O>
where
    T: ?Sized + Pointee<Stored<O>: fmt::Debug>,
    E: ByteOrder,
    O: Size + fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ref<{}> {{ offset: {:?}, metadata: {:?} }}",
            core::any::type_name::<T>(),
            self.offset,
            self.metadata,
        )
    }
}

impl<T, E, O> Clone for Ref<T, E, O>
where
    T: ?Sized + Pointee,
    E: ByteOrder,
    O: Size,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E, O> Copy for Ref<T, E, O>
where
    T: ?Sized + Pointee,
    E: ByteOrder,
    O: Size,
{
}

impl<T, E, O> PartialEq for Ref<T, E, O>
where
    T: ?Sized + Pointee<Stored<O>: PartialEq>,
    E: ByteOrder,
    O: PartialEq + Size,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.metadata == other.metadata
    }
}

impl<T, E, O> Eq for Ref<T, E, O>
where
    T: ?Sized + Pointee<Stored<O>: Eq>,
    E: ByteOrder,
    O: Eq + Size,
{
}

impl<T, E, O> PartialOrd for Ref<T, E, O>
where
    T: ?Sized + Pointee<Stored<O>: PartialOrd>,
    E: ByteOrder,
    O: Ord + Size,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.offset.partial_cmp(&other.offset) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        self.metadata.partial_cmp(&other.metadata)
    }
}

impl<T, E, O> Ord for Ref<T, E, O>
where
    T: ?Sized + Pointee<Stored<O>: Ord>,
    E: ByteOrder,
    O: Ord + Size,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.offset.cmp(&other.offset) {
            Ordering::Equal => {}
            ord => return ord,
        }

        self.metadata.cmp(&other.metadata)
    }
}

impl<T, E, O> Hash for Ref<T, E, O>
where
    T: ?Sized + Pointee<Stored<O>: Hash>,
    E: ByteOrder,
    O: Hash + Size,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        self.metadata.hash(state);
    }
}
