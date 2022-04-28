use crate::no_std::ToOwned;
use core::borrow::Borrow;
use core::fmt;

use crate::error::Error;
use crate::expecting::{self, BadVisitorType, Expecting, InvalidType};

/// A visitor for data where it might be possible to borrow it without copying
/// from the underlying [Decoder].
///
/// A visitor is required with [Decoder::decode_bytes] and
/// [Decoder::decode_string] because the caller doesn't know if the encoding
/// format is capable of producing references to the underlying data directly or
/// if it needs to be processed.
///
/// By requiring a visitor we ensure that the caller has to handle both
/// scenarios, even if one involves erroring. A type like
/// [Cow][std::borrow::Cow] is an example of a type which can comfortably handle
/// both.
pub trait ValueVisitor<'de>: Sized {
    /// The value being visited.
    type Target: ?Sized + ToOwned;
    /// The value produced.
    type Ok;
    /// The error produced.
    type Error: Error;

    /// Format an error indicating what was expected by this visitor.
    ///
    /// Override to be more specific about the type that failed.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Visit an owned value.
    #[inline]
    fn visit_owned(self, value: <Self::Target as ToOwned>::Owned) -> Result<Self::Ok, Self::Error> {
        self.visit_any(value.borrow())
    }

    /// Visit a string that is borrowed directly from the source data.
    #[inline]
    fn visit_borrowed(self, value: &'de Self::Target) -> Result<Self::Ok, Self::Error> {
        self.visit_any(value)
    }

    /// Visit a string that is provided from the decoder in any manner possible.
    /// Which might require additional decoding work.
    #[inline]
    fn visit_any(self, _: &Self::Target) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::message(BadVisitorType::new(
            expecting::AnyValue,
            &ReferenceVisistorExpecting(self),
        )))
    }
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
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error>;
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
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next element.
    #[must_use = "decoders must be consumed"]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error>;
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
    fn size_hint(&self) -> Option<usize>;

    /// Decode the next key. This returns `Ok(None)` where there are no more elements to decode.
    #[must_use = "decoders must be consumed"]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error>;
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
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error>;

    /// Decode the second value in the pair..
    #[must_use = "decoders must be consumed"]
    fn second(self) -> Result<Self::Second, Self::Error>;

    /// Indicate that the second value should be skipped.
    ///
    /// The boolean returned indicates if the value was skipped or not.
    fn skip_second(self) -> Result<bool, Self::Error>;
}

