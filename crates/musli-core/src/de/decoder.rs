#![allow(unused_variables)]

use core::fmt;

use crate::expecting::{self, Expecting};
use crate::hint::{MapHint, SequenceHint};
use crate::Context;

use super::{
    AsDecoder, Decode, DecodeUnsized, DecodeUnsizedBytes, EntriesDecoder, MapDecoder,
    NumberVisitor, SequenceDecoder, Skip, ValueVisitor, VariantDecoder, Visitor,
};

/// Trait governing the implementation of a decoder.
#[must_use = "Decoders must be consumed through one of its decode_* methods"]
pub trait Decoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context<Error = Self::Error, Mode = Self::Mode>;
    /// Error associated with decoding.
    type Error;
    /// Mode associated with decoding.
    type Mode;
    /// [`Decoder`] with a different context returned by
    /// [`Decoder::with_context`]
    type WithContext<'this, U>: Decoder<'de, Cx = U, Error = U::Error, Mode = U::Mode>
    where
        U: 'this + Context;
    /// Decoder returned by [`Decoder::decode_buffer`].
    type DecodeBuffer: AsDecoder<Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_option`].
    type DecodeSome: Decoder<'de, Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// Decoder used by [`Decoder::decode_pack`].
    type DecodePack: SequenceDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_sequence`].
    type DecodeSequence: SequenceDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_sequence_hint`].
    type DecodeSequenceHint: SequenceDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_map`].
    type DecodeMap: MapDecoder<'de, Cx = Self::Cx>;
    /// Decoder used by [`Decoder::decode_map_hint`].
    type DecodeMapHint: MapDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_map_entries`].
    type DecodeMapEntries: EntriesDecoder<'de, Cx = Self::Cx>;
    /// Decoder used by [`Decoder::decode_variant`].
    type DecodeVariant: VariantDecoder<'de, Cx = Self::Cx>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::decoder]`][musli::decoder] attribute
    /// macro when implementing [`Decoder`].
    #[doc(hidden)]
    type __UseMusliDecoderAttributeMacro;

    /// Return the context associated with the decoder.
    fn cx(&self) -> &Self::Cx;

    /// Construct an decoder with a different context.
    fn with_context<U>(
        self,
        cx: &U,
    ) -> Result<Self::WithContext<'_, U>, <Self::Cx as Context>::Error>
    where
        U: Context,
    {
        Err(self.cx().message(format_args!(
            "Context switch not supported, expected {}",
            ExpectingWrapper::new(&self).format()
        )))
    }

    /// Format the human-readable message that should occur if the decoder was
    /// expecting to decode some specific kind of value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fmt;
    /// use std::convert::Infallible;
    ///
    /// use musli::Context;
    /// use musli::de::{self, Decoder, Decode};
    ///
    /// struct MyDecoder<'a, C: ?Sized> {
    ///     cx: &'a C,
    /// }
    ///
    /// #[musli::decoder]
    /// impl<'de, C: ?Sized + Context> Decoder<'de> for MyDecoder<'_, C> {
    ///     type Cx = C;
    ///
    ///     fn cx(&self) -> &C {
    ///         self.cx
    ///     }
    ///
    ///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "32-bit unsigned integers")
    ///     }
    ///
    ///     fn decode_u32(self) -> Result<u32, <Self::Cx as Context>::Error> {
    ///         Ok(42)
    ///     }
    /// }
    /// ```
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Decode the current decoder into the value `T`.
    ///
    /// This calls the appropriate [`Decode`] implementation for the given type.
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode>;

    /// Decode an unsized value by reference through the specified closure.
    fn decode_unsized<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsized<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, <Self::Cx as Context>::Error>;

    /// Decode an unsized value as bytes by reference through the specified
    /// closure.
    fn decode_unsized_bytes<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsizedBytes<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, <Self::Cx as Context>::Error>;

    /// Skip over the current next value.
    #[inline]
    fn skip(self) -> Result<(), <Self::Cx as Context>::Error> {
        Err(self.cx().message(format_args!(
            "Skipping is not supported, expected {}",
            ExpectingWrapper::new(&self).format()
        )))
    }

    /// This is a variant of [`Decoder::skip`], but instead of erroring in case
    /// skipping is not supported it must return [`Skip::Unsupported`].
    #[inline(always)]
    fn try_skip(self) -> Result<Skip, <Self::Cx as Context>::Error> {
        Ok(Skip::Unsupported)
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
    /// use musli::de::{AsDecoder, MapDecoder, EntryDecoder};
    /// use musli::mode::Binary;
    ///
    /// #[derive(Decode)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// enum Enum {
    ///     Empty,
    ///     Person(Person),
    /// }
    ///
    /// impl<'de> Decode<'de, Binary> for Enum {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de, Mode = Binary>,
    ///     {
    ///         let mut buffer = decoder.decode_buffer()?;
    ///
    ///         let discriminant = buffer.as_decoder()?.decode_map(|st| {
    ///             loop {
    ///                 let Some(mut e) = st.decode_entry()? else {
    ///                     return Err(cx.missing_variant_tag("Enum"));
    ///                 };
    ///
    ///                 let found = e.decode_key()?.decode_unsized(|string: &str| {
    ///                     Ok(string == "type")
    ///                 })?;
    ///
    ///                 if found {
    ///                     break Ok(e.decode_value()?.decode()?);
    ///                 }
    ///             }
    ///         })?;
    ///
    ///         match discriminant {
    ///             0 => Ok(Enum::Empty),
    ///             1 => Ok(Enum::Person(buffer.as_decoder()?.decode()?)),
    ///             other => Err(cx.invalid_variant_tag("Enum", &other)),
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, <Self::Cx as Context>::Error> {
        Err(self.cx().message(format_args!(
            "Decode buffering not supported, expected {}",
            ExpectingWrapper::new(&self).format()
        )))
    }

    /// Decode a unit.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// struct UnitStruct;
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct UnitType;
    ///
    /// impl<'de, M> Decode<'de, M> for UnitType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_unit()?;
    ///         Ok(UnitType)
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_unit(self) -> Result<(), <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unit,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a boolean.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct BooleanField {
    ///     field: bool,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct BooleanField { field: bool }
    ///
    /// impl<'de, M> Decode<'de, M> for BooleanField {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             field: decoder.decode_bool()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bool(self) -> Result<bool, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Bool,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a character.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: char,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: char }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_char()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_char(self) -> Result<char, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Char,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 8-bit unsigned integer (a.k.a. a byte).
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u8,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: u8 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u8()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u8(self) -> Result<u8, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u16,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: u16 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u16()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u16(self) -> Result<u16, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u32,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: u32 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u32()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u32(self) -> Result<u32, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u64,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: u64 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u64()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u64(self) -> Result<u64, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u128,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: u128 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_u128()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u128(self) -> Result<u128, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i8,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: i8 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i8()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i8(self) -> Result<i8, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i16,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: i16 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i16()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i16(self) -> Result<i16, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i32,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: i32 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i32()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i32(self) -> Result<i32, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i64,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: i64 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i64()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i64(self) -> Result<i64, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i128,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: i128 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_i128()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i128(self) -> Result<i128, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a [`usize`].
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: usize,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: usize }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_usize()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_usize(self) -> Result<usize, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Usize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a [`isize`].
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: isize,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: isize }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_isize()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_isize(self) -> Result<isize, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Isize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: f32,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: f32 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_f32()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f32(self) -> Result<f32, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Float32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: f64,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: f64 }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_f64()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f64(self) -> Result<f64, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Float64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode an unknown number using a visitor that can handle arbitrary
    /// precision numbers.
    #[inline]
    fn decode_number<V>(self, visitor: V) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: NumberVisitor<'de, Self::Cx>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Number,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a fixed-length array.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: [u8; 128],
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Decode, Decoder};
    /// # struct MyType { data: [u8; 128] }
    ///
    /// impl<'de, M> Decode<'de, M> for MyType {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_array()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_array<const N: usize>(self) -> Result<[u8; N], <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Array,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a sequence of bytes whos length is encoded in the payload.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct BytesReference<'de> {
    ///     data: &'de [u8],
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use std::fmt;
    ///
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::ValueVisitor;
    /// # struct BytesReference<'de> { data: &'de [u8] }
    ///
    /// impl<'de, M> Decode<'de, M> for BytesReference<'de> {
    ///     #[inline]
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor;
    ///
    ///         impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
    ///         where
    ///             C: ?Sized + Context,
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
    ///             data: decoder.decode_bytes(Visitor)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: ValueVisitor<'de, Self::Cx, [u8]>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Bytes,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a string slice from the current decoder.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct StringReference<'de> {
    ///     data: &'de str,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use std::fmt;
    ///
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::ValueVisitor;
    /// # struct StringReference<'de> { data: &'de str }
    ///
    /// impl<'de, M> Decode<'de, M> for StringReference<'de> {
    ///     #[inline]
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor;
    ///
    ///         impl<'de, C> ValueVisitor<'de, C, str> for Visitor
    ///         where
    ///             C: ?Sized + Context,
    ///         {
    ///             type Ok = &'de str;
    ///
    ///             #[inline]
    ///             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///                 write!(f, "exact bytes reference")
    ///             }
    ///
    ///             #[inline]
    ///             fn visit_borrowed(self, _: &C, bytes: &'de str) -> Result<Self::Ok, C::Error> {
    ///                 Ok(bytes)
    ///             }
    ///         }
    ///
    ///         Ok(Self {
    ///             data: decoder.decode_string(Visitor)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: ValueVisitor<'de, Self::Cx, str>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::String,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode an optional value.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::{Context, Decode};
    ///
    /// #[derive(Decode)]
    /// struct OptionalField {
    ///     data: Option<String>,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// # struct OptionalField { data: Option<String>}
    ///
    /// impl<'de, M> Decode<'de, M> for OptionalField {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let data = if let Some(decoder) = decoder.decode_option()? {
    ///             Some(decoder.decode()?)
    ///         } else {
    ///             None
    ///         };
    ///
    ///         Ok(Self { data })
    ///     }
    /// }
    /// ```
    #[inline]
    #[must_use = "Decoders must be consumed"]
    fn decode_option(self) -> Result<Option<Self::DecodeSome>, <Self::Cx as Context>::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Option,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Construct an unpack that can decode more than one element at a time.
    ///
    /// This hints to the format that it should attempt to decode all of the
    /// elements in the packed sequence from an as compact format as possible
    /// compatible with what's being returned by
    /// [Encoder::pack][crate::Encoder::encode_pack].
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 128],
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    /// # struct PackedStruct { field: u32, data: [u8; 128] }
    ///
    /// impl<'de, M> Decode<'de, M> for PackedStruct {
    ///     #[inline]
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_pack(|pack| Ok(Self {
    ///             field: pack.next()?,
    ///             data: pack.next()?,
    ///         }))
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_pack<F, O>(self, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, <Self::Cx as Context>::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Pack,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a sequence.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// struct VectorField {
    ///     data: Vec<String>,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    ///
    /// struct VectorField {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for VectorField {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_sequence(|seq| {
    ///             let mut data = Vec::new();
    ///
    ///             while let Some(decoder) = seq.try_decode_next()? {
    ///                 data.push(decoder.decode()?);
    ///             }
    ///
    ///             Ok(Self { data })
    ///         })
    ///     }
    /// }
    /// ```
    ///
    /// Deriving an implementation for a tuple:
    ///
    /// ```
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// struct TupleStruct(String, u32);
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    /// use musli::hint::SequenceHint;
    /// # struct TupleStruct(String, u32);
    ///
    /// impl<'de, M> Decode<'de, M> for TupleStruct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         static HINT: SequenceHint = SequenceHint::with_size(2);
    ///
    ///         decoder.decode_sequence_hint(&HINT, |tuple| {
    ///             Ok(Self(tuple.next()?, tuple.next()?))
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_sequence<F, O>(self, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, <Self::Cx as Context>::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Sequence,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a sequence with a `hint` indicating its expected characteristics.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    /// use musli::hint::SequenceHint;
    /// # struct TupleStruct(String, u32);
    ///
    /// impl<'de, M> Decode<'de, M> for TupleStruct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         static HINT: SequenceHint = SequenceHint::with_size(2);
    ///
    ///         decoder.decode_sequence_hint(&HINT, |tuple| {
    ///             Ok(Self(tuple.next()?, tuple.next()?))
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_sequence_hint<F, O>(
        self,
        hint: &SequenceHint,
        f: F,
    ) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeSequenceHint) -> Result<O, <Self::Cx as Context>::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Sequence,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a map who's size is not known at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::{Decode, Decoder};
    /// use musli::de::{MapDecoder, EntryDecoder};
    /// # struct MapStruct { data: HashMap<String, u32> }
    ///
    /// impl<'de, M> Decode<'de, M> for MapStruct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_map(|map| {
    ///             let mut data = HashMap::with_capacity(map.size_hint().or_default());
    ///
    ///             while let Some((key, value)) = map.entry()? {
    ///                 data.insert(key, value);
    ///             }
    ///
    ///             Ok(Self { data })
    ///         })
    ///     }
    /// }
    /// ```
    fn decode_map<F, O>(self, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, <Self::Cx as Context>::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::UnsizedStruct,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a map using a simplified function.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    ///
    /// # Examples
    ///
    /// Deriving an implementation from a struct:
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// struct Struct {
    ///     string: String,
    ///     integer: u32,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::{MapDecoder, EntryDecoder};
    /// use musli::hint::MapHint;
    ///
    /// struct Struct {
    ///     string: String,
    ///     integer: u32,
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for Struct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         static HINT: MapHint = MapHint::with_size(2);
    ///
    ///         decoder.decode_map_hint(&HINT, |st| {
    ///             let mut string = None;
    ///             let mut integer = None;
    ///
    ///             while let Some(mut field) = st.decode_entry()? {
    ///                 // Note: to avoid allocating `decode_string` needs to be used with a visitor.
    ///                 let tag = field.decode_key()?.decode::<String>()?;
    ///
    ///                 match tag.as_str() {
    ///                     "string" => {
    ///                         string = Some(field.decode_value()?.decode()?);
    ///                     }
    ///                     "integer" => {
    ///                         integer = Some(field.decode_value()?.decode()?);
    ///                     }
    ///                     tag => {
    ///                         return Err(cx.invalid_field_tag("Struct", tag));
    ///                     }
    ///                 }
    ///             }
    ///
    ///             Ok(Self {
    ///                 string: string.ok_or_else(|| cx.expected_tag("Struct", "string"))?,
    ///                 integer: integer.ok_or_else(|| cx.expected_tag("Struct", "integer"))?,
    ///             })
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_map_hint<F, O>(self, hint: &MapHint, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeMapHint) -> Result<O, <Self::Cx as Context>::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Map,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Simplified decoding a map of unknown length.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    #[inline]
    fn decode_map_entries(self) -> Result<Self::DecodeMapEntries, <Self::Cx as Context>::Error>
    where
        Self: Sized,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::MapEntries,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a variant using a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Decode};
    /// use musli::de::{Decoder, VariantDecoder};
    ///
    /// enum Enum {
    ///     Number(u32),
    ///     String(String),
    /// }
    ///
    /// impl<'de, M> Decode<'de, M> for Enum {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_variant(|variant| {
    ///             let tag = variant.decode_tag()?.decode()?;
    ///             let value = variant.decode_value()?;
    ///
    ///             match tag {
    ///                 0 => Ok(Self::Number(value.decode()?)),
    ///                 1 => Ok(Self::String(value.decode()?)),
    ///                 tag => Err(cx.invalid_variant_tag("Enum", &tag)),
    ///             }
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_variant<F, O>(self, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, <Self::Cx as Context>::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Variant,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode dynamically through a [`Visitor`].
    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: Visitor<'de, Self::Cx>,
    {
        Err(self.cx().message(format_args!(
            "Any type not supported, expected {}",
            ExpectingWrapper::new(&self).format()
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T> {
    inner: T,
}

impl<T> ExpectingWrapper<T> {
    fn new(inner: &T) -> &Self {
        // Safety: `ExpectingWrapper` is repr(transparent) over `T`.
        unsafe { &*(inner as *const T as *const Self) }
    }
}

impl<'de, T> Expecting for ExpectingWrapper<T>
where
    T: Decoder<'de>,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
