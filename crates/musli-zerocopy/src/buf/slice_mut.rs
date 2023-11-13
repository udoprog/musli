use core::borrow::Borrow;
use core::marker::PhantomData;
use core::mem::{align_of, size_of, size_of_val, ManuallyDrop};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::slice::{self, SliceIndex};

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;

use crate::buf::{self, Buf, DefaultAlignment, Padder, StoreBuf};
use crate::endian::{ByteOrder, Native};
use crate::error::Error;
use crate::mem::MaybeUninit;
use crate::pointer::{DefaultSize, Ref, Size};
use crate::traits::{UnsizedZeroCopy, ZeroCopy};

/// A fixed buffer wrapping a `&mut [u8]` with a dynamic alignment.
///
/// By default this buffer starts out having the same alignment as `usize`,
/// making it platform specific. But this alignment can grow in demand to the
/// types being stored in it.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{SliceMut, ZeroCopy};
///
/// #[derive(ZeroCopy)]
/// #[repr(C, align(128))]
/// struct Custom { field: u32 }
///
/// let mut buf = [0; 1024];
/// let mut buf = SliceMut::new(&mut buf);
/// buf.store(&Custom { field: 10 });
/// ```
pub struct SliceMut<'a, E: ByteOrder = Native, O: Size = DefaultSize> {
    /// Base data pointer.
    data: NonNull<u8>,
    /// The initialized length of the buffer.
    len: usize,
    /// The capacity of the buffer.
    capacity: usize,
    /// The requested alignment.
    requested: usize,
    /// Sticky endianness and pointer size.
    _marker: PhantomData<(&'a mut [u8], E, O)>,
}

impl<'a> SliceMut<'a> {
    /// Construct a new empty buffer with a requested default alignment.
    ///
    /// The default alignment is guaranteed to be larger than 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let buf = SliceMut::new(&mut buf);
    /// assert!(buf.is_empty());
    /// ```
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self::with_alignment::<DefaultAlignment>(bytes)
    }

    /// Construct a new empty buffer with the an alignment request matching that
    /// of `T`
    ///
    /// Note that this does not guarantee that the underlying buffer is aligned.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let buf = SliceMut::with_alignment::<u64>(&mut buf);
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub fn with_alignment<T>(bytes: &'a mut [u8]) -> Self {
        let align = align_of::<T>();
        let capacity = bytes.len();

        Self {
            data: unsafe { NonNull::new_unchecked(bytes.as_mut_ptr()) },
            len: 0,
            capacity,
            requested: align,
            _marker: PhantomData,
        }
    }
}

impl<'a, E: ByteOrder, O: Size> SliceMut<'a, E, O> {
    /// Modify the buffer to utilize the specified pointer size when inserting
    /// references.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = SliceMut::new(&mut [0; 16])
    ///     .with_size::<u8>();
    /// ```
    #[inline]
    pub fn with_size<U: Size>(self) -> SliceMut<'a, E, U> {
        let this = ManuallyDrop::new(self);

