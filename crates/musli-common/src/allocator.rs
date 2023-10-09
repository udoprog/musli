//! Trait used to govern allocations.
//! Trait used to handle individual buffer allocations.

#[cfg(feature = "alloc")]
mod alloc;
#[cfg(feature = "alloc")]
pub use self::alloc::Alloc;

mod disabled;
pub use self::disabled::Disabled;

mod no_std;
pub use self::no_std::NoStd;

mod default_alloc {
    #![allow(missing_docs)]

    #[cfg(feature = "alloc")]
    pub type Default = super::Alloc;
    #[cfg(not(feature = "alloc"))]
    pub type Default = super::NoStd<1024>;
}

/// The default allocator.
///
/// The exact implementation depends on which features are enabled (first one
/// takes preference):
/// * If `alloc` is enabled, this is the [`Alloc`] allocator.
/// * Otherwise this is the [`NoStd`] allocator.
#[doc(inline)]
pub use self::default_alloc::Default;

use musli::context::Buffer;

/// An allocator that can be used in combination with a context.
///
/// # Examples
///
/// ```
/// use musli_common::allocator::{self, Allocator};
/// use musli::context::Buffer;
///
/// let alloc = musli_common::allocator::Default::default();
/// let alloc = &alloc;
///
/// let mut a = alloc.alloc();
/// let mut b = alloc.alloc();
///
/// b.write(b"He11o");
/// a.copy_back(b);
///
/// assert_eq!(unsafe { a.as_slice() }, b"He11o");
/// assert_eq!(a.len(), 5);
///
/// a.write(b" W0rld");
///
/// assert_eq!(unsafe { a.as_slice() }, b"He11o W0rld");
/// assert_eq!(a.len(), 11);
///
/// let mut c = alloc.alloc();
/// c.write(b"!");
/// assert!(a.write_at(7, b"o"));
/// assert!(!a.write_at(11, b"!"));
/// a.copy_back(c);
///
/// assert_eq!(unsafe { a.as_slice() }, b"He11o World!");
/// assert_eq!(a.len(), 12);
///
/// assert!(a.write_at(2, b"ll"));
///
/// assert_eq!(unsafe { a.as_slice() }, b"Hello World!");
/// assert_eq!(a.len(), 12);
/// ```
pub trait Allocator {
    /// An allocated buffer.
    type Buf: Buffer;

    /// Allocate an empty, uninitialized buffer. Just calling this function
    /// doesn't cause any allocations to occur, for that to happen the returned
    /// allocation has to be written to.
    fn alloc(&self) -> Self::Buf;
}

impl<A> Allocator for &A
where
    A: ?Sized + Allocator,
{
    type Buf = A::Buf;

    #[inline(always)]
    fn alloc(&self) -> Self::Buf {
        (*self).alloc()
    }
}
