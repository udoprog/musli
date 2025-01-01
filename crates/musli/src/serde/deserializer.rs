use core::any::TypeId;
use core::fmt;

use serde::de;

use crate::de::{
    Decoder, EntriesDecoder, MapDecoder, SequenceDecoder, SizeHint, VariantDecoder, Visitor,
};
use crate::hint::SequenceHint;
use crate::mode::Text;
use crate::Context;

#[cfg(feature = "alloc")]
use rust_alloc::string::String;
#[cfg(feature = "alloc")]
use rust_alloc::vec::Vec;

pub struct Deserializer<D> {
    decoder: D,
}

impl<D> Deserializer<D> {
    /// Construct a new deserializer out of a decoder.
    pub fn new(decoder: D) -> Self {
        Self { decoder }
    }
}

impl<'de, D> de::Deserializer<'de> for Deserializer<D>
where
    D: Decoder<'de, Cx: Context<Error: de::Error>>,
{
    type Error = <D::Cx as Context>::Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_any(AnyVisitor::new(visitor))
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_bool()?;
        visitor.visit_bool(value)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i8()?;
        visitor.visit_i8(value)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i16()?;
        visitor.visit_i16(value)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i32()?;
        visitor.visit_i32(value)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i64()?;
        visitor.visit_i64(value)
    }

    #[inline]
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i128()?;
        visitor.visit_i128(value)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u8()?;
        visitor.visit_u8(value)
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u16()?;
        visitor.visit_u16(value)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u32()?;
        visitor.visit_u32(value)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u64()?;
        visitor.visit_u64(value)
    }

    #[inline]
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u128()?;
        visitor.visit_u128(value)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_f32()?;
        visitor.visit_f32(value)
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_f64()?;
        visitor.visit_f64(value)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_char(self.decoder.decode_char()?)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_string(StringVisitor::new(visitor))
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_bytes(BytesVisitor::new(visitor))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.decoder.decode_option()? {
            Some(decoder) => visitor.visit_some(Deserializer::new(decoder)),
            None => visitor.visit_none(),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_empty()?;
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_empty()?;
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(Deserializer::new(self.decoder))
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder
            .decode_sequence(|d| visitor.visit_seq(SeqAccess::new(d)))
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let hint = SequenceHint::with_size(len);

        self.decoder
            .decode_sequence_hint(&hint, |d| visitor.visit_seq(SequenceAccess::new(d)))
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder
            .decode_map_entries(|decoder| visitor.visit_map(MapAccess::new(decoder)))
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder
            .decode_map_entries(|decoder| visitor.visit_map(StructAccess::new(decoder)))
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder
            .decode_variant(|decoder| visitor.visit_enum(EnumAccess::new(decoder)))
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.skip()?;
        visitor.visit_unit()
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        TypeId::of::<D::Mode>() == TypeId::of::<Text>()
    }
}

struct SequenceAccess<'a, D>
where
    D: ?Sized,
{
    decoder: &'a mut D,
}

impl<'a, D> SequenceAccess<'a, D>
where
    D: ?Sized,
{
    fn new(decoder: &'a mut D) -> Self {
        SequenceAccess { decoder }
    }
}

impl<'de, D> de::SeqAccess<'de> for SequenceAccess<'_, D>
where
    D: SequenceDecoder<'de, Cx: Context<Error: de::Error>>,
{
    type Error = <D::Cx as Context>::Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let Some(decoder) = self.decoder.try_decode_next()? else {
            return Ok(None);
        };

        let output = seed.deserialize(Deserializer::new(decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.decoder.size_hint().into_option()
    }
}
struct StructAccess<'a, D>
where
    D: ?Sized,
{
    decoder: &'a mut D,
}

impl<'a, D> StructAccess<'a, D>
where
    D: ?Sized,
{
    #[inline]
    fn new(decoder: &'a mut D) -> Self {
        StructAccess { decoder }
    }
}

impl<'de, D> de::MapAccess<'de> for StructAccess<'_, D>
where
    D: ?Sized + EntriesDecoder<'de, Cx: Context<Error: de::Error>>,
{
    type Error = <D::Cx as Context>::Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some(decoder) = self.decoder.decode_entry_key()? else {
            return Ok(None);
        };

        let output = seed.deserialize(Deserializer::new(decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let decoder = self.decoder.decode_entry_value()?;
        let output = seed.deserialize(Deserializer::new(decoder))?;
        Ok(output)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.decoder.size_hint().into_option()
    }
}

struct BytesVisitor<V> {
    visitor: V,
}

impl<V> BytesVisitor<V> {
    #[inline]
    fn new(visitor: V) -> Self {
        Self { visitor }
    }
}

impl<'de, C, V> crate::de::UnsizedVisitor<'de, C, [u8]> for BytesVisitor<V>
where
    C: Context<Error: de::Error>,
    V: de::Visitor<'de>,
{
    type Ok = V::Value;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(f)
    }

    #[inline]
    #[cfg(feature = "alloc")]
    fn visit_owned(self, _: C, value: Vec<u8>) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_byte_buf(self.visitor, value)
    }

    #[inline]
    fn visit_borrowed(self, _: C, value: &'de [u8]) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_borrowed_bytes(self.visitor, value)
    }

    #[inline]
    fn visit_ref(self, _: C, value: &[u8]) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_bytes(self.visitor, value)
    }
}

