//! Type flags available for `musli-wire`.

use core::fmt;
use core::mem;

use musli::mode::Mode;
use musli::{Decode, Decoder};

/// The marker indicating an absent value.
pub const ABSENT: u8 = 0b00000;
/// The marker indicating a value that is present.
pub const PRESENT: u8 = 0b00001;
/// The marker indicating the value true.
pub const TRUE: u8 = 0b00011;
/// The marker indicating the value false.
pub const FALSE: u8 = 0b00010;
/// The marker indicating that the value is a variant.
pub const VARIANT: u8 = 0b00100;

/// 8-bit unsigned number.
pub const U8: u8 = 0b00001;
/// 8-bit signed number.
pub const I8: u8 = 0b10001;
/// 16-bit unsigned number.
pub const U16: u8 = 0b00010;
/// 16-bit signed number.
pub const I16: u8 = 0b10010;
/// 32-bit unsigned number.
pub const U32: u8 = 0b00011;
/// 32-bit signed number.
pub const I32: u8 = 0b10011;
/// 32-bit float hint.
pub const F32: u8 = 0b01011;
/// 64-bit unsigned number.
pub const U64: u8 = 0b00100;
/// 64-bit signed number.
pub const I64: u8 = 0b10100;
/// 64-bit float hint.
pub const F64: u8 = 0b01100;
/// 128-bit number hint.
pub const U128: u8 = 0b00101;
/// 128-bit signed number.
pub const I128: u8 = 0b10101;
/// The marker for a usize.
#[cfg(target_pointer_width = "32")]
pub const USIZE: u8 = U32;
/// The marker for a usize.
#[cfg(target_pointer_width = "64")]
pub const USIZE: u8 = U64;
/// The marker for a isize.
#[cfg(target_pointer_width = "32")]
pub const ISIZE: u8 = I32;
/// The marker for a isize.
#[cfg(target_pointer_width = "64")]
pub const ISIZE: u8 = I64;

/// Data masked into the data type.
pub(crate) const DATA_MASK: u8 = 0b000_11111;
/// The maximum length that can be inlined in the tag without adding additional
/// data to the wire format.
pub const MAX_INLINE_LEN: usize = (DATA_MASK - 1) as usize;

/// The structure of a type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    /// A positive continuation-encoded numerical value.
    Number = 0b000_00000,
    /// A length-prefixed sequence of value.
    Sequence = 0b001_00000,
    /// A length-prefixed map.
    Map = 0b010_00000,
    /// A sequence of raw bytes.
    Bytes = 0b011_00000,
    /// A variant.
    Variant = 0b100_00000,
    /// A marker value.
    Marker = 0b101_00000,
    /// An unknown tag.
    Unknown = 0b111_00000,
}

/// A type tag.
///
/// The [Kind] of the element is indicates by its 2 MSBs, and remaining 6 bits
/// is the data field. The exact use of the data field depends on the [Kind] in
/// question. It is primarily used to smuggle extra data for the kind in
/// question.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Tag {
    /// The internal representation of the tag.
    repr: u8,
}

impl Tag {
    /// Construct a new tag through an unchecked constructor.
    ///
    /// `data` must not be equal to or larger than [MAX_INLINE_LEN], or else it
    /// could corrupt the payload.
    #[inline]
    pub const fn new(kind: Kind, data: u8) -> Self {
        Self {
            repr: kind as u8 | data,
        }
    }

    /// Construct a new empty tag of the given [Kind].
    #[inline]
    pub const fn empty(kind: Kind) -> Self {
        Self {
            repr: kind as u8 | DATA_MASK,
        }
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

    /// Access the kind of the tag.
    #[inline]
    pub const fn kind(self) -> Kind {
        // SAFETY: this is safe because we've ensured that all available Kind
        // variants occupy all available bit patterns.
        unsafe { mem::transmute(self.repr & !DATA_MASK) }
    }

    /// Perform raw access over the data payload. Will return [DATA_MASK] if
    /// data is empty.
    #[inline]
    const fn data_raw(self) -> u8 {
        self.repr & DATA_MASK
    }

    /// Perform checked access over the internal data. Returns [None] if data is
    /// empty.
    #[inline]
    pub const fn data(self) -> Option<u8> {
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
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tag")
            .field("kind", &self.kind())
            .field("data", &self.data())
            .finish()
    }
}

impl<'de, M> Decode<'de, M> for Tag
where
    M: Mode,
{
    #[inline]
    fn decode<D>(decoder: D) -> Result<Self, D::Error>
    where
        D: Decoder<'de>,
    {
        Ok(Self::from_byte(decoder.decode_u8()?))
    }
}
