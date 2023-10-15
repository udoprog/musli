use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::{align_of, size_of, size_of_val, ManuallyDrop};
use core::ptr::NonNull;
use core::slice;

use ::alloc::alloc;

use crate::buf::{Buf, BufMut, DefaultAlignment, Padder};
use crate::mem::MaybeUninit;
use crate::pointer::{DefaultSize, Ref, Size, Slice, Unsized};
use crate::traits::{UnsizedZeroCopy, ZeroCopy};

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
/// struct Custom { field: u32 }
///
/// let mut buf = AlignedBuf::new();
/// buf.store(&Custom { field: 10 });
/// ```
pub struct AlignedBuf<O: Size = DefaultSize> {
    data: NonNull<u8>,
    /// The initialized length of the buffer.
    len: usize,
    /// The capacity of the buffer.
    capacity: usize,
    /// The requested alignment.
    requested: usize,
    /// The current alignment.
    align: usize,
    /// Holding onto the current pointer size.
    _marker: PhantomData<O>,
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
        Self::with_alignment::<DefaultAlignment>()
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
        Self::with_capacity_and_alignment::<DefaultAlignment>(capacity)
    }

    /// Construct a new empty buffer with the specified alignment.
    ///
    /// The alignment will be rounded up to the next power of two.
    ///
    /// # Panics
    ///
    /// Panics if the specified alignment is not a power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::with_alignment::<u64>();
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.align(), 8);
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub const fn with_alignment<T>() -> Self
    where
        T: ZeroCopy,
    {
        let align = align_of::<T>();

        Self {
            // SAFETY: Alignment is asserted through `T`.
            data: unsafe { dangling(align) },
            len: 0,
            capacity: 0,
            requested: align,
            align,
            _marker: PhantomData,
        }
    }
}

impl<O: Size> AlignedBuf<O> {
    /// Allocate a new buffer with the given capacity and default alignment.
    ///
    /// The buffer must allocate for at least the given `capacity`, but might
    /// allocate more. If the capacity specified is `0` it will not allocate.
    ///
    /// This constructor also allows for specifying the [`Size`] through the `O`
    /// parameter.
    ///
    /// The available [`Size`] implementations are:
    /// * `u32` for 32-bit sized pointers (the default).
    /// * `usize` for target-dependently sized pointers.
    ///
    /// To initialize an [`AlignedBuf`] with a custom [`Size`] you simply use
    /// this constructor while specifying one of the above parameters:
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::buf::DefaultAlignment;
    ///
    /// let mut buf = AlignedBuf::<usize>::with_capacity_and_alignment::<DefaultAlignment>(0);
    /// ```
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
    /// let max = isize::MAX as usize - (8 - 1);
    /// AlignedBuf::<u32>::with_capacity_and_alignment::<u64>(max);
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let buf = AlignedBuf::<u32>::with_capacity_and_alignment::<u16>(6);
    /// assert!(buf.capacity() >= 6);
    /// assert_eq!(buf.align(), 2);
    /// ```
    pub fn with_capacity_and_alignment<T>(capacity: usize) -> Self
    where
        T: ZeroCopy,
    {
        // SAFETY: Alignment of `T` is always a power of two.
        unsafe { Self::with_capacity_and_custom_alignment(capacity, align_of::<T>()) }
    }

