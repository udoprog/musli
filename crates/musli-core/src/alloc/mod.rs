//! Traits related to memory allocation.

mod allocator;
#[doc(inline)]
pub use self::allocator::Allocator;

mod raw_vec;
#[doc(inline)]
pub use self::raw_vec::RawVec;
