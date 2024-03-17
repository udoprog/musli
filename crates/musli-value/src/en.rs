#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::en::Encoder;
#[cfg(feature = "alloc")]
use musli::en::{
    Encode, MapEncoder, MapEntriesEncoder, MapEntryEncoder, SequenceEncoder, StructEncoder,
    StructFieldEncoder, VariantEncoder,
};
use musli::Context;

use crate::value::{Number, Value};

/// Insert a value into the given receiver.
trait ValueOutput {
    fn write(self, value: Value);
}

impl ValueOutput for &mut Value {
    #[inline]
    fn write(self, value: Value) {
        *self = value;
    }
}

#[cfg(feature = "alloc")]
impl ValueOutput for &mut Vec<Value> {
    #[inline]
    fn write(self, value: Value) {
        self.push(value);
    }
}

/// Writer which writes an optional value that is present.
#[cfg(feature = "alloc")]
pub struct SomeValueWriter<O> {
    output: O,
}

#[cfg(feature = "alloc")]
impl<O> ValueOutput for SomeValueWriter<O>
where
    O: ValueOutput,
{
    fn write(self, value: Value) {
        self.output.write(Value::Option(Some(Box::new(value))));
    }
}

/// Encoder for a single value.
pub struct ValueEncoder<O> {
    output: O,
}

impl<O> ValueEncoder<O> {
    #[inline]
    pub(crate) fn new(output: O) -> Self {
        Self { output }
    }
}

