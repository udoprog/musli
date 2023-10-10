use core::alloc::Layout;
use core::hash::Hash;
use core::mem::align_of;
use core::mem::ManuallyDrop;
use core::ops::Range;
use core::ptr;
use core::slice;

use ::alloc::alloc;
use ::alloc::vec::Vec;

use crate::buf::{AnyValue, Buf, BufMut};
use crate::error::{Error, ErrorKind};
use crate::map::MapRef;
use crate::pair::Pair;
use crate::ptr::Ptr;
use crate::r#ref::Ref;
use crate::r#unsized::Unsized;
use crate::slice::Slice;
use crate::zero_copy::{UnsizedZeroCopy, ZeroCopy};

/// Default alignment to use with [`AlignedBuf`].
const DEFAULT_ALIGNMENT: usize = align_of::<usize>();

/// An allocating buffer with dynamic alignment.
///
/// By default this buffer starts out having the same alignment as `usize`,
/// making it platform specific. But this alignment can grow in demand to the
/// types being used.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{AlignedBuf, ZeroCopy};
///
/// #[derive(ZeroCopy)]
/// #[repr(C, align(128))]
/// struct Custom {
///     field: u32,
/// }
///
/// let mut buf = AlignedBuf::new();
/// buf.write(&Custom { field: 10 });
/// ```
pub struct AlignedBuf {
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

impl AlignedBuf {
    /// Construct a new empty buffer with the default alignment.
    ///
    /// The default alignment is guaranteed to be larger than 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::new();
    /// assert!(buf.is_empty());
    /// assert!(buf.align() > 0);
    /// assert_eq!(buf.align(), buf.requested());
    /// ```
    pub const fn new() -> Self {
        Self::with_alignment(DEFAULT_ALIGNMENT)
    }

    /// Construct a new empty buffer with the specified alignment.
    ///
    /// The alignment will be rounded up to the next power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::with_alignment(8);
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

    /// Allocate a new buffer with the given capacity and default alignment.
    ///
    /// The buffer must allocate for at least the given `capacity`, but might
    /// allocate more. If the capacity specified is `0` it will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::with_capacity(6);
    /// assert!(buf.capacity() >= 6);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_alignment(capacity, DEFAULT_ALIGNMENT)
    }

    /// Allocate a new buffer with the given capacity and default alignment.
    ///
    /// The buffer must allocate for at least the given `capacity`, but might
    /// allocate more. If the capacity specified is `0` it will not allocate.
    ///
    /// # Panics
    ///
    /// Panics if the specified capacity and memory layout are illegal, which
    /// happens if:
    /// * The alignment is not a power of two.
    /// * The specified capacity causes the needed memory to overflow
    ///   `isize::MAX`.
    ///
    /// ```should_panic
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let align = 8usize;
    /// let max = isize::MAX as usize - (align - 1);
    ///
    /// AlignedBuf::with_capacity_and_alignment(max, align);
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::with_capacity_and_alignment(6, 2);
    /// assert!(buf.capacity() >= 6);
    /// assert_eq!(buf.align(), 2);
    /// ```
    pub fn with_capacity_and_alignment(capacity: usize, align: usize) -> Self {
        if capacity == 0 {
            return Self::with_alignment(align);
        }

        let layout = Layout::from_size_align(capacity, align).expect("Illegal memory layout");

        unsafe {
            let data = alloc::alloc(layout);

            if data.is_null() {
                alloc::handle_alloc_error(layout);
            }

            Self {
                data: ptr::NonNull::new_unchecked(data),
                len: 0,
                capacity,
                requested: align,
                align,
            }
        }
    }

