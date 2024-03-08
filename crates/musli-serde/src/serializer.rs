use std::fmt;
use std::marker::PhantomData;

use musli::en::{PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::{Context, Encode, Encoder, Mode};

use serde::ser::{self, Serialize};

pub struct Serializer<'a, C, E, M> {
    cx: &'a C,
    encoder: E,
    mode: PhantomData<M>,
}

impl<'a, C, E, M> Serializer<'a, C, E, M> {
    /// Construct a new deserializer out of an encoder.
    pub fn new(cx: &'a C, encoder: E) -> Self {
        Self {
            cx,
            encoder,
            mode: PhantomData,
        }
    }
}

impl<'a, C, E, M> ser::Serializer for Serializer<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: Encoder,
    M: Mode,
{
    type Ok = E::Ok;
    type Error = C::Error;

    type SerializeSeq = SerializeSeq<'a, C, E::Sequence, M>;
    type SerializeTuple = SerializeSeq<'a, C, E::Tuple, M>;
    type SerializeTupleStruct = SerializeTupleStruct<'a, C, E::Struct, M>;
    type SerializeTupleVariant = SerializeSeq<'a, C, E::TupleVariant, M>;
    type SerializeMap = SerializeMap<'a, C, E::MapPairs, M>;
    type SerializeStruct = SerializeStruct<'a, C, E::Struct, M>;
    type SerializeStructVariant = SerializeStructVariant<'a, C, E::StructVariant, M>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_bool(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i8(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i16(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i32(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_i64(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u8(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u16(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u32(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_u64(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_f32(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_f64(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_char(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_string(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_bytes(self.cx.adapt(), v)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_none(self.cx.adapt())
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.encode_some(self.cx.adapt())?;
        value.serialize(Serializer::<_, _, M>::new(self.cx.adapt(), encoder))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.encode_unit(self.cx.adapt())
    }

    #[inline]
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        let encoder = self.encoder.encode_struct(self.cx.adapt(), 0)?;
        encoder.end(self.cx.adapt())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        encode_variant::<_, _, M, _, _, _>(self.cx, self.encoder, &variant_index, |encoder| {
            encoder.encode_unit(self.cx.adapt())
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
        encode_newtype::<_, _, M, _>(self.cx.adapt(), self.encoder, value)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        encode_variant::<_, _, M, _, _, _>(self.cx, self.encoder, &variant_index, move |encoder| {
            encode_newtype::<_, _, M, _>(self.cx, encoder, value)
        })
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let Some(len) = len else {
            return Err(ser::Error::custom(
                "Can only encode sequences with known lengths",
            ));
        };

        let encoder = self.encoder.encode_sequence(self.cx.adapt(), len)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let encoder = self.encoder.encode_tuple(self.cx.adapt(), len)?;
        Ok(SerializeSeq::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let encoder = self.encoder.encode_struct(self.cx.adapt(), len)?;
        Ok(SerializeTupleStruct::<_, _, M>::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let encoder =
            self.encoder
                .encode_tuple_variant::<M, _, _>(self.cx.adapt(), &variant_index, len)?;
        Ok(SerializeSeq::<_, _, M>::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let Some(len) = len else {
            return Err(ser::Error::custom(
                "Can only encode maps with known lengths",
            ));
        };

        let encoder = self.encoder.encode_map_pairs(self.cx.adapt(), len)?;
        Ok(SerializeMap::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let encoder = self.encoder.encode_struct(self.cx.adapt(), len)?;
        Ok(SerializeStruct::new(self.cx, encoder))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let encoder =
            self.encoder
                .encode_struct_variant::<M, _, _>(self.cx.adapt(), &variant_index, len)?;
        Ok(SerializeStructVariant::<_, _, M>::new(self.cx, encoder))
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
fn encode_variant<C, E, M, T, F, O>(
    cx: &C,
    encoder: E,
    variant_tag: &T,
    f: F,
) -> Result<O, C::Error>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: Encoder,
    M: Mode,
    T: Serialize,
    F: FnOnce(<E::Variant as VariantEncoder>::Variant<'_>) -> Result<O, C::Error>,
{
    let mut variant = encoder.encode_variant(cx.adapt())?;

    let tag = variant.tag(cx.adapt())?;
    variant_tag.serialize(Serializer::<_, _, M>::new(cx, tag))?;

    let value = variant.variant(cx.adapt())?;
    let output = f(value)?;

    variant.end(cx.adapt())?;
    Ok(output)
}

#[inline]
fn encode_newtype<C, E, M, T>(cx: &C, encoder: E, value: &T) -> Result<E::Ok, C::Error>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: Encoder,
    M: Mode,
    T: ?Sized + Serialize,
{
    let mut encoder = encoder.encode_struct(cx.adapt(), 1)?;

    let mut field = encoder.next(cx.adapt())?;

    let k = field.first(cx.adapt())?;
    Encode::<M>::encode(&0usize, cx.adapt(), k)?;

    let v = field.second(cx.adapt())?;
    value.serialize(Serializer::<_, _, M>::new(cx, v))?;

    field.end(cx.adapt())?;
    encoder.end(cx.adapt())
}

pub struct SerializeSeq<'a, C, E, M> {
    cx: &'a C,
    encoder: E,
    mode: PhantomData<M>,
}

impl<'a, C, E, M> SerializeSeq<'a, C, E, M> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self {
            cx,
            encoder,
            mode: PhantomData,
        }
    }
}

impl<'a, C, E, M> ser::SerializeSeq for SerializeSeq<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: SequenceEncoder,
    M: Mode,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.next(self.cx.adapt())?;
        value.serialize(Serializer::<_, _, M>::new(self.cx.adapt(), encoder))?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx.adapt())
    }
}

impl<'a, C, E, M> ser::SerializeTuple for SerializeSeq<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: SequenceEncoder,
    M: Mode,
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

impl<'a, C, E, M> ser::SerializeTupleVariant for SerializeSeq<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: SequenceEncoder,
    M: Mode,
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

pub struct SerializeTupleStruct<'a, C, E, M> {
    cx: &'a C,
    encoder: E,
    field: usize,
    mode: PhantomData<M>,
}