struct SeqAccess<'a, D>
where
    D: ?Sized,
{
    decoder: &'a mut D,
}

impl<'a, D> SeqAccess<'a, D>
where
    D: ?Sized,
{
    fn new(decoder: &'a mut D) -> Self {
        Self { decoder }
    }
}

impl<'de, D> de::SeqAccess<'de> for SeqAccess<'_, D>
where
    D: ?Sized + SequenceDecoder<'de, Cx: Context<Error: de::Error>>,
{
    type Error = <D::Cx as Context>::Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let Some(decoder) = self.decoder.try_decode_next()? else {
            return Ok(None);
        };

        let output = seed.deserialize(Deserializer::new(decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.decoder.size_hint().into_option()
    }
}

struct MapAccess<'a, D>
where
    D: ?Sized,
{
    decoder: &'a mut D,
}

impl<'a, D> MapAccess<'a, D>
where
    D: ?Sized,
{
    fn new(decoder: &'a mut D) -> Self {
        Self { decoder }
    }
}

impl<'de, D> de::MapAccess<'de> for MapAccess<'_, D>
where
    D: ?Sized + EntriesDecoder<'de, Cx: Context<Error: de::Error>>,
{
    type Error = <D::Cx as Context>::Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some(decoder) = self.decoder.decode_entry_key()? else {
            return Ok(None);
        };

        let output = seed.deserialize(Deserializer::new(decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let decoder = self.decoder.decode_entry_value()?;
        let output = seed.deserialize(Deserializer::new(decoder))?;
        Ok(output)
    }
}

struct StringVisitor<V> {
    visitor: V,
}

impl<V> StringVisitor<V> {
    fn new(visitor: V) -> Self {
        Self { visitor }
    }
}

impl<'de, C, V> crate::de::UnsizedVisitor<'de, C, str> for StringVisitor<V>
where
    C: Context<Error: de::Error>,
    V: de::Visitor<'de>,
{
    type Ok = V::Value;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(f)
    }

    #[inline]
    #[cfg(feature = "alloc")]
    fn visit_owned(self, _: C, value: String) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_string(self.visitor, value)
    }

    #[inline]
    fn visit_borrowed(self, _: C, value: &'de str) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_borrowed_str(self.visitor, value)
    }

    #[inline]
    fn visit_ref(self, _: C, value: &str) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_str(self.visitor, value)
    }
}

struct EnumAccess<'a, D>
where
    D: ?Sized,
{
    decoder: &'a mut D,
}

impl<'a, D> EnumAccess<'a, D>
where
    D: ?Sized,
{
    fn new(decoder: &'a mut D) -> Self {
        Self { decoder }
    }
}

impl<'de, D> de::VariantAccess<'de> for EnumAccess<'_, D>
where
    D: ?Sized + VariantDecoder<'de, Cx: Context<Error: de::Error>>,
{
    type Error = <D::Cx as Context>::Error;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        self.decoder.decode_value()?.decode_empty()
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer::new(self.decoder.decode_value()?))
    }

    #[inline]
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let hint = SequenceHint::with_size(len);

        self.decoder
            .decode_value()?
            .decode_sequence_hint(&hint, |tuple| visitor.visit_seq(SequenceAccess::new(tuple)))
    }

    #[inline]
    fn struct_variant<V>(
        self,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder
            .decode_value()?
            .decode_map_entries(|decoder| visitor.visit_map(StructAccess::new(decoder)))
    }
}

impl<'de, D> de::EnumAccess<'de> for EnumAccess<'_, D>
where
    D: VariantDecoder<'de, Cx: Context<Error: de::Error>>,
{
    type Error = <D::Cx as Context>::Error;
    type Variant = Self;

    #[inline]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let tag = self.decoder.decode_tag()?;
        let value = seed.deserialize(Deserializer::new(tag))?;
        Ok((value, self))
    }
}

struct AnyVisitor<V> {
    visitor: V,
}

impl<V> AnyVisitor<V> {
    fn new(visitor: V) -> Self {
        Self { visitor }
    }
}

