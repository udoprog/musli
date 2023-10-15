//! Types associated within reading and binding to a [`Buf`].
//!
//! Buffers are slices of bytes without any inherent alignment. They allow use
//! to safely convert offset-like types to references for types which implements
//! [`ZeroCopy`].
//!
//! # Extension traits
//!
//! This module provides a couple of extension traits which can be used to alter
//! how types interact with buffers.
//!
//! The following is a more compact [`Slice<u8>`] which only occupies 4 bytes,
//! but restricts the offset and length.
//!
//! [`Slice<u8>`]: crate::pointer::Slice
//!
//! ```
//! use musli_zerocopy::{Error, ZeroCopy};
//! use musli_zerocopy::buf::{AlignedBuf, Buf, Load, LoadMut, Visit};
//! use musli_zerocopy::pointer::{Slice, Ref};
//!
//! #[derive(ZeroCopy)]
//! #[repr(C, packed)]
//! struct CompactSlice {
//!     // 3 bytes of offset.
//!     offset: [u8; 3],
//!     // one byte of length.
//!     len: u8,
//! }
//!
//! impl CompactSlice {
//!     /// Construct a new compact slice, panic if offset and length
//!     /// overflow 3 and 1 byte integers respectively.
//!     pub fn new(offset: usize, len: usize) -> Self {
//!         assert!(offset <= 0xffffffusize && len <= 0xff);
//!         let [a, b, c, ..] = offset.to_le_bytes();
//!
//!         Self {
//!             offset: [a, b, c],
//!             len: len as u8,
//!         }
//!     }
//!
//!     /// Get the length of the compact slice.
//!     pub fn len(&self) -> usize {
//!         self.len as usize
//!     }
//!
//!     fn to_slice(&self) -> Slice<u8> {
//!         let [a, b, c] = self.offset;
//!         let offset = u32::from_le_bytes([a, b, c, 0]);
//!         Slice::new(offset as usize, self.len as usize)
//!     }
//! }
//!
//! impl Load for CompactSlice {
//!     type Target = [u8];
//!
//!     fn load<'buf>(&self, buf: &'buf Buf) -> Result<&'buf Self::Target, Error> {
//!         buf.load(self.to_slice())
//!     }
//! }
//!
//! impl LoadMut for CompactSlice {
//!     fn load_mut<'buf>(&self, buf: &'buf mut Buf) -> Result<&'buf mut Self::Target, Error> {
//!         buf.load_mut(self.to_slice())
//!     }
//! }
//!
//! let mut buf1 = AlignedBuf::new();
//! let slice = buf1.store_slice(&[1u8, 2, 3, 4]);
//! let slice_ref: Ref<Slice<u8>> = buf1.store(&slice);
//! assert_eq!(buf1.len(), 12);
//!
//! let buf1 = buf1.as_aligned();
//! let slice = buf1.load(slice_ref)?;
//! assert_eq!(slice.len(), 4);
//! assert_eq!(buf1.load(slice)?, &[1, 2, 3, 4]);
//!
//! let mut buf2 = AlignedBuf::new();
//! let slice = buf2.store_slice(&[1u8, 2, 3, 4]);
//! let slice = CompactSlice::new(slice.offset(), slice.len());
//! let slice_ref: Ref<CompactSlice> = buf2.store(&slice);
//! assert_eq!(buf2.len(), 8);
//!
//! let buf2 = buf2.as_aligned();
//! let slice = buf2.load(slice_ref)?;
//! assert_eq!(slice.len(), 4);
//! assert_eq!(buf2.load(slice)?, &[1, 2, 3, 4]);
//! # Ok::<_, musli_zerocopy::Error>(())
//! ```

#[cfg(test)]
mod tests;

pub use self::buf::Buf;
mod buf;

pub use self::bind::Bindable;
mod bind;

pub use self::load::{Load, LoadMut};
mod load;

pub use self::visit::Visit;
pub use musli_macros::Visit;
pub(crate) mod visit;

pub use self::validator::Validator;
mod validator;

pub use self::padder::Padder;
mod padder;

