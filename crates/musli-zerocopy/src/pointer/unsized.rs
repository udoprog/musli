use core::marker::PhantomData;

use crate::pointer::{DefaultSize, Size};
use crate::traits::UnsizedZeroCopy;
use crate::ZeroCopy;

/// A reference to an unsized value packed as a wide pointer.
///
/// The `T` that can be stored in here is determined by [`UnsizedZeroCopy`], is
/// inserted through [`AlignedBuf::store_unsized`], and is represented by this
/// type.
///
/// This contains a pointer to the unsized element and the length of the
/// element.
///
/// [`UnsizedZeroCopy`]: crate::traits::UnsizedZeroCopy
/// [`AlignedBuf::store_unsized`]: crate::buf::AlignedBuf::store_unsized
///
/// # Examples
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::AlignedBuf;
/// use musli_zerocopy::pointer::Unsized;
///
/// let mut buf = AlignedBuf::new();
/// let ptr = buf.next_offset::<u8>();
/// buf.extend_from_slice(b"Hello World!")?;
///
/// let buf = buf.as_ref();
///
/// let bytes = Unsized::<str>::new(ptr, 12);
///
/// assert_eq!(buf.load(bytes)?, "Hello World!");
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct Unsized<T: ?Sized + UnsizedZeroCopy, O: Size = DefaultSize> {
    offset: O,
    size: O,
    #[zero_copy(ignore)]
    _marker: PhantomData<T>,
}

impl<T: ?Sized, O: Size> Unsized<T, O>
where
    T: UnsizedZeroCopy,
{
    /// Construct a new unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Unsized;
    ///
    /// let bytes = Unsized::<str>::new(0, 2);
    /// ```
    pub fn new(offset: usize, size: usize) -> Self {
        let Some(offset) = O::from_usize(offset) else {
            panic!(
                "Unsized offset {offset} not in the legal range of 0-{}",
                O::MAX
            );
        };

        let Some(size) = O::from_usize(size) else {
            panic!("Unsized size {size} not in the legal range of 0-{}", O::MAX);
        };

        Self {
            offset,
            size,
            _marker: PhantomData,
        }
    }

    /// Get the pointer element of the unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Unsized;
    ///
    /// let bytes = Unsized::<str>::new(0, 2);
    /// assert_eq!(bytes.offset(), 0);
    /// ```
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset.as_usize()
    }

    /// Get the size in bytes of the unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::pointer::Unsized;
    ///
    /// let bytes = Unsized::<str>::new(0, 2);
    /// assert_eq!(bytes.size(), 2);
    /// ```
    #[inline]
    pub fn size(&self) -> usize {
        self.size.as_usize()
    }
}

impl<T: ?Sized + UnsizedZeroCopy, O: Size> Clone for Unsized<T, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + UnsizedZeroCopy, O: Size> Copy for Unsized<T, O> {}
