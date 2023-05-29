#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use musli::en::Encoder;
#[cfg(feature = "alloc")]
use musli::en::{PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};
use musli::Context;

use crate::error::Error;
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
impl<O> Encoder for ValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = Error;
    #[cfg(feature = "alloc")]
    type Some = ValueEncoder<SomeValueWriter<O>>;
    #[cfg(feature = "alloc")]
    type Pack = SequenceValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type Sequence = SequenceValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type Tuple = SequenceValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type Map = MapValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type Struct = MapValueEncoder<O>;
    #[cfg(feature = "alloc")]
    type Variant = VariantValueEncoder<O>;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be encoded")
    }

    #[inline]
    fn encode_unit<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(())
    }

    #[inline]
    fn encode_bool<'buf, C>(self, _: &mut C, b: bool) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Bool(b));
        Ok(())
    }

    #[inline]
    fn encode_char<'buf, C>(self, _: &mut C, c: char) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Char(c));
        Ok(())
    }

    #[inline]
    fn encode_u8<'buf, C>(self, _: &mut C, n: u8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::U8(n)));
        Ok(())
    }

    #[inline]
    fn encode_u16<'buf, C>(self, _: &mut C, n: u16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::U16(n)));
        Ok(())
    }

    #[inline]
    fn encode_u32<'buf, C>(self, _: &mut C, n: u32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::U32(n)));
        Ok(())
    }

    #[inline]
    fn encode_u64<'buf, C>(self, _: &mut C, n: u64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::U64(n)));
        Ok(())
    }

    #[inline]
    fn encode_u128<'buf, C>(self, _: &mut C, n: u128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::U128(n)));
        Ok(())
    }

    #[inline]
    fn encode_i8<'buf, C>(self, _: &mut C, n: i8) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::I8(n)));
        Ok(())
    }

    #[inline]
    fn encode_i16<'buf, C>(self, _: &mut C, n: i16) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::I16(n)));
        Ok(())
    }

    #[inline]
    fn encode_i32<'buf, C>(self, _: &mut C, n: i32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::I32(n)));
        Ok(())
    }

    #[inline]
    fn encode_i64<'buf, C>(self, _: &mut C, n: i64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::I64(n)));
        Ok(())
    }

    #[inline]
    fn encode_i128<'buf, C>(self, _: &mut C, n: i128) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::I128(n)));
        Ok(())
    }

    #[inline]
    fn encode_usize<'buf, C>(self, _: &mut C, n: usize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::Usize(n)));
        Ok(())
    }

    #[inline]
    fn encode_isize<'buf, C>(self, _: &mut C, n: isize) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::Isize(n)));
        Ok(())
    }

    #[inline]
    fn encode_f32<'buf, C>(self, _: &mut C, n: f32) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::F32(n)));
        Ok(())
    }

    #[inline]
    fn encode_f64<'buf, C>(self, _: &mut C, n: f64) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Number(Number::F64(n)));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_array<'buf, C, const N: usize>(
        self,
        _: &mut C,
        array: [u8; N],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Bytes(array.into()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_bytes<'buf, C>(self, _: &mut C, bytes: &[u8]) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Bytes(bytes.to_vec()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_bytes_vectored<'buf, C>(
        self,
        _: &mut C,
        input: &[&[u8]],
    ) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        let mut bytes = Vec::new();

        for b in input {
            bytes.extend_from_slice(b);
        }

        self.output.write(Value::Bytes(bytes));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_string<'buf, C>(self, _: &mut C, string: &str) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::String(string.into()));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_some<'buf, C>(self, _: &mut C) -> Result<Self::Some, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(ValueEncoder::new(SomeValueWriter {
            output: self.output,
        }))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_none<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Option(None));
        Ok(())
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_pack<'buf, C>(self, _: &mut C) -> Result<Self::Pack, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_sequence<'buf, C>(self, _: &mut C, _: usize) -> Result<Self::Sequence, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_tuple<'buf, C>(self, _: &mut C, _: usize) -> Result<Self::Tuple, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_map<'buf, C>(self, _: &mut C, _: usize) -> Result<Self::Map, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(MapValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_struct<'buf, C>(self, _: &mut C, _: usize) -> Result<Self::Struct, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(MapValueEncoder::new(self.output))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_variant<'buf, C>(self, _: &mut C) -> Result<Self::Variant, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(VariantValueEncoder::new(self.output))
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
impl<O> SequenceEncoder for SequenceValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = Error;

    type Encoder<'this> = ValueEncoder<&'this mut Vec<Value>>
    where
        Self: 'this;

    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(ValueEncoder::new(&mut self.values))
    }

    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
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
impl<O> PairsEncoder for MapValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = Error;

    type Encoder<'this> = PairValueEncoder<'this>
    where
        Self: 'this;

    fn next<'buf, C>(&mut self, _: &mut C) -> Result<Self::Encoder<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(PairValueEncoder::new(&mut self.values))
    }

    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
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
impl<'a> PairEncoder for PairValueEncoder<'a> {
    type Ok = ();
    type Error = Error;

    type First<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    type Second<'this> = ValueEncoder<&'this mut Value> where Self: 'this;

    #[inline]
    fn first<'buf, C>(&mut self, _: &mut C) -> Result<Self::First<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(ValueEncoder::new(&mut self.pair.0))
    }

    #[inline]
    fn second<'buf, C>(&mut self, _: &mut C) -> Result<Self::Second<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(ValueEncoder::new(&mut self.pair.1))
    }

    #[inline]
    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
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
impl<O> VariantEncoder for VariantValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = Error;

    type Tag<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    type Variant<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    fn tag<'buf, C>(&mut self, _: &mut C) -> Result<Self::Tag<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(ValueEncoder::new(&mut self.pair.0))
    }

    fn variant<'buf, C>(&mut self, _: &mut C) -> Result<Self::Variant<'_>, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        Ok(ValueEncoder::new(&mut self.pair.1))
    }

    fn end<'buf, C>(self, _: &mut C) -> Result<Self::Ok, C::Error>
    where
        C: Context<'buf, Input = Self::Error>,
    {
        self.output.write(Value::Variant(Box::new(self.pair)));
        Ok(())
    }
}
