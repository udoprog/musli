use std::fmt;

use musli::de::ValueVisitor;
use musli::{Context, Decode, Decoder};

#[derive(Debug, PartialEq)]
pub struct BytesReference<'de> {
    data: &'de [u8],
}

impl<'de, M> Decode<'de, M> for BytesReference<'de> {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, [u8]> for Visitor
        where
            C: ?Sized + Context,
        {
            type Ok = &'de [u8];

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a literal byte reference")
            }

            #[inline]
            fn visit_borrowed(self, _: &C, bytes: &'de [u8]) -> Result<Self::Ok, C::Error> {
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
    let value = musli_value::Value::Bytes(vec![0, 1, 2, 3]);

    assert_eq!(
        musli_value::decode::<BytesReference>(&value).unwrap(),
        BytesReference {
            data: &[0, 1, 2, 3]
        }
    );

    let value = musli_value::Value::Number(42u32.into());

    assert_eq!(
        musli_value::decode::<BytesReference>(&value)
            .unwrap_err()
            .to_string(),
        "Value buffer expected bytes, but found u32"
    );
}

#[derive(Debug, PartialEq)]
pub struct StringReference<'de> {
    data: &'de str,
}

impl<'de, M> Decode<'de, M> for StringReference<'de> {
    #[inline]
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        struct Visitor;

        impl<'de, C> ValueVisitor<'de, C, str> for Visitor
        where
            C: ?Sized + Context,
        {
            type Ok = &'de str;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "exact bytes reference")
            }

            #[inline]
            fn visit_borrowed(self, _: &C, bytes: &'de str) -> Result<Self::Ok, C::Error> {
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
    let value = musli_value::Value::String(String::from("Hello!"));

    assert_eq!(
        musli_value::decode::<StringReference>(&value).unwrap(),
        StringReference { data: "Hello!" }
    );

    let value = musli_value::Value::Number(42u32.into());

    assert_eq!(
        musli_value::decode::<StringReference>(&value)
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

impl<'de, M> Decode<'de, M> for OwnedFn {
    fn decode<D>(cx: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        decoder.decode_string(musli::utils::visit_owned_fn(
            "A string variant for Enum",
            |variant: &str| match variant {
                "A" => Ok(OwnedFn::A),
                "B" => Ok(OwnedFn::A),
                other => {
                    Err(cx.message(format_args!("Expected either 'A' or 'B' but got {other}")))
                }
            },
        ))
    }
}

#[test]
fn owned_fn() {
    let value = musli_value::Value::String("A".to_string());
    assert_eq!(musli_value::decode::<OwnedFn>(&value).unwrap(), OwnedFn::A);
}
