//! Helpers for writing tests.

use core::fmt::Debug;

use crate::{Decode, Encode};

use super::tag::Tag;

/// A typed field, which is prefixed with a type tag.
///
/// This is used in combination with the storage deserializer to "inspect" type
/// tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[musli(crate, packed)]
pub(crate) struct Typed<const N: usize> {
    tag: Tag,
    #[musli(bytes)]
    value: [u8; N],
}

impl<const N: usize> Typed<N> {
    /// Construct a new typed field.
    #[cfg(test)]
    pub(crate) const fn new(tag: Tag, value: [u8; N]) -> Self {
        Self { tag, value }
    }
}

crate::macros::test_fns!("wire", crate::mode::Binary);
