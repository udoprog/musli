//! A variable-length 7-bit encoder where each bit indicates if there is a
//! continuation of the sequence or not.
//!
//! ```
//! use musli_common::int::continuation as c;
//! use musli_common::fixed_bytes::FixedBytes;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut bytes = FixedBytes::<8>::new();
//! c::encode(&mut bytes, 5000u32)?;
//! assert_eq!(bytes.as_slice(), &[0b1000_1000, 0b0010_0111]);
//!
//! let number: u32 = c::decode(bytes.as_slice())?;
//! assert_eq!(number, 5000u32);
//! # Ok(()) }
//! ```

#![allow(unused)]

use crate::int;
use crate::reader::Reader;
use crate::writer::Writer;
use musli::error::Error;

use super::Unsigned;

const MASK_BYTE: u8 = 0b0111_1111;
const CONT_BYTE: u8 = 0b1000_0000;

/// Decode the given length using variable int encoding.
#[inline(never)]
pub fn decode<'de, R, T>(mut r: R) -> Result<T, R::Error>
where
    R: Reader<'de>,
    T: int::Unsigned,
{
    let mut b = r.read_byte()?;

    if b & 0b1000_0000 == 0 {
        return Ok(T::from_byte(b));
    }

    let mut value = T::from_byte(b & MASK_BYTE);
    let mut shift = 0u32;

    while b & CONT_BYTE == CONT_BYTE {
        shift += 7;
        b = r.read_byte()?;

        value = T::from_byte(b & MASK_BYTE)
            .checked_shl(shift)
            .and_then(|add| value.checked_add(add))
            .ok_or_else(|| R::Error::custom("length overflow"))?;
    }

    Ok(value)
}

/// Encode the given length using variable length encoding.
#[inline(never)]
pub fn encode<W, T>(mut w: W, mut value: T) -> Result<(), W::Error>
where
    W: Writer,
    T: int::Unsigned,
{
    let mut b = value.as_byte();

    if value < T::from_byte(0b1000_0000) {
        w.write_byte(b)?;
        return Ok(());
    }

    loop {
        value = value
            .checked_shr(7)
            .ok_or_else(|| W::Error::custom("length underflow"))?;

        if value.is_zero() {
            w.write_byte(b & MASK_BYTE)?;
            break;
        }

        w.write_byte(b | CONT_BYTE)?;
        b = value.as_byte();
    }

    Ok(())
}
