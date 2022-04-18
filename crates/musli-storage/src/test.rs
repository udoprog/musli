//! Helpers for writing tests.

use anyhow::Result;
use core::fmt::Debug;
use musli::{Decode, Encode};

/// Roundtrip encode the given value.
#[inline(never)]
pub fn rt<T>(expected: T) -> Result<T>
where
    T: Debug + PartialEq + for<'de> Decode<'de> + Encode,
{
    let out = crate::to_vec(&expected)?;
    let mut buf = &out[..];
    let value: T = crate::decode(&mut buf)?;
    assert!(buf.is_empty());
    assert_eq!(value, expected);
    Ok(value)
}

/// Encode a type as one and decode as another.
#[inline(never)]
pub fn transcode<T, O>(value: T) -> Result<O>
where
    T: Debug + PartialEq + Encode,
    O: for<'de> Decode<'de>,
{
    let out = crate::to_vec(&value)?;
    let mut buf = &out[..];
    let value: O = crate::decode(&mut buf)?;
    assert!(buf.is_empty());
    Ok(value)
}