        SliceMut {
            data: this.data,
            len: this.len,
            capacity: this.capacity,
            requested: this.requested,
            _marker: PhantomData,
        }
    }

    /// Modify the buffer to utilize the specified byte order when inserting
    /// references.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{endian, SliceMut};
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf)
    ///     .with_byte_order::<endian::Little>();
    /// ```
    #[inline]
    pub fn with_byte_order<U: ByteOrder>(self) -> SliceMut<'a, U, O> {
        let this = ManuallyDrop::new(self);

        SliceMut {
            data: this.data,
            len: this.len,
            capacity: this.capacity,
            requested: this.requested,
            _marker: PhantomData,
        }
    }

    /// Get the current length of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let buf = SliceMut::new(&mut buf);
    /// assert_eq!(buf.len(), 0);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Clear the current buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    /// assert_eq!(buf.capacity(), 1024);
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// assert_eq!(buf.len(), 4);
    /// buf.clear();
    /// assert_eq!(buf.capacity(), 1024);
    /// assert_eq!(buf.len(), 0);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Test if the buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let buf = SliceMut::new(&mut buf);
    /// assert!(buf.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the current capacity of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let buf = SliceMut::new(&mut buf);
    /// assert_eq!(buf.capacity(), 1024);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Return the requested alignment of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let buf = SliceMut::with_alignment::<u64>(&mut buf);
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.requested(), 8);
    /// ```
    #[inline]
    pub fn requested(&self) -> usize {
        self.requested
    }

    /// Reserve capacity for at least `capacity` more bytes in this buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    /// assert_eq!(buf.capacity(), 1024);
    ///
    /// buf.reserve(10);
    /// assert!(buf.capacity() >= 10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        let new_capacity = self.len + capacity;
        self.ensure_capacity(new_capacity);
    }

    /// Advance the length of the owned buffer by `size`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that bytes up until `len() + size` has been
    /// initialized in this buffer.
    #[inline]
    pub unsafe fn advance(&mut self, size: usize) {
        self.len += size;
    }

    /// Get get a raw pointer to the current buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr() as *const _
    }

    /// Get get a raw mutable pointer to the current buffer.
    #[inline]
    pub fn as_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_ptr()
    }

    /// Extract a slice containing the entire buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    /// buf.extend_from_slice(b"hello world");
    /// assert_eq!(buf.as_slice(), b"hello world");
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    /// Extract a mutable slice containing the entire buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    /// buf.extend_from_slice(b"hello world");
    /// buf.as_mut_slice().make_ascii_uppercase();
    /// assert_eq!(buf.as_slice(), b"HELLO WORLD");
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.as_ptr_mut(), self.len()) }
    }

    /// Store an uninitialized value.
    ///
    /// This allows values to be inserted before they can be initialized, which
    /// can be useful if you need them to be in a certain location in the buffer
    /// but don't have access to their value yet.
    ///
    /// The memory for `T` will be zero-initialized at [`next_offset<T>()`] and
    /// the length and alignment requirement of `SliceMut` updated to reflect
    /// that an instance of `T` has been stored. But that representation might
    /// not match the representation of `T`[^non-zero].
    ///
    /// To get the offset where the value will be written, call
    /// [`next_offset<T>()`] before storing the value.
    ///
    /// > **Note:** this does not return [`std::mem::MaybeUninit`], instead we
    /// > use an internal [`MaybeUninit`] which is similar but has different
    /// > properties. See [its documentation][MaybeUninit] for more.
    ///
    /// [`next_offset<T>()`]: Self::next_offset()
    /// [^non-zero]: Like with [`NonZero*`][core::num] types.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::mem::MaybeUninit;
    /// use musli_zerocopy::{SliceMut, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
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
    pub fn store_uninit<T>(&mut self) -> Ref<MaybeUninit<T>, E, O>
    where
        T: ZeroCopy,
    {
        // SAFETY: We've just reserved capacity for this write.
        unsafe {
            self.next_offset_with_and_reserve(align_of::<T>(), size_of::<T>());
            let offset = self.len;
            self.data
                .as_ptr()
                .add(self.len)
                .write_bytes(0, size_of::<T>());
            self.len += size_of::<T>();
            Ref::new(offset)
        }
    }

    /// Write a reference that might not have been initialized.
    ///
    /// This does not prevent [`Ref`] from different instances of [`SliceMut`]
    /// from being written. It would only result in garbled data, but wouldn't
    /// be a safety concern.
    ///
    /// > **Note:** this does not return [`std::mem::MaybeUninit`], instead we
    /// > use an internal [`MaybeUninit`] which is similar but has different
    /// > properties. See [its documentation][MaybeUninit] for more.
    ///
    /// # Panics
    ///
    /// Panics if the reference [`Ref::offset()`] and size of `T` does not fit
    /// within the [`len()`] of the current structure. This might happen if you
    /// try and use a reference constructed from a different [`SliceMut`]
    /// instance.
    ///
    /// [`len()`]: Self::len()
    ///
    /// ```should_panic
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf1 = SliceMut::new(&mut buf);
    /// buf1.store(&1u32);
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf2 = SliceMut::new(&mut buf);
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
    /// use musli_zerocopy::{SliceMut, Ref, ZeroCopy};
    /// use musli_zerocopy::mem::MaybeUninit;
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
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
    pub fn load_uninit_mut<T, U: ByteOrder, I: Size>(
        &mut self,
        reference: Ref<MaybeUninit<T>, U, I>,
    ) -> &mut MaybeUninit<T>
    where
        T: ZeroCopy,
    {
        let at = reference.offset();

        // Note: We only need this as debug assertion, because `MaybeUninit<T>`
        // does not implement `ZeroCopy`, so there is no way to construct.
        assert!(at + size_of::<T>() <= self.len, "Length overflow");

        // SAFETY: `MaybeUninit<T>` has no representation requirements and is
        // unaligned.
        unsafe { &mut *(self.data.as_ptr().add(at) as *mut MaybeUninit<T>) }
    }

    /// Insert a value with the given size.
    ///
    /// The memory for `T` will be initialized at [`next_offset<T>()`] and the
    /// length and alignment requirement of `SliceMut` updated to reflect that
    /// an instance of `T` has been stored.
    ///
    /// To get the offset where the value will be written, call
    /// [`next_offset<T>()`] before storing the value or access the offset
    /// through the [`Ref::offset`] being returned.
    ///
    /// [`next_offset<T>()`]: Self::next_offset
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{SliceMut, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    ///
    /// let string = buf.store_unsized("string");
    /// let custom = buf.store(&Custom { field: 1, string });
    /// let custom2 = buf.store(&Custom { field: 2, string });
    ///
    /// let buf = buf.to_requested();
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
    /// use musli_zerocopy::{ZeroCopy, SliceMut};
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
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    /// let array = buf.store(&values);
    ///
    /// let buf = buf.to_requested();
    ///
    /// assert_eq!(buf.load(array)?, &values);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store<T>(&mut self, value: &T) -> Ref<T, E, O>
    where
        T: ZeroCopy,
    {
        self.next_offset_with_and_reserve(align_of::<T>(), size_of::<T>());

        // SAFETY: We're ensuring to both align the internal buffer and store
        // the value.
        unsafe { self.store_unchecked(value) }
    }

    /// Insert a value with the given size without ensuring that the buffer has
    /// the reserved capacity for to or is properly aligned.
    ///
    /// This is a low level API which is tricky to use correctly. The
    /// recommended way to use this is through [`SliceMut::store`].
    ///
    /// [`SliceMut::store`]: Self::store
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the buffer has the required capacity for
    /// `&T` and is properly aligned. This can easily be accomplished by calling
    /// [`request_align::<T>()`] followed by [`align_in_place()`] before this
    /// function. A safe variant of this function is [`SliceMut::store`].
    ///
    /// [`align_in_place()`]: Self::align_in_place
    /// [`SliceMut::store`]: Self::store
    /// [`request_align::<T>()`]: Self::request_align
    ///
    /// # Examples
    ///
    /// ```
    /// use std::mem::size_of;
    ///
    /// use musli_zerocopy::{SliceMut, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C, align(4096))]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = [0; 12288];
    /// let mut buf = SliceMut::new(&mut buf);
    ///
    /// let string = buf.store_unsized("string");
    ///
    /// buf.request_align::<Custom>();
    /// buf.reserve(2 * size_of::<Custom>());
    ///
    /// // SAFETY: We've ensure that the buffer is internally aligned and sized just above.
    /// let custom = unsafe { buf.store_unchecked(&Custom { field: 1, string }) };
    /// let custom2 = unsafe { buf.store_unchecked(&Custom { field: 2, string }) };
    ///
    /// let buf = buf.to_requested();
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
    #[inline]
    pub unsafe fn store_unchecked<T>(&mut self, value: &T) -> Ref<T, E, O>
    where
        T: ZeroCopy,
    {
        let offset = self.len;

        let ptr = NonNull::new_unchecked(self.data.as_ptr().add(offset));
        buf::store_unaligned(ptr, value);
        self.len += size_of::<T>();
        Ref::new(offset)
    }

    /// Either return the current buffer, or allocate one which has a
    /// [`requested()`] alignment.
    ///
    /// [`requested()`]: Self::requested
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    ///
    /// let first = buf.store_unsized("first");
    /// let second = buf.store_unsized("second");
    ///
    /// let buf = buf.to_requested();
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    pub fn to_requested(&self) -> Cow<'_, Buf> {
        self.to_aligned_with(self.requested)
    }

    /// Write a value to the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    ///
    /// let first = buf.store_unsized("first");
    /// let second = buf.store_unsized("second");
    ///
    /// let buf = buf.to_requested();
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store_unsized<T: ?Sized>(&mut self, value: &T) -> Ref<T, E, O>
    where
        T: UnsizedZeroCopy,
    {
        unsafe {
            let size = size_of_val(value);
            self.next_offset_with_and_reserve(T::ALIGN, size);
            let offset = self.len;
            let ptr = NonNull::new_unchecked(self.data.as_ptr().add(offset));
            ptr.as_ptr().copy_from_nonoverlapping(value.as_ptr(), size);

            if T::PADDED {
                let mut padder = Padder::new(ptr);
                value.pad(&mut padder);
                padder.remaining_unsized(value);
            }

            self.len += size;
            Ref::with_metadata(offset, value.metadata())
        }
    }

    /// Insert a slice into the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    ///
    /// let mut values = Vec::new();
    ///
    /// values.push(buf.store_unsized("first"));
    /// values.push(buf.store_unsized("second"));
    ///
    /// let slice_ref = buf.store_slice(&values);
    ///
    /// let buf = buf.to_requested();
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
    #[inline(always)]
    pub fn store_slice<T>(&mut self, values: &[T]) -> Ref<[T], E, O>
    where
        T: ZeroCopy,
    {
        self.store_unsized(values)
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
    /// be respected.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{SliceMut, Ref};
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::with_alignment::<()>(&mut buf);
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u32>());
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// let buf = buf.to_requested();
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.reserve(bytes.len());

        // SAFETY: We just checked that there is space in the slice.
        unsafe {
            self.store_bytes(bytes);
        }
    }

    /// Fill and initialize the buffer with `byte` up to `len`.
    pub(crate) fn fill(&mut self, byte: u8, len: usize) {
        self.reserve(len);

        unsafe {
            let ptr = self.data.as_ptr().add(self.len);
            ptr.write_bytes(byte, len);
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
    pub(crate) unsafe fn store_bytes<T>(&mut self, values: &[T])
    where
        T: ZeroCopy,
    {
        let dst = self.as_ptr_mut().add(self.len);
        dst.copy_from_nonoverlapping(values.as_ptr().cast(), size_of_val(values));
        self.len += size_of_val(values);
    }

    /// Request that the current buffer should have at least the specified
    /// alignment and zero-initialize the buffer up to the next position which
    /// matches the given alignment.
    ///
    /// Note that this does not guarantee that the internal buffer is aligned
    /// in-memory. An instance of [`SliceMut`] cannot guarantee this.
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    ///
    /// buf.extend_from_slice(&[1, 2]);
    /// buf.request_align::<u32>();
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 0, 0]);
    /// ```
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the alignment is a power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
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
        self.ensure_aligned_and_reserve(align_of::<T>(), size_of::<T>());
    }

    /// Ensure that the current buffer is aligned under the assumption that it
    /// needs to be allocated.
    #[inline]
    fn ensure_aligned_and_reserve(&mut self, align: usize, reserve: usize) {
        let extra = buf::padding_to(self.len, align);
        self.reserve(extra + reserve);

        // SAFETY: The length is ensures to be within the address space.
        unsafe {
            self.data.as_ptr().add(self.len).write_bytes(0, extra);
            self.len += extra;
        }
    }

    /// Construct a pointer aligned for `align` into the current buffer which
    /// points to the next location that will be written.
    #[inline]
    pub(crate) fn next_offset_with_and_reserve(&mut self, align: usize, reserve: usize) {
        self.requested = self.requested.max(align);
        self.ensure_aligned_and_reserve(align, reserve);
    }

    /// Construct a pointer aligned for `T` into the current buffer which points
    /// to the next location that will be written.
    ///
    /// This ensures that the alignment of the pointer is a multiple of `align`
    /// and that the current buffer has the capacity for store `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{SliceMut, Ref};
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1]);
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u32>());
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    ///
    /// let buf = buf.to_requested();
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn next_offset<T>(&mut self) -> usize {
        // SAFETY: The alignment of `T` is guaranteed to be a power of two. We
        // also make sure to reserve space for `T` since it is very likely that
        // it will be written immediately after this.
        self.next_offset_with_and_reserve(align_of::<T>(), size_of::<T>());
        self.len
    }

    // Ensure that the new capacity is available or panic.
    #[inline]
    fn ensure_capacity(&mut self, new_capacity: usize) {
        let new_capacity = new_capacity.max(self.requested);

        if self.capacity < new_capacity {
            panic!(
                "Underlying slice has the capacity {}, but {} bytes are needed",
                self.capacity, new_capacity
            )
        }
    }
}

