use core::fmt;

use crate::en::Encode;
use crate::error::Error;
use crate::expecting::{self, Expecting, InvalidType};

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
    type Variant: PairEncoder<Error = Self::Error>;

    /// An expectation error. Every other implementation defers to this to
    /// report that something unexpected happened.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Encode a unit or something that is empty.
    #[inline]
    fn encode_unit(self) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unit,
            &ExpectingWrapper(self),
        )))
    }

    /// Construct a pack that can encode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to encode all of the
    /// elements in the packed sequence as compact as possible and that
    /// subsequent unpackers will know the exact length of the element being
    /// unpacked.
    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Pack,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode fixed-length array.
    #[inline]
    fn encode_array<const N: usize>(self, _: [u8; N]) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Array,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a sequence of bytes who's length is included in the payload.
    #[inline]
    fn encode_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Bytes,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode the given slice of bytes in sequence, with one following another.
    #[inline]
    fn encode_bytes_vectored(self, _: &[&[u8]]) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Bytes,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a string who's length is included in the payload.
    #[inline]
    fn encode_string(self, _: &str) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::String,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a boolean value.
    #[inline]
    fn encode_bool(self, _: bool) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Bool,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a character.
    #[inline]
    fn encode_char(self, _: char) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Char,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 8-bit unsigned integer.
    #[inline]
    fn encode_u8(self, _: u8) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned8,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 16-bit unsigned integer.
    #[inline]
    fn encode_u16(self, _: u16) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned16,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 32-bit unsigned integer.
    #[inline]
    fn encode_u32(self, _: u32) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned32,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 64-bit unsigned integer.
    #[inline]
    fn encode_u64(self, _: u64) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned64,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 128-bit unsigned integer.
    #[inline]
    fn encode_u128(self, _: u128) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned128,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 8-bit signed integer.
    #[inline]
    fn encode_i8(self, _: i8) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed8,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 16-bit signed integer.
    #[inline]
    fn encode_i16(self, _: i16) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed16,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 32-bit signed integer.
    #[inline]
    fn encode_i32(self, _: i32) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed32,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 64-bit signed integer.
    ///
    /// Defaults to using [Encoder::encode_u64] with the signed value casted.
    #[inline]
    fn encode_i64(self, _: i64) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed64,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 128-bit signed integer.
    ///
    /// Defaults to using [Encoder::encode_u128] with the signed value casted.
    #[inline]
    fn encode_i128(self, _: i128) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed128,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a usize value.
    #[inline]
    fn encode_usize(self, _: usize) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Usize,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a isize value.
    #[inline]
    fn encode_isize(self, _: isize) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Isize,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 32-bit floating point value.
    ///
    /// Default to taking the 32-bit in-memory IEEE 754 encoding and writing it byte-by-byte.
    #[inline]
    fn encode_f32(self, _: f32) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Float32,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a 64-bit floating point value.
    ///
    /// Default to taking the 64-bit in-memory IEEE 754 encoding and writing it byte-by-byte.
    #[inline]
    fn encode_f64(self, _: f64) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Float64,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode an optional value that is present.
    #[inline]
    fn encode_some(self) -> Result<Self::Some, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Option,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode an optional value that is absent.
    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Option,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a sequence with a known length.
    #[inline]
    fn encode_sequence(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Sequence,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a map with a known length.
    #[inline]
    fn encode_map(self, _: usize) -> Result<Self::Map, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Map,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a struct.
    #[inline]
    fn encode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Struct,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a tuple struct.
    #[inline]
    fn encode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Tuple,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode a unit struct.
    #[inline]
    fn encode_unit_struct(self) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::UnitStruct,
            &ExpectingWrapper(self),
        )))
    }

    /// Encode an enum variant.
    #[inline]
    fn encode_variant(self) -> Result<Self::Variant, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Variant,
            &ExpectingWrapper(self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T>(T);

impl<'de, T> Expecting for ExpectingWrapper<T>
where
    T: Encoder,
{
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
