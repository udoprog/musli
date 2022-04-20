//! Helpers for writing tests.

use core::fmt::Debug;
use musli::{Decode, Encode};

/// Roundtrip encode the given value.
#[macro_export]
macro_rules! rt {
    ($expr:expr) => {{
        let mut value = $expr;
        let out = $crate::to_vec(&value).expect("encode failed");
        let mut buf = &out[..];
        value = $crate::decode(&mut buf).expect("decode failed");
        assert!(buf.is_empty());
        assert_eq!(value, $expr);
        value
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
