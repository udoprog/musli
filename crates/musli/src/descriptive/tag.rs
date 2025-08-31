//! Type flags available for `musli::descriptive`.

#![allow(clippy::unusual_byte_groupings)]

use core::fmt;
use core::mem;

use crate::{Allocator, Decode, Decoder};

/// Variant corresponding to marks.
#[derive(Debug)]
#[repr(u8)]
pub(crate) enum Mark {
    /// The marker indicating an absent value.
    None = 0b000,
    /// The marker indicating a value that is present.
    Some = 0b001,
    /// The marker indicating the value true.
    True = 0b010,
    /// The marker indicating the value false.
    False = 0b011,
    /// The marker indicating that the value is a variant.
    Variant = 0b100,
    /// A single character.
    Char = 0b101,
    /// A unit type.
    Unit = 0b110,
    /// A reserved mark.
    #[allow(unused)]
    Reserved0 = 0b111,
}

/// The kind of a number.
///
/// Not that this enum occupies all possible low 2-bit patterns, which allows it
/// to be transmuted from a byte masked over `0b11`.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub(crate) enum NumberKind {
    /// The numerical type is a signed value.
    Signed = 0b00,
    /// The numerical type is an unsigned value.
    Unsigned = 0b01,
    /// The numerical type is a float.
    Float = 0b10,
    /// Reserved number kind.
    #[allow(unused)]
    Reserved0 = 0b11,
}

/// The width of the field.
///
/// Note that this representation directly corresponds to 2 to the power of the
/// specified width.
#[repr(u8)]
pub(crate) enum Width {
    #[allow(unused)]
    Reserved0 = 0b000,
    #[allow(unused)]
    Reserved1 = 0b001,
    #[allow(unused)]
    Reserved2 = 0b010,
    /// 8-bit width.
    U8 = 0b011,
    /// 16-bit width.
    U16 = 0b100,
    /// 32-bit width.
    U32 = 0b101,
    /// 64-bit width.
    U64 = 0b110,
    /// 128-bit width.
    U128 = 0b111,
}

#[test]
fn ensure_width() {
    assert_eq!(2u32.pow(Width::U8 as u32), 8u32);
    assert_eq!(1u32 << Width::U8 as u32, 8u32);
    assert_eq!(2u32.pow(Width::U16 as u32), 16u32);
    assert_eq!(1u32 << Width::U16 as u32, 16u32);
    assert_eq!(2u32.pow(Width::U32 as u32), 32u32);
    assert_eq!(1u32 << Width::U32 as u32, 32u32);
    assert_eq!(2u32.pow(Width::U64 as u32), 64u32);
    assert_eq!(1u32 << Width::U64 as u32, 64u32);
    assert_eq!(2u32.pow(Width::U128 as u32), 128u32);
    assert_eq!(1u32 << Width::U128 as u32, 128u32);
}

/// 8-bit unsigned number.
pub(crate) const U8: u8 = ((Width::U8 as u8) << 2) | NumberKind::Unsigned as u8;
/// 16-bit unsigned number.
pub(crate) const U16: u8 = ((Width::U16 as u8) << 2) | NumberKind::Unsigned as u8;
/// 32-bit unsigned number.
pub(crate) const U32: u8 = ((Width::U32 as u8) << 2) | NumberKind::Unsigned as u8;
/// 64-bit unsigned number.
pub(crate) const U64: u8 = ((Width::U64 as u8) << 2) | NumberKind::Unsigned as u8;
/// 128-bit number hint.
pub(crate) const U128: u8 = ((Width::U128 as u8) << 2) | NumberKind::Unsigned as u8;
/// 8-bit signed number.
pub(crate) const I8: u8 = ((Width::U8 as u8) << 2) | NumberKind::Signed as u8;
/// 16-bit signed number.
pub(crate) const I16: u8 = ((Width::U16 as u8) << 2) | NumberKind::Signed as u8;
/// 32-bit signed number.
pub(crate) const I32: u8 = ((Width::U32 as u8) << 2) | NumberKind::Signed as u8;
/// 64-bit signed number.
pub(crate) const I64: u8 = ((Width::U64 as u8) << 2) | NumberKind::Signed as u8;
/// 128-bit signed number.
pub(crate) const I128: u8 = ((Width::U128 as u8) << 2) | NumberKind::Signed as u8;
/// 32-bit float hint.
pub(crate) const F32: u8 = ((Width::U32 as u8) << 2) | NumberKind::Float as u8;
/// 64-bit float hint.
pub(crate) const F64: u8 = ((Width::U64 as u8) << 2) | NumberKind::Float as u8;
/// The marker for a usize.
#[cfg(target_pointer_width = "32")]
pub(crate) const USIZE: u8 = U32;
/// The marker for a usize.
#[cfg(target_pointer_width = "64")]
pub(crate) const USIZE: u8 = U64;
/// The marker for a isize.
#[cfg(target_pointer_width = "32")]
pub(crate) const ISIZE: u8 = I32;
/// The marker for a isize.
#[cfg(target_pointer_width = "64")]
pub(crate) const ISIZE: u8 = I64;

