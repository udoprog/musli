//! Fixed containers.
//!
//! These can be used to store or reference a fixed amount of data, usually on
//! the stack.

use core::fmt;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

use crate::Context;
use crate::alloc::Vec;
use crate::writer::Writer;

/// A fixed-size bytes storage which keeps track of how much has been
/// initialized.
pub struct FixedBytes<const N: usize> {
    /// Data storage.
    data: [MaybeUninit<u8>; N],
    /// How many bytes have been initialized.
    init: usize,
}

impl<const N: usize> FixedBytes<N> {
    /// Construct a new fixed bytes array storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<128>::new();
    /// assert_eq!(buffer.len(), 0);
    /// assert!(buffer.is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            // SAFETY: MaybeUnint::uninit_array is not stable.
            data: unsafe { MaybeUninit::<[MaybeUninit<u8>; N]>::uninit().assume_init() },
            init: 0,
        }
    }

    /// Construct a fixed bytes while asserting that the given runtime capacity isn't violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let buffer = FixedBytes::<128>::with_capacity(64);
    /// assert_eq!(buffer.len(), 0);
    /// assert_eq!(buffer.remaining(), 128);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the requested capacity is larger than `N`.
    ///
    /// ```should_panic
    /// use musli::fixed::FixedBytes;
    ///
    /// // This will panic
    /// let _buffer = FixedBytes::<10>::with_capacity(20);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(
            capacity <= N,
            "Requested capacity {capacity} is larger than {N}"
        );
        Self::new()
    }

    /// Get the length of the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<10>::new();
    /// assert_eq!(buffer.len(), 0);
    /// buffer.push(42);
    /// assert_eq!(buffer.len(), 1);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.init
    }

    /// Check if the current container is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<10>::new();
    /// assert!(buffer.is_empty());
    /// buffer.push(42);
    /// assert!(!buffer.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.init == 0
    }

    /// Clear the [FixedBytes] container.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<10>::new();
    /// buffer.push(42);
    /// assert_eq!(buffer.len(), 1);
    ///
    /// buffer.clear();
    /// assert_eq!(buffer.len(), 0);
    /// assert!(buffer.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.init = 0;
    }

    /// Get the remaining capacity of the [FixedBytes].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<10>::new();
    /// assert_eq!(buffer.remaining(), 10);
    ///
    /// buffer.push(42);
    /// assert_eq!(buffer.remaining(), 9);
    /// ```
    #[inline]
    pub const fn remaining(&self) -> usize {
        N.saturating_sub(self.init)
    }

    /// Coerce into the underlying bytes if all of them have been initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// // Converting a full buffer returns the complete array
    /// let mut buffer = FixedBytes::<3>::new();
    /// buffer.extend_from_slice(&[1, 2, 3]);
    ///
    /// let bytes = buffer.into_bytes().expect("Buffer should be full");
    /// assert_eq!(bytes, [1, 2, 3]);
    ///
    /// // Partial buffers cannot be converted to arrays
    /// let partial_buffer = FixedBytes::<3>::new();
    /// assert_eq!(partial_buffer.into_bytes(), None);
    /// ```
    #[inline]
    pub fn into_bytes(self) -> Option<[u8; N]> {
        if self.init == N {
            // SAFETY: All of the bytes in the sequence have been initialized
            // and can be safety transmuted.
            //
            // Method of transmuting comes from the implementation of
            // `MaybeUninit::array_assume_init` which is not yet stable.
            unsafe { Some((&self.data as *const _ as *const [u8; N]).read()) }
        } else {
            None
        }
    }

    /// Coerce into the slice of initialized memory which is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<10>::new();
    /// buffer.extend_from_slice(&[1, 2, 3]);
    ///
    /// let slice = buffer.as_slice();
    /// assert_eq!(slice, &[1, 2, 3]);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        if self.init == 0 {
            return &[];
        }

        // SAFETY: We've asserted that `initialized` accounts for the number of
        // bytes that have been initialized.
        unsafe { slice::from_raw_parts(self.data.as_ptr().cast(), self.init) }
    }

    /// Coerce into the mutable slice of initialized memory which is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<10>::new();
    /// buffer.extend_from_slice(&[1, 2, 3]);
    ///
    /// let slice = buffer.as_mut_slice();
    /// slice[0] = 42;
    /// assert_eq!(buffer.as_slice(), &[42, 2, 3]);
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.init == 0 {
            return &mut [];
        }

        // SAFETY: We've asserted that `initialized` accounts for the number of
        // bytes that have been initialized.
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.init) }
    }

    /// Try and push a single byte.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<3>::new();
    /// assert!(buffer.push(1));
    /// assert!(buffer.push(2));
    /// assert!(buffer.push(3));
    /// // Buffer is full
    /// assert!(!buffer.push(4));
    ///
    /// assert_eq!(buffer.as_slice(), &[1, 2, 3]);
    /// ```
    #[inline]
    pub fn push(&mut self, value: u8) -> bool {
        if N.saturating_sub(self.init) == 0 {
            return false;
        }

        unsafe {
            self.data
                .as_mut_ptr()
                .cast::<u8>()
                .add(self.init)
                .write(value)
        }

        self.init += 1;
        true
    }

    /// Try and extend from the given slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::fixed::FixedBytes;
    ///
    /// let mut buffer = FixedBytes::<10>::new();
    /// assert!(buffer.extend_from_slice(&[1, 2, 3]));
    /// assert!(buffer.extend_from_slice(&[4, 5]));
    /// // Would exceed capacity
    /// assert!(!buffer.extend_from_slice(&[6, 7, 8, 9, 10, 11]));
    ///
    /// assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 5]);
    /// ```
    #[inline]
    pub fn extend_from_slice(&mut self, source: &[u8]) -> bool {
        if source.len() > N.saturating_sub(self.init) {
            return false;
        }

        unsafe {
            let dst = (self.data.as_mut_ptr() as *mut u8).add(self.init);
            ptr::copy_nonoverlapping(source.as_ptr(), dst, source.len());
        }

        self.init = self.init.wrapping_add(source.len());
        true
    }

    /// Try and extend from the given slice.
    /// Write bytes to the buffer using a context for error handling.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::context;
    /// use musli::fixed::FixedBytes;
    ///
    /// let cx = context::new();
    ///
    /// // Writing a few bytes to a buffer with capacity succeeds
    /// let mut buffer = FixedBytes::<10>::new();
    /// buffer.write_bytes(&cx, &[1, 2, 3]).unwrap();
    /// assert_eq!(buffer.as_slice(), &[1, 2, 3]);
    ///
    /// // Writing more data than the buffer capacity fails
    /// let mut small_buffer = FixedBytes::<2>::new();
    /// let result = small_buffer.write_bytes(&cx, &[1, 2, 3]);
    /// assert!(result.is_err());
    /// ```
    #[inline]
    pub fn write_bytes<C>(&mut self, cx: C, source: &[u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        if !self.extend_from_slice(source) {
            return Err(cx.message(FixedBytesOverflow {
                at: self.init,
                additional: source.len(),
                capacity: N,
            }));
        }

        Ok(())
    }
}

