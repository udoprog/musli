use crate::en::Encoder;
use crate::mode::{DefaultMode, Mode};
use crate::Context;

pub use musli_macros::Encode;

/// Trait governing how types are encoded.
pub trait Encode<M = DefaultMode>
where
    M: Mode,
{
    /// Encode the given output.
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder;
}

impl<T, M> Encode<M> for &T
where
    T: ?Sized + Encode<M>,
    M: Mode,
{
    #[inline]
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder,
    {
        T::encode(*self, cx, encoder)
    }
}

impl<T, M> Encode<M> for &mut T
where
    T: ?Sized + Encode<M>,
    M: Mode,
{
    #[inline]
    fn encode<'buf, C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    where
        C: Context<'buf, Input = E::Error>,
        E: Encoder,
    {
        T::encode(*self, cx, encoder)
    }
}
