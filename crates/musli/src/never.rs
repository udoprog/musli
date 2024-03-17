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
/// impl<C: ?Sized + Context> Decoder<'_, C> for MyDecoder where {
///     fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "32-bit unsigned integers")
///     }
///
///     fn decode_u32(self, cx: &C) -> Result<u32, C::Error> {
///         if self.0 == 42 {
///             return Ok(self.0);
///         }
///
///         Err(cx.custom("I do not have the answer..."))
///     }
/// }
/// ```
pub struct Never<C = NeverMarker, B: ?Sized = NeverMarker> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<(C, B)>,
}

impl<'de, C: ?Sized + Context> Decoder<'de, C> for Never {
    type Decoder<U> = Self
    where
        U: Context;
    type Buffer = Self;
    type Pack = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Some = Self;
    type Struct = Self;
    type Variant = Self;
    type __UseMusliDecoderAttributeMacro = ();

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::Decoder<U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<C: ?Sized + Context> AsDecoder<C> for Never {
    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> StructFieldDecoder<'de, C> for Never {
    type FieldName<'this> = Self
    where
        Self: 'this;

    type FieldValue = Self;

    #[inline]
    fn field_name(&mut self, _: &C) -> Result<Self::FieldName<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn field_value(self, _: &C) -> Result<Self::FieldValue, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_field_value(self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapPairsDecoder<'de, C> for Never {
    type MapPairsKey<'this> = Self
    where
        Self: 'this;

    type MapPairsValue<'this> = Self where Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Option<Self::MapPairsKey<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_map_pairs_value(&mut self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> StructDecoder<'de, C> for Never {
    type Field<'this> = Self
    where
        Self: 'this;

    type StructPairs = Self;

    type __UseMusliStructDecoderAttributeMacro = ();

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn into_struct_pairs(self, _: &C) -> Result<Self::StructPairs, C::Error> {
        match self._never {}
    }

    #[inline]
    fn field(&mut self, _: &C) -> Result<Option<Self::Field<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> StructPairsDecoder<'de, C> for Never {
    type FieldName<'this> = Self
    where
        Self: 'this;

    type FieldValue<'this> = Self where Self: 'this;

    #[inline]
    fn field_name(&mut self, _: &C) -> Result<Self::FieldName<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn field_value(&mut self, _: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_field_value(&mut self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> VariantDecoder<'de, C> for Never {
    type Tag<'this> = Self
    where
        Self: 'this;

    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn variant(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_variant(&mut self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapDecoder<'de, C> for Never {
    type Entry<'this> = Self
    where
        Self: 'this;

    type MapPairs = Self;

    type __UseMusliMapDecoderAttributeMacro = ();

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn into_map_pairs(self, _: &C) -> Result<Self::MapPairs, C::Error> {
        match self._never {}
    }

    #[inline]
    fn entry(&mut self, _: &C) -> Result<Option<Self::Entry<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> MapEntryDecoder<'de, C> for Never {
    type MapKey<'this> = Self
    where
        Self: 'this;

    type MapValue = Self;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn map_value(self, _: &C) -> Result<Self::MapValue, C::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_map_value(self, _: &C) -> Result<bool, C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> SequenceDecoder<'de, C> for Never {
    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self, _: &C) -> SizeHint {
        match self._never {}
    }

    #[inline]
    fn next(&mut self, _: &C) -> Result<Option<Self::Decoder<'_>>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<'de, C: ?Sized + Context> PackDecoder<'de, C> for Never {
    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self, _: &C) -> Result<Self::Decoder<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<(), C::Error> {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> Encoder<C> for Never<O> {
    type Ok = O;
    type Encoder<U> = Self where U: Context;
    type Pack<'this> = Self where C: 'this;
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
    fn with_context<U>(self, _: &C) -> Result<Self::Encoder<U>, C::Error>
    where
        U: Context,
    {
        match self._never {}
    }

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, O, C: ?Sized + Context> NumberVisitor<'de, C> for Never<O> {
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<'de, O, C, T> ValueVisitor<'de, C, T> for Never<O, T>
where
    C: ?Sized + Context,
    T: ?Sized + ToOwned,
{
    type Ok = O;

    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> SequenceEncoder<C> for Never<O> {
    type Ok = O;

    type Encoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self, _: &C) -> Result<Self::Encoder<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> MapEncoder<C> for Never<O> {
    type Ok = O;
    type Entry<'this> = Self where Self: 'this;

    #[inline]
    fn entry(&mut self, _: &C) -> Result<Self::Entry<'_>, C::Error> {
        match self._never {}
    }

    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> MapEntryEncoder<C> for Never<O> {
    type Ok = O;
    type MapKey<'this> = Self
    where
        Self: 'this;
    type MapValue<'this> = Self where Self: 'this;

    #[inline]
    fn map_key(&mut self, _: &C) -> Result<Self::MapKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn map_value(&mut self, _: &C) -> Result<Self::MapValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> MapPairsEncoder<C> for Never<O> {
    type Ok = O;
    type MapPairsKey<'this> = Self
    where
        Self: 'this;
    type MapPairsValue<'this> = Self where Self: 'this;

    #[inline]
    fn map_pairs_key(&mut self, _: &C) -> Result<Self::MapPairsKey<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn map_pairs_value(&mut self, _: &C) -> Result<Self::MapPairsValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> StructEncoder<C> for Never<O> {
    type Ok = O;
    type Field<'this> = Self where Self: 'this;

    #[inline]
    fn field(&mut self, _: &C) -> Result<Self::Field<'_>, C::Error> {
        match self._never {}
    }

    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> StructFieldEncoder<C> for Never<O> {
    type Ok = O;
    type FieldName<'this> = Self
    where
        Self: 'this;
    type FieldValue<'this> = Self where Self: 'this;

    #[inline]
    fn field_name(&mut self, _: &C) -> Result<Self::FieldName<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn field_value(&mut self, _: &C) -> Result<Self::FieldValue<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}

impl<O, C: ?Sized + Context> VariantEncoder<C> for Never<O> {
    type Ok = O;
    type Tag<'this> = Self
    where
        Self: 'this;
    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag(&mut self, _: &C) -> Result<Self::Tag<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn variant(&mut self, _: &C) -> Result<Self::Variant<'_>, C::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        match self._never {}
    }
}
