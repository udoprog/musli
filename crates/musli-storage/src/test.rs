//! Helpers for writing tests.

use anyhow::Result;
use core::fmt::Debug;
use musli::{Decode, Encode};

/// Roundtrip encode the given value.
pub fn rt<T>(value: T) -> Result<T>
where
    T: Debug + PartialEq + for<'de> Decode<'de> + Encode,
{
    let out = crate::to_vec(&value)?;
    let mut buf = &out[..];
    let value: T = crate::decode(&mut buf)?;
    assert!(buf.is_empty());
    assert_eq!(value, value);
    Ok(value)
}

/// Encode a type as one and decode as another.
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
