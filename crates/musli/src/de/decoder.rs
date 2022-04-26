use core::fmt;

use crate::error::Error;

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
        Err(Self::Error::collect_from_display(RefVisitorExpected(self)))
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
    fn finish(self) -> Result<(), Self::Error>;
}

/// Trait governing how to decode a sequence.
pub trait SequenceDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder for individual items.
    type Next<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next element.
    fn decode_next(&mut self) -> Result<Option<Self::Next<'_>>, Self::Error>;
}

/// Trait governing how to decode a map entry.
pub trait MapEntryDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a key.
    type Key<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a key.
    type Value<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Decode the next key.
    fn decode_key(&mut self) -> Result<Self::Key<'_>, Self::Error>;

    /// Follow up the decoding of a key by decoding a value.
    fn decode_value(&mut self) -> Result<Self::Value<'_>, Self::Error>;
}

/// Trait governing how to decode a map.
pub trait MapDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a key.
    type Entry<'this>: MapEntryDecoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next key. This returns `Ok(None)` where there are no more elements to decode.
    fn decode_entry(&mut self) -> Result<Option<Self::Entry<'_>>, Self::Error>;
}

/// Trait governing how to decode a map.
pub trait StructDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a key.
    type Field<'this>: PairDecoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next key. This returns `Ok(None)` where there are no more elements to decode.
    fn decode_field(&mut self) -> Result<Option<Self::Field<'_>>, Self::Error>;
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

    /// Decoder for the next index.
    fn decode_first(&mut self) -> Result<Self::First<'_>, Self::Error>;

    /// Decoder for the next value.
    fn decode_second(self) -> Result<Self::Second, Self::Error>;

    /// Indicate that the second element is not compatible with the current
    /// struct and skip it.
    ///
    /// Returns a boolean indicating if the second value was successfully
    /// skipped.
    fn skip_second(self) -> Result<bool, Self::Error>;
}

/// Trait governing the implementation of a decoder.
pub trait Decoder<'de>: Sized {
    /// Error type raised by the decoder.
    type Error: Error;

    /// Trait for an unpack.
    type Pack: PackDecoder<'de, Error = Self::Error>;

    /// The type of a sequence decoder.
    type Sequence: SequenceDecoder<'de, Error = Self::Error>;

    /// The type of a map decoder.
    type Map: MapDecoder<'de, Error = Self::Error>;

    /// Decoder to use when an optional value is present.
    type Some: Decoder<'de, Error = Self::Error>;

    /// Decoder returned to decode a struct variant.
    type Struct: StructDecoder<'de, Error = Self::Error>;

    /// Decoder returned to decode a tuple struct.
    type Tuple: StructDecoder<'de, Error = Self::Error>;

    /// Decode a variant.
    type Variant: PairDecoder<'de, Error = Self::Error>;

    /// An expectation error. Every other implementation defers to this to
    /// report that something unexpected happened.
    fn expected(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Decode a unit, or something that is empty.
    #[inline]
    fn decode_unit(self) -> Result<(), Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Construct an unpack that can decode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to decode all of the
    /// elements in the packed sequence from an as compact format as possible
    /// compatible with what's being returned by
    /// [Encoder::pack][crate::Encoder::encode_pack].
    #[inline]
    fn decode_pack(self) -> Result<Self::Pack, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a fixed-length array.
    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a sequence of bytes whos length is encoded in the payload.
    #[inline]
    fn decode_bytes<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a string slice from the current decoder.
    #[inline]
    fn decode_string<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = str, Error = Self::Error>,
    {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a boolean.
    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a character.
    #[inline]
    fn decode_char(self) -> Result<char, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 8-bit unsigned integer (a.k.a. a byte).
    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 16-bit unsigned integer.
    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 32-bit unsigned integer.
    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 64-bit unsigned integer.
    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 128-bit unsigned integer.
    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 8-bit signed integer.
    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 16-bit signed integer.
    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 32-bit signed integer.
    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 64-bit signed integer.
    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 128-bit signed integer.
    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a usize value.
    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a isize value.
    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 32-bit floating point value.
    ///
    /// Default to reading the 32-bit in-memory IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f32(self) -> Result<f32, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a 64-bit floating point value.
    ///
    /// Default to reading the 64-bit in-memory IEEE 754 encoding byte-by-byte.
    #[inline]
    fn decode_f64(self) -> Result<f64, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode an optional value.
    #[inline]
    fn decode_option(self) -> Result<Option<Self::Some>, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a sequence, this returns a decoder that can be used to define the structure of the sequence.
    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a map, this returns a decoder that can be used to extract map-like values.
    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Return a helper to decode a struct with named fields.
    #[inline]
    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Return a helper to decode a tuple struct.
    #[inline]
    fn decode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Decode a unit variant.
    #[inline]
    fn decode_unit_struct(self) -> Result<(), Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }

    /// Return decoder for a variant.
    #[inline]
    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
        Err(Self::Error::collect_from_display(Expected(self)))
    }
}

struct Expected<D>(D);

impl<'de, D> fmt::Display for Expected<D>
where
    D: Decoder<'de>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expected(f)
    }
}
