use core::fmt;

use musli::de::{
    Decoder, MapDecoder, MapPairsDecoder, PackDecoder, SequenceDecoder, SizeHint, StructDecoder,
    StructPairsDecoder, VariantDecoder, Visitor,
};
use musli::Context;
use serde::de;

#[cfg(feature = "alloc")]
use alloc::string::String;

pub struct Deserializer<'a, C, D> {
    cx: &'a C,
    decoder: D,
}

impl<'a, C, D> Deserializer<'a, C, D> {
    /// Construct a new deserializer out of a decoder.
    pub fn new(cx: &'a C, decoder: D) -> Self {
        Self { cx, decoder }
    }
}

impl<'de, 'a, C, D> de::Deserializer<'de> for Deserializer<'a, C, D>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: Decoder<'de>,
{
    type Error = C::Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_any(self.cx, AnyVisitor::new(visitor))
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_bool(self.cx)?;
        visitor.visit_bool(value)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i8(self.cx)?;
        visitor.visit_i8(value)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i16(self.cx)?;
        visitor.visit_i16(value)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i32(self.cx)?;
        visitor.visit_i32(value)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i64(self.cx)?;
        visitor.visit_i64(value)
    }

    #[inline]
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i128(self.cx)?;
        visitor.visit_i128(value)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u8(self.cx)?;
        visitor.visit_u8(value)
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u16(self.cx)?;
        visitor.visit_u16(value)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u32(self.cx)?;
        visitor.visit_u32(value)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u64(self.cx)?;
        visitor.visit_u64(value)
    }

    #[inline]
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u128(self.cx)?;
        visitor.visit_u128(value)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_f32(self.cx)?;
        visitor.visit_f32(value)
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_f64(self.cx)?;
        visitor.visit_f64(value)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_char(self.cx)?;
        visitor.visit_char(value)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder
            .decode_string(self.cx, StringVisitor::new(visitor))
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
        self.decoder
            .decode_bytes(self.cx, BytesVisitor::new(visitor))
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
        match self.decoder.decode_option(self.cx)? {
            Some(decoder) => visitor.visit_some(Deserializer::new(self.cx, decoder)),
            None => visitor.visit_none(),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_unit(self.cx)?;
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
        self.decoder.decode_unit(self.cx)?;
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
        visitor.visit_newtype_struct(Deserializer::new(self.cx, self.decoder))
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut decoder = self.decoder.decode_sequence(self.cx)?;
        let value = visitor.visit_seq(SeqAccess::new(self.cx, &mut decoder))?;
        decoder.end(self.cx)?;
        Ok(value)
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut tuple = self.decoder.decode_tuple(self.cx, len)?;
        let value = visitor.visit_seq(TupleAccess::new(self.cx, &mut tuple, len))?;
        tuple.end(self.cx)?;
        Ok(value)
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
        let mut decoder = self.decoder.decode_map(self.cx)?.into_map_pairs(self.cx)?;
        let output = visitor.visit_map(MapAccess::new(self.cx, &mut decoder))?;
        decoder.end(self.cx)?;
        Ok(output)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut decoder = self
            .decoder
            .decode_struct(self.cx, Some(fields.len()))?
            .into_struct_pairs(self.cx)?;
        let output = visitor.visit_map(StructAccess::new(self.cx, &mut decoder, fields))?;
        decoder.end(self.cx)?;
        Ok(output)
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
        let decoder = self.decoder.decode_variant(self.cx)?;
        visitor.visit_enum(EnumAccess::new(self.cx, decoder))
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
        self.decoder.skip(self.cx)?;
        visitor.visit_unit()
    }
}

struct TupleAccess<'a, C, D> {
    cx: &'a C,
    decoder: &'a mut D,
    remaining: usize,
}

impl<'a, C, D> TupleAccess<'a, C, D> {
    fn new(cx: &'a C, decoder: &'a mut D, len: usize) -> Self
where {
        TupleAccess {
            cx,
            decoder,
            remaining: len,
        }
    }
}

impl<'de, 'a, C, D> de::SeqAccess<'de> for TupleAccess<'a, C, D>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: PackDecoder<'de>,
{
    type Error = C::Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;