    /// Get the current length of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::new();
    /// assert_eq!(buf.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Set the initialized length of this buffer.
    ///
    /// # Safety
    ///
    /// The buffer must be allocated and initialized up to the given length.
    /// Failure to abide by this will result in safe APIs exhibiting undefined
    /// behavior.
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    /// Clear the current buffer.
    ///
    /// This won't cause any reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut b = AlignedBuf::new();
    /// assert_eq!(b.capacity(), 0);
    /// b.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// assert_eq!(b.len(), 4);
    /// b.clear();
    /// assert!(b.capacity() > 0);
    /// assert_eq!(b.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Test if the buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::new();
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
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::new();
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
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::with_alignment(8);
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
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::with_alignment(8);
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.align(), 8);
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub fn align(&self) -> usize {
        self.align
    }

    /// Get get a raw pointer to the current buffer.
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr() as *const _
    }

    /// Get get a raw mutable pointer to the current buffer.
    pub fn as_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_ptr()
    }

    /// Get the current buffer as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut b = AlignedBuf::new();
    /// b.extend_from_slice(b"hello world");
    /// assert_eq!(b.as_slice(), b"hello world");
    /// ```
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    /// Get the current buffer as a mutable slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut b = AlignedBuf::new();
    /// b.extend_from_slice(b"hello world");
    /// b.as_mut_slice().make_ascii_uppercase();
    /// assert_eq!(b.as_slice(), b"HELLO WORLD");
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.as_ptr_mut(), self.len()) }
    }

    /// Write a value to the buffer.
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
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
    pub fn insert_unsized<T>(&mut self, value: &T) -> Result<Unsized<T>, Error>
    where
        T: ?Sized + UnsizedZeroCopy,
    {
        let ptr = self.next_pointer(T::ALIGN);
        value.write_to(self)?;
        Ok(Unsized::new(ptr, value.len()))
    }

    /// Insert a value with the given size.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::ZeroCopy;
    /// use musli_zerocopy::{AlignedBuf, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     string: Unsized<str>,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
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
        let ptr = self.next_pointer(align_of::<T>());
        value.write_to(self)?;
        Ok(Ref::new(ptr))
    }

    /// Insert a slice into the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
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
    pub fn insert_slice<T>(&mut self, values: &[T]) -> Result<Slice<T>, Error>
    where
        T: ZeroCopy,
    {
        let ptr = self.next_pointer(align_of::<T>());

        for value in values {
            value.write_to(self)?;
        }

        Ok(Slice::new(ptr, values.len()))
    }

    /// Insert a map into the buffer.
    ///
    /// This will utilize a perfect hash functions derived from the [`phf`
    /// crate] to construt a persistent hash map.
    ///
    /// This returns a [`MapRef`] which can be bound into a [`Map`] through the
    /// [`bind()`] method for convenience.
    ///
    /// [`phf` crate]: https://crates.io/crates/phf
    /// [`Map`]: crate::Map
    /// [`bind()`]: Buf::bind
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut pairs = Vec::new();
    ///
    /// pairs.push(Pair::new(buf.insert_unsized("first")?, 1u32));
    /// pairs.push(Pair::new(buf.insert_unsized("second")?, 2u32));
    ///
    /// let map = buf.insert_map(&mut pairs)?;
    /// let buf = buf.as_aligned_buf();
    /// let map = buf.bind(map)?;
    ///
    /// assert_eq!(map.get(&"first")?, Some(&1));
    /// assert_eq!(map.get(&"second")?, Some(&2));
    /// assert_eq!(map.get(&"third")?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// Using non-references as keys:
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut pairs = Vec::new();
    ///
    /// pairs.push(Pair::new(10u64, 1u32));
    /// pairs.push(Pair::new(20u64, 2u32));
    ///
    /// let map = buf.insert_map(&mut pairs)?;
    /// let buf = buf.as_aligned_buf();
    ///
    /// assert_eq!(map.get(buf, &10u64)?, Some(&1));
    /// assert_eq!(map.get(buf, &20u64)?, Some(&2));
    /// assert_eq!(map.get(buf, &30u64)?, None);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn insert_map<K, V>(&mut self, entries: &mut [Pair<K, V>]) -> Result<MapRef<K, V>, Error>
    where
        K: AnyValue + ZeroCopy,
        V: ZeroCopy,
        K::Target: Hash,
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
        self.request_align(align_of::<T>());
        value.write_to(self)?;
        Ok(())
    }

    /// Extend the buffer from a slice.
    ///
    /// Note that this only extends the underlying buffer but does not ensure
    /// that any required alignment is abided by.
    ///
    /// To do this, the caller must call [`request_align()`] with the appropriate
    /// alignment, otherwise the necessary alignment to decode the buffer again
    /// will be lost.
    ///
    /// [`request_align()`]: Self::request_align
    ///
    /// # Errors
    ///
    /// This is a raw API, and does not guarantee that any given alignment will
    /// be respected. The following exemplifies incorrect use since the u32 type
    /// required a 4-byte alignment:
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, Ref};
    ///
    /// let mut buf = AlignedBuf::with_alignment(4);
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_pointer(1));
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its interior alignment:
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
    /// use musli_zerocopy::{AlignedBuf, Ref};
    ///
    /// let mut buf = AlignedBuf::with_alignment(1);
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_pointer(4));
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its interior alignment:
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
            let dst = self.as_ptr_mut().wrapping_add(self.len);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
            self.len = self.len.wrapping_add(bytes.len());
        }

        Ok(())
    }

    /// Return a cloned variant of this buffer that is aligned per its
    /// [`requested()`] alignment.
    ///
    /// [`requested()`]: Self::requested
    ///
    /// # Panics
    ///
    /// This panics if the proposed layout is not valid as per
    /// [`Layout::from_size_align`] using `len()` and `requested()` as
    /// parameters.
    ///
    /// [`len()`]: Self::len
    /// [`requested()`]: Self::requested
    pub fn as_aligned_owned_buf(&self) -> Self {
        let mut new = Self::with_capacity_and_alignment(self.len, self.requested);

        unsafe {
            ptr::copy_nonoverlapping(self.as_ptr(), new.as_ptr_mut(), self.len);
            new.set_len(self.len);
        }

        new
    }

    /// Access the current buffer for reading.
    ///
    /// # Errors
    ///
    /// This will fail if the buffer isn't aligned per it's [`requested()`]
    /// alignment.
    ///
    /// [`requested()`]: Self::requested
    pub fn as_buf(&self) -> Result<&Buf, Error> {
        if !self.is_aligned_to(self.requested) {
            return Err(Error::new(ErrorKind::AlignmentMismatch {
                range: self.range(),
                align: self.requested,
            }));
        }

        Ok(Buf::new(self.as_slice()))
    }

    /// Unchecked conversion into a [`Buf`].
    ///
    /// # Safety
    ///
    /// The caller must themselves ensure that the current buffer is aligned as
    /// per its required [`requested()`].
    ///
    /// [`requested()`]: Self::requested
    pub unsafe fn as_buf_unchecked(&self) -> &Buf {
        Buf::new(self.as_slice())
    }

    /// Convert the current buffer into an aligned buffer and return the aligned
    /// buffer.
    ///
    /// If [`requested()`] does not equal [`align()`] this will cause the buffer
    /// to be reallocated before it is returned.
    ///
    /// [`requested()`]: Self::requested
    /// [`align()`]: Self::align
    /// [`as_buf`]: Self::as_buf
    pub fn as_aligned_buf(&mut self) -> &Buf {
        // SAFETY: We're ensuring that the requested alignment is being abided.
        unsafe {
            if self.requested != self.align {
                let (old_layout, new_layout) = self.layouts(self.capacity);
                self.alloc_new(old_layout, new_layout);
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
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::new();
    /// buf.is_aligned_to(0);
    /// ```
    #[inline]
    pub fn is_aligned_to(&self, align: usize) -> bool {
        crate::buf::is_aligned_to(self.as_ptr(), align)
    }

    /// Request that the current buffer should have at least the specified
    /// alignment and zero-initialize the buffer up to the next position which
    /// matches the given alignment.
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// let mut buf = AlignedBuf::new();
    ///
    /// buf.extend_from_slice(&[1, 2]);
    /// buf.request_align(4);
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 0, 0]);
    /// ```
    ///
    /// Calling this function only causes the underlying buffer to be realigned
    /// if a reallocation is triggered due to reaching its [`capacity()`].
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// let mut buf = AlignedBuf::with_capacity_and_alignment(4, 2);
    ///
    /// buf.extend_from_slice(&[1, 2]);
    /// buf.request_align(4);
    ///
    /// assert_eq!(buf.requested(), 4);
    /// assert_eq!(buf.align(), 2);
    ///
    /// buf.extend_from_slice(&[1, 2, 3]);
    /// assert_eq!(buf.requested(), 4);
    /// assert_eq!(buf.align(), 4);
    /// ```
    ///
    /// [`capacity()`]: Self::capacity
    /// [`as_aligned_owned_buf()`]: Self::as_aligned_owned_buf
    ///
    /// # Panics
    ///
    /// Panics if the specified alignment is not a power of two.
    ///
    /// ```should_panic
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// buf.request_align(3);
    /// ````
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// buf.request_align(8);
    /// buf.extend_from_slice(&[5, 6, 7, 8]);
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 0, 0, 0, 0, 5, 6, 7, 8]);
    /// ```
    pub fn request_align(&mut self, align: usize) {
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
                ptr::write_bytes(self.as_ptr_mut().wrapping_add(self.len), 0, len - self.len);
            }

            self.len = len;
        }
    }

    /// Construct an aligned pointer into the current buffer which points to the
    /// next location that will be written.
    ///
    /// This ensures that the alignment of the pointer is a multiple of `align`.
    ///
    /// # Panics
    ///
    /// Panics if the specified alignment is not a power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::mem::align_of;
    /// use musli_zerocopy::{AlignedBuf, Ref};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_pointer(align_of::<u32>()));
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its interior alignment:
    /// let buf = buf.as_buf()?;
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn next_pointer(&mut self, align: usize) -> Ptr {
        self.request_align(align);
        Ptr::new(self.len)
    }

    fn ensure_capacity(&mut self, new_capacity: usize) {
        if self.capacity >= new_capacity {
            return;
        }

        let (old_layout, new_layout) = self.layouts(new_capacity.max(self.requested));

        if old_layout.size() == 0 {
            self.alloc_init(new_layout);
        } else if new_layout.align() == old_layout.align() {
            self.alloc_realloc(old_layout, new_layout);
        } else {
            self.alloc_new(old_layout, new_layout);
        }
    }

    /// Return a pair of the currently allocated layout, and new layout that is
    /// requested with the given capacity.
    fn layouts(&self, new_capacity: usize) -> (Layout, Layout) {
        // SAFETY: The existing layout cannot be invalid since it's either
        // checked as it's replacing the old layout, or is initialized with
        // known good values.
        let old_layout = unsafe { Layout::from_size_align_unchecked(self.capacity, self.align) };
        let layout =
            Layout::from_size_align(new_capacity, self.requested).expect("Proposed layout invalid");
        (old_layout, layout)
    }

    /// Perform the initial allocation with the given layout and capacity.
    fn alloc_init(&mut self, new_layout: Layout) {
        unsafe {
            let ptr = alloc::alloc(new_layout);

            if ptr.is_null() {
                alloc::handle_alloc_error(new_layout);
            }

            self.data = ptr::NonNull::new_unchecked(ptr);
            self.capacity = new_layout.size();
            self.align = self.requested;
        }
    }

    /// Reallocate, note that the alignment of the old layout must match the new
    /// one.
    fn alloc_realloc(&mut self, old_layout: Layout, new_layout: Layout) {
        debug_assert_eq!(old_layout.align(), new_layout.align());

        unsafe {
            let ptr = alloc::realloc(self.as_ptr_mut(), old_layout, new_layout.size());

            if ptr.is_null() {
                alloc::handle_alloc_error(old_layout);
            }

            // NB: We may simply forget the old allocation, since `realloc` is
            // responsible for freeing it.
            self.data = ptr::NonNull::new_unchecked(ptr);
            self.capacity = new_layout.size();
        }
    }

    /// Perform a new allocation, deallocating the old one in the process.
    fn alloc_new(&mut self, old_layout: Layout, new_layout: Layout) {
        unsafe {
            let ptr = alloc::alloc(new_layout);

            if ptr.is_null() {
                alloc::handle_alloc_error(new_layout);
            }

            ptr::copy_nonoverlapping(self.as_ptr(), ptr, self.len);
            alloc::dealloc(self.as_ptr_mut(), old_layout);

            // We've deallocated the old pointer.
            self.data = ptr::NonNull::new_unchecked(ptr);
            self.capacity = new_layout.size();
            self.align = self.requested;
        }
    }

    /// Return representation of the pointer range.
    pub(crate) fn range(&self) -> Range<usize> {
        let end = self.data.as_ptr().wrapping_add(self.len);
        self.data.as_ptr() as usize..end as usize
    }
}

