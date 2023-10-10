use core::alloc::Layout;
use core::fmt;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::Range;
use core::slice;

use crate::aligned_buf::StructWriter;
use crate::bind::Bindable;
use crate::error::{Error, ErrorKind};
use crate::r#ref::Ref;
use crate::r#unsized::Unsized;
use crate::slice::Slice;
use crate::zero_copy::{UnsizedZeroCopy, ZeroCopy};

/// A mutable buffer to write zero copy types to.
///
/// This is implemented by [`AlignedBuf`].
///
/// [`AlignedBuf`]: crate::AlignedBuf
pub trait BufMut {
    /// Extend the current buffer from the given slice.
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error>;

    /// Write the given zero copy type to the buffer.
    fn write<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy;

    /// Setup a writer for the given type.
    ///
    /// This API writes the type directly using an unaligned pointer write and
    /// just ensures that any padding is zeroed.
    ///
    /// # Safety
    ///
    /// While calling just this function is not unsafe, finishing writing with
    /// [`StructWriter::finish`] is unsafe.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{ZeroCopy, AlignedBuf, BufMut};
    ///
    /// #[derive(Debug, PartialEq, Eq, ZeroCopy)]
    /// #[repr(C)]
    /// struct ZeroPadded {
    ///     a: u8,
    ///     b: u64,
    ///     c: u16,
    ///     d: u32,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let padded = ZeroPadded {
    ///     a: 0x01u8,
    ///     b: 0x0203_0405_0607_0809u64,
    ///     c: 0x0e0fu16,
    ///     d: 0x0a0b_0c0du32,
    /// };
    ///
    /// let mut w = buf.writer(&padded);
    /// w.pad::<u8>();
    /// w.pad::<u64>();
    /// w.pad::<u16>();
    /// w.pad::<u32>();
    ///
    /// // SAFETY: We've asserted that the struct fields have been correctly padded.
    /// let ptr = unsafe { w.finish()? };
    ///
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf.as_slice(), &[1, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0, 12, 13, 14, 15]);
    /// } else {
    ///     assert_eq!(buf.as_slice(), &[1, 0, 0, 0, 0, 0, 0, 0, 9, 8, 7, 6, 5, 4, 3, 2, 15, 14, 0, 0, 13, 12, 11, 10]);
    /// }
    ///
    /// let buf = buf.as_aligned();
    ///
    /// assert_eq!(buf.load(ptr)?, &padded);
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    fn writer<T>(&mut self, value: &T) -> StructWriter<'_, T>
    where
        T: ZeroCopy;
}

impl<B: ?Sized> BufMut for &mut B
where
    B: BufMut,
{
    #[inline]
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error> {
        (**self).extend_from_slice(bytes)
    }

    #[inline]
    fn write<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        (**self).write(value)
    }

    #[inline]
    fn writer<T>(&mut self, value: &T) -> StructWriter<'_, T>
    where
        T: ZeroCopy,
    {
        (**self).writer(value)
    }
}

/// Trait used for loading any kind of reference.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
pub unsafe trait Load {
    /// The target being read.
    type Target: ?Sized;

    /// Validate the value.
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error>;
}

/// Trait used for loading any kind of reference.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
pub unsafe trait LoadMut: Load {
    /// Validate the value.
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error>;
}

/// Trait used for handling any kind of zero copy value, be they references or
/// not.
pub trait AnyValue {
    /// The target being read.
    type Target: ?Sized;

    /// Validate the value.
    fn visit<V, O>(&self, buf: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O;
}

impl<T: ?Sized> AnyValue for T
where
    T: Load,
{
    type Target = T::Target;

    fn visit<V, O>(&self, buf: &Buf, visitor: V) -> Result<O, Error>
    where
        V: FnOnce(&Self::Target) -> O,
    {
        let value = buf.load(self)?;
        Ok(visitor(value))
    }
}

// SAFETY: Blanket implementation is fine over known sound implementations.
unsafe impl<T: ?Sized> Load for &T
where
    T: Load,
{
    type Target = T::Target;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        T::load(self, buf)
    }
}

