use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::marker::PhantomData;
use core::mem::size_of;

use crate::pointer::{DefaultSize, Ref, Size};
use crate::ZeroCopy;

/// A reference to a slice packed as a wide pointer.
///
/// Slices are stored in buffers through [`AlignedBuf::store_slice`].
///
/// This contains a pointer to the first element and the length of the slice.
///
/// [`AlignedBuf::store_slice`]: crate::buf::AlignedBuf::store_slice
///
/// # Examples
///
/// ```
/// use musli_zerocopy::AlignedBuf;
///
/// let mut buf = AlignedBuf::new();
/// let slice = buf.store_slice(&[1, 2, 3, 4]);
///
/// let buf = buf.as_aligned();
///
/// assert_eq!(buf.load(slice)?, &[1, 2, 3, 4]);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Manually constructing a slice into a buffer:
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::pointer::Slice;
///
/// let mut buf = AlignedBuf::with_alignment::<u32>();
/// buf.extend_from_slice(&[0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]);
///
/// let buf = buf.as_ref();
///
/// let slice = Slice::<u32>::new(4, 2);
///
/// let expected = [
///     u32::from_ne_bytes([1, 2, 3, 4]),
///     u32::from_ne_bytes([5, 6, 7, 8]),
/// ];
///
/// assert_eq!(buf.load(slice)?, &expected[..]);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
///
/// Example using a slice with zero-sized elements:
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::pointer::Slice;
///
/// let buf = AlignedBuf::with_alignment::<()>();
/// let buf = buf.as_ref();
///
/// let slice = Slice::<()>::new(0, 2);
///
/// let expected = [(), ()];
///
/// assert_eq!(buf.load(slice)?, &expected[..]);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct Slice<T, O: Size = DefaultSize> {
    offset: O,
    len: O,
    #[zero_copy(ignore)]
    _marker: PhantomData<T>,
}

impl<T, O: Size> Slice<T, O>
where
    T: ZeroCopy,
{
    /// Construct a new slice reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Slice;
    ///
    /// let slice = Slice::<u32>::new(0, 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn new(offset: usize, len: usize) -> Self {
        let Some(offset) = O::from_usize(offset) else {
            panic!(
                "Slice offset {offset} not in the legal range of 0-{}",
                O::MAX
            );
        };

        let Some(len) = O::from_usize(len) else {
            panic!("Slice length {len} not in the legal range of 0-{}", O::MAX);
        };

        Self {
            offset,
            len,
            _marker: PhantomData,
        }
    }

    /// Coerce an offset and a length directly into a slice.
    pub(crate) fn new_with_offset(offset: O, len: usize) -> Self {
        let Some(len) = O::from_usize(len) else {
            panic!("Slice length {len} not in the legal range of 0-{}", O::MAX);
        };

        Self {
            offset,
            len,
            _marker: PhantomData,
        }
    }

    /// The pointer part of the slice reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Slice;
    ///
    /// let slice = Slice::<u32>::new(0, 2);
    /// assert_eq!(slice.offset(), 0);
    /// ```
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset.as_usize()
    }

    /// The number of elements in the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Slice;
    ///
    /// let slice = Slice::<u32>::new(0, 2);
    /// assert_eq!(slice.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len.as_usize()
    }

    /// If the slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Slice;
    ///
    /// let slice = Slice::<u32>::new(0, 0);
    /// assert!(slice.is_empty());
    ///
    /// let slice = Slice::<u32>::new(0, 2);
    /// assert!(!slice.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len.is_zero()
    }

    /// Try to get a reference directly out of the slice without validation.
    ///
    /// This avoids having to validate every element in a slice in order to
    /// address them.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// let slice = buf.store_slice(&[1, 2, 3, 4]);
    ///
    /// let buf = buf.as_aligned();
    ///
    /// let two = slice.get(2).expect("Missing element 2");
    /// assert_eq!(buf.load(two)?, &3);
    ///
    /// assert!(slice.get(4).is_none());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    pub fn get(&self, index: usize) -> Option<Ref<T, O>> {
        if index >= self.len() {
            return None;
        }

        let ptr = self
            .offset()
            .wrapping_add(size_of::<T>().wrapping_mul(index));

        Some(Ref::new(ptr))
    }
}

impl<T, O: Size> fmt::Debug for Slice<T, O>
where
    O: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Slice<{}> {{ offset: {:?}, len: {:?} }}",
            core::any::type_name::<T>(),
            self.offset,
            self.len
        )
    }
}

impl<T, O: Size> Clone for Slice<T, O>
where
    T: ZeroCopy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, O: Size> Copy for Slice<T, O> where T: ZeroCopy {}

impl<T, O: Size> PartialEq for Slice<T, O>
where
    O: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.len == other.len
    }
}

impl<T, O: Size> Eq for Slice<T, O> where O: Eq {}

impl<T, O: Size> PartialOrd for Slice<T, O>
where
    O: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.offset.partial_cmp(&other.offset) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        self.len.partial_cmp(&other.len)
    }
}

impl<T, O: Size> Ord for Slice<T, O>
where
    O: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.offset.cmp(&other.offset) {
            Ordering::Equal => {}
            ord => return ord,
        }

        self.len.cmp(&other.len)
    }
}

impl<T, O: Size> Hash for Slice<T, O>
where
    O: Hash,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        self.len.hash(state);
    }
}
