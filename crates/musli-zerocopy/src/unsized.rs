use core::marker::PhantomData;

use crate::ptr::Ptr;
use crate::zero_copy::ZeroCopy;

/// A reference to an unsized value packed as a wide pointer.
///
/// The `T` that can be stored in here is determined by [`UnsizedZeroCopy`], is
/// inserted through [`AlignedBuf::write_unsized`], and is represented by this
/// type.
///
/// This contains a pointer to the unsized element and the length of the
/// element.
///
/// [`UnsizedZeroCopy`]: crate::zero_copy::UnsizedZeroCopy
/// [`AlignedBuf::write_unsized`]: crate::aligned_buf::AlignedBuf::write_unsized
///
/// # Examples
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::{AlignedBuf, Unsized, Ptr};
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
#[zero_copy(crate = crate)]
pub struct Unsized<T: ?Sized> {
    ptr: Ptr,
    size: usize,
    #[zero_copy(ignore)]
    _marker: PhantomData<T>,
}

impl<T: ?Sized> Unsized<T> {
    /// Construct a new unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Unsized, Ptr};
    ///
    /// let bytes = Unsized::<str>::new(Ptr::ZERO, 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn new(ptr: Ptr, len: usize) -> Self {
        Self {
            ptr,
            size: len,
            _marker: PhantomData,
        }
    }

    /// Get the pointer element of the unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Unsized, Ptr};
    ///
    /// let bytes = Unsized::<str>::new(Ptr::ZERO, 2);
    /// assert_eq!(bytes.ptr(), Ptr::ZERO);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn ptr(&self) -> Ptr {
        self.ptr
    }

    /// Get the size in bytes of the unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Unsized, Ptr};
    ///
    /// let bytes = Unsized::<str>::new(Ptr::ZERO, 2);
    /// assert_eq!(bytes.size(), 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T: ?Sized> Clone for Unsized<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Unsized<T> {}
