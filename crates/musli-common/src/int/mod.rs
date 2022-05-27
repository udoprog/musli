//! Traits and utilities for dealing with integers.
//!
//! In particular the [Signed] and [Unsigned] traits are defined in here which
//! allows for musli to work over signed and unsigned numbers generically.
//!
//! We also have a 7-bit [continuation] encoding, and [zigzag] encoding which
//! are based on these.

mod byteorder;
pub mod continuation;
mod encoding;
mod formats;
mod traits;
pub mod zigzag;

pub use self::byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, NetworkEndian};
pub use self::encoding::{IntegerEncoding, UsizeEncoding};
pub use self::formats::{Fixed, FixedUsize, Variable};
pub use self::traits::{ByteOrderIo, Signed, Unsigned};

#[cfg(test)]
mod tests;
