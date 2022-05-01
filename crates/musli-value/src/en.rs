use musli::en::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder, VariantEncoder};

use crate::error::ValueError;
use crate::value::{Number, Value};

/// Insert a value into the given receiver.
trait ValueOutput {
    fn write(self, value: Value);
}

impl ValueOutput for &mut Value {
    fn write(self, value: Value) {
        *self = value;
    }
}

impl ValueOutput for &mut Vec<Value> {
    fn write(self, value: Value) {
        self.push(value);
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

impl<O> Encoder for ValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = ValueError;
    type Some = Self;
    type Pack = SequenceValueEncoder<O>;
    type Sequence = SequenceValueEncoder<O>;
    type Tuple = SequenceValueEncoder<O>;
    type Map = MapValueEncoder<O>;
    type Struct = MapValueEncoder<O>;
    type Variant = VariantValueEncoder<O>;

    #[inline]
    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "value that can be encoded")
    }

    #[inline]
    fn encode_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn encode_bool(self, b: bool) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Bool(b));
        Ok(())
    }

    #[inline]
    fn encode_char(self, c: char) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Char(c));
        Ok(())
    }

    #[inline]
    fn encode_u8(self, n: u8) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::U8(n)));
        Ok(())
    }

    #[inline]
    fn encode_u16(self, n: u16) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::U16(n)));
        Ok(())
    }

    #[inline]
    fn encode_u32(self, n: u32) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::U32(n)));
        Ok(())
    }

    #[inline]
    fn encode_u64(self, n: u64) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::U64(n)));
        Ok(())
    }

    #[inline]
    fn encode_u128(self, n: u128) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::U128(n)));
        Ok(())
    }

    #[inline]
    fn encode_i8(self, n: i8) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::I8(n)));
        Ok(())
    }

    #[inline]
    fn encode_i16(self, n: i16) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::I16(n)));
        Ok(())
    }

    #[inline]
    fn encode_i32(self, n: i32) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::I32(n)));
        Ok(())
    }

    #[inline]
    fn encode_i64(self, n: i64) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::I64(n)));
        Ok(())
    }

    #[inline]
    fn encode_i128(self, n: i128) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::I128(n)));
        Ok(())
    }

    #[inline]
    fn encode_usize(self, n: usize) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::Usize(n)));
        Ok(())
    }

    #[inline]
    fn encode_isize(self, n: isize) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::Isize(n)));
        Ok(())
    }

    #[inline]
    fn encode_f32(self, n: f32) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::F32(n)));
        Ok(())
    }

    #[inline]
    fn encode_f64(self, n: f64) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Number(Number::F64(n)));
        Ok(())
    }

    #[inline]
    fn encode_array<const N: usize>(self, array: [u8; N]) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Bytes(array.into()));
        Ok(())
    }

    #[inline]
    fn encode_bytes(self, bytes: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Bytes(bytes.to_vec()));
        Ok(())
    }

    #[inline]
    fn encode_bytes_vectored(self, input: &[&[u8]]) -> Result<Self::Ok, Self::Error> {
        let mut bytes = Vec::new();

        for b in input {
            bytes.extend_from_slice(b);
        }

        self.output.write(Value::Bytes(bytes));
        Ok(())
    }

    #[inline]
    fn encode_string(self, string: &str) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::String(string.into()));
        Ok(())
    }

    #[inline]
    fn encode_some(self) -> Result<Self::Some, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn encode_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn encode_pack(self) -> Result<Self::Pack, Self::Error> {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[inline]
    fn encode_sequence(self, _: usize) -> Result<Self::Sequence, Self::Error> {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[inline]
    fn encode_tuple(self, _: usize) -> Result<Self::Tuple, Self::Error> {
        Ok(SequenceValueEncoder::new(self.output))
    }

    #[inline]
    fn encode_map(self, _: usize) -> Result<Self::Map, Self::Error> {
        Ok(MapValueEncoder::new(self.output))
    }

    #[inline]
    fn encode_struct(self, _: usize) -> Result<Self::Struct, Self::Error> {
        Ok(MapValueEncoder::new(self.output))
    }

    #[inline]
    fn encode_variant(self) -> Result<Self::Variant, Self::Error> {
        Ok(VariantValueEncoder::new(self.output))
    }
}

/// A pack encoder.
pub struct SequenceValueEncoder<O> {
    output: O,
    values: Vec<Value>,
}

impl<O> SequenceValueEncoder<O> {
    #[inline]
    fn new(output: O) -> Self {
        Self {
            output,
            values: Vec::new(),
        }
    }
}

impl<O> SequenceEncoder for SequenceValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = ValueError;

    type Encoder<'this> = ValueEncoder<&'this mut Vec<Value>>
    where
        Self: 'this;

    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(ValueEncoder::new(&mut self.values))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Sequence(self.values));
        Ok(())
    }
}

/// A pairs encoder.
pub struct MapValueEncoder<O> {
    output: O,
    values: Vec<(Value, Value)>,
}

impl<O> MapValueEncoder<O> {
    #[inline]
    fn new(output: O) -> Self {
        Self {
            output,
            values: Vec::new(),
        }
    }
}

impl<O> PairsEncoder for MapValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = ValueError;

    type Encoder<'this> = PairValueEncoder<'this>
    where
        Self: 'this;

    fn next(&mut self) -> Result<Self::Encoder<'_>, Self::Error> {
        Ok(PairValueEncoder::new(&mut self.values))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Map(self.values));
        Ok(())
    }
}

/// A pairs encoder.
pub struct PairValueEncoder<'a> {
    output: &'a mut Vec<(Value, Value)>,
    pair: (Value, Value),
}

impl<'a> PairValueEncoder<'a> {
    #[inline]
    fn new(output: &'a mut Vec<(Value, Value)>) -> Self {
        Self {
            output,
            pair: (Value::Unit, Value::Unit),
        }
    }
}

impl<'a> PairEncoder for PairValueEncoder<'a> {
    type Ok = ();
    type Error = ValueError;

    type First<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    type Second<'this> = ValueEncoder<&'this mut Value> where Self: 'this;

    #[inline]
    fn first(&mut self) -> Result<Self::First<'_>, Self::Error> {
        Ok(ValueEncoder::new(&mut self.pair.0))
    }

    #[inline]
    fn second(&mut self) -> Result<Self::Second<'_>, Self::Error> {
        Ok(ValueEncoder::new(&mut self.pair.1))
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output.push(self.pair);
        Ok(())
    }
}

/// A pairs encoder.
pub struct VariantValueEncoder<O> {
    output: O,
    pair: (Value, Value),
}

impl<O> VariantValueEncoder<O> {
    #[inline]
    fn new(output: O) -> Self {
        Self {
            output,
            pair: (Value::Unit, Value::Unit),
        }
    }
}

impl<O> VariantEncoder for VariantValueEncoder<O>
where
    O: ValueOutput,
{
    type Ok = ();
    type Error = ValueError;

    type Tag<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    type Variant<'this> = ValueEncoder<&'this mut Value>
    where
        Self: 'this;

    fn tag(&mut self) -> Result<Self::Tag<'_>, Self::Error> {
        Ok(ValueEncoder::new(&mut self.pair.0))
    }

    fn variant(&mut self) -> Result<Self::Variant<'_>, Self::Error> {
        Ok(ValueEncoder::new(&mut self.pair.1))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output.write(Value::Variant(Box::new(self.pair)));
        Ok(())
    }
}