pub use self::buf_mut::BufMut;
mod buf_mut;

pub(crate) use self::raw_buf_mut::RawBufMut;
mod raw_buf_mut;

#[cfg(feature = "alloc")]
pub use self::aligned_buf::AlignedBuf;
#[cfg(feature = "alloc")]
mod aligned_buf;

use core::mem::{align_of, size_of};

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;

use crate::error::Error;
use crate::traits::ZeroCopy;

/// The type used to calculate default alignment for [`AlignedBuf`].
pub type DefaultAlignment = usize;

/// Return the max capacity of this vector. This depends on the requested
/// alignment.
///
/// This follows how it's defined by `max_size_for_align` in [`Layout`].
///
/// [`Layout`]: core::alloc::Layout
#[inline]
pub fn max_capacity_for_align(align: usize) -> usize {
    isize::MAX as usize - (align - 1)
}

/// Construct a buffer with an alignment matching that of `T` which is either
/// wrapped in a [`Buf`] if it is already correctly aligned, or inside of an
/// allocated [`AlignedBuf`].
///
/// # Examples
///
/// ```no_run
/// use std::fs::read;
/// use musli_zerocopy::ZeroCopy;
/// use musli_zerocopy::buf;
/// use musli_zerocopy::pointer::{Ref, Unsized};
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Person {
///     name: Unsized<str>,
///     age: u32,
/// }
///
/// let bytes = read("person.bin")?;
/// let buf = buf::aligned_buf::<u128>(&bytes);
///
/// let s = buf.load(Ref::<Person>::zero())?;
/// # Ok::<_, anyhow::Error>(())
/// ```
#[cfg(feature = "alloc")]
pub fn aligned_buf<T>(bytes: &[u8]) -> Cow<'_, Buf> {
    aligned_buf_with(bytes, align_of::<T>())
}

/// Construct a buffer with a specific alignment which is either wrapped in a
/// [`Buf`] if it is already correctly aligned, or inside of an allocated
/// [`AlignedBuf`].
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
/// use musli_zerocopy::ZeroCopy;
/// use musli_zerocopy::buf;
/// use musli_zerocopy::pointer::{Ref, Unsized};
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Person {
///     name: Unsized<str>,
///     age: u32,
/// }
///
/// let bytes = read("person.bin")?;
/// let buf = buf::aligned_buf_with(&bytes, 16);
///
/// let s = buf.load(Ref::<Person>::zero())?;
/// # Ok::<_, anyhow::Error>(())
/// ```
#[cfg(feature = "alloc")]
pub fn aligned_buf_with(bytes: &[u8], align: usize) -> Cow<'_, Buf> {
    assert!(align.is_power_of_two(), "Alignment must be power of two");

    let buf = Buf::new(bytes);

    if buf.is_aligned_to(align) {
        Cow::Borrowed(buf)
    } else {
        let mut buf = unsafe { AlignedBuf::with_capacity_and_custom_alignment(bytes.len(), align) };

        unsafe {
            buf.store_bytes(bytes);
        }

        Cow::Owned(buf)
    }
}

#[inline]
pub(crate) fn is_aligned_to(ptr: *const u8, align: usize) -> bool {
    assert!(align.is_power_of_two(), "alignment is not a power-of-two");
    (ptr as usize) & (align - 1) == 0
}

#[inline]
pub(crate) unsafe fn validate_array<S, T>(
    validator: &mut Validator<'_, S>,
    len: usize,
) -> Result<(), Error>
where
    S: ?Sized,
    T: ZeroCopy,
{
    validator.align_with(align_of::<T>());

    if !T::ANY_BITS && size_of::<T>() > 0 {
        for _ in 0..len / size_of::<T>() {
            // SAFETY: The passed in buffer is required to be aligned per the
            // requirements of this trait, so any size_of::<T>() chunks are
            // aligned too.
            validator.validate_only::<T>()?;
        }
    }

    Ok(())
}

/// Calculate padding with the assumption that alignment is a power of two.
pub(crate) fn padding_to(len: usize, align: usize) -> usize {
    let mask = align - 1;
    (align - (len & mask)) & mask
}
