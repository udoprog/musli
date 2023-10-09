use core::alloc::Layout;
use core::fmt;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::Range;
use core::slice;

use crate::bind::Bindable;
use crate::error::{Error, ErrorKind};
use crate::r#ref::Ref;
use crate::r#unsized::Unsized;
use crate::slice::Slice;
use crate::zero_copy::{UnsizedZeroCopy, ZeroCopy};

/// A mutable buffer to write zero copy types to.
///
/// This is implemented by [`OwnedBuf`].
///
/// [`OwnedBuf`]: crate::OwnedBuf
pub trait BufMut {
    /// Extend the current buffer from the given slice.
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error>;

    /// Write the given zero copy type to the buffer.
    fn write<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy;
}

impl<B: ?Sized> BufMut for &mut B
where
    B: BufMut,
{
    fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<(), Error> {
        (**self).extend_from_slice(bytes)
    }

    fn write<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        (**self).write(value)
    }
}

/// Trait used for loading any kind of reference.
///
/// # Safety
///
/// This can only be implemented correctly by types under certain conditions:
/// * The type has a strict, well-defined layout or is `repr(C)`.
pub unsafe trait AnyRef {
    /// The target being read.
    type Target: ?Sized;

    /// Validate the value.
    fn read_from<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error>;
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
    T: AnyRef,
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
unsafe impl<T: ?Sized> AnyRef for &T
where
    T: AnyRef,
{
    type Target = T::Target;

    #[inline]
    fn read_from<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        T::read_from(self, buf)
    }
}

unsafe impl<T: ?Sized> AnyRef for Unsized<T>
where
    T: UnsizedZeroCopy,
{
    type Target = T;

    fn read_from<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_unsized(*self)
    }
}

unsafe impl<T: ?Sized> AnyRef for Ref<T>
where
    T: ZeroCopy,
{
    type Target = T;

    fn read_from<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_sized(*self)
    }
}

unsafe impl<T> AnyRef for Slice<T>
where
    T: ZeroCopy,
{
    type Target = [T];

    fn read_from<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
        buf.load_slice(*self)
    }
}

/// A raw slice buffer.
#[repr(transparent)]
pub struct Buf {
    data: [u8],
}

impl Buf {
    /// Wrap the given slice as a buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is aligned per the requirements
    /// of the types you intend to read from it.
    pub unsafe fn new_unchecked<T>(data: &T) -> &Buf
    where
        T: ?Sized + AsRef<[u8]>,
    {
        // SAFETY: The struct is repr(transparent) over [u8].
        unsafe { &*(data.as_ref() as *const _ as *const Buf) }
    }

    /// Get the underlying bytes of the buffer.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Access the underlying slice as a pointer.
    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// The numerical range of the buffer.
    pub(crate) fn range(&self) -> Range<usize> {
        let range = self.data.as_ptr_range();
        range.start as usize..range.end as usize
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
        // SAFETY: We specifically test for alignment.
        unsafe {
            let data = self.get_unchecked(range)?;

            if !data.is_aligned_to(align) {
                return Err(Error::new(ErrorKind::BadAlignment {
                    ptr: data.as_ptr() as usize,
                    align,
                }));
            }

            Ok(data)
        }
    }

    /// Get the given range without checking that it corresponds to any given alignment.
    pub(crate) unsafe fn get_unchecked(&self, range: Range<usize>) -> Result<&Buf, Error> {
        let Some(data) = self.data.get(range.clone()) else {
            return Err(Error::new(ErrorKind::OutOfRangeBounds {
                range,
                len: self.data.len(),
            }));
        };

        Ok(Buf::new_unchecked(data))
    }

    /// Load an unsized reference.
    ///
    /// # Examples
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
        T::read_from(self.get(start..end, T::ALIGN)?)
    }

    /// Load the given sized value as a reference.
    pub fn load_sized<T: ?Sized>(&self, ptr: Ref<T>) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(size_of::<T>());
        T::read_from(self.get(start..end, align_of::<T>())?)
    }

    /// Load the given slice.
    pub fn load_slice<T>(&self, ptr: Slice<T>) -> Result<&[T], Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len().wrapping_mul(size_of::<T>()));
        let buf = self.get(start..end, align_of::<T>())?;
        validate_array::<T>(buf, ptr.len())?;
        Ok(unsafe { slice::from_raw_parts(buf.as_ptr().cast(), ptr.len()) })
    }

    /// Load the given value as a reference.
    pub fn load<T>(&self, ptr: T) -> Result<&T::Target, Error>
    where
        T: AnyRef,
    {
        ptr.read_from(self)
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
    /// use musli_zerocopy::{OwnedBuf, Pair};
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let mut map = Vec::new();
    ///
    /// map.push(Pair::new(1, 2));
    /// map.push(Pair::new(2, 3));
    ///
    /// let map = buf.insert_map(&mut map)?;
    /// let buf = buf.as_aligned_buf();
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
    /// The caller must ensure that the buffer is correctly sized and aligned for the destination type.
    pub unsafe fn cast<T>(&self) -> &T {
        &*self.data.as_ptr().cast()
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
    /// use musli_zerocopy::{OwnedBuf, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let custom = buf.insert_sized(Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// let buf = buf.as_aligned_buf();
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
        if !self.is_compatible(Layout::new::<T>()) {
            return Err(Error::new(ErrorKind::LayoutMismatch {
                layout: Layout::new::<T>(),
                buf: self.range(),
            }));
        }

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
    /// use musli_zerocopy::{OwnedBuf, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let custom = buf.insert_sized(Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// let buf = buf.as_aligned_buf();
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
            let data = self.data.get_unchecked(start..end)?;
            F::validate_aligned(data)?;
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
    /// use musli_zerocopy::{OwnedBuf, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let custom = buf.insert_sized(Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// buf.extend_from_slice(&[0]);
    /// let buf = buf.as_aligned_buf();
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
    /// use musli_zerocopy::{OwnedBuf, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     field: u32,
    ///     field2: u64,
    /// }
    ///
    /// let mut buf = OwnedBuf::new();
    ///
    /// let custom = buf.insert_sized(Custom {
    ///     field: 42,
    ///     field2: 85,
    /// })?;
    /// let buf = buf.as_aligned_buf();
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
                expected: offset,
                len: self.data.len(),
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
            buf: buf.range(),
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
        for chunk in buf.as_bytes().chunks_exact(size_of::<T>()) {
            // SAFETY: The passed in buffer is required to be aligned per the
            // requirements of this trait, so any size_of::<T>() chunks are aligned
            // too.
            unsafe {
                T::validate_aligned(Buf::new_unchecked(chunk))?;
            }
        }
    }

    Ok(())
}

pub(crate) fn is_aligned_to(ptr: *const u8, align: usize) -> bool {
    assert!(align.is_power_of_two(), "alignment is not a power-of-two");
    (ptr as usize) & (align - 1) == 0
}
