use core::fmt;

use crate::de::{NumberVisitor, SizeHint, TypeHint, ValueVisitor, Visitor};
use crate::error::Error;
use crate::expecting::{self, Expecting};
use crate::mode::Mode;
use crate::Context;

/// Trait that allows a type to be repeatedly coerced into a decoder.
pub trait AsDecoder {
    /// Error type raised by calling `as_decoder`.
    type Error: Error;

    /// The decoder we reborrow as.
    type Decoder<'this>: Decoder<'this, Error = Self::Error>
    where
        Self: 'this;

    /// Borrow self as a new decoder.
    fn as_decoder<C>(&self, cx: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;
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
    #[must_use = "decoders must be consumed"]
    fn next<C>(&mut self, cx: &mut C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Stop decoding the current pack.
    ///
    /// This is required to call after a pack has finished decoding.
    fn end<C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
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
    fn size_hint(&self) -> SizeHint;

    /// Decode the next element.
    #[must_use = "decoders must be consumed"]
    fn next<C>(&mut self, cx: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Stop decoding the current sequence.
    ///
    /// This is required to call after a sequence has finished decoding.
    fn end<C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
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
    fn size_hint(&self) -> SizeHint;

    /// Decode the next key. This returns `Ok(None)` where there are no more
    /// elements to decode.
    #[must_use = "decoders must be consumed"]
    fn next<C>(&mut self, cx: &mut C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// End the pair decoder.
    ///
    /// If there are any remaining elements in the sequence of pairs, this
    /// indicates that they should be flushed.
    fn end<C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
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
    #[must_use = "decoders must be consumed"]
    fn first<C>(&mut self, cx: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "decoders must be consumed"]
    fn second<C>(self, cx: &mut C) -> Result<Self::Second, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_second<C>(self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing how to decode a variant.
pub trait VariantDecoder<'de> {
    /// Error type.
    type Error: Error;

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
    fn tag<C>(&mut self, cx: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "decoders must be consumed"]
    fn variant<C>(&mut self, cx: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_variant<C>(&mut self, cx: &mut C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>;

    /// End the pair decoder.
    fn end<C>(self, cx: &mut C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>;
}

/// Trait governing the implementation of a decoder.
pub trait Decoder<'de>: Sized {
    /// Error type raised by the decoder.
    type Error: Error;
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
    /// Map decoder implementation.
    type Map: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a struct.
    ///
    /// The caller receives a [PairsDecoder] which when advanced with
    /// [PairsDecoder::next] indicates the fields of the structure.
    type Struct: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a variant.
    ///
    /// The caller receives a [PairDecoder] which when advanced with
    /// [PairDecoder::first] indicates which variant is being decoded and
    /// [PairDecoder::second] is the content of the variant.
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
    /// use std::fmt;
    ///
    /// use musli::Context;
    /// use musli::de::{self, Decoder};
    ///
    /// struct MyDecoder;
    ///
    /// #[musli::decoder]
    /// impl Decoder<'_> for MyDecoder {
    ///     type Error = de::Error;
    ///
    ///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "32-bit unsigned integers")
    ///     }
    ///
    ///     fn decode_u32<C>(self, _: &mut C) -> Result<u32, C::Error>
    ///     where
    ///         C: Context<Input = Self::Error>
    ///     {
    ///         Ok(42)
    ///     }
    /// }
    /// ```
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Return a [TypeHint] indicating which type is being produced by the
    /// [Decoder].
    ///
    /// Not all formats support type hints, and they might be ranging from
    /// detailed (`a 32-bit unsigned integer`) to vague (`a number`).
    ///
    /// This is used to construct dynamic containers of types.
    fn type_hint<C>(&mut self, _: &mut C) -> Result<TypeHint, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    /// use musli::de::{AsDecoder, PairsDecoder, PairDecoder};
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
    /// impl<'de, M> Decode<'de, M> for MyVariantType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut buffer = decoder.decode_buffer::<M, _>(cx)?;
    ///
    ///         let mut st = buffer.as_decoder(cx)?.decode_map(cx)?;
    ///
    ///         let mut discriminant = None::<u32>;
    ///
    ///         while let Some(mut e) = st.next()? {
    ///             let found = e.first()?.decode_string(cx, musli::utils::visit_string_fn(|cx, string| {
    ///                 Ok(string == "type")
    ///             }))?;
    ///
    ///             if found {
    ///                 discriminant = Some(e.second(cx).and_then(|v| Decode::<M>::decode(cx, v))?);
    ///                 break;
    ///             }
    ///         }
    ///
    ///         st.end(cx)?;
    ///
    ///         match discriminant {
    ///             Some(0) => Ok(MyVariantType::Variant1),
    ///             Some(1) => Ok(MyVariantType::Variant2(buffer.as_decoder(cx).and_then(|v| Decode::<M>::decode(cx, v))?)),
    ///             Some(other) => Err(cx.invalid_variant_tag("MyVariantType", other)),
    ///             None => Err(cx.missing_variant_tag("MyVariantType")),
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_buffer<M, C>(self, cx: &mut C) -> Result<Self::Buffer, C::Error>
    where
        M: Mode,
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct UnitType;
    ///
    /// impl<'de, M> Decode<'de, M> for UnitType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
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
    fn decode_unit<C>(self, cx: &mut C) -> Result<(), C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: bool,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_bool()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bool<C>(self, cx: &mut C) -> Result<bool, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: char,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_char()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_char<C>(self, cx: &mut C) -> Result<char, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: u8,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u8()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u8<C>(self, cx: &mut C) -> Result<u8, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: u16,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u16()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u16<C>(self, cx: &mut C) -> Result<u16, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    /// use musli::de;
    ///
    /// struct MyType {
    ///     data: u32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut Context<Input = D::Error>, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u32(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u32<C>(self, cx: &mut C) -> Result<u32, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: u64,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u64()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u64<C>(self, cx: &mut C) -> Result<u64, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: u128,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u128()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u128<C>(self, cx: &mut C) -> Result<u128, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: i8,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i8()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i8<C>(self, cx: &mut C) -> Result<i8, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: i16,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i16()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i16<C>(self, cx: &mut C) -> Result<i16, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: i32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i32()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i32<C>(self, cx: &mut C) -> Result<i32, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: i64,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i64()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i64<C>(self, cx: &mut C) -> Result<i64, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: i128,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i128()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i128<C>(self, cx: &mut C) -> Result<i128, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: usize,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_usize()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_usize<C>(self, cx: &mut C) -> Result<usize, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: isize,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_isize()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_isize<C>(self, cx: &mut C) -> Result<isize, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: f32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_f32()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f32<C>(self, cx: &mut C) -> Result<f32, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: f64,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_f64()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f64<C>(self, cx: &mut C) -> Result<f64, C::Error>
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
    fn decode_number<V>(
        self,
        cx: &mut V::Context,
        _: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: NumberVisitor<'de, Error = Self::Error>,
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_array()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_array<C, const N: usize>(self, cx: &mut C) -> Result<[u8; N], C::Error>
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
    /// use std::marker;
    ///
    /// use musli::{Context, Decode, Decoder, Mode};
    /// use musli::de::{ValueVisitor};
    ///
    /// struct BytesReference<'de> {
    ///     data: &'de [u8],
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for BytesReference<'de> where M: Mode {
    ///     #[inline]
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor<C, E>(marker::PhantomData<(C, E)>);
    ///
    ///         impl<'de, C, E> ValueVisitor<'de> for Visitor<C, E>
    ///         where
    ///             C: Context<Input = E>,
    ///             E: Error,
    ///         {
    ///             type Target = [u8];
    ///             type Ok = &'de [u8];
    ///             type Error = E;
    ///             type Context = C;
    ///
    ///             #[inline]
    ///             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///                 write!(f, "exact bytes reference")
    ///             }
    ///
    ///             #[inline]
    ///             fn visit_borrowed(self, bytes: &'de [u8]) -> Result<Self::Ok, Self::Error> {
    ///                 Ok(bytes)
    ///             }
    ///         }
    ///
    ///         Ok(Self {
    ///             data: decoder.decode_bytes(Visitor(marker::PhantomData))?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bytes<V>(
        self,
        cx: &mut V::Context,
        _: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
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
    /// use std::marker;
    ///
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, ValueVisitor};
    /// use musli::mode::Mode;
    /// use musli::error::Error;
    ///
    /// struct StringReference<'de> {
    ///     data: &'de str,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for StringReference<'de> where M: Mode {
    ///     #[inline]
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor<C, E>(marker::PhantomData<(C, E)>);
    ///
    ///         impl<'de, C, E> ValueVisitor<'de> for Visitor<C, E>
    ///         where
    ///             C: Context<Input = E>,
    ///             E: Error,
    ///         {
    ///             type Target = str;
    ///             type Ok = &'de str;
    ///             type Error = E;
    ///             type Context = C;
    ///
    ///             #[inline]
    ///             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///                 write!(f, "exact bytes reference")
    ///             }
    ///
    ///             #[inline]
    ///             fn visit_borrowed(self, cx: &mut Self::Context, bytes: &'de str) -> Result<Self::Ok, Self::Context::Error> {
    ///                 Ok(bytes)
    ///             }
    ///         }
    ///
    ///         Ok(Self {
    ///             data: decoder.decode_string(cx, Visitor(marker::PhantomData))?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_string<V>(
        self,
        cx: &mut V::Context,
        _: V,
    ) -> Result<V::Ok, <V::Context as Context>::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
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
    /// use musli::{Context, Decode, Decoder, Mode};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let data = if let Some(decoder) = decoder.decode_option()? {
    ///             Some(<String as Decode<M>>::decode(decoder)?)
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
    fn decode_option<C>(self, cx: &mut C) -> Result<Option<Self::Some>, C::Error>
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
    /// use musli::{Context, Mode};
    /// use musli::de::{Decode, Decoder, PackDecoder};
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for PackedStruct where M: Mode {
    ///     #[inline]
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut unpack = decoder.decode_pack(cx)?;
    ///         let field = unpack.next(cx).and_then(|v| Decode::<M>::decode(cx, v))?;
    ///         let data = unpack.next(cx).and_then(|v| Decode::<M>::decode(cx, v))?;
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
    fn decode_pack<C>(self, cx: &mut C) -> Result<Self::Pack, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    /// use musli::de::{SequenceDecoder};
    ///
    /// struct MyType {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut seq = decoder.decode_sequence(cx)?;
    ///         let mut data = Vec::new();
    ///
    ///         while let Some(decoder) = seq.next(cx)? {
    ///             data.push(<String as Decode<M>>::decode(cx, decoder)?);
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
    fn decode_sequence<C>(self, cx: &mut C) -> Result<Self::Sequence, C::Error>
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
    /// use musli::mode::Mode;
    ///
    /// struct TupleStruct(String, u32);
    ///
    /// impl<'de, M> Decode<'de, M> for TupleStruct where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut tuple = decoder.decode_tuple(cx, 2)?;
    ///         let string = tuple.next(cx).and_then(|v| <String as Decode<M>>::decode(cx, v))?;
    ///         let integer = tuple.next(cx).and_then(|v| <u32 as Decode<M>>::decode(cx, v))?;
    ///         tuple.end(cx)?;
    ///         Ok(Self(string, integer))
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_tuple<C>(
        self,
        cx: &mut C,
        #[allow(unused)] len: usize,
    ) -> Result<Self::Tuple, C::Error>
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
    /// use musli::de::{Decode, Decoder, PairsDecoder, PairDecoder};
    /// use musli::mode::Mode;
    ///
    /// struct MapStruct {
    ///     data: HashMap<String, u32>,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for MapStruct where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut map = decoder.decode_map()?;
    ///         let mut data = HashMap::with_capacity(map.size_hint().or_default());
    ///
    ///         while let Some(mut entry) = map.next()? {
    ///             let key = entry.first().and_then(<String as Decode<M>>::decode)?;
    ///             let value = entry.second().and_then(<u32 as Decode<M>>::decode)?;
    ///             data.insert(key, value);
    ///         }
    ///
    ///         map.end()?;
    ///
    ///         Ok(Self {
    ///             data
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_map<C>(self, cx: &mut C) -> Result<Self::Map, C::Error>
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
    /// use musli::{Context, Decode, Decoder, Mode};
    /// use musli::de::{PairsDecoder, PairDecoder};
    ///
    /// struct Struct {
    ///     string: String,
    ///     integer: u32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for Struct where M: Mode {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut st = decoder.decode_map(cx)?;
    ///         let mut string = None;
    ///         let mut integer = None;
    ///
    ///         while let Some(mut entry) = st.next(cx)? {
    ///             // Note: to avoid allocating `decode_string` needs to be used with a visitor.
    ///             let tag = entry.first(cx).and_then(|v| <String as Decode<M>>::decode(cx, v))?;
    ///
    ///             match tag.as_str() {
    ///                 "string" => {
    ///                     string = Some(entry.second(cx).and_then(|v| <String as Decode<M>>::decode(cx, v))?);
    ///                 }
    ///                 "integer" => {
    ///                     integer = Some(entry.second(cx).and_then(|v| <u32 as Decode<M>>::decode(cx, v))?);
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
    fn decode_struct<C>(self, cx: &mut C, _: usize) -> Result<Self::Struct, C::Error>
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
    /// use musli::{Context, Decode, Mode};
    /// use musli::de::{Decoder, VariantDecoder};
    ///
    /// enum Enum {
    ///     Variant1(u32),
    ///     Variant2(String),
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for Enum
    /// where
    ///     M: Mode
    /// {
    ///     fn decode<C, D>(cx: &mut C, decoder: D) -> Result<Self, C::Error>
    ///     where
    ///         C: Context<Input = D::Error>,
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut variant = decoder.decode_variant(cx)?;
    ///         let tag = variant.tag(cx).and_then(|v| <usize as Decode<M>>::decode(cx, v))?;
    ///
    ///         let this = match tag {
    ///             0 => {
    ///                 Self::Variant1(variant.variant(cx).and_then(|v| <u32 as Decode<M>>::decode(cx, v))?)
    ///             }
    ///             1 => {
    ///                 Self::Variant2(variant.variant(cx).and_then(|v| <String as Decode<M>>::decode(cx, v))?)
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
    fn decode_variant<C>(self, cx: &mut C) -> Result<Self::Variant, C::Error>
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
    fn decode_any<C, V>(self, cx: &mut C, _: V) -> Result<V::Ok, C::Error>
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
