#![allow(unused_variables)]

use core::fmt;

use crate::de::{NumberVisitor, StructDecoder, TypeHint, ValueVisitor, Visitor};
use crate::expecting::{self, Expecting};
use crate::Context;

use super::{AsDecoder, MapDecoder, PackDecoder, SequenceDecoder, VariantDecoder};

/// Trait governing the implementation of a decoder.
pub trait Decoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: ?Sized + Context<Error = Self::Error, Mode = Self::Mode>;
    /// Error associated with decoding.
    type Error;
    /// Mode associated with decoding.
    type Mode;
    /// [`Decoder`] with a different context returned by
    /// [`Decoder::with_context`]
    type WithContext<U>: Decoder<'de, Cx = U, Error = U::Error, Mode = U::Mode>
    where
        U: Context;
    /// Decoder returned by [`Decoder::decode_buffer`].
    type DecodeBuffer: AsDecoder<Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_option`].
    type DecodeSome: Decoder<'de, Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// Decoder returned by [`Decoder::decode_pack`].
    type DecodePack: PackDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_sequence`].
    type DecodeSequence: SequenceDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_tuple`].
    type DecodeTuple: PackDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_map`].
    type DecodeMap: MapDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_struct`].
    type DecodeStruct: StructDecoder<'de, Cx = Self::Cx>;
    /// Decoder returned by [`Decoder::decode_variant`].
    type DecodeVariant: VariantDecoder<'de, Cx = Self::Cx>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::decoder]`][crate::decoder] attribute
    /// macro when implementing [`Decoder`].
    #[doc(hidden)]
    type __UseMusliDecoderAttributeMacro;

    /// Construct an decoder with a different context.
    fn with_context<U>(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::WithContext<U>, <Self::Cx as Context>::Error>
    where
        U: Context,
    {
        Err(cx.message(format_args!(
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
    /// use std::marker::PhantomData;
    ///
    /// use musli::Context;
    /// use musli::de::{self, Decoder};
    ///
    /// struct MyDecoder<C: ?Sized> {
    ///     _marker: PhantomData<C>,
    /// }
    ///
    /// #[musli::decoder]
    /// impl<C: ?Sized + Context> Decoder<'_> for MyDecoder<C> {
    ///     type Cx = C;
    ///
    ///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "32-bit unsigned integers")
    ///     }
    ///
    ///     fn decode_u32(self, _: &C) -> Result<u32, <Self::Cx as Context>::Error> {
    ///         Ok(42)
    ///     }
    /// }
    /// ```
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Skip over the current next value.
    fn skip(self, cx: &Self::Cx) -> Result<(), <Self::Cx as Context>::Error> {
        Err(cx.message(format_args!(
            "Skipping is not supported, expected {}",
            ExpectingWrapper::new(&self).format()
        )))
    }

    /// Return a [TypeHint] indicating which type is being produced by the
    /// [Decoder].
    ///
    /// Not all formats support type hints, and they might be ranging from
    /// detailed (`a 32-bit unsigned integer`) to vague (`a number`).
    ///
    /// This is used to construct dynamic containers of types.
    fn type_hint(&mut self, cx: &Self::Cx) -> Result<TypeHint, <Self::Cx as Context>::Error> {
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
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut buffer = decoder.decode_buffer(cx)?;
    ///
    ///         let mut st = buffer.as_decoder(cx)?.decode_map(cx)?;
    ///
    ///         let discriminant = loop {
    ///             let Some(mut e) = st.decode_entry(cx)? else {
    ///                 return Err(cx.missing_variant_tag("MyVariantType"));
    ///             };
    ///
    ///             let found = e.decode_map_key(cx)?.decode_string(cx, musli::utils::visit_owned_fn("a string that is 'type'", |cx: &D::Cx, string: &str| {
    ///                 Ok(string == "type")
    ///             }))?;
    ///
    ///             if found {
    ///                 break cx.decode(e.decode_map_value(cx)?)?;
    ///             }
    ///         };
    ///
    ///         st.end(cx)?;
    ///
    ///         match discriminant {
    ///             0 => Ok(MyVariantType::Variant1),
    ///             1 => Ok(MyVariantType::Variant2(cx.decode(buffer.as_decoder(cx)?)?)),
    ///             other => Err(cx.invalid_variant_tag("MyVariantType", other)),
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_buffer(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeBuffer, <Self::Cx as Context>::Error> {
        Err(cx.message(format_args!(
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
    ///         decoder.decode_unit(cx)?;
    ///         Ok(UnitType)
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_unit(self, cx: &Self::Cx) -> Result<(), <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             field: decoder.decode_bool(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bool(self, cx: &Self::Cx) -> Result<bool, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_char(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_char(self, cx: &Self::Cx) -> Result<char, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_u8(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u8(self, cx: &Self::Cx) -> Result<u8, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_u16(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u16(self, cx: &Self::Cx) -> Result<u16, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_u32(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u32(self, cx: &Self::Cx) -> Result<u32, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_u64(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u64(self, cx: &Self::Cx) -> Result<u64, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_u128(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_u128(self, cx: &Self::Cx) -> Result<u128, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_i8(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i8(self, cx: &Self::Cx) -> Result<i8, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_i16(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i16(self, cx: &Self::Cx) -> Result<i16, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_i32(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i32(self, cx: &Self::Cx) -> Result<i32, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_i64(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i64(self, cx: &Self::Cx) -> Result<i64, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_i128(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_i128(self, cx: &Self::Cx) -> Result<i128, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_usize(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_usize(self, cx: &Self::Cx) -> Result<usize, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_isize(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_isize(self, cx: &Self::Cx) -> Result<isize, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_f32(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f32(self, cx: &Self::Cx) -> Result<f32, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_f64(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_f64(self, cx: &Self::Cx) -> Result<f64, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Float64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode an unknown number using a visitor that can handle arbitrary
    /// precision numbers.
    #[inline]
    fn decode_number<V>(
        self,
        cx: &Self::Cx,
        visitor: V,
    ) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: NumberVisitor<'de, Self::Cx>,
    {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_array(cx)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_array<const N: usize>(
        self,
        cx: &Self::Cx,
    ) -> Result<[u8; N], <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///             data: decoder.decode_bytes(cx, Visitor)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bytes<V>(
        self,
        cx: &Self::Cx,
        visitor: V,
    ) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: ValueVisitor<'de, Self::Cx, [u8]>,
    {
        Err(cx.message(expecting::unsupported_type(
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
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, ValueVisitor};
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
    ///             data: decoder.decode_string(cx, Visitor)?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_string<V>(
        self,
        cx: &Self::Cx,
        visitor: V,
    ) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: ValueVisitor<'de, Self::Cx, str>,
    {
        Err(cx.message(expecting::unsupported_type(
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
    ///         let data = if let Some(decoder) = decoder.decode_option(cx)? {
    ///             Some(cx.decode(decoder)?)
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
    fn decode_option(
        self,
        cx: &Self::Cx,
    ) -> Result<Option<Self::DecodeSome>, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, PackDecoder};
    /// # struct PackedStruct { field: u32, data: [u8; 128] }
    ///
    /// impl<'de, M> Decode<'de, M> for PackedStruct {
    ///     #[inline]
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut pack = decoder.decode_pack(cx)?;
    ///         let field = pack.next(cx)?;
    ///         let data = pack.next(cx)?;
    ///         pack.end(cx)?;
    ///
    ///         Ok(Self { field, data })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_pack(self, cx: &Self::Cx) -> Result<Self::DecodePack, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Pack,
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
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, PackDecoder};
    /// # struct PackedStruct { field: u32, data: [u8; 128] }
    ///
    /// impl<'de, M> Decode<'de, M> for PackedStruct {
    ///     #[inline]
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_pack_fn(cx, |pack| Ok(Self {
    ///             field: pack.next(cx)?,
    ///             data: pack.next(cx)?,
    ///         }))
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_pack_fn<F, O>(self, cx: &Self::Cx, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, <Self::Cx as Context>::Error>,
    {
        let mut pack = self.decode_pack(cx)?;
        let result = f(&mut pack)?;
        pack.end(cx)?;
        Ok(result)
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
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, SequenceDecoder};
    /// # struct VectorField { data: Vec<String> }
    ///
    /// impl<'de, M> Decode<'de, M> for VectorField {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut seq = decoder.decode_sequence(cx)?;
    ///         let mut data = Vec::new();
    ///
    ///         while let Some(decoder) = seq.decode_next(cx)? {
    ///             data.push(cx.decode(decoder)?);
    ///         }
    ///
    ///         seq.end(cx)?;
    ///
    ///         Ok(Self { data })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_sequence(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeSequence, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Sequence,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a sequence using a closure which is easier to get right.
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
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, SequenceDecoder};
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
    ///         decoder.decode_sequence_fn(cx, |seq| {
    ///             let mut data = Vec::new();
    ///
    ///             while let Some(decoder) = seq.decode_next(cx)? {
    ///                 data.push(cx.decode(decoder)?);
    ///             }
    ///
    ///             Ok(Self { data })
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_sequence_fn<F, O>(
        self,
        cx: &Self::Cx,
        f: F,
    ) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, <Self::Cx as Context>::Error>,
    {
        let mut sequence = self.decode_sequence(cx)?;
        let result = f(&mut sequence)?;
        sequence.end(cx)?;
        Ok(result)
    }

    /// Decode a fixed-length sequence of elements of length `len`.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
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
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, PackDecoder};
    /// # struct TupleStruct(String, u32);
    ///
    /// impl<'de, M> Decode<'de, M> for TupleStruct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut tuple = decoder.decode_tuple(cx, 2)?;
    ///         let string = cx.decode(tuple.decode_next(cx)?)?;
    ///         let integer = cx.decode(tuple.decode_next(cx)?)?;
    ///         tuple.end(cx)?;
    ///         Ok(Self(string, integer))
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_tuple(
        self,
        cx: &Self::Cx,
        #[allow(unused)] len: usize,
    ) -> Result<Self::DecodeTuple, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Tuple,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a fixed-length sequence of elements of length `len`.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
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
    /// use musli::Context;
    /// use musli::de::{Decode, Decoder, PackDecoder};
    /// # struct TupleStruct(String, u32);
    ///
    /// impl<'de, M> Decode<'de, M> for TupleStruct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_tuple_fn(cx, 2, |tuple| {
    ///             Ok(Self(tuple.next(cx)?, tuple.next(cx)?))
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_tuple_fn<F, O>(
        self,
        cx: &Self::Cx,
        len: usize,
        f: F,
    ) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeTuple) -> Result<O, <Self::Cx as Context>::Error>,
    {
        let mut tuple = self.decode_tuple(cx, len)?;
        let result = f(&mut tuple)?;
        tuple.end(cx)?;
        Ok(result)
    }

    /// Decode a map.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::{Context, Decode};
    ///
    /// #[derive(Decode)]
    /// struct MapStruct {
    ///     data: HashMap<String, u32>,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Context, Decode, Decoder};
    /// use musli::de::{MapDecoder, MapEntryDecoder};
    /// # use std::collections::HashMap;
    /// # struct MapStruct { data: HashMap<String, u32> }
    ///
    /// impl<'de, M> Decode<'de, M> for MapStruct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut map = decoder.decode_map(cx)?;
    ///         let mut data = HashMap::with_capacity(map.size_hint(cx).or_default());
    ///
    ///         while let Some(mut entry) = map.decode_entry(cx)? {
    ///             let key = cx.decode(entry.decode_map_key(cx)?)?;
    ///             let value = cx.decode(entry.decode_map_value(cx)?)?;
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
    fn decode_map(self, cx: &Self::Cx) -> Result<Self::DecodeMap, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Map,
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
    /// Deriving an implementation:
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::Decode;
    ///
    /// #[derive(Decode)]
    /// #[musli(packed)]
    /// struct MapStruct {
    ///     data: HashMap<String, u32>,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::{Decode, Decoder};
    /// use musli::de::{MapDecoder, MapEntryDecoder};
    /// # struct MapStruct { data: HashMap<String, u32> }
    ///
    /// impl<'de, M> Decode<'de, M> for MapStruct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_map_fn(cx, |map| {
    ///             let mut data = HashMap::with_capacity(map.size_hint(cx).or_default());
    ///
    ///             while let Some((key, value)) = map.entry(cx)? {
    ///                 data.insert(key, value);
    ///             }
    ///
    ///             Ok(Self { data })
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_map_fn<F, O>(self, cx: &Self::Cx, f: F) -> Result<O, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, <Self::Cx as Context>::Error>,
    {
        let mut map = self.decode_map(cx)?;
        let result = f(&mut map)?;
        map.end(cx)?;
        Ok(result)
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
    /// Deriving an implementation:
    ///
    /// ```
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
    /// use musli::de::{StructDecoder, StructFieldDecoder};
    /// # struct Struct { string: String, integer: u32 }
    ///
    /// impl<'de, M> Decode<'de, M> for Struct {
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut st = decoder.decode_struct(cx, None)?;
    ///         let mut string = None;
    ///         let mut integer = None;
    ///
    ///         while let Some(mut field) = st.decode_field(cx)? {
    ///             // Note: to avoid allocating `decode_string` needs to be used with a visitor.
    ///             let tag: String = cx.decode(field.decode_field_name(cx)?)?;
    ///
    ///             match tag.as_str() {
    ///                 "string" => {
    ///                     string = Some(cx.decode(field.decode_field_value(cx)?)?);
    ///                 }
    ///                 "integer" => {
    ///                     integer = Some(cx.decode(field.decode_field_value(cx)?)?);
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
    fn decode_struct(
        self,
        cx: &Self::Cx,
        fields: Option<usize>,
    ) -> Result<Self::DecodeStruct, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Struct,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Return a decoder for a variant.
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
    ///     fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut variant = decoder.decode_variant(cx)?;
    ///         let tag = cx.decode(variant.decode_tag(cx)?)?;
    ///
    ///         let this = match tag {
    ///             0 => {
    ///                 Self::Variant1(cx.decode(variant.decode_value(cx)?)?)
    ///             }
    ///             1 => {
    ///                 Self::Variant2(cx.decode(variant.decode_value(cx)?)?)
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
    fn decode_variant(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::DecodeVariant, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Variant,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode dynamically through a [`Visitor`].
    ///
    /// If the current encoding does not support dynamic decoding,
    /// [`Visitor::visit_any`] will be called with the current decoder.
    #[inline]
    fn decode_any<V>(self, cx: &Self::Cx, visitor: V) -> Result<V::Ok, <Self::Cx as Context>::Error>
    where
        V: Visitor<'de, Self::Cx>,
    {
        Err(cx.message(format_args!(
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
