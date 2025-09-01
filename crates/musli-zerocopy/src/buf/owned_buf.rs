use core::alloc::Layout;
use core::borrow::Borrow;
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop, align_of, size_of, size_of_val};
use core::ops::Deref;
use core::ptr::NonNull;
use core::slice::{self, SliceIndex};

#[cfg(feature = "std")]
use std::io;

use alloc::alloc;

use crate::buf::{self, AllocError, Buf, DefaultAlignment, Padder, StoreBuf};
use crate::endian::{ByteOrder, Native};
use crate::error::{Error, ErrorKind};
use crate::mem::MaybeUninit;
use crate::pointer::{DefaultSize, Ref, Size};
use crate::traits::{UnsizedZeroCopy, ZeroCopy};

/// An allocating buffer with dynamic alignment.
///
/// By default this buffer starts out having the same alignment as `usize`,
/// making it platform specific. But this alignment can grow in demand to the
/// types being stored in it.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{OwnedBuf, ZeroCopy};
///
/// #[derive(ZeroCopy)]
/// #[repr(C, align(128))]
/// struct Custom {
///     field: u32
/// }
///
/// let mut buf = OwnedBuf::new();
/// buf.store(&Custom { field: 10 })?;
/// assert!(buf.alignment() >= 128);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
pub struct OwnedBuf<E = Native, O = DefaultSize>
where
    E: ByteOrder,
    O: Size,
{
    data: NonNull<mem::MaybeUninit<u8>>,
    /// The initialized length of the buffer.
    len: usize,
    /// The capacity of the buffer.
    capacity: usize,
    /// The requested alignment.
    requested: usize,
    /// The current alignment.
    align: usize,
    /// Holding onto the current pointer size.
    _marker: PhantomData<(E, O)>,
}

impl Default for OwnedBuf {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
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
    /// assert!(buf.alignment() > 0);
    /// assert!(buf.alignment() >= buf.requested());
    /// ```
    pub const fn new() -> Self {
        Self::with_alignment::<DefaultAlignment>()
    }

    /// Allocate a new buffer with the given capacity and default alignment.
    ///
    /// The buffer must allocate for at least the given `capacity`, but might
    /// allocate more. If the capacity specified is `0` it will not allocate.
    ///
    /// # Errors
    ///
    /// Errors if the specified capacity and memory layout are illegal, which
    /// happens if:
    /// * The alignment is not a power of two.
    /// * The specified capacity causes the needed memory to overflow
    ///   `isize::MAX`.
    ///
    /// ```should_panic
    /// use std::mem::align_of;
    ///
    /// use musli_zerocopy::{endian, DefaultAlignment, OwnedBuf};
    ///
    /// let max = isize::MAX as usize - (align_of::<DefaultAlignment>() - 1);
    /// OwnedBuf::<endian::Native, u32>::with_capacity(max)?;
    /// # Ok::<_, musli_zerocopy::buf::AllocError>(())
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::with_capacity(6)?;
    /// assert!(buf.capacity() >= 6);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn with_capacity(capacity: usize) -> Result<Self, AllocError> {
        Self::with_capacity_and_alignment::<DefaultAlignment>(capacity)
    }

    /// Construct a new empty buffer with an alignment matching that of `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::with_alignment::<u64>();
    /// assert!(buf.is_empty());
    /// assert!(buf.alignment() >= 8);
    /// assert_eq!(buf.requested(), 8);
    /// ```
    pub const fn with_alignment<T>() -> Self {
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

    /// Allocate a new buffer with the given `capacity` and an alignment
    /// matching that of `T`.
    ///
    /// The buffer must allocate for at least the given `capacity`, but might
    /// allocate more. If the capacity specified is `0` it will not allocate.
    ///
    /// # Errors
    ///
    /// Errors if the specified capacity and memory layout are illegal, which
    /// happens if:
    /// * The alignment is not a power of two.
    /// * The specified capacity causes the needed memory to overflow
    ///   `isize::MAX`.
    ///
    /// ```should_panic
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let max = isize::MAX as usize - (8 - 1);
    /// OwnedBuf::with_capacity_and_alignment::<u64>(max)?;
    /// # Ok::<_, musli_zerocopy::buf::AllocError>(())
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::with_capacity_and_alignment::<u16>(6)?;
    /// assert!(buf.capacity() >= 6);
    /// assert!(buf.alignment() >= 2);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn with_capacity_and_alignment<T>(capacity: usize) -> Result<Self, AllocError> {
        // SAFETY: Alignment of `T` is always a power of two.
        Ok(unsafe { Self::with_capacity_and_custom_alignment(capacity, align_of::<T>())? })
    }
}

