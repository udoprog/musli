#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::en::{Encode, Encoder};
#[cfg(feature = "alloc")]
use musli::en::{
    MapEncoder, MapEntriesEncoder, MapEntryEncoder, SequenceEncoder, StructEncoder,
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
pub struct ValueEncoder<'a, O, C: ?Sized> {
    cx: &'a C,
    output: O,
}

impl<'a, O, C: ?Sized> ValueEncoder<'a, O, C> {
    #[inline]
    pub(crate) fn new(cx: &'a C, output: O) -> Self {
        Self { cx, output }
    }
}

#[musli::encoder]
impl<'a, O, C> Encoder for ValueEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U> = ValueEncoder<'this, O, U> where U: 'this + Context;
    #[cfg(feature = "alloc")]
    type EncodeSome = ValueEncoder<'a, SomeValueWriter<O>, C>;
    #[cfg(feature = "alloc")]
    type EncodePack = SequenceValueEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeSequence = SequenceValueEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeTuple = SequenceValueEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeMap = MapValueEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeMapEntries = MapValueEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeStruct = MapValueEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeVariant = VariantValueEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeTupleVariant = VariantSequenceEncoder<'a, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeStructVariant = VariantStructEncoder<'a, O, C>;

    #[inline]
    fn cx(&self) -> &C {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: &U) -> Result<Self::WithContext<'_, U>, C::Error>
    where
        U: Context,
    {
        Ok(ValueEncoder::new(cx, self.output))
    }

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be encoded")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<Self::Ok, C::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.encode(self.cx, self)
    }

    #[inline]
    fn encode_unit(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }

    #[inline]
    fn encode_bool(self, b: bool) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Bool(b));
        Ok(())
    }

    #[inline]
    fn encode_char(self, c: char) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Char(c));
        Ok(())
    }

    #[inline]
    fn encode_u8(self, n: u8) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U8(n)));
        Ok(())
    }

    #[inline]
    fn encode_u16(self, n: u16) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U16(n)));
        Ok(())
    }

    #[inline]
    fn encode_u32(self, n: u32) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U32(n)));
        Ok(())
    }

    #[inline]
    fn encode_u64(self, n: u64) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U64(n)));
        Ok(())
    }

    #[inline]
    fn encode_u128(self, n: u128) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::U128(n)));
        Ok(())
    }

    #[inline]
    fn encode_i8(self, n: i8) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I8(n)));
        Ok(())
    }

    #[inline]
    fn encode_i16(self, n: i16) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I16(n)));
        Ok(())
    }

    #[inline]
    fn encode_i32(self, n: i32) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I32(n)));
        Ok(())
    }

    #[inline]
    fn encode_i64(self, n: i64) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I64(n)));
        Ok(())
    }

    #[inline]
    fn encode_i128(self, n: i128) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::I128(n)));
        Ok(())
    }

    #[inline]
    fn encode_usize(self, n: usize) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::Usize(n)));
        Ok(())
    }

    #[inline]
    fn encode_isize(self, n: isize) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::Isize(n)));
        Ok(())
    }

    #[inline]
    fn encode_f32(self, n: f32) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::F32(n)));
        Ok(())
    }

    #[inline]
    fn encode_f64(self, n: f64) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Number(Number::F64(n)));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_array<const N: usize>(self, array: &[u8; N]) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Bytes(array.into()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_bytes(self, bytes: &[u8]) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Bytes(bytes.to_vec()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_bytes_vectored<I>(self, len: usize, vectors: I) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let mut bytes = Vec::with_capacity(len);

        for b in vectors {
            bytes.extend_from_slice(b.as_ref());
        }

        self.output.write(Value::Bytes(bytes));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_string(self, string: &str) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::String(string.into()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, C::Error> {
        Ok(ValueEncoder::new(
            self.cx,
            SomeValueWriter {
                output: self.output,
            },
        ))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_none(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Option(None));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        Ok(SequenceValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_sequence(self, _: usize) -> Result<Self::EncodeSequence, C::Error> {
        Ok(SequenceValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_tuple(self, _: usize) -> Result<Self::EncodeTuple, C::Error> {
        Ok(SequenceValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_map(self, _: usize) -> Result<Self::EncodeMap, C::Error> {
        Ok(MapValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_map_entries(self, _: usize) -> Result<Self::EncodeMapEntries, C::Error> {
        Ok(MapValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_struct(self, _: usize) -> Result<Self::EncodeStruct, C::Error> {
        Ok(MapValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, C::Error> {
        Ok(VariantValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_unit_variant<T>(self, tag: &T) -> Result<(), C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = self.encode_variant()?;
        variant.encode_tag()?.encode(tag)?;
        variant.encode_value()?.encode_unit()?;
        variant.end()?;
        Ok(())
    }

    #[inline]
    #[cfg(feature = "alloc")]
    fn encode_tuple_variant<T>(
        self,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = Value::Unit;
        ValueEncoder::new(self.cx, &mut variant).encode(tag)?;
        Ok(VariantSequenceEncoder::new(
            self.cx,
            self.output,
            variant,
            len,
        ))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_struct_variant<T>(
        self,
        tag: &T,
        len: usize,
    ) -> Result<Self::EncodeStructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = Value::Unit;
        ValueEncoder::new(self.cx, &mut variant).encode(tag)?;
        Ok(VariantStructEncoder::new(
            self.cx,
            self.output,
            variant,
            len,
        ))
    }
}

/// A pack encoder.
#[cfg(feature = "alloc")]
pub struct SequenceValueEncoder<'a, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    values: Vec<Value>,
}

#[cfg(feature = "alloc")]
impl<'a, O, C: ?Sized> SequenceValueEncoder<'a, O, C> {
    #[inline]
    fn new(cx: &'a C, output: O) -> Self {
        Self {
            cx,
            output,
            values: Vec::new(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, O, C> SequenceEncoder for SequenceValueEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeNext<'this> = ValueEncoder<'a, &'this mut Vec<Value>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Sequence(self.values));
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct MapValueEncoder<'a, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    values: Vec<(Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<'a, O, C: ?Sized> MapValueEncoder<'a, O, C> {
    #[inline]
    fn new(cx: &'a C, output: O) -> Self {
        Self {
            cx,
            output,
            values: Vec::new(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, O, C> MapEncoder for MapValueEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeEntry<'this> = PairValueEncoder<'this, C>
    where
        Self: 'this;

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a, O, C> MapEntriesEncoder for MapValueEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = ValueEncoder<'a, &'this mut Value, C>
    where
        Self: 'this;
    type EncodeMapEntryValue<'this> = ValueEncoder<'a, &'this mut Value, C>
    where
        Self: 'this;

    #[inline]
    fn encode_map_entry_key(&mut self) -> Result<Self::EncodeMapEntryKey<'_>, C::Error> {
        self.values.push((Value::Unit, Value::Unit));

        let Some((key, _)) = self.values.last_mut() else {
            return Err(self.cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(self.cx, key))
    }

    #[inline]
    fn encode_map_entry_value(&mut self) -> Result<Self::EncodeMapEntryValue<'_>, C::Error> {
        let Some((_, value)) = self.values.last_mut() else {
            return Err(self.cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(self.cx, value))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a, O, C> StructEncoder for MapValueEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeField<'this> = PairValueEncoder<'this, C>
    where
        Self: 'this;

    #[inline]
    fn encode_field(&mut self) -> Result<Self::EncodeField<'_>, C::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct PairValueEncoder<'a, C: ?Sized> {
    cx: &'a C,
    output: &'a mut Vec<(Value, Value)>,
    pair: (Value, Value),
}

#[cfg(feature = "alloc")]
impl<'a, C: ?Sized> PairValueEncoder<'a, C> {
    #[inline]
    fn new(cx: &'a C, output: &'a mut Vec<(Value, Value)>) -> Self {
        Self {
            cx,
            output,
            pair: (Value::Unit, Value::Unit),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, C> MapEntryEncoder for PairValueEncoder<'a, C>
where
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = ValueEncoder<'a, &'this mut Value, C>
    where
        Self: 'this;
    type EncodeMapValue<'this> = ValueEncoder<'a, &'this mut Value, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.0))
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.1))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.push(self.pair);
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a, C> StructFieldEncoder for PairValueEncoder<'a, C>
where
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeFieldName<'this> = ValueEncoder<'a, &'this mut Value, C>
    where
        Self: 'this;
    type EncodeFieldValue<'this> = ValueEncoder<'a, &'this mut Value, C> where Self: 'this;

    #[inline]
    fn encode_field_name(&mut self) -> Result<Self::EncodeFieldName<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.0))
    }

    #[inline]
    fn encode_field_value(&mut self) -> Result<Self::EncodeFieldValue<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.1))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.push(self.pair);
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct VariantValueEncoder<'a, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    pair: (Value, Value),
}

#[cfg(feature = "alloc")]
impl<'a, O, C: ?Sized> VariantValueEncoder<'a, O, C> {
    #[inline]
    fn new(cx: &'a C, output: O) -> Self {
        Self {
            cx,
            output,
            pair: (Value::Unit, Value::Unit),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, O, C> VariantEncoder for VariantValueEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = ValueEncoder<'a, &'this mut Value, C>
    where
        Self: 'this;
    type EncodeValue<'this> = ValueEncoder<'a, &'this mut Value, C>
    where
        Self: 'this;

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.0))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.1))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new(self.pair)));
        Ok(())
    }
}

/// A variant sequence encoder.
#[cfg(feature = "alloc")]
pub struct VariantSequenceEncoder<'a, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    variant: Value,
    values: Vec<Value>,
}

#[cfg(feature = "alloc")]
impl<'a, O, C: ?Sized> VariantSequenceEncoder<'a, O, C> {
    #[inline]
    fn new(cx: &'a C, output: O, variant: Value, len: usize) -> Self {
        Self {
            cx,
            output,
            variant,
            values: Vec::with_capacity(len),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, O, C> SequenceEncoder for VariantSequenceEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeNext<'this> = ValueEncoder<'a, &'this mut Vec<Value>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new((
            self.variant,
            Value::Sequence(self.values),
        ))));
        Ok(())
    }
}

/// A variant struct encoder.
#[cfg(feature = "alloc")]
pub struct VariantStructEncoder<'a, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    variant: Value,
    fields: Vec<(Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<'a, O, C: ?Sized> VariantStructEncoder<'a, O, C> {
    #[inline]
    fn new(cx: &'a C, output: O, variant: Value, len: usize) -> Self {
        Self {
            cx,
            output,
            variant,
            fields: Vec::with_capacity(len),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, O, C> StructEncoder for VariantStructEncoder<'a, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeField<'this> = PairValueEncoder<'this, C>
    where
        Self: 'this;

    #[inline]
    fn encode_field(&mut self) -> Result<Self::EncodeField<'_>, C::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.fields))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new((
            self.variant,
            Value::Map(self.fields),
        ))));
        Ok(())
    }
}
