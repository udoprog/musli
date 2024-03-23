#![allow(unused_variables)]

use core::fmt;

use crate::expecting::{self, Expecting};
use crate::Context;

use super::{
    Encode, MapEncoder, MapEntriesEncoder, SequenceEncoder, StructEncoder, VariantEncoder,
};

/// Trait governing how the encoder works.
pub trait Encoder: Sized {
    /// Context associated with the encoder.
    type Cx: ?Sized + Context<Error = Self::Error, Mode = Self::Mode>;
    /// The type returned by the encoder. For [Encode] implementations ensures
    /// that they are used correctly, since only functions returned by the
    /// [Encoder] is capable of returning this value.
    type Ok;
    /// Error associated with encoding.
    type Error;
    /// Mode associated with encoding.
    type Mode;
    /// Constructed [`Encoder`] with a different context.
    type WithContext<U>: Encoder<Cx = U, Ok = Self::Ok, Error = U::Error, Mode = U::Mode>
    where
        U: Context;
    /// A simple pack that packs a sequence of elements.
    type EncodePack<'this>: SequenceEncoder<Cx = Self::Cx, Ok = Self::Ok>
    where
        Self::Cx: 'this;
    /// Encoder returned when encoding an optional value which is present.
    type EncodeSome: Encoder<Cx = Self::Cx, Ok = Self::Ok, Error = Self::Error, Mode = Self::Mode>;
    /// The type of a sequence encoder.
    type EncodeSequence: SequenceEncoder<Cx = Self::Cx, Ok = Self::Ok>;
    /// The type of a tuple encoder.
    type EncodeTuple: SequenceEncoder<Cx = Self::Cx, Ok = Self::Ok>;
    /// The type of a map encoder.
    type EncodeMap: MapEncoder<Cx = Self::Cx, Ok = Self::Ok>;
    /// Streaming encoder for map pairs.
    type EncodeMapEntries: MapEntriesEncoder<Cx = Self::Cx, Ok = Self::Ok>;
    /// Encoder that can encode a struct.
    type EncodeStruct: StructEncoder<Cx = Self::Cx, Ok = Self::Ok>;
    /// Encoder for a struct variant.
    type EncodeVariant: VariantEncoder<Cx = Self::Cx, Ok = Self::Ok>;
    /// Specialized encoder for a tuple variant.
    type EncodeTupleVariant: SequenceEncoder<Cx = Self::Cx, Ok = Self::Ok>;
    /// Specialized encoder for a struct variant.
    type EncodeStructVariant: StructEncoder<Cx = Self::Cx, Ok = Self::Ok>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::encoder]`][crate::encoder] attribute
    /// macro when implementing [`Encoder`].
    #[doc(hidden)]
    type __UseMusliEncoderAttributeMacro;

    /// Construct an encoder with a different context.
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

    /// An expectation error. Every other implementation defers to this to
    /// report that something unexpected happened.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_unit(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit(self, cx: &Self::Cx) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Unit,
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_bool(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bool(self, cx: &Self::Cx, v: bool) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_char(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_char(self, cx: &Self::Cx, v: char) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u8(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u8(self, cx: &Self::Cx, v: u8) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u16(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u16(self, cx: &Self::Cx, v: u16) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u32(self, cx: &Self::Cx, v: u32) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u64(self, cx: &Self::Cx, v: u64) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_u128(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u128(self, cx: &Self::Cx, v: u128) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i8(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i8(self, cx: &Self::Cx, v: i8) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i16(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i16(self, cx: &Self::Cx, v: i16) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i32(self, cx: &Self::Cx, v: i32) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i64(self, cx: &Self::Cx, v: i64) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_i128(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i128(self, cx: &Self::Cx, v: i128) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_usize(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_usize(
        self,
        cx: &Self::Cx,
        v: usize,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_isize(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_isize(
        self,
        cx: &Self::Cx,
        v: isize,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_f32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f32(self, cx: &Self::Cx, v: f32) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_f64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f64(self, cx: &Self::Cx, v: f64) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_array(cx, &self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_array<const N: usize>(
        self,
        cx: &Self::Cx,
        array: &[u8; N],
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_bytes(cx, self.data.as_slice())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes(
        self,
        cx: &Self::Cx,
        bytes: &[u8],
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let (first, second) = self.data.as_slices();
    ///         encoder.encode_bytes_vectored(cx, self.data.len(), [first, second])
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes_vectored<I>(
        self,
        cx: &Self::Cx,
        len: usize,
        vectors: I,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_string(cx, self.data.as_str())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_string(
        self,
        cx: &Self::Cx,
        string: &str,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::String,
            ExpectingWrapper::new(&self),
        )))
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
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
    fn encode_some(self, cx: &Self::Cx) -> Result<Self::EncodeSome, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
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
    fn encode_none(self, cx: &Self::Cx) -> Result<Self::Ok, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    /// use musli::en::{SequenceEncoder};
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut pack = encoder.encode_pack(cx)?;
    ///         self.field.encode(cx, pack.encode_next(cx)?)?;
    ///         self.data.encode(cx, pack.encode_next(cx)?)?;
    ///         pack.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::EncodePack<'_>, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    /// use musli::en::{SequenceEncoder};
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_pack_fn(cx, |pack| {
    ///             self.field.encode(cx, pack.encode_next(cx)?)?;
    ///             self.data.encode(cx, pack.encode_next(cx)?)?;
    ///             Ok(())
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack_fn<F>(
        self,
        cx: &Self::Cx,
        f: F,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodePack<'_>) -> Result<(), <Self::Cx as Context>::Error>,
    {
        let mut pack = self.encode_pack(cx)?;
        f(&mut pack)?;
        pack.end(cx)
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
    /// use musli::en::{SequenceEncoder};
    /// # struct MyType { data: Vec<String> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
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
        cx: &Self::Cx,
        len: usize,
    ) -> Result<Self::EncodeSequence, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Sequence,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a sequence using a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::{SequenceEncoder};
    /// # struct MyType { data: Vec<String> }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_sequence_fn(cx, self.data.len(), |seq| {
    ///             for element in &self.data {
    ///                 seq.push(cx, element)?;
    ///             }
    ///
    ///             Ok(())
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence_fn<F>(
        self,
        cx: &Self::Cx,
        len: usize,
        f: F,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodeSequence) -> Result<(), <Self::Cx as Context>::Error>,
    {
        let mut seq = self.encode_sequence(cx, len)?;
        f(&mut seq)?;
        seq.end(cx)
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
    /// use musli::{Encode, Encoder};
    /// use musli::en::{SequenceEncoder};
    ///
    /// struct PackedTuple(u32, [u8; 128]);
    ///
    /// impl<M> Encode<M> for PackedTuple {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
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
        cx: &Self::Cx,
        len: usize,
    ) -> Result<Self::EncodeTuple, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Tuple,
            ExpectingWrapper::new(&self),
        )))
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
    /// use musli::en::{MapEncoder};
    /// # struct Struct { name: String, age: u32 }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut map = encoder.encode_map(cx, 2)?;
    ///         map.insert_entry(cx, "name", &self.name)?;
    ///         map.insert_entry(cx, "age", self.age)?;
    ///         map.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map(
        self,
        cx: &Self::Cx,
        len: usize,
    ) -> Result<Self::EncodeMap, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    /// use musli::en::{MapEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_map_fn(cx, 2, |map| {
    ///             map.insert_entry(cx, "name", &self.name)?;
    ///             map.insert_entry(cx, "age", self.age)?;
    ///             Ok(())
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map_fn<F>(
        self,
        cx: &Self::Cx,
        len: usize,
        f: F,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodeMap) -> Result<(), <Self::Cx as Context>::Error>,
    {
        let mut map = self.encode_map(cx, len)?;
        f(&mut map)?;
        map.end(cx)
    }

    /// Encode a map through pairs with a known length `len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::{MapEntriesEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
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
        cx: &Self::Cx,
        len: usize,
    ) -> Result<Self::EncodeMapEntries, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::MapPairs,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::{StructEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         let mut st = encoder.encode_struct(cx, 2)?;
    ///         st.insert_field(cx, "name", &self.name)?;
    ///         st.insert_field(cx, "age", self.age)?;
    ///         st.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct(
        self,
        cx: &Self::Cx,
        fields: usize,
    ) -> Result<Self::EncodeStruct, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
            &expecting::Struct,
            ExpectingWrapper::new(&self),
        )))
    }

    /// Encode a struct using a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder};
    /// use musli::en::{StructEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         encoder.encode_struct_fn(cx, 2, |st| {
    ///             st.insert_field(cx, "name", &self.name)?;
    ///             st.insert_field(cx, "age", self.age)?;
    ///             Ok(())
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct_fn<F>(
        self,
        cx: &Self::Cx,
        fields: usize,
        f: F,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodeStruct) -> Result<(), <Self::Cx as Context>::Error>,
    {
        let mut st = self.encode_struct(cx, fields)?;
        f(&mut st)?;
        st.end(cx)
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
    /// use musli::en::{VariantEncoder, StructEncoder, SequenceEncoder};
    /// # enum Enum { UnitVariant, TupleVariant(String), StructVariant { data: String, age: u32 } }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
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
    ///             Enum::StructVariant { data, age } => {
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
    fn encode_variant(
        self,
        cx: &Self::Cx,
    ) -> Result<Self::EncodeVariant, <Self::Cx as Context>::Error> {
        Err(cx.message(expecting::unsupported_type(
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
    /// use musli::en::{VariantEncoder, StructEncoder, SequenceEncoder};
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
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 encoder.encode_variant(cx)?.insert_variant(cx, "variant1", ())
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 encoder.encode_variant(cx)?.insert_variant(cx, "variant2", data)
    ///             }
    ///             Enum::StructVariant { data, age } => {
    ///                 encoder.encode_variant_fn(cx, |variant| {
    ///                     variant.encode_tag(cx)?.encode_string(cx, "variant3")?;
    ///
    ///                     variant.encode_value(cx)?.encode_struct_fn(cx, 2, |st| {
    ///                         st.insert_field(cx, "data", data)?;
    ///                         st.insert_field(cx, "age", age)?;
    ///                         Ok(())
    ///                     })?;
    ///
    ///                     Ok(())
    ///                 })
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_variant_fn<F>(
        self,
        cx: &Self::Cx,
        f: F,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        F: FnOnce(&mut Self::EncodeVariant) -> Result<(), <Self::Cx as Context>::Error>,
    {
        let mut variant = self.encode_variant(cx)?;
        f(&mut variant)?;
        variant.end(cx)
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
    /// use musli::en::{VariantEncoder, StructEncoder, SequenceEncoder};
    /// # enum Enum { UnitVariant }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 encoder.encode_unit_variant(cx, &"variant1")
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit_variant<T>(
        self,
        cx: &Self::Cx,
        tag: &T,
    ) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        T: ?Sized + Encode<<Self::Cx as Context>::Mode>,
    {
        let mut variant = self.encode_variant(cx)?;
        let t = variant.encode_tag(cx)?;
        tag.encode(cx, t)?;
        let v = variant.encode_value(cx)?;
        v.encode_unit(cx)?;
        variant.end(cx)
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
    /// use musli::en::{VariantEncoder, StructEncoder, SequenceEncoder};
    /// # enum Enum { TupleVariant(String) }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::TupleVariant(data) => {
    ///                 let mut variant = encoder.encode_tuple_variant(cx, &"variant2", 1)?;
    ///                 variant.push(cx, data)?;
    ///                 variant.end(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_tuple_variant<T>(
        self,
        cx: &Self::Cx,
        tag: &T,
        fields: usize,
    ) -> Result<Self::EncodeTupleVariant, <Self::Cx as Context>::Error>
    where
        T: ?Sized + Encode<<Self::Cx as Context>::Mode>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::TupleVariant,
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
    /// use musli::en::{VariantEncoder, StructEncoder, SequenceEncoder};
    /// # enum Enum { StructVariant { data: String, age: u32 } }
    ///
    /// impl<M> Encode<M> for Enum {
    ///     fn encode<E>(&self, cx: &E::Cx, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder,
    ///     {
    ///         match self {
    ///             Enum::StructVariant { data, age } => {
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
        cx: &Self::Cx,
        tag: &T,
        fields: usize,
    ) -> Result<Self::EncodeStructVariant, <Self::Cx as Context>::Error>
    where
        T: ?Sized + Encode<<Self::Cx as Context>::Mode>,
    {
        Err(cx.message(expecting::unsupported_type(
            &expecting::StructVariant,
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
