use core::fmt;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::en::{Encode, Encoder};
#[cfg(feature = "alloc")]
use musli::en::{
    MapEncoder, MapEntriesEncoder, MapEntryEncoder, PackEncoder, SequenceEncoder, TupleEncoder,
    VariantEncoder,
};
#[cfg(feature = "alloc")]
use musli::hint::{MapHint, SequenceHint, StructHint, TupleHint};
#[cfg(feature = "alloc")]
use musli::Buf;
use musli::Context;
#[cfg(feature = "alloc")]
use musli_storage::en::StorageEncoder;
#[cfg(feature = "alloc")]
use musli_utils::writer::BufWriter;
use musli_utils::Options;

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
pub struct ValueEncoder<'a, const OPT: Options, O, C: ?Sized> {
    cx: &'a C,
    output: O,
}

impl<'a, const OPT: Options, O, C: ?Sized> ValueEncoder<'a, OPT, O, C> {
    #[inline]
    pub(crate) fn new(cx: &'a C, output: O) -> Self {
        Self { cx, output }
    }
}

#[musli::encoder]
impl<'a, const OPT: Options, O, C> Encoder for ValueEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = C::Mode;
    type WithContext<'this, U> = ValueEncoder<'this, OPT, O, U> where U: 'this + Context;
    #[cfg(feature = "alloc")]
    type EncodeSome = ValueEncoder<'a, OPT, SomeValueWriter<O>, C>;
    #[cfg(feature = "alloc")]
    type EncodePack = PackValueEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeSequence = SequenceValueEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeTuple = SequenceValueEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeMap = MapValueEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeMapEntries = MapValueEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeStruct = MapValueEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeVariant = VariantValueEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeTupleVariant = VariantSequenceEncoder<'a, OPT, O, C>;
    #[cfg(feature = "alloc")]
    type EncodeStructVariant = VariantStructEncoder<'a, OPT, O, C>;

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

    #[inline]
    fn collect_string<T>(self, value: &T) -> Result<Self::Ok, <Self::Cx as Context>::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let buf = self.cx.collect_string(value)?;
        self.encode_string(buf.as_ref())
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
        PackValueEncoder::new(self.cx, self.output)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_sequence(self, _: &SequenceHint) -> Result<Self::EncodeSequence, C::Error> {
        Ok(SequenceValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_tuple(self, _: &TupleHint) -> Result<Self::EncodeTuple, C::Error> {
        Ok(SequenceValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_map(self, _: &MapHint) -> Result<Self::EncodeMap, C::Error> {
        Ok(MapValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_map_entries(self, _: &MapHint) -> Result<Self::EncodeMapEntries, C::Error> {
        Ok(MapValueEncoder::new(self.cx, self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_struct(self, _: &StructHint) -> Result<Self::EncodeStruct, C::Error> {
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
        variant.finish_variant()?;
        Ok(())
    }

    #[inline]
    #[cfg(feature = "alloc")]
    fn encode_tuple_variant<T>(
        self,
        tag: &T,
        hint: &TupleHint,
    ) -> Result<Self::EncodeTupleVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = Value::Unit;
        ValueEncoder::<OPT, _, _>::new(self.cx, &mut variant).encode(tag)?;

        Ok(VariantSequenceEncoder::new(
            self.cx,
            self.output,
            variant,
            hint.size,
        ))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_struct_variant<T>(
        self,
        tag: &T,
        hint: &StructHint,
    ) -> Result<Self::EncodeStructVariant, C::Error>
    where
        T: ?Sized + Encode<C::Mode>,
    {
        let mut variant = Value::Unit;
        ValueEncoder::<OPT, _, _>::new(self.cx, &mut variant).encode(tag)?;

        Ok(VariantStructEncoder::new(
            self.cx,
            self.output,
            variant,
            hint.size,
        ))
    }
}

/// A sequence encoder.
#[cfg(feature = "alloc")]
pub struct SequenceValueEncoder<'a, const OPT: Options, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    values: Vec<Value>,
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C: ?Sized> SequenceValueEncoder<'a, OPT, O, C> {
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
impl<'a, const OPT: Options, O, C> SequenceEncoder for SequenceValueEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeElement<'this> = ValueEncoder<'a, OPT, &'this mut Vec<Value>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_element(&mut self) -> Result<Self::EncodeElement<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Sequence(self.values));
        Ok(())
    }
}

/// A pack encoder.
#[cfg(feature = "alloc")]
pub struct PackValueEncoder<'a, const OPT: Options, O, C>
where
    C: ?Sized + Context,
{
    cx: &'a C,
    output: O,
    writer: BufWriter<C::Buf<'a>>,
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C> PackValueEncoder<'a, OPT, O, C>
where
    C: ?Sized + Context,
{
    #[inline]
    fn new(cx: &'a C, output: O) -> Result<Self, C::Error> {
        let Some(buf) = cx.alloc() else {
            return Err(cx.message("Failed to allocate buffer"));
        };

        Ok(Self {
            cx,
            output,
            writer: BufWriter::new(buf),
        })
    }
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C> PackEncoder for PackValueEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodePacked<'this> = StorageEncoder<'a, &'this mut BufWriter<C::Buf<'a>>, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_packed(&mut self) -> Result<Self::EncodePacked<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.writer))
    }

    #[inline]
    fn finish_pack(self) -> Result<Self::Ok, C::Error> {
        let buf = self.writer.into_inner();
        self.output.write(Value::Bytes(buf.as_slice().into()));
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C> TupleEncoder for SequenceValueEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeTupleField<'this> = ValueEncoder<'a, OPT, &'this mut Vec<Value>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_tuple_field(&mut self) -> Result<Self::EncodeTupleField<'_>, C::Error> {
        SequenceEncoder::encode_element(self)
    }

    #[inline]
    fn finish_tuple(self) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::finish_sequence(self)
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct MapValueEncoder<'a, const OPT: Options, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    values: Vec<(Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C: ?Sized> MapValueEncoder<'a, OPT, O, C> {
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
impl<'a, const OPT: Options, O, C> MapEncoder for MapValueEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntry<'this> = PairValueEncoder<'this, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_map_entry(&mut self) -> Result<Self::EncodeMapEntry<'_>, C::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C> MapEntriesEncoder for MapValueEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapEntryKey<'this> = ValueEncoder<'a, OPT, &'this mut Value, C>
    where
        Self: 'this;
    type EncodeMapEntryValue<'this> = ValueEncoder<'a, OPT, &'this mut Value, C>
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
    fn finish_map_entries(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct PairValueEncoder<'a, const OPT: Options, C: ?Sized> {
    cx: &'a C,
    output: &'a mut Vec<(Value, Value)>,
    pair: (Value, Value),
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, C: ?Sized> PairValueEncoder<'a, OPT, C> {
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
impl<'a, const OPT: Options, C> MapEntryEncoder for PairValueEncoder<'a, OPT, C>
where
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeMapKey<'this> = ValueEncoder<'a, OPT, &'this mut Value, C>
    where
        Self: 'this;
    type EncodeMapValue<'this> = ValueEncoder<'a, OPT, &'this mut Value, C> where Self: 'this;

    #[inline]
    fn encode_map_key(&mut self) -> Result<Self::EncodeMapKey<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.0))
    }

    #[inline]
    fn encode_map_value(&mut self) -> Result<Self::EncodeMapValue<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.1))
    }

    #[inline]
    fn finish_map_entry(self) -> Result<Self::Ok, C::Error> {
        self.output.push(self.pair);
        Ok(())
    }
}