// SAFETY: Blanket implementation is fine over known sound implementations.
unsafe impl<T: ?Sized> Load for &mut T
where
    T: Load,
{
    type Target = T::Target;

    #[inline]
    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        T::load(self, buf)
    }
}

// SAFETY: Blanket implementation is fine over known sound implementations.
unsafe impl<T: ?Sized> LoadMut for &mut T
where
    T: LoadMut,
{
    #[inline]
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        T::load_mut(self, buf)
    }
}

unsafe impl<T: ?Sized> Load for Unsized<T>
where
    T: UnsizedZeroCopy,
{
    type Target = T;

    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

unsafe impl<T: ?Sized> LoadMut for Unsized<T>
where
    T: UnsizedZeroCopy,
{
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_unsized_mut(*self)
    }
}

unsafe impl<T> Load for Ref<T>
where
    T: ZeroCopy,
{
    type Target = T;

    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_sized(*self)
    }
}

unsafe impl<T> LoadMut for Ref<T>
where
    T: ZeroCopy,
{
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_sized_mut(*self)
    }
}

unsafe impl<T> Load for Slice<T>
where
    T: ZeroCopy,
{
    type Target = [T];

    fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_slice(*self)
    }
}

unsafe impl<T> LoadMut for Slice<T>
where
    T: ZeroCopy,
{
    fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
        buf.load_slice_mut(*self)
    }
}

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

    /// Test if the current buffer is compatible with the given layout.
    pub(crate) fn is_compatible(&self, layout: Layout) -> bool {
        self.is_aligned_to(layout.align()) && self.data.len() == layout.size()
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

    /// Get the length of the current buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Test if the current buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the given range while checking its required alignment.
    pub(crate) fn get(&self, range: Range<usize>, align: usize) -> Result<&Buf, Error> {
        let buf = self.get_unaligned(range)?;

        if !buf.is_aligned_to(align) {
            return Err(Error::new(ErrorKind::AlignmentMismatch {
                range: self.range(),
                align,
            }));
        }

        Ok(buf)
    }

    /// Get the given range mutably while checking its required alignment.
    pub(crate) fn get_mut(&mut self, range: Range<usize>, align: usize) -> Result<&mut Buf, Error> {
        let buf = self.get_mut_unaligned(range)?;

        if !buf.is_aligned_to(align) {
            return Err(Error::new(ErrorKind::AlignmentMismatch {
                range: buf.range(),
                align,
            }));
        }

        Ok(buf)
    }

    /// Get the given range without checking that it corresponds to any given alignment.
    pub(crate) fn get_unaligned(&self, range: Range<usize>) -> Result<&Buf, Error> {
        let Some(data) = self.data.get(range.clone()) else {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range,
                len: self.data.len(),
            }));
        };

        Ok(Buf::new(data))
    }
    /// Get the given range mutably without checking that it corresponds to any given alignment.
    pub(crate) fn get_mut_unaligned(&mut self, range: Range<usize>) -> Result<&mut Buf, Error> {
        let len = self.data.len();

        let Some(data) = self.data.get_mut(range.clone()) else {
            return Err(Error::new(ErrorKind::OutOfRangeBounds { range, len }));
        };

        Ok(Buf::new_mut(data))
    }

    /// Load an unsized reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let first = buf.write_unsized("first")?;
    /// let second = buf.write_unsized("second")?;
    ///
    /// let buf = buf.as_ref()?;
    ///
    /// assert_eq!(buf.load_unsized(first)?, "first");
    /// assert_eq!(buf.load_unsized(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn load_unsized<T: ?Sized>(&self, ptr: Unsized<T>) -> Result<&T, Error>
    where
        T: UnsizedZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.size());
        let buf = self.get(start..end, T::ALIGN)?;
        T::coerce(buf)
    }

    /// Load an unsized mutable reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::AlignedBuf;
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let first = buf.write_unsized("first")?;
    /// let second = buf.write_unsized("second")?;
    ///
    /// let buf = buf.as_mut()?;
    ///
    /// buf.load_unsized_mut(first)?.make_ascii_uppercase();
    ///
    /// assert_eq!(buf.load_unsized(first)?, "FIRST");
    /// assert_eq!(buf.load_unsized(second)?, "second");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    pub fn load_unsized_mut<T: ?Sized>(&mut self, ptr: Unsized<T>) -> Result<&mut T, Error>
    where
        T: UnsizedZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.size());
        let buf = self.get_mut(start..end, T::ALIGN)?;
        T::coerce_mut(buf)
    }

    /// Load the given sized value as a reference.
    pub fn load_sized<T>(&self, ptr: Ref<T>) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(size_of::<T>());
        let buf = self.get(start..end, align_of::<T>())?;

        if T::ANY_BITS {
            // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
            // type being coerced into can really inhabit any bit pattern.
            Ok(unsafe { buf.cast() })
        } else {
            T::coerce(buf)
        }
    }

    /// Load the given sized value as a mutable reference.
    pub fn load_sized_mut<T>(&mut self, ptr: Ref<T>) -> Result<&mut T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(size_of::<T>());
        let buf = self.get_mut(start..end, align_of::<T>())?;

        if T::ANY_BITS {
            // SAFETY: Implementing ANY_BITS is unsafe, and requires that the
            // type being coerced into can really inhabit any bit pattern.
            Ok(unsafe { buf.cast_mut() })
        } else {
            T::coerce_mut(buf)
        }
    }

    /// Load the given slice.
    pub fn load_slice<T>(&self, ptr: Slice<T>) -> Result<&[T], Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len().wrapping_mul(size_of::<T>()));
        let buf = self.get_unaligned(start..end)?;
        validate_array::<T>(buf, ptr.len())?;
        Ok(unsafe { slice::from_raw_parts(buf.as_ptr().cast(), ptr.len()) })
    }

    /// Load the given slice mutably.
    pub fn load_slice_mut<T>(&mut self, ptr: Slice<T>) -> Result<&mut [T], Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len().wrapping_mul(size_of::<T>()));
        let buf = self.get_mut_unaligned(start..end)?;
        validate_array::<T>(buf, ptr.len())?;
        Ok(unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr().cast(), ptr.len()) })
    }

    /// Load the given value as a reference.
    pub fn load<T>(&self, ptr: T) -> Result<&T::Target, Error>
    where
        T: Load,
    {
        ptr.load(self)
    }

    /// Load the given value as a mutable reference.
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
    /// let custom = buf.write(&Custom {
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

    /// Construct a validator over the current buffer which assumes it's
    /// correctly aligned.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is appropriately aligned.
    pub unsafe fn validate_unchecked<T>(&self) -> Result<Validator<'_, T>, Error>
    where
        T: ZeroCopy,
    {
        Ok(Validator {
            data: self,
            offset: 0,
            _marker: PhantomData,
        })
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
    /// let custom = buf.write(&Custom {
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
            let data = self.data.get_unaligned(start..end)?;
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
    /// let custom = buf.write(&Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// buf.extend_from_slice(&[0]);
    /// let buf = buf.as_aligned();
    ///
    /// // We can only cause the error if we assert that the buffer is aligned.
    /// let mut v = unsafe { buf.validate_unchecked::<Custom>()? };
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
    /// let custom = buf.write(&Custom {
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

pub(crate) fn validate_array<T>(buf: &Buf, len: usize) -> Result<(), Error>
where
    T: ZeroCopy,
{
    let layout =
        Layout::array::<T>(len).map_err(|error| Error::new(ErrorKind::LayoutError { error }))?;

    if !buf.is_compatible(layout) {
        return Err(Error::new(ErrorKind::LayoutMismatch {
            layout,
            range: buf.range(),
        }));
    }

    validate_array_aligned::<T>(buf)?;
    Ok(())
}

pub(crate) fn validate_array_aligned<T>(buf: &Buf) -> Result<(), Error>
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