        let decoder = self.decoder.next(self.cx)?;
        let output = seed.deserialize(Deserializer::new(self.cx, decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}
struct StructAccess<'a, C, D> {
    cx: &'a C,
    decoder: &'a mut D,
    remaining: usize,
}

impl<'a, C, D> StructAccess<'a, C, D> {
    #[inline]
    fn new(cx: &'a C, decoder: &'a mut D, fields: &'static [&'static str]) -> Self {
        StructAccess {
            cx,
            decoder,
            remaining: fields.len(),
        }
    }
}

impl<'de, 'a, C, D> de::MapAccess<'de> for StructAccess<'a, C, D>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: StructPairsDecoder<'de>,
{
    type Error = C::Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        let decoder = self.decoder.field_name(self.cx)?;
        let output = seed.deserialize(Deserializer::new(self.cx, decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let decoder = self.decoder.field_value(self.cx)?;
        let output = seed.deserialize(Deserializer::new(self.cx, decoder))?;
        Ok(output)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
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

impl<'de, C, V> musli::de::ValueVisitor<'de, C, [u8]> for BytesVisitor<V>
where
    C: Context,
    C::Error: de::Error,
    V: de::Visitor<'de>,
{
    type Ok = V::Value;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(f)
    }

    #[inline]
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_owned(self, _: &C, value: Vec<u8>) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_byte_buf(self.visitor, value)
    }

    #[inline]
    fn visit_borrowed(self, _: &C, value: &'de [u8]) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_borrowed_bytes(self.visitor, value)
    }

    #[inline]
    fn visit_ref(self, _: &C, value: &[u8]) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_bytes(self.visitor, value)
    }
}

struct SeqAccess<'a, C, D> {
    cx: &'a C,
    decoder: &'a mut D,
}

impl<'a, C, D> SeqAccess<'a, C, D> {
    fn new(cx: &'a C, decoder: &'a mut D) -> Self {
        Self { cx, decoder }
    }
}

impl<'de, 'a, C, D> de::SeqAccess<'de> for SeqAccess<'a, C, D>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: SequenceDecoder<'de>,
{
    type Error = C::Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let Some(decoder) = self.decoder.next(self.cx)? else {
            return Ok(None);
        };

        let output = seed.deserialize(Deserializer::new(self.cx, decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        match self.decoder.size_hint() {
            SizeHint::Exact(n) => Some(n),
            _ => None,
        }
    }
}

struct MapAccess<'a, C, D: ?Sized> {
    cx: &'a C,
    decoder: &'a mut D,
}

impl<'a, C, D: ?Sized> MapAccess<'a, C, D> {
    fn new(cx: &'a C, decoder: &'a mut D) -> Self {
        Self { cx, decoder }
    }
}

impl<'de, 'a, C, D: ?Sized> de::MapAccess<'de> for MapAccess<'a, C, D>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: MapPairsDecoder<'de>,
{
    type Error = C::Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some(decoder) = self.decoder.map_pairs_key(self.cx)? else {
            return Ok(None);
        };

        let output = seed.deserialize(Deserializer::new(self.cx, decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let decoder = self.decoder.map_pairs_value(self.cx)?;
        let output = seed.deserialize(Deserializer::new(self.cx, decoder))?;
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

impl<'de, C, V> musli::de::ValueVisitor<'de, C, str> for StringVisitor<V>
where
    C: Context,
    C::Error: de::Error,
    V: de::Visitor<'de>,
{
    type Ok = V::Value;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(f)
    }

    #[inline]
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_owned(self, _: &C, value: String) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_string(self.visitor, value)
    }

    #[inline]
    fn visit_borrowed(self, _: &C, value: &'de str) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_borrowed_str(self.visitor, value)
    }

    #[inline]
    fn visit_ref(self, _: &C, value: &str) -> Result<Self::Ok, C::Error> {
        de::Visitor::visit_str(self.visitor, value)
    }
}

struct NumberVisitor<V> {
    visitor: V,
}

impl<V> NumberVisitor<V> {
    fn new(visitor: V) -> Self {
        Self { visitor }
    }
}

impl<'de, C, V> musli::de::NumberVisitor<'de, C> for NumberVisitor<V>
where
    C: Context,
    C::Error: de::Error,
    V: de::Visitor<'de>,
{
    type Ok = V::Value;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(f)
    }

