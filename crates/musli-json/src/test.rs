//! Helpers for writing tests.

use core::fmt::Debug;

use musli::mode::DefaultMode;
use musli::{Decode, Encode};

/// Roundtrip encode the given value.
#[macro_export]
macro_rules! rt {
    ($enum:ident :: $variant:ident $($tt:tt)?) => {
        $crate::rt!($enum, $enum :: $variant $($tt)*)
    };

    ($struct:ident $($tt:tt)?) => {
        $crate::rt!($struct, $struct $($tt)*)
    };

    ($ty:ty, $expr:expr) => {{
        let value: $ty = $expr;
        let out = $crate::to_vec(&value).expect(concat!("json: ", stringify!($ty), ": failed to encode"));
        let decoded: $ty = $crate::from_slice(out.as_slice()).expect(concat!("json: ", stringify!($ty), ": failed to decode"));
        assert_eq!(decoded, $expr, concat!("json: ", stringify!($ty), ": roundtrip does not match"));

        let value_decode: musli_value::Value = $crate::from_slice(out.as_slice()).expect(concat!("json: ", stringify!($ty), ": failed to decode into value type"));
        let value_decoded: $ty = musli_value::decode(&value_decode).expect(concat!("json: ", stringify!($ty), ": failed to decode from value type"));
        assert_eq!(value_decoded, $expr, concat!("json: ", stringify!($ty), ": value roundtrip does not match"));
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
    let buf = out.as_slice();
    let value: O = crate::from_slice(buf).expect("failed to decode");
    value
}
