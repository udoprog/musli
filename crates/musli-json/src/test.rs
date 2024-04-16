//! Helpers for writing tests.

use core::fmt::Debug;

use musli::mode::Binary;
use musli::{Decode, Encode};

musli_utils::test_fns!("json");

/// Encode a type as one and decode as another.
#[inline(never)]
#[track_caller]
pub fn transcode<T, O>(value: T) -> O
where
    T: Debug + PartialEq + Encode<Binary>,
    O: for<'de> Decode<'de, Binary>,
{
    let out = crate::to_vec(&value).expect("failed to encode");
    let buf = out.as_slice();
    let value: O = crate::from_slice(buf).expect("failed to decode");
    value
}
