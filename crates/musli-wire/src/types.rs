//! Type flags available for `musli-wire`.

/// A single encoded byte. The contents of which is packed in 7 least
/// significant bits. If all LSBs are set to 1 (i.e. `0b1111_1111`), the next
/// byte is used as the byte of the tag. All other types avoids having the MSB
/// set.
pub const FIXED8: u8 = 0b1000_0000;
/// Read the entire next byte.
pub const FIXED8_NEXT: u8 = 0b1111_1111;
/// An absent optional value.
pub const OPTION_NONE: u8 = 0b0111_1110;
/// A present optional value.
pub const OPTION_SOME: u8 = 0b0111_1111;
/// A pair of typed values are being encoded.
pub const PAIR: u8 = 0b0100_0000;
/// Fixed-length 2 bytes.
pub const FIXED16: u8 = 0b0001_0010;
/// Fixed-length 4 bytes.
pub const FIXED32: u8 = 0b0001_0100;
/// Fixed-length 8 bytes.
pub const FIXED64: u8 = 0b0001_0110;
/// Fixed-length 16 bytes.
pub const FIXED128: u8 = 0b0001_1000;
/// The next integer is using continuation integer encoding.
pub const CONTINUATION: u8 = 0b0001_1010;
/// A length-prefixed byte sequence.
pub const PREFIXED: u8 = 0b0010_0000;
/// A length-prefixed sequence of typed values.
pub const SEQUENCE: u8 = 0b0010_0010;
/// A length-prefixed sequence of typed pairs of values.
pub const PAIR_SEQUENCE: u8 = 0b0010_0100;
