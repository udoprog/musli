use core::alloc::Layout;
use core::hash::Hash;
use core::mem::align_of;
use core::ptr;
use core::slice;

use ::alloc::alloc;
use ::alloc::vec::Vec;

use crate::buf::AnyRef;
use crate::buf::Buf;
use crate::error::{Error, ErrorKind};
use crate::map::MapRef;
use crate::pair::Pair;
use crate::ptr::Ptr;
use crate::ref_::Ref;
use crate::slice_ref::SliceRef;
use crate::unsized_ref::UnsizedRef;
use crate::zero_copy::{UnsizedZeroCopy, ZeroCopy};

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
    /// let buf = buf.as_aligned_buf();
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_unsized<T>(&mut self, value: &T) -> Result<UnsizedRef<T>, Error>
    where
        T: ?Sized + UnsizedZeroCopy,
    {
        let ptr = self.ptr(T::ALIGN);
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
    /// let buf = buf.as_aligned_buf();
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
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{Error, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let mut values = Vec::new();
    ///
    /// values.push(buf.insert_unsized("first")?);
    /// values.push(buf.insert_unsized("second")?);
    ///
    /// let slice_ref = buf.insert_slice(&values)?;
    ///
    /// let buf = buf.as_aligned_buf();
    ///
    /// let slice = buf.load(slice_ref)?;
    ///
    /// let mut strings = Vec::new();
    ///
    /// for value in slice {
    ///     strings.push(buf.load(*value)?);
    /// }
    ///
    /// assert_eq!(&strings, &["first", "second"][..]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_slice<T>(&mut self, values: &[T]) -> Result<SliceRef<T>, Error>
    where
        T: ZeroCopy,
    {
        let ptr = self.ptr(T::ALIGN);

        for value in values {
            value.write_to(self)?;
        }

        Ok(SliceRef::new(ptr, values.len()))
    }

    /// Insert a map into the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, Pair};
    ///
    /// let mut values = Vec::new();
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// values.push(Pair::new(buf.insert_unsized("first")?, 1u32));
    /// values.push(Pair::new(buf.insert_unsized("second")?, 2u32));
    ///
    /// let values = buf.insert_map(&mut values)?;
    ///
    /// let buf = buf.as_aligned_buf();
    ///
    /// assert_eq!(values.get(buf, &"first")?, Some(&1));
    /// assert_eq!(values.get(buf, &"second")?, Some(&2));
    /// assert_eq!(values.get(buf, &"third")?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_map<K, V>(&mut self, entries: &mut [Pair<K, V>]) -> Result<MapRef<K, V>, Error>
    where
        K: AnyRef + ZeroCopy,
        V: ZeroCopy,
        K::Target: Hash + Eq,
    {
        let mut hash_state = {
            let buf = self.as_aligned_buf();
            crate::map::generator::generate_hash(buf, entries)?
        };

        for a in 0..hash_state.map.len() {
            loop {
                let b = hash_state.map[a];

                if hash_state.map[a] != a {
                    entries.swap(a, b);
                    hash_state.map.swap(a, b);
                    continue;
                }

                break;
            }
        }

        let entries = self.insert_slice(entries)?;

        let mut displacements = Vec::new();

        for (a, b) in hash_state.displacements {
            displacements.push(Pair { a, b });
        }

        let displacements = self.insert_slice(&displacements)?;
        Ok(MapRef::new(hash_state.key, entries, displacements))
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
    ///
    /// Note that this only extends the underlying buffer but does not ensure
    /// that any required alignment is abided by.
    ///
    /// To do this, the caller must call [`align_buf`] with the appropriate
    /// alignment, otherwise the necessary alignment to decode the buffer again
    /// will be lost.
    ///
    /// # Errors
    ///
    /// This is a raw API, and does not guarantee that any given alignment will
    /// be respected. The following exemplifies incorrect use since the u32 type
    /// required a 4-byte alignment::
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, Ref};
    ///
    /// let mut buf = OwnedBuf::with_alignment(1);
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.ptr(1));
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its intereior alignment:
    /// let buf = buf.as_buf()?;
    ///
    /// // This will fail, because the buffer is not aligned.
    /// assert!(buf.load(ptr).is_err());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, Ref};
    ///
    /// let mut buf = OwnedBuf::with_alignment(1);
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.ptr(4));
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its intereior alignment:
    /// let buf = buf.as_buf()?;
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
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

    /// Return a cloned variant of this buffer that is aligned per its
    /// [`requested()`] alignment.
    ///
    /// [`requested()`]: OwnedBuf::requested
    pub fn as_aligned_owned_buf(&self) -> Self {
        if !self.is_aligned_to(self.requested) {
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
    ///
    /// # Errors
    ///
    /// This will fail if the buffer isn't aligned per it's [`requested()`]
    /// alignment.
    ///
    /// [`requested()`]: OwnedBuf::requested
    pub fn as_buf(&self) -> Result<&Buf, Error> {
        if !self.is_aligned_to(self.requested) {
            return Err(Error::new(ErrorKind::BadAlignment {
                ptr: self.data.as_ptr() as usize,
                align: self.requested,
            }));
        }

        // SAFETY: alignment has been checked above.
        Ok(unsafe { Buf::new_unchecked(self.as_slice()) })
    }

    /// Unchecked conversion into a [`Buf`].
    ///
    /// # Safety
    ///
    /// The caller must themselves ensure that the current buffer is aligned as
    /// per its required [`requested()`].
    ///
    /// [`requested()`]: OwnedBuf::requested
    pub unsafe fn as_buf_unchecked(&self) -> &Buf {
        Buf::new_unchecked(self.as_slice())
    }

    /// Convert the current buffer into an aligned buffer and return the aligned
    /// buffer.
    ///
    /// If [`requested()`] does not equal [`align()`] this will cause the buffer
    /// to be reallocated before it is returned.
    ///
    /// [`as_buf`]: OwnedBuf::as_buf
    pub fn as_aligned_buf(&mut self) -> &Buf {
        // SAFETY: We're ensuring that the requested alignment is being abided.
        unsafe {
            if self.requested != self.align {
                let (old_layout, layout) = self.layouts(self.capacity);
                self.reallocate(old_layout, layout, self.capacity);
            }

            self.as_buf_unchecked()
        }
    }

    /// Test if the current allocation uses the specified allocation.
    ///
    /// # Panics
    ///
    /// Panics if the specified alignment is not a power of two.
    ///
    /// ```should_panic
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// buf.is_aligned_to(0);
    /// ```
    pub fn is_aligned_to(&self, align: usize) -> bool {
        is_aligned_to(self.data.as_ptr(), align)
    }

    fn layouts(&self, new_capacity: usize) -> (Layout, Layout) {
        // SAFETY: type invariants ensures that alignment is correct.
        unsafe {
            let old_layout = Layout::from_size_align_unchecked(self.capacity, self.align);
            let layout = Layout::from_size_align_unchecked(new_capacity, self.requested);
            (old_layout, layout)
        }
    }

    /// Request that the current buffer should have at least the specified
    /// alignment and zero-initialize the buffer up to the given alignment.
    ///
    /// This will cause the buffer to be re-aligned the next time it's
    /// reallocated or re-alignment is requested through methods such as
    /// [`as_aligned_owned_buf`].
    ///
    /// [`as_aligned_owned_buf`]: OwnedBuf::as_aligned_owned_buf
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

    /// Construct an aligned pointer into the current buffer.
    pub fn ptr(&mut self, align: usize) -> Ptr {
        self.align_buf(align);
        Ptr::new(self.len)
    }

    fn ensure_capacity(&mut self, new_capacity: usize) {
        if self.capacity >= new_capacity {
            return;
        }

        let new_capacity = new_capacity.next_power_of_two().max(self.requested);

        unsafe {
            let (old_layout, layout) = self.layouts(new_capacity);

            if old_layout.size() == 0 {
                self.allocate(layout);
            } else if layout.align() == old_layout.align() {
                self.resize(old_layout, new_capacity);
            } else {
                self.reallocate(old_layout, layout, new_capacity);
            }
        }
    }

    /// Perform the initial allocation with the given layout and capacity.
    fn allocate(&mut self, layout: Layout) {
        unsafe {
            let ptr = alloc::alloc(layout);

            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }

            self.data = ptr::NonNull::new_unchecked(ptr);
            self.capacity = layout.size();
            self.align = self.requested;
        }
    }

    fn reallocate(&mut self, old_layout: Layout, layout: Layout, new_capacity: usize) {
        unsafe {
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

    unsafe fn resize(&mut self, old_layout: Layout, new_capacity: usize) {
        let ptr = alloc::realloc(self.data.as_ptr(), old_layout, new_capacity);

        if ptr.is_null() {
            alloc::handle_alloc_error(old_layout);
        }

        self.data = ptr::NonNull::new_unchecked(ptr);
        self.capacity = new_capacity;
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

pub(crate) fn is_aligned_to(ptr: *const u8, align: usize) -> bool {
    assert!(align.is_power_of_two(), "alignment is not a power-of-two");
    (ptr as usize) & (align - 1) == 0
}