    // # Safety
    //
    // The specified alignment must be a power of two.
    pub(crate) unsafe fn with_capacity_and_custom_alignment(capacity: usize, align: usize) -> Self where
    {
        if capacity == 0 {
            return Self {
                // SAFETY: Alignment is asserted through `T`.
                data: dangling(align),
                len: 0,
                capacity: 0,
                requested: align,
                align,
                _marker: PhantomData,
            };
        }

        let layout = Layout::from_size_align(capacity, align).expect("Illegal memory layout");

        unsafe {
            let data = alloc::alloc(layout);

            if data.is_null() {
                alloc::handle_alloc_error(layout);
            }

            Self {
                data: NonNull::new_unchecked(data),
                len: 0,
                capacity,
                requested: align,
                align,
                _marker: PhantomData,
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
    /// let buf = AlignedBuf::with_alignment::<u64>();
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
    /// let buf = AlignedBuf::with_alignment::<u64>();
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.align(), 8);
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub fn align(&self) -> usize {
        self.align
    }

    /// Reserve capacity for at least `capacity` more bytes in this buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// assert_eq!(buf.capacity(), 0);
    ///
    /// buf.reserve(10);
    /// assert!(buf.capacity() >= 10);
    /// ```
    pub fn reserve(&mut self, capacity: usize) {
        let new_capacity = self.len.wrapping_add(capacity);
        self.ensure_capacity(new_capacity);
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

    /// Store an uninitialized value.
    ///
    /// This allows values to be inserted before they can be initialized, which
    /// can be useful if you need them to be in a certain location in the buffer
    /// but don't have access to their fields yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::mem::MaybeUninit;
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    /// use musli_zerocopy::pointer::{Ref, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Unsized<str> }
    ///
    /// let mut buf = AlignedBuf::new();
    /// let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();
    ///
    /// let string = buf.store_unsized("Hello World!");
    ///
    /// buf.load_uninit_mut(reference).write(&Custom { field: 42, string });
    ///
    /// let reference = reference.assume_init();
    /// assert_eq!(reference.offset(), 0);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store_uninit<T>(&mut self) -> Ref<MaybeUninit<T>, O>
    where
        T: ZeroCopy,
    {
        // SAFETY: We've just reserved capacity for this write.
        unsafe {
            let len = self.next_offset_with(align_of::<T>(), size_of::<T>());
            self.data.as_ptr().add(len).write_bytes(0, size_of::<T>());
            self.len = self.len.wrapping_add(size_of::<T>());
            Ref::new_raw(len)
        }
    }

    /// Write a reference that might not have been initialized.
    ///
    /// This does not prevent [`Ref`] from different instances of [`AlignedBuf`]
    /// from being written. It would only result in potentially garbled data,
    /// but wouldn't be a safety issue.
    ///
    /// # Panics
    ///
    /// Panics if the reference [`Ref::offset()`] and size of `T` does not fit
    /// within the [`len()`] of the current structure. This might happen if you
    /// try and use a reference constructed from a different [`AlignedBuf`]
    /// instance.
    ///
    /// [`len()`]: Self::len()
    ///
    /// ```should_panic
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf1 = AlignedBuf::new();
    /// buf1.store(&1u32);
    ///
    /// let mut buf2 = AlignedBuf::new();
    /// buf2.store(&10u32);
    ///
    /// let number = buf2.store_uninit::<u32>();
    ///
    /// buf1.load_uninit_mut(number);
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    /// use musli_zerocopy::pointer::{Ref, Unsized};
    /// use musli_zerocopy::mem::MaybeUninit;
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Unsized<str> }
    ///
    /// let mut buf = AlignedBuf::new();
    /// let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();
    ///
    /// let string = buf.store_unsized("Hello World!");
    ///
    /// buf.load_uninit_mut(reference).write(&Custom { field: 42, string });
    ///
    /// let reference = reference.assume_init();
    /// assert_eq!(reference.offset(), 0);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn load_uninit_mut<T>(&mut self, reference: Ref<MaybeUninit<T>>) -> &mut MaybeUninit<T>
    where
        T: ZeroCopy,
    {
        let at = reference.offset();

        // Note: We only need this as debug assertion, because `MaybeUninit<T>`
        // does not implement `ZeroCopy`, so there is no way to construct.
        assert!(
            at.wrapping_add(size_of::<T>()) <= self.len,
            "Capacity overflow"
        );

        unsafe { &mut *(self.data.as_ptr().add(at) as *mut MaybeUninit<T>) }
    }

    /// Insert a value with the given size.
    ///
    /// To get the pointer where the value will be written, call
    /// [`next_offset<T>()`] before writing it.
    ///
    /// [`next_offset<T>()`]: Self::next_offset
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{ZeroCopy, AlignedBuf};
    /// use musli_zerocopy::pointer::Unsized;
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Unsized<str> }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let string = buf.store_unsized("string");
    /// let custom = buf.store(&Custom { field: 1, string });
    /// let custom2 = buf.store(&Custom { field: 2, string });
    ///
    /// let buf = buf.as_aligned();
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
    ///
    /// Storing an array:
    ///
    ///
    /// ```
    /// use musli_zerocopy::{ZeroCopy, AlignedBuf};
    ///
    /// // Element with padding.
    /// #[derive(Debug, PartialEq, ZeroCopy)]
    /// #[repr(C)]
    /// struct Element {
    ///     first: u8,
    ///     second: u32,
    /// }
    ///
    /// let values = [
    ///     Element { first: 0x01, second: 0x01020304u32 },
    ///     Element { first: 0x02, second: 0x01020304u32 }
    /// ];
    ///
    /// let mut buf = AlignedBuf::new();
    /// let array = buf.store(&values);
    /// let buf = buf.as_aligned();
    /// assert_eq!(buf.load(array)?, &values);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store<T>(&mut self, value: &T) -> Ref<T, O>
    where
        T: ZeroCopy,
    {
        // SAFETY: We're ensuring that these elements are interacted with
        // correctly.
        unsafe {
            let ptr = self.next_offset_with(align_of::<T>(), size_of::<T>());
            T::store_to(value, self);
            Ref::new(ptr)
        }
    }

    /// Write a value to the buffer.
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let first = buf.store_unsized("first");
    /// let second = buf.store_unsized("second");
    ///
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store_unsized<T>(&mut self, value: &T) -> Unsized<T, O>
    where
        T: ?Sized + UnsizedZeroCopy,
    {
        unsafe {
            let ptr = self.next_offset_with(T::ALIGN, value.bytes());
            value.store_to(self);
            Unsized::new(ptr, value.size())
        }
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
    /// values.push(buf.store_unsized("first"));
    /// values.push(buf.store_unsized("second"));
    ///
    /// let slice_ref = buf.store_slice(&values);
    ///
    /// let buf = buf.as_aligned();
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
    #[inline]
    pub fn store_slice<T>(&mut self, values: &[T]) -> Slice<T, O>
    where
        T: ZeroCopy,
    {
        let ptr = self.store_array(values);
        Slice::new(ptr, values.len())
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
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let mut buf = AlignedBuf::with_alignment::<u32>();
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u8>());
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its interior alignment:
    /// let buf = buf.as_ref();
    ///
    /// // This will fail, because the buffer is not aligned.
    /// assert!(buf.load(ptr).is_err());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let mut buf = AlignedBuf::with_alignment::<()>();
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u32>());
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its interior alignment:
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.reserve(bytes.len());

        // SAFETY: We just allocated space for the slice.
        unsafe {
            self.store_bytes(bytes);
        }
    }

