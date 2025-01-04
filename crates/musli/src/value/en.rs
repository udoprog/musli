#![allow(clippy::type_complexity)]

use core::marker::PhantomData;

use crate::alloc::{Box, String, Vec};
use crate::en::{Encode, Encoder};
use crate::en::{EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder};
use crate::hint::{MapHint, SequenceHint};
use crate::storage::en::StorageEncoder;
use crate::writer::BufWriter;
use crate::{Allocator, Context, Options};

use super::value::{Number, Value};

/// Insert a value into the given receiver.
trait ValueOutput<A>
where
    A: Allocator,
{
    /// Write a value into the receiver.
    fn write<C>(self, cx: C, value: Value<A>) -> Result<(), C::Error>
    where
        C: Context<Allocator = A>;
}

impl<A> ValueOutput<A> for &mut Value<A>
where
    A: Allocator,
{
    #[inline]
    fn write<C>(self, _: C, value: Value<A>) -> Result<(), C::Error>
    where
        C: Context<Allocator = A>,
    {
        *self = value;
        Ok(())
    }
}

impl<A> ValueOutput<A> for &mut Vec<Value<A>, A>
where
    A: Allocator,
{
    #[inline]
    fn write<C>(self, cx: C, value: Value<A>) -> Result<(), C::Error>
    where
        C: Context<Allocator = A>,
    {
        self.push(value).map_err(cx.map())
    }
}

/// Writer which writes an optional value that is present.
pub struct SomeValueWriter<O> {
    output: O,
}

impl<O, A> ValueOutput<A> for SomeValueWriter<O>
where
    O: ValueOutput<A>,
    A: Allocator,
{
    #[inline]
    fn write<C>(self, cx: C, value: Value<A>) -> Result<(), C::Error>
    where
        C: Context<Allocator = A>,
    {
        let value = Box::new_in(value, cx.alloc()).map_err(cx.map())?;
        self.output.write(cx, Value::Option(Some(value)))?;
        Ok(())
    }
}

