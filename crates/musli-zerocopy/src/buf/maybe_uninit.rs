use core::fmt;
use core::mem::{size_of, ManuallyDrop};
use core::slice;

use crate::buf::RawBufMut;
use crate::traits::ZeroCopy;

/// A value which might or might not have been initialized.
///
/// This differs from the standard library
/// [`MaybeUninit`][core::mem::MaybeUninit] in that it's methods does not assume
/// that the value is aligned.
#[repr(C, packed(1))]
pub union MaybeUninit<T> {
    uninit: (),
    value: ManuallyDrop<T>,
}

impl<T> MaybeUninit<T> {
    /// Creates a new `MaybeUninit<T>` in an uninitialized state.
    ///
    /// Note that dropping a `MaybeUninit<T>` will never call `T`'s drop code.
    /// It is your responsibility to make sure `T` gets dropped if it got
    /// initialized.
    ///
    /// See the [type-level documentation][MaybeUninit] for some examples.
    ///
    /// # Example
    ///
    /// ```
    /// use musli_zerocopy::buf::MaybeUninit;
    ///
    /// let mut v: MaybeUninit<u32> = MaybeUninit::uninit();
    /// ```
    pub const fn uninit() -> Self {
        MaybeUninit { uninit: () }
    }

    /// Write a value to the current location being pointed to.
    ///
    /// Note that we cannot return a reference to the written value, because it
    /// might not be aligned.
    ///
    /// We can however return the underlying bytes that were written because of
    /// this type, since they are now initialized.
    ///
    /// See the [type-level documentation][MaybeUninit] for some examples.
    ///
    /// # Example
    ///
    /// Writing to an uninitialized location on the stack:
    ///
    /// ```
    /// use musli_zerocopy::buf::MaybeUninit;
    ///
    /// let mut v: MaybeUninit<u32> = MaybeUninit::uninit();
    /// assert_eq!(v.write(&10u32.to_le()), &[10, 0, 0, 0]);
    /// ```
    ///
    /// Writing to a pre-allocation location in an [`AlignedBuf`].
    ///
    /// [`AlignedBuf`]: crate::buf::AlignedBuf
    ///
    /// ```
    /// use musli_zerocopy::{AlignedBuf, ZeroCopy};
    /// use musli_zerocopy::buf::MaybeUninit;
    /// use musli_zerocopy::pointer::{Ref, Unsized};
    ///
    /// #[derive(ZeroCopy)]
    /// #[repr(C)]
    /// struct Custom {
    ///     string: Unsized<str>,
    /// }
    ///
    /// let mut buf = AlignedBuf::new();
    ///
    /// let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();
    ///
    /// let string = buf.store_unsized("Hello World!");
    ///
    /// buf.load_uninit_mut(reference).write(&Custom {
    ///     string,
    /// });
    ///
    /// let buf = buf.as_aligned();
    /// let reference = reference.assume_init();
    ///
    /// assert_eq!(reference.offset(), 0);
    ///
    /// let custom = buf.load(reference)?;
    /// assert_eq!(buf.load(custom.string)?, "Hello World!");
    /// # Ok::<_, musli_zerocopy::Error>(())
    /// ```
    #[inline]
    pub fn write(&mut self, value: &T) -> &mut [u8]
    where
        T: ZeroCopy,
    {
        unsafe {
            let ptr = self as *mut Self as *mut u8;
            value.store_to(&mut RawBufMut::new(ptr));
            slice::from_raw_parts_mut(ptr, size_of::<T>())
        }
    }
}

impl<T> fmt::Debug for MaybeUninit<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaybeUninit").finish_non_exhaustive()
    }
}
