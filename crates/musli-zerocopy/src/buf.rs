use core::alloc::Layout;
use core::fmt;
use core::ops::Range;
use core::slice;

use crate::error::{Error, ErrorKind};
use crate::owned_buf::is_aligned_to;
use crate::ptr::Ptr;
use crate::ref_::Ref;
use crate::slice_ref::SliceRef;
use crate::unsized_ref::UnsizedRef;
use crate::zero_copy::{UnsizedZeroCopy, ZeroCopy};

/// Trait used for any kind of pointer or reference.
pub unsafe trait AnyRef {
    const ALIGN: usize;

    /// The target of the pointer.
    type Target: ?Sized;

    /// The base pointer.
    fn ptr(&self) -> Ptr;

    /// The length of the value to load.
    ///
    /// # Safety
    ///
    /// Must be a multiple of align.
    fn len(&self) -> usize;

    /// Validate the value.
    fn validate(buf: &Buf) -> Result<&Self::Target, Error>;
}

unsafe impl<T: ?Sized> AnyRef for UnsizedRef<T>
where
    T: UnsizedZeroCopy,
{
    const ALIGN: usize = T::ALIGN;

    type Target = T;

    fn ptr(&self) -> Ptr {
        UnsizedRef::ptr(self)
    }

    fn len(&self) -> usize {
        UnsizedRef::len(self)
    }

    fn validate(buf: &Buf) -> Result<&Self::Target, Error> {
        T::validate(buf)
    }
}

unsafe impl<T: ?Sized> AnyRef for Ref<T>
where
    T: ZeroCopy,
{
    const ALIGN: usize = T::ALIGN;

    type Target = T;

    fn ptr(&self) -> Ptr {
        Ref::ptr(self)
    }

    fn len(&self) -> usize {
        T::SIZE
    }

    fn validate(buf: &Buf) -> Result<&Self::Target, Error> {
        T::validate(buf)
    }
}

unsafe impl<T> AnyRef for SliceRef<T>
where
    T: ZeroCopy,
{
    const ALIGN: usize = T::ALIGN;

    type Target = [T];

    fn ptr(&self) -> Ptr {
        SliceRef::ptr(self)
    }

    fn len(&self) -> usize {
        SliceRef::len(self).wrapping_mul(T::SIZE)
    }

    fn validate(buf: &Buf) -> Result<&Self::Target, Error> {
        let len = buf.len().wrapping_div(T::SIZE);

        for chunk in buf.as_bytes().chunks_exact(T::SIZE) {
            // SAFETY: The passed in buffer is required to be aligned per the
            // requirements of this trait, so any T::SIZE chunks are aligned
            // too.
            T::validate(unsafe { Buf::new_unchecked(chunk) })?;
        }

        Ok(unsafe { slice::from_raw_parts(buf.as_ptr().cast(), len) })
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

    /// Get the given range while checking its required alignment.
    fn get(&self, range: Range<usize>, align: usize) -> Result<&Buf, Error> {
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
        T::validate(self.get(start..end, T::ALIGN)?)
    }

    /// Load the given sized value as a reference.
    pub fn load_sized<T: ?Sized>(&self, ptr: Ref<T>) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(T::SIZE);
        T::validate(self.get(start..end, T::ALIGN)?)
    }

    /// Load the given value as a reference.
    pub fn load<T>(&self, ptr: T) -> Result<&T::Target, Error>
    where
        T: AnyRef,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len());
        T::validate(self.get(start..end, T::ALIGN)?)
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
    pub fn validator<T>(&self) -> Result<Validator<'_>, Error> {
        if !self.is_compatible(Layout::new::<T>()) {
            return Err(Error::new(ErrorKind::LayoutMismatch {
                layout: Layout::new::<T>(),
                buf: self.range(),
            }));
        }

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
    pub fn validate<T>(&mut self) -> Result<(), Error>
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
        T::validate(unsafe { Buf::new_unchecked(head) })?;
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
