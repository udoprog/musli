use core::fmt;
use core::marker::PhantomData;

use musli::de::{
    Decoder, MapPairsDecoder, PackDecoder, PairDecoder, PairsDecoder, SequenceDecoder, SizeHint,
    StructPairsDecoder, VariantDecoder,
};
use musli::{Context, Mode};
use serde::de;

#[cfg(feature = "alloc")]
use alloc::string::String;

pub struct Deserializer<'a, C, D, M> {
    cx: &'a C,
    decoder: D,
    _mode: PhantomData<M>,
}

impl<'a, C, D, M> Deserializer<'a, C, D, M> {
    /// Construct a new deserializer out of a decoder.
    pub fn new(cx: &'a C, decoder: D) -> Self {
        Self {
            cx,
            decoder,
            _mode: PhantomData,
        }
    }
}

impl<'de, 'a, C, D, M> de::Deserializer<'de> for Deserializer<'a, C, D, M>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: Decoder<'de>,
    M: Mode,
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
        let value = self.decoder.decode_bool(self.cx.adapt())?;
        visitor.visit_bool(value)
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i8(self.cx.adapt())?;
        visitor.visit_i8(value)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i16(self.cx.adapt())?;
        visitor.visit_i16(value)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i32(self.cx.adapt())?;
        visitor.visit_i32(value)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_i64(self.cx.adapt())?;
        visitor.visit_i64(value)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u8(self.cx.adapt())?;
        visitor.visit_u8(value)
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u16(self.cx.adapt())?;
        visitor.visit_u16(value)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u32(self.cx.adapt())?;
        visitor.visit_u32(value)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_u64(self.cx.adapt())?;
        visitor.visit_u64(value)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_f32(self.cx.adapt())?;
        visitor.visit_f32(value)
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_f64(self.cx.adapt())?;
        visitor.visit_f64(value)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.decoder.decode_char(self.cx.adapt())?;
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

        self.decoder
            .decode_string(self.cx.adapt(), Visitor(visitor))
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

        self.decoder.decode_bytes(self.cx.adapt(), Visitor(visitor))
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
        match self.decoder.decode_option(self.cx.adapt())? {
            Some(decoder) => {
                visitor.visit_some(Deserializer::<_, _, M>::new(self.cx.adapt(), decoder))
            }
            None => visitor.visit_none(),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.decode_unit(self.cx.adapt())?;
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
        let decoder = self.decoder.decode_struct(self.cx.adapt(), 0)?;
        decoder.end(self.cx.adapt())?;
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
        let mut decoder = self.decoder.decode_struct(self.cx.adapt(), 1)?;

        let Some(mut field) = decoder.next(self.cx.adapt())? else {
            return Err(self.cx.message("newtype struct missing first field"));
        };

        let k = field.first(self.cx.adapt())?;

        if k.decode_usize(self.cx.adapt())? != 0 {
            return Err(self.cx.message("newtype struct missing first field"));
        }

        let v = field.second(self.cx.adapt())?;
        let output =
            visitor.visit_newtype_struct(Deserializer::<_, _, M>::new(self.cx.adapt(), v))?;

        decoder.end(self.cx.adapt())?;
        Ok(output)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoder = self.decoder.decode_sequence(self.cx.adapt())?;

        struct SeqAccess<'a, C, D, M> {
            cx: &'a C,
            decoder: Option<D>,
            _mode: PhantomData<M>,
        }

        impl<'de, 'a, C, D, M> de::SeqAccess<'de> for SeqAccess<'a, C, D, M>
        where
            C: Context<Input = D::Error>,
            C::Error: de::Error,
            D: SequenceDecoder<'de>,
            M: Mode,
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

                let Some(decoder) = decoder.next(self.cx.adapt())? else {
                    if let Some(decoder) = self.decoder.take() {
                        decoder.end(self.cx.adapt())?;
                    }

                    return Ok(None);
                };

                let output =
                    seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), decoder))?;
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
            _mode: PhantomData::<M>,
        })
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut tuple = self.decoder.decode_tuple(self.cx.adapt(), len)?;
        let value = visitor.visit_seq(TupleAccess::<_, _, M>::new(self.cx, &mut tuple, len))?;
        tuple.end(self.cx.adapt())?;
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
        struct MapAccess<'a, C, D, M> {
            cx: &'a C,
            decoder: &'a mut D,
            _mode: PhantomData<M>,
        }

        impl<'de, 'a, C, D, M> de::MapAccess<'de> for MapAccess<'a, C, D, M>
        where
            C: Context<Input = D::Error>,
            C::Error: de::Error,
            D: MapPairsDecoder<'de>,
            M: Mode,
        {
            type Error = C::Error;

            #[inline]
            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: de::DeserializeSeed<'de>,
            {
                let Some(decoder) = self.decoder.key(self.cx.adapt())? else {
                    return Ok(None);
                };

                let output =
                    seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), decoder))?;
                Ok(Some(output))
            }

            #[inline]
            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
            where
                V: de::DeserializeSeed<'de>,
            {
                let decoder = self.decoder.value(self.cx.adapt())?;
                let output =
                    seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), decoder))?;
                Ok(output)
            }
        }

        let mut decoder = self.decoder.decode_map_pairs(self.cx.adapt())?;

        let map = MapAccess {
            cx: self.cx,
            decoder: &mut decoder,
            _mode: PhantomData::<M>,
        };

        let output = visitor.visit_map(map)?;
        decoder.end(self.cx.adapt())?;
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
            .decode_struct_pairs(self.cx.adapt(), fields.len())?;
        let output = visitor.visit_map(StructAccess::<_, _, M>::new(
            self.cx,
            &mut decoder,
            fields.len(),
        ))?;
        decoder.end(self.cx.adapt())?;
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
        struct EnumAccess<'a, C, D, M> {
            cx: &'a C,
            decoder: &'a mut D,
            _mode: PhantomData<M>,
        }

        impl<'a, 'de, C, D, M> de::EnumAccess<'de> for EnumAccess<'a, C, D, M>
        where
            C: Context<Input = D::Error>,
            C::Error: de::Error,
            D: VariantDecoder<'de>,
            M: Mode,
        {
            type Error = C::Error;
            type Variant = Self;

            #[inline]
            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
            where
                V: de::DeserializeSeed<'de>,
            {
                let t = self.decoder.tag(self.cx.adapt())?;
                let value = seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), t))?;
                Ok((value, self))
            }
        }

        impl<'a, 'de, C, D, M> de::VariantAccess<'de> for EnumAccess<'a, C, D, M>
        where
            C: Context<Input = D::Error>,
            C::Error: de::Error,
            D: VariantDecoder<'de>,
            M: Mode,
        {
            type Error = C::Error;

            #[inline]
            fn unit_variant(self) -> Result<(), Self::Error> {
                let decoder = self.decoder.variant(self.cx.adapt())?;
                let st = decoder.decode_struct(self.cx.adapt(), 0)?;
                st.end(self.cx.adapt())?;
                Ok(())
            }

            #[inline]
            fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                let decoder = self.decoder.variant(self.cx.adapt())?;
                let mut tuple = decoder.decode_tuple(self.cx.adapt(), 1)?;
                let field = tuple.next(self.cx.adapt())?;
                let value =
                    seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), field))?;
                tuple.end(self.cx.adapt())?;
                Ok(value)
            }

            #[inline]
            fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                let decoder = self.decoder.variant(self.cx.adapt())?;
                let mut tuple = decoder.decode_tuple(self.cx.adapt(), len)?;

                let value =
                    visitor.visit_seq(TupleAccess::<_, _, M>::new(self.cx, &mut tuple, len))?;

                tuple.end(self.cx.adapt())?;
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
                let decoder = self.decoder.variant(self.cx.adapt())?;
                let mut st = decoder.decode_struct_pairs(self.cx.adapt(), fields.len())?;
                let value = visitor.visit_map(StructAccess::<_, _, M>::new(
                    self.cx,
                    &mut st,
                    fields.len(),
                ))?;
                st.end(self.cx.adapt())?;
                Ok(value)
            }
        }

        let mut decoder = self.decoder.decode_variant(self.cx.adapt())?;

        let enum_access = EnumAccess {
            cx: self.cx,
            decoder: &mut decoder,
            _mode: PhantomData::<M>,
        };

        let value = visitor.visit_enum(enum_access)?;
        decoder.end(self.cx.adapt())?;
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
        self.decoder.skip(self.cx.adapt())?;
        visitor.visit_unit()
    }
}

