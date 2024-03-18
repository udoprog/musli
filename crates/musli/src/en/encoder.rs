use core::fmt;
use core::marker::PhantomData;

use crate::expecting::{self, Expecting};
use crate::Context;

use super::{
    Encode, MapEncoder, MapEntriesEncoder, SequenceEncoder, StructEncoder, VariantEncoder,
};

/// Trait governing how the encoder works.
pub trait Encoder<C: ?Sized + Context>: Sized {
    /// The type returned by the encoder. For [Encode] implementations ensures
    /// that they are used correctly, since only functions returned by the
    /// [Encoder] is capable of returning this value.
    type Ok;
    /// Constructed [`Encoder`] with a different context.
    type WithContext<U>: Encoder<U, Ok = Self::Ok>
    where
        U: Context;
    /// A simple pack that packs a sequence of elements.
    type EncodePack<'this>: SequenceEncoder<C, Ok = Self::Ok>
    where
        C: 'this;
    /// Encoder returned when encoding an optional value which is present.
    type EncodeSome: Encoder<C, Ok = Self::Ok>;
    /// The type of a sequence encoder.
    type EncodeSequence: SequenceEncoder<C, Ok = Self::Ok>;
    /// The type of a tuple encoder.
    type EncodeTuple: SequenceEncoder<C, Ok = Self::Ok>;
    /// The type of a map encoder.
    type EncodeMap: MapEncoder<C, Ok = Self::Ok>;
    /// Streaming encoder for map pairs.
    type EncodeMapEntries: MapEntriesEncoder<C, Ok = Self::Ok>;
    /// Encoder that can encode a struct.
    type EncodeStruct: StructEncoder<C, Ok = Self::Ok>;
    /// Encoder for a struct variant.
    type EncodeVariant: VariantEncoder<C, Ok = Self::Ok>;
    /// Specialized encoder for a tuple variant.
    type EncodeTupleVariant: SequenceEncoder<C, Ok = Self::Ok>;
    /// Specialized encoder for a struct variant.
    type EncodeStructVariant: StructEncoder<C, Ok = Self::Ok>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::encoder]`][crate::encoder] attribute
    /// macro when implementing [`Encoder`].
    #[doc(hidden)]
    type __UseMusliEncoderAttributeMacro;

    /// Construct an encoder with a different context.
    fn with_context<U>(self, cx: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Err(cx.message(format_args!(
            "Context switch not supported, expected {}",
            ExpectingWrapper::new(self).format()
        )))
    }

    /// An expectation error. Every other implementation defers to this to
    /// report that something unexpected happened.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Encode a unit or something that is completely empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct EmptyStruct;
    ///
    /// impl<M> Encode<M> for EmptyStruct {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_unit(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit(self, cx: &C) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unit,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a boolean value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: bool,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_bool(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bool(self, cx: &C, _: bool) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Bool,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a character.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: char,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_char(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_char(self, cx: &C, _: char) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Char,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u8,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_u8(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u8(self, cx: &C, _: u8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u16,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_u16(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u16(self, cx: &C, _: u16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_u32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u32(self, cx: &C, _: u32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_u64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u64(self, cx: &C, _: u64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u128,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_u128(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u128(self, cx: &C, _: u128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unsigned128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i8,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_i8(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i8(self, cx: &C, _: i8) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i16,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_i16(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i16(self, cx: &C, _: i16) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_i32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i32(self, cx: &C, _: i32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_i64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i64(self, cx: &C, _: i64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i128,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_i128(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i128(self, cx: &C, _: i128) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Signed128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: usize,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_usize(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_usize(self, cx: &C, _: usize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Usize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`isize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: isize,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_isize(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_isize(self, cx: &C, _: isize) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Isize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: f32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_f32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f32(self, cx: &C, _: f32) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Float32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: f64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_f64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f64(self, cx: &C, _: f64) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Float64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode fixed-length array.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_array(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_array<const N: usize>(self, cx: &C, _: [u8; N]) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Array,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a sequence of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: Vec<u8>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_bytes(cx, self.data.as_slice())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes(self, cx: &C, _: &[u8]) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Bytes,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode the given slices of bytes in sequence, with one following another
    /// as a single contiguous byte array.
    ///
    /// This can be useful to avoid allocations when a caller doesn't have
    /// access to a single byte sequence like in
    /// [VecDeque][std::collections::VecDeque].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::VecDeque;
    ///
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: VecDeque<u8>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let (first, second) = self.data.as_slices();
    ///         encoder.encode_bytes_vectored(cx, &[first, second])
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes_vectored(self, cx: &C, _: &[&[u8]]) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Bytes,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: String,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         encoder.encode_string(cx, self.data.as_str())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_string(self, cx: &C, _: &str) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::String,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 data.encode(cx, encoder.encode_some(cx)?)
    ///             }
    ///             None => {
    ///                 encoder.encode_none(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_some(self, cx: &C) -> Result<Self::EncodeSome, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Option,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 data.encode(cx, encoder.encode_some(cx)?)
    ///             }
    ///             None => {
    ///                 encoder.encode_none(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_none(self, cx: &C) -> Result<Self::Ok, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Option,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Construct a pack that can encode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to encode all of the
    /// elements in the packed sequence as compact as possible and that
    /// subsequent unpackers will know the exact length of the element being
    /// unpacked.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut pack = encoder.encode_pack(cx)?;
    ///         self.field.encode(cx, pack.encode_next(cx)?)?;
    ///         self.data.encode(cx, pack.encode_next(cx)?)?;
    ///         pack.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack(self, cx: &C) -> Result<Self::EncodePack<'_>, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Pack,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a sequence with a known length `len`.
    ///
    /// A sequence encodes one element following another and must in some way
    /// encode the length of the sequence in the underlying format. It is
    /// decoded with [Decoder::decode_sequence].
    ///
    /// [Decoder::decode_sequence]: crate::de::Decoder::decode_sequence
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    ///
    /// struct MyType {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut seq = encoder.encode_sequence(cx, self.data.len())?;
    ///
    ///         for element in &self.data {
    ///             seq.push(cx, element)?;
    ///         }
    ///
    ///         seq.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence(
        self,
        cx: &C,
        #[allow(unused)] len: usize,
    ) -> Result<Self::EncodeSequence, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Sequence,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a tuple with a known length `len`.
    ///
    /// This is almost identical to [Encoder::encode_sequence] except that we
    /// know that we are encoding a fixed-length container of length `len`, and
    /// assuming the size of that container doesn't change in size it can be
    /// decoded using [Decoder::decode_tuple] again without the underlying
    /// format having to encode the size of the container.
    ///
    /// [Decoder::decode_tuple]: crate::de::Decoder::decode_tuple
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    ///
    /// struct PackedTuple(u32, [u8; 128]);
    ///
    /// impl<M> Encode<M> for PackedTuple {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut tuple = encoder.encode_tuple(cx, 2)?;
    ///         self.0.encode(cx, tuple.encode_next(cx)?)?;
    ///         self.1.encode(cx, tuple.encode_next(cx)?)?;
    ///         tuple.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_tuple(
        self,
        cx: &C,
        #[allow(unused)] len: usize,
    ) -> Result<Self::EncodeTuple, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Tuple,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a map with a known length `len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, MapEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut map = encoder.encode_map(cx, 2)?;
    ///         map.insert_entry(cx, "name", &self.name)?;
    ///         map.insert_entry(cx, "age", self.age)?;
    ///         map.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map(self, cx: &C, #[allow(unused)] len: usize) -> Result<Self::EncodeMap, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Map,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a map through pairs with a known length `len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, MapEntriesEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut m = encoder.encode_map_entries(cx, 2)?;
    ///
    ///         // Simplified encoding.
    ///         m.insert_entry(cx, "name", &self.name)?;
    ///
    ///         // Key and value encoding as a stream.
    ///         "age".encode(cx, m.encode_map_entry_key(cx)?)?;
    ///         self.age.encode(cx, m.encode_map_entry_value(cx)?)?;
    ///         m.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map_entries(
        self,
        cx: &C,
        #[allow(unused)] len: usize,
    ) -> Result<Self::EncodeMapEntries, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::MapPairs,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, StructEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut st = encoder.encode_struct(cx, 2)?;
    ///         st.insert_field(cx, "name", &self.name)?;
    ///         st.insert_field(cx, "age", self.age)?;
    ///         st.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct(self, cx: &C, _: usize) -> Result<Self::EncodeStruct, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Struct,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, VariantEncoder, StructEncoder};
    ///
    /// enum Enum {
    ///     UnitVariant,
    ///     TupleVariant(String),
    ///     Variant {
    ///         data: String,
    ///         age: u32,
    ///     }
    /// }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut variant = encoder.encode_variant(cx)?;
    ///
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 variant.insert_variant(cx, "variant1", ())
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 variant.insert_variant(cx, "variant2", data)
    ///             }
    ///             Enum::Variant { data, age } => {
    ///                 variant.encode_tag(cx)?.encode_string(cx, "variant3")?;
    ///
    ///                 let mut st = variant.encode_value(cx)?.encode_struct(cx, 2)?;
    ///                 st.insert_field(cx, "data", data)?;
    ///                 st.insert_field(cx, "age", age)?;
    ///                 st.end(cx)?;
    ///
    ///                 variant.end(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_variant(self, cx: &C) -> Result<Self::EncodeVariant, C::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Variant,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Simplified encoding for a unit variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, VariantEncoder, StructEncoder, SequenceEncoder};
    ///
    /// enum Enum {
    ///     UnitVariant,
    ///     TupleVariant(String),
    ///     Variant {
    ///         data: String,
    ///         age: u32,
    ///     }
    /// }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 encoder.encode_unit_variant(cx, &"variant1")
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 let mut variant = encoder.encode_tuple_variant(cx, &"variant2", 1)?;
    ///                 variant.push(cx, data)?;
    ///                 variant.end(cx)
    ///             }
    ///             Enum::Variant { data, age } => {
    ///                 let mut variant = encoder.encode_struct_variant(cx, &"variant3", 2)?;
    ///                 variant.insert_field(cx, "data", data)?;
    ///                 variant.insert_field(cx, "age", age)?;
    ///                 variant.end(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit_variant<T>(self, cx: &C, tag: &T) -> Result<Self::Ok, C::Error>
    where
        T: Encode<C::Mode>,
    {
        let mut variant = self.encode_variant(cx)?;
        let t = variant.encode_tag(cx)?;
        Encode::encode(tag, cx, t)?;
        let v = variant.encode_value(cx)?;
        v.encode_unit(cx)?;
        variant.end(cx)
    }

    /// Simplified encoding for a tuple variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, VariantEncoder, StructEncoder, SequenceEncoder};
    ///
    /// enum Enum {
    ///     UnitVariant,
    ///     TupleVariant(String),
    ///     Variant {
    ///         data: String,
    ///         age: u32,
    ///     }
    /// }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 let mut variant = encoder.encode_tuple_variant(cx, &"variant1", 0)?;
    ///                 variant.end(cx)
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 let mut variant = encoder.encode_tuple_variant(cx, &"variant2", 1)?;
    ///                 variant.push(cx, data)?;
    ///                 variant.end(cx)
    ///             }
    ///             Enum::Variant { data, age } => {
    ///                 let mut variant = encoder.encode_struct_variant(cx, &"variant3", 2)?;
    ///                 variant.insert_field(cx, "data", data)?;
    ///                 variant.insert_field(cx, "age", age)?;
    ///                 variant.end(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_tuple_variant<T>(
        self,
        cx: &C,
        _: &T,
        _: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::TupleVariant,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Simplified encoding for a struct variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, VariantEncoder, StructEncoder, SequenceEncoder};
    ///
    /// enum Enum {
    ///     UnitVariant,
    ///     TupleVariant(String),
    ///     Variant {
    ///         data: String,
    ///         age: u32,
    ///     }
    /// }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: ?Sized + Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 let mut variant = encoder.encode_tuple_variant(cx, &"variant1", 0)?;
    ///                 variant.end(cx)
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 let mut variant = encoder.encode_tuple_variant(cx, &"variant2", 1)?;
    ///                 variant.push(cx, data)?;
    ///                 variant.end(cx)
    ///             }
    ///             Enum::Variant { data, age } => {
    ///                 let mut variant = encoder.encode_struct_variant(cx, &"variant3", 2)?;
    ///                 variant.insert_field(cx, "data", data)?;
    ///                 variant.insert_field(cx, "age", age)?;
    ///                 variant.end(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct_variant<T>(
        self,
        cx: &C,
        _: &T,
        _: usize,
    ) -> Result<Self::EncodeStructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::StructVariant,
            &ExpectingWrapper::new(self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<'a, T, C: ?Sized> {
    inner: T,
    _marker: PhantomData<&'a C>,
}

impl<'a, T, C: ?Sized> ExpectingWrapper<'a, T, C> {
    #[inline]
    const fn new(inner: T) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, C> Expecting for ExpectingWrapper<'a, T, C>
where
    T: Encoder<C>,
    C: ?Sized + Context,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
