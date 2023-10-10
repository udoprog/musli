use core::marker::PhantomData;

use crate::offset::{Offset, OffsetSize};
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
/// [`UnsizedZeroCopy`]: crate::zero_copy::UnsizedZeroCopy
/// [`AlignedBuf::store_unsized`]: crate::aligned_buf::AlignedBuf::store_unsized
///
/// # Examples
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::{AlignedBuf, Unsized, Offset};
///
/// let mut buf = AlignedBuf::new();
/// let ptr = buf.next_pointer::<u8>();
/// buf.extend_from_slice(b"Hello World!")?;
///
/// let buf = buf.as_ref()?;
///
/// let bytes = Unsized::<str>::new(ptr, 12);
///
/// assert_eq!(buf.load(bytes)?, "Hello World!");
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct Unsized<T: ?Sized> {
    ptr: Offset,
    size: OffsetSize,
    #[zero_copy(ignore)]
    _marker: PhantomData<T>,
}

impl<T: ?Sized> Unsized<T> {
    /// Construct a new unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Unsized, Offset};
    ///
    /// let bytes = Unsized::<str>::new(Offset::ZERO, 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn new(ptr: Offset, size: usize) -> Self {
        let Ok(size) = OffsetSize::try_from(size) else {
            panic!(
                "Unsized size {size} not in the legal range of 0-{}",
                OffsetSize::MAX
            );
        };

        Self {
            ptr,
            size,
            _marker: PhantomData,
        }
    }

    /// Get the pointer element of the unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Unsized, Offset};
    ///
    /// let bytes = Unsized::<str>::new(Offset::ZERO, 2);
    /// assert_eq!(bytes.ptr(), Offset::ZERO);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn ptr(&self) -> Offset {
        self.ptr
    }

    /// Get the size in bytes of the unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Unsized, Offset};
    ///
    /// let bytes = Unsized::<str>::new(Offset::ZERO, 2);
    /// assert_eq!(bytes.size(), 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn size(&self) -> usize {
        self.size as usize
    }
}

impl<T: ?Sized> Clone for Unsized<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Unsized<T> {}
