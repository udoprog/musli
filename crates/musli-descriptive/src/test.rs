//! Helpers for writing tests.

use core::fmt::Debug;
use musli::de::PackDecoder;
use musli::mode::{DefaultMode, Mode};
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

impl<'de, M, T> Decode<'de, M> for Typed<T>
where
    M: Mode,
    T: Decode<'de, M>,
{
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        let mut unpack = decoder.decode_pack()?;
        let tag = unpack.next().and_then(<Tag as Decode<M>>::decode)?;
        let value = unpack.next().and_then(<T as Decode<M>>::decode)?;
        unpack.end()?;
        Ok(Self { tag, value })
    }
}

/// Roundtrip encode the given value.
#[macro_export]
macro_rules! rt {
    ($enum:ident :: $variant:ident $($body:tt)?) => {
        $crate::rt!($enum, $enum :: $variant $($body)*)
    };

    ($struct:ident $($body:tt)?) => {
        $crate::rt!($struct, $struct $($body)*)
    };

    ($ty:ty, $expr:expr) => {{
        let value: $ty = $expr;
        let out = $crate::to_vec(&value).expect(concat!("self: ", stringify!($ty), ": failed to encode"));
        let decoded: $ty = $crate::from_slice(&out[..]).expect(concat!("self: ", stringify!($ty), ": failed to decode"));
        // assert!(buf.is_empty(), concat!("self: ", stringify!($ty), ": decoded buffer should be empty.\nwas: {:?}\noriginal: {:?}\n"), buf, &out[..]);
        assert_eq!(decoded, $expr, concat!("self: ", stringify!($ty), ": roundtrip does not match"));
        decoded
    }};
}

/// Encode a type as one and decode as another.
#[inline(never)]
pub fn transcode<T, O>(value: T) -> O
where
    T: Debug + PartialEq + Encode<DefaultMode>,
    O: for<'de> Decode<'de, DefaultMode>,
{
    let out = crate::to_vec(&value).expect("failed to encode");
    let mut buf = &out[..];
    let value: O = crate::decode(&mut buf).expect("failed to decode");
    assert!(buf.is_empty());
    value
}
