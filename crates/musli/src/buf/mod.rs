//! [`Buf`] utilities.
//!
//! [`Buf`]: crate::Buf

mod buf_string;
pub(crate) use self::buf_string::collect_string;
pub use self::buf_string::BufString;

mod buf_vec;
pub use self::buf_vec::BufVec;

#[doc(inline)]
pub use musli_core::buf::{Buf, Error};
