use core::alloc::Layout;
use core::fmt;
use core::ops::Range;
use core::slice;

use crate::error::{Error, ErrorKind};
use crate::owned_buf::is_aligned_to;
use crate::ref_::Ref;
use crate::slice_ref::SliceRef;
use crate::unsized_ref::UnsizedRef;
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

unsafe impl<T: ?Sized> AnyRef for UnsizedRef<T>
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

unsafe impl<T> AnyRef for SliceRef<T>
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

    /// Access the underlying slice as a pointer.
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get the underlying bytes of the buffer.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// The numerical range of the buffer.
    pub fn range(&self) -> Range<usize> {
        let range = self.data.as_ptr_range();
        range.start as usize..range.end as usize
    }

    /// Test if the current buffer is compatible with the given layout.
    pub fn is_compatible(&self, layout: Layout) -> bool {
        self.is_aligned_to(layout.align()) && self.data.len() == layout.size()
    }

    pub fn is_aligned_to(&self, align: usize) -> bool {
        is_aligned_to(self.data.as_ptr(), align)
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
    pub fn get(&self, range: Range<usize>, align: usize) -> Result<&Buf, Error> {
        let Some(data) = self.data.get(range.clone()) else {
            return Err(Error::new(ErrorKind::OutOfBounds {
                range,
                len: self.data.len(),
            }));
        };

        if !is_aligned_to(data.as_ptr(), align) {
            return Err(Error::new(ErrorKind::BadAlignment {
                ptr: data.as_ptr() as usize,
                align,
            }));
        }

        // SAFETY: Alignment is tested for above.
        Ok(unsafe { Buf::new_unchecked(data) })
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
    pub fn load_unsized<T: ?Sized>(&self, ptr: UnsizedRef<T>) -> Result<&T, Error>
    where
        T: UnsizedZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len());
        T::read_from(self.get(start..end, T::ALIGN)?)
    }

    /// Load the given sized value as a reference.
    pub fn load_sized<T: ?Sized>(&self, ptr: Ref<T>) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(T::SIZE);
        T::read_from(self.get(start..end, T::ALIGN)?)
    }

    /// Load the given slice.
    pub fn load_slice<T>(&self, ptr: SliceRef<T>) -> Result<&[T], Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len().wrapping_mul(T::SIZE));
        let buf = self.get(start..end, T::ALIGN)?;

        let len = buf.len().wrapping_div(T::SIZE);

        // Only validate each element if they cannot inhabit all possible bit
        // patterns.
        if !T::ANY_BITS {
            for chunk in buf.as_bytes().chunks_exact(T::SIZE) {
                // SAFETY: The passed in buffer is required to be aligned per the
                // requirements of this trait, so any T::SIZE chunks are aligned
                // too.
                unsafe {
                    T::validate_aligned(Buf::new_unchecked(chunk))?;
                }
            }
        }

        Ok(unsafe { slice::from_raw_parts(buf.as_ptr().cast(), len) })
    }

    /// Load the given value as a reference.
    pub fn load<T>(&self, ptr: T) -> Result<&T::Target, Error>
    where
        T: AnyRef,
    {
        ptr.read_from(self)
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
    pub fn validate<T>(&self) -> Result<Validator<'_>, Error> {
        if !self.is_compatible(Layout::new::<T>()) {
            return Err(Error::new(ErrorKind::LayoutMismatch {
                layout: Layout::new::<T>(),
                buf: self.range(),
            }));
        }

        Ok(Validator { data: &self.data })
    }

    /// Construct a validator over the current buffer which assumes it's
    /// correctly aligned.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer is appropriately aligned.
    pub unsafe fn validate_aligned(&self) -> Result<Validator<'_>, Error> {
        Ok(Validator { data: &self.data })
    }
}

impl fmt::Debug for Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Buf").field(&self.data.len()).finish()
    }
}

/// Validator over a [Buf].
pub struct Validator<'a> {
    data: &'a [u8],
}

impl Validator<'_> {
    /// Validate the given type.
    pub fn field<T>(&mut self) -> Result<(), Error>
    where
        T: ZeroCopy,
    {
        let rem = (self.data.as_ptr() as usize) % T::ALIGN;

        if rem != 0 {
            let start = T::ALIGN - rem;

            let Some(d) = self.data.get(start..) else {
                return Err(Error::new(ErrorKind::OutOfStartBound {
                    start,
                    len: self.data.len(),
                }));
            };

            self.data = d;
        }

        if T::SIZE > self.data.len() {
            return Err(Error::new(ErrorKind::OutOfStartBound {
                start: T::SIZE,
                len: self.data.len(),
            }));
        }

        let (head, tail) = self.data.split_at(T::SIZE);

        // SAFETY: We've ensured that the provided buffer is aligned above.
        T::read_from(unsafe { Buf::new_unchecked(head) })?;
        self.data = tail;
        Ok(())
    }

    /// Ensure that the buffer is empty.
    pub fn finalize(self) -> Result<(), Error> {
        // NB: due to alignment and padding, the buffer might not be completely
        // consumed at this point.
        Ok(())
    }
}
