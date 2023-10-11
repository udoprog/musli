use core::marker::PhantomData;

use crate::offset::{DefaultTargetSize, Offset};
use crate::{TargetSize, ZeroCopy};

/// A reference to a slice packed as a wide pointer.
///
/// This contains a pointer to the first element and the length of the slice.
///
/// # Examples
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::{AlignedBuf, Slice, Offset};
///
/// let mut buf = AlignedBuf::with_alignment(align_of::<u32>());
/// buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
///
/// let buf = buf.as_ref()?;
///
/// let slice = Slice::<u32>::new(Offset::ZERO, 2);
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
/// use musli_zerocopy::{AlignedBuf, Slice, Offset};
///
/// let buf = AlignedBuf::with_alignment(align_of::<()>());
/// let buf = buf.as_ref()?;
///
/// let slice = Slice::<()>::new(Offset::ZERO, 2);
///
/// let expected = [(), ()];
///
/// assert_eq!(buf.load(slice)?, &expected[..]);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct Slice<T: ?Sized, O: TargetSize = DefaultTargetSize> {
    ptr: Offset<O>,
    len: O,
    #[zero_copy(ignore)]
    _marker: PhantomData<T>,
}

impl<T: ?Sized, O: TargetSize> Slice<T, O> {
    /// Construct a new slice reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Slice, Offset};
    ///
    /// let slice = Slice::<u32>::new(Offset::ZERO, 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn new(ptr: Offset<O>, len: usize) -> Self {
        let Some(len) = O::from_usize(len) else {
            panic!("Slice length {len} not in the legal range of 0-{}", O::MAX);
        };

        Self {
            ptr,
            len,
            _marker: PhantomData,
        }
    }

    /// The pointer part of the slice reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Slice, Offset};
    ///
    /// let slice = Slice::<u32>::new(Offset::ZERO, 2);
    /// assert_eq!(slice.ptr(), Offset::ZERO);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn ptr(&self) -> Offset<O> {
        self.ptr
    }

    /// The number of elements in the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Slice, Offset};
    ///
    /// let slice = Slice::<u32>::new(Offset::ZERO, 2);
    /// assert_eq!(slice.len(), 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
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
    /// use musli_zerocopy::{Slice, Offset};
    ///
    /// let slice = Slice::<u32>::new(Offset::ZERO, 0);
    /// assert!(slice.is_empty());
    ///
    /// let slice = Slice::<u32>::new(Offset::ZERO, 2);
    /// assert!(!slice.is_empty());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len.is_zero()
    }
}

impl<T: ?Sized, O: TargetSize> Clone for Slice<T, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized, O: TargetSize> Copy for Slice<T, O> {}
