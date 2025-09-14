#![allow(clippy::type_complexity)]

use core::marker::PhantomData;

use crate::alloc::{Box, String, Vec};
use crate::en::{Encode, Encoder};
use crate::en::{EntriesEncoder, EntryEncoder, MapEncoder, SequenceEncoder, VariantEncoder};
use crate::hint::{MapHint, SequenceHint};
use crate::storage::en::StorageEncoder;
use crate::writer::BufWriter;
use crate::{Allocator, Context, Options};

use super::{Number, Value, ValueKind};

/// Insert a value into the given receiver.
trait ValueOutput<A>
where
    A: Allocator,
{
    /// Write a value into the receiver.
    fn write<C>(&mut self, cx: C, some: usize, value: ValueKind<A>) -> Result<(), C::Error>
    where
        C: Context<Allocator = A>;
}

impl<A> ValueOutput<A> for Value<A>
where
    A: Allocator,
{
    #[inline]
    fn write<C>(&mut self, cx: C, some: usize, mut value: ValueKind<A>) -> Result<(), C::Error>
    where
        C: Context<Allocator = A>,
    {
        for _ in 0..some {
            value = ValueKind::Option(Some(
                Box::new_in(Value::new(value), cx.alloc()).map_err(cx.map())?,
            ));
        }

        self.kind = value;
        Ok(())
    }
}

impl<A> ValueOutput<A> for Vec<Value<A>, A>
where
    A: Allocator,
{
    #[inline]
    fn write<C>(&mut self, cx: C, some: usize, mut value: ValueKind<A>) -> Result<(), C::Error>
    where
        C: Context<Allocator = A>,
    {
        for _ in 0..some {
            value = ValueKind::Option(Some(
                Box::new_in(Value::new(value), cx.alloc()).map_err(cx.map())?,
            ));
        }

        self.push(Value::new(value)).map_err(cx.map())
    }
}