/// `SliceMut` are `Send` because the data they reference is unaliased.
unsafe impl<'a> Send for SliceMut<'a> {}
/// `SliceMut` are `Sync` since they are `Send` and the data they reference is
/// unaliased.
unsafe impl<'a> Sync for SliceMut<'a> {}

impl<'a, E: ByteOrder, O: Size> Deref for SliceMut<'a, E, O> {
    type Target = Buf;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Buf::new(self.as_slice())
    }
}

impl<'a, E: ByteOrder, O: Size> DerefMut for SliceMut<'a, E, O> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Buf::new_mut(self.as_mut_slice())
    }
}

impl<'a, E: ByteOrder, O: Size> AsRef<Buf> for SliceMut<'a, E, O> {
    /// Trivial `AsRef<Buf>` implementation for `SliceMut<O>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    /// let slice = buf.store_unsized("hello world");
    ///
    /// let buf = buf.to_requested();
    ///
    /// assert_eq!(buf.load(slice)?, "hello world");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn as_ref(&self) -> &Buf {
        self
    }
}

impl<'a, E: ByteOrder, O: Size> AsMut<Buf> for SliceMut<'a, E, O> {
    /// Trivial `AsMut<Buf>` implementation for `SliceMut<O>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::SliceMut;
    ///
    /// let mut buf = [0; 1024];
    /// let mut buf = SliceMut::new(&mut buf);
    /// let slice = buf.store_unsized("hello world");
    ///
    /// let mut buf = buf.as_mut();
    ///
    /// buf.load_mut(slice)?.make_ascii_uppercase();
    /// assert_eq!(buf.load(slice)?, "HELLO WORLD");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn as_mut(&mut self) -> &mut Buf {
        self
    }
}

