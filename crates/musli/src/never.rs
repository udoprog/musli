//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.

use core::{fmt, marker};

use crate::de::{
    AsDecoder, Decoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, VariantDecoder,
};
use crate::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use crate::error::Error;

enum NeverMarker {}

/// An uninhabitable never type which implements all possible encoders and
/// decoders. This can be used if your [Encoder] implementation doesn't
/// implement a particular function.
///
/// ```
/// use std::fmt;
///
/// use musli::de::Decoder;
/// use musli::never::Never;
///
/// struct MyDecoder;
///
/// impl Decoder<'_> for MyDecoder {
///     type Error = String;
///     type Buffer = Never<Self::Error>;
///     type Pack = Never<Self::Error>;
///     type Sequence = Never<Self::Error>;
///     type Tuple = Never<Self::Error>;
///     type Map = Never<Self::Error>;
///     type Some = Never<Self::Error>;
///     type Struct = Never<Self::Error>;
///     type Variant = Never<Self::Error>;
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
pub struct Never<T> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<T>,
}

impl<'de, E> Decoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;
    type Buffer = Self;
    type Pack = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Some = Self;
    type Struct = Self;
    type Variant = Self;

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<E> AsDecoder for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Never<E>
    where
        Self: 'this;

    #[inline]
    fn as_decoder(&self) -> Result<Self::Decoder<'_>, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> PairDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type First<'this> = Self
    where
        Self: 'this;

    type Second = Self;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn second(self) -> Result<Self::Second, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_second(self) -> Result<bool, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> VariantDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Tag<'this> = Self
    where
        Self: 'this;

    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_variant(&mut self) -> Result<bool, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<'de, E> PairsDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self._never {}
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<'de, E> SequenceDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self._never {}
    }

    #[inline]
    fn next(&mut self) -> Result<Option<Self::Decoder<'_>>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<'de, E> PackDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Decoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<T> Encoder for Never<T>
where
    T: Encoder,
{
    type Ok = T::Ok;
    type Error = T::Error;
    type Pack = Self;
    type Some = Self;
    type Sequence = Self;
    type Tuple = Self;
    type Map = Self;
    type Struct = Self;
    type Variant = Self;

    #[inline]
    fn expecting(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<T> SequenceEncoder for Never<T>
where
    T: Encoder,
{
    type Ok = T::Ok;
    type Error = T::Error;

    type Encoder<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<T> PairsEncoder for Never<T>
where
    T: Encoder,
{
    type Ok = T::Ok;
    type Error = T::Error;
    type Encoder<'this> = Self where Self: 'this;

    #[inline]
    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        match self._never {}
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<T> PairEncoder for Never<T>
where
    T: Encoder,
{
    type Ok = T::Ok;
    type Error = T::Error;
    type First<'this> = Self
    where
        Self: 'this;
    type Second<'this> = Self where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}

impl<T> VariantEncoder for Never<T>
where
    T: Encoder,
{
    type Ok = T::Ok;
    type Error = T::Error;
    type Tag<'this> = Self
    where
        Self: 'this;
    type Variant<'this> = Self where Self: 'this;

    #[inline]
    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self._never {}
    }
}
