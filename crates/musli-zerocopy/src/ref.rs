use core::marker::PhantomData;

use crate::size::{DefaultSize, Size};
use crate::ZeroCopy;

/// A sized reference.
///
/// This is used to type a pointer with a [`ZeroCopy`] parameter so that it can
/// be used in combination with [`Buf`] to load the value from a buffer.
///
/// Note that the constructor is safe, because alignment and validation checks
/// happens whenever a value is loaded from a bare buffer.
///
/// [`Buf`]: crate::buf::Buf
///
/// # Examples
///
/// ```
/// use core::mem::align_of;
/// use musli_zerocopy::{AlignedBuf, Ref};
///
/// let mut buf = AlignedBuf::with_alignment(align_of::<u32>());
/// buf.extend_from_slice(&[1, 2, 3, 4]);
///
/// let buf = buf.as_ref()?;
///
/// let number = Ref::<u32>::new(0);
/// assert_eq!(*buf.load(number)?, u32::from_ne_bytes([1, 2, 3, 4]));
/// # Ok::<_, musli_zerocopy::Error>(())
/// ```
#[derive(Debug, ZeroCopy)]
#[repr(C)]
#[zero_copy(crate)]
pub struct Ref<T: ZeroCopy, O: Size = DefaultSize> {
    offset: O,
    #[zero_copy(ignore)]
    _marker: PhantomData<T>,
}

impl<T, O: Size> Ref<T, O>
where
    T: ZeroCopy,
{
    /// Construct a typed reference to the first position in a buffer.
    pub const fn zero() -> Self {
        Self {
            offset: O::ZERO,
            _marker: PhantomData,
        }
    }

    /// Construct a reference wrapping the given type at the specified address.
    pub fn new(offset: usize) -> Self {
        let Some(offset) = O::from_usize(offset) else {
            panic!("Ref offset {offset} not in the legal range of 0-{}", O::MAX);
        };

        Self {
            offset,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn offset(&self) -> usize {
        self.offset.as_usize()
    }
}

impl<T: ZeroCopy, O: Size> Clone for Ref<T, O> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ZeroCopy, O: Size> Copy for Ref<T, O> {}
