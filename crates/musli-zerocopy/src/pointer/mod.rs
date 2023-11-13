//! Pointer-like types that can be used directly in [`ZeroCopy`] structs.
//!
//! Pointers are types which points to data inside of a [`Buf`], and can be used
//! in combination with methods such as [`Buf::load`] to load the pointer into a
//! reference.
//!
//! * [`Ref<T>`] is a simple pointer to a typed reference, where `T` implements
//!   [`ZeroCopy`]. It loads into `&T`.
//! * [`Ref<[T]>`] is a wide pointer encoding both a plain pointer and a length
//!   where `T` implements [`ZeroCopy`]. It loads into `&[T]`.
//! * [`Ref<T>`] where `T: ?Sized` is a wide pointer encoding both a plain
//!   pointer and a size to a typed reference where `T` implements
//!   [`UnsizedZeroCopy`]. It loads into `&T` and is implemented by types such
//!   as `str` and `[u8]`.`
//!
//! [`ZeroCopy`]: crate::traits::ZeroCopy
//! [`UnsizedZeroCopy`]: crate::traits::UnsizedZeroCopy
//! [`Buf`]: crate::buf::Buf
//! [`Buf::load`]: crate::buf::Buf::load

#[doc(inline)]
pub use self::size::{DefaultSize, Size};
mod size;

#[doc(inline)]
pub use self::r#ref::Ref;
mod r#ref;

#[doc(inline)]
pub use self::pointee::Pointee;
mod pointee;

#[doc(inline)]
pub use self::packable::Packable;
mod packable;

pub use self::coerce::Coerce;
mod coerce;

pub use self::coerce_slice::CoerceSlice;
mod coerce_slice;
