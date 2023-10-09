use core::alloc::Layout;
use core::mem::{align_of, replace, ManuallyDrop};
use core::ptr;
use core::slice;

use ::alloc::alloc;
use ::alloc::borrow::Cow;
use ::alloc::vec::Vec;

use crate::buf::{Buf, CowBuf};
use crate::error::{Error, ErrorKind};
use crate::ptr::Ptr;
use crate::ref_::Ref;
use crate::slice_ref::SliceRef;
use crate::unsized_ref::UnsizedRef;
use crate::zero_copy::{SliceZeroCopy, UnsizedZeroCopy, ZeroCopy};

/// An owned buffer.
#[derive(Clone)]
pub struct OwnedBuf {
    data: ptr::NonNull<u8>,
    /// The initialized length of the buffer.
    len: usize,
    /// The capacity of the buffer.
    capacity: usize,
    /// The requested alignment.
    requested: usize,
    /// The current alignment.
    align: usize,
}

impl OwnedBuf {
    /// Construct a new empty buffer with the default alignment.
    ///
    /// The default alignment is guaranteed to be larger than 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// assert!(buf.is_empty());
    /// assert!(buf.align() > 0);
    /// assert_eq!(buf.align(), buf.requested());
    /// ```
    pub const fn new() -> Self {
        Self::with_alignment(align_of::<u64>())
    }

