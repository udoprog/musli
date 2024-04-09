//! A variable-length 7-bit encoder where each bit indicates if there is a
//! continuation of the sequence or not.
//!
//! ```
//! use musli_utils::context::Ignore;
//! use musli_utils::fixed::FixedBytes;
//! use musli_utils::allocator;
//! use musli_utils::int::continuation as c;
//!
//! let mut buf = allocator::buffer();
//! let alloc = allocator::new(&mut buf);
//!
//! let cx = Ignore::marker(&alloc);
//! let mut bytes = FixedBytes::<8>::new();
//! c::encode(&cx, &mut bytes, 5000u32).unwrap();
//! assert_eq!(bytes.as_slice(), &[0b1000_1000, 0b0010_0111]);
//!
//! let cx = Ignore::marker(&alloc);
//! let number: u32 = c::decode(&cx, bytes.as_slice()).unwrap();
//! assert_eq!(number, 5000u32);
//! ```

#![allow(unused)]

use musli::de;
use musli::Context;

use crate::int;
use crate::reader::Reader;
use crate::writer::Writer;

use super::Unsigned;

const MASK_BYTE: u8 = 0b0111_1111;
const CONT_BYTE: u8 = 0b1000_0000;

/// Decode the given length using variable int encoding.
#[inline(never)]
pub fn decode<'de, C, R, T>(cx: &C, mut r: R) -> Result<T, C::Error>
where
    C: ?Sized + Context,
    R: Reader<'de>,
    T: int::Unsigned,
{
    let mut b = r.read_byte(cx)?;

    if b & 0b1000_0000 == 0 {
        return Ok(T::from_byte(b));
    }

    let mut value = T::from_byte(b & MASK_BYTE);
    let mut shift = 0u32;

    while b & CONT_BYTE == CONT_BYTE {
        shift += 7;

        if shift >= T::BITS {
            return Err(cx.message("Bits overflow"));
        }

        b = r.read_byte(cx)?;
        value = value.wrapping_add(T::from_byte(b & MASK_BYTE).wrapping_shl(shift));
    }

    Ok(value)
}

/// Encode the given length using variable length encoding.
#[inline(never)]
pub fn encode<C, W, T>(cx: &C, mut w: W, mut value: T) -> Result<(), C::Error>
where
    C: ?Sized + Context,
    W: Writer,
    T: int::Unsigned,
{
    let mut b = value.as_byte();

    if value < T::from_byte(0b1000_0000) {
        w.write_byte(cx, b)?;
        return Ok(());
    }

    loop {
        value = value >> 7;

        if value.is_zero() {
            w.write_byte(cx, b & MASK_BYTE)?;
            break;
        }

        w.write_byte(cx, b | CONT_BYTE)?;
        b = value.as_byte();
    }

    Ok(())
}