/// Encoder for a single value.
pub struct ValueEncoder<const OPT: Options, O, C, M> {
    cx: C,
    output: O,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, O, C, M> ValueEncoder<OPT, O, C, M> {
    #[inline]
    pub(crate) fn new(cx: C, output: O) -> Self {
        Self {
            cx,
            output,
            _marker: PhantomData,
        }
    }
}

#[crate::encoder(crate)]
impl<const OPT: Options, O, C, M> Encoder for ValueEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Ok = ();
    type Mode = M;
    type WithContext<U>
        = ValueEncoder<OPT, O, U, M>
    where
        U: Context<Allocator = <Self::Cx as Context>::Allocator>;
    type EncodeSome = ValueEncoder<OPT, SomeValueWriter<O>, C, M>;
    type EncodePack = PackValueEncoder<OPT, O, C, M>;
    type EncodeSequence = SequenceValueEncoder<OPT, O, C, M>;
    type EncodeMap = MapValueEncoder<OPT, O, C, M>;
    type EncodeMapEntries = MapValueEncoder<OPT, O, C, M>;
    type EncodeVariant = VariantValueEncoder<OPT, O, C, M>;
    type EncodeSequenceVariant = VariantSequenceEncoder<OPT, O, C, M>;
    type EncodeMapVariant = VariantStructEncoder<OPT, O, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn with_context<U>(self, cx: U) -> Result<Self::WithContext<U>, C::Error>
    where
        U: Context<Allocator = <Self::Cx as Context>::Allocator>,
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
        value.as_encode().encode(self)
    }

    #[inline]
    fn encode_empty(self) -> Result<Self::Ok, C::Error> {
        Ok(())
    }

    #[inline]
    fn encode_bool(self, b: bool) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Bool(b))?;
        Ok(())
    }

    #[inline]
    fn encode_char(self, c: char) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Char(c))?;
        Ok(())
    }

    #[inline]
    fn encode_u8(self, n: u8) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::U8(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u16(self, n: u16) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::U16(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u32(self, n: u32) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::U32(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u64(self, n: u64) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::U64(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u128(self, n: u128) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::U128(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i8(self, n: i8) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::I8(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i16(self, n: i16) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::I16(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i32(self, n: i32) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::I32(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i64(self, n: i64) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::I64(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i128(self, n: i128) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::I128(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_f32(self, n: f32) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::F32(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_f64(self, n: f64) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Number(Number::F64(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_usize(self, n: usize) -> Result<Self::Ok, C::Error> {
        self.output
            .write(self.cx, Value::Number(Number::Usize(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_isize(self, n: isize) -> Result<Self::Ok, C::Error> {
        self.output
            .write(self.cx, Value::Number(Number::Isize(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: &[u8; N]) -> Result<Self::Ok, C::Error> {
        let mut bytes =
            Vec::with_capacity_in(array.len(), self.cx.alloc()).map_err(self.cx.map())?;
        bytes.extend_from_slice(array).map_err(self.cx.map())?;
        self.output.write(self.cx, Value::Bytes(bytes))?;
        Ok(())
    }

    #[inline]
    fn encode_bytes(self, b: &[u8]) -> Result<Self::Ok, C::Error> {
        let mut bytes = Vec::with_capacity_in(b.len(), self.cx.alloc()).map_err(self.cx.map())?;
        bytes.extend_from_slice(b).map_err(self.cx.map())?;
        self.output.write(self.cx, Value::Bytes(bytes))?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(self, len: usize, vectors: I) -> Result<Self::Ok, C::Error>
    where
        I: IntoIterator<Item: AsRef<[u8]>>,
    {
        let mut bytes = Vec::with_capacity_in(len, self.cx.alloc()).map_err(self.cx.map())?;

        for b in vectors {
            bytes.extend_from_slice(b.as_ref()).map_err(self.cx.map())?;
        }

        self.output.write(self.cx, Value::Bytes(bytes))?;
        Ok(())
    }

    #[inline]
    fn encode_string(self, s: &str) -> Result<Self::Ok, C::Error> {
        let mut string = String::new_in(self.cx.alloc());
        string.push_str(s).map_err(self.cx.map())?;
        self.output.write(self.cx, Value::String(string))?;
        Ok(())
    }

    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, C::Error> {
        Ok(ValueEncoder::new(
            self.cx,
            SomeValueWriter {
                output: self.output,
            },
        ))
    }

    #[inline]
    fn encode_none(self) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Option(None))?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, C::Error> {
        PackValueEncoder::new(self.cx, self.output)
    }

    #[inline]
    fn encode_sequence(self, _: &SequenceHint) -> Result<Self::EncodeSequence, C::Error> {
        Ok(SequenceValueEncoder::new(self.cx, self.output))
    }

    #[inline]
    fn encode_map(self, _: &MapHint) -> Result<Self::EncodeMap, C::Error> {
        MapValueEncoder::new(self.cx, self.output)
    }

    #[inline]
    fn encode_map_entries(self, _: &MapHint) -> Result<Self::EncodeMapEntries, C::Error> {
        MapValueEncoder::new(self.cx, self.output)
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, C::Error> {
        Ok(VariantValueEncoder::new(self.cx, self.output))
    }

    #[inline]
    fn encode_unit_variant<T>(self, tag: &T) -> Result<(), C::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        let mut variant = self.encode_variant()?;
        variant.encode_tag()?.encode(tag)?;
        variant.encode_data()?.encode_empty()?;
        variant.finish_variant()?;
        Ok(())
    }

    #[inline]
    fn encode_sequence_variant<T>(
        self,
        tag: &T,
        hint: &SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, C::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        let mut variant = Value::Unit;
        ValueEncoder::<OPT, _, _, Self::Mode>::new(self.cx, &mut variant).encode(tag)?;
        VariantSequenceEncoder::new(self.cx, self.output, variant, hint.size)
    }

    #[inline]
    fn encode_map_variant<T>(
        self,
        tag: &T,
        hint: &MapHint,
    ) -> Result<Self::EncodeMapVariant, C::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        let mut variant = Value::Unit;
        ValueEncoder::<OPT, _, _, Self::Mode>::new(self.cx, &mut variant).encode(tag)?;
        VariantStructEncoder::new(self.cx, self.output, variant, hint.size)
    }
}

/// A sequence encoder.
pub struct SequenceValueEncoder<const OPT: Options, O, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: O,
    values: Vec<Value<C::Allocator>, C::Allocator>,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, O, C, M> SequenceValueEncoder<OPT, O, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, output: O) -> Self {
        let values = Vec::new_in(cx.alloc());

        Self {
            cx,
            output,
            values,
            _marker: PhantomData,
        }
    }
}

impl<const OPT: Options, O, C, M> SequenceEncoder for SequenceValueEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeNext<'this>
        = ValueEncoder<OPT, &'this mut Vec<Value<C::Allocator>, C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Sequence(self.values))?;
        Ok(())
    }
}

/// A pack encoder.
pub struct PackValueEncoder<const OPT: Options, O, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: O,
    writer: BufWriter<C::Allocator>,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, O, C, M> PackValueEncoder<OPT, O, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, output: O) -> Result<Self, C::Error> {
        Ok(Self {
            cx,
            output,
            writer: BufWriter::new(cx.alloc()),
            _marker: PhantomData,
        })
    }
}

impl<const OPT: Options, O, C, M> SequenceEncoder for PackValueEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeNext<'this>
        = StorageEncoder<OPT, true, &'this mut BufWriter<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.writer))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        let buf = self.writer.into_inner();
        self.output.write(self.cx, Value::Bytes(buf))?;
        Ok(())
    }
}

/// A pairs encoder.
pub struct MapValueEncoder<const OPT: Options, O, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: O,
    values: Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, O, C, M> MapValueEncoder<OPT, O, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, output: O) -> Result<Self, C::Error> {
        let values = Vec::new_in(cx.alloc());
        Ok(Self {
            cx,
            output,
            values,
            _marker: PhantomData,
        })
    }
}

impl<const OPT: Options, O, C, M> MapEncoder for MapValueEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeEntry<'this>
        = PairValueEncoder<'this, OPT, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Map(self.values))?;
        Ok(())
    }
}

impl<const OPT: Options, O, C, M> EntriesEncoder for MapValueEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeEntryKey<'this>
        = ValueEncoder<OPT, &'this mut Value<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = ValueEncoder<OPT, &'this mut Value<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, C::Error> {
        self.values
            .push((Value::Unit, Value::Unit))
            .map_err(self.cx.map())?;

        let Some((key, _)) = self.values.last_mut() else {
            return Err(self.cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(self.cx, key))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, C::Error> {
        let Some((_, value)) = self.values.last_mut() else {
            return Err(self.cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(self.cx, value))
    }

    #[inline]
    fn finish_entries(self) -> Result<Self::Ok, C::Error> {
        self.output.write(self.cx, Value::Map(self.values))?;
        Ok(())
    }
}

/// A pairs encoder.
pub struct PairValueEncoder<'a, const OPT: Options, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: &'a mut Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    pair: (Value<C::Allocator>, Value<C::Allocator>),
    _marker: PhantomData<M>,
}

impl<'a, const OPT: Options, C, M> PairValueEncoder<'a, OPT, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(
        cx: C,
        output: &'a mut Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    ) -> Self {
        Self {
            cx,
            output,
            pair: (Value::Unit, Value::Unit),
            _marker: PhantomData,
        }
    }
}

impl<const OPT: Options, C, M> EntryEncoder for PairValueEncoder<'_, OPT, C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeKey<'this>
        = ValueEncoder<OPT, &'this mut Value<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeValue<'this>
        = ValueEncoder<OPT, &'this mut Value<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.0))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.1))
    }

    #[inline]
    fn finish_entry(self) -> Result<Self::Ok, C::Error> {
        self.output.push(self.pair).map_err(self.cx.map())?;
        Ok(())
    }
}

/// A pairs encoder.
pub struct VariantValueEncoder<const OPT: Options, O, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: O,
    pair: (Value<C::Allocator>, Value<C::Allocator>),
    _marker: PhantomData<M>,
}

impl<const OPT: Options, O, C, M> VariantValueEncoder<OPT, O, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, output: O) -> Self {
        Self {
            cx,
            output,
            pair: (Value::Unit, Value::Unit),
            _marker: PhantomData,
        }
    }
}

impl<const OPT: Options, O, C, M> VariantEncoder for VariantValueEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeTag<'this>
        = ValueEncoder<OPT, &'this mut Value<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeData<'this>
        = ValueEncoder<OPT, &'this mut Value<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.0))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.pair.1))
    }

    #[inline]
    fn finish_variant(self) -> Result<Self::Ok, C::Error> {
        let value = Box::new_in(self.pair, self.cx.alloc()).map_err(self.cx.map())?;
        self.output.write(self.cx, Value::Variant(value))?;
        Ok(())
    }
}

