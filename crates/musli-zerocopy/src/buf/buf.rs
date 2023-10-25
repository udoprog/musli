use core::alloc::Layout;
use core::fmt;
use core::mem::{align_of, size_of};
use core::ops::{Index, IndexMut, Range};
use core::slice::SliceIndex;

#[cfg(feature = "alloc")]
use alloc::borrow::{Cow, ToOwned};

#[cfg(feature = "alloc")]
use crate::buf::OwnedBuf;
use crate::buf::{Bindable, Load, LoadMut, Validator};
use crate::endian::ByteOrder;
use crate::error::{Error, ErrorKind};
use crate::pointer::{Pointee, Ref, Size};
use crate::traits::{UnsizedZeroCopy, ZeroCopy};

/// A buffer wrapping a slice of bytes.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{Buf, Ref};
///
/// let buf = Buf::new(b"Hello World!");
/// let unsize: Ref<str> = Ref::with_metadata(0, 12);
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
    /// use musli_zerocopy::{Buf, Ref};
    ///
    /// let buf = Buf::new(b"Hello World!");
    /// let unsize: Ref<str> = Ref::with_metadata(0, 12);
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
    /// use musli_zerocopy::{Buf, Ref};
    ///
    /// let mut bytes: [u8; 12] = *b"Hello World!";
    ///
    /// let buf = Buf::new_mut(&mut bytes[..]);
    /// let unsize = Ref::<str>::with_metadata(0, 12);
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

    /// Construct a buffer with an alignment matching that of `T` which is
    /// either wrapped in a [`Buf`] if it is already correctly aligned, or
    /// inside of an allocated [`OwnedBuf`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::read;
    ///
    /// use musli_zerocopy::{Buf, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Person {
    ///     name: Ref<str>,
    ///     age: u32,
    /// }
    ///
    /// let bytes = read("person.bin")?;
    /// let buf = Buf::new(&bytes).to_aligned::<u128>();
    ///
    /// let s = buf.load(Ref::<Person>::zero())?;
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_aligned<T>(&self) -> Cow<'_, Buf> {
        self.to_aligned_with(align_of::<T>())
    }

    /// Construct a buffer with a specific alignment which is either wrapped in
    /// a [`Buf`] if it is already correctly aligned, or inside of an allocated
    /// [`OwnedBuf`].
    ///
    /// # Panics
    ///
    /// Panics if `align` is not a power of two or if the size of the buffer is
    /// larger than [`max_capacity_for_align(align)`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::read;
    ///
    /// use musli_zerocopy::{Buf, Ref, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Person {
    ///     name: Ref<str>,
    ///     age: u32,
    /// }
    ///
    /// let bytes = read("person.bin")?;
    /// let buf = Buf::new(&bytes).to_aligned_with(16);
    ///
    /// let s = buf.load(Ref::<Person>::zero())?;
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_aligned_with(&self, align: usize) -> Cow<'_, Buf> {
        assert!(align.is_power_of_two(), "Alignment must be power of two");

        // SAFETY: align is checked as a power of two just above.
        if unsafe { self.is_aligned_with_unchecked(align) } {
            Cow::Borrowed(self)
        } else {
            let mut buf =
                unsafe { OwnedBuf::with_capacity_and_custom_alignment(self.len(), align) };

            // SAFETY: Space for the slice has been allocated.
            unsafe {
                buf.store_bytes(&self.data);
            }

            Cow::Owned(buf)
        }
    }

    /// Get the alignment of the current buffer.
    pub fn alignment(&self) -> usize {
        // NB: Maximum alignment supported by Rust is 2^29.
        1usize << (self.data.as_ptr() as usize).trailing_zeros().min(29)
    }

    /// Test if the current buffer is layout compatible with the given `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::with_alignment::<u32>();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// let buf = buf.into_aligned();
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
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::with_alignment::<u32>();
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// let buf = buf.into_aligned();
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

    /// Returns a reference to an element or subslice depending on the type of
    /// index.
    ///
    /// - If given a position, returns a reference to the element at that
    ///   position or `None` if out of bounds.
    /// - If given a range, returns the subslice corresponding to that range, or
    ///   `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let buf = Buf::new(b"Hello World!");
    ///
    /// assert_eq!(buf.get(..5), Some(&b"Hello"[..]));
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.data.get(index)
    }

    /// Returns a mutable reference to an element or subslice depending on the
    /// type of index (see [`get`]) or `None` if the index is out of bounds.
    ///
    /// [`get`]: slice::get
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let mut bytes: [u8; 12] = *b"Hello World!";
    ///
    /// let buf = Buf::new_mut(&mut bytes[..]);
    ///
    /// if let Some(bytes) = buf.get_mut(..) {
    ///     bytes.make_ascii_uppercase();
    /// }
    ///
    /// assert_eq!(&buf[..], &b"HELLO WORLD!"[..]);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.data.get_mut(index)
    }

    /// Load the given value as a reference.
    ///
    /// # Errors
    ///
    /// This will error if the current buffer is not aligned for the type `T`,
    /// or for other reasons specific to what needs to be done to validate a
    /// `&T` reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
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
    /// # Errors
    ///
    /// This will error if the current buffer is not aligned for the type `T`,
    /// or for other reasons specific to what needs to be done to validate a
    /// `&mut T` reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let mut buf = OwnedBuf::new();
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
    /// This provides a more convenient API for complex types like
    /// [`swiss::MapRef`] and [`swiss::SetRef`], and makes sure that all the
    /// internals related to the type being bound have been validated.
    ///
    /// Binding a type can be be faster in cases where you interact with the
    /// bound type a lot since accesses do not require validation, but might be
    /// slower if the access is a "one of", or infrequent.
    ///
    /// [`swiss::MapRef`]: crate::swiss::MapRef
    /// [`swiss::SetRef`]: crate::swiss::SetRef
    ///
    /// ## Examples
    ///
    /// Binding a [`swiss::Map`] ensures that all the internals of the map have
    /// been validated:
    ///
    /// [`swiss::Map`]: crate::swiss::Map
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    /// use musli_zerocopy::swiss;
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let map = swiss::store_map(&mut buf, [(1, 2), (2, 3)])?;
    /// let buf = buf.into_aligned();
    /// let map = buf.bind(map)?;
    ///
    /// assert_eq!(map.get(&1)?, Some(&2));
    /// assert_eq!(map.get(&2)?, Some(&3));
    /// assert_eq!(map.get(&3)?, None);
    ///
    /// assert!(map.contains_key(&1)?);
    /// assert!(!map.contains_key(&3)?);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
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
    /// use musli_zerocopy::{OwnedBuf, ZeroCopy};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom { field: u32, field2: u64 }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let custom = buf.store(&Custom { field: 42, field2: 85 });
    /// let buf = buf.into_aligned();
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
        Ok(Validator::new(self.data.as_ptr()))
    }

    pub(crate) unsafe fn get_range_from(
        &self,
        start: usize,
        align: usize,
    ) -> Result<(*const u8, usize), Error> {
        if self.data.len() < start {
            return Err(Error::new(ErrorKind::OutOfRangeFromBounds {
                range: start..,
                len: self.data.len(),
            }));
        };

        let ptr = self.data.as_ptr().wrapping_add(start);
        let remaining = self.data.len() - start;

        if !crate::buf::is_aligned_with(ptr, align) {
            return Err(Error::new(ErrorKind::AlignmentRangeFromMismatch {
                range: start..,
                align,
            }));
        }

        Ok((ptr, remaining))
    }

    pub(crate) unsafe fn get_mut_range_from(
        &mut self,
        start: usize,
        align: usize,
    ) -> Result<(*mut u8, usize), Error> {
        if self.data.len() < start {
            return Err(Error::new(ErrorKind::OutOfRangeFromBounds {
                range: start..,
                len: self.data.len(),
            }));
        };

        let ptr = self.data.as_mut_ptr().wrapping_add(start);
        let remaining = self.data.len() - start;

        if !crate::buf::is_aligned_with(ptr, align) {
            return Err(Error::new(ErrorKind::AlignmentRangeFromMismatch {
                range: start..,
                align,
            }));
        }

        Ok((ptr, remaining))
    }

    /// Get the given range while checking its required alignment.
    ///
    /// # Safety
    ///
    /// Specified `align` must be a power of two.
    #[inline]
    pub(crate) unsafe fn inner_get(
        &self,
        start: usize,
        end: usize,
        align: usize,
    ) -> Result<&Buf, Error> {
        let buf = self.inner_get_unaligned(start, end)?;

        if !crate::buf::is_aligned_with(buf.as_ptr(), align) {
            return Err(Error::new(ErrorKind::AlignmentRangeMismatch {
                addr: buf.as_ptr() as usize,
                range: start..end,
                align,
            }));
        }

        Ok(Buf::new(buf))
    }

    /// Get the given range mutably while checking its required alignment.
    ///
    /// # Safety
    ///
    /// Specified `align` must be a power of two.
    #[inline]
    pub(crate) unsafe fn inner_get_mut(
        &mut self,
        start: usize,
        end: usize,
        align: usize,
    ) -> Result<&mut Buf, Error> {
        let buf = self.inner_get_mut_unaligned(start, end)?;

        if !crate::buf::is_aligned_with(buf.as_ptr(), align) {
            return Err(Error::new(ErrorKind::AlignmentRangeMismatch {
                addr: buf.as_ptr() as usize,
                range: start..end,
                align,
            }));
        }

        Ok(Buf::new_mut(buf))
    }

    /// Get the given range without checking that it corresponds to any given
    /// alignment.
    #[inline]
    pub(crate) fn inner_get_unaligned(&self, start: usize, end: usize) -> Result<&[u8], Error> {
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
    pub(crate) fn inner_get_mut_unaligned(
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
    pub(crate) fn load_unsized<P: ?Sized, O: Size, E: ByteOrder>(
        &self,
        unsize: Ref<P, O, E>,
    ) -> Result<&P, Error>
    where
        P: Pointee<O, Packed = O> + UnsizedZeroCopy<P, O>,
    {
        let start = unsize.offset();
        let metadata = unsize.metadata();

        // SAFETY: Alignment and size is checked just above when getting the
        // buffer slice.
        unsafe {
            let (buf, remaining) = self.get_range_from(start, P::ALIGN)?;
            let metadata = P::validate_unsized::<E>(buf, remaining, metadata)?;
            Ok(&*P::ptr_with_metadata(buf, metadata))
        }
    }

    /// Load an unsized mutable reference.
    #[inline]
    pub(crate) fn load_unsized_mut<P: ?Sized, O: Size, E: ByteOrder>(
        &mut self,
        unsize: Ref<P, O, E>,
    ) -> Result<&mut P, Error>
    where
        P: Pointee<O, Packed = O> + UnsizedZeroCopy<P, O>,
    {
        let start = unsize.offset();
        let metadata = unsize.metadata();

        // SAFETY: Alignment and size is checked just above when getting the
        // buffer slice.
        unsafe {
            let (buf, remaining) = self.get_mut_range_from(start, P::ALIGN)?;
            let metadata = P::validate_unsized::<E>(buf, remaining, metadata)?;
            Ok(&mut *P::ptr_with_metadata_mut(buf, metadata))
        }
    }

    /// Load the given sized value as a reference.
    #[inline]
    pub(crate) fn load_sized<P, O: Size, E: ByteOrder>(
        &self,
        ptr: Ref<P, O, E>,
    ) -> Result<&P, Error>
    where
        P: ZeroCopy,
    {
        let start = ptr.offset();
        let end = start.wrapping_add(size_of::<P>());

        unsafe {
            // SAFETY: align_of::<T>() is always a power of two.
            let buf = self.inner_get(start, end, align_of::<P>())?;

            if !P::ANY_BITS {
                // SAFETY: We've checked the size and alignment of the buffer above.
                // The remaining safety requirements depend on the implementation of
                // validate.
                P::validate(&mut Validator::new(buf.as_ptr()))?;
            }

            // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
            // type being coerced into can really inhabit any bit pattern.
            Ok(buf.cast())
        }
    }

    /// Load the given sized value as a mutable reference.
    #[inline]
    pub(crate) fn load_sized_mut<P, O: Size, E: ByteOrder>(
        &mut self,
        ptr: Ref<P, O, E>,
    ) -> Result<&mut P, Error>
    where
        P: ZeroCopy,
    {
        let start = ptr.offset();
        let end = start.wrapping_add(size_of::<P>());

        unsafe {
            // SAFETY: align_of::<T>() is always a power of two.
            let buf = self.inner_get_mut(start, end, align_of::<P>())?;

            if !P::ANY_BITS {
                // SAFETY: We've checked the size and alignment of the buffer above.
                // The remaining safety requirements depend on the implementation of
                // validate.
                P::validate(&mut Validator::new(buf.as_ptr()))?;
            }

            // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
            // type being coerced into can really inhabit any bit pattern.
            Ok(buf.cast_mut())
        }
    }

    /// Access the underlying slice as a pointer.
    #[inline]
    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// The numerical range of the buffer.
    #[inline]
    pub(crate) fn range(&self) -> Range<usize> {
        let range = self.data.as_ptr_range();
        range.start as usize..range.end as usize
    }

    /// Test if the current buffer is compatible with the given layout.
    #[inline]
    pub(crate) fn is_compatible(&self, layout: Layout) -> bool {
        // SAFETY: Layout::align is a power of two.
        unsafe {
            self.is_aligned_with_unchecked(layout.align()) && self.data.len() >= layout.size()
        }
    }

    /// Test if the current allocation uses the alignment of `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// #[repr(align(4096))]
    /// struct Align4096;
    ///
    /// let buf = OwnedBuf::new();
    /// assert!(buf.is_aligned::<u32>());
    /// // NB: We might have gotten lucky and hit a wide alignment by chance.
    /// assert!(buf.is_aligned::<Align4096>() || !buf.is_aligned::<Align4096>());
    /// ```
    #[inline]
    pub fn is_aligned<T>(&self) -> bool {
        unsafe { crate::buf::is_aligned_with(self.as_ptr(), align_of::<T>()) }
    }

    /// Test if the current allocation uses the specified alignment.
    ///
    /// # Panics
    ///
    /// Panics if the specified alignment is not a power of two.
    ///
    /// ```should_panic
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// buf.is_aligned_with(0);
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::OwnedBuf;
    ///
    /// let buf = OwnedBuf::new();
    /// assert!(buf.is_aligned_with(8));
    /// ```
    #[inline]
    pub fn is_aligned_with(&self, align: usize) -> bool {
        assert!(align.is_power_of_two(), "Alignment is not a power of two");
        // SAFETY: align is a power of two.
        unsafe { crate::buf::is_aligned_with(self.as_ptr(), align) }
    }

    #[inline]
    pub(crate) unsafe fn is_aligned_with_unchecked(&self, align: usize) -> bool {
        crate::buf::is_aligned_with(self.as_ptr(), align)
    }
}

impl fmt::Debug for Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Buf").field(&self.data.len()).finish()
    }
}