    /// Fill and initialize the buffer with `byte` up to `len`.
    pub(crate) fn fill(&mut self, byte: u8, len: usize) {
        self.reserve(len);

        let base = self.data.as_ptr().wrapping_add(self.len);

        unsafe {
            base.write_bytes(byte, len);
            self.len += len;
        }
    }

    /// Store the slice without allocating.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer has the capacity for
    /// `bytes.len()` and that the value being stored is not padded as per
    /// `ZeroCopy::PADDED`.
    #[inline]
    pub unsafe fn store_bytes<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        let dst = self.as_ptr_mut().wrapping_add(self.len);
        dst.copy_from_nonoverlapping(values.as_ptr().cast(), size_of_val(values));
        self.len = self.len.wrapping_add(size_of_val(values));
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
    #[inline]
    pub fn as_aligned_owned_buf(&self) -> Self {
        // SAFETY: Alignment of `requested` is always a power of two.
        let mut new = unsafe { Self::with_capacity_and_custom_alignment(self.len, self.requested) };

        unsafe {
            new.as_ptr_mut()
                .copy_from_nonoverlapping(self.as_ptr(), self.len);
            new.set_len(self.len);
        }

        new
    }

    /// Convert the current buffer into an aligned buffer and return the aligned
    /// buffer.
    ///
    /// If [`requested()`] does not equal [`align()`] this will cause the buffer
    /// to be reallocated before it is returned.
    ///
    /// [`requested()`]: Self::requested
    /// [`align()`]: Self::align
    /// [`as_ref`]: Self::as_ref
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::with_alignment::<()>();
    /// let number = buf.store(&1u32);
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(buf.load(number)?, &1u32);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn as_aligned(&mut self) -> &Buf {
        if self.requested > self.align {
            let (old_layout, new_layout) = self.layouts(self.capacity);
            self.alloc_new(old_layout, new_layout);
        }

        Buf::new(self.as_slice())
    }

