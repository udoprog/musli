use core::alloc::Layout;
use core::fmt;
use core::mem::{align_of, size_of};
use core::ops::Range;
use core::slice;

use crate::buf::{Bindable, Cursor, Load, LoadMut, Validator};
use crate::error::{Error, ErrorKind};
use crate::pointer::{Ref, Size, Slice, Unsized};
use crate::traits::{UnsizedZeroCopy, ZeroCopy};

/// A buffer wrapping a slice of bytes.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Buf;
/// use musli_zerocopy::pointer::Unsized;
///
/// let buf = Buf::new(b"Hello World!");
/// let unsize = Unsized::<str>::new(0, 12);
///
/// assert_eq!(buf.load(unsize)?, "Hello World!");
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[repr(transparent)]
pub struct Buf {
    data: [u8],
}

impl Buf {
    /// Wrap the given bytes as a buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    /// use musli_zerocopy::pointer::Unsized;
    ///
    /// let buf = Buf::new(b"Hello World!");
    /// let unsize = Unsized::<str>::new(0, 12);
    ///
    /// assert_eq!(buf.load(unsize)?, "Hello World!");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub const fn new(data: &[u8]) -> &Buf {
        // SAFETY: The struct is repr(transparent) over [u8].
        unsafe { &*(data as *const [u8] as *const Self) }
    }

    /// Wrap the given bytes as a mutable buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    /// use musli_zerocopy::pointer::Unsized;
    ///
    /// let mut bytes: [u8; 12] = *b"Hello World!";
    ///
    /// let buf = Buf::new_mut(&mut bytes[..]);
    /// let unsize = Unsized::<str>::new(0, 12);
    ///
    /// buf.load_mut(unsize)?.make_ascii_uppercase();
    /// assert_eq!(buf.load(unsize)?, "HELLO WORLD!");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn new_mut(data: &mut [u8]) -> &mut Buf {
        // SAFETY: The struct is repr(transparent) over [u8].
        unsafe { &mut *(data as *mut [u8] as *mut Self) }
    }

    /// Access the backing slice of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let buf = Buf::new(b"Hello World!");
    ///
    /// assert_eq!(buf.as_slice(), &b"Hello World!"[..]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Access the backing slice of the buffer mutably.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let mut bytes: [u8; 12] = *b"Hello World!";
    ///
    /// let buf = Buf::new_mut(&mut bytes[..]);
    /// buf.as_mut_slice().make_ascii_uppercase();
    ///
    /// assert_eq!(buf.as_slice(), &b"HELLO WORLD!"[..]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Test if the current buffer is layout compatible with the given `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::with_alignment::<u32>();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// let buf = buf.as_aligned();
    ///
    /// assert!(buf.is_compatible_with::<u32>());
    /// assert!(!buf.is_compatible_with::<u64>());
    /// ```
    #[inline]
    pub fn is_compatible_with<T>(&self) -> bool
    where
        T: ZeroCopy,
    {
        self.is_compatible(Layout::new::<T>())
    }

    /// Ensure that the current buffer is layout compatible with the given `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::with_alignment::<u32>();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// let buf = buf.as_aligned();
    ///
    /// assert!(buf.ensure_compatible_with::<u32>().is_ok());
    /// assert!(buf.ensure_compatible_with::<u64>().is_err());
    /// ```
    #[inline]
    pub fn ensure_compatible_with<T>(&self) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        if !self.is_compatible_with::<T>() {
            return Err(Error::new(ErrorKind::LayoutMismatch {
                layout: Layout::new::<T>(),
                range: self.range(),
            }));
        }

        Ok(())
    }

    /// Get the length of the backing buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let buf = Buf::new(b"Hello World!");
    /// assert_eq!(buf.len(), 12);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Test if the backing buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let buf = Buf::new(b"Hello World!");
    /// assert!(!buf.is_empty());
    ///
    /// let buf = Buf::new(b"");
    /// assert!(buf.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Load the given value as a reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let first = buf.store_unsized("first");
    /// let second = buf.store_unsized("second");
    ///
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn load<T>(&self, ptr: T) -> Result<&T::Target, Error>
    where
        T: Load,
    {
        ptr.load(self)
    }

    /// Load the given value as a mutable reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let first = buf.store_unsized("first");
    /// let second = buf.store_unsized("second");
    ///
    /// let buf = buf.as_mut();
    ///
    /// buf.load_mut(first)?.make_ascii_uppercase();
    ///
    /// assert_eq!(buf.load(first)?, "FIRST");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn load_mut<T>(&mut self, ptr: T) -> Result<&mut T::Target, Error>
    where
        T: LoadMut,
    {
        ptr.load_mut(self)
    }

    /// Bind the current buffer to a value.
    ///
    /// This provides a more convenient API for complex types like [`MapRef`]
    /// and [`SetRef`], and makes sure that all the internals related to the
    /// type being bound have been validated.
    ///
    /// Binding a type can be be faster in cases where you interact with the
    /// bound type a lot since accesses do not require validation, but might be
    /// slower if the access is a "one of", or infrequent.
    ///
    /// [`MapRef`]: crate::map::MapRef
    /// [`SetRef`]: crate::set::SetRef
    ///
    /// ## Examples
    ///
    /// Binding a [`Map`] ensures that all the internals of the map have been
    /// validated:
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    /// use musli_zerocopy::map::Entry;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Entry::new(1, 2));
    /// map.push(Entry::new(2, 3));
    ///
    /// let map = buf.store_map(&mut map)?;
    /// let buf = buf.as_aligned();
    /// let map = buf.bind(map)?;
    ///
    /// assert_eq!(map.get(&1)?, Some(&2));
    /// assert_eq!(map.get(&2)?, Some(&3));
    /// assert_eq!(map.get(&3)?, None);
    ///
    /// assert!(map.contains_key(&1)?);
    /// assert!(!map.contains_key(&3)?);
    /// Ok::<_, musli_zerocopy::Error>(())
    /// ```
    ///
    /// [`Map`]: crate::map::Map
    #[inline]
    pub fn bind<T>(&self, ptr: T) -> Result<T::Bound<'_>, Error>
    where
        T: Bindable,
    {
        ptr.bind(self)
    }

    /// Cast the current buffer into the given type.
    ///
    /// This is usually only used indirectly by deriving [`ZeroCopy`].
    ///
    /// [`ZeroCopy`]: derive@crate::ZeroCopy
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is correctly sized, aligned and
    /// contains a valid bit pattern for the destination type.
    #[inline]
    pub unsafe fn cast<T>(&self) -> &T {
        &*self.data.as_ptr().cast()
    }

    /// Cast the current buffer into the given mutable type.
    ///
    /// This is usually only used indirectly by deriving [`ZeroCopy`].
    ///
    /// [`ZeroCopy`]: derive@crate::ZeroCopy
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is correctly sized, aligned and
    /// contains a valid bit pattern for the destination type.
    #[inline]
    pub unsafe fn cast_mut<T>(&mut self) -> &mut T {
        &mut *self.data.as_mut_ptr().cast()
    }

    /// Construct a validator over the current buffer.
    ///
    /// This is a struct validator, which checks that the fields specified in
    /// order of subsequent calls to [`field`] conform to the `repr(C)`
    /// representation.
    ///
    /// [`field`]: Validator::field
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, field2: u64 }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let custom = buf.store(&Custom { field: 42, field2: 85 });
    /// let buf = buf.as_aligned();
    ///
    /// let mut v = buf.validate_struct::<Custom>()?;
    ///
    /// // SAFETY: We're only validating fields we know are
    /// // part of the struct, and do not go beyond.
    /// unsafe {
    ///     v.field::<u32>()?;
    ///     v.field::<u64>()?;
    /// }
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn validate_struct<T>(&self) -> Result<Validator<'_, T>, Error>
    where
        T: ZeroCopy,
    {
        self.ensure_compatible_with::<T>()?;
        Ok(Validator::new(self.cursor()))
    }

    /// Construct a cursor over the current buffer.
    ///
    /// This is a raw, low-level API to access the underlying buffer. The safety of most cursor methods depend on not violating.
    pub fn cursor(&self) -> Cursor {
        Cursor::new(&self.data)
    }

    /// Get the given range while checking its required alignment.
    #[inline]
    pub(crate) fn get(&self, start: usize, end: usize, align: usize) -> Result<&Buf, Error> {
        let buf = Buf::new(self.get_unaligned(start, end)?);

        if !buf.is_aligned_to(align) {
            return Err(Error::new(ErrorKind::AlignmentMismatch {
                range: start..end,
                align,
            }));
        }

        Ok(buf)
    }

    /// Get the given range mutably while checking its required alignment.
    #[inline]
    pub(crate) fn get_mut(
        &mut self,
        start: usize,
        end: usize,
        align: usize,
    ) -> Result<&mut Buf, Error> {
        let buf = Buf::new_mut(self.get_mut_unaligned(start, end)?);

        if !buf.is_aligned_to(align) {
            return Err(Error::new(ErrorKind::AlignmentMismatch {
                range: start..end,
                align,
            }));
        }

        Ok(buf)
    }

    /// Get the given range without checking that it corresponds to any given
    /// alignment.
    #[inline]
    pub(crate) fn get_unaligned(&self, start: usize, end: usize) -> Result<&[u8], Error> {
        let Some(data) = self.data.get(start..end) else {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range: start..end,
                len: self.data.len(),
            }));
        };

        Ok(data)
    }

    /// Get the given range mutably without checking that it corresponds to any given alignment.
    #[inline]
    pub(crate) fn get_mut_unaligned(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<&mut [u8], Error> {
        let len = self.data.len();

        let Some(data) = self.data.get_mut(start..end) else {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range: start..end,
                len,
            }));
        };

        Ok(data)
    }

    /// Load an unsized reference.
    #[inline]
    pub(crate) fn load_unsized<T: ?Sized, O: Size>(
        &self,
        unsize: Unsized<T, O>,
    ) -> Result<&T, Error>
    where
        T: UnsizedZeroCopy,
    {
        let start = unsize.offset();
        let size = unsize.size();
        let end = start.wrapping_add(size.wrapping_mul(T::SIZE));
        let buf = self.get(start, end, T::ALIGN)?;

        // SAFETY: Alignment and size is checked just above when getting the
        // buffer slice.
        unsafe { Ok(&*T::coerce(buf.as_ptr(), size)?) }
    }

    /// Load an unsized mutable reference.
    #[inline]
    pub(crate) fn load_unsized_mut<T: ?Sized, O: Size>(
        &mut self,
        unsize: Unsized<T, O>,
    ) -> Result<&mut T, Error>
    where
        T: UnsizedZeroCopy,
    {
        let start = unsize.offset();
        let size = unsize.size();
        let end = start.wrapping_add(size.wrapping_mul(T::SIZE));
        let buf = self.get_mut(start, end, T::ALIGN)?;

        // SAFETY: Alignment and size is checked just above when getting the
        // buffer slice.
        unsafe { Ok(&mut *T::coerce_mut(buf.as_mut_ptr(), size)?) }
    }

    /// Load the given sized value as a reference.
    #[inline]
    pub(crate) fn load_sized<T, O: Size>(&self, ptr: Ref<T, O>) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.offset();
        let end = start.wrapping_add(size_of::<T>());
        let buf = self.get(start, end, align_of::<T>())?;

        if !T::ANY_BITS {
            // SAFETY: We've checked the size and alignment of the buffer above.
            // The remaining safety requirements depend on the implementation of
            // validate.
            unsafe {
                T::validate(buf.cursor())?;
            }
        }

        // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
        // type being coerced into can really inhabit any bit pattern.
        Ok(unsafe { buf.cast() })
    }

    /// Load the given sized value as a mutable reference.
    #[inline]
    pub(crate) fn load_sized_mut<T, O: Size>(&mut self, ptr: Ref<T, O>) -> Result<&mut T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.offset();
        let end = start.wrapping_add(size_of::<T>());
        let buf = self.get_mut(start, end, align_of::<T>())?;

        if !T::ANY_BITS {
            // SAFETY: We've checked the size and alignment of the buffer above.
            // The remaining safety requirements depend on the implementation of
            // validate.
            unsafe {
                T::validate(buf.cursor())?;
            }
        }

        // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
        // type being coerced into can really inhabit any bit pattern.
        Ok(unsafe { buf.cast_mut() })
    }

    /// Load the given slice.
    #[inline]
    pub(crate) fn load_slice<T, O: Size>(&self, slice: Slice<T, O>) -> Result<&[T], Error>
    where
        T: ZeroCopy,
    {
        let start = slice.offset();
        let end = start.wrapping_add(slice.len().wrapping_mul(size_of::<T>()));
        let buf = self.get(start, end, align_of::<T>())?;
        let len = buf.len();
        crate::buf::validate_array::<T>(buf.cursor(), len)?;
        Ok(unsafe { slice::from_raw_parts(buf.as_ptr().cast(), slice.len()) })
    }

    /// Load the given slice mutably.
    #[inline]
    pub(crate) fn load_slice_mut<T, O: Size>(&mut self, ptr: Slice<T, O>) -> Result<&mut [T], Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.offset();
        let end = start.wrapping_add(ptr.len().wrapping_mul(size_of::<T>()));
        let buf = self.get_mut_unaligned(start, end)?;
        let len = buf.len();
        crate::buf::validate_array::<T>(Cursor::new(buf), len)?;
        Ok(unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr().cast(), ptr.len()) })
    }

    /// Access the underlying slice as a pointer.
    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Access the underlying slice as a mutable pointer.
    pub(crate) fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// The numerical range of the buffer.
    pub(crate) fn range(&self) -> Range<usize> {
        let range = self.data.as_ptr_range();
        range.start as usize..range.end as usize
    }

    /// Test if the current buffer is compatible with the given layout.
    #[inline]
    pub(crate) fn is_compatible(&self, layout: Layout) -> bool {
        self.is_aligned_to(layout.align()) && self.data.len() >= layout.size()
    }

    /// Test if the buffer is aligned with the given alignment.
    ///
    /// # Panics
    ///
    /// Panics if `align` is not a power of two.
    #[inline]
    pub(crate) fn is_aligned_to(&self, align: usize) -> bool {
        crate::buf::is_aligned_to(self.data.as_ptr(), align)
    }
}

impl fmt::Debug for Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Buf").field(&self.data.len()).finish()
    }
}
