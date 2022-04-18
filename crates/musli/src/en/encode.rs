use crate::en::Encoder;
pub use musli_macros::Encode;

/// Trait governing how types are encoded.
pub trait Encode {
    /// Encode the given output.
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder;
}

impl<T> Encode for &T
where
    T: ?Sized + Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        T::encode(*self, encoder)
    }
}

impl<T> Encode for &mut T
where
    T: ?Sized + Encode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        T::encode(*self, encoder)
    }
}
