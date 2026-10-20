use core::fmt;
use core::marker::PhantomData;
use core::str::FromStr;

use crate::Context;
use crate::de::{Decoder, SizeHint, Skip, UnsizedVisitor, Visitor};

/// The key of a field in a record.
pub(crate) enum Key<'a> {
    /// The name of the field from the header row.
    Name(&'a str),
    /// The position of the field, used when there is no header row.
    Index(usize),
}

/// A decoder for the key of a field in a record.
pub(crate) struct KeyDecoder<'a, C, M> {
    cx: C,
    key: Key<'a>,
    _marker: PhantomData<M>,
}

impl<'a, C, M> KeyDecoder<'a, C, M>
where
    C: Context,
{
    #[inline]
    pub(crate) fn new(cx: C, key: Key<'a>) -> Self {
        Self {
            cx,
            key,
            _marker: PhantomData,
        }
    }

    /// Decode the key as a number.
    #[inline]
    fn parse<T>(self, what: &'static str) -> Result<T, C::Error>
    where
        T: FromStr + TryFrom<usize>,
    {
        match self.key {
            Key::Name(name) => match name.parse() {
                Ok(value) => Ok(value),
                Err(..) => Err(self
                    .cx
                    .message(format_args!("Expected {what} field name, but found {name:?}"))),
            },
            Key::Index(index) => match T::try_from(index) {
                Ok(value) => Ok(value),
                Err(..) => Err(self
                    .cx
                    .message(format_args!("Field index {index} is out of range for {what}"))),
            },
        }
    }
}

#[crate::trait_defaults(crate)]
impl<'de, C, M> Decoder<'de> for KeyDecoder<'_, C, M>
where
    C: Context,
    M: 'static,
{
    type Cx = C;
    type Error = C::Error;
    type Allocator = C::Allocator;
    type Mode = M;

    #[inline]
    fn cx(&self) -> Self::Cx {
        self.cx
    }

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value that can be decoded from a CSV header")
    }

    #[inline]
    fn skip(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn try_skip(self) -> Result<Skip, Self::Error> {
        Ok(Skip::Skipped)
    }

    #[inline]
    fn decode_u8(self) -> Result<u8, Self::Error> {
        self.parse("an unsigned 8-bit integer")
    }

    #[inline]
    fn decode_u16(self) -> Result<u16, Self::Error> {
        self.parse("an unsigned 16-bit integer")
    }

    #[inline]
    fn decode_u32(self) -> Result<u32, Self::Error> {
        self.parse("an unsigned 32-bit integer")
    }

    #[inline]
    fn decode_u64(self) -> Result<u64, Self::Error> {
        self.parse("an unsigned 64-bit integer")
    }

    #[inline]
    fn decode_u128(self) -> Result<u128, Self::Error> {
        self.parse("an unsigned 128-bit integer")
    }

    #[inline]
    fn decode_i8(self) -> Result<i8, Self::Error> {
        self.parse("a signed 8-bit integer")
    }

    #[inline]
    fn decode_i16(self) -> Result<i16, Self::Error> {
        self.parse("a signed 16-bit integer")
    }

    #[inline]
    fn decode_i32(self) -> Result<i32, Self::Error> {
        self.parse("a signed 32-bit integer")
    }

    #[inline]
    fn decode_i64(self) -> Result<i64, Self::Error> {
        self.parse("a signed 64-bit integer")
    }

    #[inline]
    fn decode_i128(self) -> Result<i128, Self::Error> {
        self.parse("a signed 128-bit integer")
    }

    #[inline]
    fn decode_usize(self) -> Result<usize, Self::Error> {
        self.parse("an unsigned pointer-sized integer")
    }

    #[inline]
    fn decode_isize(self) -> Result<isize, Self::Error> {
        self.parse("a signed pointer-sized integer")
    }

    #[inline]
    fn decode_string<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: UnsizedVisitor<'de, C, str, Error = Self::Error, Allocator = Self::Allocator>,
    {
        match self.key {
            Key::Name(name) => visitor.visit_ref(self.cx, name),
            Key::Index(index) => {
                let mut buffer = itoa::Buffer::new();
                visitor.visit_ref(self.cx, buffer.format(index))
            }
        }
    }

    #[inline]
    fn decode_any<V>(self, visitor: V) -> Result<V::Ok, V::Error>
    where
        V: Visitor<'de, C, Error = Self::Error, Allocator = Self::Allocator>,
    {
        let visitor = visitor.visit_string(self.cx, SizeHint::any())?;
        self.decode_string(visitor)
    }
}
