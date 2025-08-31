use core::alloc::{Layout, LayoutError};
use core::fmt;
use core::mem::{ManuallyDrop, size_of};
use core::ptr::NonNull;
use core::slice;

use crate::ByteOrder;
use crate::buf;
use crate::error::CoerceError;
use crate::pointer::{Pointee, Size};
use crate::traits::ZeroCopy;

/// A value which might or might not have been initialized.
///
/// This differs from the standard library
/// [`MaybeUninit`][core::mem::MaybeUninit] in that its methods does not inherit
/// the alignment of the inner value so it can correctly refer to elements of
/// `T` in unaligned memory. Which [`OwnedBuf`] might refer to.
///
/// # Examples
///
/// Writing to a pre-allocation location in an [`OwnedBuf`].
///
/// [`OwnedBuf`]: crate::buf::OwnedBuf
///
/// ```
/// use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};
/// use musli_zerocopy::mem::MaybeUninit;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Custom { string: Ref<str> }
///
/// let mut buf = OwnedBuf::new();
///
/// let reference: Ref<MaybeUninit<Custom>> = buf.store_uninit::<Custom>();
///
/// let string = buf.store_unsized("Hello World!");
///
/// buf.load_uninit_mut(reference).write(&Custom { string });
///
/// let reference = reference.assume_init();
///
/// assert_eq!(reference.offset(), 0);
///
/// let custom = buf.load(reference)?;
/// assert_eq!(buf.load(custom.string)?, "Hello World!");
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[repr(C, packed)]
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
    /// use musli_zerocopy::mem::MaybeUninit;
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
    /// use musli_zerocopy::mem::MaybeUninit;
    ///
    /// let mut v: MaybeUninit<u32> = MaybeUninit::uninit();
    /// assert_eq!(v.write(&10u32.to_le()), &[10, 0, 0, 0]);
    /// ```
    #[inline]
    pub fn write(&mut self, value: &T) -> &mut [u8]
    where
        T: ZeroCopy,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(self as *mut Self as *mut u8);
            buf::store_unaligned(ptr, value);
            slice::from_raw_parts_mut(ptr.as_ptr(), size_of::<T>())
        }
    }
}

impl<T> fmt::Debug for MaybeUninit<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaybeUninit").finish_non_exhaustive()
    }
}

impl<T> Pointee for MaybeUninit<T>
where
    T: Pointee,
{
    type Metadata = T::Metadata;
    type Stored<O>
        = T::Stored<O>
    where
        O: Size;

    #[inline]
    fn try_from_metadata<O>(metadata: Self::Metadata) -> Result<Self::Stored<O>, CoerceError>
    where
        O: Size,
    {
        T::try_from_metadata(metadata)
    }

    #[inline]
    fn size<E, O>(metadata: Self::Stored<O>) -> Option<usize>
    where
        E: ByteOrder,
        O: Size,
    {
        T::size::<E, O>(metadata)
    }

    #[inline]
    fn align<E, O>(metadata: Self::Stored<O>) -> usize
    where
        E: ByteOrder,
        O: Size,
    {
        T::align::<E, O>(metadata)
    }

    #[inline]
    fn pointee_layout<E, O>(metadata: Self::Stored<O>) -> Result<Layout, LayoutError>
    where
        E: ByteOrder,
        O: Size,
    {
        T::pointee_layout::<E, O>(metadata)
    }
}