#[crate::visitor(crate)]
impl<'de, C, V> Visitor<'de, C> for AnyVisitor<V>
where
    C: Context<Error: de::Error>,
    V: de::Visitor<'de>,
{
    type Ok = V::Value;

    type String = StringVisitor<V>;
    type Bytes = BytesVisitor<V>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(f)
    }

    #[inline]
    fn visit_empty(self, _: C) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_unit()
    }

    #[inline]
    fn visit_bool(self, _: C, v: bool) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_bool(v)
    }

    #[inline]
    fn visit_char(self, _: C, v: char) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_char(v)
    }

    #[inline]
    fn visit_u8(self, _: C, v: u8) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u8(v)
    }

    #[inline]
    fn visit_u16(self, _: C, v: u16) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u16(v)
    }

    #[inline]
    fn visit_u32(self, _: C, v: u32) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u32(v)
    }

    #[inline]
    fn visit_u64(self, _: C, v: u64) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u64(v)
    }

    #[inline]
    fn visit_u128(self, _: C, v: u128) -> Result<Self::Ok, C::Error> {
        // Serde's 128-bit support is very broken, so just try to avoid it if we can.
        // See: https://github.com/serde-rs/serde/issues/2576
        if let Ok(v) = u64::try_from(v) {
            return self.visitor.visit_u64(v);
        }

        self.visitor.visit_u128(v)
    }

    #[inline]
    fn visit_i8(self, _: C, v: i8) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i8(v)
    }

    #[inline]
    fn visit_i16(self, _: C, v: i16) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i16(v)
    }

    #[inline]
    fn visit_i32(self, _: C, v: i32) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i32(v)
    }

    #[inline]
    fn visit_i64(self, _: C, v: i64) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i64(v)
    }

    #[inline]
    fn visit_i128(self, _: C, v: i128) -> Result<Self::Ok, C::Error> {
        // Serde's 128-bit support is very broken, so just try to avoid it if we can.
        // See: https://github.com/serde-rs/serde/issues/2576
        if let Ok(v) = i64::try_from(v) {
            return self.visitor.visit_i64(v);
        }

        self.visitor.visit_i128(v)
    }

    #[inline]
    fn visit_usize(self, cx: C, v: usize) -> Result<Self::Ok, C::Error> {
        if let Some(value) = unsigned_value(self.visitor, v)? {
            return Ok(value);
        }

        Err(cx.message(format_args!("Unsupported usize value {v}")))
    }

    #[inline]
    fn visit_isize(self, cx: C, v: isize) -> Result<Self::Ok, C::Error> {
        if let Some(value) = signed_value(self.visitor, v)? {
            return Ok(value);
        }

        Err(cx.message(format_args!("Unsupported isize value {v}")))
    }

    #[inline]
    fn visit_f32(self, _: C, v: f32) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_f32(v)
    }

    #[inline]
    fn visit_f64(self, _: C, v: f64) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_f64(v)
    }

    #[inline]
    fn visit_option<D>(self, _: C, v: Option<D>) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, Cx = C>,
    {
        match v {
            Some(v) => self.visitor.visit_some(Deserializer::new(v)),
            None => self.visitor.visit_none(),
        }
    }

    #[inline]
    fn visit_sequence<D>(self, decoder: &mut D) -> Result<Self::Ok, C::Error>
    where
        D: ?Sized + SequenceDecoder<'de, Cx = C>,
    {
        self.visitor.visit_seq(SeqAccess::new(decoder))
    }

    #[inline]
    fn visit_map<D>(self, decoder: &mut D) -> Result<Self::Ok, C::Error>
    where
        D: ?Sized + MapDecoder<'de, Cx = C>,
    {
        let mut map_decoder = decoder.decode_remaining_entries()?;
        let value = self.visitor.visit_map(MapAccess::new(&mut map_decoder))?;
        map_decoder.end_entries()?;
        Ok(value)
    }

    #[inline]
    fn visit_string(self, _: C, _: SizeHint) -> Result<Self::String, C::Error> {
        Ok(StringVisitor::new(self.visitor))
    }

    #[inline]
    fn visit_bytes(self, _: C, _: SizeHint) -> Result<Self::Bytes, C::Error> {
        Ok(BytesVisitor::new(self.visitor))
    }
}

fn unsigned_value<'de, V, E>(visitor: V, v: usize) -> Result<Option<V::Value>, E>
where
    V: de::Visitor<'de>,
    E: de::Error,
{
    if let Ok(v) = u32::try_from(v) {
        return Ok(Some(visitor.visit_u32(v)?));
    }

    if let Ok(v) = u64::try_from(v) {
        return Ok(Some(visitor.visit_u64(v)?));
    }

    if let Ok(v) = u128::try_from(v) {
        return Ok(Some(visitor.visit_u128(v)?));
    }

    Ok(None)
}

fn signed_value<'de, V, E>(visitor: V, v: isize) -> Result<Option<V::Value>, E>
where
    V: de::Visitor<'de>,
    E: de::Error,
{
    if let Ok(v) = i32::try_from(v) {
        return Ok(Some(visitor.visit_i32(v)?));
    }

    if let Ok(v) = i64::try_from(v) {
        return Ok(Some(visitor.visit_i64(v)?));
    }

    if let Ok(v) = i128::try_from(v) {
        return Ok(Some(visitor.visit_i128(v)?));
    }

    Ok(None)
}
