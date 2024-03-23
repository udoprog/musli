//! Wrapper types which ensures that a given field is encoded or decoded as a
//! certain kind of value.

use crate::de::{Decode, DecodeBytes, Decoder, SequenceDecoder};
use crate::en::{Encode, EncodeBytes, Encoder, SequenceEncoder};

/// Ensures that the given value `T` is encoded as a sequence.
///
/// This exists as a simple shim for certain types, to ensure they're encoded as
/// a sequence, such as `Sequence<()>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Sequence<T>(pub T);

impl<T> Sequence<T> {
    /// Construct a new sequence wrapper.
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<M> Encode<M> for Sequence<()> {
    #[inline]
    fn encode<E>(&self, _: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        encoder.encode_sequence(0)?.end()
    }
}

impl<'de, M> Decode<'de, M> for Sequence<()> {
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let seq = decoder.decode_sequence()?;
        seq.end()?;
        Ok(Self(()))
    }
}

/// Ensures that the given value `T` is encoded as bytes.
///
/// This is useful for values which have a generic implementation to be encoded
/// as a sequence, such as [`Vec`] and [`VecDeque`].
///
/// [`Vec`]: alloc::vec::Vec
/// [`VecDeque`]: alloc::collections::VecDeque
///
/// # Examples
///
/// ```
/// use musli::{Decode, Decoder};
/// use musli::compat::Bytes;
///
/// struct Struct {
///     field: Vec<u8>,
/// }
///
/// impl<'de, M> Decode<'de, M> for Struct {
///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de, Mode = M>,
///     {
///         let Bytes(field) = Decode::decode(cx, decoder)?;
///
///         Ok(Struct {
///             field,
///         })
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[musli(crate, bound = {T: EncodeBytes<M>}, decode_bound = {T: DecodeBytes<'de, M>})]
#[repr(transparent)]
pub struct Bytes<T>(#[musli(bytes)] pub T);

impl<T> AsRef<[u8]> for Bytes<T>
where
    T: AsRef<[u8]>,
{
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T> AsMut<[u8]> for Bytes<T>
where
    T: AsMut<[u8]>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

/// Treat `T` as if its packed.
///
/// This is for example implemented for tuples.
///
/// # Examples
///
/// ```
/// use musli::{Decode, Decoder};
/// use musli::compat::Packed;
///
/// struct Struct {
///     field: u8,
///     field2: u32,
/// }
///
/// impl<'de, M> Decode<'de, M> for Struct {
///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de, Mode = M>,
///     {
///         let Packed((field, field2)) = Decode::decode(cx, decoder)?;
///
///         Ok(Struct {
///             field,
///             field2,
///         })
///     }
/// }
/// ```
pub struct Packed<T>(pub T);