impl<const N: usize> Deref for FixedBytes<N> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<const N: usize> DerefMut for FixedBytes<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<const N: usize> Default for FixedBytes<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Writer for FixedBytes<N> {
    type Ok = ();
    type Mut<'this>
        = &'this mut Self
    where
        Self: 'this;

    #[inline]
    fn finish<C>(&mut self, _: C) -> Result<Self::Ok, C::Error>
    where
        C: Context,
    {
        Ok(())
    }

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn extend<C>(&mut self, cx: C, buffer: Vec<u8, C::Allocator>) -> Result<(), C::Error>
    where
        C: Context,
    {
        // SAFETY: the buffer never outlives this function call.
        self.write_bytes(cx, buffer.as_slice())
    }

    #[inline]
    fn write_bytes<C>(&mut self, cx: C, bytes: &[u8]) -> Result<(), C::Error>
    where
        C: Context,
    {
        FixedBytes::write_bytes(self, cx, bytes)?;
        cx.advance(bytes.len());
        Ok(())
    }
}

/// Capacity error raised by trying to write to a [FixedBytes] with no remaining
/// capacity.
#[derive(Debug)]
#[allow(missing_docs)]
#[non_exhaustive]
pub(crate) struct FixedBytesOverflow {
    at: usize,
    additional: usize,
    capacity: usize,
}

impl fmt::Display for FixedBytesOverflow {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FixedBytesOverflow {
            at,
            additional,
            capacity,
        } = self;

        write!(
            f,
            "Tried to write {additional} bytes at {at} with capacity {capacity}"
        )
    }
}