/// Clone the [`AlignedBuf`].
///
/// While this causes another allocation, it doesn't ensure that the returned
/// buffer has the [`requested()`] alignment. To achieve this prefer using
/// [`as_aligned_owned_buf()`].
///
/// [`requested()`]: Self::requested()
/// [`as_aligned_owned_buf()`]: Self::as_aligned_owned_buf
///
/// # Examples
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::AlignedBuf;
///
/// assert_ne!(align_of::<u16>(), align_of::<u32>());
///
/// let mut buf = AlignedBuf::with_capacity_and_alignment(4, align_of::<u16>());
/// buf.extend_from_slice(&[1, 2, 3, 4]);
/// buf.request_align(align_of::<u32>());
///
/// let buf2 = buf.clone();
/// assert_eq!(buf2.align(), align_of::<u16>());
///
/// let buf3 = buf.as_aligned_owned_buf();
/// assert_eq!(buf3.align(), align_of::<u32>());
/// ```
impl Clone for AlignedBuf {
    fn clone(&self) -> Self {
        unsafe {
            let mut new =
                ManuallyDrop::new(Self::with_capacity_and_alignment(self.len, self.align));
            ptr::copy_nonoverlapping(self.as_ptr(), new.as_ptr_mut(), self.len);
            // Set requested to the same as original.
            new.requested = self.requested;
            new.set_len(self.len);
            ManuallyDrop::into_inner(new)
        }
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        unsafe {
            if self.capacity != 0 {
                // SAFETY: This is guaranteed to be valid per the construction
                // of this type.
                let layout = Layout::from_size_align_unchecked(self.capacity, self.align);
                alloc::dealloc(self.as_ptr_mut(), layout);
            }
        }
    }
}

impl BufMut for AlignedBuf {
    #[inline]
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error> {
        AlignedBuf::extend_from_slice(self, bytes)
    }

    #[inline]
    fn write<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        AlignedBuf::write(self, value)
    }
}
