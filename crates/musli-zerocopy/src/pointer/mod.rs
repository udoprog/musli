//! Pointer-like types that can be used directly in [`ZeroCopy`] structs.
//!
//! Pointers are types which points to data inside of a [`Buf`], and can be used
//! in combination with methods such as [`Buf::load`] to load the pointer into a
//! reference.
//!
//! * [`Ref<P>`] is a simple pointer to a typed reference, where `T` implements
//!   [`ZeroCopy`]. It loads into `&T`.
//! * [`Ref<[T]>`] is a wide pointer encoding both a plain pointer and a length
//!   where `T` implements [`ZeroCopy`]. It loads into `&[T]`.
//! * [`Ref<P>`] where `T: ?Sized` is a wide pointer encoding both a plain
//!   pointer and a size to a typed reference where `T` implements
//!   [`UnsizedZeroCopy`]. It loads into `&T` and is implemented by types such
//!   as `str` and `[u8]`.`
//!
//! [`ZeroCopy`]: crate::traits::ZeroCopy
//! [`UnsizedZeroCopy`]: crate::traits::UnsizedZeroCopy
//! [`Buf`]: crate::buf::Buf
//! [`Buf::load`]: crate::buf::Buf::load

pub use self::size::{DefaultSize, Size};
mod size;

pub use self::r#ref::Ref;
mod r#ref;

pub use self::pointee::{Packable, Pointee};
mod pointee;
