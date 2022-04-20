//! Helpers for writing tests.

use core::fmt::Debug;
use musli::{Decode, Encode};

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
        let out = $crate::to_vec(&value).expect(concat!("storage: ", stringify!($ty), ": failed to encode"));
        let mut buf = &out[..];
        let decoded: $ty = $crate::from_slice(buf).expect(concat!("storage: ", stringify!($ty), ": failed to decode"));
        // assert!(buf.is_empty(), concat!("storage: ", stringify!($ty), ": decoded buffer should be empty.\nwas: {:?}\noriginal: {:?}\n"), buf, &out[..]);
        assert_eq!(decoded, $expr, concat!("storage: ", stringify!($ty), ": roundtrip does not match"));
        decoded
    }};
}

/// Encode a type as one and decode as another.
#[inline(never)]
pub fn transcode<T, O>(value: T) -> O
where
    T: Debug + PartialEq + Encode,
    O: for<'de> Decode<'de>,
{
    let out = crate::to_vec(&value).expect("encode failed");
    let mut buf = &out[..];
    let value: O = crate::decode(&mut buf).expect("decode failed");
    assert!(buf.is_empty());
    value
}