/// Encoder for a single value.
pub struct ValueEncoder<'out, const OPT: Options, O, C, M>
where
    O: ?Sized,
{
    cx: C,
    // Levels of some option nesting in effect.
    some: usize,
    // Output to write to.
    output: &'out mut O,
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, O, C, M> ValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized,
{
    #[inline]
    pub(crate) fn new(cx: C, some: usize, output: &'out mut O) -> Self {
        Self {
            cx,
            output,
            some,
            _marker: PhantomData,
        }
    }
}

#[crate::trait_defaults(crate)]
impl<'out, const OPT: Options, O, C, M> Encoder for ValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeSome = ValueEncoder<'out, OPT, O, C, M>;
    type EncodePack = PackValueEncoder<'out, OPT, O, C, M>;
    type EncodeSequence = SequenceValueEncoder<'out, OPT, O, C, M>;
    type EncodeMap = MapValueEncoder<'out, OPT, O, C, M>;
    type EncodeMapEntries = MapValueEncoder<'out, OPT, O, C, M>;
    type EncodeVariant = VariantValueEncoder<'out, OPT, O, C, M>;
    type EncodeSequenceVariant = VariantSequenceEncoder<'out, OPT, O, C, M>;
    type EncodeMapVariant = VariantStructEncoder<'out, OPT, O, C, M>;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be encoded")
    }

    #[inline]
    fn encode<T>(self, value: T) -> Result<(), Self::Error>
    where
        T: Encode<Self::Mode>,
    {
        value.as_encode().encode(self)
    }

    #[inline]
    fn encode_empty(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn encode_bool(self, b: bool) -> Result<(), Self::Error> {
        self.output.write(self.cx, self.some, ValueKind::Bool(b))?;
        Ok(())
    }

    #[inline]
    fn encode_char(self, c: char) -> Result<(), Self::Error> {
        self.output.write(self.cx, self.some, ValueKind::Char(c))?;
        Ok(())
    }

    #[inline]
    fn encode_u8(self, n: u8) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::U8(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u16(self, n: u16) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::U16(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u32(self, n: u32) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::U32(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u64(self, n: u64) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::U64(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_u128(self, n: u128) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::U128(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i8(self, n: i8) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::I8(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i16(self, n: i16) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::I16(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i32(self, n: i32) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::I32(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i64(self, n: i64) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::I64(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_i128(self, n: i128) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::I128(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_f32(self, n: f32) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::F32(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_f64(self, n: f64) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::F64(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_usize(self, n: usize) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::Usize(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_isize(self, n: isize) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Number(Number::Isize(n)))?;
        Ok(())
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: &[u8; N]) -> Result<(), Self::Error> {
        let mut bytes =
            Vec::with_capacity_in(array.len(), self.cx.alloc()).map_err(self.cx.map())?;
        bytes.extend_from_slice(array).map_err(self.cx.map())?;
        self.output
            .write(self.cx, self.some, ValueKind::Bytes(bytes))?;
        Ok(())
    }

    #[inline]
    fn encode_bytes(self, b: &[u8]) -> Result<(), Self::Error> {
        let mut bytes = Vec::with_capacity_in(b.len(), self.cx.alloc()).map_err(self.cx.map())?;
        bytes.extend_from_slice(b).map_err(self.cx.map())?;
        self.output
            .write(self.cx, self.some, ValueKind::Bytes(bytes))?;
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored<I>(self, len: usize, vectors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item: AsRef<[u8]>>,
    {
        let mut bytes = Vec::with_capacity_in(len, self.cx.alloc()).map_err(self.cx.map())?;

        for b in vectors {
            bytes.extend_from_slice(b.as_ref()).map_err(self.cx.map())?;
        }

        self.output
            .write(self.cx, self.some, ValueKind::Bytes(bytes))?;
        Ok(())
    }

    #[inline]
    fn encode_string(self, s: &str) -> Result<(), Self::Error> {
        let mut string = String::new_in(self.cx.alloc());
        string.push_str(s).map_err(self.cx.map())?;
        self.output
            .write(self.cx, self.some, ValueKind::String(string))?;
        Ok(())
    }

    #[inline]
    fn encode_some(self) -> Result<Self::EncodeSome, Self::Error> {
        Ok(ValueEncoder::new(self.cx, self.some + 1, self.output))
    }

    #[inline]
    fn encode_none(self) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Option(None))?;
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::EncodePack, Self::Error> {
        Ok(PackValueEncoder::new(self.cx, self.some, self.output))
    }

    #[inline]
    fn encode_sequence(self, _: impl SequenceHint) -> Result<Self::EncodeSequence, Self::Error> {
        Ok(SequenceValueEncoder::new(self.cx, self.some, self.output))
    }

    #[inline]
    fn encode_map(self, _: impl MapHint) -> Result<Self::EncodeMap, Self::Error> {
        Ok(MapValueEncoder::new(self.cx, self.some, self.output))
    }

    #[inline]
    fn encode_map_entries(self, _: impl MapHint) -> Result<Self::EncodeMapEntries, Self::Error> {
        Ok(MapValueEncoder::new(self.cx, self.some, self.output))
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::EncodeVariant, Self::Error> {
        Ok(VariantValueEncoder::new(self.cx, self.some, self.output))
    }

    #[inline]
    fn encode_unit_variant<T>(self, tag: &T) -> Result<(), Self::Error>
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
        hint: impl SequenceHint,
    ) -> Result<Self::EncodeSequenceVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        let size = hint.require(self.cx)?;
        let mut variant = Value::new(ValueKind::Empty);
        ValueEncoder::<OPT, _, _, Self::Mode>::new(self.cx, 0, &mut variant).encode(tag)?;
        VariantSequenceEncoder::new(self.cx, self.some, self.output, variant, size)
    }

    #[inline]
    fn encode_map_variant<T>(
        self,
        tag: &T,
        hint: impl MapHint,
    ) -> Result<Self::EncodeMapVariant, Self::Error>
    where
        T: ?Sized + Encode<Self::Mode>,
    {
        let size = hint.require(self.cx)?;
        let mut variant = Value::new(ValueKind::Empty);
        ValueEncoder::<OPT, _, _, Self::Mode>::new(self.cx, 0, &mut variant).encode(tag)?;
        VariantStructEncoder::new(self.cx, self.some, self.output, variant, size)
    }
}

/// A sequence encoder.
pub struct SequenceValueEncoder<'out, const OPT: Options, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    cx: C,
    some: usize,
    output: &'out mut O,
    values: Vec<Value<C::Allocator>, C::Allocator>,
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, O, C, M> SequenceValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, some: usize, output: &'out mut O) -> Self {
        let values = Vec::new_in(cx.alloc());

        Self {
            cx,
            some,
            output,
            values,
            _marker: PhantomData,
        }
    }
}

impl<'out, const OPT: Options, O, C, M> SequenceEncoder for SequenceValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = ValueEncoder<'this, OPT, Vec<Value<C::Allocator>, C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Ok(ValueEncoder::new(self.cx, 0, &mut self.values))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Sequence(self.values))?;
        Ok(())
    }
}

/// A pack encoder.
pub struct PackValueEncoder<'out, const OPT: Options, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    cx: C,
    some: usize,
    output: &'out mut O,
    writer: BufWriter<C::Allocator>,
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, O, C, M> PackValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, some: usize, output: &'out mut O) -> Self {
        Self {
            cx,
            some,
            output,
            writer: BufWriter::new(cx.alloc()),
            _marker: PhantomData,
        }
    }
}

impl<'out, const OPT: Options, O, C, M> SequenceEncoder for PackValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
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
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Ok(StorageEncoder::new(self.cx, &mut self.writer))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        let buf = self.writer.into_inner();
        self.output
            .write(self.cx, self.some, ValueKind::Bytes(buf))?;
        Ok(())
    }
}

/// A pairs encoder.
pub struct MapValueEncoder<'out, const OPT: Options, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    cx: C,
    some: usize,
    output: &'out mut O,
    values: Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, O, C, M> MapValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, some: usize, output: &'out mut O) -> Self {
        let values = Vec::new_in(cx.alloc());

        Self {
            cx,
            some,
            output,
            values,
            _marker: PhantomData,
        }
    }
}

