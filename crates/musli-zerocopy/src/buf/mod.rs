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
//! The following is a more compact [`Ref<[u8]>`] which only occupies 4 bytes,
//! but restricts the offset and length.
//!
//! [`Ref<u8>`]: crate::pointer::Ref
//!
//! ```
//! use musli_zerocopy::{Error, Ref, ZeroCopy};
//! use musli_zerocopy::buf::{OwnedBuf, Buf, Load, LoadMut, Visit};
//!
//! #[derive(Clone, Copy, ZeroCopy)]
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
//!     fn to_slice(&self) -> Ref<[u8]> {
//!         let [a, b, c] = self.offset;
//!         let offset = u32::from_le_bytes([a, b, c, 0]);
//!         Ref::with_metadata(offset as usize, self.len as usize)
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
//! let mut buf1 = OwnedBuf::new();
//! let slice = buf1.store_slice(&[1u8, 2, 3, 4]);
//! let slice_ref: Ref<Ref<[u8]>> = buf1.store(&slice);
//! assert_eq!(buf1.len(), 12);
//!
//! let slice = buf1.load(slice_ref)?;
//! assert_eq!(slice.len(), 4);
//! assert_eq!(buf1.load(*slice)?, &[1, 2, 3, 4]);
//!
//! let mut buf2 = OwnedBuf::new();
//! let slice = buf2.store_slice(&[1u8, 2, 3, 4]);
//! let slice = CompactSlice::new(slice.offset(), slice.len());
//! let slice_ref: Ref<CompactSlice> = buf2.store(&slice);
//! assert_eq!(buf2.len(), 8);
//!
//! let slice = buf2.load(slice_ref)?;
//! assert_eq!(slice.len(), 4);
//! assert_eq!(buf2.load(*slice)?, &[1, 2, 3, 4]);
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

pub use self::store_buf::StoreBuf;
mod store_buf;

#[cfg(feature = "alloc")]
pub use self::owned_buf::OwnedBuf;
#[cfg(feature = "alloc")]
mod owned_buf;

pub use self::slice_mut::SliceMut;
mod slice_mut;

use core::mem::{align_of, size_of};

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;

use crate::error::Error;
use crate::traits::ZeroCopy;

/// The type used to calculate default alignment for [`OwnedBuf`].
#[repr(transparent)]
pub struct DefaultAlignment(usize);

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
/// allocated [`OwnedBuf`].
///
/// # Examples
///
/// ```no_run
/// use std::fs::read;
///
/// use musli_zerocopy::{Ref, ZeroCopy};
/// use musli_zerocopy::buf;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Person {
///     name: Ref<str>,
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
    Buf::new(bytes).to_aligned::<T>()
}

/// Construct a buffer with a specific alignment which is either wrapped in a
/// [`Buf`] if it is already correctly aligned, or inside of an allocated
/// [`OwnedBuf`].
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
///
/// use musli_zerocopy::{Ref, ZeroCopy};
/// use musli_zerocopy::buf;
///
/// #[derive(ZeroCopy)]
/// #[repr(C)]
/// struct Person {
///     name: Ref<str>,
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
    Buf::new(bytes).to_aligned_with(align)
}

/// # Safety
///
/// Must be called with an alignment that is a power of two.
#[inline]
pub(crate) fn is_aligned_with(ptr: *const u8, align: usize) -> bool {
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

    if !T::ANY_BITS {
        for _ in 0..len {
            // SAFETY: The passed in buffer is required to be aligned per the
            // requirements of this trait, so any size_of::<T>() chunks are
            // aligned too.
            validator.validate_only::<T>()?;
        }
    }

    Ok(())
}

/// Calculate padding with the assumption that alignment is a power of two.
#[inline(always)]
pub(crate) fn padding_to(len: usize, align: usize) -> usize {
    let mask = align - 1;
    (align - (len & mask)) & mask
}

/// Store the raw bytes associated with `*const T` into the buffer and advance
/// its position by `size_of::<T>()`.
///
/// This does not require `T` to be aligned.
///
/// # Safety
///
/// The caller must ensure that any store call only includes data up-to the size
/// of `Self`.
///
/// Also see the [type level safety documentation][#safety]
#[inline]
pub(crate) unsafe fn store_unaligned<T>(ptr: *mut u8, value: *const T)
where
    T: ZeroCopy,
{
    ptr.copy_from_nonoverlapping(value.cast(), size_of::<T>());

    if T::PADDED {
        let mut padder = Padder::new(ptr);
        T::pad(&mut padder);
        padder.remaining();
    }
}
