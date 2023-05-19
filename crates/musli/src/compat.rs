//! Wrapper types which ensures that a given field is encoded or decoded as a
//! certain kind of value.

#[cfg(feature = "alloc")]
mod alloc;
mod packed;

pub use self::packed::Packed;

use crate::de::{Decode, Decoder, SequenceDecoder};
use crate::en::{Encode, Encoder, SequenceEncoder};
use crate::mode::Mode;
use crate::Context;

/// Ensures that the given value `T` is encoded as a sequence.
///
/// In contrast to the typical values that are encoded as sequences such as
/// [`Vec`], this can take a sequence by reference.
///
/// We must use a wrapper like this, because we can't provide an implementation
/// for `&[T]` since it would conflict with `&[u8]` which is specialized to
/// encode and decode as a byte array.
///
/// [`Vec`]: std::vec::Vec
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Sequence<T>(pub T);

impl<T> Sequence<T> {
    /// Construct a new sequence wrapper.
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<M, T> Encode<M> for Sequence<&'_ [T]>
where
    M: Mode,
    T: Encode<M>,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        let mut seq = encoder.encode_sequence(self.0.len())?;

        for value in self.0 {
            let encoder = seq.next()?;
            T::encode(value, encoder)?;
        }

        seq.end()
    }
}

impl<M> Encode<M> for Sequence<()>
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_sequence(0)?.end()
    }
}

impl<'de, M> Decode<'de, M> for Sequence<()>
where
    M: Mode,
{
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<D::Error>,
        D: Decoder<'de>,
    {
        let seq = decoder.decode_sequence(cx)?;
        seq.end(cx)?;
        Ok(Self(()))
    }
}

/// Ensures that the given value `T` is encoded as bytes.
///
/// This is useful for values which have a generic implementation to be encoded
/// as a sequence, such as [`Vec`] and [`VecDeque`].
///
/// We must use a wrapper like this, because we can't provide an implementation
/// for `Vec<T>` since it would conflict with `Vec<u8>` which is generalized to
/// encode as a sequence.
///
/// [`Vec`]: std::vec::Vec
/// [`VecDeque`]: std::collections::VecDeque
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Bytes<T>(pub T);

impl<const N: usize, M> Encode<M> for Bytes<[u8; N]>
where
    M: Mode,
{
    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_array(self.0)
    }
}

impl<'de, M, const N: usize> Decode<'de, M> for Bytes<[u8; N]>
where
    M: Mode,
{
    #[inline]
    fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    where
        C: Context<D::Error>,
        D: Decoder<'de>,
    {
        decoder.decode_array(cx).map(Self)
    }
}
