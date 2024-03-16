use core::fmt;
use core::marker::PhantomData;

use crate::en::Encode;
use crate::expecting::{self, Expecting};
use crate::Context;

/// Trait governing how to encode a sequence.
pub trait SequenceEncoder<C>
where
    C: Context,
{
    /// Result type of the encoder.
    type Ok;

    /// The encoder returned when advancing the sequence encoder.
    type Encoder<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Prepare encoding of the next element.
    #[must_use = "Encoder must be consumed"]
    fn next(&mut self, cx: &C) -> Result<Self::Encoder<'_>, C::Error>;

    /// Push an element into the sequence.
    #[inline]
    fn push<T>(&mut self, cx: &C, value: T) -> Result<(), C::Error>
    where
        T: Encode<C::Mode>,
    {
        let encoder = self.next(cx)?;
        value.encode(cx, encoder)?;
        Ok(())
    }

    /// End the sequence.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;
}

/// Encoder for a map.
pub trait MapEncoder<C>
where
    C: Context,
{
    /// Result type of the encoder.
    type Ok;

    /// Encode the next pair.
    type Entry<'this>: MapEntryEncoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Insert a pair immediately.
    #[inline]
    fn insert_entry<F, S>(&mut self, cx: &C, first: F, second: S) -> Result<(), C::Error>
    where
        Self: Sized,
        F: Encode<C::Mode>,
        S: Encode<C::Mode>,
    {
        self.entry(cx)?.insert_entry(cx, first, second)?;
        Ok(())
    }

    /// Encode the next pair.
    fn entry(&mut self, cx: &C) -> Result<Self::Entry<'_>, C::Error>;

    /// Finish encoding pairs.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;
}

/// Trait governing how to encode a map entry.
pub trait MapEntryEncoder<C>
where
    C: Context,
{
    /// Result type of the encoder.
    type Ok;

    /// The encoder returned when advancing the map encoder to encode the key.
    type MapKey<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// The encoder returned when advancing the map encoder to encode the value.
    type MapValue<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Insert the pair immediately.
    #[inline]
    fn insert_entry<K, V>(mut self, cx: &C, key: K, value: V) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        K: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        self.map_key(cx).and_then(|e| key.encode(cx, e))?;
        self.map_value(cx).and_then(|e| value.encode(cx, e))?;
        self.end(cx)
    }

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn map_key(&mut self, cx: &C) -> Result<Self::MapKey<'_>, C::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn map_value(&mut self, cx: &C) -> Result<Self::MapValue<'_>, C::Error>;

    /// Stop encoding this pair.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;
}

/// Trait governing how to encode a map entry.
///
/// This trait exists so that decoders can implement a mode that is compatible
/// with serde serialization.
///
/// If you do not intend to implement this, then serde compatibility for your
/// format might be degraded.
pub trait MapPairsEncoder<C>
where
    C: Context,
{
    /// Result type of the encoder.
    type Ok;

    /// The encoder returned when advancing the map encoder to encode the key.
    type MapPairsKey<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// The encoder returned when advancing the map encoder to encode the value.
    type MapPairsValue<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Insert the pair immediately.
    #[inline]
    fn map_pairs_insert<K, V>(&mut self, cx: &C, key: K, value: V) -> Result<(), C::Error>
    where
        K: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        self.map_pairs_key(cx).and_then(|e| key.encode(cx, e))?;
        self.map_pairs_value(cx).and_then(|e| value.encode(cx, e))?;
        Ok(())
    }

    /// Return the encoder for the key in the entry.
    #[must_use = "Encoders must be consumed"]
    fn map_pairs_key(&mut self, cx: &C) -> Result<Self::MapPairsKey<'_>, C::Error>;

    /// Return encoder for value in the entry.
    #[must_use = "Encoders must be consumed"]
    fn map_pairs_value(&mut self, cx: &C) -> Result<Self::MapPairsValue<'_>, C::Error>;

    /// Stop encoding this pair.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;
}

