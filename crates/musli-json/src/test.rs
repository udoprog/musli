//! Helpers for writing tests.

use core::fmt::Debug;

use musli::mode::Text;
use musli::{Decode, Encode};

musli_utils::test_fns!("json", ::musli::mode::Text);

/// Encode a type as one and decode as another.
#[inline(never)]
#[track_caller]
pub fn transcode<T, O>(value: T) -> O
where
    T: Debug + PartialEq + Encode<Text>,
    O: for<'de> Decode<'de, Text>,
{
    let out = crate::to_vec(&value).expect("failed to encode");
    let buf = out.as_slice();
    let value: O = crate::from_slice(buf).expect("failed to decode");
    value
}
