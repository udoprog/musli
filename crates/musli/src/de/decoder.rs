use crate::error::Error;

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
    type Field<'this>: StructFieldDecoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next key. This returns `Ok(None)` where there are no more elements to decode.
    fn decode_field(&mut self) -> Result<Option<Self::Field<'_>>, Self::Error>;
}

/// Trait governing how to decode a field.
pub trait StructFieldDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a field name.
    type FieldTag<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a field value.
    type FieldValue<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Decode for the name of the field.
    fn decode_field_tag(&mut self) -> Result<Self::FieldTag<'_>, Self::Error>;

    /// Decoder for the value of the field.
    fn decode_field_value(&mut self) -> Result<Self::FieldValue<'_>, Self::Error>;

    /// Indicate that the identified tag doesn't exist and should be skipped.
    ///
    /// The returned boolean indicates whether the field was sucessfully
    /// skipped.
    fn skip_field_value(&mut self) -> Result<bool, Self::Error>;
}

/// Trait governing how to decode a map.
pub trait TupleDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a field.
    type Field<'this>: TupleFieldDecoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next key. This returns `Ok(None)` where there are no more elements to decode.
    fn decode_field(&mut self) -> Result<Option<Self::Field<'_>>, Self::Error>;
}

/// Trait governing how to decode a field.
pub trait TupleFieldDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a tuple field index.
    type FieldTag<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a tuple field value.
    type FieldValue<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Decoder for the next index.
    fn decode_field_tag(&mut self) -> Result<Self::FieldTag<'_>, Self::Error>;

    /// Decoder for the next value.
    fn decode_field_value(&mut self) -> Result<Self::FieldValue<'_>, Self::Error>;

    /// Indicate that the identified tag doesn't exist and should be skipped.
    ///
    /// The returned boolean indicates whether the field was sucessfully
    /// skipped.
    fn skip_field_value(&mut self) -> Result<bool, Self::Error>;
}

/// Trait governing how to decode a variant.
pub trait VariantDecoder<'de> {
    /// Error type.
    type Error: Error;

    /// The decoder to use for a variant tag.
    type VariantTag<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a variant value.
    type VariantValue: Decoder<'de, Error = Self::Error>;

    /// Decoder for the next tag.
    fn decode_variant_tag(&mut self) -> Result<Self::VariantTag<'_>, Self::Error>;

    /// Decoder for the next value.
    fn decode_variant_value(self) -> Result<Self::VariantValue, Self::Error>;
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
    type Tuple: TupleDecoder<'de, Error = Self::Error>;

    /// Decode a variant.
    type Variant: VariantDecoder<'de, Error = Self::Error>;

    /// Decode a unit, or something that is empty.
    fn decode_unit(self) -> Result<(), Self::Error>;

    /// Construct an unpack that can decode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to decode all of the
    /// elements in the packed sequence from an as compact format as possible
    /// compatible with what's being returned by
    /// [Encoder::pack][crate::Encoder::encode_pack].
    fn decode_pack(self) -> Result<Self::Pack, Self::Error>;

    /// Decode a fixed-length array.
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error>;

    /// Decode a sequence of bytes whos length is encoded in the payload.
    fn decode_bytes(self) -> Result<&'de [u8], Self::Error>;

    /// Decode a string slice from the current decoder.
    fn decode_str(self) -> Result<&'de str, Self::Error>;

    /// Decode a boolean.
    fn decode_bool(self) -> Result<bool, Self::Error>;

    /// Decode a character.
    fn decode_char(self) -> Result<char, Self::Error>;

    /// Decode a 8-bit unsigned integer (a.k.a. a byte).
    fn decode_u8(self) -> Result<u8, Self::Error>;

    /// Decode a 16-bit unsigned integer.
    fn decode_u16(self) -> Result<u16, Self::Error>;

    /// Decode a 32-bit unsigned integer.
    fn decode_u32(self) -> Result<u32, Self::Error>;

    /// Decode a 64-bit unsigned integer.
    fn decode_u64(self) -> Result<u64, Self::Error>;

    /// Decode a 128-bit unsigned integer.
    fn decode_u128(self) -> Result<u128, Self::Error>;

    /// Decode a 8-bit signed integer.
    fn decode_i8(self) -> Result<i8, Self::Error>;

    /// Decode a 16-bit signed integer.
    fn decode_i16(self) -> Result<i16, Self::Error>;

    /// Decode a 32-bit signed integer.
    fn decode_i32(self) -> Result<i32, Self::Error>;

    /// Decode a 64-bit signed integer.
    fn decode_i64(self) -> Result<i64, Self::Error>;

    /// Decode a 128-bit signed integer.
    fn decode_i128(self) -> Result<i128, Self::Error>;

    /// Decode a usize value.
    fn decode_usize(self) -> Result<usize, Self::Error>;

    /// Decode a isize value.
    fn decode_isize(self) -> Result<isize, Self::Error>;

    /// Decode a 32-bit floating point value.
    ///
    /// Default to reading the 32-bit in-memory IEEE 754 encoding byte-by-byte.
    fn decode_f32(self) -> Result<f32, Self::Error>;

    /// Decode a 64-bit floating point value.
    ///
    /// Default to reading the 64-bit in-memory IEEE 754 encoding byte-by-byte.
    fn decode_f64(self) -> Result<f64, Self::Error>;

    /// Decode an optional value.
    fn decode_option(self) -> Result<Option<Self::Some>, Self::Error>;

    /// Decode a sequence, this returns a decoder that can be used to define the structure of the sequence.
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error>;

    /// Decode a map, this returns a decoder that can be used to extract map-like values.
    fn decode_map(self) -> Result<Self::Map, Self::Error>;

    /// Return a helper to decode a struct with named fields.
    fn decode_struct(self, fields: usize) -> Result<Self::Struct, Self::Error>;

    /// Return a helper to decode a tuple struct.
    fn decode_tuple(self, fields: usize) -> Result<Self::Tuple, Self::Error>;

    /// Decode a unit variant.
    fn decode_unit_struct(self) -> Result<(), Self::Error>;

    /// Return decoder for a variant.
    fn decode_variant(self) -> Result<Self::Variant, Self::Error>;
}
