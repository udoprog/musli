use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::{any, fmt};

use crate::endian::{Big, ByteOrder, Little, Native};
use crate::ZeroCopy;

/// Wrapper capable of enforcing a custom [`ByteOrder`].
///
/// This can be used to store values in a zero-copy container in a portable
/// manner, which is especially important to transfer types such as `char` which
/// have a limited supported bit-pattern.
#[derive(ZeroCopy)]
#[zero_copy(crate, swap_bytes, bounds = {T: ZeroCopy})]
#[repr(transparent)]
pub struct Endian<T, E: ByteOrder> {
    value: T,
    #[zero_copy(ignore)]
    _marker: PhantomData<E>,
}

impl<T: ZeroCopy> Endian<T, Little> {
    /// Construct new value wrapper with [`Little`] [`ByteOrder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Endian;
    ///
    /// let value = Endian::le(42u32);
    /// assert_eq!(value.to_ne(), 42);
    /// assert_eq!(value.to_raw(), 42u32.to_le());
    /// ```
    #[inline]
    pub fn le(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: ZeroCopy> Endian<T, Big> {
    /// Construct new value wrapper with [`Big`] [`ByteOrder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Endian;
    ///
    /// let value = Endian::be(42u32);
    /// assert_eq!(value.to_ne(), 42);
    /// assert_eq!(value.to_raw(), 42u32.to_be());
    /// ```
    #[inline]
    pub fn be(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: ZeroCopy, E: ByteOrder> Endian<T, E> {
    /// Construct new value wrapper with the specified [`ByteOrder`].
    ///
    /// # Panics
    ///
    /// Panics if we try to use this with a `ZeroCopy` type that cannot be
    /// byte-ordered.
    ///
    /// ```should_panic
    /// use musli_zerocopy::{endian, Endian};
    ///
    /// let _: Endian<_, endian::Little> = Endian::new('a');
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::{endian, Endian, ZeroCopy};
    ///
    /// let mut a: Endian<_, endian::Big> = Endian::new('a' as u32);
    /// let mut b: Endian<_, endian::Little> = Endian::new('a' as u32);
    ///
    /// assert_eq!(a.to_ne(), 'a' as u32);
    /// assert_eq!(a.to_bytes(), &[0, 0, 0, 97]);
    ///
    /// assert_eq!(b.to_ne(), 'a' as u32);
    /// assert_eq!(b.to_bytes(), &[97, 0, 0, 0]);
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

    /// Get interior value in native [`ByteOrder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Endian;
    ///
    /// let value = Endian::le(42u32);
    /// assert_eq!(value.to_ne(), 42);
    /// assert_eq!(value.to_raw(), 42u32.to_le());
    /// ```
    #[inline]
    pub fn to_ne(self) -> T {
        T::swap_bytes::<E>(self.value)
    }

    /// Get the raw inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli_zerocopy::Endian;
    ///
    /// let value = Endian::le(42u32);
    /// assert_eq!(value.to_ne(), 42);
    /// assert_eq!(value.to_raw(), 42u32.to_le());
    /// ```
    #[inline]
    pub fn to_raw(self) -> T {
        self.value
    }
}

impl<T: ZeroCopy, E: ByteOrder> fmt::Debug for Endian<T, E>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Endian<{}>({:?})", any::type_name::<E>(), self.value)
    }
}

impl<T: ZeroCopy, E: ByteOrder> Clone for Endian<T, E>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: self._marker,
        }
    }
}

impl<T: ZeroCopy, E: ByteOrder> Copy for Endian<T, E> where T: Copy {}

/// Any `Endian<T>` implements [`Deref<Target = T>`] for natively wrapped types.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Endian;
///
/// let value = Endian::new(42u32);
/// assert_eq!(*value, 42u32);
/// ```
impl<T> Deref for Endian<T, Native> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Any `Endian<T>` implements [`DerefMut<Target = T>`] for natively wrapped types.
///
/// # Examples
///
/// ```
/// use musli_zerocopy::Endian;
///
/// let mut value = Endian::new(42u32);
/// assert_eq!(*value, 42u32);
/// *value += 1;
/// assert_eq!(*value, 43u32);
/// ```
impl<T> DerefMut for Endian<T, Native> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
