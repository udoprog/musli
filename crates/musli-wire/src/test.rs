//! Helpers for writing tests.

use anyhow::Result;
use core::fmt::Debug;
use musli::de::PackDecoder;
use musli::{Decode, Decoder, Encode};

use crate::tag::Tag;

/// A typed field, which is prefixed with a type tag.
///
/// This is used in combination with the storage deserializer to "inspect" type
/// tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Typed<T> {
    tag: Tag,
    value: T,
}

impl<T> Typed<T> {
    /// Construct a new typed field.
    pub const fn new(tag: Tag, value: T) -> Self {
        Self { tag, value }
    }
}

impl<'de, T> Decode<'de> for Typed<T>
where
    T: Decode<'de>,
{
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut pack = decoder.decode_pack()?;
        let tag = pack.next().and_then(Tag::decode)?;
        let value = pack.next().and_then(T::decode)?;

        Ok(Self { tag, value })
    }
}

/// Roundtrip encode the given value.
#[inline(never)]
pub fn rt<T>(expected: T) -> T
where
    T: Debug + PartialEq + for<'de> Decode<'de> + Encode,
{
    let out = crate::to_vec(&expected).expect("failed to encode");
    let mut buf = &out[..];
    let value: T = crate::decode(&mut buf).expect("failed to decode");
    assert!(buf.is_empty(), "deserialized buffer should be empty");
    assert_eq!(value, expected, "roundtrip does not match");
    value
}

/// Encode a type as one and decode as another.
#[inline(never)]
pub fn transcode<T, O>(value: T) -> O
where
    T: Debug + PartialEq + Encode,
    O: for<'de> Decode<'de>,
{
    let out = crate::to_vec(&value).expect("failed to encode");
    let mut buf = &out[..];
    let value: O = crate::decode(&mut buf).expect("failed to decode");
    assert!(buf.is_empty());
    value
}