impl<'a, E: ByteOrder, O: Size> Borrow<Buf> for SliceMut<'a, E, O> {
    #[inline]
    fn borrow(&self) -> &Buf {
        self.as_ref()
    }
}

impl<'a, E: ByteOrder, O: Size> StoreBuf for SliceMut<'a, E, O> {
    type ByteOrder = E;
    type Size = O;

    #[inline]
    fn len(&self) -> usize {
        SliceMut::len(self)
    }

    #[inline]
    fn truncate(&mut self, len: usize) {
        if self.len > len {
            self.len = len;
        }
    }

    #[inline]
    fn store_unsized<T: ?Sized>(&mut self, value: &T) -> Ref<T, Self::ByteOrder, Self::Size>
    where
        T: UnsizedZeroCopy,
    {
        SliceMut::store_unsized(self, value)
    }

    #[inline]
    fn store<T>(&mut self, value: &T) -> Ref<T, Self::ByteOrder, Self::Size>
    where
        T: ZeroCopy,
    {
        SliceMut::store(self, value)
    }

    #[inline]
    fn swap<T>(
        &mut self,
        a: Ref<T, Self::ByteOrder, Self::Size>,
        b: Ref<T, Self::ByteOrder, Self::Size>,
    ) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        Buf::swap(self, a, b)
    }

    #[inline]
    fn align_in_place(&mut self) {
        // SAFETY: self.requested is guaranteed to be a power of two.
        if !buf::is_aligned_with(self.as_ptr(), self.requested) {
            panic!("Slice is not aligned by {}", self.requested);
        }
    }

    #[inline]
    fn next_offset<T>(&mut self) -> usize {
        SliceMut::next_offset::<T>(self)
    }

    #[inline]
    fn next_offset_with_and_reserve(&mut self, align: usize, reserve: usize) {
        SliceMut::next_offset_with_and_reserve(self, align, reserve)
    }

    #[inline]
    fn fill(&mut self, byte: u8, len: usize) {
        SliceMut::fill(self, byte, len);
    }

    #[inline]
    fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[u8]>,
    {
        Buf::get(self, index)
    }

    #[inline]
    fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[u8]>,
    {
        Buf::get_mut(self, index)
    }

    #[inline]
    fn as_buf(&self) -> &Buf {
        self
    }

    #[inline]
    fn as_mut_buf(&mut self) -> &mut Buf {
        self
    }
}
