//! Type flags available for `musli::jsonb`.

#![allow(clippy::unusual_byte_groupings)]

use core::fmt;
use core::mem;

use crate::{Allocator, Decode, Decoder};

/// 8-bit unsigned number.
pub(crate) const U8: u8 = 8;
/// 16-bit unsigned number.
pub(crate) const U16: u8 = 16;
/// 32-bit unsigned number.
pub(crate) const U32: u8 = 32;
/// 64-bit unsigned number.
pub(crate) const U64: u8 = 64;
/// 128-bit number hint.
pub(crate) const U128: u8 = 128;
/// 8-bit signed number.
pub(crate) const I8: u8 = 8;
/// 16-bit signed number.
pub(crate) const I16: u8 = 16;
/// 32-bit signed number.
pub(crate) const I32: u8 = 32;
/// 64-bit signed number.
pub(crate) const I64: u8 = 64;
/// 128-bit signed number.
pub(crate) const I128: u8 = 128;
/// 32-bit float hint.
pub(crate) const F32: u8 = 32;
/// 64-bit float hint.
pub(crate) const F64: u8 = 64;
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
pub(crate) const SIZE_MASK: u8 = 0xf0;
pub(crate) const KIND_MASK: u8 = 0x0f;

/// The structure of a type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Kind {
    /// The element is a JSON "null". The payload size for a true JSON NULL must
    /// must be zero. Future versions of SQLite might extend the JSONB format
    /// with elements that have a zero element type but a non-zero size. In that
    /// way, legacy versions of SQLite will interpret the element as a NULL for
    /// backwards compatibility while newer versions will interpret the element
    /// in some other way.
    Null = 0x00,
    /// The element is a JSON "true". The payload size must be zero for a actual
    /// "true" value. Elements with type 1 and a non-zero payload size are
    /// reserved for future expansion. Legacy implementations that see an
    /// element type of 1 with a non-zero payload size should continue to
    /// interpret that element as "true" for compatibility.
    True = 0x01,
    /// The element is a JSON "false". The payload size must be zero for a
    /// actual "false" value. Elements with type 2 and a non-zero payload size
    /// are reserved for future expansion. Legacy implementations that see an
    /// element type of 2 with a non-zero payload size should continue to
    /// interpret that element as "false" for compatibility.
    False = 0x02,
    /// The element is a JSON integer value in the canonical RFC 8259 format,
    /// without extensions. The payload is the ASCII text representation of that
    /// numeric value.
    Int = 0x03,
    /// The element is a JSON integer value that is not in the canonical format.
    /// The payload is the ASCII text representation of that numeric value.
    /// Because the payload is in a non-standard format, it will need to be
    /// translated when the JSONB is converted into RFC 8259 text JSON.
    Int5 = 0x04,
    /// The element is a JSON floating-point value in the canonical RFC 8259
    /// format, without extensions. The payload is the ASCII text representation
    /// of that numeric value.
    Float = 0x05,
    /// The element is a JSON floating-point value that is not in the canonical
    /// format. The payload is the ASCII text representation of that numeric
    /// value. Because the payload is in a non-standard format, it will need to
    /// be translated when the JSONB is converted into RFC 8259 text JSON.
    Float5 = 0x06,
    /// The element is a JSON string value that does not contain any escapes nor
    /// any characters that need to be escaped for either SQL or JSON. The
    /// payload is the UTF8 text representation of the string value. The payload
    /// does not include string delimiters.
    Text = 0x07,
    /// The element is a JSON string value that contains RFC 8259 character
    /// escapes (such as "\n" or "\u0020"). Those escapes will need to be
    /// translated into actual UTF8 if this element is extracted into SQL. The
    /// payload is the UTF8 text representation of the escaped string value. The
    /// payload does not include string delimiters.
    TextJ = 0x08,
    /// The element is a JSON string value that contains character escapes,
    /// including some character escapes that part of JSON5 and which are not
    /// found in the canonical RFC 8259 spec. Those escapes will need to be
    /// translated into standard JSON prior to rendering the JSON as text, or
    /// into their actual UTF8 characters if this element is extracted into SQL.
    /// The payload is the UTF8 text representation of the escaped string value.
    /// The payload does not include string delimiters.
    Text5 = 0x09,
    /// The element is a JSON string value that contains UTF8 characters that
    /// need to be escaped if this string is rendered into standard JSON text.
    /// The payload does not include string delimiters.
    TextRaw = 0x0a,
    /// The element is a JSON array. The payload contains JSONB elements that
    /// comprise values contained within the array.
    Array = 0x0b,
    /// The element is a JSON object. The payload contains pairs of JSONB
    /// elements that comprise entries for the JSON object. The first element in
    /// each pair must be a string (types 7 through 10). The second element of
    /// each pair may be any types, including nested arrays or objects.
    Object = 0x0c,
    /// Reserved for future expansion. Legacy implements that encounter this
    /// element type should raise an error.
    Reserved13 = 0x0d,
    /// Reserved for future expansion. Legacy implements that encounter this
    /// element type should raise an error.
    Reserved14 = 0x0e,
    /// Reserved for future expansion. Legacy implements that encounter this
    /// element type should raise an error.
    Reserved15 = 0x0f,
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
    #[inline]
    pub(crate) fn from_byte(repr: u8) -> Self {
        Self { repr }
    }

    #[inline]
    pub(crate) fn bits(self) -> u8 {
        self.repr
    }

    /// Construct a new tag through an unchecked constructor.
    ///
    /// The `size` argument must fit within 4 bits.
    #[inline]
    pub(crate) const fn new(kind: Kind, size: u8) -> Self {
        debug_assert!(size <= 0xf, "size must fit in 4 bits");

        Self {
            repr: kind as u8 | (size << 4),
        }
    }

    /// Access the kind of the tag.
    #[inline]
    pub(crate) const fn kind(self) -> Kind {
        // SAFETY: this is safe because we've ensured that all available Kind
        // variants occupy all available bit patterns.
        unsafe { mem::transmute(self.repr & KIND_MASK) }
    }

    /// Access the size of the value.
    #[inline]
    pub(crate) const fn size(self) -> u8 {
        (self.repr & SIZE_MASK) >> 4
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tag")
            .field("kind", &self.kind())
            .field("size", &self.size())
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
