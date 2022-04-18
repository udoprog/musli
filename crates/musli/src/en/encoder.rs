use crate::en::Encode;
use crate::error::Error;

/// A pack that can construct encoders.
pub trait PackEncoder {
    /// The error type of the pack.
    type Error: Error;

    /// The encoder to use for the pack.
    type Encoder<'this>: Encoder<Error = Self::Error>
    where
        Self: 'this;

    /// Construct a decoder for the next element to pack.
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error>;

    /// Push a value into the pack.
    #[inline]
    fn push<T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        T: Encode,
    {
        let encoder = self.next()?;
        Encode::encode(&value, encoder)
    }

    /// Finish packing.
    fn finish(self) -> Result<(), Self::Error>;
}

/// Trait governing how to encode a sequence.
pub trait SequenceEncoder {
    /// The error raised by a sequence encoder.
    type Error: Error;

    /// The encoder returned when advancing the sequence encoder.
    type Next<'this>: Encoder<Error = Self::Error>
    where
        Self: 'this;

    /// Prepare encoding of the next element.
    fn encode_next(&mut self) -> Result<Self::Next<'_>, Self::Error>;

    /// Finish encoding the sequence.
    fn finish(self) -> Result<(), Self::Error>;
}

/// Trait governing how to encode a sequence of pairs.
pub trait PairEncoder {
    /// The error raised by a map encoder.
    type Error: Error;

    /// The encoder returned when advancing the map encoder to encode the key.
    type First<'this>: Encoder<Error = Self::Error>
    where
        Self: 'this;

    /// The encoder returned when advancing the map encoder to encode the value.
    type Second<'this>: Encoder<Error = Self::Error>
    where
        Self: 'this;

    /// Encode the first element in a pair.
    fn encode_first(&mut self) -> Result<Self::First<'_>, Self::Error>;

    /// Encode the second element in the pair.
    fn encode_second(&mut self) -> Result<Self::Second<'_>, Self::Error>;

    /// Finish encoding the sequence of pairs.
    fn finish(self) -> Result<(), Self::Error>;
}

/// Encoding a variant tag.
pub trait VariantEncoder {
    /// The error raised by an encoder.
    type Error: Error;

    /// Tag encoder.
    type VariantTag<'this>: Encoder<Error = Self::Error>
    where
        Self: 'this;

    /// Encoder for the value of the variant.
    type VariantValue: Encoder<Error = Self::Error>;

    /// Start encoding a tag.
    fn encode_variant_tag(&mut self) -> Result<Self::VariantTag<'_>, Self::Error>;

    /// Setup encoder for the value of the variant.
    fn encode_variant_value(self) -> Result<Self::VariantValue, Self::Error>;
}

/// Trait governing how the encoder works.
pub trait Encoder: Sized {
    /// The error raised by an encoder.
    type Error: Error;

    /// A simple pack that packs a sequence of elements.
    type Pack: PackEncoder<Error = Self::Error>;

    /// Encoder returned when encoding an optional value which is present.
    type Some: Encoder<Error = Self::Error>;

    /// The type of a sequence encoder.
    type Sequence: SequenceEncoder<Error = Self::Error>;

    /// The type of a map encoder.
    type Map: PairEncoder<Error = Self::Error>;

    /// Encoder that can encode a struct.
    type Struct: PairEncoder<Error = Self::Error>;

    /// Encoder that can encode a tuple struct.
    type Tuple: PairEncoder<Error = Self::Error>;

    /// Encoder for a unit variant.
    type Variant: VariantEncoder<Error = Self::Error>;

    /// Encode a unit or something that is empty.
    fn encode_unit(self) -> Result<(), Self::Error>;

    /// Construct a pack that can encode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to encode all of the
    /// elements in the packed sequence as compact as possible and that
    /// subsequent unpackers will know the exact length of the element being
    /// unpacked.
    fn encode_pack(self) -> Result<Self::Pack, Self::Error>;

    /// Encode fixed-length array.
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<(), Self::Error>;

    /// Encode a sequence of bytes who's length is included in the payload.
    fn encode_bytes(self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Encode the given slice of bytes in sequence, with one following another.
    fn encode_bytes_vectored(self, vectors: &[&[u8]]) -> Result<(), Self::Error>;

    /// Encode a string who's length is included in the payload.
    fn encode_str(self, string: &str) -> Result<(), Self::Error>;

    /// Encode a usize value.
    fn encode_usize(self, value: usize) -> Result<(), Self::Error>;

    /// Encode a isize value.
    fn encode_isize(self, value: isize) -> Result<(), Self::Error>;

    /// Encode a boolean value.
    fn encode_bool(self, value: bool) -> Result<(), Self::Error>;

    /// Encode a character.
    fn encode_char(self, value: char) -> Result<(), Self::Error>;

    /// Encode a 8-bit unsigned integer.
    fn encode_u8(self, value: u8) -> Result<(), Self::Error>;

    /// Encode a 16-bit unsigned integer.
    fn encode_u16(self, value: u16) -> Result<(), Self::Error>;

    /// Encode a 32-bit unsigned integer.
    fn encode_u32(self, value: u32) -> Result<(), Self::Error>;

    /// Encode a 64-bit unsigned integer.
    fn encode_u64(self, value: u64) -> Result<(), Self::Error>;

    /// Encode a 128-bit unsigned integer.
    fn encode_u128(self, value: u128) -> Result<(), Self::Error>;

    /// Encode a 8-bit signed integer.
    fn encode_i8(self, value: i8) -> Result<(), Self::Error>;

    /// Encode a 16-bit signed integer.
    fn encode_i16(self, value: i16) -> Result<(), Self::Error>;

    /// Encode a 32-bit signed integer.
    fn encode_i32(self, value: i32) -> Result<(), Self::Error>;

    /// Encode a 64-bit signed integer.
    ///
    /// Defaults to using [Encoder::encode_u64] with the signed value casted.
    fn encode_i64(self, value: i64) -> Result<(), Self::Error>;

    /// Encode a 128-bit signed integer.
    ///
    /// Defaults to using [Encoder::encode_u128] with the signed value casted.
    fn encode_i128(self, value: i128) -> Result<(), Self::Error>;

    /// Encode a 32-bit floating point value.
    ///
    /// Default to taking the 32-bit in-memory IEEE 754 encoding and writing it byte-by-byte.
    fn encode_f32(self, value: f32) -> Result<(), Self::Error>;

    /// Encode a 64-bit floating point value.
    ///
    /// Default to taking the 64-bit in-memory IEEE 754 encoding and writing it byte-by-byte.
    fn encode_f64(self, value: f64) -> Result<(), Self::Error>;

    /// Encode an optional value that is present.
    fn encode_some(self) -> Result<Self::Some, Self::Error>;

    /// Encode an optional value that is absent.
    fn encode_none(self) -> Result<(), Self::Error>;

    /// Encode a sequence with a known length.
    fn encode_sequence(self, len: usize) -> Result<Self::Sequence, Self::Error>;

    /// Encode a map with a known length.
    fn encode_map(self, len: usize) -> Result<Self::Map, Self::Error>;

    /// Encode a struct.
    fn encode_struct(self, fields: usize) -> Result<Self::Struct, Self::Error>;

    /// Encode a tuple struct.
    fn encode_tuple(self, fields: usize) -> Result<Self::Tuple, Self::Error>;

    /// Encode a unit struct.
    fn encode_unit_struct(self) -> Result<(), Self::Error>;

    /// Encode a struct variant.
    fn encode_variant(self) -> Result<Self::Variant, Self::Error>;
}