    /// Convert the current buffer into an aligned mutable buffer and return the
    /// aligned buffer.
    ///
    /// If [`requested()`] does not equal [`align()`] this will cause the buffer
    /// to be reallocated before it is returned.
    ///
    /// [`requested()`]: Self::requested
    /// [`align()`]: Self::align
    /// [`as_ref`]: Self::as_ref
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::with_alignment::<()>();
    /// let number = buf.store(&1u32);
    /// let buf = buf.as_mut_aligned();
    ///
    /// *buf.load_mut(number)? += 1;
    /// assert_eq!(buf.load(number)?, &2u32);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn as_mut_aligned(&mut self) -> &mut Buf {
        if self.requested > self.align {
            let (old_layout, new_layout) = self.layouts(self.capacity);
            self.alloc_new(old_layout, new_layout);
        }

        Buf::new_mut(self.as_mut_slice())
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
    /// buf.request_align::<u32>();
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 0, 0]);
    /// ```
    ///
    /// Calling this function only causes the underlying buffer to be realigned
    /// if a reallocation is triggered due to reaching its [`capacity()`].
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// let mut buf = AlignedBuf::<u32>::with_capacity_and_alignment::<u16>(32);
    ///
    /// buf.extend_from_slice(&[1, 2]);
    /// assert_eq!(buf.align(), 2);
    /// buf.request_align::<u32>();
    ///
    /// assert_eq!(buf.requested(), 4);
    /// assert_eq!(buf.align(), 2);
    ///
    /// buf.extend_from_slice(&[0; 32]);
    /// assert_eq!(buf.requested(), 4);
    /// assert_eq!(buf.align(), 4);
    /// ```
    ///
    /// [`capacity()`]: Self::capacity
    /// [`as_aligned_owned_buf()`]: Self::as_aligned_owned_buf
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the alignment is a power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// buf.request_align::<u64>();
    /// buf.extend_from_slice(&[5, 6, 7, 8]);
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 0, 0, 0, 0, 5, 6, 7, 8]);
    /// ```
    #[inline]
    pub fn request_align<T>(&mut self)
    where
        T: ZeroCopy,
    {
        self.requested = self.requested.max(align_of::<T>());
        self.ensure_aligned(align_of::<T>(), size_of::<T>());
    }

    #[inline]
    fn store_bits<T>(&mut self, value: *const T)
    where
        T: ZeroCopy,
    {
        let len = self.len.wrapping_add(size_of::<T>());
        self.ensure_capacity(len);

        let start = self.as_ptr_mut().wrapping_add(self.len);

        unsafe {
            start.copy_from_nonoverlapping(value.cast(), size_of::<T>());
        }

        self.len = len;
    }

    #[inline]
    unsafe fn store_struct<T>(&mut self, value: *const T) -> Padder<'_, T>
    where
        T: ZeroCopy,
    {
        let len = self.len.wrapping_add(size_of::<T>());
        self.ensure_capacity(len);

        let start = self.as_ptr_mut().wrapping_add(self.len);

        // This is what makes calling `store_struct` unsafe, we're preemptively
        // pretending that the buffer has been initialized, while in reality
        // that is the job of the caller.
        self.len = len;

        unsafe {
            start.copy_from_nonoverlapping(value.cast(), size_of::<T>());
        }

        Padder::new(start)
    }

    /// Write a [`ZeroCopy`] value directly into the buffer.
    ///
    /// If you want to know the pointer where this value will be written, use
    /// `next_offset::<T>()` before calling this function.
    #[inline]
    unsafe fn store_inner<T>(&mut self, value: &T)
    where
        T: ZeroCopy,
    {
        self.request_align::<T>();
        T::store_to(value, self);
    }

    #[inline]
    fn ensure_aligned(&mut self, align: usize, reserve: usize) {
        let extra = crate::buf::padding_to(self.len, align);
        self.reserve(extra.wrapping_add(reserve));

        // SAFETY: The length is ensures to be within the address space.
        unsafe {
            self.data.as_ptr().add(self.len).write_bytes(0, extra);
            self.len = self.len.wrapping_add(extra);
        }
    }

    /// Construct a pointer aligned for `align` into the current buffer which
    /// points to the next location that will be written.
    #[inline]
    pub(crate) unsafe fn next_offset_with(&mut self, align: usize, reserve: usize) -> usize {
        self.requested = self.requested.max(align);
        self.ensure_aligned(align, reserve);
        self.len
    }

    /// Construct a pointer aligned for `T` into the current buffer which points
    /// to the next location that will be written.
    ///
    /// This ensures that the alignment of the pointer is a multiple of `align`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::pointer::Ref;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u32>());
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// // This will succeed because the buffer follows its interior alignment:
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn next_offset<T>(&mut self) -> usize
    where
        T: ZeroCopy,
    {
        // SAFETY: The alignment of `T` is guaranteed to be a power of two.
        unsafe { self.next_offset_with(align_of::<T>(), 0) }
    }

    #[inline]
    fn ensure_capacity(&mut self, new_capacity: usize) {
        let new_capacity = new_capacity.max(self.requested);

        if self.capacity >= new_capacity {
            return;
        }

        let new_capacity = new_capacity.max((self.capacity as f32 * 1.5) as usize);
        let (old_layout, new_layout) = self.layouts(new_capacity);

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
    #[inline]
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

            self.data = NonNull::new_unchecked(ptr);
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
            self.data = NonNull::new_unchecked(ptr);
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

            ptr.copy_from_nonoverlapping(self.as_ptr(), self.len);
            alloc::dealloc(self.as_ptr_mut(), old_layout);

            // We've deallocated the old pointer.
            self.data = NonNull::new_unchecked(ptr);
            self.capacity = new_layout.size();
            self.align = self.requested;
        }
    }

    fn store_array<T>(&mut self, values: &[T]) -> usize
    where
        T: ZeroCopy,
    {
        // SAFETY: We're interacting with all elements correctly.
        unsafe {
            let size = size_of_val(values);
            let offset = self.next_offset_with(align_of::<T>(), size);

            if T::PADDED {
                for value in values {
                    T::store_to(value, self);
                }
            } else {
                self.data
                    .as_ptr()
                    .add(self.len)
                    .copy_from_nonoverlapping(values.as_ptr().cast::<u8>(), size);

                self.len = self.len.wrapping_add(size);
            }

            offset
        }
    }
}

