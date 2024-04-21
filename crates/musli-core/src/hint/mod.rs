//! Core encoding hints.
//!
//! These are passed when encoding or decoding different types.

mod map_hint;
pub use self::map_hint::MapHint;

mod sequence_hint;
pub use self::sequence_hint::SequenceHint;