    /// Construct a new empty buffer with the specified alignment.
    ///
    /// The alignment will be rounded up to the next power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::with_alignment(8);
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.align(), 8);
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub const fn with_alignment(align: usize) -> Self {
        let align = align.next_power_of_two();

        Self {
            data: ptr::NonNull::dangling(),
            len: 0,
            capacity: 0,
            requested: align,
            align,
        }
    }

    /// Get the current length of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// assert_eq!(buf.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Test if the buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// assert!(buf.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the current capacity of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// assert_eq!(buf.capacity(), 0);
    /// ```
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Return the requested alignment of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::with_alignment(8);
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.align(), 8);
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub fn requested(&self) -> usize {
        self.requested
    }

    /// Return the current alignment of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::with_alignment(8);
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.align(), 8);
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub fn align(&self) -> usize {
        self.align
    }

    /// Get the current buffer as a slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const _, self.len) }
    }

    /// Write a value to the buffer.
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let first = buf.insert_unsized("first")?;
    /// let second = buf.insert_unsized("second")?;
    ///
    /// let buf = buf.as_buf()?;
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_unsized<T>(&mut self, value: &T) -> Result<UnsizedRef<T>, Error>
    where
        T: ?Sized + UnsizedZeroCopy,
    {
        let ptr = self.ptr(value.align());
        value.write_to(self)?;
        Ok(UnsizedRef::new(ptr, value.len()))
    }

    /// Insert a value with the given size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::ZeroCopy;
    /// use musli_zerocopy::{Error, OwnedBuf, UnsizedRef};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     string: UnsizedRef<str>,
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let string = buf.insert_unsized("string")?;
    /// let custom = buf.insert_sized(Custom { field: 1, string })?;
    /// let custom2 = buf.insert_sized(Custom { field: 2, string })?;
    ///
    /// let buf = buf.as_buf()?;
    ///
    /// let custom = buf.load(custom)?;
    /// assert_eq!(custom.field, 1);
    /// assert_eq!(buf.load(custom.string)?, "string");
    ///
    /// let custom2 = buf.load(custom2)?;
    /// assert_eq!(custom2.field, 2);
    /// assert_eq!(buf.load(custom2.string)?, "string");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_sized<T>(&mut self, value: T) -> Result<Ref<T>, Error>
    where
        T: ZeroCopy,
    {
        let ptr = self.ptr(T::ALIGN);
        value.write_to(self)?;
        Ok(Ref::new(ptr))
    }

    /// Insert a slice into the buffer.
    pub fn insert_slice<T>(&mut self, value: &[T]) -> Result<SliceRef<T>, Error>
    where
        [T]: SliceZeroCopy,
    {
        let ptr = self.ptr(value.align());
        value.write_to(self)?;
        Ok(SliceRef::new(ptr, value.len()))
    }

    /// Write the bytes of a value.
    pub fn write<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        self.align_buf(T::ALIGN);
        value.write_to(self)?;
        Ok(())
    }

    /// Extend the buffer from a slice.
    #[inline]
    pub fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let Some(capacity) = self.capacity.checked_add(bytes.len()) else {
            panic!("Capacity overflow");
        };

        self.ensure_capacity(capacity);

        unsafe {
            let dst = self.data.as_ptr().wrapping_add(self.len);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
            self.len = self.len.wrapping_add(bytes.len());
        }

        Ok(())
    }

    /// Return a cloned variant of this buffer that is aligned.
    pub fn as_aligned(&self) -> Self {
        if (self.data.as_ptr() as usize) % self.requested == 0 {
            return self.clone();
        }

        assert!(
            self.requested.is_power_of_two(),
            "Alignment has to be a power of two, but was {}",
            self.requested
        );

        unsafe {
            let layout = Layout::from_size_align_unchecked(self.len, self.requested);

            let ptr = alloc::alloc(layout);

            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }

            ptr::copy_nonoverlapping(self.data.as_ptr(), ptr, self.len);

            Self {
                data: ptr::NonNull::new_unchecked(ptr),
                len: self.len,
                capacity: self.len,
                requested: self.requested,
                align: self.requested,
            }
        }
    }

    /// Access the current buffer for reading.
    pub fn as_buf(&self) -> Result<&Buf, Error> {
        if (self.data.as_ptr() as usize) % self.requested != 0 {
            return Err(Error::new(ErrorKind::BadAlignment {
                ptr: self.data.as_ptr() as usize,
                align: self.requested,
            }));
        }

        Ok(Buf::new(self.as_slice()))
    }

    /// Unchecked conversion into a [`Buf`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the current buffer is aligned as per its
    /// required [`requested()`].
    ///
    /// [`requested()`]: OwnedBuf::requested
    pub(crate) unsafe fn as_buf_unchecked(&self) -> &Buf {
        Buf::new(self.as_slice())
    }

    /// Coerce the current buffer into an aligned buffer. This is a non-fallible
    /// variant of [`as_buf`].
    ///
    /// This might require the current buffer to be reallocated.
    ///
    /// [`as_buf`]: OwnedBuf::as_buf
    pub fn as_aligned_buf(&self) -> CowBuf<'_> {
        if (self.data.as_ptr() as usize) % self.requested == 0 {
            CowBuf::borrowed(Buf::new(self.as_slice()))
        } else {
            // SAFETY: as_aligned ensures that the buffer is aligned.
            unsafe { CowBuf::owned(self.as_aligned()) }
        }
    }

    /// Request that the current buffer should have at least the specified
    /// alignment and zero-initialize the buffer up to the given alignment.
    ///
    /// This will cause the buffer to be re-aligned the next time it's
    /// reallocated or re-alignment is requested through methods such as
    /// [`as_aligned`].
    ///
    /// [`as_aligned`]: OwnedBuf::as_aligned
    ///
    /// # Panics
    ///
    /// Panics if the specified alignment is not a power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// buf.align_buf(8);
    /// buf.extend_from_slice(&[5, 6, 7, 8]);
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 0, 0, 0, 0, 5, 6, 7, 8]);
    /// ```
    pub fn align_buf(&mut self, align: usize) {
        assert!(
            align.is_power_of_two(),
            "Alignment has to be a power of two"
        );

        let len = self.len.next_multiple_of(align);
        self.requested = self.requested.max(align);

        if len > self.len {
            self.ensure_capacity(len);

            // Zero-initialize the buffer expansion.
            //
            // SAFETY: We've ensured that the capacity is valid by calling
            // `ensure_capacity`.
            unsafe {
                ptr::write_bytes(self.data.as_ptr().wrapping_add(self.len), 0, len - self.len);
            }

            self.len = len;
        }
    }

    /// Get an aligned pointer.
    pub(crate) fn ptr(&mut self, align: usize) -> Ptr {
        self.align_buf(align);
        Ptr::new(self.len)
    }

    fn ensure_capacity(&mut self, new_capacity: usize) {
        if self.capacity >= new_capacity {
            return;
        }

        let new_capacity = new_capacity.next_power_of_two().max(self.requested);

        unsafe {
            let layout = Layout::from_size_align_unchecked(new_capacity, self.requested);
            let old_layout = Layout::from_size_align_unchecked(self.capacity, self.align);

            if old_layout.size() == 0 {
                let ptr = alloc::alloc(layout);

                if ptr.is_null() {
                    alloc::handle_alloc_error(layout);
                }

                self.data = ptr::NonNull::new_unchecked(ptr);
                self.capacity = new_capacity;
                self.align = self.requested;
            } else if layout.align() == old_layout.align() {
                let ptr = alloc::realloc(self.data.as_ptr(), old_layout, new_capacity);

                if ptr.is_null() {
                    alloc::handle_alloc_error(layout);
                }

                self.data = ptr::NonNull::new_unchecked(ptr);
                self.capacity = new_capacity;
            } else {
                let ptr = alloc::alloc(layout);

                if ptr.is_null() {
                    alloc::handle_alloc_error(layout);
                }

                ptr::copy_nonoverlapping(self.data.as_ptr(), ptr, self.len);

                alloc::dealloc(self.data.as_ptr(), old_layout);

                self.data = ptr::NonNull::new_unchecked(ptr);
                self.capacity = new_capacity;
                self.align = self.requested;
            }
        }
    }

    /// Swap two pointer positions.
    ///
    /// The signature and calculation performed to swap guarantees that the
    /// elements do not overlap.
    pub(crate) fn swap(&mut self, base: usize, a: usize, b: usize, len: usize) {
        if a == b {
            return;
        }

        macro_rules! bounds_check {
            ($var:ident) => {
                assert! {
                    $var.wrapping_add(len) <= self.len,
                    "range {}-{} out of bounds 0-{}",
                    $var,
                    $var.wrapping_add(len),
                    self.len
                };
            };
        }

        let a = base.wrapping_add(a.wrapping_mul(len));
        let b = base.wrapping_add(b.wrapping_mul(len));

        bounds_check!(a);
        bounds_check!(b);

        let d = self.data.as_ptr();

        // SAFETY: We've checked that the pointers are in bounds. The signature
        // of the function guarantees that the slices are non-overlapping.
        unsafe {
            ptr::swap_nonoverlapping(d.wrapping_add(a), d.wrapping_add(b), len);
        }
    }
}

impl Drop for OwnedBuf {
    fn drop(&mut self) {
        unsafe {
            if self.capacity != 0 {
                let layout = Layout::from_size_align_unchecked(self.capacity, self.align);
                alloc::dealloc(self.data.as_ptr(), layout);
            }
        }
    }
}
