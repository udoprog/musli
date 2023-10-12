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
/// let slice = buf.store_slice(&[1, 2, 3, 4])?;
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
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct Slice<T, O: Size = DefaultSize>
where
    T: ZeroCopy,
{
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
    /// let slice = buf.store_slice(&[1, 2, 3, 4])?;
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

impl<T, O: Size> Clone for Slice<T, O>
where
    T: ZeroCopy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, O: Size> Copy for Slice<T, O> where T: ZeroCopy {}
