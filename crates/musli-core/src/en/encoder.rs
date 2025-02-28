use core::fmt;

use crate::expecting::{self, Expecting};
use crate::hint::{MapHint, SequenceHint};
use crate::Context;

use super::{utils, Encode, EntriesEncoder, MapEncoder, SequenceEncoder, VariantEncoder};

/// An outcome of a fast encode attempt.
#[non_exhaustive]
pub enum TryFastEncode<T, E>
where
    E: Encoder,
{
    /// The encode attempt was successful.
    Ok,
    /// The encode was unsupported.
    Unsupported(T, E),
}

/// Trait governing how the encoder works.
#[must_use = "Encoders must be consumed through one of its encode_* methods"]
#[allow(unused_variables)]
pub trait Encoder: Sized {
    /// Context associated with the encoder.
    type Cx: Context<Error = Self::Error>;
    /// Error associated with encoding.
    type Error;
    /// Mode associated with encoding.
    type Mode: 'static;
    /// A simple pack that packs a sequence of elements.
    type EncodePack: SequenceEncoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// Encoder returned when encoding an optional value which is present.
    type EncodeSome: Encoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// The type of a sequence encoder.
    type EncodeSequence: SequenceEncoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// The type of a map encoder.
    type EncodeMap: MapEncoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// Streaming encoder for map pairs.
    type EncodeMapEntries: EntriesEncoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// Encoder for a variant.
    type EncodeVariant: VariantEncoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;
    /// Encoder for a sequence variant.
    type EncodeSequenceVariant: SequenceEncoder<
        Cx = Self::Cx,
        Error = Self::Error,
        Mode = Self::Mode,
    >;
    /// Encoder for a map variant.
    type EncodeMapVariant: MapEncoder<Cx = Self::Cx, Error = Self::Error, Mode = Self::Mode>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::encoder]`][musli::encoder]
    /// attribute macro when implementing [`Encoder`].
    #[doc(hidden)]
    type __UseMusliEncoderAttributeMacro;

    /// Access the context associated with the encoder.
    fn cx(&self) -> Self::Cx;

    /// An expectation error. Every other implementation defers to this to
    /// report that something unexpected happened.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Try to quickly encode the specified value.
    ///
    /// The default implementation simply returns the current encoder as
    /// `Err(Self)`.
    ///
    /// This is intended to be a fast path when encoding a value when an
    /// encoding permits it.
    #[inline]
    fn try_fast_encode<T>(self, value: T) -> Result<TryFastEncode<T, Self>, Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        Ok(TryFastEncode::Unsupported(value, self))
    }

    /// Encode the value `T` into the current encoder.
    ///
    /// This calls the appropriate [`Encode`] implementation for the given type.
    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        match self.try_fast_encode(value)? {
            TryFastEncode::Ok => Ok(()),
            TryFastEncode::Unsupported(value, this) => value.encode(this),
        }
    }

    /// Encode a unit or something that is completely empty.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// struct UnitStruct;
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct UnitStruct;
    ///
    /// impl<M> Encode<M> for UnitStruct {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_empty()
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_empty(self) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Empty,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a boolean value.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: bool
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: bool }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_bool(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bool(self, v: bool) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Bool,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a character.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: char
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: char }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_char(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_char(self, v: char) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Char,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 8-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u8
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: u8 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u8(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u8(self, v: u8) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u16
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: u16 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u16(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u16(self, v: u16) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u32
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: u32 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u32(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u32(self, v: u32) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u64
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: u64 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u64(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u64(self, v: u64) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: u128
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: u128 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u128(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u128(self, v: u128) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Unsigned128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i8
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: i8 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i8(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i8(self, v: i8) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed8,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i16
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: i16 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i16(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i16(self, v: i16) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed16,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i32
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: i32 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i32(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i32(self, v: i32) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i64
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: i64 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i64(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i64(self, v: i64) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: i128
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: i128 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i128(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i128(self, v: i128) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Signed128,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a [`usize`].
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: usize
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: usize }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_usize(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_usize(self, v: usize) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Usize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a [`isize`].
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: isize
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: isize }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_isize(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_isize(self, v: isize) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Isize,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: f32
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: f32 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_f32(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f32(self, v: f32) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Float32,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: f64
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: f64 }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_f64(self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f64(self, v: f64) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Float64,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode fixed-length array.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: [u8; 128]
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: [u8; 128] }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_array(&self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_array<const N: usize>(self, array: &[u8; N]) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Array,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a sequence of bytes.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     #[musli(bytes)]
    ///     data: Vec<u8>
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: Vec<u8> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_bytes(self.data.as_slice())
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes(self, bytes: &[u8]) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Bytes,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode the given slices of bytes in sequence, with one following another
    /// as a single contiguous byte array.
    ///
    /// The provided `len` is trusted, but providing the wrong length must never
    /// result in any memory unsafety. It might just cause the payload to be
    /// corrupted.
    ///
    /// This can be useful to avoid allocations when a caller doesn't have
    /// access to a single byte sequence like in
    /// [`VecDeque`][std::collections::VecDeque].
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use std::collections::VecDeque;
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: VecDeque<u8>
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # use std::collections::VecDeque;
    /// # struct MyType { data: VecDeque<u8> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let (first, second) = self.data.as_slices();
    ///         encoder.encode_bytes_vectored(self.data.len(), [first, second])
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes_vectored<I>(self, len: usize, vectors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item: AsRef<[u8]>>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Bytes,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a string.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: String
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: String }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_string(self.data.as_str())
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_string(self, string: &str) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::String,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a value that implements [`Display`] as a string.
    ///
    /// [`Display`]: fmt::Display
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: String,
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.collect_string(self.data.as_str())
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn collect_string<T>(self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let cx = self.cx();
        let buf = crate::alloc::collect_string(cx.alloc(), value).map_err(cx.map())?;
        self.encode_string(buf.as_ref())
    }

    /// Encode an optional value that is present.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: Option<String>
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: Option<String> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some()?.encode(data)
    ///             }
    ///             None => {
    ///                 encoder.encode_none()
    ///             }
    ///         }
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Option,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode an optional value that is absent.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: Option<String>
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: Option<String> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some()?.encode(data)
    ///             }
    ///             None => {
    ///                 encoder.encode_none()
    ///             }
    ///         }
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Option,
            ExpectingWrapper::new(&self),
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
    /// use musli::{Encode, Encoder};
    /// use musli::en::SequenceEncoder;
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut pack = encoder.encode_pack()?;
    ///         pack.encode_next()?.encode(self.field)?;
    ///         pack.encode_next()?.encode(self.data)?;
    ///         pack.finish_sequence()
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Pack,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encodes a pack using a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::SequenceEncoder;
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_pack_fn(|pack| {
    ///             pack.encode_next()?.encode(self.field)?;
    ///             pack.encode_next()?.encode(&self.data)?;
    ///             Ok(())
    ///         })
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack_fn<F>(self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::EncodePack) -> Result<(), Self::Error>,
    {
        let mut pack = self.encode_pack()?;
        f(&mut pack)?;
        pack.finish_sequence()
    }

    /// Encode a slice as a sequence.
    ///
    /// This defaults to using [`Encoder::encode_sequence`] and if specialized
    /// must implement the same format as would calling that method.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: Vec<String>
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// # struct MyType { data: Vec<String> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_slice(&self.data)
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_slice<T>(self, slice: impl AsRef<[T]>) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        utils::default_encode_slice(self, slice)
    }

    /// Encode a sequence extracted from an iterator of slices with a trusted
    /// length.
    ///
    /// This defaults to using [`Encoder::encode_sequence`] and if specialized
    /// must implement the same format as would calling that method.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use std::collections::VecDeque;
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: VecDeque<String>,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// # use musli::Encode;
    /// # use std::collections::VecDeque;
    /// # struct MyType { data: VecDeque<String> }
    /// use musli::Encoder;
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let (a, b) = self.data.as_slices();
    ///         encoder.encode_slices(self.data.len(), [a, b])
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_slices<T>(
        self,
        len: usize,
        slices: impl IntoIterator<Item: AsRef<[T]>>,
    ) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        utils::default_encode_slices(self, len, slices)
    }

    /// Encode a sequence with a known length `len`.
    ///
    /// A sequence encodes one element following another and must in some way
    /// encode the length of the sequence in the underlying format. It is
    /// decoded with [`Decoder::decode_sequence`].
    ///
    /// [`Decoder::decode_sequence`]: crate::de::Decoder::decode_sequence
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// #[musli(packed)]
    /// struct MyType {
    ///     data: Vec<String>
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::SequenceEncoder;
    /// # struct MyType { data: Vec<String> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut seq = encoder.encode_sequence(self.data.len())?;
    ///
    ///         for element in &self.data {
    ///             seq.push(element)?;
    ///         }
    ///
    ///         seq.finish_sequence()
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    ///
    /// Encoding a tuple:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::SequenceEncoder;
    ///
    /// struct PackedTuple(u32, [u8; 128]);
    ///
    /// impl<M> Encode<M> for PackedTuple {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut tuple = encoder.encode_sequence(2)?;
    ///         tuple.encode_next()?.encode(self.0)?;
    ///         tuple.encode_next()?.encode(&self.1)?;
    ///         tuple.finish_sequence()
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence(self, hint: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::SequenceWith(hint.size_hint()),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a sequence using a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::SequenceEncoder;
    /// # struct MyType { data: Vec<String> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_sequence_fn(self.data.len(), |seq| {
    ///             for element in &self.data {
    ///                 seq.push(element)?;
    ///             }
    ///
    ///             Ok(())
    ///         })
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    ///
    /// Encoding a tuple:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::SequenceEncoder;
    ///
    /// struct PackedTuple(u32, [u8; 128]);
    ///
    /// impl<M> Encode<M> for PackedTuple {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_sequence_fn(2, |tuple| {
    ///             tuple.encode_next()?.encode(self.0)?;
    ///             tuple.encode_next()?.encode(&self.1)?;
    ///             Ok(())
    ///         })
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence_fn<F>(self, hint: impl SequenceHint, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::EncodeSequence) -> Result<(), Self::Error>,
    {
        let mut seq = self.encode_sequence(hint)?;
        f(&mut seq)?;
        seq.finish_sequence()
    }

    /// Encode a map with a known length `len`.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::MapEncoder;
    /// # struct Struct { name: String, age: u32 }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut map = encoder.encode_map(2)?;
    ///         map.insert_entry("name", &self.name)?;
    ///         map.insert_entry("age", self.age)?;
    ///         map.finish_map()
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    #[must_use = "Map encoders must be consumed"]
    fn encode_map(self, hint: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Map,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a map using a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::MapEncoder;
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_map_fn(2, |map| {
    ///             map.insert_entry("name", &self.name)?;
    ///             map.insert_entry("age", self.age)?;
    ///             Ok(())
    ///         })
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map_fn<F>(self, hint: impl MapHint, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::EncodeMap) -> Result<(), Self::Error>,
    {
        let mut map = self.encode_map(hint)?;
        f(&mut map)?;
        map.finish_map()
    }

    /// Encode a map through pairs with a known length `len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::EntriesEncoder;
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut m = encoder.encode_map_entries(2)?;
    ///
    ///         // Simplified encoding.
    ///         m.insert_entry("name", &self.name)?;
    ///
    ///         // Key and value encoding as a stream.
    ///         m.encode_entry_key()?.encode("age")?;
    ///         m.encode_entry_value()?.encode(self.age)?;
    ///         m.finish_entries()
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map_entries(self, hint: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::MapWith(hint.size_hint()),
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// enum Enum {
    ///     UnitVariant,
    ///     TupleVariant(String),
    ///     StructVariant {
    ///         data: String,
    ///         age: u32,
    ///     }
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::{VariantEncoder, SequenceEncoder, MapEncoder};
    /// # enum Enum { UnitVariant, TupleVariant(String), StructVariant { data: String, age: u32 } }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut variant = encoder.encode_variant()?;
    ///
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 variant.insert_variant("UnitVariant", ())
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 variant.encode_tag()?.encode_string("TupleVariant")?;
    ///
    ///                 let mut tuple = variant.encode_data()?.encode_sequence(1)?;
    ///                 tuple.push(data)?;
    ///                 tuple.finish_sequence()?;
    ///
    ///                 variant.finish_variant()
    ///             }
    ///             Enum::StructVariant { data, age } => {
    ///                 variant.encode_tag()?.encode_string("StructVariant")?;
    ///
    ///                 let mut st = variant.encode_data()?.encode_map(2)?;
    ///                 st.insert_entry("data", data)?;
    ///                 st.insert_entry("age", age)?;
    ///                 st.finish_map()?;
    ///
    ///                 variant.finish_variant()
    ///             }
    ///         }
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::Variant,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a variant using a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::{VariantEncoder, SequenceEncoder, MapEncoder};
    ///
    /// enum Enum {
    ///     UnitVariant,
    ///     TupleVariant(String),
    ///     StructVariant {
    ///         data: String,
    ///         age: u32,
    ///     }
    /// }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 encoder.encode_variant()?.insert_variant("variant1", ())
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 encoder.encode_variant_fn(|variant| {
    ///                     variant.encode_tag()?.encode("TupleVariant")?;
    ///
    ///                     variant.encode_data()?.encode_sequence_fn(2, |tuple| {
    ///                         tuple.push(data)?;
    ///                         Ok(())
    ///                     })?;
    ///
    ///                     Ok(())
    ///                 })
    ///             }
    ///             Enum::StructVariant { data, age } => {
    ///                 encoder.encode_variant_fn(|variant| {
    ///                     variant.encode_tag()?.encode("variant3")?;
    ///
    ///                     variant.encode_data()?.encode_map_fn(2, |st| {
    ///                         st.insert_entry("data", data)?;
    ///                         st.insert_entry("age", age)?;
    ///                         Ok(())
    ///                     })?;
    ///
    ///                     Ok(())
    ///                 })
    ///             }
    ///         }
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_variant_fn<F>(self, f: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut Self::EncodeVariant) -> Result<(), Self::Error>,
    {
        let mut variant = self.encode_variant()?;
        f(&mut variant)?;
        variant.finish_variant()
    }

    /// Simplified encoding for a unit variant.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// enum Enum {
    ///     UnitVariant,
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::{VariantEncoder, MapEncoder, SequenceEncoder};
    /// # enum Enum { UnitVariant }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 encoder.encode_unit_variant("variant1")
    ///             }
    ///         }
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit_variant<T>(self, tag: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        self.encode_variant_fn(|variant| {
            variant.encode_tag()?.encode(tag)?;
            variant.encode_data()?.encode_empty()?;
            Ok(())
        })
    }

    /// Simplified encoding for a tuple variant.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// enum Enum {
    ///     TupleVariant(String),
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::SequenceEncoder;
    /// # enum Enum { TupleVariant(String) }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::TupleVariant(data) => {
    ///                 let mut variant = encoder.encode_sequence_variant("variant2", 1)?;
    ///                 variant.push(data)?;
    ///                 variant.finish_sequence()
    ///             }
    ///         }
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence_variant<T>(
        self,
        tag: &T,
        hint: impl SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::SequenceVariant,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Simplified encoding for a struct variant.
    ///
    /// # Examples
    ///
    /// Deriving an implementation:
    ///
    /// ```
    /// use musli::Encode;
    ///
    /// #[derive(Encode)]
    /// enum Enum {
    ///     StructVariant {
    ///         data: String,
    ///         age: u32,
    ///     }
    /// }
    /// ```
    ///
    /// Implementing manually:
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::MapEncoder;
    /// # enum Enum { StructVariant { data: String, age: u32 } }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     type Encode = Self;
    ///
    ///     #[inline]
    ///     fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::StructVariant { data, age } => {
    ///                 let mut variant = encoder.encode_map_variant("variant3", 2)?;
    ///                 variant.insert_entry("data", data)?;
    ///                 variant.insert_entry("age", age)?;
    ///                 variant.finish_map()
    ///             }
    ///         }
    ///     }
    ///
    ///     #[inline]
    ///     fn as_encode(&self) -> &Self::Encode {
    ///         self
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map_variant<T>(
        self,
        tag: &T,
        hint: impl MapHint,
    ) -> Result<Self::EncodeMapVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        Err(self.cx().message(expecting::unsupported_type(
            &expecting::MapVariant,
            ExpectingWrapper::new(&self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T> {
    inner: T,
}

impl<T> ExpectingWrapper<T> {
    #[inline]
    const fn new(value: &T) -> &Self {
        // SAFETY: `ExpectingWrapper` is repr(transparent) over `T`.
        unsafe { &*(value as *const T as *const Self) }
    }
}

impl<T> Expecting for ExpectingWrapper<T>
where
    T: Encoder,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