/// A variant sequence encoder.
pub struct VariantSequenceEncoder<const OPT: Options, O, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: O,
    variant: Value<C::Allocator>,
    values: Vec<Value<C::Allocator>, C::Allocator>,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, O, C, M> VariantSequenceEncoder<OPT, O, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, output: O, variant: Value<C::Allocator>, len: usize) -> Result<Self, C::Error> {
        let values = Vec::with_capacity_in(len, cx.alloc()).map_err(cx.map())?;

        Ok(Self {
            cx,
            output,
            variant,
            values,
            _marker: PhantomData,
        })
    }
}

impl<const OPT: Options, O, C, M> SequenceEncoder for VariantSequenceEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeNext<'this>
        = ValueEncoder<OPT, &'this mut Vec<Value<C::Allocator>, C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, C::Error> {
        Ok(ValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn finish_sequence(self) -> Result<Self::Ok, C::Error> {
        let value = (self.variant, Value::Sequence(self.values));
        let value = Box::new_in(value, self.cx.alloc()).map_err(self.cx.map())?;
        self.output.write(self.cx, Value::Variant(value))?;
        Ok(())
    }
}

/// A variant struct encoder.
pub struct VariantStructEncoder<const OPT: Options, O, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: O,
    variant: Value<C::Allocator>,
    fields: Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    _marker: PhantomData<M>,
}

impl<const OPT: Options, O, C, M> VariantStructEncoder<OPT, O, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, output: O, variant: Value<C::Allocator>, len: usize) -> Result<Self, C::Error> {
        let fields = Vec::with_capacity_in(len, cx.alloc()).map_err(cx.map())?;

        Ok(Self {
            cx,
            output,
            variant,
            fields,
            _marker: PhantomData,
        })
    }
}

impl<const OPT: Options, O, C, M> MapEncoder for VariantStructEncoder<OPT, O, C, M>
where
    O: ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Ok = ();
    type Mode = M;
    type EncodeEntry<'this>
        = PairValueEncoder<'this, OPT, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, C::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.fields))
    }

    #[inline]
    fn finish_map(self) -> Result<Self::Ok, C::Error> {
        let value = (self.variant, Value::Map(self.fields));
        let value = Box::new_in(value, self.cx.alloc()).map_err(self.cx.map())?;
        self.output.write(self.cx, Value::Variant(value))?;
        Ok(())
    }
}
