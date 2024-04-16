//! Helpers for writing tests.

use core::fmt::Debug;

use musli::mode::Binary;
use musli::{Decode, Encode};

use crate::tag::Tag;

/// A typed field, which is prefixed with a type tag.
///
/// This is used in combination with the storage deserializer to "inspect" type
/// tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[musli(packed)]
pub struct Typed<const N: usize> {
    tag: Tag,
    #[musli(bytes)]
    value: [u8; N],
}

impl<const N: usize> Typed<N> {
    /// Construct a new typed field.
    pub const fn new(tag: Tag, value: [u8; N]) -> Self {
        Self { tag, value }
    }
}

musli_utils::test_fns!("wire");

/// Encode a type as one and decode as another.
#[inline(never)]
#[track_caller]
pub fn transcode<T, O>(value: T) -> O
where
    T: Debug + PartialEq + Encode<Binary>,
    O: for<'de> Decode<'de, Binary>,
{
    let out = crate::to_vec(&value).expect("failed to encode");
    let mut buf = out.as_slice();
    let value: O = crate::decode(&mut buf).expect("failed to decode");
    assert!(buf.is_empty());
    value
}
