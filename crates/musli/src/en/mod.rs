//! Traits for generically dealing with an encoding framework.
//!
//! The central traits are [Encode] and [Encoder].
//!
//! A type implementing [Encode] can use an [Encoder] to encode itself. This
//! also comes with a derive allowing you to derive high performance encoding
//! associated with native Rust types.
//!
//! ```rust
//! use musli::Encode;
//!
//! #[derive(Encode)]
//! pub struct Person<'a> {
//!     name: &'a str,
//!     age: u32,
//! }
//! ```

mod encode;
mod encoder;

pub use self::encode::Encode;
pub use self::encoder::{Encoder, PairEncoder, PairsEncoder, SequenceEncoder};
