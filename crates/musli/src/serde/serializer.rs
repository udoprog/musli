use core::any::TypeId;
use core::fmt;

use crate::en::{EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder};
use crate::hint::{MapHint, SequenceHint};
use crate::mode::Text;
use crate::{Context, Encoder};

use serde::ser::{self, Serialize};

pub struct Serializer<'a, E>
where
    E: Encoder,
{
    cx: &'a E::Cx,
    encoder: E,
}

impl<'a, E> Serializer<'a, E>
where
    E: Encoder,
{
    /// Construct a new deserializer out of an encoder.
    pub fn new(cx: &'a E::Cx, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<'a, E> ser::Serializer for Serializer<'a, E>
where
    E: Encoder,
    <E::Cx as Context>::Error: ser::Error,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    type SerializeSeq = SerializeSeq<'a, E::EncodeSequence>;
    type SerializeTuple = SerializeSeq<'a, E::EncodeSequence>;
    type SerializeTupleStruct = SerializeSeq<'a, E::EncodeSequence>;
    type SerializeTupleVariant = SerializeSeq<'a, E::EncodeSequenceVariant>;
    type SerializeMap = SerializeMap<'a, E::EncodeMapEntries>;
    type SerializeStruct = SerializeStruct<'a, E::EncodeMap>;
    type SerializeStructVariant = SerializeStructVariant<'a, E::EncodeMapVariant>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_bool(v)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i8(v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i16(v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i32(v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i64(v)
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i128(v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u8(v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u16(v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u32(v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u64(v)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u128(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_f32(v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_f64(v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_char(v)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_string(v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_bytes(v)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_none()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let encoder = self.encoder.encode_some()?;
        value.serialize(Serializer::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_empty()
    }

    #[inline]
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_empty()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant_name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        encode_variant(self.cx, self.encoder, variant_name, |encoder| {
            encoder.encode_empty()
        })
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(Serializer::new(self.cx, self.encoder))
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        variant_name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
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

        let hint = SequenceHint::with_size(len);
        let encoder = self.encoder.encode_sequence(&hint)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let hint = SequenceHint::with_size(len);
        let encoder = self.encoder.encode_sequence(&hint)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let hint = SequenceHint::with_size(len);
        let encoder = self.encoder.encode_sequence(&hint)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant_name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let hint = SequenceHint::with_size(len);
        let encoder = self.encoder.encode_sequence_variant(variant_name, &hint)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let Some(len) = len else {
            return Err(self
                .cx
                .message("Can only serialize maps with known lengths"));
        };

        let hint = MapHint::with_size(len);
        let encoder = self.encoder.encode_map_entries(&hint)?;
        Ok(SerializeMap::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let hint = MapHint::with_size(len);
        let encoder = self.encoder.encode_map(&hint)?;
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
        let hint = MapHint::with_size(len);
        let encoder = self.encoder.encode_map_variant(variant_name, &hint)?;
        Ok(SerializeStructVariant::new(self.cx, encoder))
    }

    #[inline]
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let string = self.cx.collect_string(value)?;
        self.serialize_str(string.as_ref())
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        TypeId::of::<E::Mode>() == TypeId::of::<Text>()
    }
}

#[inline]
fn encode_variant<E, T, F, O>(
    cx: &E::Cx,
    encoder: E,
    variant_tag: &T,
    f: F,
) -> Result<O, <E::Cx as Context>::Error>
where
    <E::Cx as Context>::Error: ser::Error,
    E: Encoder,
    T: ?Sized + Serialize,
    F: FnOnce(
        <E::EncodeVariant as VariantEncoder>::EncodeData<'_>,
    ) -> Result<O, <E::Cx as Context>::Error>,
{
    let mut variant = encoder.encode_variant()?;
    variant_tag.serialize(Serializer::new(cx, variant.encode_tag()?))?;
    let output = f(variant.encode_data()?)?;
    variant.finish_variant()?;
    Ok(output)
}

pub struct SerializeSeq<'a, E>
where
    E: SequenceEncoder,
{
    cx: &'a E::Cx,
    encoder: E,
}

impl<'a, E> SerializeSeq<'a, E>
where
    E: SequenceEncoder,
{
    fn new(cx: &'a E::Cx, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<E> ser::SerializeSeq for SerializeSeq<'_, E>
where
    <E::Cx as Context>::Error: ser::Error,
    E: SequenceEncoder,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let encoder = self.encoder.encode_next()?;
        value.serialize(Serializer::new(self.cx, encoder))?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.finish_sequence()
    }
}

impl<E> ser::SerializeTupleStruct for SerializeSeq<'_, E>
where
    <E::Cx as Context>::Error: ser::Error,
    E: SequenceEncoder,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<E> ser::SerializeTuple for SerializeSeq<'_, E>
where
    <E::Cx as Context>::Error: ser::Error,
    E: SequenceEncoder,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<E> ser::SerializeTupleVariant for SerializeSeq<'_, E>
where
    <E::Cx as Context>::Error: ser::Error,
    E: SequenceEncoder,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

pub struct SerializeMap<'a, E>
where
    E: EntriesEncoder,
{
    cx: &'a E::Cx,
    encoder: E,
}

impl<'a, E> SerializeMap<'a, E>
where
    E: EntriesEncoder,
{
    fn new(cx: &'a E::Cx, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<E> ser::SerializeMap for SerializeMap<'_, E>
where
    <E::Cx as Context>::Error: ser::Error,
    E: EntriesEncoder,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let encoder = self.encoder.encode_entry_key()?;
        key.serialize(Serializer::new(self.cx, encoder))?;
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let encoder = self.encoder.encode_entry_value()?;
        value.serialize(Serializer::new(self.cx, encoder))?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.finish_entries()
    }
}

pub struct SerializeStruct<'a, E>
where
    E: MapEncoder,
{
    cx: &'a E::Cx,
    encoder: E,
}

impl<'a, E> SerializeStruct<'a, E>
where
    E: MapEncoder,
{
    fn new(cx: &'a E::Cx, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<E> ser::SerializeStruct for SerializeStruct<'_, E>
where
    <E::Cx as Context>::Error: ser::Error,
    E: MapEncoder,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let mut field = self.encoder.encode_entry()?;
        field.encode_key()?.encode(key)?;
        value.serialize(Serializer::new(self.cx, field.encode_value()?))?;
        field.finish_entry()?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.finish_map()
    }
}

pub struct SerializeStructVariant<'a, E>
where
    E: MapEncoder,
{
    cx: &'a E::Cx,
    encoder: E,
}

impl<'a, E> SerializeStructVariant<'a, E>
where
    E: MapEncoder,
{
    fn new(cx: &'a E::Cx, encoder: E) -> Self {
        Self { cx, encoder }
    }
}

impl<E> ser::SerializeStructVariant for SerializeStructVariant<'_, E>
where
    <E::Cx as Context>::Error: ser::Error,
    E: MapEncoder,
{
    type Ok = E::Ok;
    type Error = <E::Cx as Context>::Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.encoder.encode_entry_fn(|field| {
            field.encode_key()?.encode(key)?;
            value.serialize(Serializer::new(self.cx, field.encode_value()?))?;
            Ok(())
        })?;

        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.finish_map()
    }
}
