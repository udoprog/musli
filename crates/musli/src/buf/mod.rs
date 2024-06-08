//! [`Buf`] utilities.
//!
//! [`Buf`]: crate::Buf

mod buf_string;
pub(crate) use self::buf_string::collect_string;
pub use self::buf_string::BufString;

#[doc(inline)]
pub use musli_core::buf::{AlignedBuf, Buf, BytesBuf, Error};
