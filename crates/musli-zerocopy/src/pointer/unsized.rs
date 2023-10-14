use core::marker::PhantomData;

use crate::pointer::{DefaultSize, Ref, Size};
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
/// buf.extend_from_slice(b"Hello World!");
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
#[zero_copy(crate, skip_visit)]
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

impl<T, O: Size> Unsized<[T], O>
where
    [T]: UnsizedZeroCopy,
    T: ZeroCopy,
{
    /// Get the length of the unsized slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// let unsize = buf.store_unsized(&b"abcd"[..]);
    ///
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(unsize.len(), 4);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        self.size.as_usize()
    }

    /// Test if the slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// let unsize = buf.store_unsized(&b""[..]);
    /// assert!(unsize.is_empty());
    ///
    /// let unsize = buf.store_unsized(&b"abcd"[..]);
    ///
    /// let buf = buf.as_aligned();
    ///
    /// assert!(!unsize.is_empty());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.size.is_zero()
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
    /// let unsize = buf.store_unsized(&b"abcd"[..]);
    ///
    /// let buf = buf.as_aligned();
    ///
    /// let two = unsize.get(2).expect("missing element 2");
    /// assert_eq!(buf.load(two)?, &b'c');
    ///
    /// assert!(unsize.get(4).is_none());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    pub fn get(&self, index: usize) -> Option<Ref<T, O>> {
        if index >= self.len() {
            return None;
        }

        let ptr = self.offset().wrapping_add(index.wrapping_mul(<[T]>::SIZE));
        Some(Ref::new(ptr))
    }
}

impl<T: ?Sized + UnsizedZeroCopy, O: Size> Clone for Unsized<T, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + UnsizedZeroCopy, O: Size> Copy for Unsized<T, O> {}