    #[inline]
    fn visit_u8(self, _: &C, v: u8) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_u8(v)
    }

    #[inline]
    fn visit_u16(self, _: &C, v: u16) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_u16(v)
    }

    #[inline]
    fn visit_u32(self, _: &C, v: u32) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_u32(v)
    }

    #[inline]
    fn visit_u64(self, _: &C, v: u64) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_u64(v)
    }

    #[inline]
    fn visit_u128(self, _: &C, v: u128) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_u128(v)
    }

    #[inline]
    fn visit_i8(self, _: &C, v: i8) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_i8(v)
    }

    #[inline]
    fn visit_i16(self, _: &C, v: i16) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_i16(v)
    }

    #[inline]
    fn visit_i32(self, _: &C, v: i32) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_i32(v)
    }

    #[inline]
    fn visit_i64(self, _: &C, v: i64) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_i64(v)
    }

    #[inline]
    fn visit_i128(self, _: &C, v: i128) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_i128(v)
    }

    #[inline]
    fn visit_f32(self, _: &C, v: f32) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_f32(v)
    }

    #[inline]
    fn visit_f64(self, _: &C, v: f64) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_f64(v)
    }

    #[inline]
    fn visit_usize(self, cx: &C, v: usize) -> Result<Self::Ok, C::Error> {
        if let Some(value) = unsigned_value(self.visitor, v)? {
            return Ok(value);
        }

        Err(cx.message(format_args!("Unsupported usize value {v}")))
    }

    #[inline]
    fn visit_isize(self, cx: &C, v: isize) -> Result<Self::Ok, C::Error> {
        if let Some(value) = signed_value(self.visitor, v)? {
            return Ok(value);
        }

        Err(cx.message(format_args!("Unsupported isize value {v}")))
    }

    #[inline]
    fn visit_bytes(self, _: &C, v: &'de [u8]) -> Result<Self::Ok, <C as Context>::Error> {
        self.visitor.visit_bytes(v)
    }
}

struct EnumAccess<'a, C, D> {
    cx: &'a C,
    decoder: D,
}

impl<'a, C, D> EnumAccess<'a, C, D> {
    fn new(cx: &'a C, decoder: D) -> Self {
        Self { cx, decoder }
    }
}

impl<'a, 'de, C, D> de::VariantAccess<'de> for EnumAccess<'a, C, D>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: VariantDecoder<'de>,
{
    type Error = C::Error;

    #[inline]
    fn unit_variant(mut self) -> Result<(), Self::Error> {
        self.decoder.variant(self.cx)?.decode_unit(self.cx)
    }

    #[inline]
    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(Deserializer::new(self.cx, self.decoder.variant(self.cx)?))?;
        self.decoder.end(self.cx)?;
        Ok(value)
    }

    #[inline]
    fn tuple_variant<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoder = self.decoder.variant(self.cx)?;
        let mut tuple = decoder.decode_tuple(self.cx, len)?;
        let value = visitor.visit_seq(TupleAccess::new(self.cx, &mut tuple, len))?;
        tuple.end(self.cx)?;
        self.decoder.end(self.cx)?;
        Ok(value)
    }

    #[inline]
    fn struct_variant<V>(
        mut self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoder = self.decoder.variant(self.cx)?;
        let mut st = decoder
            .decode_struct(self.cx, Some(fields.len()))?
            .into_struct_pairs(self.cx)?;
        let value = visitor.visit_map(StructAccess::new(self.cx, &mut st, fields))?;
        st.end(self.cx)?;
        self.decoder.end(self.cx)?;
        Ok(value)
    }
}

impl<'a, 'de, C, D> de::EnumAccess<'de> for EnumAccess<'a, C, D>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: VariantDecoder<'de>,
{
    type Error = C::Error;
    type Variant = Self;