/// Trait governing the implementation of a decoder.
pub trait Decoder<'de>: Sized {
    /// Error type raised by the decoder.
    type Error: Error;
    /// Pack decoder implementation.
    type Pack: PackDecoder<'de, Error = Self::Error>;
    /// Sequence decoder implementation.
    type Sequence: SequenceDecoder<'de, Error = Self::Error>;
    /// Tuple decoder implementation.
    type Tuple: PackDecoder<'de, Error = Self::Error>;
    /// Map decoder implementation.
    type Map: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a value that is present.
    type Some: Decoder<'de, Error = Self::Error>;
    /// Decoder for a struct.
    ///
    /// The caller receives a [PairsDecoder] which when advanced with
    /// [PairsDecoder::next] indicates the fields of the structure.
    type Struct: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a tuple struct.
    ///
    /// The caller receives a [PairsDecoder] which when advanced with
    /// [PairsDecoder::next] indicates the elements in the tuple.
    type TupleStruct: PairsDecoder<'de, Error = Self::Error>;
    /// Decoder for a variant.
    ///
    /// The caller receives a [PairDecoder] which when advanced with
    /// [PairDecoder::first] indicates which variant is being decoded and
    /// [PairDecoder::second] is the content of the variant.
    type Variant: PairDecoder<'de, Error = Self::Error>;

    /// Format the human-readable message that should occur if the decoder was
    /// expecting to decode some specific kind of value.
    ///
    /// ```
    /// #![feature(generic_associated_types)]
    ///
    /// use std::fmt;
    ///
    /// use musli::de::Decoder;
    /// use musli::never::Never;
    ///
    /// struct MyDecoder;
    ///
    /// impl Decoder<'_> for MyDecoder {
    ///     type Error = String;
    ///     type Pack = Never<Self>;
    ///     type Sequence = Never<Self>;
    ///     type Tuple = Never<Self>;
    ///     type Map = Never<Self>;
    ///     type Some = Never<Self>;
    ///     type Struct = Never<Self>;
    ///     type TupleStruct = Never<Self>;
    ///     type Variant = Never<Self>;
    ///
    ///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "32-bit unsigned integers")
    ///     }
    ///
    ///     fn decode_u32(self) -> Result<u32, Self::Error> {
    ///         Ok(42)
    ///     }
    /// }
    /// ```
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Decode a unit or something that is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct UnitType;
    ///
    /// impl<'de> Decode<'de> for UnitType {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_unit()?;
    ///         Ok(UnitType)
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_unit(self) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Unit,
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
    /// use musli::de::{Decode, Decoder, PackDecoder};
    ///
    /// struct PackedStruct {
    ///     field: u32,
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<'de> Decode<'de> for PackedStruct {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut unpack = decoder.decode_pack()?;
    ///         let field = unpack.next().and_then(Decode::decode)?;
    ///         let data = unpack.next().and_then(Decode::decode)?;
    ///
    ///         Ok(Self {
    ///             field,
    ///             data,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_pack(self) -> Result<Self::Pack, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Pack,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a fixed-length array.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: [u8; 128],
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Array,
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
    /// use musli::de::{Decode, Decoder, ValueVisitor};
    /// use musli::error::Error;
    ///
    /// struct BytesReference<'de> {
    ///     data: &'de [u8],
    /// }
    ///
    /// impl<'de> Decode<'de> for BytesReference<'de> {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         return Ok(Self {
    ///             data: decoder.decode_bytes(Visitor(marker::PhantomData))?,
    ///         });
    ///
    ///         struct Visitor<E>(marker::PhantomData<E>);
    ///
    ///         impl<'de, E> ValueVisitor<'de> for Visitor<E>
    ///         where
    ///             E: Error,
    ///         {
    ///             type Target = [u8];
    ///             type Ok = &'de [u8];
    ///             type Error = E;
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
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bytes<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        Err(Self::Error::message(InvalidType::new(
            expecting::Bytes,
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
    /// use musli::de::{Decode, Decoder, ValueVisitor};
    /// use musli::error::Error;
    ///
    /// struct StringReference<'de> {
    ///     data: &'de str,
    /// }
    ///
    /// impl<'de> Decode<'de> for StringReference<'de> {
    ///     #[inline]
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         return Ok(Self {
    ///             data: decoder.decode_string(Visitor(marker::PhantomData))?,
    ///         });
    ///
    ///         struct Visitor<E>(marker::PhantomData<E>);
    ///
    ///         impl<'de, E> ValueVisitor<'de> for Visitor<E>
    ///         where
    ///             E: Error,
    ///         {
    ///             type Target = str;
    ///             type Ok = &'de str;
    ///             type Error = E;
    ///
    ///             #[inline]
    ///             fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///                 write!(f, "exact bytes reference")
    ///             }
    ///
    ///             #[inline]
    ///             fn visit_borrowed(self, bytes: &'de str) -> Result<Self::Ok, Self::Error> {
    ///                 Ok(bytes)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_string<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ValueVisitor<'de, Target = str, Error = Self::Error>,
    {
        Err(Self::Error::message(InvalidType::new(
            expecting::String,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a boolean.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: bool,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         Ok(Self {
    ///             data: decoder.decode_bool()?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_bool(self) -> Result<bool, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Bool,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a character.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: char,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Char,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 8-bit unsigned integer (a.k.a. a byte).
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u8,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned8,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 16-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u16,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned16,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u32,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u64,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 128-bit unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: u128,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Unsigned128,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 8-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i8,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed8,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 16-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i16,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed16,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i32,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i64,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 128-bit signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: i128,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Signed128,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode Rusts [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: usize,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Usize,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode Rusts [`isize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: isize,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Isize,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 32-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: f32,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Float32,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a 64-bit floating point value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: f64,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
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
        Err(Self::Error::message(InvalidType::new(
            expecting::Float64,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode an optional value.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct MyType {
    ///     data: Option<String>,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let data = if let Some(decoder) = decoder.decode_option()? {
    ///             Some(String::decode(decoder)?)
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
    fn decode_option(self) -> Result<Option<Self::Some>, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Option,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder, SequenceDecoder};
    ///
    /// struct MyType {
    ///     data: Vec<String>,
    /// }
    ///
    /// impl<'de> Decode<'de> for MyType {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut seq = decoder.decode_sequence()?;
    ///         let mut data = Vec::new();
    ///
    ///         while let Some(decoder) = seq.next()? {
    ///             data.push(String::decode(decoder)?);
    ///         }
    ///
    ///         Ok(Self {
    ///             data
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Sequence,
            &ExpectingWrapper(self),
        )))
    }

    /// Return a helper to decode a tuple.
    ///
    /// A tuple is a fixed-length sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::de::{Decode, Decoder, PackDecoder};
    /// use musli::error::Error;
    ///
    /// struct TupleStruct(String, u32);
    ///
    /// impl<'de> Decode<'de> for TupleStruct {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut tuple = decoder.decode_tuple(2)?;
    ///         let string = tuple.next().and_then(String::decode)?;
    ///         let integer = tuple.next().and_then(u32::decode)?;
    ///         Ok(Self(string, integer))
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Tuple,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a map.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::de::{Decode, Decoder, PairsDecoder, PairDecoder};
    ///
    /// struct MapStruct {
    ///     data: HashMap<String, u32>,
    /// }
    ///
    /// impl<'de> Decode<'de> for MapStruct {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut map = decoder.decode_map()?;
    ///         let mut data = HashMap::with_capacity(map.size_hint().unwrap_or_default());
    ///
    ///         while let Some(mut entry) = map.next()? {
    ///             let key = entry.first().and_then(String::decode)?;
    ///             let value = entry.second().and_then(u32::decode)?;
    ///             data.insert(key, value);
    ///         }
    ///
    ///         Ok(Self {
    ///             data
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Map,
            &ExpectingWrapper(self),
        )))
    }

    /// Return a helper to decode a struct with named fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::de::{Decode, Decoder, PairsDecoder, PairDecoder};
    /// use musli::error::Error;
    ///
    /// struct Struct {
    ///     string: String,
    ///     integer: u32,
    /// }
    ///
    /// impl<'de> Decode<'de> for Struct {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut st = decoder.decode_struct(2)?;
    ///         let mut string = None;
    ///         let mut integer = None;
    ///
    ///         while let Some(mut entry) = st.next()? {
    ///             // Note: to avoid allocating `decode_string` needs to be used with a visitor.
    ///             let tag = entry.first().and_then(String::decode)?;
    ///
    ///             match tag.as_str() {
    ///                 "string" => {
    ///                     string = Some(entry.second().and_then(String::decode)?);
    ///                 }
    ///                 "integer" => {
    ///                     integer = Some(entry.second().and_then(u32::decode)?);
    ///                 }
    ///                 tag => {
    ///                     return Err(D::Error::invalid_field_tag("Struct", tag))
    ///                 }
    ///             }
    ///         }
    ///
    ///         Ok(Self {
    ///             string: string.ok_or_else(|| D::Error::expected_tag("Struct", "string"))?,
    ///             integer: integer.ok_or_else(|| D::Error::expected_tag("Struct", "integer"))?,
    ///         })
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Struct,
            &ExpectingWrapper(self),
        )))
    }

    /// Return a helper to decode a tuple struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use musli::de::{Decode, Decoder, PairsDecoder, PairDecoder};
    /// use musli::error::Error;
    ///
    /// struct TupleStruct(String, u32);
    ///
    /// impl<'de> Decode<'de> for TupleStruct {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut st = decoder.decode_tuple_struct(2)?;
    ///         let mut string = None;
    ///         let mut integer = None;
    ///
    ///         while let Some(mut entry) = st.next()? {
    ///             let tag = entry.first().and_then(usize::decode)?;
    ///
    ///             match tag {
    ///                 0 => {
    ///                     string = Some(entry.second().and_then(String::decode)?);
    ///                 }
    ///                 1 => {
    ///                     integer = Some(entry.second().and_then(u32::decode)?);
    ///                 }
    ///                 tag => {
    ///                     return Err(D::Error::invalid_field_tag("Struct", tag))
    ///                 }
    ///             }
    ///         }
    ///
    ///         let string = string.ok_or_else(|| D::Error::expected_tag("Struct", "string"))?;
    ///         let integer = integer.ok_or_else(|| D::Error::expected_tag("Struct", "integer"))?;
    ///
    ///         Ok(Self(string, integer))
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_tuple_struct(self, _: usize) -> Result<Self::TupleStruct, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::TupleStruct,
            &ExpectingWrapper(self),
        )))
    }

    /// Decode a unit variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder};
    ///
    /// struct UnitStruct;
    ///
    /// impl<'de> Decode<'de> for UnitStruct {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         decoder.decode_unit_struct()?;
    ///         Ok(Self)
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_unit_struct(self) -> Result<(), Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::UnitStruct,
            &ExpectingWrapper(self),
        )))
    }

    /// Return decoder for a variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use musli::de::{Decode, Decoder, PairsDecoder, PairDecoder};
    /// use musli::error::Error;
    ///
    /// enum Enum {
    ///     Variant1(u32),
    ///     Variant2(String),
    /// }
    ///
    /// impl<'de> Decode<'de> for Enum {
    ///     fn decode<D>(decoder: D) -> Result<Self, D::Error>
    ///     where
    ///         D: Decoder<'de>,
    ///     {
    ///         let mut variant = decoder.decode_variant()?;
    ///         let tag = variant.first().and_then(usize::decode)?;
    ///
    ///         match tag {
    ///             0 => {
    ///                 Ok(Self::Variant1(variant.second().and_then(u32::decode)?))
    ///             }
    ///             1 => {
    ///                 Ok(Self::Variant2(variant.second().and_then(String::decode)?))
    ///             }
    ///             tag => {
    ///                 Err(D::Error::invalid_variant_tag("Enum", tag))
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
        Err(Self::Error::message(InvalidType::new(
            expecting::Variant,
            &ExpectingWrapper(self),
        )))
    }
}

#[repr(transparent)]
struct ExpectingWrapper<T>(T);

impl<'de, T> Expecting for ExpectingWrapper<T>
where
    T: Decoder<'de>,
{
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}

#[repr(transparent)]
struct ReferenceVisistorExpecting<T>(T);

impl<'de, T> Expecting for ReferenceVisistorExpecting<T>
where
    T: ValueVisitor<'de>,
{
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
}
