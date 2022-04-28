use crate::en::Encoder;
pub use musli_macros::Encode;

/// Trait governing how types are encoded.
pub trait Encode<Mode> {
    /// Encode the given output.
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder;
}

impl<T, Mode> Encode<Mode> for &T
where
    T: ?Sized + Encode<Mode>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        T::encode(*self, encoder)
    }
}

impl<T, Mode> Encode<Mode> for &mut T
where
    T: ?Sized + Encode<Mode>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        T::encode(*self, encoder)
    }
}