/// `AlignedBuf` are `Send` because the data they reference is unaliased.
unsafe impl Send for AlignedBuf {}
/// `AlignedBuf` are `Sync` since they are `Send` and the data they reference is
/// unaliased.
unsafe impl Sync for AlignedBuf {}

impl<O: Size> AsRef<Buf> for AlignedBuf<O> {
    /// Access the current buffer immutably while checking that the buffer is
    /// aligned.
    ///
    /// # Errors
    ///
    /// This will fail if the buffer isn't aligned per its [`requested()`]
    /// alignment.
    ///
    /// [`requested()`]: Self::requested
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// let slice = buf.store_unsized("hello world");
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(buf.load(slice)?, "hello world");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn as_ref(&self) -> &Buf {
        Buf::new(self.as_slice())
    }
}

impl<O: Size> AsMut<Buf> for AlignedBuf<O> {
    /// Access the current buffer mutably while checking that the buffer is
    /// aligned.
    ///
    /// # Errors
    ///
    /// This will fail if the buffer isn't aligned per its [`requested()`]
    /// alignment.
    ///
    /// [`requested()`]: Self::requested
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    /// let slice = buf.store_unsized("hello world");
    /// let buf = buf.as_mut();
    ///
    /// buf.load_mut(slice)?.make_ascii_uppercase();
    /// assert_eq!(buf.load(slice)?, "HELLO WORLD");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn as_mut(&mut self) -> &mut Buf {
        Buf::new_mut(self.as_mut_slice())
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
/// let mut buf = AlignedBuf::<u32>::with_capacity_and_alignment::<u16>(32);
/// buf.extend_from_slice(&[1, 2, 3, 4]);
/// buf.request_align::<u32>();
///
/// let buf2 = buf.clone();
/// assert_eq!(buf2.align(), align_of::<u16>());
///
/// let buf3 = buf.as_aligned_owned_buf();
/// assert_eq!(buf3.align(), align_of::<u32>());
/// ```
impl<O: Size> Clone for AlignedBuf<O> {
    fn clone(&self) -> Self {
        unsafe {
            let mut new = ManuallyDrop::new(Self::with_capacity_and_custom_alignment(
                self.len, self.align,
            ));
            new.as_ptr_mut()
                .copy_from_nonoverlapping(self.as_ptr(), self.len);
            // Set requested to the same as original.
            new.requested = self.requested;
            new.set_len(self.len);
            ManuallyDrop::into_inner(new)
        }
    }
}

impl<O: Size> Drop for AlignedBuf<O> {
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

impl<O: Size> BufMut for AlignedBuf<O> {
    #[inline]
    unsafe fn store_bytes<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        AlignedBuf::store_bytes(self, values)
    }

