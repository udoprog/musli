//! Traits related to memory allocation.

mod allocator;
#[doc(inline)]
pub use self::allocator::Allocator;

mod buf;
#[doc(inline)]
pub use self::buf::Buf;