/// Encoder for a struct.
pub trait StructEncoder<C>
where
    C: Context,
{
    /// Result type of the encoder.
    type Ok;

    /// Encoder for the next struct field.
    type Field<'this>: StructFieldEncoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Insert a field immediately.
    #[inline]
    fn insert_field<F, V>(&mut self, cx: &C, field: F, value: V) -> Result<(), C::Error>
    where
        F: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        self.field(cx)?.insert_field(cx, field, value)?;
        Ok(())
    }

    /// Encode the next field.
    fn field(&mut self, cx: &C) -> Result<Self::Field<'_>, C::Error>;

    /// Finish encoding the struct.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;
}

/// Trait governing how to encode a sequence of pairs.
pub trait StructFieldEncoder<C>
where
    C: Context,
{
    /// Result type of the encoder.
    type Ok;

    /// The encoder returned when advancing the map encoder to encode the key.
    type FieldName<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// The encoder returned when advancing the map encoder to encode the value.
    type FieldValue<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Insert the pair immediately.
    #[inline]
    fn insert_field<N, V>(mut self, cx: &C, name: N, value: V) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        N: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        self.field_name(cx).and_then(|e| name.encode(cx, e))?;
        self.field_value(cx).and_then(|e| value.encode(cx, e))?;
        self.end(cx)
    }

    /// Return the encoder for the field in the struct.
    #[must_use = "Encoders must be consumed"]
    fn field_name(&mut self, cx: &C) -> Result<Self::FieldName<'_>, C::Error>;

    /// Return encoder for the field value in the struct.
    #[must_use = "Encoders must be consumed"]
    fn field_value(&mut self, cx: &C) -> Result<Self::FieldValue<'_>, C::Error>;

    /// Stop encoding this field.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;
}

/// Trait governing how to encode a variant.
pub trait VariantEncoder<C>
where
    C: Context,
{
    /// Result type of the encoder.
    type Ok;

    /// The encoder returned when advancing the map encoder to encode the key.
    type Tag<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// The encoder returned when advancing the map encoder to encode the value.
    type Variant<'this>: Encoder<C, Ok = Self::Ok>
    where
        Self: 'this;

    /// Insert the variant immediately.
    #[inline]
    fn insert_variant<T, V>(mut self, cx: &C, tag: T, value: V) -> Result<Self::Ok, C::Error>
    where
        Self: Sized,
        T: Encode<C::Mode>,
        V: Encode<C::Mode>,
    {
        tag.encode(cx, self.tag(cx)?)?;
        value.encode(cx, self.variant(cx)?)?;
        self.end(cx)
    }

    /// Return the encoder for the first element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn tag(&mut self, cx: &C) -> Result<Self::Tag<'_>, C::Error>;

    /// Return encoder for the second element in the variant.
    #[must_use = "Encoders must be consumed"]
    fn variant(&mut self, cx: &C) -> Result<Self::Variant<'_>, C::Error>;

    /// End the variant encoder.
    fn end(self, cx: &C) -> Result<Self::Ok, C::Error>;
}