    #[inline]
    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let tag = self.decoder.tag(self.cx)?;
        let value = seed.deserialize(Deserializer::new(self.cx, tag))?;
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

#[musli::visitor]
impl<'de, C, V> Visitor<'de, C> for AnyVisitor<V>
where
    C: Context,
    C::Error: de::Error,
    V: de::Visitor<'de>,
{
    type Ok = V::Value;

    type String = StringVisitor<V>;
    type Bytes = BytesVisitor<V>;
    type Number = NumberVisitor<V>;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visitor.expecting(f)
    }

    #[inline]
    fn visit_unit(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_unit()
    }

    #[inline]
    fn visit_bool(self, _: &C, v: bool) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_bool(v)
    }

    #[inline]
    fn visit_char(self, _: &C, v: char) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_char(v)
    }

    #[inline]
    fn visit_u8(self, _: &C, v: u8) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u8(v)
    }

    #[inline]
    fn visit_u16(self, _: &C, v: u16) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u16(v)
    }

    #[inline]
    fn visit_u32(self, _: &C, v: u32) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u32(v)
    }

    #[inline]
    fn visit_u64(self, _: &C, v: u64) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u64(v)
    }

    #[inline]
    fn visit_u128(self, _: &C, v: u128) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_u128(v)
    }

    #[inline]
    fn visit_i8(self, _: &C, v: i8) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i8(v)
    }

    #[inline]
    fn visit_i16(self, _: &C, v: i16) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i16(v)
    }

    #[inline]
    fn visit_i32(self, _: &C, v: i32) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i32(v)
    }

    #[inline]
    fn visit_i64(self, _: &C, v: i64) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i64(v)
    }

    #[inline]
    fn visit_i128(self, _: &C, v: i128) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_i128(v)
    }

    #[inline]
    fn visit_usize(self, cx: &C, v: usize) -> Result<Self::Ok, C::Error> {
        if let Some(value) = unsigned_value(self.visitor, v)? {
            return Ok(value);
        }

        Err(cx.message(format_args!("Unsupported usize value {v}")))
    }

    #[inline]
    fn visit_isize(self, cx: &C, v: isize) -> Result<Self::Ok, C::Error> {
        if let Some(value) = signed_value(self.visitor, v)? {
            return Ok(value);
        }

        Err(cx.message(format_args!("Unsupported isize value {v}")))
    }

    #[inline]
    fn visit_f32(self, _: &C, v: f32) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_f32(v)
    }

    #[inline]
    fn visit_f64(self, _: &C, v: f64) -> Result<Self::Ok, C::Error> {
        self.visitor.visit_f64(v)
    }

    #[inline]
    fn visit_option<D>(self, cx: &C, v: Option<D>) -> Result<Self::Ok, C::Error>
    where
        D: Decoder<'de, Error = C::Input>,
    {
        match v {
            Some(v) => self.visitor.visit_some(Deserializer::new(cx, v)),
            None => self.visitor.visit_none(),
        }
    }

    #[inline]
    fn visit_sequence<D>(self, cx: &C, mut decoder: D) -> Result<Self::Ok, C::Error>
    where
        D: SequenceDecoder<'de, Error = C::Input>,
    {
        let value = self.visitor.visit_seq(SeqAccess::new(cx, &mut decoder))?;
        decoder.end(cx)?;
        Ok(value)
    }

    #[inline]
    fn visit_map<D>(self, cx: &C, decoder: D) -> Result<Self::Ok, C::Error>
    where
        D: MapDecoder<'de, Error = C::Input>,
    {
        let mut map_decoder = decoder.into_map_pairs(cx)?;
        let value = self
            .visitor
            .visit_map(MapAccess::new(cx, &mut map_decoder))?;
        map_decoder.end(cx)?;
        Ok(value)
    }

    #[inline]
    fn visit_string(self, _: &C, _: SizeHint) -> Result<Self::String, C::Error> {
        Ok(StringVisitor::new(self.visitor))
    }

    #[inline]
    fn visit_bytes(self, _: &C, _: SizeHint) -> Result<Self::Bytes, C::Error> {
        Ok(BytesVisitor::new(self.visitor))
    }

    #[inline]
    fn visit_number(self, _: &C, _: musli::de::NumberHint) -> Result<Self::Number, C::Error> {
        Ok(NumberVisitor::new(self.visitor))
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
