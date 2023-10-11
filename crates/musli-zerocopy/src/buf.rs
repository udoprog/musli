use core::alloc::Layout;
use core::fmt;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::Range;
use core::slice;

use crate::bind::Bindable;
use crate::error::{Error, ErrorKind};
use crate::load::{Load, LoadMut};
use crate::r#ref::Ref;
use crate::r#unsized::Unsized;
use crate::slice::Slice;
use crate::zero_copy::{UnsizedZeroCopy, ZeroCopy};
use crate::TargetSize;

/// A raw slice buffer.
#[repr(transparent)]
pub struct Buf {
    data: [u8],
}

impl Buf {
    /// Wrap the given bytes as a buffer.
    pub const fn new(data: &[u8]) -> &Buf {
        // SAFETY: The struct is repr(transparent) over [u8].
        unsafe { &*(data as *const [u8] as *const Self) }
    }

    /// Wrap the given bytes as a buffer.
    pub fn new_mut(data: &mut [u8]) -> &mut Buf {
        // SAFETY: The struct is repr(transparent) over [u8].
        unsafe { &mut *(data as *mut [u8] as *mut Self) }
    }

    /// Get the underlying bytes of the buffer.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get the underlying bytes of the buffer mutably.
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
    /// let mut buf = AlignedBuf::with_alignment(4);
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// let buf = buf.as_aligned();
    ///
    /// assert!(buf.is_compatible_with::<u32>());
    /// assert!(!buf.is_compatible_with::<u64>());
    /// ```
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
    /// let mut buf = AlignedBuf::with_alignment(4);
    /// buf.extend_from_slice(&[1, 2, 3, 4]);
    /// let buf = buf.as_aligned();
    ///
    /// assert!(buf.ensure_compatible_with::<u32>().is_ok());
    /// assert!(buf.ensure_compatible_with::<u64>().is_err());
    /// ```
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

    /// Get the length of the current buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Test if the current buffer is empty.
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
    /// let first = buf.store_unsized("first")?;
    /// let second = buf.store_unsized("second")?;
    ///
    /// let buf = buf.as_ref()?;
    ///
    /// assert_eq!(buf.load(first)?, "first");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
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
    /// let first = buf.store_unsized("first")?;
    /// let second = buf.store_unsized("second")?;
    ///
    /// let buf = buf.as_mut()?;
    ///
    /// buf.load_mut(first)?.make_ascii_uppercase();
    ///
    /// assert_eq!(buf.load(first)?, "FIRST");
    /// assert_eq!(buf.load(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn load_mut<T>(&mut self, ptr: T) -> Result<&mut T::Target, Error>
    where
        T: LoadMut,
    {
        ptr.load_mut(self)
    }

