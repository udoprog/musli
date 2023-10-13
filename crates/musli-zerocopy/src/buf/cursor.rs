use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ops::Range;
use core::ptr::NonNull;

use crate::buf::Validator;
use crate::traits::ZeroCopy;

/// A validation cursor used over a buffer.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Cursor<'a> {
    pointer: NonNull<u8>,
    _marker: PhantomData<&'a [u8]>,
}

impl<'a> Cursor<'a> {
    /// Construct a new cursor from a checked slice.
    pub(crate) fn new(data: &'a [u8]) -> Cursor<'a> {
        // SAFETY: The pointer is guaranteed to be non-null.
        unsafe {
            Self {
                pointer: NonNull::new_unchecked(data.as_ptr() as *mut _),
                _marker: PhantomData,
            }
        }
    }

    /// Construct a validator over the current buffer.
    ///
    /// This is a struct validator, which checks that the fields specified in
    /// order of subsequent calls to [`field`] conform to the `repr(C)`
    /// representation.
    ///
    /// [`field`]: Validator::field
    ///
    /// # Safety
    ///
    /// The returned validator will have an unbounded lifetime, so it's up to
    /// the caller to ensure that the it's not coerced into something
    /// inappropriate.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
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
    /// });
    ///
    /// let buf = buf.as_aligned();
    ///
    /// unsafe {
    ///     let mut v = buf.validate_struct::<Custom>()?;
    ///     v.field::<u32>()?;
    ///     v.field::<u64>()?;
    /// }
    ///
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub unsafe fn validate_struct<T>(self) -> Validator<'a, T>
    where
        T: ZeroCopy,
    {
        Validator::new(self)
    }

    /// Raw advance function.
    #[inline]
    pub(crate) unsafe fn advance_raw(&mut self, len: usize) {
        self.pointer = NonNull::new_unchecked(self.pointer.as_ptr().add(len));
    }

    /// Advance the cursor by the size of `T`.
    ///
    /// # Safety
    ///
    /// Caller must ensure that advancing the pointer by size of `T` doesn't
    /// wrap around the address space.
    #[inline]
    pub unsafe fn advance<T>(&mut self) {
        self.advance_raw(size_of::<T>());
    }

    /// Get the align offset needed to align `T`.
    pub(crate) unsafe fn align_offset<T>(&self) -> usize {
        self.pointer.as_ptr().align_offset(align_of::<T>())
    }

    /// Align the pointer to the alignment needed by type `T`.
    ///
    /// # Safety
    ///
    /// Caller must ensure that advancing the pointer to the alignment of `T`
    /// doesn't wrap around the address space.
    #[inline]
    pub unsafe fn align<T>(&mut self) -> usize {
        let offset = self.align_offset::<T>();

        if offset > 0 {
            self.advance_raw(offset);
        }

        offset
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
    ///
    /// This also returns an unbounded lifetime, which the caller is required to
    /// ensure doesn't get coerced to something inappropriate.
    #[inline]
    pub unsafe fn cast<T>(self) -> &'a T {
        &*(self.pointer.as_ptr() as *const u8).cast()
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
    ///
    /// This also returns an unbounded lifetime, which the caller is required to
    /// ensure doesn't get coerced to something inappropriate.
    #[inline]
    pub unsafe fn cast_mut<T>(self) -> &'a mut T {
        &mut *self.pointer.as_ptr().cast()
    }

    /// Get the range corresponding to the cursor.
    pub(crate) fn range<T>(&self) -> Range<usize> {
        let start = self.pointer.as_ptr() as usize;
        let end = start.wrapping_add(size_of::<T>());
        start..end
    }
}
