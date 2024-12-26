//! Type flags available for `musli::wire`.

#![allow(clippy::unusual_byte_groupings)]

use core::fmt;
use core::mem;

#[cfg(feature = "test")]
use crate::{Decode, Encode};

/// Data masked into the data type.
pub(crate) const DATA_MASK: u8 = 0b00_111111;

/// The structure of a type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Kind {
    /// A reserved value.
    #[allow(unused)]
    Reserved = 0b00_000000,
    /// A fixed element where data indicates how many bytes it consists of. Data
    /// contains the prefix length unless it's set to all 1s after which a
    /// continuation sequence indicating the length should be decoded.
    Prefix = 0b01_000000,
    /// A length-prefixed sequence of values. Data contains the length of the
    /// sequence if it's short enough to fit in 6 bits. All bits as 1s is
    /// reserved to indicate when it's empty.
    Sequence = 0b10_000000,
    /// A continuation-encoded value. Data is the immediate value embedded if
    /// it's small enough to fit in 6 bits. All bits as 1s is reserved to
    /// indicate when a continuation sequence is used.
    Continuation = 0b11_000000,
}

/// A type tag.
///
/// The [Kind] of the element is indicates by its 2 MSBs, and remaining 6 bits
/// is the data field. The exact use of the data field depends on the [Kind] in
/// question. It is primarily used to smuggle extra data for the kind in
/// question.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "test", derive(Encode, Decode))]
#[repr(transparent)]
#[cfg_attr(feature = "test", musli(crate, transparent))]
pub(crate) struct Tag {
    /// The internal representation of the tag.
    repr: u8,
}

impl Tag {
    /// Construct a new tag through an unchecked constructor.
    ///
    /// The `data` argument must fit within the numerical bounds specified by
    /// [`DATA_MASK`].
    #[inline(always)]
    pub(crate) const fn new(kind: Kind, data: u8) -> Self {
        debug_assert!(data <= DATA_MASK, "Data must fit in DATA_MASK");

        Self {
            repr: kind as u8 | data,
        }
    }

    /// Construct a new empty tag of the given [Kind].
    #[inline(always)]
    pub(crate) const fn empty(kind: Kind) -> Self {
        Self {
            repr: kind as u8 | DATA_MASK,
        }
    }

    /// Construct from a byte.
    #[inline(always)]
    pub(crate) const fn from_byte(repr: u8) -> Self {
        Self { repr }
    }

    /// Coerce type flag into a byte.
    #[inline(always)]
    pub(crate) const fn byte(self) -> u8 {
        self.repr
    }

    /// Access the kind of the tag.
    #[inline(always)]
    pub(crate) const fn kind(self) -> Kind {
        // SAFETY: this is safe because we've ensured that all available Kind
        // variants occupy all available bit patterns.
        unsafe { mem::transmute(self.repr & !DATA_MASK) }
    }

    /// Perform raw access over the data payload. Will return [DATA_MASK] if
    /// data is empty.
    #[inline(always)]
    pub(crate) const fn data_raw(self) -> u8 {
        self.repr & DATA_MASK
    }

    /// Perform checked access over the internal data. Returns [None] if data is
    /// empty.
    #[inline(always)]
    pub(crate) const fn data(self) -> Option<u8> {
        let data = self.data_raw();

        if data == DATA_MASK {
            None
        } else {
            Some(data)
        }
    }

    /// Attempt to construct a type tag with the given length embedded.
    ///
    /// Returns a tuple where the boolean indicates if the value was embedded or
    /// not.
    #[inline(always)]
    pub(crate) const fn with_len(kind: Kind, len: usize) -> (Self, bool) {
        if len < DATA_MASK as usize {
            (Self::new(kind, len as u8), true)
        } else {
            (Self::new(kind, DATA_MASK), false)
        }
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tag")
            .field("kind", &self.kind())
            .field("data", &self.data())
            .finish()
    }
}
