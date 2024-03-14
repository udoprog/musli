//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.
//!
//! This is a private module of musli, and is not intended for use outside of
//! the implementation attributes:
//!
//! * [`#[musli::encoder]`][crate::encoder].
//! * [`#[musli::decoder]`][crate::decoder].

use core::fmt;
use core::marker;

use crate::no_std::ToOwned;

use crate::de::{
    AsDecoder, Decoder, MapDecoder, MapEntryDecoder, MapPairsDecoder, NumberVisitor, PackDecoder,
    SequenceDecoder, SizeHint, StructDecoder, StructFieldDecoder, StructPairsDecoder, ValueVisitor,
    VariantDecoder,
};
use crate::en::{
    Encoder, MapEncoder, MapEntryEncoder, MapPairsEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use crate::{Buf, Context};

/// An empty buffer.
pub enum NeverBuffer {}

impl Buf for NeverBuffer {
    #[inline(always)]
    fn write(&mut self, _: &[u8]) -> bool {
        false
    }

    #[inline(always)]
    fn len(&self) -> usize {
        0
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        &[]
    }
}

/// Marker type used for the [`Never`] type.
#[doc(hidden)]
pub enum NeverMarker {}

/// An uninhabitable never type which implements all possible encoders and
/// decoders. This can be used if your [Encoder] implementation doesn't
/// implement a particular function.
///
/// ```
/// use std::fmt;
///
/// use musli::Context;
/// use musli::de::Decoder;
///
/// struct MyDecoder(u32);
///
/// #[musli::decoder]
/// impl Decoder<'_> for MyDecoder {
///     type Error = String;
///
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32<C>(self, cx: &C) -> Result<u32, C::Error>
///     where
///         C: Context<Input = Self::Error>
///     {
///         if self.0 == 42 {
///             return Ok(self.0);
///         }
///
///         Err(cx.custom("I do not have the answer..."))
///     }
/// }
/// ```
pub struct Never<A, B: ?Sized = NeverMarker, C = NeverMarker> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<(A, C, B)>,
}

impl<'de, E: 'static> Decoder<'de> for Never<E> {
    type Error = E;
    type Buffer = Self;
    type Pack = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type MapPairs = Self;
    type Some = Self;
    type Struct = Self;
    type StructPairs = Self;
    type Variant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<E: 'static> AsDecoder for Never<E> {
    type Error = E;

    type Decoder<'this> = Never<E>
    where
        Self: 'this;

    #[inline]
    fn as_decoder<C>(&self, _: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> StructFieldDecoder<'de> for Never<E> {
    type Error = E;

    type FieldName<'this> = Self
    where
        Self: 'this;

    type FieldValue = Self;

    #[inline]
    fn field_name<C>(&mut self, _: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn field_value<C>(self, _: &C) -> Result<Self::FieldValue, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn skip_field_value<C>(self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> MapPairsDecoder<'de> for Never<E> {
    type Error = E;

    type MapPairsKey<'this> = Self
    where
        Self: 'this;

    type MapPairsValue<'this> = Self where Self: 'this;

    #[inline]
    fn map_pairs_key<C>(&mut self, _: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn map_pairs_value<C>(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn skip_map_pairs_value<C>(&mut self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> StructDecoder<'de> for Never<E> {
    type Error = E;

    type Field<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn field<C>(&mut self, _: &C) -> Result<Option<Self::Field<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> StructPairsDecoder<'de> for Never<E> {
    type Error = E;

    type FieldName<'this> = Self
    where
        Self: 'this;

    type FieldValue<'this> = Self where Self: 'this;

    #[inline]
    fn field_name<C>(&mut self, _: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn field_value<C>(&mut self, _: &C) -> Result<Self::FieldValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn skip_field_value<C>(&mut self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> VariantDecoder<'de> for Never<E> {
    type Error = E;

    type Tag<'this> = Self
    where
        Self: 'this;

    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn variant<C>(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn skip_variant<C>(&mut self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> MapDecoder<'de> for Never<E> {
    type Error = E;

    type Entry<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn entry<C>(&mut self, _: &C) -> Result<Option<Self::Entry<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> MapEntryDecoder<'de> for Never<E> {
    type Error = E;

    type MapKey<'this> = Self
    where
        Self: 'this;

    type MapValue = Self;

    #[inline]
    fn map_key<C>(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn map_value<C>(self, _: &C) -> Result<Self::MapValue, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn skip_map_value<C>(self, _: &C) -> Result<bool, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> SequenceDecoder<'de> for Never<E> {
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Option<Self::Decoder<'_>>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<'de, E: 'static> PackDecoder<'de> for Never<E> {
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Self::Decoder<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<(), C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E: 'static> Encoder for Never<O, E> {
    type Ok = O;
    type Error = E;
    type Pack<'this, C> = Self where C: 'this + Context;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type MapPairs = Self;
    type Struct = Self;
    type Variant = Self;
    type TupleVariant = Self;
    type StructVariant = Self;
    type __UseMusliEncoderAttributeMacro = ();

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, O, C> NumberVisitor<'de, C> for Never<O, C>
where
    C: Context,
{
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, O, T, C> ValueVisitor<'de, C, T> for Never<O, T, C>
where
    T: ?Sized + ToOwned,
    C: Context,
{
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<O, E: 'static> SequenceEncoder for Never<O, E> {
    type Ok = O;
    type Error = E;

    type Encoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next<C>(&mut self, _: &C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<Input = E>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = E>,
    {
        match self._never {}
    }
}

impl<O, E: 'static> MapEncoder for Never<O, E> {
    type Ok = O;
    type Error = E;
    type Entry<'this> = Self where Self: 'this;

    #[inline]
    fn entry<C>(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E: 'static> MapEntryEncoder for Never<O, E> {
    type Ok = O;
    type Error = E;
    type MapKey<'this> = Self
    where
        Self: 'this;
    type MapValue<'this> = Self where Self: 'this;

    #[inline]
    fn map_key<C>(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn map_value<C>(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E: 'static> MapPairsEncoder for Never<O, E> {
    type Ok = O;
    type Error = E;
    type MapPairsKey<'this> = Self
    where
        Self: 'this;
    type MapPairsValue<'this> = Self where Self: 'this;

    #[inline]
    fn map_pairs_key<C>(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn map_pairs_value<C>(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E: 'static> StructEncoder for Never<O, E> {
    type Ok = O;
    type Error = E;
    type Field<'this> = Self where Self: 'this;

    #[inline]
    fn field<C>(&mut self, _: &C) -> Result<Self::Field<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E: 'static> StructFieldEncoder for Never<O, E> {
    type Ok = O;
    type Error = E;
    type FieldName<'this> = Self
    where
        Self: 'this;
    type FieldValue<'this> = Self where Self: 'this;

    #[inline]
    fn field_name<C>(&mut self, _: &C) -> Result<Self::FieldName<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn field_value<C>(&mut self, _: &C) -> Result<Self::FieldValue<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}

impl<O, E: 'static> VariantEncoder for Never<O, E> {
    type Ok = O;
    type Error = E;
    type Tag<'this> = Self
    where
        Self: 'this;
    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag<C>(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn variant<C>(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }

    #[inline]
    fn end<C>(self, _: &C) -> Result<Self::Ok, C::Error>
    where
        C: Context<Input = Self::Error>,
    {
        match self._never {}
    }
}
