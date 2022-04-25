//! Module that provides a never type which conveniently implements all the
//! encoder and decoder traits so that it can be used as a placeholder.

use core::marker;

use crate::de::{
    Decoder, MapDecoder, MapEntryDecoder, PackDecoder, PairDecoder, ReferenceVisitor,
    SequenceDecoder, StructDecoder,
};
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

    fn decode_unit(self) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn decode_pack(self) -> Result<Self::Pack, Self::Error> {
        unreachable!()
    }

    fn decode_array<const N: usize>(self) -> Result<[u8; N], Self::Error> {
        unreachable!()
    }

    fn decode_bytes<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = [u8], Error = Self::Error>,
    {
        unreachable!()
    }

    fn decode_string<V>(self, _: V) -> Result<V::Ok, V::Error>
    where
        V: ReferenceVisitor<'de, Target = str, Error = Self::Error>,
    {
        unreachable!()
    }

    fn decode_bool(self) -> Result<bool, Self::Error> {
        unreachable!()
    }

    fn decode_char(self) -> Result<char, Self::Error> {
        unreachable!()
    }

    fn decode_u8(self) -> Result<u8, Self::Error> {
        unreachable!()
    }

    fn decode_u16(self) -> Result<u16, Self::Error> {
        unreachable!()
    }

    fn decode_u32(self) -> Result<u32, Self::Error> {
        unreachable!()
    }

    fn decode_u64(self) -> Result<u64, Self::Error> {
        unreachable!()
    }

    fn decode_u128(self) -> Result<u128, Self::Error> {
        unreachable!()
    }

    fn decode_i8(self) -> Result<i8, Self::Error> {
        unreachable!()
    }

    fn decode_i16(self) -> Result<i16, Self::Error> {
        unreachable!()
    }

    fn decode_i32(self) -> Result<i32, Self::Error> {
        unreachable!()
    }

    fn decode_i64(self) -> Result<i64, Self::Error> {
        unreachable!()
    }

    fn decode_i128(self) -> Result<i128, Self::Error> {
        unreachable!()
    }

    fn decode_usize(self) -> Result<usize, Self::Error> {
        unreachable!()
    }

    fn decode_isize(self) -> Result<isize, Self::Error> {
        unreachable!()
    }

    fn decode_f32(self) -> Result<f32, Self::Error> {
        unreachable!()
    }

    fn decode_f64(self) -> Result<f64, Self::Error> {
        unreachable!()
    }

    fn decode_option(self) -> Result<Option<Self::Some>, Self::Error> {
        unreachable!()
    }

    fn decode_sequence(self) -> Result<Self::Sequence, Self::Error> {
        unreachable!()
    }

    fn decode_map(self) -> Result<Self::Map, Self::Error> {
        unreachable!()
    }

    fn decode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        unreachable!()
    }

    fn decode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        unreachable!()
    }

    fn decode_unit_struct(self) -> Result<(), Self::Error> {
        unreachable!()
    }

    fn decode_variant(self) -> Result<Self::Variant, Self::Error> {
        unreachable!()
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

    fn decode_first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        unreachable!()
    }

    fn decode_second(self) -> Result<Self::Second, Self::Error> {
        unreachable!()
    }

    fn skip_second(self) -> Result<bool, Self::Error> {
        unreachable!()
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

    fn size_hint(&self) -> Option<usize> {
        unreachable!()
    }

    fn decode_field(&mut self) -> Result<Option<Self::Field<'_>>, Self::Error> {
        unreachable!()
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

    fn size_hint(&self) -> Option<usize> {
        unreachable!()
    }

    fn decode_entry(&mut self) -> Result<Option<Self::Entry<'_>>, Self::Error> {
        unreachable!()
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

    fn decode_key(&mut self) -> Result<Self::Key<'_>, Self::Error> {
        unreachable!()
    }

    fn decode_value(&mut self) -> Result<Self::Value<'_>, Self::Error> {
        unreachable!()
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

    fn size_hint(&self) -> Option<usize> {
        unreachable!()
    }

    fn decode_next(&mut self) -> Result<Option<Self::Next<'_>>, Self::Error> {
        unreachable!()
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

    fn next(&mut self) -> Result<Self::Decoder<'_>, Self::Error> {
        unreachable!()
    }

    fn finish(self) -> Result<(), Self::Error> {
        unreachable!()
    }
}
