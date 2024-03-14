use core::fmt;

use musli::de::{
    Decoder, MapPairsDecoder, PackDecoder, SequenceDecoder, SizeHint, StructDecoder,
    StructFieldDecoder, StructPairsDecoder, VariantDecoder,
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
    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(self
            .cx
            .message("Deserialization of any value is not yet supported"))
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
        struct Visitor<V>(V);

        impl<'de, C, V> musli::de::ValueVisitor<'de, C, str> for Visitor<V>
        where
            C: Context,
            C::Error: de::Error,
            V: de::Visitor<'de>,
        {
            type Ok = V::Value;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[inline]
            #[cfg(any(feature = "std", feature = "alloc"))]
            fn visit_owned(self, _: &C, value: String) -> Result<Self::Ok, C::Error> {
                de::Visitor::visit_string(self.0, value)
            }

            #[inline]
            fn visit_borrowed(self, _: &C, value: &'de str) -> Result<Self::Ok, C::Error> {
                de::Visitor::visit_borrowed_str(self.0, value)
            }

            #[inline]
            fn visit_ref(self, _: &C, value: &str) -> Result<Self::Ok, C::Error> {
                de::Visitor::visit_str(self.0, value)
            }
        }

        self.decoder.decode_string(self.cx, Visitor(visitor))
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
        struct Visitor<V>(V);

        impl<'de, C, V> musli::de::ValueVisitor<'de, C, [u8]> for Visitor<V>
        where
            C: Context,
            C::Error: de::Error,
            V: de::Visitor<'de>,
        {
            type Ok = V::Value;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.expecting(f)
            }

            #[inline]
            #[cfg(any(feature = "std", feature = "alloc"))]
            fn visit_owned(self, _: &C, value: Vec<u8>) -> Result<Self::Ok, C::Error> {
                de::Visitor::visit_byte_buf(self.0, value)
            }

            #[inline]
            fn visit_borrowed(self, _: &C, value: &'de [u8]) -> Result<Self::Ok, C::Error> {
                de::Visitor::visit_borrowed_bytes(self.0, value)
            }

            #[inline]
            fn visit_ref(self, _: &C, value: &[u8]) -> Result<Self::Ok, C::Error> {
                de::Visitor::visit_bytes(self.0, value)
            }
        }

        self.decoder.decode_bytes(self.cx, Visitor(visitor))
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
        let decoder = self.decoder.decode_struct(self.cx, Some(0))?;
        decoder.end(self.cx)?;
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
        let mut decoder = self.decoder.decode_struct(self.cx, Some(1))?;

        let Some(mut field) = decoder.field(self.cx)? else {
            return Err(self.cx.message("newtype struct missing first field"));
        };

        if field.field_name(self.cx)?.decode_usize(self.cx)? != 0 {
            return Err(self.cx.message("newtype struct missing first field"));
        }

        let output = visitor
            .visit_newtype_struct(Deserializer::new(self.cx, field.field_value(self.cx)?))?;

        decoder.end(self.cx)?;
        Ok(output)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoder = self.decoder.decode_sequence(self.cx)?;

        struct SeqAccess<'a, C, D> {
            cx: &'a C,
            decoder: Option<D>,
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
                let Some(decoder) = &mut self.decoder else {
                    return Ok(None);
                };

                let Some(decoder) = decoder.next(self.cx)? else {
                    if let Some(decoder) = self.decoder.take() {
                        decoder.end(self.cx)?;
                    }

                    return Ok(None);
                };

                let output = seed.deserialize(Deserializer::new(self.cx, decoder))?;
                Ok(Some(output))
            }

            #[inline]
            fn size_hint(&self) -> Option<usize> {
                let decoder = self.decoder.as_ref()?;

                match decoder.size_hint() {
                    SizeHint::Exact(n) => Some(n),
                    _ => None,
                }
            }
        }

        visitor.visit_seq(SeqAccess {
            cx: self.cx,
            decoder: Some(decoder),
        })
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
        struct MapAccess<'a, C, D> {
            cx: &'a C,
            decoder: &'a mut D,
        }

        impl<'de, 'a, C, D> de::MapAccess<'de> for MapAccess<'a, C, D>
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

        let mut decoder = self.decoder.decode_map_pairs(self.cx)?;

        let map = MapAccess {
            cx: self.cx,
            decoder: &mut decoder,
        };

        let output = visitor.visit_map(map)?;
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
            .decode_struct_pairs(self.cx, Some(fields.len()))?;
        let output = visitor.visit_map(StructAccess::new(self.cx, &mut decoder, fields.len()))?;
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
        struct EnumAccess<'a, C, D> {
            cx: &'a C,
            decoder: &'a mut D,
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
            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
            where
                V: de::DeserializeSeed<'de>,
            {
                let t = self.decoder.tag(self.cx)?;
                let value = seed.deserialize(Deserializer::new(self.cx, t))?;
                Ok((value, self))
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
            fn unit_variant(self) -> Result<(), Self::Error> {
                let decoder = self.decoder.variant(self.cx)?;
                let st = decoder.decode_struct(self.cx, Some(0))?;
                st.end(self.cx)?;
                Ok(())
            }

            #[inline]
            fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                let decoder = self.decoder.variant(self.cx)?;
                let mut tuple = decoder.decode_tuple(self.cx, 1)?;
                let field = tuple.next(self.cx)?;
                let value = seed.deserialize(Deserializer::new(self.cx, field))?;
                tuple.end(self.cx)?;
                Ok(value)
            }

            #[inline]
            fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                let decoder = self.decoder.variant(self.cx)?;
                let mut tuple = decoder.decode_tuple(self.cx, len)?;

                let value = visitor.visit_seq(TupleAccess::new(self.cx, &mut tuple, len))?;

                tuple.end(self.cx)?;
                Ok(value)
            }

            #[inline]
            fn struct_variant<V>(
                self,
                fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                let decoder = self.decoder.variant(self.cx)?;
                let mut st = decoder.decode_struct_pairs(self.cx, Some(fields.len()))?;
                let value = visitor.visit_map(StructAccess::new(self.cx, &mut st, fields.len()))?;
                st.end(self.cx)?;
                Ok(value)
            }
        }

        let mut decoder = self.decoder.decode_variant(self.cx)?;

        let enum_access = EnumAccess {
            cx: self.cx,
            decoder: &mut decoder,
        };

        let value = visitor.visit_enum(enum_access)?;
        decoder.end(self.cx)?;
        Ok(value)
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
    fn new(cx: &'a C, decoder: &'a mut D, remaining: usize) -> Self {
        StructAccess {
            cx,
            decoder,
            remaining,
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