impl<'a, C, E, M> SerializeTupleStruct<'a, C, E, M> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self {
            cx,
            encoder,
            field: 0,
            mode: PhantomData,
        }
    }
}

impl<'a, C, E, M> ser::SerializeTupleStruct for SerializeTupleStruct<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: PairsEncoder,
    M: Mode,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let mut field = self.encoder.next(self.cx.adapt())?;

        let k = field.first(self.cx.adapt())?;
        Encode::<M>::encode(&self.field, self.cx.adapt(), k)?;

        let v = field.second(self.cx.adapt())?;
        value.serialize(Serializer::<_, _, M>::new(self.cx.adapt(), v))?;

        field.end(self.cx.adapt())?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx.adapt())
    }
}

pub struct SerializeMap<'a, C, E, M> {
    cx: &'a C,
    encoder: E,
    mode: PhantomData<M>,
}

impl<'a, C, E, M> SerializeMap<'a, C, E, M> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self {
            cx,
            encoder,
            mode: PhantomData,
        }
    }
}

impl<'a, C, E, M> ser::SerializeMap for SerializeMap<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: PairEncoder,
    M: Mode,
{
    type Ok = E::Ok;
    type Error = C::Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.first(self.cx.adapt())?;
        key.serialize(Serializer::<_, _, M>::new(self.cx.adapt(), encoder))?;
        Ok(())
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        let encoder = self.encoder.second(self.cx.adapt())?;
        value.serialize(Serializer::<_, _, M>::new(self.cx.adapt(), encoder))?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx.adapt())
    }
}

pub struct SerializeStruct<'a, C, E, M> {
    cx: &'a C,
    encoder: E,
    mode: PhantomData<M>,
}

impl<'a, C, E, M> SerializeStruct<'a, C, E, M> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self {
            cx,
            encoder,
            mode: PhantomData,
        }
    }
}

impl<'a, C, E, M> ser::SerializeStruct for SerializeStruct<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: PairsEncoder,
    M: Mode,
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
        let mut field = self.encoder.next(self.cx.adapt())?;
        let k = field.first(self.cx.adapt())?;
        Encode::<M>::encode(key, self.cx.adapt(), k)?;
        let v = field.second(self.cx.adapt())?;
        value.serialize(Serializer::<_, _, M>::new(self.cx.adapt(), v))?;
        field.end(self.cx.adapt())?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx.adapt())
    }
}

pub struct SerializeStructVariant<'a, C, E, M> {
    cx: &'a C,
    encoder: E,
    _mode: PhantomData<M>,
}

impl<'a, C, E, M> SerializeStructVariant<'a, C, E, M> {
    fn new(cx: &'a C, encoder: E) -> Self {
        Self {
            cx,
            encoder,
            _mode: PhantomData,
        }
    }
}

impl<'a, C, E, M> ser::SerializeStructVariant for SerializeStructVariant<'a, C, E, M>
where
    C: Context<Input = E::Error>,
    C::Error: ser::Error,
    E: PairsEncoder,
    M: Mode,
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
        let mut field = self.encoder.next(self.cx.adapt())?;
        let k = field.first(self.cx.adapt())?;
        Encode::<M>::encode(key, self.cx.adapt(), k)?;
        let v = field.second(self.cx.adapt())?;
        value.serialize(Serializer::<_, _, M>::new(self.cx.adapt(), v))?;
        field.end(self.cx.adapt())?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.end(self.cx.adapt())
    }
}
