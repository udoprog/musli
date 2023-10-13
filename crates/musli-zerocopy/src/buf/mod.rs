//! Types associated within reading and binding to a [`Buf`].
//!
//! Buffers are slices of bytes without any inherent alignment. They allow use
//! to safely convert offset-like types to references for types which implements
//! [`ZeroCopy`].

#[cfg(test)]
mod tests;

pub use self::buf::Buf;
mod buf;

pub use self::bind::Bindable;
mod bind;

pub use self::load::{Load, LoadMut};
mod load;

pub use self::visit::Visit;
pub(crate) mod visit;

pub use self::validator::Validator;
mod validator;

pub use self::struct_padder::StructPadder;
mod struct_padder;

pub use self::buf_mut::BufMut;
mod buf_mut;

pub use self::cursor::Cursor;
mod cursor;

pub(crate) use self::raw_buf_mut::RawBufMut;
mod raw_buf_mut;

#[cfg(feature = "alloc")]
pub use self::aligned_buf::AlignedBuf;
#[cfg(feature = "alloc")]
mod aligned_buf;

pub use self::maybe_uninit::MaybeUninit;
mod maybe_uninit;

use core::mem::size_of;

use crate::error::Error;
use crate::traits::ZeroCopy;

/// The type used to calculate default alignment for [`AlignedBuf`].
pub type DefaultAlignment = usize;

/// Return the max capacity of this vector. This depends on the requested
/// alignment.
///
/// This follows how it's defined by `max_size_for_align` in [`Layout`].
#[inline]
pub fn max_capacity_for_align(align: usize) -> usize {
    isize::MAX as usize - (align - 1)
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
/// use musli_zerocopy::buf::aligned_buf;
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
/// let buf = aligned_buf(&bytes, 16);
///
/// let buf = match &buf {
///     Ok(buf) => *buf,
///     Err(buf) => buf.as_ref(),
/// };
///
/// let s = buf.load(Ref::<Person>::zero())?;
/// # Ok::<_, anyhow::Error>(())
/// ```
pub fn aligned_buf(bytes: &[u8], align: usize) -> Result<&Buf, AlignedBuf> {
    assert!(align.is_power_of_two(), "Alignment must be power of two");

    let buf = Buf::new(bytes);

    if buf.is_aligned_to(align) {
        Ok(buf)
    } else {
        let mut buf = unsafe { AlignedBuf::with_capacity_and_custom_alignment(bytes.len(), align) };

        unsafe {
            buf.store_bytes(bytes);
        }

        Err(buf)
    }
}

#[inline]
pub(crate) fn is_aligned_to(ptr: *const u8, align: usize) -> bool {
    assert!(align.is_power_of_two(), "alignment is not a power-of-two");
    (ptr as usize) & (align - 1) == 0
}

#[inline]
pub(crate) fn validate_array<T>(mut cursor: Cursor<'_>, len: usize) -> Result<(), Error>
where
    T: ZeroCopy,
{
    if !T::ANY_BITS && size_of::<T>() > 0 {
        for _ in 0..len / size_of::<T>() {
            // SAFETY: The passed in buffer is required to be aligned per the
            // requirements of this trait, so any size_of::<T>() chunks are aligned
            // too.
            unsafe {
                T::validate(cursor)?;
                cursor.advance::<T>();
            }
        }
    }

    Ok(())
}

/// Calculate padding with the assumption that alignment is a power of two.
pub(crate) fn padding_to(len: usize, align: usize) -> usize {
    let mask = align - 1;
    (align - (len & mask)) & mask
}
