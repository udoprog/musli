use core::marker::PhantomData;
use core::{any, fmt};

use crate::buf::{Padder, Validator};
use crate::endian::ByteOrder;
use crate::ZeroCopy;

/// A value capable of enforcing a custom [`ByteOrder`].
///
/// This can be used to store values in a zero-copy container in a portable
/// manner, which is especially important to transfer types such as `char` which
/// have a limited supported bit-pattern.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::{Ordered, ZeroCopy};
/// use musli_zerocopy::endian::{BigEndian, LittleEndian};
///
/// let mut a: Ordered<_, BigEndian> = Ordered::new('a' as u32);
/// let mut b: Ordered<_, LittleEndian> = Ordered::new('a' as u32);
///
/// assert_eq!(a.into_value(), 'a' as u32);
/// assert_eq!(a.to_bytes(), &[0, 0, 0, 97]);
///
/// assert_eq!(b.into_value(), 'a' as u32);
/// assert_eq!(b.to_bytes(), &[97, 0, 0, 0]);
/// ```
#[repr(transparent)]
pub struct Ordered<T, E: ByteOrder> {
    value: T,
    _marker: PhantomData<E>,
}

impl<T: ZeroCopy, E: ByteOrder> Ordered<T, E> {
    /// Construct new value wrapper with the specified [`ByteOrder`].
    ///
    /// # Panics
    ///
    /// Panics if we try to use this with a `ZeroCopy` type that cannot be
    /// byte-ordered.
    ///
    /// ```should_panic
    /// use musli_zerocopy::Ordered;
    /// use musli_zerocopy::endian::LittleEndian;
    ///
    /// let _: Ordered<_, LittleEndian> = Ordered::new('a');
    /// ```
    #[inline]
    pub fn new(value: T) -> Self {
        assert!(
            T::CAN_SWAP_BYTES,
            "Type `{}` cannot be byte-ordered since it would not inhabit valid types",
            any::type_name::<T>()
        );

        Self {
            value: T::swap_bytes::<E>(value),
            _marker: PhantomData,
        }
    }

    /// Get interior value with the desired native alignment.
    #[inline]
    pub fn into_value(self) -> T {
        T::swap_bytes::<E>(self.value)
    }
}

unsafe impl<T, E> ZeroCopy for Ordered<T, E>
where
    T: ZeroCopy,
    E: ByteOrder,
{
    const ANY_BITS: bool = T::ANY_BITS;
    const PADDED: bool = T::PADDED;
    const CAN_SWAP_BYTES: bool = true;

    #[inline]
    unsafe fn pad(padder: &mut Padder<'_, Self>) {
        T::pad(padder.transparent())
    }

    #[inline]
    unsafe fn validate(validator: &mut Validator<'_, Self>) -> Result<(), crate::Error> {
        T::validate(validator.transparent())
    }

    #[inline]
    fn swap_bytes<__E: ByteOrder>(self) -> Self {
        // NB: This is a no-op, since the byte-ordering is recorded at the type
        // level and doesn't change no matter what ordering is requested.
        self
    }
}

impl<T, E> fmt::Debug for Ordered<T, E>
where
    T: fmt::Debug + ZeroCopy,
    E: ByteOrder,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ordered")
            .field("value", &self.value)
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<T, E> Clone for Ordered<T, E>
where
    T: Clone + ZeroCopy,
    E: ByteOrder,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: self._marker,
        }
    }
}

impl<T, E> Copy for Ordered<T, E>
where
    T: Copy + ZeroCopy,
    E: ByteOrder,
{
}
