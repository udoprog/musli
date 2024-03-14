use core::fmt;

use crate::de::{NumberVisitor, SizeHint, TypeHint, ValueVisitor, Visitor};
use crate::expecting::{self, Expecting};
use crate::Context;

/// Trait that allows a type to be repeatedly coerced into a decoder.
pub trait AsDecoder {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder we reborrow as.
    type Decoder<'this>: Decoder<'this, Error = Self::Error>
    where
        Self: 'this;

    /// Borrow self as a new decoder.
    fn as_decoder<C>(&self, cx: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// A pack that can construct decoders.
pub trait PackDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The encoder to use for the pack.
    type Decoder<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Return decoder to unpack the next element.
    #[must_use = "decoders must be consumed"]
    fn next<C>(&mut self, cx: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Stop decoding the current pack.
    ///
    /// This is required to call after a pack has finished decoding.
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a sequence.
pub trait SequenceDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder for individual items.
    type Decoder<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> SizeHint;

    /// Decode the next element.
    #[must_use = "decoders must be consumed"]
    fn next<C>(&mut self, cx: &C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Stop decoding the current sequence.
    ///
    /// This is required to call after a sequence has finished decoding.
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a sequence of pairs.
pub trait MapDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder to use for a key.
    type Entry<'this>: MapEntryDecoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining elements.
    fn size_hint(&self) -> SizeHint;

    /// Decode the next key. This returns `Ok(None)` where there are no more
    /// elements to decode.
    #[must_use = "Decoders must be consumed"]
    fn entry<C>(&mut self, cx: &C) -> Result<Option<Self::Entry<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// End the pair decoder.
    ///
    /// If there are any remaining elements in the sequence of pairs, this
    /// indicates that they should be flushed.
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode fields in a struct.
pub trait StructDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder to use for a key.
    type Field<'this>: StructFieldDecoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Get a size hint of known remaining fields.
    fn size_hint(&self) -> SizeHint;

    /// Decode the next field.
    #[must_use = "Decoders must be consumed"]
    fn field<C>(&mut self, cx: &C) -> Result<Option<Self::Field<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// End the pair decoder.
    ///
    /// If there are any remaining elements in the sequence of pairs, this
    /// indicates that they should be flushed.
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a map entry.
pub trait MapEntryDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder to use for a tuple field index.
    type MapKey<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a tuple field value.
    type MapValue: Decoder<'de, Error = Self::Error>;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "decoders must be consumed"]
    fn map_key<C>(&mut self, cx: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "decoders must be consumed"]
    fn map_value<C>(self, cx: &C) -> Result<Self::MapValue, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_map_value<C>(self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a struct field.
pub trait StructFieldDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder to use for a tuple field index.
    type FieldName<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a tuple field value.
    type FieldValue: Decoder<'de, Error = Self::Error>;

    /// Return the decoder for the field name.
    #[must_use = "decoders must be consumed"]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Decode the field value.
    #[must_use = "decoders must be consumed"]
    fn field_value<C>(self, cx: &C) -> Result<Self::FieldValue, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Indicate that the field value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_field_value<C>(self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a sequence of map pairs.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde deserialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait MapPairsDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder to use for a tuple field index.
    type MapPairsKey<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a tuple field value.
    type MapPairsValue<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Try to return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "decoders must be consumed"]
    fn map_pairs_key<C>(&mut self, cx: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Decode the value in the map.
    #[must_use = "decoders must be consumed"]
    fn map_pairs_value<C>(&mut self, cx: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Indicate that the value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_map_pairs_value<C>(&mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// End pair decoding.
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a sequence of struct pairs.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde deserialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait StructPairsDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder to use for a tuple field index.
    type FieldName<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for a tuple field value.
    type FieldValue<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Try to return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "decoders must be consumed"]
    fn field_name<C>(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "decoders must be consumed"]
    fn field_value<C>(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_field_value<C>(&mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// End pair decoding.
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a variant.
pub trait VariantDecoder<'de> {
    /// Error type for decoder.
    type Error: 'static;

    /// The decoder to use for the variant tag.
    type Tag<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// The decoder to use for the variant value.
    type Variant<'this>: Decoder<'de, Error = Self::Error>
    where
        Self: 'this;

    /// Return the decoder for the first value in the pair.
    ///
    /// If this is a map the first value would be the key of the map, if this is
    /// a struct the first value would be the field of the struct.
    #[must_use = "decoders must be consumed"]
    fn tag<C>(&mut self, cx: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "decoders must be consumed"]
    fn variant<C>(&mut self, cx: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_variant<C>(&mut self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// End the pair decoder.
    fn end<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing the implementation of a decoder.
pub trait Decoder<'de>: Sized {
    /// Error type for decoder.
    type Error: 'static;

    /// The type returned when the decoder is buffered.
    type Buffer: AsDecoder<Error = Self::Error>;
    /// Decoder for a value that is present.
    type Some: Decoder<'de, Error = Self::Error>;
    /// Pack decoder implementation.
    type Pack: PackDecoder<'de, Error = Self::Error>;
    /// Sequence decoder implementation.
    type Sequence: SequenceDecoder<'de, Error = Self::Error>;
    /// Tuple decoder implementation.
    type Tuple: PackDecoder<'de, Error = Self::Error>;
    /// Decoder for a map.
    type Map: MapDecoder<'de, Error = Self::Error>;
    /// Decoder for a sequence of map pairs.
    type MapPairs: MapPairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a struct.
    type Struct: StructDecoder<'de, Error = Self::Error>;
    /// Decoder for a sequence of struct pairs.
    type StructPairs: StructPairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a variant.
    type Variant: VariantDecoder<'de, Error = Self::Error>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::decoder]`][crate::decoder] attribute
    /// macro when implementing [`Decoder`].
    #[doc(hidden)]
    type __UseMusliDecoderAttributeMacro;

    /// Format the human-readable message that should occur if the decoder was
    /// expecting to decode some specific kind of value.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::fmt;
    /// use core::convert::Infallible;
    ///
    /// use musli::Context;
    /// use musli::de::{self, Decoder};
    ///
    /// struct MyDecoder;
    ///
    /// #[musli::decoder]
    /// impl Decoder<'_> for MyDecoder {
    ///     type Error = Infallible;
    ///
    ///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "32-bit unsigned integers")
    ///     }
    ///
    ///     fn decode_u32<C>(self, _: &C) -> Result<u32, C::Error>
    ///     where
    ///         C: Context<Input = Self::Error>
    ///     {
    ///         Ok(42)
    ///     }
    /// }
    /// ```
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Skip over the current value.
    fn skip<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(format_args!(
            "Skipping is not supported, expected {}",
            ExpectingWrapper(self).format()
        )))
    }

    /// Return a [TypeHint] indicating which type is being produced by the
    /// [Decoder].
    ///
    /// Not all formats support type hints, and they might be ranging from
    /// detailed (`a 32-bit unsigned integer`) to vague (`a number`).
    ///
    /// This is used to construct dynamic containers of types.
    fn type_hint<C>(&mut self, _: &C) -> Result<TypeHint, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Ok(TypeHint::Any)
    }

    /// Buffer the current decoder into a buffer that can be used multiple times.
    ///
    /// Buffering a decoder is necessary when additional introspection is needed
    /// to decode a type, but it also means that:
    ///
    /// * The entire contents of the decoder needs to be dynamically buffered in
    ///   memory.
    /// * The in-memory representation might be lossy in some trivial ways. Such
    ///   as arbitrary precision numbers being punted into a 64-bit float.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::{AsDecoder, MapDecoder, MapEntryDecoder};
    ///
    /// #[derive(Decode)]
    /// struct Variant2 {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// enum MyVariantType {
    ///     Variant1,
    ///     Variant2(Variant2),
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyVariantType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut buffer = decoder.decode_buffer(cx)?;
    ///
    ///         let mut st = buffer.as_decoder(cx)?.decode_map(cx)?;
    ///
    ///         let discriminant = loop {
    ///             let Some(mut e) = st.entry(cx)? else {
    ///                 return Err(cx.missing_variant_tag("MyVariantType"));
    ///             };
    ///
    ///             let found = e.map_key(cx)?.decode_string(cx, musli::utils::visit_owned_fn("a string that is 'type'", |cx: &C, string: &str| {
    ///                 Ok(string == "type")
    ///             }))?;
    ///
    ///             if found {
    ///                 break e.map_value(cx).and_then(|v| cx.decode(v))?;
    ///             }
    ///         };
    ///
    ///         st.end(cx)?;
    ///
    ///         match discriminant {
    ///             0 => Ok(MyVariantType::Variant1),
    ///             1 => Ok(MyVariantType::Variant2(buffer.as_decoder(cx).and_then(|v| cx.decode(v))?)),
    ///             other => Err(cx.invalid_variant_tag("MyVariantType", other)),
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_buffer<C>(self, cx: &C) -> Result<Self::Buffer, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(format_args!(
            "buffering not supported, expected {}",
            ExpectingWrapper(self).format()
        )))
    }

    /// Decode a unit or something that is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct UnitType;
    ///
    /// impl<'de, M> Decode<'de, M> for UnitType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_unit(cx)?;
    ///         Ok(UnitType)
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_unit<C>(self, cx: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unit,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a boolean.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: bool,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_bool(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bool<C>(self, cx: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Bool,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a character.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: char,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_char(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_char<C>(self, cx: &C) -> Result<char, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Char,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 8-bit unsigned integer (a.k.a. a byte).
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u8,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u8(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u8<C>(self, cx: &C) -> Result<u8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned8,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u16,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u16(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u16<C>(self, cx: &C) -> Result<u16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned16,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de;
    ///
    /// struct MyType {
    ///     data: u32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u32(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u32<C>(self, cx: &C) -> Result<u32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u64,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u64(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u64<C>(self, cx: &C) -> Result<u64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u128,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u128(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u128<C>(self, cx: &C) -> Result<u128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned128,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i8,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i8(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i8<C>(self, cx: &C) -> Result<i8, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed8,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i16,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i16(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i16<C>(self, cx: &C) -> Result<i16, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed16,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i32(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i32<C>(self, cx: &C) -> Result<i32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i64,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i64(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i64<C>(self, cx: &C) -> Result<i64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i128,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i128(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i128<C>(self, cx: &C) -> Result<i128, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed128,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode Rusts [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: usize,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_usize(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_usize<C>(self, cx: &C) -> Result<usize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Usize,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode Rusts [`isize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: isize,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_isize(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_isize<C>(self, cx: &C) -> Result<isize, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Isize,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: f32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_f32(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f32<C>(self, cx: &C) -> Result<f32, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Float32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: f64,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_f64(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f64<C>(self, cx: &C) -> Result<f64, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Float64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode an unknown number using a visitor that can handle arbitrary
    /// precision numbers.
    #[inline]
    fn decode_number<C, V>(self, cx: &C, _: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: NumberVisitor<'de, C>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Number,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a fixed-length array.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_array(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_array<C, const N: usize>(self, cx: &C) -> Result<[u8; N], C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Array,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a sequence of bytes whos length is encoded in the payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fmt;
    ///
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::ValueVisitor;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct BytesReference<'de> {
    ///     data: &'de [u8],
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for BytesReference<'de> {
    ///     #[inline]
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor;
    ///
    ///         impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
    ///         where
    ///             C: Context,
    ///         {
    ///             type Ok = &'de [u8];
    ///
    ///             #[inline]
    ///             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///                 write!(f, "a literal byte reference")
    ///             }
    ///
    ///             #[inline]
    ///             fn visit_borrowed(self, _: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
    ///                 Ok(bytes)
    ///             }
    ///         }
    ///
    ///         Ok(Self {
    ///             data: decoder.decode_bytes(cx, Visitor)?,
    ///         })
    ///     }
    /// }
    ///
    /// let value = musli_value::Value::Bytes(vec![0, 1, 2, 3]);
    /// assert_eq!(musli_value::decode::<BytesReference>(&value)?, BytesReference { data: &[0, 1, 2, 3] });
    ///
    /// let value = musli_value::Value::Number(42u32.into());
    /// assert_eq!(musli_value::decode::<BytesReference>(&value).unwrap_err().to_string(), "Expected bytes, but found number");
    /// # Ok::<_, musli_value::Error>(())
    /// ```
    #[inline]
    fn decode_bytes<C, V>(self, cx: &C, _: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, [u8]>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Bytes,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a string slice from the current decoder.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fmt;
    ///
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, ValueVisitor};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct StringReference<'de> {
    ///     data: &'de str,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for StringReference<'de> {
    ///     #[inline]
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor;
    ///
    ///         impl<'de, C> ValueVisitor<'de, C, str> for Visitor
    ///         where
    ///             C: Context,
    ///         {
    ///             type Ok = &'de str;
    ///
    ///             #[inline]
    ///             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///                 write!(f, "exact bytes reference")
    ///             }
    ///
    ///             #[inline]
    ///             fn visit_borrowed(self, cx: &C, bytes: &'de str) -> Result<Self::Ok, C::Error> {
    ///                 Ok(bytes)
    ///             }
    ///         }
    ///
    ///         Ok(Self {
    ///             data: decoder.decode_string(cx, Visitor)?,
    ///         })
    ///     }
    /// }
    ///
    /// let value = musli_value::Value::String(String::from("Hello!"));
    /// assert_eq!(musli_value::decode::<StringReference>(&value)?, StringReference { data: "Hello!" });
    ///
    /// let value = musli_value::Value::Number(42u32.into());
    /// assert_eq!(musli_value::decode::<StringReference>(&value).unwrap_err().to_string(), "Expected string, but found number");
    /// # Ok::<_, musli_value::Error>(())
    /// ```
    #[inline]
    fn decode_string<C, V>(self, cx: &C, _: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: ValueVisitor<'de, C, str>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::String,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode an optional value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let data = if let Some(decoder) = decoder.decode_option(cx)? {
    ///             Some(cx.decode(decoder)?)
    ///         } else {
    ///             None
    ///         };
    ///
    ///         Ok(Self {
    ///             data,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    #[must_use = "decoders must be consumed"]
    fn decode_option<C>(self, cx: &C) -> Result<Option<Self::Some>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Option,
            &ExpectingWrapper(self),
        )))
    }

    /// Construct an unpack that can decode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to decode all of the
    /// elements in the packed sequence from an as compact format as possible
    /// compatible with what's being returned by
    /// [Encoder::pack][crate::Encoder::encode_pack].
    ///
    /// ```
    /// use musli::{Context};
    /// use musli::de::{Decode, Decoder, PackDecoder};
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for PackedStruct {
    ///     #[inline]
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut unpack = decoder.decode_pack(cx)?;
    ///         let field = unpack.next(cx).and_then(|v| cx.decode(v))?;
    ///         let data = unpack.next(cx).and_then(|v| cx.decode(v))?;
    ///         unpack.end(cx)?;
    ///
    ///         Ok(Self {
    ///             field,
    ///             data,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_pack<C>(self, cx: &C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Pack,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::{SequenceDecoder};
    ///
    /// struct MyType {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut seq = decoder.decode_sequence(cx)?;
    ///         let mut data = Vec::new();
    ///
    ///         while let Some(decoder) = seq.next(cx)? {
    ///             data.push(cx.decode(decoder)?);
    ///         }
    ///
    ///         seq.end(cx)?;
    ///
    ///         Ok(Self {
    ///             data
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_sequence<C>(self, cx: &C) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Sequence,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a fixed-length sequence of elements of length `len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, PackDecoder};
    ///
    /// struct TupleStruct(String, u32);
    ///
    /// impl<'de, M> Decode<'de, M> for TupleStruct {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut tuple = decoder.decode_tuple(cx, 2)?;
    ///         let string = tuple.next(cx).and_then(|v| cx.decode(v))?;
    ///         let integer = tuple.next(cx).and_then(|v| cx.decode(v))?;
    ///         tuple.end(cx)?;
    ///         Ok(Self(string, integer))
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_tuple<C>(self, cx: &C, #[allow(unused)] len: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Tuple,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a map of unknown length.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::{MapDecoder, MapEntryDecoder};
    ///
    /// struct MapStruct {
    ///     data: HashMap<String, u32>,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MapStruct {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut map = decoder.decode_map(cx)?;
    ///         let mut data = HashMap::with_capacity(map.size_hint().or_default());
    ///
    ///         while let Some(mut entry) = map.entry(cx)? {
    ///             let key = entry.map_key(cx).and_then(|v| cx.decode(v))?;
    ///             let value = entry.map_value(cx).and_then(|v| cx.decode(v))?;
    ///             data.insert(key, value);
    ///         }
    ///
    ///         map.end(cx)?;
    ///
    ///         Ok(Self {
    ///             data
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_map<C>(self, cx: &C) -> Result<Self::Map, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Map,
            &ExpectingWrapper(self),
        )))
    }

    /// Simplified decoding a map of unknown length.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    #[inline]
    fn decode_map_pairs<C>(self, cx: &C) -> Result<Self::MapPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Map,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a struct which has an expected `len` number of elements.
    ///
    /// The `len` indicates how many fields the decoder is *expecting* depending
    /// on how many fields are present in the underlying struct being decoded,
    /// butit should only be considered advisory.
    ///
    /// The size of a struct might therefore change from one session to another.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::{StructDecoder, StructFieldDecoder};
    ///
    /// struct Struct {
    ///     string: String,
    ///     integer: u32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for Struct {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut st = decoder.decode_struct(cx, None)?;
    ///         let mut string = None;
    ///         let mut integer = None;
    ///
    ///         while let Some(mut field) = st.field(cx)? {
    ///             // Note: to avoid allocating `decode_string` needs to be used with a visitor.
    ///             let tag: String = field.field_name(cx).and_then(|v| cx.decode(v))?;
    ///
    ///             match tag.as_str() {
    ///                 "string" => {
    ///                     string = Some(field.field_value(cx).and_then(|v| cx.decode(v))?);
    ///                 }
    ///                 "integer" => {
    ///                     integer = Some(field.field_value(cx).and_then(|v| cx.decode(v))?);
    ///                 }
    ///                 tag => {
    ///                     return Err(cx.invalid_field_tag("Struct", tag))
    ///                 }
    ///             }
    ///         }
    ///
    ///         st.end(cx)?;
    ///
    ///         Ok(Self {
    ///             string: string.ok_or_else(|| cx.expected_tag("Struct", "string"))?,
    ///             integer: integer.ok_or_else(|| cx.expected_tag("Struct", "integer"))?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_struct<C>(self, cx: &C, _: Option<usize>) -> Result<Self::Struct, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Struct,
            &ExpectingWrapper(self),
        )))
    }

    /// Simplified decoding of a struct which has an expected `len` number of
    /// elements.
    ///
    /// The `len` indicates how many fields the decoder is *expecting* depending
    /// on how many fields are present in the underlying struct being decoded,
    /// butit should only be considered advisory.
    ///
    /// The size of a struct might therefore change from one session to another.
    #[inline]
    fn decode_struct_pairs<C>(self, cx: &C, _: Option<usize>) -> Result<Self::StructPairs, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Struct,
            &ExpectingWrapper(self),
        )))
    }

    /// Return decoder for a variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode};
    /// use musli::de::{Decoder, VariantDecoder};
    ///
    /// enum Enum {
    ///     Variant1(u32),
    ///     Variant2(String),
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for Enum {
    ///     fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Mode = M, Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut variant = decoder.decode_variant(cx)?;
    ///         let tag = variant.tag(cx).and_then(|v| cx.decode(v))?;
    ///
    ///         let this = match tag {
    ///             0 => {
    ///                 Self::Variant1(variant.variant(cx).and_then(|v| cx.decode(v))?)
    ///             }
    ///             1 => {
    ///                 Self::Variant2(variant.variant(cx).and_then(|v| cx.decode(v))?)
    ///             }
    ///             tag => {
    ///                 return Err(cx.invalid_variant_tag("Enum", tag));
    ///             }
    ///         };
    ///
    ///         variant.end(cx)?;
    ///         Ok(this)
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_variant<C>(self, cx: &C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Variant,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode dynamically through a [`Visitor`].
    ///
    /// If the current encoding does not support dynamic decoding,
    /// [`Visitor::visit_any`] will be called with the current decoder.
    #[inline]
    fn decode_any<C, V>(self, cx: &C, _: V) -> Result<V::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
        V: Visitor<'de, Error = Self::Error>,
    {
        Err(cx.message(format_args!(
            "visitor not supported, expected {}",
            ExpectingWrapper(self).format()
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T>(T);

impl<'de, T> Expecting for ExpectingWrapper<T>
where
    T: Decoder<'de>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