#[musli::encoder]
impl<C, O> Encoder<C> for ValueEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();
    type WithContext<U> = Self where U: Context;
    #[cfg(feature = "alloc")]
    type EncodeSome = ValueEncoder<SomeValueWriter<O>>;
    #[cfg(feature = "alloc")]
    type EncodePack<'this> = SequenceValueEncoder<O> where C: 'this;
    #[cfg(feature = "alloc")]
    type EncodeSequence = SequenceValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type EncodeTuple = SequenceValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type EncodeMap = MapValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type EncodeMapEntries = MapValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type EncodeStruct = MapValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type EncodeVariant = VariantValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type EncodeTupleVariant = VariantSequenceEncoder<O>;
    #[cfg(feature = "alloc")]
    type EncodeStructVariant = VariantStructEncoder<O>;

    #[inline]
    fn with_context<U>(self, _: &C) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context,
    {
        Ok(self)
    }

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be encoded")
    }

    #[inline]
    fn encode_unit(self, _: &C) -> Result<Self::Ok, C::Error> {
        Ok(())
    }

    #[inline]
    fn encode_bool(self, _: &C, b: bool) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Bool(b));
        Ok(())
    }

    #[inline]
    fn encode_char(self, _: &C, c: char) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Char(c));
        Ok(())
    }

    #[inline]
    fn encode_u8(self, _: &C, n: u8) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U8(n)));
        Ok(())
    }

    #[inline]
    fn encode_u16(self, _: &C, n: u16) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U16(n)));
        Ok(())
    }

    #[inline]
    fn encode_u32(self, _: &C, n: u32) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U32(n)));
        Ok(())
    }

    #[inline]
    fn encode_u64(self, _: &C, n: u64) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U64(n)));
        Ok(())
    }

    #[inline]
    fn encode_u128(self, _: &C, n: u128) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U128(n)));
        Ok(())
    }

    #[inline]
    fn encode_i8(self, _: &C, n: i8) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I8(n)));
        Ok(())
    }

    #[inline]
    fn encode_i16(self, _: &C, n: i16) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I16(n)));
        Ok(())
    }

    #[inline]
    fn encode_i32(self, _: &C, n: i32) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I32(n)));
        Ok(())
    }

    #[inline]
    fn encode_i64(self, _: &C, n: i64) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I64(n)));
        Ok(())
    }

    #[inline]
    fn encode_i128(self, _: &C, n: i128) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I128(n)));
        Ok(())
    }

    #[inline]
    fn encode_usize(self, _: &C, n: usize) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::Usize(n)));
        Ok(())
    }

    #[inline]
    fn encode_isize(self, _: &C, n: isize) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::Isize(n)));
        Ok(())
    }

    #[inline]
    fn encode_f32(self, _: &C, n: f32) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::F32(n)));
        Ok(())
    }

    #[inline]
    fn encode_f64(self, _: &C, n: f64) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::F64(n)));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_array<const N: usize>(self, _: &C, array: [u8; N]) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Bytes(array.into()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_bytes(self, _: &C, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Bytes(bytes.to_vec()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_bytes_vectored(self, _: &C, input: &[&[u8]]) -> Result<Self::Ok, C::Error> {
        let mut bytes = Vec::new();

        for b in input {
            bytes.extend_from_slice(b);
        }

        self.output.write(Value::Bytes(bytes));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_string(self, _: &C, string: &str) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::String(string.into()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_some(self, _: &C) -> Result<Self::EncodeSome, C::Error> {
        Ok(ValueEncoder::new(SomeValueWriter {
            output: self.output,
        }))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_none(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Option(None));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_pack(self, _: &'_ C) -> Result<Self::EncodePack<'_>, C::Error> {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_sequence(self, _: &C, _: usize) -> Result<Self::EncodeSequence, C::Error> {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_tuple(self, _: &C, _: usize) -> Result<Self::EncodeTuple, C::Error> {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_map(self, _: &C, _: usize) -> Result<Self::EncodeMap, C::Error> {
        Ok(MapValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_map_entries(self, _: &C, _: usize) -> Result<Self::EncodeMapEntries, C::Error> {
        Ok(MapValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_struct(self, _: &C, _: usize) -> Result<Self::EncodeStruct, C::Error> {
        Ok(MapValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_variant(self, _: &C) -> Result<Self::EncodeVariant, C::Error> {
        Ok(VariantValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_unit_variant<T>(self, cx: &C, tag: &T) -> Result<(), C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = self.encode_variant(cx)?;
        tag.encode(cx, variant.encode_tag(cx)?)?;
        variant.encode_value(cx)?.encode_unit(cx)?;
        variant.end(cx)?;
        Ok(())
    }

    #[inline]
    #[cfg(feature = "alloc")]
    fn encode_tuple_variant<T>(
        self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = Value::Unit;
        tag.encode(cx, ValueEncoder::new(&mut variant))?;
        Ok(VariantSequenceEncoder::new(self.output, variant, len))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_struct_variant<T>(
        self,
        cx: &C,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeStructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = Value::Unit;
        tag.encode(cx, ValueEncoder::new(&mut variant))?;
        Ok(VariantStructEncoder::new(self.output, variant, len))
    }
}

/// A pack encoder.
#[cfg(feature = "alloc")]
pub struct SequenceValueEncoder<O> {
    output: O,
    values: Vec<Value>,
}

#[cfg(feature = "alloc")]
impl<O> SequenceValueEncoder<O> {
    #[inline]
    fn new(output: O) -> Self {
        Self {
            output,
            values: Vec::new(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<C, O> SequenceEncoder<C> for SequenceValueEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();

    type EncodeNext<'this> = ValueEncoder<&'this mut Vec<Value>>
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.values))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Sequence(self.values));
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct MapValueEncoder<O> {
    output: O,
    values: Vec<(Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<O> MapValueEncoder<O> {
    #[inline]
    fn new(output: O) -> Self {
        Self {
            output,
            values: Vec::new(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<C, O> MapEncoder<C> for MapValueEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();
    type EncodeEntry<'this> = PairValueEncoder<'this>
    where
        Self: 'this;

    #[inline]
    fn encode_entry(&mut self, _: &C) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(PairValueEncoder::new(&mut self.values))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<C, O> MapEntriesEncoder<C> for MapValueEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();
    type EncodeMapEntryKey<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;
    type EncodeMapEntryValue<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self, cx: &C) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        self.values.push((Value::Unit, Value::Unit));

        let Some((key, _)) = self.values.last_mut() else {
            return Err(cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(key))
    }

    #[inline]
    fn encode_map_entry_value(
        &mut self,
        cx: &C,
    ) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        let Some((_, value)) = self.values.last_mut() else {
            return Err(cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(value))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<C, O> StructEncoder<C> for MapValueEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();

    type EncodeField<'this> = PairValueEncoder<'this>
    where
        Self: 'this;

    #[inline]
    fn encode_field(&mut self, _: &C) -> Result<Self::EncodeField<'_>, C::Error> {
        Ok(PairValueEncoder::new(&mut self.values))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct PairValueEncoder<'a> {
    output: &'a mut Vec<(Value, Value)>,
    pair: (Value, Value),
}

#[cfg(feature = "alloc")]
impl<'a> PairValueEncoder<'a> {
    #[inline]
    fn new(output: &'a mut Vec<(Value, Value)>) -> Self {
        Self {
            output,
            pair: (Value::Unit, Value::Unit),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, C> MapEntryEncoder<C> for PairValueEncoder<'a>
where
    C: ?Sized + Context,
{
    type Ok = ();
    type EncodeMapKey<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;
    type EncodeMapValue<'this> = ValueEncoder<&'this mut Value> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self, _: &C) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.pair.0))
    }

    #[inline]
    fn encode_map_value(&mut self, _: &C) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.pair.1))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.push(self.pair);
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a, C> StructFieldEncoder<C> for PairValueEncoder<'a>
where
    C: ?Sized + Context,
{
    type Ok = ();
    type EncodeFieldName<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;
    type EncodeFieldValue<'this> = ValueEncoder<&'this mut Value> where Self: 'this;

    #[inline]
    fn encode_field_name(&mut self, _: &C) -> Result<Self::EncodeFieldName<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.pair.0))
    }

    #[inline]
    fn encode_field_value(&mut self, _: &C) -> Result<Self::EncodeFieldValue<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.pair.1))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.push(self.pair);
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct VariantValueEncoder<O> {
    output: O,
    pair: (Value, Value),
}

#[cfg(feature = "alloc")]
impl<O> VariantValueEncoder<O> {
    #[inline]
    fn new(output: O) -> Self {
        Self {
            output,
            pair: (Value::Unit, Value::Unit),
        }
    }
}

#[cfg(feature = "alloc")]
impl<C, O> VariantEncoder<C> for VariantValueEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();
    type EncodeTag<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;
    type EncodeValue<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    #[inline]
    fn encode_tag(&mut self, _: &C) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.pair.0))
    }

    #[inline]
    fn encode_value(&mut self, _: &C) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.pair.1))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new(self.pair)));
        Ok(())
    }
}

/// A variant sequence encoder.
#[cfg(feature = "alloc")]
pub struct VariantSequenceEncoder<O> {
    output: O,
    variant: Value,
    values: Vec<Value>,
}

#[cfg(feature = "alloc")]
impl<O> VariantSequenceEncoder<O> {
    #[inline]
    fn new(output: O, variant: Value, len: usize) -> Self {
        Self {
            output,
            variant,
            values: Vec::with_capacity(len),
        }
    }
}

#[cfg(feature = "alloc")]
impl<C, O> SequenceEncoder<C> for VariantSequenceEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();

    type EncodeNext<'this> = ValueEncoder<&'this mut Vec<Value>>
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self, _: &C) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(ValueEncoder::new(&mut self.values))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new((
            self.variant,
            Value::Sequence(self.values),
        ))));
        Ok(())
    }
}

/// A variant struct encoder.
#[cfg(feature = "alloc")]
pub struct VariantStructEncoder<O> {
    output: O,
    variant: Value,
    fields: Vec<(Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<O> VariantStructEncoder<O> {
    #[inline]
    fn new(output: O, variant: Value, len: usize) -> Self {
        Self {
            output,
            variant,
            fields: Vec::with_capacity(len),
        }
    }
}

#[cfg(feature = "alloc")]
impl<C, O> StructEncoder<C> for VariantStructEncoder<O>
where
    C: ?Sized + Context,
    O: ValueOutput,
{
    type Ok = ();

    type EncodeField<'this> = PairValueEncoder<'this>
    where
        Self: 'this;

    #[inline]
    fn encode_field(&mut self, _: &C) -> Result<Self::EncodeField<'_>, C::Error> {
        Ok(PairValueEncoder::new(&mut self.fields))
    }

    #[inline]
    fn end(self, _: &C) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new((
            self.variant,
            Value::Map(self.fields),
        ))));
        Ok(())
    }
}
