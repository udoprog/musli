//! Type flags available for `musli-wire`.

use std::mem;

use musli::{Decode, Decoder};

/// Mark for the empty placeholder.
pub const EMPTY: Tag = Tag::new(Kind::Mark, 0);
/// Mark for a present value.
pub const SOME: Tag = Tag::new(Kind::Mark, 1);
/// Mark for a continuation sequence.
pub const CONTINUATION: Tag = Tag::new(Kind::Mark, 2);

/// Data masked into the data type.
const DATA_MASK: u8 = 0b000_11111;

/// The structure of a type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    /// A single byte.
    Byte = 0b000_00000,
    /// A fixed element where the length how many bytes it consists of.
    Fixed = 0b001_00000,
    /// A length-prefixed byte sequence. The length bits indicate the length of
    /// the sequence unless they are all set to 1s.
    Prefixed = 0b010_00000,
    /// A length-prefixed sequence of typed values. The length bits indicate the
    /// length of the sequence unless they are all set to 1s.
    Sequence = 0b011_00000,
    /// A length-prefixed sequence of typed pairs of values.
    PairSequence = 0b100_00000,
    /// A special kind of mark.
    Mark = 0b101_00000,
    /// Unknown kind.
    Unknown0 = 0b110_00000,
    /// Unknown kind.
    Unknown1 = 0b111_00000,
}

/// A type tag.
///
/// The type of the element is the 3 MSBs, which indicates that it's one of the
/// specified variants in the [Kind] enumeration.
///
/// The remaining 5 bits are the data field, and its use depends on the [Kind]
/// in question. Usually it's just used to smuggle extra data in case a value is
/// small (which it usually is).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Tag {
    /// The internal representation of the tag.
    repr: u8,
}

impl Tag {
    /// Construct a new tag.
    ///
    /// If `data` is larger or equal to [DATA_MASK] it is considered as empty.
    #[inline]
    pub const fn new(kind: Kind, data: u8) -> Self {
        Self {
            repr: kind as u8 | if data < DATA_MASK { data } else { DATA_MASK },
        }
    }

    /// Access the kind of the tag.
    #[inline]
    pub const fn kind(self) -> Kind {
        // SAFETY: this is safe because we've ensured that all available Kind
        // variants occupy all available bit patterns.
        unsafe { mem::transmute(self.repr & !DATA_MASK) }
    }

    /// Access the data of the tag.
    #[inline]
    pub const fn data_raw(self) -> u8 {
        self.repr & DATA_MASK
    }

    /// Construct from a byte.
    #[inline]
    pub const fn from_byte(repr: u8) -> Self {
        Self { repr }
    }

    /// Coerce type flag into a byte.
    #[inline]
    pub const fn byte(self) -> u8 {
        self.repr
    }

    /// Attempt to construct a type tag with the given length embedded.
    ///
    /// Returns a tuple where the boolean indicates if the value was embedded or
    /// not.
    #[inline]
    pub const fn with_len(kind: Kind, len: usize) -> (Self, bool) {
        if len < DATA_MASK as usize {
            (Self::new(kind, len as u8), true)
        } else {
            (Self::new(kind, DATA_MASK), false)
        }
    }

    /// Attempt to construct a type tag with the given length embedded.
    ///
    /// Returns a tuple where the boolean indicates if the value was embedded or
    /// not.
    #[inline]
    pub const fn with_byte(kind: Kind, len: u8) -> (Self, bool) {
        if len < DATA_MASK {
            (Self::new(kind, len), true)
        } else {
            (Self::new(kind, DATA_MASK), false)
        }
    }

    /// Get the embedded length as a byte.
    #[inline]
    pub const fn data(self) -> Option<u8> {
        if self.data_raw() == DATA_MASK {
            None
        } else {
            Some(self.data_raw())
        }
    }
}

impl<'de> Decode<'de> for Tag {
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(Self::from_byte(decoder.decode_u8()?))
    }
}
