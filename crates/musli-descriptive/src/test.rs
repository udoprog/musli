//! Helpers for writing tests.

use core::fmt::Debug;

use musli::de::PackDecoder;
use musli::mode::DefaultMode;
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
    T: Decode<'de, M>,
{
    fn decode<D>(_: &D::Cx, decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M>,
    {
        decoder.decode_pack(|pack| {
            let tag = pack.decode_next()?.decode()?;
            let value = pack.decode_next()?.decode()?;
            Ok(Self { tag, value })
        })
    }
}

/// Roundtrip encode the given value.
#[macro_export]
#[doc(hidden)]
macro_rules! rt {
    ($enum:ident :: $variant:ident $($tt:tt)?) => {
        $crate::rt!($enum, $enum :: $variant $($tt)*)
    };

    ($struct:ident $($tt:tt)?) => {
        $crate::rt!($struct, $struct $($tt)*)
    };

    ($ty:ty, $expr:expr) => {{
        let value: $ty = $expr;
        let out = $crate::to_vec(&value).expect(concat!("descriptive: ", stringify!($ty), ": failed to encode"));
        let decoded: $ty = $crate::from_slice(out.as_slice()).expect(concat!("descriptive: ", stringify!($ty), ": failed to decode"));
        assert_eq!(decoded, $expr, concat!("descriptive: ", stringify!($ty), ": roundtrip does not match"));

        let value_decode: musli_value::Value = $crate::from_slice(out.as_slice()).expect(concat!("descriptive: ", stringify!($ty), ": failed to decode into value type"));
        let value_decoded: $ty = musli_value::decode(&value_decode).expect(concat!("descriptive: ", stringify!($ty), ": failed to decode from value type"));
        assert_eq!(value_decoded, $expr, concat!("descriptive: ", stringify!($ty), ": value roundtrip does not match"));
        decoded
    }};
}

/// Encode a type as one and decode as another.
#[inline(never)]
#[track_caller]
pub fn transcode<T, O>(value: T) -> O
where
    T: Debug + PartialEq + Encode<DefaultMode>,
    O: for<'de> Decode<'de, DefaultMode>,
{
    let out = crate::to_vec(&value).expect("failed to encode");
    let mut buf = out.as_slice();
    let value: O = crate::decode(&mut buf).expect("failed to decode");
    assert!(buf.is_empty());
    value
}