impl<E, O> OwnedBuf<E, O>
where
    E: ByteOrder,
    O: Size,
{
    /// Modify the buffer to utilize the specified pointer size when inserting
    /// references.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::with_capacity(1024)?
    ///     .with_size::<u8>();
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn with_size<U>(self) -> OwnedBuf<E, U>
    where
        U: Size,
    {
        let this = ManuallyDrop::new(self);

        OwnedBuf {
            data: this.data,
            len: this.len,
            capacity: this.capacity,
            requested: this.requested,
            align: this.align,
            _marker: PhantomData,
        }
    }

    /// Modify the buffer to utilize the specified byte order when inserting
    /// references.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{endian, OwnedBuf};
    ///
    /// let mut buf = OwnedBuf::with_capacity(1024)?
    ///     .with_byte_order::<endian::Little>();
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn with_byte_order<U>(self) -> OwnedBuf<U, O>
    where
        U: ByteOrder,
    {
        let this = ManuallyDrop::new(self);

        OwnedBuf {
            data: this.data,
            len: this.len,
            capacity: this.capacity,
            requested: this.requested,
            align: this.align,
            _marker: PhantomData,
        }
    }

    // # Safety
    //
    // The specified alignment must be a power of two.
    pub(crate) unsafe fn with_capacity_and_custom_alignment(
        capacity: usize,
        align: usize,
    ) -> Result<Self, AllocError> where {
        if capacity == 0 {
            return Ok(Self {
                // SAFETY: Alignment is asserted through `T`.
                data: unsafe { dangling(align) },
                len: 0,
                capacity: 0,
                requested: align,
                align,
                _marker: PhantomData,
            });
        }

        let layout = Layout::from_size_align(capacity, align).expect("Illegal memory layout");

        unsafe {
            let data = alloc::alloc(layout);

            if data.is_null() {
                return Err(AllocError::alloc_error(layout));
            }

            Ok(Self {
                data: NonNull::new_unchecked(data.cast()),
                len: 0,
                capacity,
                requested: align,
                align,
                _marker: PhantomData,
            })
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
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Clear the current buffer.
    ///
    /// This won't cause any reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// assert_eq!(buf.capacity(), 0);
    /// buf.extend_from_slice(&[1, 2, 3, 4])?;
    ///
    /// assert_eq!(buf.len(), 4);
    /// buf.clear();
    /// assert!(buf.capacity() > 0);
    /// assert_eq!(buf.len(), 0);
    /// # Ok::<_, musli_zerocopy::Error>(())
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
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
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
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// assert_eq!(buf.capacity(), 0);
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
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::with_alignment::<u64>();
    /// assert!(buf.is_empty());
    /// assert!(buf.alignment() >= 8);
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
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// assert_eq!(buf.capacity(), 0);
    ///
    /// buf.reserve(10);
    /// assert!(buf.capacity() >= 10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, capacity: usize) -> Result<(), AllocError> {
        let new_capacity = self.len + capacity;
        self.ensure_capacity(new_capacity)?;
        Ok(())
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
    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr() as *const _
    }

    /// Get get a raw mutable pointer to the current buffer.
    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut mem::MaybeUninit<u8> {
        self.data.as_ptr()
    }

    /// Get get a raw mutable pointer to the current buffer.
    #[inline]
    #[cfg(test)]
    pub(crate) fn as_nonnull(&mut self) -> NonNull<mem::MaybeUninit<u8>> {
        self.data
    }

    /// Extract a slice containing the entire buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// buf.extend_from_slice(b"hello world")?;
    /// assert_eq!(buf.as_slice(), b"hello world");
    /// # Ok::<_, musli_zerocopy::Error>(())
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
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// buf.extend_from_slice(b"hello world")?;
    /// buf.as_mut_slice().make_ascii_uppercase();
    /// assert_eq!(buf.as_slice(), b"HELLO WORLD");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: We only expose the initialized part of the buffer.
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr().cast(), self.len()) }
    }

    /// Access the buffer mutably.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// let slice = buf.store_unsized("hello world")?;
    ///
    /// // SAFETY: We don't manipulate the underlying buffer in a way which leaves uninitialized data.
    /// let buf = unsafe { buf.as_mut_buf() };
    ///
    /// buf.load_mut(slice)?.make_ascii_uppercase();
    /// assert_eq!(buf.load(slice)?, "HELLO WORLD");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// # Safety
    ///
    /// Since this allows the underlying buffer to be mutated, depending on how
    /// the buffer is used it might result in undefined bit-patterns like
    /// padding bytes being written to it. The caller must ensure this is not
    /// done with the structures being written by for example calling
    /// [`ZeroCopy::initialize_padding()`] after the contents of the buffer is
    /// modified.
    ///
    /// See [`Buf::new_mut`] for more information.
    #[inline]
    pub unsafe fn as_mut_buf(&mut self) -> &mut Buf {
        unsafe { Buf::new_mut(self.as_mut_slice()) }
    }

    /// Store an uninitialized value.
    ///
    /// This allows values to be inserted before they can be initialized, which
    /// can be useful if you need them to be in a certain location in the buffer
    /// but don't have access to their value yet.
    ///
    /// The memory for `T` will be zero-initialized at [`next_offset<T>()`] and
    /// the length and alignment requirement of `OwnedBuf` updated to reflect
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
    /// use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = OwnedBuf::new();
    /// let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>()?;
    ///
    /// let string = buf.store_unsized("Hello World!")?;
    ///
    /// buf.load_uninit_mut(reference)?.write(&Custom { field: 42, string });
    ///
    /// let reference = reference.assume_init();
    /// assert_eq!(reference.offset(), 0);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store_uninit<T>(&mut self) -> Result<Ref<MaybeUninit<T>, E, O>, Error>
    where
        T: ZeroCopy,
    {
        // SAFETY: We've just reserved capacity for this write.
        unsafe {
            self.next_offset_with_and_reserve(align_of::<T>(), size_of::<T>())?;
            let offset = self.len;

            self.data
                .as_ptr()
                .add(self.len)
                .write_bytes(0, size_of::<T>());

            self.len += size_of::<T>();
            Ok(Ref::try_with_metadata_unchecked(offset, ())?)
        }
    }

    /// Write a reference that might not have been initialized.
    ///
    /// This does not prevent [`Ref`] from different instances of [`OwnedBuf`]
    /// from being written. It would only result in garbled data, but wouldn't
    /// be a safety concern.
    ///
    /// > **Note:** this does not return [`std::mem::MaybeUninit`], instead we
    /// > use an internal [`MaybeUninit`] which is similar but has different
    /// > properties. See [its documentation][MaybeUninit] for more.
    ///
    /// # Errors
    ///
    /// Errors if the reference [`Ref::offset()`] and size of `T` does not fit
    /// within the [`len()`] of the current structure. This might happen if you
    /// try and use a reference constructed from a different [`OwnedBuf`]
    /// instance.
    ///
    /// [`len()`]: Self::len()
    ///
    /// ```should_panic
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf2 = OwnedBuf::new();
    /// let number = buf2.store_uninit::<u32>()?;
    ///
    /// let mut buf1 = OwnedBuf::new();
    /// buf1.load_uninit_mut(number)?.write(&42u32);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
    /// use musli_zerocopy::mem::MaybeUninit;
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = OwnedBuf::new();
    /// let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>()?;
    ///
    /// let string = buf.store_unsized("Hello World!")?;
    ///
    /// buf.load_uninit_mut(reference)?.write(&Custom { field: 42, string });
    ///
    /// let reference = reference.assume_init();
    /// assert_eq!(reference.offset(), 0);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn load_uninit_mut<T, U, I>(
        &mut self,
        at: Ref<MaybeUninit<T>, U, I>,
    ) -> Result<&mut MaybeUninit<T>, Error>
    where
        T: ZeroCopy,
        U: ByteOrder,
        I: Size,
    {
        let offset = at.offset();

        if offset + size_of::<T>() > self.len {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range: offset..offset + size_of::<T>(),
                len: self.len,
            }));
        }

        // SAFETY: `MaybeUninit<T>` has no representation requirements and is
        // unaligned.
        Ok(unsafe { &mut *(self.data.as_ptr().add(offset) as *mut MaybeUninit<T>) })
    }

    /// Insert a value with the given size.
    ///
    /// The memory for `T` will be initialized at [`next_offset<T>()`] and the
    /// length and alignment requirement of `OwnedBuf` updated to reflect that
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
    /// use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let string = buf.store_unsized("string")?;
    /// let custom = buf.store(&Custom { field: 1, string })?;
    /// let custom2 = buf.store(&Custom { field: 2, string })?;
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
    /// use musli_zerocopy::{ZeroCopy, OwnedBuf};
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
    /// let mut buf = OwnedBuf::new();
    /// let array = buf.store(&values)?;
    /// assert_eq!(buf.load(array)?, &values);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store<T>(&mut self, value: &T) -> Result<Ref<T, E, O>, Error>
    where
        T: ZeroCopy,
    {
        self.next_offset_with_and_reserve(align_of::<T>(), size_of::<T>())?;

        // SAFETY: We're ensuring to both align the internal buffer and store
        // the value.
        unsafe { self.store_unchecked(value) }
    }

    /// Insert a value with the given size without ensuring that the buffer has
    /// the reserved capacity for to or is properly aligned.
    ///
    /// This is a low level API which is tricky to use correctly. The
    /// recommended way to use this is through [`OwnedBuf::store`].
    ///
    /// [`OwnedBuf::store`]: Self::store
    ///
    /// # Safety
    ///
    /// The caller has to ensure that the buffer has the required capacity for
    /// `&T` and is properly aligned. This can easily be accomplished by calling
    /// [`request_align::<T>()`] followed by [`align_in_place()`] before this
    /// function. A safe variant of this function is [`OwnedBuf::store`].
    ///
    /// [`align_in_place()`]: Self::align_in_place
    /// [`OwnedBuf::store`]: Self::store
    /// [`request_align::<T>()`]: Self::request_align
    ///
    /// # Examples
    ///
    /// ```
    /// use std::mem::size_of;
    ///
    /// use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C, align(4096))]
    /// struct Custom { field: u32, string: Ref<str> }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let string = buf.store_unsized("string")?;
    ///
    /// buf.request_align::<Custom>()?;
    /// buf.reserve(2 * size_of::<Custom>());
    /// buf.align_in_place()?;
    ///
    /// // SAFETY: We've ensure that the buffer is internally aligned and sized just above.
    /// let custom = unsafe { buf.store_unchecked(&Custom { field: 1, string })? };
    /// let custom2 = unsafe { buf.store_unchecked(&Custom { field: 2, string })? };
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
    pub unsafe fn store_unchecked<T>(&mut self, value: &T) -> Result<Ref<T, E, O>, Error>
    where
        T: ZeroCopy,
    {
        let offset = self.len;

        unsafe {
            let ptr = NonNull::new_unchecked(self.data.as_ptr().add(offset));
            buf::store_unaligned(ptr, value);
            self.len += size_of::<T>();
            // SAFETY: We trust the calculated offset since there language
            // invariants ensures that it cannot overflow the container size.
            Ok(Ref::try_with_metadata_unchecked(offset, ())?)
        }
    }

    /// Write a value to the buffer.
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let first = buf.store_unsized("first")?;
    /// let second = buf.store_unsized("second")?;
    ///
    /// dbg!(first, second);
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn store_unsized<T>(&mut self, value: &T) -> Result<Ref<T, E, O>, Error>
    where
        T: ?Sized + UnsizedZeroCopy,
    {
        unsafe {
            let size = size_of_val(value);
            self.next_offset_with_and_reserve(T::ALIGN, size)?;
            let offset = self.len;
            let ptr = NonNull::new_unchecked(self.data.as_ptr().add(offset));
            ptr.as_ptr()
                .copy_from_nonoverlapping(value.as_ptr().cast(), size);

            if T::PADDED {
                let mut padder = Padder::new(ptr);
                value.pad(&mut padder);
                padder.remaining_unsized(value);
            }

            self.len += size;

            // SAFETY: We trust the calculated offset since there language
            // invariants ensures that it cannot overflow the container size.
            Ok(Ref::try_with_metadata_unchecked(offset, value.metadata())?)
        }
    }

    /// Insert a slice into the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let values = [buf.store_unsized("first")?, buf.store_unsized("second")?];
    /// let slice_ref = buf.store_slice(&values)?;
    ///
    /// let slice = buf.load(slice_ref)?;
    /// assert_eq!(buf.load(slice[0])?, "first");
    /// assert_eq!(buf.load(slice[1])?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline(always)]
    pub fn store_slice<T>(&mut self, values: &[T]) -> Result<Ref<[T], E, O>, Error>
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
    /// be respected. The following exemplifies incorrect use since the u32 type
    /// required a 4-byte alignment:
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, Ref};
    ///
    /// let mut buf = OwnedBuf::with_alignment::<u32>();
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1])?;
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u8>()?);
    /// buf.extend_from_slice(&[1, 2, 3, 4])?;
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
    /// use musli_zerocopy::{OwnedBuf, Ref};
    ///
    /// let mut buf = OwnedBuf::with_alignment::<()>();
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1])?;
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u32>()?);
    /// buf.extend_from_slice(&[1, 2, 3, 4])?;
    ///
    /// // This will succeed because the buffer follows its interior alignment:
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), AllocError> {
        self.reserve(bytes.len())?;

        // SAFETY: We just allocated space for the slice.
        unsafe {
            self.store_bytes(bytes);
        }

        Ok(())
    }

    /// Fill and initialize the buffer with `byte` up to `len`.
    pub(crate) fn fill(&mut self, byte: u8, len: usize) -> Result<(), AllocError> {
        self.reserve(len)?;

        unsafe {
            let ptr = self.data.as_ptr().add(self.len);
            ptr.write_bytes(byte, len);
            self.len += len;
        }

        Ok(())
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
        unsafe {
            let dst = self.as_mut_ptr().add(self.len);
            dst.copy_from_nonoverlapping(values.as_ptr().cast(), size_of_val(values));
            self.len += size_of_val(values);
        }
    }

    /// Align a buffer in place if necessary.
    ///
    /// If [`requested()`] does not equal [`alignment()`] this will cause the buffer
    /// to be reallocated before it is returned.
    ///
    /// [`requested()`]: Self::requested
    /// [`alignment()`]: Buf::alignment
    /// [`as_ref`]: Self::as_ref
    ///
    /// # Examples
    ///
    /// A buffer has to be a aligned in order for `load` calls to succeed
    /// without errors.
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::with_alignment::<()>();
    /// let number = buf.store(&1u32)?;
    ///
    /// buf.align_in_place()?;
    ///
    /// assert_eq!(buf.load(number)?, &1u32);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// Example using a mutable buffer. A buffer has to be a aligned in order
    /// for `load` and `load_mut` calls to succeed without errors.
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::with_alignment::<()>();
    /// let number = buf.store(&1u32)?;
    ///
    /// buf.align_in_place()?;
    ///
    /// // SAFETY: We're not writing data in a way which leaves uninitialized regions.
    /// unsafe {
    ///     *buf.as_mut_buf().load_mut(number)? += 1;
    /// }
    /// assert_eq!(buf.load(number)?, &2u32);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn align_in_place(&mut self) -> Result<(), AllocError> {
        // SAFETY: self.requested is guaranteed to be a power of two.
        if !buf::is_aligned_with(self.as_ptr(), self.requested) {
            let (old_layout, new_layout) = self.layouts(self.capacity);
            self.alloc_new(old_layout, new_layout)?;
        }

        Ok(())
    }

    /// Request that the current buffer should have at least the specified
    /// alignment and zero-initialize the buffer up to the next position which
    /// matches the given alignment.
    ///
    /// Note that this does not guarantee that the internal buffer is aligned
    /// in-memory, to ensure this you can use [`align_in_place()`].
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    /// let mut buf = OwnedBuf::new();
    ///
    /// buf.extend_from_slice(&[1, 2])?;
    /// buf.request_align::<u32>()?;
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 0, 0]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// Calling this function only causes the underlying buffer to be realigned
    /// if a reallocation is triggered due to reaching its [`capacity()`].
    ///
    /// ```
    /// use musli_zerocopy::{endian, OwnedBuf};
    /// let mut buf = OwnedBuf::<endian::Native, u32>::with_capacity_and_alignment::<u16>(32)?;
    ///
    /// buf.extend_from_slice(&[1, 2])?;
    /// assert!(buf.alignment() >= 2);
    /// buf.request_align::<u32>()?;
    ///
    /// assert_eq!(buf.requested(), 4);
    /// assert!(buf.alignment() >= 2);
    ///
    /// buf.extend_from_slice(&[0; 32])?;
    /// assert_eq!(buf.requested(), 4);
    /// assert!(buf.alignment() >= 4);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// [`capacity()`]: Self::capacity
    /// [`align_in_place()`]: Self::align_in_place
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the alignment is a power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// buf.extend_from_slice(&[1, 2, 3, 4])?;
    /// buf.request_align::<u64>()?;
    /// buf.extend_from_slice(&[5, 6, 7, 8])?;
    ///
    /// assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 0, 0, 0, 0, 5, 6, 7, 8]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn request_align<T>(&mut self) -> Result<(), AllocError>
    where
        T: ZeroCopy,
    {
        self.requested = self.requested.max(align_of::<T>());
        self.ensure_aligned_and_reserve(align_of::<T>(), size_of::<T>())?;
        Ok(())
    }

    /// Ensure that the current buffer is aligned under the assumption that it needs to be allocated.
    #[inline]
    fn ensure_aligned_and_reserve(
        &mut self,
        align: usize,
        reserve: usize,
    ) -> Result<(), AllocError> {
        let extra = buf::padding_to(self.len, align);
        self.reserve(extra + reserve)?;

        // SAFETY: The length is ensures to be within the address space.
        unsafe {
            self.data.as_ptr().add(self.len).write_bytes(0, extra);
            self.len += extra;
        }

        Ok(())
    }

    /// Construct a pointer aligned for `align` into the current buffer which
    /// points to the next location that will be written.
    #[inline]
    pub(crate) fn next_offset_with_and_reserve(
        &mut self,
        align: usize,
        reserve: usize,
    ) -> Result<(), AllocError> {
        self.requested = self.requested.max(align);
        self.ensure_aligned_and_reserve(align, reserve)?;
        Ok(())
    }

    /// Construct an offset aligned for `T` into the current buffer which points
    /// to the next location that will be written.
    ///
    /// This ensures that the alignment of the pointer is a multiple of `align`
    /// and that the current buffer has the capacity for store `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{OwnedBuf, Ref};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// // Add one byte of padding to throw of any incidental alignment.
    /// buf.extend_from_slice(&[1])?;
    ///
    /// let ptr: Ref<u32> = Ref::new(buf.next_offset::<u32>()?);
    /// buf.extend_from_slice(&[1, 2, 3, 4])?;
    ///
    /// // This will succeed because the buffer follows its interior alignment:
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(*buf.load(ptr)?, u32::from_ne_bytes([1, 2, 3, 4]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn next_offset<T>(&mut self) -> Result<usize, AllocError> {
        // SAFETY: The alignment of `T` is guaranteed to be a power of two. We
        // also make sure to reserve space for `T` since it is very likely that
        // it will be written immediately after this.
        self.next_offset_with_and_reserve(align_of::<T>(), size_of::<T>())?;
        Ok(self.len)
    }

    // We never want this call to be inlined, because we take great care to
    // ensure that reallocations we perform publicly are performed in a sparse
    // way.
    #[inline(never)]
    fn ensure_capacity(&mut self, new_capacity: usize) -> Result<(), AllocError> {
        let new_capacity = new_capacity.max(self.requested);

        if self.capacity >= new_capacity {
            return Ok(());
        }

        let new_capacity = new_capacity.max((self.capacity as f32 * 1.5) as usize);
        let (old_layout, new_layout) = self.layouts(new_capacity);

        if old_layout.size() == 0 {
            self.alloc_init(new_layout)
        } else if new_layout.align() == old_layout.align() {
            self.alloc_realloc(old_layout, new_layout)
        } else {
            self.alloc_new(old_layout, new_layout)
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
    fn alloc_init(&mut self, new_layout: Layout) -> Result<(), AllocError> {
        unsafe {
            let ptr = alloc::alloc(new_layout);

            if ptr.is_null() {
                return Err(AllocError::alloc_error(new_layout));
            }

            self.data = NonNull::new_unchecked(ptr.cast());
            self.capacity = new_layout.size();
            self.align = self.requested;
            Ok(())
        }
    }

    /// Reallocate, note that the alignment of the old layout must match the new
    /// one.
    fn alloc_realloc(&mut self, old_layout: Layout, new_layout: Layout) -> Result<(), AllocError> {
        debug_assert_eq!(old_layout.align(), new_layout.align());

        unsafe {
            let ptr = alloc::realloc(self.as_mut_ptr().cast(), old_layout, new_layout.size());

            if ptr.is_null() {
                return Err(AllocError::alloc_error(new_layout));
            }

            // NB: We may simply forget the old allocation, since `realloc` is
            // responsible for freeing it.
            self.data = NonNull::new_unchecked(ptr.cast());
            self.capacity = new_layout.size();
            Ok(())
        }
    }

    /// Perform a new allocation, deallocating the old one in the process.
    #[inline(always)]
    fn alloc_new(&mut self, old_layout: Layout, new_layout: Layout) -> Result<(), AllocError> {
        unsafe {
            let ptr = alloc::alloc(new_layout);

            if ptr.is_null() {
                return Err(AllocError::alloc_error(new_layout));
            }

            ptr.copy_from_nonoverlapping(self.as_ptr(), self.len);
            alloc::dealloc(self.as_mut_ptr().cast(), old_layout);

            // We've deallocated the old pointer.
            self.data = NonNull::new_unchecked(ptr.cast());
            self.capacity = new_layout.size();
            self.align = self.requested;
            Ok(())
        }
    }
}

/// `OwnedBuf` are `Send` because the data they reference is unaliased.
unsafe impl Send for OwnedBuf {}
/// `OwnedBuf` are `Sync` since they are `Send` and the data they reference is
/// unaliased.
unsafe impl Sync for OwnedBuf {}

impl<E, O> Deref for OwnedBuf<E, O>
where
    E: ByteOrder,
    O: Size,
{
    type Target = Buf;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Buf::new(self.as_slice())
    }
}

impl<E, O> AsRef<Buf> for OwnedBuf<E, O>
where
    E: ByteOrder,
    O: Size,
{
    /// Trivial `AsRef<Buf>` implementation for `OwnedBuf<O>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
    /// let slice = buf.store_unsized("hello world")?;
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(buf.load(slice)?, "hello world");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn as_ref(&self) -> &Buf {
        self
    }
}

impl<E, O> Borrow<Buf> for OwnedBuf<E, O>
where
    E: ByteOrder,
    O: Size,
{
    #[inline]
    fn borrow(&self) -> &Buf {
        self.as_ref()
    }
}

/// Clone the [`OwnedBuf`].
///
/// While this causes another allocation, it doesn't ensure that the returned
/// buffer has the [`requested()`] alignment. To achieve this prefer using
/// [`align_in_place()`].
///
/// [`requested()`]: Self::requested()
/// [`align_in_place()`]: Self::align_in_place
///
/// # Examples
///
/// ```
/// use std::mem::align_of;
///
/// use musli_zerocopy::{endian, OwnedBuf};
///
/// assert_ne!(align_of::<u16>(), align_of::<u32>());
///
/// let mut buf = OwnedBuf::<endian::Native, u32>::with_capacity_and_alignment::<u16>(32)?;
/// buf.extend_from_slice(&[1, 2, 3, 4])?;
/// buf.request_align::<u32>()?;
///
/// let buf2 = buf.clone();
/// assert!(buf2.alignment() >= align_of::<u16>());
///
/// buf.align_in_place()?;
/// assert!(buf.alignment() >= align_of::<u32>());
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
impl<E, O> Clone for OwnedBuf<E, O>
where
    E: ByteOrder,
    O: Size,
{
    fn clone(&self) -> Self {
        unsafe {
            let result = Self::with_capacity_and_custom_alignment(self.len, self.align);

            let mut new = match result {
                Ok(new) => ManuallyDrop::new(new),
                Err(_) => alloc::handle_alloc_error(Layout::from_size_align_unchecked(
                    self.len, self.align,
                )),
            };

            new.as_mut_ptr()
                .copy_from_nonoverlapping(self.as_ptr().cast(), self.len);
            // Set requested to the same as original.
            new.requested = self.requested;
            new.len = self.len;
            ManuallyDrop::into_inner(new)
        }
    }
}

impl<E, O> Drop for OwnedBuf<E, O>
where
    E: ByteOrder,
    O: Size,
{
    fn drop(&mut self) {
        unsafe {
            if self.capacity != 0 {
                // SAFETY: This is guaranteed to be valid per the construction
                // of this type.
                let layout = Layout::from_size_align_unchecked(self.capacity, self.align);
                alloc::dealloc(self.data.as_ptr().cast(), layout);
            }
        }
    }
}

const unsafe fn dangling(align: usize) -> NonNull<mem::MaybeUninit<u8>> {
    unsafe { NonNull::new_unchecked(invalid_mut(align)) }
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

impl<E, O> StoreBuf for OwnedBuf<E, O>
where
    E: ByteOrder,
    O: Size,
{
    type ByteOrder = E;
    type Size = O;

    #[inline]
    fn len(&self) -> usize {
        OwnedBuf::len(self)
    }

    #[inline]
    fn truncate(&mut self, len: usize) {
        if self.len > len {
            self.len = len;
        }
    }

    #[inline]
    fn store_unsized<T>(&mut self, value: &T) -> Result<Ref<T, Self::ByteOrder, Self::Size>, Error>
    where
        T: ?Sized + UnsizedZeroCopy,
    {
        OwnedBuf::store_unsized(self, value)
    }

    #[inline]
    fn store<T>(&mut self, value: &T) -> Result<Ref<T, Self::ByteOrder, Self::Size>, Error>
    where
        T: ZeroCopy,
    {
        OwnedBuf::store(self, value)
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
        // SAFETY: Since we are swapping two locations which have the same type
        // `T`, it does not affect the initialized state of the buffer.
        let buf = unsafe { self.as_mut_buf() };
        Buf::swap(buf, a, b)
    }

    #[inline]
    fn align_in_place(&mut self) -> Result<(), AllocError> {
        OwnedBuf::align_in_place(self)
    }

    #[inline]
    fn next_offset<T>(&mut self) -> Result<usize, AllocError> {
        OwnedBuf::next_offset::<T>(self)
    }

    #[inline]
    fn next_offset_with_and_reserve(
        &mut self,
        align: usize,
        reserve: usize,
    ) -> Result<(), AllocError> {
        OwnedBuf::next_offset_with_and_reserve(self, align, reserve)
    }

    #[inline]
    fn fill(&mut self, byte: u8, len: usize) -> Result<(), AllocError> {
        OwnedBuf::fill(self, byte, len)
    }

    #[inline]
    fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[u8]>,
    {
        Buf::get(self, index)
    }

    #[inline]
    unsafe fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[u8]>,
    {
        unsafe { OwnedBuf::as_mut_buf(self).get_mut(index) }
    }

    #[inline]
    fn as_buf(&self) -> &Buf {
        self
    }

    #[inline]
    unsafe fn as_mut_buf(&mut self) -> &mut Buf {
        unsafe { OwnedBuf::as_mut_buf(self) }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl io::Write for OwnedBuf {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf).map_err(io::Error::other)?;
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