/// Data masked into the data type.
pub(crate) const DATA_MASK: u8 = 0b000_11111;
pub(crate) const MARK_MASK: u8 = 0b00000_111;
pub(crate) const NUMBER_KIND_MASK: u8 = 0b000000_11;

/// The structure of a type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Kind {
    /// Reserved 0.
    #[allow(unused)]
    Reserved0 = 0b000_00000,
    /// Reserved 1.
    #[allow(unused)]
    Reserved1 = 0b001_00000,
    /// A continuation-encoded numerical value.
    Number = 0b010_00000,
    /// A length-prefixed sequence of value.
    Sequence = 0b011_00000,
    /// A length-prefixed map.
    Map = 0b100_00000,
    /// A sequence of raw bytes.
    Bytes = 0b101_00000,
    /// A string.
    String = 0b110_00000,
    /// A distinct mark.
    Mark = 0b111_00000,
}

/// A type tag.
///
/// The [Kind] of the element is indicates by its 2 MSBs, and remaining 6 bits
/// is the data field. The exact use of the data field depends on the [Kind] in
/// question. It is primarily used to smuggle extra data for the kind in
/// question.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct Tag {
    /// The internal representation of the tag.
    repr: u8,
}

impl Tag {
    /// Construct a new tag through an unchecked constructor.
    ///
    /// The `data` argument must fit within the numerical bounds specified by
    /// [`DATA_MASK`].
    #[inline]
    pub(crate) const fn new(kind: Kind, data: u8) -> Self {
        debug_assert!(data <= DATA_MASK, "Data must fit in DATA_MASK");

        Self {
            repr: kind as u8 | data,
        }
    }

    /// Construct a tag corresponding to a mark.
    #[inline]
    pub(crate) const fn from_mark(mark: Mark) -> Self {
        Self {
            repr: Kind::Mark as u8 | mark as u8,
        }
    }

    /// Access the mark this byte corresponds to.
    #[inline]
    pub(crate) const fn mark(&self) -> Mark {
        // SAFETY: The representation used by `Mark` is exhaustive over all
        // emitted bit-patterns.
        unsafe { mem::transmute(self.repr & MARK_MASK) }
    }

    /// Access the number kind this tag corresponds to.
    #[inline]
    pub(crate) const fn number_kind(&self) -> NumberKind {
        // SAFETY: The representation used by `Mark` is exhaustive over all
        // emitted bit-patterns.
        unsafe { mem::transmute(self.repr & NUMBER_KIND_MASK) }
    }

    /// Construct from a byte.
    #[inline]
    pub(crate) const fn from_byte(repr: u8) -> Self {
        Self { repr }
    }

    /// Coerce type flag into a byte.
    #[inline]
    pub(crate) const fn byte(self) -> u8 {
        self.repr
    }

    /// Access the kind of the tag.
    #[inline]
    pub(crate) const fn kind(self) -> Kind {
        // SAFETY: this is safe because we've ensured that all available Kind
        // variants occupy all available bit patterns.
        unsafe { mem::transmute(self.repr & !DATA_MASK) }
    }

    /// Perform raw access over the data payload. Will return [DATA_MASK] if
    /// data is empty.
    #[inline]
    pub(crate) const fn data_raw(self) -> u8 {
        self.repr & DATA_MASK
    }

    /// Perform checked access over the internal data. Returns [None] if data is
    /// empty.
    #[inline]
    pub(crate) const fn data(self) -> Option<u8> {
        let data = self.data_raw();

        if data == DATA_MASK { None } else { Some(data) }
    }

    /// Attempt to construct a type tag with the given length embedded.
    ///
    /// Returns a tuple where the boolean indicates if the value was embedded or
    /// not.
    #[inline]
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

impl<'de, M, A> Decode<'de, M, A> for Tag
where
    A: Allocator,
{
    // Every bit pattern is valid for a tag.
    const IS_BITWISE_DECODE: bool = true;

    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de, Mode = M, Allocator = A>,
    {
        Ok(Self::from_byte(decoder.decode_u8()?))
    }
}
