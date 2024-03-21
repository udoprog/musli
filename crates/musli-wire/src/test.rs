//! Helpers for writing tests.

use core::fmt::Debug;

use musli::de::PackDecoder;
use musli::mode::DefaultMode;
use musli::Context;
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
    fn decode<C, D>(cx: &C, decoder: D) -> Result<Self, C::Error>
    where
        C: ?Sized + Context<Mode = M>,
        D: Decoder<'de, C>,
    {
        let mut unpack = decoder.decode_pack(cx)?;
        let tag = cx.decode(unpack.decode_next(cx)?)?;
        let value = cx.decode(unpack.decode_next(cx)?)?;
        unpack.end(cx)?;
        Ok(Self { tag, value })
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
        let out = $crate::to_vec(&value).expect(concat!("wire: ", stringify!($ty), ": failed to encode"));
        let decoded: $ty = $crate::from_slice(out.as_slice()).expect(concat!("wire: ", stringify!($ty), ": failed to decode"));
        assert_eq!(decoded, $expr, concat!("wire: ", stringify!($ty), ": roundtrip does not match"));
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