impl<'out, const OPT: Options, O, C, M> MapEncoder for MapValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
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
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.values))
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Map(self.values))?;
        Ok(())
    }
}

impl<'out, const OPT: Options, O, C, M> EntriesEncoder for MapValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeEntryKey<'this>
        = ValueEncoder<'this, OPT, Value<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeEntryValue<'this>
        = ValueEncoder<'this, OPT, Value<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_entry_key(&mut self) -> Result<Self::EncodeEntryKey<'_>, Self::Error> {
        self.values
            .push((Value::new(ValueKind::Empty), Value::new(ValueKind::Empty)))
            .map_err(self.cx.map())?;

        let Some((key, _)) = self.values.last_mut() else {
            return Err(self.cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(self.cx, 0, key))
    }

    #[inline]
    fn encode_entry_value(&mut self) -> Result<Self::EncodeEntryValue<'_>, Self::Error> {
        let Some((_, value)) = self.values.last_mut() else {
            return Err(self.cx.message("Pair has not been encoded"));
        };

        Ok(ValueEncoder::new(self.cx, 0, value))
    }

    #[inline]
    fn finish_entries(self) -> Result<(), Self::Error> {
        self.output
            .write(self.cx, self.some, ValueKind::Map(self.values))?;
        Ok(())
    }
}

/// A pairs encoder.
pub struct PairValueEncoder<'out, const OPT: Options, C, M>
where
    C: Context,
    M: 'static,
{
    cx: C,
    output: &'out mut Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    pair: (Value<C::Allocator>, Value<C::Allocator>),
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, C, M> PairValueEncoder<'out, OPT, C, M>
where
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(
        cx: C,
        output: &'out mut Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    ) -> Self {
        Self {
            cx,
            output,
            pair: (Value::new(ValueKind::Empty), Value::new(ValueKind::Empty)),
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
    type Error = C::Error;
    type Mode = M;
    type EncodeKey<'this>
        = ValueEncoder<'this, OPT, Value<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeValue<'this>
        = ValueEncoder<'this, OPT, Value<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_key(&mut self) -> Result<Self::EncodeKey<'_>, Self::Error> {
        Ok(ValueEncoder::new(self.cx, 0, &mut self.pair.0))
    }

    #[inline]
    fn encode_value(&mut self) -> Result<Self::EncodeValue<'_>, Self::Error> {
        Ok(ValueEncoder::new(self.cx, 0, &mut self.pair.1))
    }

    #[inline]
    fn finish_entry(self) -> Result<(), Self::Error> {
        self.output.push(self.pair).map_err(self.cx.map())?;
        Ok(())
    }
}

/// A pairs encoder.
pub struct VariantValueEncoder<'out, const OPT: Options, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    cx: C,
    some: usize,
    output: &'out mut O,
    pair: (Value<C::Allocator>, Value<C::Allocator>),
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, O, C, M> VariantValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(cx: C, some: usize, output: &'out mut O) -> Self {
        Self {
            cx,
            some,
            output,
            pair: (Value::new(ValueKind::Empty), Value::new(ValueKind::Empty)),
            _marker: PhantomData,
        }
    }
}