    /// Bind the current buffer to a value.
    ///
    /// This provides a more conveninent API for complex types like [`MapRef`],
    /// and makes sure that all the internals related to the type being bound
    /// has been validated and initialized.
    ///
    /// Binding a type can therefore be faster in cases where you interact with
    /// the bound type a lot because most validation associated with the type
    /// can be performed up front. But slower if you just intend to perform the
    /// casual lookup.
    ///
    /// [`MapRef`]: crate::MapRef
    ///
    /// ## Examples
    ///
    /// Binding a [`Map`] ensures that all the internals of the map have been
    /// validated:
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, Pair};
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
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
    /// [`Map`]: crate::Map
    pub fn bind<T>(&self, ptr: T) -> Result<T::Bound<'_>, Error>
    where
        T: Bindable,
    {
        ptr.bind(self)
    }

    /// Cast the current buffer into the given type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is correctly sized and aligned
    /// for the destination type.
    pub unsafe fn cast<T>(&self) -> &T {
        &*self.data.as_ptr().cast()
    }

    /// Cast the current buffer into the given mutable type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is correctly sized and aligned
    /// for the destination type.
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
    /// use musli_zerocopy::ZeroCopy;
    /// use musli_zerocopy::{AlignedBuf, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let custom = buf.store(&Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// let buf = buf.as_aligned();
    ///
    /// let mut v = buf.validate::<Custom>()?;
    /// v.field::<u32>()?;
    /// v.field::<u64>()?;
    /// v.end()?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn validate<T>(&self) -> Result<Validator<'_, T>, Error>
    where
        T: ZeroCopy,
    {
        self.ensure_compatible_with::<T>()?;

        Ok(Validator {
            data: self,
            offset: 0,
            _marker: PhantomData,
        })
    }

    /// Get the given range while checking its required alignment.
    pub(crate) fn get(&self, start: usize, end: usize, align: usize) -> Result<&Buf, Error> {
        let buf = self.get_unaligned(start, end)?;

        if !buf.is_aligned_to(align) {
            return Err(Error::new(ErrorKind::AlignmentMismatch {
                range: start..end,
                align,
            }));
        }

        Ok(buf)
    }

    /// Get the given range mutably while checking its required alignment.
    pub(crate) fn get_mut(
        &mut self,
        start: usize,
        end: usize,
        align: usize,
    ) -> Result<&mut Buf, Error> {
        let buf = self.get_mut_unaligned(start, end)?;

        if !buf.is_aligned_to(align) {
            return Err(Error::new(ErrorKind::AlignmentMismatch {
                range: start..end,
                align,
            }));
        }

        Ok(buf)
    }

    /// Get the given range without checking that it corresponds to any given alignment.
    pub(crate) fn get_unaligned(&self, start: usize, end: usize) -> Result<&Buf, Error> {
        let Some(data) = self.data.get(start..end) else {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range: start..end,
                len: self.data.len(),
            }));
        };

        Ok(Buf::new(data))
    }

    /// Get the given range mutably without checking that it corresponds to any given alignment.
    pub(crate) fn get_mut_unaligned(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<&mut Buf, Error> {
        let len = self.data.len();

        let Some(data) = self.data.get_mut(start..end) else {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range: start..end,
                len,
            }));
        };

        Ok(Buf::new_mut(data))
    }

    /// Load an unsized reference.
    pub(crate) fn load_unsized<T: ?Sized, O: TargetSize>(
        &self,
        ptr: Unsized<T, O>,
    ) -> Result<&T, Error>
    where
        T: UnsizedZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.size());
        let buf = self.get(start, end, T::ALIGN)?;

        // SAFETY: Alignment and size is checked just above when getting the
        // buffer slice.
        unsafe { T::coerce(buf) }
    }

    /// Load an unsized mutable reference.
    pub(crate) fn load_unsized_mut<T: ?Sized, O: TargetSize>(
        &mut self,
        ptr: Unsized<T, O>,
    ) -> Result<&mut T, Error>
    where
        T: UnsizedZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.size());
        let buf = self.get_mut(start, end, T::ALIGN)?;

        // SAFETY: Alignment and size is checked just above when getting the
        // buffer slice.
        unsafe { T::coerce_mut(buf) }
    }

    /// Load the given sized value as a reference.
    pub(crate) fn load_sized<T, O: TargetSize>(&self, ptr: Ref<T, O>) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(size_of::<T>());
        let buf = self.get(start, end, align_of::<T>())?;

        if T::ANY_BITS {
            // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
            // type being coerced into can really inhabit any bit pattern.
            Ok(unsafe { buf.cast() })
        } else {
            // SAFETY: Alignment and size is checked just above when getting the
            // buffer slice.
            unsafe { T::coerce(buf) }
        }
    }

    /// Load the given sized value as a mutable reference.
    pub(crate) fn load_sized_mut<T, O: TargetSize>(
        &mut self,
        ptr: Ref<T, O>,
    ) -> Result<&mut T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(size_of::<T>());
        let buf = self.get_mut(start, end, align_of::<T>())?;

        if T::ANY_BITS {
            // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
            // type being coerced into can really inhabit any bit pattern.
            Ok(unsafe { buf.cast_mut() })
        } else {
            // SAFETY: Alignment and size is checked just above when getting the
            // buffer slice.
            unsafe { T::coerce_mut(buf) }
        }
    }

    /// Load the given slice.
    pub(crate) fn load_slice<T, O: TargetSize>(&self, ptr: Slice<T, O>) -> Result<&[T], Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len().wrapping_mul(size_of::<T>()));
        let buf = self.get(start, end, align_of::<T>())?;
        validate_array::<T>(buf)?;
        Ok(unsafe { slice::from_raw_parts(buf.as_ptr().cast(), ptr.len()) })
    }

    /// Load the given slice mutably.
    pub(crate) fn load_slice_mut<T, O: TargetSize>(
        &mut self,
        ptr: Slice<T, O>,
    ) -> Result<&mut [T], Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len().wrapping_mul(size_of::<T>()));
        let buf: &mut Buf = self.get_mut_unaligned(start, end)?;
        validate_array::<T>(buf)?;
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
        is_aligned_to(self.data.as_ptr(), align)
    }

    /// Test if the entire buffer is zeroed.
    #[inline]
    pub(crate) fn is_zeroed(&self) -> bool {
        self.data.iter().all(|b| *b == 0)
    }
}

