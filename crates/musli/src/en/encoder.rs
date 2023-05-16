use core::fmt;

use crate::en::Encode;
use crate::error::Error;
use crate::expecting::{self, Expecting};
use crate::mode::Mode;

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
    #[must_use = "encoders must be consumed"]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error>;

    /// Push an element into the sequence.
    #[inline]
    fn push<M, T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        M: Mode,
        T: Encode<M>,
    {
        let encoder = self.next()?;
        value.encode(encoder)?;
        Ok(())
    }

    /// End the sequence.
    fn end(self) -> Result<Self::Ok, Self::Error>;
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
    fn insert<M, F, S>(&mut self, first: F, second: S) -> Result<(), Self::Error>
    where
        Self: Sized,
        M: Mode,
        F: Encode<M>,
        S: Encode<M>,
    {
        self.next()?.insert(first, second)?;
        Ok(())
    }

    /// Encode the next pair.
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error>;

    /// Finish encoding pairs.
    fn end(self) -> Result<Self::Ok, Self::Error>;
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
    fn insert<M, F, S>(mut self, first: F, second: S) -> Result<Self::Ok, Self::Error>
    where
        Self: Sized,
        M: Mode,
        F: Encode<M>,
        S: Encode<M>,
    {
        self.first().and_then(|e| first.encode(e))?;
        self.second().and_then(|e| second.encode(e))?;
        self.end()
    }

    /// Return the encoder for the first element in the pair.
    #[must_use = "encoders must be consumed"]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error>;

    /// Return encoder for the second element in the pair.
    #[must_use = "encoders must be consumed"]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error>;

    /// Stop encoding this pair.
    fn end(self) -> Result<Self::Ok, Self::Error>;
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
    fn insert<M, F, S>(mut self, first: F, second: S) -> Result<Self::Ok, Self::Error>
    where
        Self: Sized,
        M: Mode,
        F: Encode<M>,
        S: Encode<M>,
    {
        self.tag().and_then(|e| first.encode(e))?;
        self.variant().and_then(|e| second.encode(e))?;
        self.end()
    }

    /// Return the encoder for the first element in the variant.
    #[must_use = "encoders must be consumed"]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error>;

    /// Return encoder for the second element in the variant.
    #[must_use = "encoders must be consumed"]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error>;

    /// End the variant encoder.
    fn end(self) -> Result<Self::Ok, Self::Error>;
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
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct EmptyStruct;
    ///
    /// impl<M> Encode<M> for EmptyStruct where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_unit()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unit,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a boolean value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: bool,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_bool(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Bool,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a character.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: char,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_char(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Char,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u8,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u8(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u16,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u16(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u32(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u64(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: u128,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_u128(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_u128(self, _: u128) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Unsigned128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i8,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i8(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i16,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i16(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i32(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i64(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: i128,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_i128(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_i128(self, _: i128) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Signed128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: usize,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_usize(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_usize(self, _: usize) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Usize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`isize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: isize,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_isize(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_isize(self, _: isize) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Isize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: f32,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_f32(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Float32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: f64,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_f64(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Float64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode fixed-length array.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_array(self.data)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_array<const N: usize>(self, _: [u8; N]) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Array,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a sequence of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: Vec<u8>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_bytes(self.data.as_slice())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
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
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: VecDeque<u8>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let (first, second) = self.data.as_slices();
    ///         encoder.encode_bytes_vectored(&[first, second])
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_bytes_vectored(self, _: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Bytes,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: String,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_string(self.data.as_str())
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_string(self, _: &str) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::String,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some().and_then(|e| Encode::<M>::encode(data, e))
    ///             }
    ///             None => {
    ///                 encoder.encode_none()
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_some(self) -> Result<Self::Some, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Option,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::{Encode, Encoder, Mode};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some().and_then(|e| Encode::<M>::encode(data, e))
    ///             }
    ///             None => {
    ///                 encoder.encode_none()
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
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
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    /// use musli::mode::Mode;
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut pack = encoder.encode_pack()?;
    ///         pack.next()?.encode_u32(self.field)?;
    ///         pack.next()?.encode_array(self.data)?;
    ///         pack.end()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
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
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    /// use musli::mode::Mode;
    ///
    /// struct MyType {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<M> Encode<M> for MyType where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut seq = encoder.encode_sequence(self.data.len())?;
    ///
    ///         for element in &self.data {
    ///             seq.push::<M, _>(element)?;
    ///         }
    ///
    ///         seq.end()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence(self, #[allow(unused)] len: usize) -> Result<Self::Sequence, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
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
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    /// use musli::mode::Mode;
    ///
    /// struct PackedTuple(u32, [u8; 364]);
    ///
    /// impl<M> Encode<M> for PackedTuple where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut tuple = encoder.encode_tuple(2)?;
    ///         tuple.next()?.encode_u32(self.0)?;
    ///         tuple.next()?.encode_array(self.1)?;
    ///         tuple.end()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_tuple(self, #[allow(unused)] len: usize) -> Result<Self::Tuple, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Tuple,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a map with a known length `len`.
    ///
    ///
    #[inline]
    fn encode_map(self, #[allow(unused)] len: usize) -> Result<Self::Map, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Map,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder, PairEncoder, PairsEncoder};
    /// use musli::mode::Mode;
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct where M: Mode {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut st = encoder.encode_struct(2)?;
    ///         st.insert::<M, _, _>("name", &self.name)?;
    ///         st.insert::<M, _, _>("age", self.age)?;
    ///         st.end()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
            &expecting::Struct,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an struct enum variant.
    ///
    /// # Examples
    ///
    /// ```
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
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut variant = encoder.encode_variant()?;
    ///
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 variant.insert::<M, _, _>("variant1", ())
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 variant.insert::<M, _, _>("variant2", data)
    ///             }
    ///             Enum::Variant { data, age } => {
    ///                 variant.tag()?.encode_string("variant3")?;
    ///
    ///                 let mut st = variant.variant()?.encode_struct(2)?;
    ///                 st.insert::<M, _, _>("data", data)?;
    ///                 st.insert::<M, _, _>("age", age)?;
    ///                 st.end()?;
    ///
    ///                 variant.end()
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_variant(self) -> Result<Self::Variant, Self::Error> {
        Err(Self::Error::message(expecting::invalid_type(
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
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
