use crate::en::Encoder;
use crate::mode::{DefaultMode, Mode};
pub use musli_macros::Encode;

/// Trait governing how types are encoded.
pub trait Encode<M = DefaultMode>
where
    M: Mode,
{
    /// Encode the given output.
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder;
}

impl<T, M> Encode<M> for &T
where
    T: ?Sized + Encode<M>,
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        T::encode(*self, encoder)
    }
}

impl<T, M> Encode<M> for &mut T
where
    T: ?Sized + Encode<M>,
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        T::encode(*self, encoder)
    }
}
