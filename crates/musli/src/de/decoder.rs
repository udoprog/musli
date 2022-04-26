use core::fmt;

use crate::error::Error;
use crate::expecting::{self, Expecting, InvalidType};

struct RefVisitorExpected<T>(T);

impl<'de, T> fmt::Display for RefVisitorExpected<T>
where
    T: ReferenceVisitor<'de>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expected(f)
    }
}

/// A visitor for data where it might be possible to borrow it without copying
/// from the underlying [Decoder].
///
/// A visitor is required with [Decoder::decode_bytes] and
/// [Decoder::decode_string] because the caller doesn't know if the encoding
/// format is capable of producing references to the underlying data directly or
/// if it needs to be processed.
///
/// By requiring a visitor we ensure that the caller has to handle both
/// scenarios, even if one involves erroring. A type like
/// [Cow][std::borrow::Cow] is an example of a type which can comfortably handle
/// both.
pub trait ReferenceVisitor<'de>: Sized {
    /// The value being visited.
    type Target: ?Sized;
    /// The value produced.
    type Ok;
    /// The error produced.
    type Error: Error;

    /// Format an error indicating what was expected.
    ///
    /// Override to be more specific about the type that failed.
    fn expected(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit a string that is borrowed directly from the source data.
    #[inline]
    fn visit_ref(self, string: &'de Self::Target) -> Result<Self::Ok, Self::Error> {
        self.visit(string)
    }

    /// Visit a string that is provided from the decoder in any manner possible.
    /// Which might require additional decoding work.
    #[inline]
    fn visit(self, _: &Self::Target) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(RefVisitorExpected(self)))
    }
}

/// A pack that can construct encoders.
pub trait PackDecoder<'de> {
    /// Error type raised by this unpack.
    type Error: Error;

    /// The encoder to use for the pack.
    type Decoder<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Return decoder to unpack the next element.
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error>;

    /// Finish unpacking.
    fn end(self) -> Result<(), Self::Error>;
}

/// Trait governing how to decode a sequence.
pub trait SequenceDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder for individual items.
    type Decoder<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next element.
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error>;
}

/// Trait governing how to decode a sequence of pairs.
///
/// Each invocation of [PairsDecoder::next] returns an implementation of
/// [PairDecoder].
pub trait PairsDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a key.
    type Decoder<'this>: PairDecoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next key. This returns `Ok(None)` where there are no more elements to decode.
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error>;
}

/// Trait governing how to decode a field.
pub trait PairDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a tuple field index.
    type First<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a tuple field value.
    type Second: Decoder<'de, Error = Self::Error>;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error>;

    /// Decode the second value in the pair..
    fn second(self) -> Result<Self::Second, Self::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_second(self) -> Result<bool, Self::Error>;
}

/// Trait governing the implementation of a decoder.
pub trait Decoder<'de>: Sized {
    /// Error type raised by the decoder.
    type Error: Error;
    /// Pack decoder implementation.
    type Pack: PackDecoder<'de, Error = Self::Error>;
    /// Sequence decoder implementation.
    type Sequence: SequenceDecoder<'de, Error = Self::Error>;
    /// Map decoder implementation.
    type Map: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a value that is present.
    type Some: Decoder<'de, Error = Self::Error>;
    /// Decoder for a struct.
    ///
    /// The caller receives a [PairsDecoder] which when advanced with
    /// [PairsDecoder::next] indicates the fields of the structure.
    type Struct: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a tuple struct.
    ///
    /// The caller receives a [PairsDecoder] which when advanced with
    /// [PairsDecoder::next] indicates the elements in the tuple.
    type Tuple: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a variant.
    ///
    /// The caller receives a [PairDecoder] which when advanced with
    /// [PairDecoder::first] indicates which variant is being decoded and
    /// [PairDecoder::second] is the content of the variant.
    type Variant: PairDecoder<'de, Error = Self::Error>;

    /// Format the human-readable message that should occur if the decoder was
    /// expecting to decode some specific kind of value.
    ///
    /// ```
    /// use std::fmt;
    ///
    /// use musli::de::Decoder;
    /// use musli::never::Never;
    ///
    /// struct MyDecoder;
    ///
    /// impl Decoder<'_> for MyDecoder {
    ///     type Error = String;
    ///     type Pack = Never<Self>;
    ///     type Sequence = Never<Self>;
    ///     type Map = Never<Self>;
    ///     type Some = Never<Self>;
    ///     type Struct = Never<Self>;
    ///     type Tuple = Never<Self>;
    ///     type Variant = Never<Self>;
    ///
    ///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "32-bit unsigned integers")
    ///     }
    ///
    ///     fn decode_u32(self) -> Result<u32, Self::Error> {
    ///         Ok(42)
    ///     }
    /// }
    /// ```
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Decode a unit or something that is empty.
    #[inline]
    fn decode_unit(self) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unit,
            &ExpectingWrapper(self),
        )))
    }

    /// Construct an unpack that can decode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to decode all of the
    /// elements in the packed sequence from an as compact format as possible
    /// compatible with what's being returned by
    /// [Encoder::pack][crate::Encoder::encode_pack].
    #[inline]
    fn decode_pack(self) -> Result<Self::Pack, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Pack,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a fixed-length array.
    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Array,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a sequence of bytes whos length is encoded in the payload.
    #[inline]
    fn decode_bytes<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        Err(Self::Error::message(InvalidType::new(
            expecting::Bytes,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a string slice from the current decoder.
    #[inline]
    fn decode_string<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = str, Error = Self::Error>,
    {
        Err(Self::Error::message(InvalidType::new(
            expecting::String,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a boolean.
    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Bool,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a character.
    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Char,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 8-bit unsigned integer (a.k.a. a byte).
    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned8,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 16-bit unsigned integer.
    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned16,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit unsigned integer.
    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit unsigned integer.
    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 128-bit unsigned integer.
    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned128,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 8-bit signed integer.
    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed8,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 16-bit signed integer.
    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed16,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit signed integer.
    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit signed integer.
    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 128-bit signed integer.
    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed128,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a usize value.
    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Usize,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a isize value.
    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Isize,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit floating point value.
    ///
    /// Default to reading the 32-bit in-memory IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Float32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit floating point value.
    ///
    /// Default to reading the 64-bit in-memory IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Float64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode an optional value.
    #[inline]
    fn decode_option(self) -> Result<Option<Self::Some>, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Option,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a sequence, this returns a decoder that can be used to define the structure of the sequence.
    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Sequence,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a map, this returns a decoder that can be used to extract map-like values.
    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Map,
            &ExpectingWrapper(self),
        )))
    }

    /// Return a helper to decode a struct with named fields.
    #[inline]
    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Struct,
            &ExpectingWrapper(self),
        )))
    }

    /// Return a helper to decode a tuple struct.
    #[inline]
    fn decode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Tuple,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a unit variant.
    #[inline]
    fn decode_unit_struct(self) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::UnitStruct,
            &ExpectingWrapper(self),
        )))
    }

    /// Return decoder for a variant.
    #[inline]
    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
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
    T: Decoder<'de>,
{
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
