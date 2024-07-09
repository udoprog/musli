//! Helpers for writing tests.

#[cfg(test)]
use core::fmt::Debug;

#[cfg(test)]
use crate::{Decode, Encode};

#[cfg(test)]
use super::tag::Tag;

/// A typed field, which is prefixed with a type tag.
///
/// This is used in combination with the storage deserializer to "inspect" type
/// tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[musli(crate, packed)]
#[cfg(test)]
pub(crate) struct Typed<const N: usize> {
    tag: Tag,
    #[musli(bytes)]
    value: [u8; N],
}

#[cfg(test)]
impl<const N: usize> Typed<N> {
    /// Construct a new typed field.
    pub(crate) const fn new(tag: Tag, value: [u8; N]) -> Self {
        Self { tag, value }
    }
}

crate::macros::test_fns!(Binary, "wire");
