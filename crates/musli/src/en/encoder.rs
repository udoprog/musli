use core::fmt;

use crate::en::Encode;
use crate::error::Error;
use crate::expecting::{self, Expecting};
use crate::mode::Mode;
use crate::Context;

/// Trait governing how to encode a sequence.
pub trait SequenceEncoder {
    /// Result type of the encoder.
    type Ok;
    /// The error raised by a sequence encoder.
    type Error: Error;

    /// The encoder returned when advancing the sequence encoder.
    type Encoder<'this>: Encoder<Ok = Self::Ok, Error = Self::Error>
    where
        Self: 'this;

    /// Prepare encoding of the next element.
    #[must_use = "Encoder must be consumed"]
    fn next<C>(&mut self, cx: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Self::Error>;

    /// Push an element into the sequence.
    #[inline]
    fn push<M, C, T>(&mut self, cx: &mut C, value: T) -> Result<(), C::Error>
    where
        M: Mode,
        C: Context<Self::Error>,
        T: Encode<M>,
    {
        let encoder = self.next(cx)?;
        value.encode(cx, encoder)?;
        Ok(())
    }

    /// End the sequence.
    fn end<C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>;
}

/// Encoder for a sequence of pairs.
pub trait PairsEncoder {
    /// Result type of the encoder.
    type Ok;

    /// The error raised by a map encoder.
    type Error: Error;

    /// Encode the next pair.
    type Encoder<'this>: PairEncoder<Ok = Self::Ok, Error = Self::Error>
    where
        Self: 'this;

    /// Insert a pair immediately.
    #[inline]
    fn insert<M, C, F, S>(&mut self, cx: &mut C, first: F, second: S) -> Result<(), C::Error>
    where
        Self: Sized,
        M: Mode,
        C: Context<Self::Error>,
        F: Encode<M>,
        S: Encode<M>,
    {
        self.next(cx)?.insert(cx, first, second)?;
        Ok(())
    }

    /// Encode the next pair.
    fn next<C>(&mut self, cx: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Self::Error>;

    /// Finish encoding pairs.
    fn end<C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>;
}

/// Trait governing how to encode a sequence of pairs.
pub trait PairEncoder {
    /// Result type of the encoder.
    type Ok;
    /// The error raised by a map encoder.
    type Error: Error;

    /// The encoder returned when advancing the map encoder to encode the key.
    type First<'this>: Encoder<Ok = Self::Ok, Error = Self::Error>
    where
        Self: 'this;

    /// The encoder returned when advancing the map encoder to encode the value.
    type Second<'this>: Encoder<Ok = Self::Ok, Error = Self::Error>
    where
        Self: 'this;

    /// Insert the pair immediately.
    #[inline]
    fn insert<M, C, F, S>(mut self, cx: &mut C, first: F, second: S) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        M: Mode,
        C: Context<Self::Error>,
        F: Encode<M>,
        S: Encode<M>,
    {
        self.first(cx).and_then(|e| first.encode(cx, e))?;
        self.second(cx).and_then(|e| second.encode(cx, e))?;
        self.end(cx)
    }