/// Trait governing how the encoder works.
pub trait Encoder<C>: Sized
where
    C: Context,
{
    /// The type returned by the encoder. For [Encode] implementations ensures
    /// that they are used correctly, since only functions returned by the
    /// [Encoder] is capable of returning this value.
    type Ok;
    /// Constructed [`Encoder`] with a different context.
    type Encoder<U>: Encoder<U, Ok = Self::Ok>
    where
        U: Context;
    /// Encoder returned when encoding an optional value which is present.
    type Some: Encoder<C, Ok = Self::Ok>;
    /// A simple pack that packs a sequence of elements.
    type Pack<'this>: SequenceEncoder<C, Ok = Self::Ok>
    where
        C: 'this;
    /// The type of a sequence encoder.
    type Sequence: SequenceEncoder<C, Ok = Self::Ok>;
    /// The type of a tuple encoder.
    type Tuple: SequenceEncoder<C, Ok = Self::Ok>;
    /// The type of a map encoder.
    type Map: MapEncoder<C, Ok = Self::Ok>;
    /// Streaming encoder for map pairs.
    type MapPairs: MapPairsEncoder<C, Ok = Self::Ok>;
    /// Encoder that can encode a struct.
    type Struct: StructEncoder<C, Ok = Self::Ok>;
    /// Encoder for a struct variant.
    type Variant: VariantEncoder<C, Ok = Self::Ok>;
    /// Specialized encoder for a tuple variant.
    type TupleVariant: SequenceEncoder<C, Ok = Self::Ok>;
    /// Specialized encoder for a struct variant.
    type StructVariant: StructEncoder<C, Ok = Self::Ok>;

    /// This is a type argument used to hint to any future implementor that they
    /// should be using the [`#[musli::encoder]`][crate::encoder] attribute
    /// macro when implementing [`Encoder`].
    #[doc(hidden)]
    type __UseMusliEncoderAttributeMacro;

    /// Construct an encoder with a different context.
    fn with_context<U>(self, cx: &C) -> Result<Self::Encoder<U>, C::Error>
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<M> Encode<M> for MyType {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
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
    ///         C: Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some(cx).and_then(|e| data.encode(cx, e))
    ///             }
    ///             None => {
    ///                 encoder.encode_none(cx)
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_some(self, cx: &C) -> Result<Self::Some, C::Error> {
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
    ///         C: Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         match &self.data {
    ///             Some(data) => {
    ///                 encoder.encode_some(cx).and_then(|e| data.encode(cx, e))
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
    ///     data: [u8; 364],
    /// }
    ///
    /// impl<M> Encode<M> for PackedStruct {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut pack = encoder.encode_pack(cx)?;
    ///         pack.next(cx)?.encode_u32(cx, self.field)?;
    ///         pack.next(cx)?.encode_array(cx, self.data)?;
    ///         pack.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_pack(self, cx: &C) -> Result<Self::Pack<'_>, C::Error> {
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
    ///         C: Context<Mode = M>,
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
    ) -> Result<Self::Sequence, C::Error> {
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
    /// struct PackedTuple(u32, [u8; 364]);
    ///
    /// impl<M> Encode<M> for PackedTuple {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut tuple = encoder.encode_tuple(cx, 2)?;
    ///         tuple.next(cx)?.encode_u32(cx, self.0)?;
    ///         tuple.next(cx)?.encode_array(cx, self.1)?;
    ///         tuple.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_tuple(self, cx: &C, #[allow(unused)] len: usize) -> Result<Self::Tuple, C::Error> {
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
    ///         C: Context<Mode = M>,
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
    fn encode_map(self, cx: &C, #[allow(unused)] len: usize) -> Result<Self::Map, C::Error> {
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
    /// use musli::en::{Encode, Encoder, MapPairsEncoder};
    ///
    /// struct Struct {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// impl<M> Encode<M> for Struct {
    ///     fn encode<C, E>(&self, cx: &C, encoder: E) -> Result<E::Ok, C::Error>
    ///     where
    ///         C: Context<Mode = M>,
    ///         E: Encoder<C>,
    ///     {
    ///         let mut m = encoder.encode_map_pairs(cx, 2)?;
    ///         m.map_pairs_insert(cx, "name", &self.name)?;
    ///         m.map_pairs_insert(cx, "age", self.age)?;
    ///         m.end(cx)
    ///     }
    /// }
    /// ```
    #[inline]
    fn encode_map_pairs(
        self,
        cx: &C,
        #[allow(unused)] len: usize,
    ) -> Result<Self::MapPairs, C::Error> {
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
    ///         C: Context<Mode = M>,
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
    fn encode_struct(self, cx: &C, _: usize) -> Result<Self::Struct, C::Error> {
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
    ///         C: Context<Mode = M>,
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
    ///                 variant.tag(cx)?.encode_string(cx, "variant3")?;
    ///
    ///                 let mut st = variant.variant(cx)?.encode_struct(cx, 2)?;
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
    fn encode_variant(self, cx: &C) -> Result<Self::Variant, C::Error> {
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
    ///         C: Context<Mode = M>,
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
        let t = variant.tag(cx)?;
        Encode::encode(tag, cx, t)?;
        let v = variant.variant(cx)?;
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
    ///         C: Context<Mode = M>,
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
    ) -> Result<Self::TupleVariant, C::Error>
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
    ///         C: Context<Mode = M>,
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
    ) -> Result<Self::StructVariant, C::Error>
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
struct ExpectingWrapper<T, C> {
    inner: T,
    _marker: PhantomData<C>,
}

impl<T, C> ExpectingWrapper<T, C> {
    #[inline]
    const fn new(inner: T) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<T, C> Expecting for ExpectingWrapper<T, C>
where
    T: Encoder<C>,
    C: Context,
{
    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(f)
    }
}
