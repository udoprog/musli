use core::fmt;

use crate::expecting::{self, Expecting};
use crate::hint::{MapHint, SequenceHint};
use crate::{Allocator, Context};

use super::{
    utils, AsDecoder, Decode, DecodeSliceBuilder, DecodeUnsized, DecodeUnsizedBytes,
    EntriesDecoder, MapDecoder, SequenceDecoder, Skip, UnsizedVisitor, VariantDecoder, Visitor,
};

/// An outcome of a fast decode attempt.
#[non_exhaustive]
pub enum TryFastDecode<T, D> {
    /// The decode attempt was successful.
    Ok(T),
    /// The decode was unsupported.
    Unsupported(D),
}

/// Trait governing the implementation of a decoder.
#[must_use = "Decoders must be consumed through one of its decode_* methods"]
#[allow(unused_variables)]
pub trait Decoder<'de>: Sized {
    /// Context associated with the decoder.
    type Cx: Context<Error = Self::Error, Allocator = Self::Allocator>;
    /// Error associated with decoding.
    type Error;
    /// The allocator associated with the decoder.
    type Allocator: Allocator;
    /// Mode associated with decoding.
    type Mode: 'static;
    /// Decoder returned by [`Decoder::decode_buffer`].
    type DecodeBuffer: AsDecoder<
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >;
    /// Decoder returned by [`Decoder::decode_option`].
    type DecodeSome: Decoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >;
    /// Decoder used by [`Decoder::decode_pack`].
    type DecodePack: SequenceDecoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >;
    /// Decoder returned by [`Decoder::decode_sequence`].
    type DecodeSequence: SequenceDecoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >;
    /// Decoder returned by [`Decoder::decode_map`].
    type DecodeMap: MapDecoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >;
    /// Decoder returned by [`Decoder::decode_map_entries`].
    type DecodeMapEntries: EntriesDecoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >;
    /// Decoder used by [`Decoder::decode_variant`].
    type DecodeVariant: VariantDecoder<
        'de,
        Cx = Self::Cx,
        Error = Self::Error,
        Allocator = Self::Allocator,
        Mode = Self::Mode,
    >;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::de::decoder]`][musli::de::decoder]
    /// attribute macro when implementing [`Decoder`].
    #[doc(hidden)]
    type __UseMusliDecoderAttributeMacro;

    /// Access the context associated with the decoder.
    fn cx(&self) -> Self::Cx;

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
    /// use musli::de::{self, Decoder, Decode};
    ///
    /// struct MyDecoder<C, M> {
    ///     cx: C,
    ///     _marker: PhantomData<M>,
    /// }
    ///
    /// #[musli::de::decoder]
    /// impl<'de, C, M> Decoder<'de> for MyDecoder<C, M>
    /// where
    ///     C: Context,
    ///     M: 'static,
    /// {
    ///     type Cx = C;
    ///
    ///     #[inline]
    ///     fn cx(&self) -> Self::Cx {
    ///         self.cx
    ///     }
    ///
    ///     #[inline]
    ///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "32-bit unsigned integers")
    ///     }
    ///
    ///     #[inline]
    ///     fn decode_u32(self) -> Result<u32, Self::Error> {
    ///         Ok(42)
    ///     }
    /// }
    /// ```
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Try to quickly decode the specified value.
    ///
    /// The default implementation simply returns the current decoder as
    /// `Err(Self)`.
    ///
    /// This is intended to be a fast path when decoding a value when an
    /// encoding permits it.
    #[inline]
    fn try_fast_decode<T>(self) -> Result<TryFastDecode<T, Self>, Self::Error>
    where
        T: Decode<'de, Self::Mode, Self::Allocator>,
    {
        Ok(TryFastDecode::Unsupported(self))
    }

    /// Decode the current decoder into the value `T`.
    ///
    /// This calls the appropriate [`Decode`] implementation for the given type.
    #[inline]
    fn decode<T>(self) -> Result<T, Self::Error>
    where
        T: Decode<'de, Self::Mode, Self::Allocator>,
    {
        match self.try_fast_decode::<T>()? {
            TryFastDecode::Ok(value) => Ok(value),
            TryFastDecode::Unsupported(decoder) => T::decode(decoder),
        }
    }

    /// Decode an unsized value by reference through the specified closure.
    #[inline]
    fn decode_unsized<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsized<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        T::decode_unsized(self, f)
    }

    /// Decode an unsized value as bytes by reference through the specified
    /// closure.
    #[inline]
    fn decode_unsized_bytes<T, F, O>(self, f: F) -> Result<O, Self::Error>
    where
        T: ?Sized + DecodeUnsizedBytes<'de, Self::Mode>,
        F: FnOnce(&T) -> Result<O, Self::Error>,
    {
        T::decode_unsized_bytes(self, f)
    }

    /// Skip over the current next value.
    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        Err(self.cx().message(format_args!(
            "Skipping is not supported, expected {}",
            ExpectingWrapper::new(&self).format()
        )))
    }

    /// This is a variant of [`Decoder::skip`], but instead of erroring in case
    /// skipping is not supported it must return [`Skip::Unsupported`].
    #[inline(always)]
    fn try_skip(self) -> Result<Skip, Self::Error> {
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
    /// use musli::{Allocator, Context, Decode, Decoder};
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
    /// impl<'de, A> Decode<'de, Binary, A> for Enum
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de, Mode = Binary>,
    ///     {
    ///         let cx = decoder.cx();
    ///
    ///         let mut buffer = decoder.decode_buffer()?;
    ///
    ///         let discriminant = buffer.as_decoder()?.decode_map(|st| {
    ///             loop {
    ///                 let Some(mut e) = st.decode_entry()? else {
    ///                     return Err(cx.message("Missing variant tag"));
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
    ///             other => Err(cx.message(format_args!("Invalid variant tag {other:?}"))),
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_buffer(self) -> Result<Self::DecodeBuffer, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct UnitType;
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for UnitType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_empty()?;
    ///         Ok(UnitType)
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_empty(self) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Empty,
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct BooleanField { field: bool }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for BooleanField
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_bool(self) -> Result<bool, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: char }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_char(self) -> Result<char, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: u8 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_u8(self) -> Result<u8, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: u16 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_u16(self) -> Result<u16, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: u32 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_u32(self) -> Result<u32, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: u64 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_u64(self) -> Result<u64, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: u128 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_u128(self) -> Result<u128, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: i8 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_i8(self) -> Result<i8, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: i16 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_i16(self) -> Result<i16, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: i32 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_i32(self) -> Result<i32, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: i64 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_i64(self) -> Result<i64, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: i128 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_i128(self) -> Result<i128, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: usize }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_usize(self) -> Result<usize, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: isize }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_isize(self) -> Result<isize, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: f32 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_f32(self) -> Result<f32, Self::Error> {
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: f64 }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_f64(self) -> Result<f64, Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Float64,
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
    /// use musli::{Allocator, Decode, Decoder};
    /// # struct MyType { data: [u8; 128] }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MyType
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
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
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// use musli::de::UnsizedVisitor;
    /// # struct BytesReference<'de> { data: &'de [u8] }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for BytesReference<'de>
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor;
    ///
    ///         #[musli::de::unsized_visitor]
    ///         impl<'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor
    ///         where
    ///             C: Context
    ///         {
    ///             type Ok = &'de [u8];
    ///
    ///             #[inline]
    ///             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///                 write!(f, "a literal byte reference")
    ///             }
    ///
    ///             #[inline]
    ///             fn visit_borrowed(self, cx: C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
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
    fn decode_bytes<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, Self::Cx, [u8], Error = Self::Error, Allocator = Self::Allocator>,
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
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// use musli::de::UnsizedVisitor;
    /// # struct StringReference<'de> { data: &'de str }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for StringReference<'de>
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         struct Visitor;
    ///
    ///         #[musli::de::unsized_visitor]
    ///         impl<'de, C> UnsizedVisitor<'de, C, str> for Visitor
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
    ///             fn visit_borrowed(self, _: C, bytes: &'de str) -> Result<Self::Ok, C::Error> {
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
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, Self::Cx, str, Error = Self::Error, Allocator = Self::Allocator>,
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
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// # struct OptionalField { data: Option<String>}
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for OptionalField
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_option(self) -> Result<Option<Self::DecodeSome>, Self::Error> {
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
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    /// # struct PackedStruct { field: u32, data: [u8; 128] }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for PackedStruct
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_pack<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodePack) -> Result<O, Self::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Pack,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a sequence of values.
    #[inline]
    fn decode_slice<V, T>(self) -> Result<V, Self::Error>
    where
        V: DecodeSliceBuilder<T, Self::Allocator>,
        T: Decode<'de, Self::Mode, Self::Allocator>,
    {
        utils::default_decode_slice(self)
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
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    ///
    /// struct VectorField {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for VectorField
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    /// use musli::hint::SequenceHint;
    /// # struct TupleStruct(String, u32);
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for TupleStruct
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_sequence<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::UnsizedSequence,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode a sequence with a `hint` indicating its expected characteristics.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// use musli::de::SequenceDecoder;
    /// use musli::hint::SequenceHint;
    /// # struct TupleStruct(String, u32);
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for TupleStruct
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_sequence_hint<F, O>(self, hint: &SequenceHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeSequence) -> Result<O, Self::Error>,
    {
        self.decode_sequence(f)
    }

    /// Decode a map who's size is not known at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::{Allocator, Decode, Decoder};
    /// use musli::de::{MapDecoder, EntryDecoder};
    /// # struct MapStruct { data: HashMap<String, u32> }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for MapStruct
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
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
    fn decode_map<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::UnsizedMap,
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
    /// use musli::{Allocator, Context, Decode, Decoder};
    /// use musli::de::{MapDecoder, EntryDecoder};
    /// use musli::hint::MapHint;
    ///
    /// struct Struct {
    ///     string: String,
    ///     integer: u32,
    /// }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for Struct
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         static HINT: MapHint = MapHint::with_size(2);
    ///
    ///         let cx = decoder.cx();
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
    ///                         return Err(cx.message(format_args!("Invalid field tag {tag:?}")));
    ///                     }
    ///                 }
    ///             }
    ///
    ///             Ok(Self {
    ///                 string: string.ok_or_else(|| cx.message("Expected tag `string`"))?,
    ///                 integer: integer.ok_or_else(|| cx.message("Expected tag `integer`"))?,
    ///             })
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_map_hint<F, O>(self, _: &MapHint, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMap) -> Result<O, Self::Error>,
    {
        self.decode_map(f)
    }

    /// Simplified decoding a map of unknown length.
    ///
    /// The length of the map must somehow be determined from the underlying
    /// format.
    #[inline]
    fn decode_map_entries<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeMapEntries) -> Result<O, Self::Error>,
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
    /// use musli::{Allocator, Context, Decode};
    /// use musli::de::{Decoder, VariantDecoder};
    ///
    /// enum Enum {
    ///     Number(u32),
    ///     String(String),
    /// }
    ///
    /// impl<'de, M, A> Decode<'de, M, A> for Enum
    /// where
    ///     A: Allocator,
    /// {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let cx = decoder.cx();
    ///
    ///         decoder.decode_variant(|variant| {
    ///             let tag = variant.decode_tag()?.decode()?;
    ///             let value = variant.decode_value()?;
    ///
    ///             match tag {
    ///                 0 => Ok(Self::Number(value.decode()?)),
    ///                 1 => Ok(Self::String(value.decode()?)),
    ///                 tag => Err(cx.message(format_args!("Invalid variant tag {tag:?}"))),
    ///             }
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_variant<F, O>(self, f: F) -> Result<O, Self::Error>
    where
        F: FnOnce(&mut Self::DecodeVariant) -> Result<O, Self::Error>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Variant,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode an unknown number using a visitor.
    #[inline]
    fn decode_number<V>(self, visitor: V) -> Result<V::Ok, Self::Error>
    where
        V: Visitor<'de, Self::Cx, Error = Self::Error, Allocator = Self::Allocator>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Number,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Decode dynamically through a [`Visitor`].
    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, Self::Cx, Error = Self::Error, Allocator = Self::Allocator>,
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
