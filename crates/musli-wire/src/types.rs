//! Type flags available for `musli-wire`.

use musli::{Decode, Decoder};

/// The structure of a type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeTag {
    /// unknown type tag.
    Unknown = 0b0,
    /// A single encoded byte. The contents of which is packed in 7 least
    /// significant bits. If all LSBs are set to 1 (i.e. `0b1111_1111`), the next
    /// byte is used as the byte of the tag. All other types avoids having the MSB
    /// set.
    Fixed8 = 0b1000_0000,
    /// Read the entire next byte.
    Fixed8Next = 0b1111_1111,
    /// An absent optional value.
    OptionNone = 0b0111_1110,
    /// A present optional value.
    OptionSome = 0b0111_1111,
    /// Fixed-length 2 bytes.
    Fixed16 = 0b0001_0010,
    /// Fixed-length 4 bytes.
    Fixed32 = 0b0001_0100,
    /// Fixed-length 8 bytes.
    Fixed64 = 0b0001_0110,
    /// Fixed-length 16 bytes.
    Fixed128 = 0b0001_1000,
    /// The next integer is using continuation integer encoding.
    Continuation = 0b0001_1010,
    /// A length-prefixed byte sequence.
    Prefixed = 0b0010_0000,
    /// A length-prefixed sequence of typed values.
    Sequence = 0b0010_0010,
    /// A pair of typed values are being encoded.
    Pair = 0b0100_0000,
    /// A length-prefixed sequence of typed pairs of values.
    PairSequence = 0b0010_0100,
}

impl TypeTag {
    pub(crate) const FIXED8_BYTE: u8 = TypeTag::Fixed8 as u8;
    pub(crate) const FIXED16_BYTE: u8 = TypeTag::Fixed16 as u8;
    pub(crate) const FIXED32_BYTE: u8 = TypeTag::Fixed32 as u8;
    pub(crate) const FIXED64_BYTE: u8 = TypeTag::Fixed64 as u8;
    pub(crate) const FIXED128_BYTE: u8 = TypeTag::Fixed128 as u8;
    pub(crate) const CONTINUATION_BYTE: u8 = TypeTag::Continuation as u8;
    pub(crate) const PREFIXED_BYTE: u8 = TypeTag::Prefixed as u8;
    pub(crate) const SEQUENCE_BYTE: u8 = TypeTag::Sequence as u8;
    pub(crate) const PAIR_BYTE: u8 = TypeTag::Pair as u8;
    pub(crate) const PAIR_SEQUENCE_BYTE: u8 = TypeTag::PairSequence as u8;
    pub(crate) const OPTION_SOME_BYTE: u8 = TypeTag::OptionSome as u8;
    pub(crate) const OPTION_NONE_BYTE: u8 = TypeTag::OptionNone as u8;
}

impl<'de> Decode<'de> for TypeTag {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(match decoder.decode_u8()? {
            Self::FIXED8_BYTE => Self::Fixed8,
            Self::FIXED16_BYTE => Self::Fixed16,
            Self::FIXED32_BYTE => Self::Fixed32,
            Self::FIXED64_BYTE => Self::Fixed64,
            Self::FIXED128_BYTE => Self::Fixed128,
            Self::CONTINUATION_BYTE => Self::Continuation,
            Self::PREFIXED_BYTE => Self::Prefixed,
            Self::SEQUENCE_BYTE => Self::Sequence,
            Self::PAIR_BYTE => Self::Pair,
            Self::PAIR_SEQUENCE_BYTE => Self::PairSequence,
            Self::OPTION_SOME_BYTE => Self::OptionSome,
            Self::OPTION_NONE_BYTE => Self::OptionNone,
            _ => Self::Unknown,
        })
    }
}
