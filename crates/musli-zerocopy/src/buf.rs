use core::alloc::Layout;
use core::fmt;
use core::ops::Range;

use crate::error::{Error, ErrorKind};
use crate::ptr::Ptr;
use crate::ref_::Ref;
use crate::to_buf::{UnsizedZeroCopy, ZeroCopy};
use crate::unsized_ref::UnsizedRef;

/// Trait used for any kind of pointer or reference.
pub trait AnyRef {
    /// The target of the pointer.
    type Target: ?Sized;

    /// The base pointer.
    fn ptr(&self) -> Ptr;

    /// The length of the value to load.
    fn len(&self) -> usize;

    /// Validate the value.
    fn validate(buf: &Buf) -> Result<&Self::Target, Error>;
}

impl<T: ?Sized> AnyRef for UnsizedRef<T>
where
    T: UnsizedZeroCopy,
{
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

impl<T: ?Sized> AnyRef for Ref<T>
where
    T: ZeroCopy,
{
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

/// A raw slice buffer.
#[repr(transparent)]
pub struct Buf {
    data: [u8],
}

impl Buf {
    /// Wrap the given slice as a buffer.
    pub fn new<T>(data: &T) -> &Buf
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

    /// The numerical range of the buffer.
    pub fn range(&self) -> Range<usize> {
        let range = self.data.as_ptr_range();
        range.start as usize..range.end as usize
    }

    /// Test if the current buffer is compatible with the given layout.
    pub fn is_compatible(&self, layout: Layout) -> bool {
        self.data.as_ptr() as usize % layout.align() == 0 && self.data.len() == layout.size()
    }

    /// Get the length of the current buffer.
    pub fn len(&self) -> usize {
        self.data.len()
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

        let Some(d) = self.data.get(start..end) else {
            return Err(Error::new(ErrorKind::OutOfBounds {
                start,
                end,
                len: self.data.len(),
            }));
        };

        T::validate(Buf::new(d))
    }

    /// Load the given sized value as a reference.
    pub fn load_sized<T: ?Sized>(&self, ptr: Ref<T>) -> Result<&T, Error>
    where
        T: ZeroCopy,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(T::SIZE);

        let Some(d) = self.data.get(start..end) else {
            return Err(Error::new(ErrorKind::OutOfBounds {
                start,
                end,
                len: self.data.len(),
            }));
        };

        T::validate(Buf::new(d))
    }

    /// Load the given value as a reference.
    pub fn load<T>(&self, ptr: T) -> Result<&T::Target, Error>
    where
        T: AnyRef,
    {
        let start = ptr.ptr().offset();
        let end = start.wrapping_add(ptr.len());

        let Some(d) = self.data.get(start..end) else {
            return Err(Error::new(ErrorKind::OutOfBounds {
                start,
                end,
                len: self.data.len(),
            }));
        };

        T::validate(Buf::new(d))
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
            return Err(Error::layout_mismatch(Layout::new::<T>(), self.range()));
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
        T::validate(Buf::new(head))?;
        self.data = tail;
        Ok(())
    }

    /// Ensure that the buffer is empty.
    pub fn finalize(self) -> Result<(), Error> {
        if self.data.len() > 0 {
            return Err(Error::new(ErrorKind::BufferUnderflow {
                remaining: self.data.len(),
            }));
        }

        Ok(())
    }
}
