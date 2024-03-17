use std::fmt;

use musli::en::{
    MapEntriesEncoder, SequenceEncoder, StructEncoder, StructFieldEncoder, VariantEncoder,
};
use musli::{Context, Encode, Encoder};

use serde::ser::{self, Serialize};

pub struct Serializer<'a, C: ?Sized, E> {
    cx: &'a C,
    encoder: E,
}

impl<'a, C: ?Sized, E> Serializer<'a, C, E> {
    /// Construct a new deserializer out of an encoder.
    pub fn new(cx: &'a C, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<'a, C, E> ser::Serializer for Serializer<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: Encoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    type SerializeSeq = SerializeSeq<'a, C, E::EncodeSequence>;
    type SerializeTuple = SerializeSeq<'a, C, E::EncodeTuple>;
    type SerializeTupleStruct = SerializeTupleStruct<'a, C, E::EncodeStruct>;
    type SerializeTupleVariant = SerializeSeq<'a, C, E::EncodeTupleVariant>;
    type SerializeMap = SerializeMap<'a, C, E::EncodeMapEntries>;
    type SerializeStruct = SerializeStruct<'a, C, E::EncodeStruct>;
    type SerializeStructVariant = SerializeStructVariant<'a, C, E::EncodeStructVariant>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_bool(self.cx, v)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i8(self.cx, v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i16(self.cx, v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i32(self.cx, v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i64(self.cx, v)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i128(self.cx, v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u8(self.cx, v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u16(self.cx, v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u32(self.cx, v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u64(self.cx, v)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u128(self.cx, v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_f32(self.cx, v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_f64(self.cx, v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_char(self.cx, v)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_string(self.cx, v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_bytes(self.cx, v)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_none(self.cx)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.encode_some(self.cx)?;
        value.serialize(Serializer::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_unit(self.cx)
    }

    #[inline]
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_unit(self.cx)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant_name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        encode_variant(self.cx, self.encoder, variant_name, |encoder| {
            encoder.encode_unit(self.cx)
        })
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(Serializer::new(self.cx, self.encoder))
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant_name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        encode_variant(self.cx, self.encoder, variant_name, move |encoder| {
            value.serialize(Serializer::new(self.cx, encoder))
        })
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let Some(len) = len else {
            return Err(ser::Error::custom(
                "Can only encode sequences with known lengths",
            ));
        };

        let encoder = self.encoder.encode_sequence(self.cx, len)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let encoder = self.encoder.encode_tuple(self.cx, len)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let encoder = self.encoder.encode_struct(self.cx, len)?;
        Ok(SerializeTupleStruct::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let encoder = self
            .encoder
            .encode_tuple_variant(self.cx, variant_name, len)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let Some(len) = len else {
            return Err(ser::Error::custom(
                "Can only encode maps with known lengths",
            ));
        };

        let encoder = self.encoder.encode_map_entries(self.cx, len)?;
        Ok(SerializeMap::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let encoder = self.encoder.encode_struct(self.cx, len)?;
        Ok(SerializeStruct::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let encoder = self
            .encoder
            .encode_struct_variant(self.cx, variant_name, len)?;
        Ok(SerializeStructVariant::new(self.cx, encoder))
    }

    #[inline]
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: fmt::Display,
    {
        let string = value.to_string();
        self.serialize_str(&string)
    }
}

#[inline]
fn encode_variant<C, E, T, F, O>(cx: &C, encoder: E, variant_tag: &T, f: F) -> Result<O, C::Error>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: Encoder<C>,
    T: ?Sized + Serialize,
    F: FnOnce(<E::EncodeVariant as VariantEncoder<C>>::EncodeValue<'_>) -> Result<O, C::Error>,
{
    let mut variant = encoder.encode_variant(cx)?;
    variant_tag.serialize(Serializer::new(cx, variant.encode_tag(cx)?))?;
    let output = f(variant.encode_value(cx)?)?;
    variant.end(cx)?;
    Ok(output)
}

pub struct SerializeSeq<'a, C: ?Sized, E> {
    cx: &'a C,
    encoder: E,
}

impl<'a, C: ?Sized, E> SerializeSeq<'a, C, E> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<'a, C, E> ser::SerializeSeq for SerializeSeq<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: SequenceEncoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.encode_next(self.cx)?;
        value.serialize(Serializer::new(self.cx, encoder))?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx)
    }
}

impl<'a, C, E> ser::SerializeTuple for SerializeSeq<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: SequenceEncoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, C, E> ser::SerializeTupleVariant for SerializeSeq<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: SequenceEncoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

pub struct SerializeTupleStruct<'a, C: ?Sized, E> {
    cx: &'a C,
    encoder: E,
    field: usize,
}

impl<'a, C: ?Sized, E> SerializeTupleStruct<'a, C, E> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self {
            cx,
            encoder,
            field: 0,
        }
    }
}

impl<'a, C, E> ser::SerializeTupleStruct for SerializeTupleStruct<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: StructEncoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let mut field = self.encoder.encode_field(self.cx)?;

        let k = field.encode_field_name(self.cx)?;
        self.field.encode(self.cx, k)?;

        let v = field.encode_field_value(self.cx)?;
        value.serialize(Serializer::new(self.cx, v))?;

        field.end(self.cx)?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx)
    }
}

pub struct SerializeMap<'a, C: ?Sized, E> {
    cx: &'a C,
    encoder: E,
}

impl<'a, C: ?Sized, E> SerializeMap<'a, C, E> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<'a, C, E> ser::SerializeMap for SerializeMap<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: MapEntriesEncoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.encode_map_entry_key(self.cx)?;
        key.serialize(Serializer::new(self.cx, encoder))?;
        Ok(())
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.encode_map_entry_value(self.cx)?;
        value.serialize(Serializer::new(self.cx, encoder))?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx)
    }
}

pub struct SerializeStruct<'a, C: ?Sized, E> {
    cx: &'a C,
    encoder: E,
}

impl<'a, C: ?Sized, E> SerializeStruct<'a, C, E> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<'a, C, E> ser::SerializeStruct for SerializeStruct<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: StructEncoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let mut field = self.encoder.encode_field(self.cx)?;
        key.encode(self.cx, field.encode_field_name(self.cx)?)?;
        value.serialize(Serializer::new(self.cx, field.encode_field_value(self.cx)?))?;
        field.end(self.cx)?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx)
    }
}

pub struct SerializeStructVariant<'a, C: ?Sized, E> {
    cx: &'a C,
    encoder: E,
}

impl<'a, C: ?Sized, E> SerializeStructVariant<'a, C, E> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<'a, C, E> ser::SerializeStructVariant for SerializeStructVariant<'a, C, E>
where
    C: ?Sized + Context,
    C::Error: ser::Error,
    E: StructEncoder<C>,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let mut field = self.encoder.encode_field(self.cx)?;
        key.encode(self.cx, field.encode_field_name(self.cx)?)?;
        value.serialize(Serializer::new(self.cx, field.encode_field_value(self.cx)?))?;
        field.end(self.cx)?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx)
    }
}