impl<'out, const OPT: Options, O, C, M> VariantEncoder for VariantValueEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeTag<'this>
        = ValueEncoder<'this, OPT, Value<C::Allocator>, C, M>
    where
        Self: 'this;
    type EncodeData<'this>
        = ValueEncoder<'this, OPT, Value<C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_tag(&mut self) -> Result<Self::EncodeTag<'_>, Self::Error> {
        Ok(ValueEncoder::new(self.cx, 0, &mut self.pair.0))
    }

    #[inline]
    fn encode_data(&mut self) -> Result<Self::EncodeData<'_>, Self::Error> {
        Ok(ValueEncoder::new(self.cx, 0, &mut self.pair.1))
    }

    #[inline]
    fn finish_variant(self) -> Result<(), Self::Error> {
        let value = Box::new_in(self.pair, self.cx.alloc()).map_err(self.cx.map())?;
        self.output
            .write(self.cx, self.some, ValueKind::Variant(value))?;
        Ok(())
    }
}

/// A variant sequence encoder.
pub struct VariantSequenceEncoder<'out, const OPT: Options, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    cx: C,
    some: usize,
    output: &'out mut O,
    variant: Value<C::Allocator>,
    values: Vec<Value<C::Allocator>, C::Allocator>,
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, O, C, M> VariantSequenceEncoder<'out, OPT, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(
        cx: C,
        some: usize,
        output: &'out mut O,
        variant: Value<C::Allocator>,
        len: usize,
    ) -> Result<Self, C::Error> {
        let values = Vec::with_capacity_in(len, cx.alloc()).map_err(cx.map())?;

        Ok(Self {
            cx,
            some,
            output,
            variant,
            values,
            _marker: PhantomData,
        })
    }
}

impl<'out, const OPT: Options, O, C, M> SequenceEncoder
    for VariantSequenceEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Mode = M;
    type EncodeNext<'this>
        = ValueEncoder<'this, OPT, Vec<Value<C::Allocator>, C::Allocator>, C, M>
    where
        Self: 'this;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn encode_next(&mut self) -> Result<Self::EncodeNext<'_>, Self::Error> {
        Ok(ValueEncoder::new(self.cx, 0, &mut self.values))
    }

    #[inline]
    fn finish_sequence(self) -> Result<(), Self::Error> {
        let value = (self.variant, Value::new(ValueKind::Sequence(self.values)));
        let value = Box::new_in(value, self.cx.alloc()).map_err(self.cx.map())?;
        self.output
            .write(self.cx, self.some, ValueKind::Variant(value))?;
        Ok(())
    }
}

/// A variant struct encoder.
pub struct VariantStructEncoder<'out, const OPT: Options, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    cx: C,
    some: usize,
    output: &'out mut O,
    variant: Value<C::Allocator>,
    fields: Vec<(Value<C::Allocator>, Value<C::Allocator>), C::Allocator>,
    _marker: PhantomData<M>,
}

impl<'out, const OPT: Options, O, C, M> VariantStructEncoder<'out, OPT, O, C, M>
where
    O: ?Sized,
    C: Context,
    M: 'static,
{
    #[inline]
    fn new(
        cx: C,
        some: usize,
        output: &'out mut O,
        variant: Value<C::Allocator>,
        len: usize,
    ) -> Result<Self, C::Error> {
        let fields = Vec::with_capacity_in(len, cx.alloc()).map_err(cx.map())?;

        Ok(Self {
            cx,
            some,
            output,
            variant,
            fields,
            _marker: PhantomData,
        })
    }
}

impl<'out, const OPT: Options, O, C, M> MapEncoder for VariantStructEncoder<'out, OPT, O, C, M>
where
    O: ?Sized + ValueOutput<C::Allocator>,
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
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
    fn encode_entry(&mut self) -> Result<Self::EncodeEntry<'_>, Self::Error> {
        Ok(PairValueEncoder::new(self.cx, &mut self.fields))
    }

    #[inline]
    fn finish_map(self) -> Result<(), Self::Error> {
        let value = (self.variant, Value::new(ValueKind::Map(self.fields)));
        let value = Box::new_in(value, self.cx.alloc()).map_err(self.cx.map())?;
        self.output
            .write(self.cx, self.some, ValueKind::Variant(value))?;
        Ok(())
    }
}
