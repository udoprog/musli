//! Wrapper types for tweaking how something is encoded.
//!
//! Note that most types in this module have an attribute equivalent:
//! * [`Bytes`] corresponds to using `#[musli(bytes)]` on a field.
//! * [`Packed`] corresponds to using `#[musli(packed)]` on a field.

use crate::de::{Decode, DecodeBytes, DecodePacked, Decoder};
use crate::en::{Encode, EncodeBytes, EncodePacked, Encoder};
use crate::hint::SequenceHint;
use crate::mode::{Binary, Text};

/// Ensures that the given value `T` is encoded as a sequence.
///
/// This exists as a simple shim for certain types, to ensure they're encoded as
/// a sequence, such as `Sequence<()>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Sequence<T>(pub T);

impl<T> Sequence<T> {
    /// Construct a new sequence wrapper.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<M> Encode<M> for Sequence<()> {
    const ENCODE_PACKED: bool = true;

    type Encode = Self;

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder<Mode = M>,
    {
        static HINT: SequenceHint = SequenceHint::with_size(0);

        encoder.encode_sequence_fn(&HINT, |_| Ok(()))
    }

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }
}

impl<'de, M> Decode<'de, M> for Sequence<()> {
    const DECODE_PACKED: bool = true;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_sequence(|_| Ok(Self(())))
    }
}

/// Treat `T` as if its bytes.
///
/// This corresponds to the "Bytes" type in the [data model of Müsli] and is the
/// equivalent of using [`#[musli(bytes)]`][bytes] on a field.
///
/// This is only implemented for type where the default behavior is not to pack
/// the value already, this applies to types which implements [`EncodeBytes`]
/// and [`DecodeBytes`].
///
/// [`Vec`]: alloc::vec::Vec
/// [`VecDeque`]: alloc::collections::VecDeque
/// [bytes]: crate::_help::derives
/// [data model of Müsli]: crate::_help::data_model
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
/// impl<'de, M> Decode<'de, M> for Struct
/// where
///     Bytes<Vec<u8>>: Decode<'de, M>
/// {
///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de, Mode = M>,
///     {
///         let Bytes(field) = Decode::decode(decoder)?;
///
///         Ok(Struct {
///             field,
///         })
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[musli(crate, transparent)]
#[musli(mode = Binary, bound = {T: EncodeBytes<Binary>}, decode_bound = {T: DecodeBytes<'de, Binary>})]
#[musli(mode = Text, bound = {T: EncodeBytes<Text>}, decode_bound = {T: DecodeBytes<'de, Text>})]
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
/// This corresponds to the "Bytes" type in the [data model of Müsli]. It
/// encodes any [`Encode`] / [`Decode`] type "on after another" and is the
/// equivalent of using [`#[musli(packed)]`][packed] on a field.
///
/// This is only implemented for type where the default behavior is not to pack
/// the value already, this applies to types which implements [`EncodePacked`]
/// and [`DecodePacked`].
///
/// [packed]: crate::_help::derives
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
/// impl<'de, M> Decode<'de, M> for Struct
/// where
///     Packed<(u8, u32)>: Decode<'de, M>
/// {
///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
///     where
///         D: Decoder<'de, Mode = M>,
///     {
///         let Packed((field, field2)) = Decode::decode(decoder)?;
///
///         Ok(Struct {
///             field,
///             field2,
///         })
///     }
/// }
/// ```
#[derive(Encode, Decode)]
#[musli(crate, transparent)]
#[musli(mode = Binary, bound = {T: EncodePacked<Binary>}, decode_bound = {T: DecodePacked<'de, Binary>})]
#[musli(mode = Text, bound = {T: EncodePacked<Text>}, decode_bound = {T: DecodePacked<'de, Text>})]
#[repr(transparent)]
pub struct Packed<T>(#[musli(packed)] pub T);