impl fmt::Debug for Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Buf").field(&self.data.len()).finish()
    }
}

/// Validator over a [`Buf`] constructed using [`Buf::validate`].
#[must_use = "Must call `Validator::end` when validation is completed"]
pub struct Validator<'a, T> {
    data: &'a Buf,
    offset: usize,
    _marker: PhantomData<T>,
}

impl<T> Validator<'_, T> {
    /// Validate an additional field in the struct.
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
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let custom = buf.store(&Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// let buf = buf.as_aligned();
    ///
    /// let mut v = buf.validate::<Custom>()?;
    /// v.field::<u32>()?;
    /// v.field::<u64>()?;
    /// v.end()?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn field<F>(&mut self) -> Result<(), Error>
    where
        F: ZeroCopy,
    {
        let start = self.offset.next_multiple_of(align_of::<F>());
        let end = start.wrapping_add(size_of::<F>());

        // SAFETY: We've ensured that the provided buffer is aligned and sized
        // appropriately above.
        unsafe {
            let data = self.data.get_unaligned(start, end)?;
            F::validate(data)?;
        };

        self.offset = end;
        Ok(())
    }

    /// Finish validation.
    ///
    /// # Errors
    ///
    /// If there is any buffer left uncomsumed at this point, this will return
    /// an [`Error`].
    ///
    /// ```
    /// use musli_zerocopy::ZeroCopy;
    /// use musli_zerocopy::{AlignedBuf, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let custom = buf.store(&Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// buf.extend_from_slice(&[0]);
    /// let buf = buf.as_aligned();
    ///
    /// // We can only cause the error if we assert that the buffer is aligned.
    /// let mut v = buf.validate::<Custom>()?;
    /// v.field::<u32>()?;
    /// v.field::<u64>()?;
    /// // Will error since the buffer is too large.
    /// assert!(v.end().is_err());
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
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
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let custom = buf.store(&Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// let buf = buf.as_aligned();
    ///
    /// let mut v = buf.validate::<Custom>()?;
    /// v.field::<u32>()?;
    /// v.field::<u64>()?;
    /// v.end()?;
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn end(self) -> Result<(), Error> {
        let offset = self.offset.next_multiple_of(size_of::<T>());

        if offset != self.data.len() {
            return Err(Error::new(ErrorKind::BufferUnderflow {
                range: self.data.range(),
                expected: offset,
            }));
        }

        Ok(())
    }
}

pub(crate) fn validate_array<T>(buf: &Buf) -> Result<(), Error>
where
    T: ZeroCopy,
{
    if !T::ANY_BITS && size_of::<T>() > 0 {
        for chunk in buf.as_slice().chunks_exact(size_of::<T>()) {
            // SAFETY: The passed in buffer is required to be aligned per the
            // requirements of this trait, so any size_of::<T>() chunks are aligned
            // too.
            unsafe {
                T::validate(Buf::new(chunk))?;
            }
        }
    }

    Ok(())
}

pub(crate) fn is_aligned_to(ptr: *const u8, align: usize) -> bool {
    assert!(align.is_power_of_two(), "alignment is not a power-of-two");
    (ptr as usize) & (align - 1) == 0
}
