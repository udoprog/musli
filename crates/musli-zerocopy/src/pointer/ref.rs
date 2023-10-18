use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::marker::PhantomData;
use core::mem::size_of;

use crate::mem::MaybeUninit;
use crate::pointer::{DefaultSize, Pointee, Size};
use crate::ZeroCopy;

/// A sized reference.
///
/// This is used to type a pointer with a [`ZeroCopy`] parameter so that it can
/// be used in combination with [`Buf`] to load the value from a buffer.
///
/// Note that the constructor is safe, because alignment and validation checks
/// happens whenever a value is loaded from a bare buffer.
///
/// [`Buf`]: crate::buf::Buf
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
#[zero_copy(crate, bounds = {T::Packed: ZeroCopy})]
pub struct Ref<T: ?Sized, O: Size = DefaultSize>
where
    T: Pointee<O>,
{
    offset: O,
    metadata: T::Packed,
    #[zero_copy(ignore)]
    _marker: PhantomData<T>,
}

impl<T: ?Sized, O: Size> Ref<T, O>
where
    T: Pointee<O>,
{
    /// Construct a reference with custom metadata.
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
        T::Metadata: fmt::Debug,
        T::Packed: TryFrom<T::Metadata>,
    {
        let Some(offset) = O::try_from(offset).ok() else {
            panic!("Offset {offset:?} not in legal range 0-{}", O::MAX);
        };

        let Some(metadata) = T::Packed::try_from(metadata).ok() else {
            panic!("Metadata {metadata:?} not in legal range 0-{}", O::MAX);
        };

        Self {
            offset,
            metadata,
            _marker: PhantomData,
        }
    }
}

impl<T, O: Size> Ref<[T], O>
where
    T: ZeroCopy,
{
    /// The number of elements in the slice.
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
    pub fn len(&self) -> usize {
        self.metadata.as_usize()
    }

    /// If the slice is empty.
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
    pub fn is_empty(&self) -> bool {
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
    /// let buf = buf.into_aligned();
    ///
    /// let two = slice.get(2).expect("Missing element 2");
    /// assert_eq!(buf.load(two)?, &3);
    ///
    /// assert!(slice.get(4).is_none());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<Ref<T, O>> {
        if index >= self.len() {
            return None;
        }

        let ptr = self
            .offset
            .as_usize()
            .wrapping_add(size_of::<T>().wrapping_mul(index));

        Some(Ref::new(ptr))
    }
}

impl<T: ?Sized, O: Size> Ref<T, O>
where
    T: Pointee<O, Packed = O>,
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
    pub fn metadata(&self) -> T::Packed {
        self.metadata
    }
}

impl<T, O: Size> Ref<T, O>
where
    T: Pointee<O, Packed = ()>,
{
    /// Construct a reference at the given offset.
    ///
    /// # Panics
    ///
    /// Panics if [`TryFrom::try_from`] over the provided offset errors,
    /// indicating that the offset cannot be packed into the required width.
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
        let Some(offset) = O::try_from(offset).ok() else {
            panic!("Offset {offset:?} not in the legal range 0-{}", O::MAX);
        };

        Self {
            offset,
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

impl<T: ?Sized, O: Size> Ref<T, O>
where
    T: Pointee<O>,
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
    pub fn offset(&self) -> usize {
        self.offset.as_usize()
    }

    /// Cast from one kind of reference to another.
    ///
    /// This statically checks that the metadata for the pointers are the same
    /// to prevent casts over completely different references. For now this only
    /// prevents `Ref<T>` to `Ref<[U]>` casts:
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
    pub fn cast<U: ?Sized>(self) -> Ref<U, O>
    where
        U: Pointee<O, Packed = T::Packed>,
    {
        Ref {
            offset: self.offset,
            metadata: self.metadata,
            _marker: PhantomData,
        }
    }
}

impl<T, const N: usize, O: Size> Ref<[T; N], O>
where
    T: ZeroCopy,
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
    /// let buf = buf.into_aligned();
    ///
    /// assert_eq!(buf.load(slice)?, &[1, 2, 3, 4]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn array_into_slice(self) -> Ref<[T], O> {
        Ref::with_metadata(self.offset, N)
    }
}

impl<T, O: Size> Ref<MaybeUninit<T>, O>
where
    T: Pointee<O>,
{
    /// Assume that the reference is initialized.
    ///
    /// Unlike the counterpart in Rust, this isn't actually unsafe. Because in
    /// order to load the reference again we'd have to validate it anyways.
    #[inline]
    pub const fn assume_init(self) -> Ref<T, O> {
        Ref {
            offset: self.offset,
            metadata: self.metadata,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized, O: Size> fmt::Debug for Ref<T, O>
where
    T: Pointee<O>,
    T::Packed: fmt::Debug,
    O: fmt::Debug,
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

impl<T: ?Sized, O: Size> Clone for Ref<T, O>
where
    T: Pointee<O>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized, O: Size> Copy for Ref<T, O> where T: Pointee<O> {}

impl<T: ?Sized, O: Size> PartialEq for Ref<T, O>
where
    T: Pointee<O>,
    T::Packed: PartialEq,
    O: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.metadata == other.metadata
    }
}

impl<T: ?Sized, O: Size> Eq for Ref<T, O>
where
    T: Pointee<O>,
    T::Packed: Eq,
    O: Eq,
{
}

impl<T: ?Sized, O: Size> PartialOrd for Ref<T, O>
where
    T: Pointee<O>,
    T::Packed: PartialOrd,
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

impl<T: ?Sized, O: Size> Ord for Ref<T, O>
where
    T: Pointee<O>,
    T::Packed: Ord,
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

impl<T: ?Sized, O: Size> Hash for Ref<T, O>
where
    T: Pointee<O>,
    T::Packed: Hash,
    O: Hash,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        self.metadata.hash(state);
    }
}
