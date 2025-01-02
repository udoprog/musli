use std::fmt;

use musli::de::UnsizedVisitor;
use musli::{Allocator, Context, Decode, Decoder};

#[derive(Debug, PartialEq)]
pub struct BytesReference<'de> {
    data: &'de [u8],
}

impl<'de, M, A> Decode<'de, M, A> for BytesReference<'de>
where
    A: Allocator,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> UnsizedVisitor<'de, C, [u8]> for Visitor
        where
            C: Context,
        {
            type Ok = &'de [u8];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a literal byte reference")
            }

            #[inline]
            fn visit_borrowed(self, _: C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
                Ok(bytes)
            }
        }

        Ok(Self {
            data: decoder.decode_bytes(Visitor)?,
        })
    }
}

#[test]
fn bytes_reference() {
    let value = musli::value::Value::Bytes(vec![0, 1, 2, 3]);

    assert_eq!(
        musli::value::decode::<BytesReference>(&value).unwrap(),
        BytesReference {
            data: &[0, 1, 2, 3]
        }
    );

    let value = musli::value::Value::Number(42u32.into());

    assert_eq!(
        musli::value::decode::<BytesReference>(&value)
            .unwrap_err()
            .to_string(),
        "Value buffer expected bytes, but found u32"
    );
}

#[derive(Debug, PartialEq)]
pub struct StringReference<'de> {
    data: &'de str,
}

impl<'de, M, A> Decode<'de, M, A> for StringReference<'de>
where
    A: Allocator,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> UnsizedVisitor<'de, C, str> for Visitor
        where
            C: Context,
        {
            type Ok = &'de str;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "exact bytes reference")
            }

            #[inline]
            fn visit_borrowed(self, _: C, bytes: &'de str) -> Result<Self::Ok, C::Error> {
                Ok(bytes)
            }
        }

        Ok(Self {
            data: decoder.decode_string(Visitor)?,
        })
    }
}

#[test]
fn string_reference() {
    let value = musli::value::Value::String(String::from("Hello!"));

    assert_eq!(
        musli::value::decode::<StringReference>(&value).unwrap(),
        StringReference { data: "Hello!" }
    );

    let value = musli::value::Value::Number(42u32.into());

    assert_eq!(
        musli::value::decode::<StringReference>(&value)
            .unwrap_err()
            .to_string(),
        "Value buffer expected string, but found u32"
    );
}

#[derive(Debug, PartialEq)]
pub enum OwnedFn {
    A,
    B,
}

impl<'de, M, A> Decode<'de, M, A> for OwnedFn
where
    A: Allocator,
{
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let cx = decoder.cx();

        decoder.decode_unsized(|variant: &str| match variant {
            "A" => Ok(OwnedFn::A),
            "B" => Ok(OwnedFn::A),
            other => Err(cx.message(format_args!("Expected either 'A' or 'B' but got {other}"))),
        })
    }
}

#[test]
fn owned_fn() {
    let value = musli::value::Value::String("A".to_string());
    assert_eq!(musli::value::decode::<OwnedFn>(&value).unwrap(), OwnedFn::A);
}