struct TupleAccess<'a, C, D, M> {
    cx: &'a C,
    decoder: &'a mut D,
    remaining: usize,
    _mode: PhantomData<M>,
}

impl<'a, C, D, M> TupleAccess<'a, C, D, M> {
    fn new(cx: &'a C, decoder: &'a mut D, len: usize) -> Self
    where
        M: Mode,
    {
        TupleAccess {
            cx,
            decoder,
            remaining: len,
            _mode: PhantomData::<M>,
        }
    }
}

impl<'de, 'a, C, D, M> de::SeqAccess<'de> for TupleAccess<'a, C, D, M>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: PackDecoder<'de>,
    M: Mode,
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

        let decoder = self.decoder.next(self.cx.adapt())?;
        let output = seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}
struct StructAccess<'a, C, D, M> {
    cx: &'a C,
    decoder: &'a mut D,
    remaining: usize,
    _mode: PhantomData<M>,
}

impl<'a, C, D, M> StructAccess<'a, C, D, M> {
    fn new(cx: &'a C, decoder: &'a mut D, remaining: usize) -> Self {
        StructAccess {
            cx,
            decoder,
            remaining,
            _mode: PhantomData,
        }
    }
}

impl<'de, 'a, C, D, M> de::MapAccess<'de> for StructAccess<'a, C, D, M>
where
    C: Context<Input = D::Error>,
    C::Error: de::Error,
    D: StructPairsDecoder<'de>,
    M: Mode,
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
        let decoder = self.decoder.field(self.cx.adapt())?;
        let output = seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), decoder))?;
        Ok(Some(output))
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let decoder = self.decoder.value(self.cx.adapt())?;
        let output = seed.deserialize(Deserializer::<_, _, M>::new(self.cx.adapt(), decoder))?;
        Ok(output)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}
