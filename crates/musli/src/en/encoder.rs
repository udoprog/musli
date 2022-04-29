use core::fmt;

use crate::en::Encode;
use crate::error::Error;
use crate::expecting::{self, Expecting, InvalidType};

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
    fn push<Mode, T>(&mut self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Mode>,
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
    fn insert<Mode, F, S>(&mut self, first: F, second: S) -> Result<(), Self::Error>
    where
        Self: Sized,
        F: Encode<Mode>,
        S: Encode<Mode>,
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
    fn insert<Mode, F, S>(mut self, first: F, second: S) -> Result<Self::Ok, Self::Error>
    where
        Self: Sized,
        F: Encode<Mode>,
        S: Encode<Mode>,
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

    /// End the pair encoder.
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
    /// A simple pack that packs a sequence of elements.
    type Pack: SequenceEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder returned when encoding an optional value which is present.
    type Some: Encoder<Ok = Self::Ok, Error = Self::Error>;
    /// The type of a sequence encoder.
    type Sequence: SequenceEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// The type of a tuple encoder.
    type Tuple: SequenceEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// The type of a map encoder.
    type Map: PairsEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder that can encode a struct.
    type Struct: PairsEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder that can encode a tuple struct.
    type TupleStruct: PairsEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder for a struct variant.
    type StructVariant: PairEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder for a tuple variant.
    type TupleVariant: PairEncoder<Ok = Self::Ok, Error = Self::Error>;
    /// Encoder for a unit variant.
    type UnitVariant: PairEncoder<Ok = Self::Ok, Error = Self::Error>;

    /// An expectation error. Every other implementation defers to this to
    /// report that something unexpected happened.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Encode a unit or something that is completely empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct EmptyStruct;
    ///
    /// impl<Mode> Encode<Mode> for EmptyStruct {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unit,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a boolean value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: bool,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Bool,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a character.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: char,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Char,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u8,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u16,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u32,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u64,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: u128,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i8,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed8,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i16,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed16,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i32,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i64,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: i128,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed128,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: usize,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Usize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode Rusts [`isize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: isize,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Isize,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: f32,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Float32,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: f64,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Float64,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode fixed-length array.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Array,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a sequence of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: Vec<u8>,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Bytes,
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
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: VecDeque<u8>,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Bytes,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: String,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::String,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is present.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some().and_then(|e| Encode::<Mode>::encode(data, e))
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Option,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an optional value that is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some().and_then(|e| Encode::<Mode>::encode(data, e))
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Option,
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
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<Mode> Encode<Mode> for PackedStruct {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Pack,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a sequence with a known length.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    ///
    /// struct MyType {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for MyType {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut seq = encoder.encode_sequence(self.data.len())?;
    ///
    ///         for element in &self.data {
    ///             seq.push::<Mode, _>(element)?;
    ///         }
    ///
    ///         seq.end()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_sequence(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Sequence,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a tuple.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder, SequenceEncoder};
    ///
    /// struct PackedTuple(u32, [u8; 364]);
    ///
    /// impl<Mode> Encode<Mode> for PackedTuple {
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
    fn encode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Tuple,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a map with a known length.
    #[inline]
    fn encode_map(self, _: usize) -> Result<Self::Map, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Map,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder, PairEncoder, PairsEncoder};
    ///
    /// struct TupleStruct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<Mode> Encode<Mode> for TupleStruct {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut st = encoder.encode_struct(2)?;
    ///         st.insert::<Mode, _, _>("name", &self.name)?;
    ///         st.insert::<Mode, _, _>("age", self.age)?;
    ///         st.end()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Struct,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a tuple struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder, PairEncoder, PairsEncoder};
    ///
    /// struct TupleStruct(String);
    ///
    /// impl<Mode> Encode<Mode> for TupleStruct {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         let mut tuple = encoder.encode_tuple_struct(1)?;
    ///         tuple.insert::<Mode, _, _>(0usize, &self.0)?;
    ///         tuple.end()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_tuple_struct(self, _: usize) -> Result<Self::TupleStruct, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::TupleStruct,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode a unit struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder};
    ///
    /// struct UnitStruct;
    ///
    /// impl<Mode> Encode<Mode> for UnitStruct {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         encoder.encode_unit_struct()
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_unit_struct(self) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::UnitStruct,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an struct enum variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::en::{Encode, Encoder, PairEncoder, PairsEncoder};
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
    /// impl<Mode> Encode<Mode> for Enum {
    ///     fn encode<E>(&self, encoder: E) -> Result<E::Ok, E::Error>
    ///     where
    ///         E: Encoder
    ///     {
    ///         match self {
    ///             Enum::UnitVariant => {
    ///                 let variant = encoder.encode_unit_variant()?;
    ///                 variant.insert::<Mode, _, _>("variant1", ())
    ///             }
    ///             Enum::TupleVariant(data) => {
    ///                 let variant = encoder.encode_tuple_variant(1)?;
    ///                 variant.insert::<Mode, _, _>("variant2", data)
    ///             }
    ///             Enum::StructVariant { data, age } => {
    ///                 let mut variant = encoder.encode_struct_variant(2)?;
    ///                 variant.first()?.encode_string("variant3")?;
    ///
    ///                 let mut st = variant.second()?.encode_struct(2)?;
    ///                 st.insert::<Mode, _, _>("data", data)?;
    ///                 st.insert::<Mode, _, _>("age", age)?;
    ///                 st.end()?;
    ///
    ///                 variant.end()
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_struct_variant(self, _: usize) -> Result<Self::StructVariant, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::StructVariant,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an tuple enum variant.
    #[inline]
    fn encode_tuple_variant(self, _: usize) -> Result<Self::TupleVariant, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::TupleVariant,
            &ExpectingWrapper::new(self),
        )))
    }

    /// Encode an unit enum variant.
    #[inline]
    fn encode_unit_variant(self) -> Result<Self::UnitVariant, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::UnitVariant,
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