    /// Return the encoder for the first element in the pair.
    #[must_use = "Encoder must be consumed through Encoder::encode_* methods, otherwise incomplete encoding might occur!"]
    fn first<C>(&mut self, cx: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<Self::Error>;

    /// Return encoder for the second element in the pair.
    #[must_use = "Encoder must be consumed through Encoder::encode_* methods, otherwise incomplete encoding might occur!"]
    fn second<C>(&mut self, cx: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<Self::Error>;

    /// Stop encoding this pair.
    fn end<C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>;
}

/// Trait governing how to encode a variant.
pub trait VariantEncoder {
    /// Result type of the encoder.
    type Ok;
    /// The error raised by a map encoder.
    type Error: Error;

    /// The encoder returned when advancing the map encoder to encode the key.
    type Tag<'this>: Encoder<Ok = Self::Ok, Error = Self::Error>
    where
        Self: 'this;

    /// The encoder returned when advancing the map encoder to encode the value.
    type Variant<'this>: Encoder<Ok = Self::Ok, Error = Self::Error>
    where
        Self: 'this;

    /// Insert the variant immediately.
    #[inline]
    fn insert<M, C, F, S>(mut self, cx: &mut C, first: F, second: S) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        M: Mode,
        C: Context<Self::Error>,
        F: Encode<M>,
        S: Encode<M>,
    {
        self.tag(cx).and_then(|e| first.encode(cx, e))?;
        self.variant(cx).and_then(|e| second.encode(cx, e))?;
        self.end(cx)
    }

    /// Return the encoder for the first element in the variant.
    #[must_use = "Encoder must be consumed through Encoder::encode_* methods, otherwise incomplete encoding might occur!"]
    fn tag<C>(&mut self, cx: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Self::Error>;

    /// Return encoder for the second element in the variant.
    #[must_use = "Encoder must be consumed through Encoder::encode_* methods, otherwise incomplete encoding might occur!"]
    fn variant<C>(&mut self, cx: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Self::Error>;

    /// End the variant encoder.
    fn end<C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>;
}

/// Trait governing how the encoder works.
pub trait Encoder: Sized {
    /// The type returned by the encoder. For [Encode] implementations ensures
    /// that they are used correctly, since only functions returned by the
    /// [Encoder] is capable of returning this value.
    type Ok;
    /// The error raised by an encoder.
    type Error: Error;
    /// Encoder returned when encoding an optional value which is present.
    type Some: Encoder<Ok = Self::Ok, Error = Self::Error>;
    /// A simple pack that packs a sequence of elements.
    type Pack: SequenceEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// The type of a sequence encoder.
    type Sequence: SequenceEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// The type of a tuple encoder.
    type Tuple: SequenceEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// The type of a map encoder.
    type Map: PairsEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder that can encode a struct.
    type Struct: PairsEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder for a struct variant.
    type Variant: VariantEncoder<Ok = Self::Ok, Error = Self::Error>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::encoder]`][crate::encoder] attribute
    /// macro when implementing [`Encoder`].
    #[doc(hidden)]
    type __UseMusliEncoderAttributeMacro;

    /// An expectation error. Every other implementation defers to this to
    /// report that something unexpected happened.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Encode a unit or something that is completely empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct EmptyStruct;
    ///
    /// impl<M> Encode<M> for EmptyStruct where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_unit(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit<C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unit,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a boolean value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: bool,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_bool(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bool<C>(self, cx: &mut C, _: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Bool,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a character.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: char,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_char(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_char<C>(self, cx: &mut C, _: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Char,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u8,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u8(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u8<C>(self, cx: &mut C, _: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u16,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u16(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u16<C>(self, cx: &mut C, _: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u32<C>(self, cx: &mut C, _: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u64<C>(self, cx: &mut C, _: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u128,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u128(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u128<C>(self, cx: &mut C, _: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Unsigned128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i8,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i8(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i8<C>(self, cx: &mut C, _: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i16,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i16(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i16<C>(self, cx: &mut C, _: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i32<C>(self, cx: &mut C, _: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i64<C>(self, cx: &mut C, _: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i128,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i128(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i128<C>(self, cx: &mut C, _: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Signed128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: usize,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_usize(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_usize<C>(self, cx: &mut C, _: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Usize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`isize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: isize,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_isize(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_isize<C>(self, cx: &mut C, _: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Isize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: f32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_f32(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f32<C>(self, cx: &mut C, _: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Float32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: f64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_f64(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f64<C>(self, cx: &mut C, _: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Float64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode fixed-length array.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_array(cx, self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_array<C, const N: usize>(self, cx: &mut C, _: [u8; N]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Array,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a sequence of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: Vec<u8>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_bytes(cx, self.data.as_slice())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes<C>(self, cx: &mut C, _: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
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
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: VecDeque<u8>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         let (first, second) = self.data.as_slices();
    ///         encoder.encode_bytes_vectored(cx, &[first, second])
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes_vectored<C>(self, cx: &mut C, _: &[&[u8]]) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Bytes,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: String,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         encoder.encode_string(cx, self.data.as_str())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_string<C>(self, cx: &mut C, _: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::String,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some(cx).and_then(|e| Encode::<M>::encode(data, cx, e))
    ///             }
    ///             None => {
    ///                 encoder.encode_none(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_some<C>(self, cx: &mut C) -> Result<Self::Some, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Option,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Context, Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some(cx).and_then(|e| Encode::<M>::encode(data, cx, e))
    ///             }
    ///             None => {
    ///                 encoder.encode_none(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_none<C>(self, cx: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
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
    /// use musli::mode::Mode;
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         let mut pack = encoder.encode_pack(cx)?;
    ///         pack.next(cx)?.encode_u32(cx, self.field)?;
    ///         pack.next(cx)?.encode_array(cx, self.data)?;
    ///         pack.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack<C>(self, cx: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
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
    /// use musli::mode::Mode;
    ///
    /// struct MyType {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         let mut seq = encoder.encode_sequence(cx, self.data.len())?;
    ///
    ///         for element in &self.data {
    ///             seq.push::<M, _, _>(cx, element)?;
    ///         }
    ///
    ///         seq.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence<C>(
        self,
        cx: &mut C,
        #[allow(unused)] len: usize,
    ) -> Result<Self::Sequence, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
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
    /// use musli::mode::Mode;
    ///
    /// struct PackedTuple(u32, [u8; 364]);
    ///
    /// impl<M> Encode<M> for PackedTuple where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         let mut tuple = encoder.encode_tuple(cx, 2)?;
    ///         tuple.next(cx)?.encode_u32(cx, self.0)?;
    ///         tuple.next(cx)?.encode_array(cx, self.1)?;
    ///         tuple.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_tuple<C>(
        self,
        cx: &mut C,
        #[allow(unused)] len: usize,
    ) -> Result<Self::Tuple, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Tuple,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a map with a known length `len`.
    ///
    ///
    #[inline]
    fn encode_map<C>(self, cx: &mut C, #[allow(unused)] len: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Map,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, PairEncoder, PairsEncoder};
    /// use musli::mode::Mode;
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         let mut st = encoder.encode_struct(cx, 2)?;
    ///         st.insert::<M, _, _, _>(cx, "name", &self.name)?;
    ///         st.insert::<M, _, _, _>(cx, "age", self.age)?;
    ///         st.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct<C>(self, cx: &mut C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Struct,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an struct enum variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::Context;
    /// use musli::en::{Encode, Encoder, VariantEncoder, PairsEncoder};
    /// use musli::mode::Mode;
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
    /// impl<M> Encode<M> for Enum where M: Mode {
    ///     fn encode<C, E>(&self, cx: &mut C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<E::Error>,
    ///         E: Encoder
    ///     {
    ///         let mut variant = encoder.encode_variant(cx)?;
    ///
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 variant.insert::<M, _, _, _>(cx, "variant1", ())
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 variant.insert::<M, _, _, _>(cx, "variant2", data)
    ///             }
    ///             Enum::Variant { data, age } => {
    ///                 variant.tag(cx)?.encode_string(cx, "variant3")?;
    ///
    ///                 let mut st = variant.variant(cx)?.encode_struct(cx, 2)?;
    ///                 st.insert::<M, _, _, _>(cx, "data", data)?;
    ///                 st.insert::<M, _, _, _>(cx, "age", age)?;
    ///                 st.end(cx)?;
    ///
    ///                 variant.end(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_variant<C>(self, cx: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<Self::Error>,
    {
        Err(cx.message(expecting::invalid_type(
            &expecting::Variant,
            &ExpectingWrapper::new(self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T>(T);

impl<T> ExpectingWrapper<T> {
    #[inline]
    const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Expecting for ExpectingWrapper<T>
where
    T: Encoder,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
