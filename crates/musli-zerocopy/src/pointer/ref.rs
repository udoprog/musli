use core::cmp::Ordering;
use core::hash::Hash;
use core::marker::PhantomData;
use core::mem::size_of;
use core::{any, fmt};

use crate::endian::{Big, ByteOrder, Little, Native};
use crate::error::{Error, ErrorKind, IntoRepr};
use crate::mem::MaybeUninit;
use crate::pointer::{DefaultSize, Pointee, Size};
use crate::ZeroCopy;

/// A stored reference to a type `P`.
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
#[zero_copy(crate, swap_bytes, bounds = {P::Packed: ZeroCopy})]
pub struct Ref<P: ?Sized, E: ByteOrder = Native, O: Size = DefaultSize>
where
    P: Pointee<O>,
{
    offset: O,
    metadata: P::Packed,
    #[zero_copy(ignore)]
    _marker: PhantomData<(E, P)>,
}

impl<P: ?Sized, E: ByteOrder, O: Size> Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: ZeroCopy,
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
    pub fn to_be(self) -> Ref<P, Big, O> {
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
    pub fn to_le(self) -> Ref<P, Little, O> {
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
    pub fn to_ne(self) -> Ref<P, Native, O> {
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
    pub fn to_endian<U: ByteOrder>(self) -> Ref<P, U, O> {
        Ref {
            offset: self.offset.transpose_bytes::<E, U>(),
            metadata: self.metadata.transpose_bytes::<E, U>(),
            _marker: PhantomData,
        }
    }
}

impl<P: ?Sized, O: Size, E: ByteOrder> Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: ZeroCopy,
{
    /// Construct a reference with custom metadata.
    ///
    /// # Panics
    ///
    /// This will panic if either:
    /// * The `offset` or `metadata` can't be byte swapped as per
    ///   [`ZeroCopy::CAN_SWAP_BYTES`].
    /// * Packed [`offset()`] cannot be constructed from `U` (out of range).
    /// * Packed [`metadata()`] cannot be constructed from `P::Metadata` (reason
    ///   depends on the exact metadata).
    ///
    /// To guarantee that this constructor will never panic, [`Ref<T, usize,
    /// E>`] can be used.
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
    pub fn with_metadata<U, M>(offset: U, metadata: M) -> Self
    where
        U: Copy + fmt::Debug,
        M: Copy + fmt::Debug,
        O: TryFrom<U>,
        P::Packed: TryFrom<M>,
    {
        assert!(
            O::CAN_SWAP_BYTES,
            "Offset `{}` cannot be byte-ordered since it would not inhabit valid types",
            any::type_name::<O>()
        );

        assert!(
            P::Packed::CAN_SWAP_BYTES,
            "Packed metadata `{}` cannot be byte-ordered since it would not inhabit valid types",
            any::type_name::<P::Metadata>()
        );

        let Some(offset) = O::try_from(offset).ok() else {
            panic!("Offset {offset:?} not in legal range 0-{}", O::MAX);
        };

        let Some(metadata) = P::Packed::try_from(metadata).ok() else {
            panic!("Metadata {metadata:?} not in legal range 0-{}", O::MAX);
        };

        Self {
            offset: O::swap_bytes::<E>(offset),
            metadata: P::Packed::swap_bytes::<E>(metadata),
            _marker: PhantomData,
        }
    }

    /// Fallibly try to construct a reference with metadata.
    ///
    /// # Errors
    ///
    /// This will error if either:
    /// * The `offset` or `metadata` can't be byte swapped as per
    ///   [`ZeroCopy::CAN_SWAP_BYTES`].
    /// * Packed [`offset()`] cannot be constructed from `U` (out of range).
    /// * Packed [`metadata()`] cannot be constructed from `P::Metadata` (reason
    ///   depends on the exact metadata).
    ///
    /// To guarantee that this constructor will never panic, [`Ref<T, usize,
    /// E>`] can be used.
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
    pub fn try_with_metadata<U, M>(offset: U, metadata: M) -> Result<Self, Error>
    where
        U: Copy + IntoRepr + fmt::Debug,
        M: Copy + IntoRepr + fmt::Debug,
        O: TryFrom<U>,
        P::Packed: TryFrom<M>,
    {
        if !O::CAN_SWAP_BYTES {
            return Err(Error::new(ErrorKind::InvalidOffset {
                ty: any::type_name::<O>(),
            }));
        }

        if !P::Packed::CAN_SWAP_BYTES {
            return Err(Error::new(ErrorKind::InvalidMetadata {
                ty: any::type_name::<P::Metadata>(),
                packed: any::type_name::<P::Packed>(),
            }));
        }

        let Some(offset) = O::try_from(offset).ok() else {
            return Err(Error::new(ErrorKind::InvalidOffsetRange {
                offset: U::into_repr(offset),
                max: O::into_repr(O::MAX),
            }));
        };

        let Some(metadata) = P::Packed::try_from(metadata).ok() else {
            return Err(Error::new(ErrorKind::InvalidMetadataRange {
                metadata: M::into_repr(metadata),
                max: O::into_repr(O::MAX),
            }));
        };

        Ok(Self {
            offset: O::swap_bytes::<E>(offset),
            metadata: P::Packed::swap_bytes::<E>(metadata),
            _marker: PhantomData,
        })
    }
}

impl<P, O: Size, E: ByteOrder> Ref<[P], E, O>
where
    P: ZeroCopy,
{
    /// Return the number of elements in the slice `[P]`.
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

    /// Test if the slice `[P]` is empty.
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
    pub fn get(self, index: usize) -> Option<Ref<P, E, O>> {
        if index >= self.len() {
            return None;
        }

        let offset = self.offset.as_usize::<E>() + size_of::<P>() * index;
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
    pub fn get_unchecked(self, index: usize) -> Ref<P, E, O> {
        let offset = self.offset.as_usize::<E>() + size_of::<P>() * index;
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
        let b = Self::with_metadata(offset + at * size_of::<P>(), len - at);
        (a, b)
    }

    /// Perform an fetch like `get` which panics with diagnostics in case the
    /// index is out-of-bounds.
    #[inline]
    #[cfg(feature = "alloc")]
    pub(crate) fn at(self, index: usize) -> Ref<P, E, O> {
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
    pub fn iter(self) -> Iter<P, E, O> {
        let start = self.offset.as_usize::<E>();
        let end = start + self.metadata.as_usize::<E>() * size_of::<P>();

        Iter {
            start,
            end,
            _marker: PhantomData,
        }
    }
}

impl<O: Size, E: ByteOrder> Ref<str, E, O> {
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

    /// Test if the slice `[P]` is empty.
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

/// An iterator over a `Ref<[P]>` which produces `Ref<P>` values.
///
/// See [Ref::<[P]>::iter].
pub struct Iter<P, E, O> {
    start: usize,
    end: usize,
    _marker: PhantomData<(P, E, O)>,
}

impl<P: ZeroCopy, E: ByteOrder, O: Size> Iterator for Iter<P, E, O> {
    type Item = Ref<P, E, O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let start = self.start;
        self.start += size_of::<P>();
        Some(Ref::new(start))
    }
}

impl<P: ZeroCopy, E: ByteOrder, O: Size> DoubleEndedIterator for Iter<P, E, O> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        self.end -= size_of::<P>();
        Some(Ref::new(self.end))
    }
}

impl<P: ?Sized, O: Size, E: ByteOrder> Ref<P, E, O>
where
    P: Pointee<O>,
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
    pub fn metadata(self) -> P::Packed {
        self.metadata
    }
}

impl<P, O: Size, E: ByteOrder> Ref<P, E, O>
where
    P: Pointee<O, Packed = ()>,
{
    /// Construct a reference at the given offset.
    ///
    /// # Panics
    ///
    /// This will panic if either:
    /// * The `offset` can't be byte swapped as per
    ///   [`ZeroCopy::CAN_SWAP_BYTES`].
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
    #[inline]
    pub fn new<U>(offset: U) -> Self
    where
        U: Copy + fmt::Debug,
        O: TryFrom<U>,
    {
        assert!(
            O::CAN_SWAP_BYTES,
            "Type `{}` cannot be byte-ordered since it would not inhabit valid types",
            any::type_name::<O>()
        );

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

impl<P: ?Sized, O: Size, E: ByteOrder> Ref<P, E, O>
where
    P: Pointee<O>,
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

    /// Cast from one kind of reference to another.
    ///
    /// This statically checks that the metadata for the pointers are the same
    /// to prevent casts over completely different references. For now this only
    /// prevents `Ref<P>` to `Ref<[U]>` casts:
    ///
    /// ```compile_fail
    /// use musli_zerocopy::Ref;
    ///
    /// let reference: Ref<u32> = Ref::zero();
    /// let reference2 = reference.cast::<[u32]>();
    /// ```
    ///
    /// The correct way to do the above would be to explicitly deconstruct the
    /// reference:
    ///
    /// ```
    /// use musli_zerocopy::Ref;
    ///
    /// let reference: Ref<u32> = Ref::zero();
    /// let reference2: Ref<[u32]> = Ref::with_metadata(reference.offset(), 1);
    /// ```
    pub fn cast<U: ?Sized>(self) -> Ref<U, E, O>
    where
        U: Pointee<O, Packed = P::Packed>,
    {
        Ref {
            offset: self.offset,
            metadata: self.metadata,
            _marker: PhantomData,
        }
    }
}

impl<P, const N: usize, O: Size, E: ByteOrder> Ref<[P; N], E, O>
where
    P: ZeroCopy,
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
    pub fn array_into_slice(self) -> Ref<[P], E, O> {
        Ref::with_metadata(self.offset, N)
    }
}

impl<P, O: Size, E: ByteOrder> Ref<MaybeUninit<P>, E, O>
where
    P: Pointee<O>,
{
    /// Assume that the reference is initialized.
    ///
    /// Unlike the counterpart in Rust, this isn't actually unsafe. Because in
    /// order to load the reference again we'd have to validate it anyways.
    #[inline]
    pub const fn assume_init(self) -> Ref<P, E, O> {
        Ref {
            offset: self.offset,
            metadata: self.metadata,
            _marker: PhantomData,
        }
    }
}

impl<P: ?Sized, O: Size, E: ByteOrder> fmt::Debug for Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: fmt::Debug,
    O: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ref<{}> {{ offset: {:?}, metadata: {:?} }}",
            core::any::type_name::<P>(),
            self.offset,
            self.metadata,
        )
    }
}

impl<P: ?Sized, O: Size, E: ByteOrder> Clone for Ref<P, E, O>
where
    P: Pointee<O>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ?Sized, O: Size, E: ByteOrder> Copy for Ref<P, E, O> where P: Pointee<O> {}

impl<P: ?Sized, O: Size, E: ByteOrder> PartialEq for Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: PartialEq,
    O: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.metadata == other.metadata
    }
}

impl<P: ?Sized, O: Size, E: ByteOrder> Eq for Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: Eq,
    O: Eq,
{
}

impl<P: ?Sized, O: Size, E: ByteOrder> PartialOrd for Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: PartialOrd,
    O: Ord,
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

impl<P: ?Sized, O: Size, E: ByteOrder> Ord for Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: Ord,
    O: Ord,
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

impl<P: ?Sized, O: Size, E: ByteOrder> Hash for Ref<P, E, O>
where
    P: Pointee<O>,
    P::Packed: Hash,
    O: Hash,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        self.metadata.hash(state);
    }
}
