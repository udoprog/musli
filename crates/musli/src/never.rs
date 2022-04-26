//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.

use core::{fmt, marker};

use crate::de::{
    Decoder, MapDecoder, MapEntryDecoder, PackDecoder, PairDecoder, SequenceDecoder, StructDecoder,
};
use crate::en::{Encoder, PackEncoder, PairEncoder, SequenceEncoder};
use crate::error::Error;

enum NeverMarker {}

/// A never type which implements all decoder functions so it can be used as a
/// type for a no-op implementation.
pub struct Never<E> {
    // Field makes type uninhabitable.
    _never: NeverMarker,
    _marker: marker::PhantomData<E>,
}

impl<'de, E> Decoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;
    type Pack = Self;
    type Sequence = Self;
    type Map = Self;
    type Some = Self;
    type Struct = Self;
    type Tuple = Self;
    type Variant = Self;

    #[inline]
    fn expected(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn decode_first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_second(self) -> Result<Self::Second, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn skip_second(self) -> Result<bool, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> StructDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Field<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self._never {}
    }

    #[inline]
    fn decode_field(&mut self) -> Result<Option<Self::Field<'_>>, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> MapDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Entry<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self._never {}
    }

    #[inline]
    fn decode_entry(&mut self) -> Result<Option<Self::Entry<'_>>, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> MapEntryDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Key<'this> = Self
    where
        Self: 'this;

    type Value<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn decode_key(&mut self) -> Result<Self::Key<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn decode_value(&mut self) -> Result<Self::Value<'_>, Self::Error> {
        match self._never {}
    }
}

impl<'de, E> SequenceDecoder<'de> for Never<E>
where
    E: Error,
{
    type Error = E;

    type Next<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self._never {}
    }

    #[inline]
    fn decode_next(&mut self) -> Result<Option<Self::Next<'_>>, Self::Error> {
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
    fn finish(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<E> Encoder for Never<E>
where
    E: Error,
{
    type Error = E;
    type Pack = Self;
    type Some = Self;
    type Sequence = Self;
    type Map = Self;
    type Struct = Self;
    type Tuple = Self;
    type Variant = Self;

    #[inline]
    fn expected(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self._never {}
    }
}

impl<E> PackEncoder for Never<E>
where
    E: Error,
{
    type Error = E;

    type Encoder<'this> = Self
    where
        Self: 'this;

    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        match self._never {}
    }

    fn finish(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<E> SequenceEncoder for Never<E>
where
    E: Error,
{
    type Error = E;

    type Next<'this> = Self
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::Next<'_>, Self::Error> {
        match self._never {}
    }

    #[inline]
    fn finish(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}

impl<E> PairEncoder for Never<E>
where
    E: Error,
{
    type Error = E;

    type First<'this> = Self
    where
        Self: 'this;

    type Second<'this> = Self
    where
        Self: 'this;

    fn encode_first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        match self._never {}
    }

    fn encode_second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        match self._never {}
    }

    fn finish(self) -> Result<(), Self::Error> {
        match self._never {}
    }
}