    #[inline]
    unsafe fn store_bits<T>(&mut self, value: *const T)
    where
        T: ZeroCopy,
    {
        AlignedBuf::store_bits(self, value)
    }

    #[inline]
    unsafe fn store<T>(&mut self, value: &T)
    where
        T: ZeroCopy,
    {
        AlignedBuf::store_inner(self, value)
    }

    #[inline]
    unsafe fn store_struct<T>(&mut self, value: *const T) -> Padder<'_, T>
    where
        T: ZeroCopy,
    {
        AlignedBuf::store_struct::<T>(self, value)
    }

    #[inline]
    unsafe fn store_array<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        self.store_array(values);
    }
}

const unsafe fn dangling(align: usize) -> NonNull<u8> {
    NonNull::new_unchecked(invalid_mut(align))
}

// Replace with `core::ptr::invalid_mut` once stable.
#[allow(clippy::useless_transmute)]
const fn invalid_mut<T>(addr: usize) -> *mut T {
    // FIXME(strict_provenance_magic): I am magic and should be a compiler
    // intrinsic. We use transmute rather than a cast so tools like Miri can
    // tell that this is *not* the same as from_exposed_addr. SAFETY: every
    // valid integer is also a valid pointer (as long as you don't dereference
    // that pointer).
    unsafe { core::mem::transmute(addr) }
}