#[cfg(feature = "alloc")]
impl ToOwned for Buf {
    type Owned = OwnedBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        let mut buf =
            unsafe { OwnedBuf::with_capacity_and_custom_alignment(self.len(), self.alignment()) };

        buf.extend_from_slice(&self.data);
        buf
    }
}

impl AsRef<Buf> for Buf {
    /// Trivial `AsRef<Buf>` implementation for `Buf`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let buf = Buf::new(&b"Hello World!"[..]);
    /// let buf = buf.as_ref();
    ///
    /// assert_eq!(&buf[..], b"Hello World!");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn as_ref(&self) -> &Buf {
        self
    }
}

impl AsMut<Buf> for Buf {
    /// Trivial `AsMut<Buf>` implementation for `Buf`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Buf;
    ///
    /// let mut bytes = *b"Hello World!";
    /// let buf = Buf::new_mut(&mut bytes[..]);
    /// let buf = buf.as_mut();
    ///
    /// buf[..5].make_ascii_uppercase();
    /// assert_eq!(&buf[..], b"HELLO World!");
    /// buf[11] = b'?';
    /// assert_eq!(&buf[..], b"HELLO World?");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    fn as_mut(&mut self) -> &mut Buf {
        self
    }
}

/// Index implementation to get a slice or individual byte out of a [`Buf`].
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Buf;
///
/// let buf = Buf::new(b"Hello World!");
///
/// assert_eq!(&buf[..], &b"Hello World!"[..]);
/// assert_eq!(buf[0], b'H');
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
impl<I> Index<I> for Buf
where
    I: SliceIndex<[u8]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &I::Output {
        &self.data[index]
    }
}

/// Index implementation to get a mutable slice or individual byte out of a
/// [`Buf`].
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Buf;
///
/// let mut bytes = *b"Hello World!";
/// let mut buf = Buf::new_mut(&mut bytes[..]);
///
/// buf[..5].make_ascii_uppercase();
///
/// assert_eq!(&buf[..], &b"HELLO World!"[..]);
/// buf[11] = b'?';
/// assert_eq!(&buf[..], &b"HELLO World?"[..]);
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
impl<I> IndexMut<I> for Buf
where
    I: SliceIndex<[u8]>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        &mut self.data[index]
    }
}