/// A pairs encoder.
#[cfg(feature = "alloc")]
pub struct VariantValueEncoder<'a, const OPT: Options, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    pair: (Value, Value),
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C: ?Sized> VariantValueEncoder<'a, OPT, O, C> {
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
impl<'a, const OPT: Options, O, C> VariantEncoder for VariantValueEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();
    type EncodeTag<'this> = ValueEncoder<'a, OPT, &'this mut Value, C>
    where
        Self: 'this;
    type EncodeValue<'this> = ValueEncoder<'a, OPT, &'this mut Value, C>
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
    fn finish_variant(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new(self.pair)));
        Ok(())
    }
}

/// A variant sequence encoder.
#[cfg(feature = "alloc")]
pub struct VariantSequenceEncoder<'a, const OPT: Options, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    variant: Value,
    values: Vec<Value>,
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C: ?Sized> VariantSequenceEncoder<'a, OPT, O, C> {
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
impl<'a, const OPT: Options, O, C> SequenceEncoder for VariantSequenceEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeElement<'this> = ValueEncoder<'a, OPT, &'this mut Vec<Value>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_element(&mut self) -> Result<Self::EncodeElement<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new((
            self.variant,
            Value::Sequence(self.values),
        ))));
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C> TupleEncoder for VariantSequenceEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeTupleField<'this> = ValueEncoder<'a, OPT, &'this mut Vec<Value>, C>
    where
        Self: 'this;

    #[inline]
    fn encode_tuple_field(&mut self) -> Result<Self::EncodeTupleField<'_>, C::Error> {
        SequenceEncoder::encode_element(self)
    }

    #[inline]
    fn finish_tuple(self) -> Result<Self::Ok, C::Error> {
        SequenceEncoder::finish_sequence(self)
    }
}

/// A variant struct encoder.
#[cfg(feature = "alloc")]
pub struct VariantStructEncoder<'a, const OPT: Options, O, C: ?Sized> {
    cx: &'a C,
    output: O,
    variant: Value,
    fields: Vec<(Value, Value)>,
}

#[cfg(feature = "alloc")]
impl<'a, const OPT: Options, O, C: ?Sized> VariantStructEncoder<'a, OPT, O, C> {
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
impl<'a, const OPT: Options, O, C> MapEncoder for VariantStructEncoder<'a, OPT, O, C>
where
    O: ValueOutput,
    C: ?Sized + Context,
{
    type Cx = C;
    type Ok = ();

    type EncodeMapEntry<'this> = PairValueEncoder<'this, OPT, C>
    where
        Self: 'this;

    #[inline]
    fn encode_map_entry(&mut self) -> Result<Self::EncodeMapEntry<'_>, C::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.fields))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        self.output.write(Value::Variant(Box::new((
            self.variant,
            Value::Map(self.fields),
        ))));
        Ok(())
    }
}
