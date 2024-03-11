//! Trait used to govern allocations.
//! Trait used to handle individual buffer allocations.

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
mod alloc;
#[cfg(feature = "alloc")]
pub use self::alloc::{Alloc, HeapBuffer};

mod disabled;
pub use self::disabled::Disabled;

mod no_std;
pub use self::no_std::{NoStd, StackBuffer};

#[cfg(feature = "alloc")]
mod default_alloc {
    #![allow(missing_docs)]

    use super::Allocator;

    pub struct DefaultBuffer(super::HeapBuffer);
    pub struct DefaultAllocator<'a>(super::Alloc<'a>);

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer(super::HeapBuffer::new())
    }

    pub(super) fn alloc(DefaultBuffer(buf): &mut DefaultBuffer) -> DefaultAllocator<'_> {
        DefaultAllocator(super::Alloc::new(buf))
    }

    impl<'a> Allocator for DefaultAllocator<'a> {
        type Buf<'this> = <super::Alloc<'a> as super::Allocator>::Buf<'this>
        where
            Self: 'this;

        #[inline(always)]
        fn alloc(&self) -> Option<Self::Buf<'_>> {
            self.0.alloc()
        }
    }
}

#[cfg(not(feature = "alloc"))]
mod default_alloc {
    #![allow(missing_docs)]

    use super::Allocator;

    type InnerAllocator<'a> = super::NoStd<'a>;

    pub struct DefaultBuffer(super::StackBuffer<4096>);
    pub struct DefaultAllocator<'a>(InnerAllocator<'a>);

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer(super::StackBuffer::new())
    }

    pub(super) fn alloc(DefaultBuffer(buf): &mut DefaultBuffer) -> DefaultAllocator<'_> {
        DefaultAllocator(super::NoStd::new(buf))
    }

    impl<'a> Allocator for DefaultAllocator<'a> {
        type Buf<'this> = <super::NoStd<'a> as super::Allocator>::Buf<'this>
        where
            Self: 'this;

        #[inline(always)]
        fn alloc(&self) -> Option<Self::Buf<'_>> {
            self.0.alloc()
        }
    }
}

/// Construct a new default buffer.
///
/// Uses [`HeapBuffer`] if the `alloc` feature is enabled, otherwise
/// `StackBuffer` is used with a default size of `4096`.
pub fn buffer() -> DefaultBuffer {
    self::default_alloc::buffer()
}

/// Construct a new default allocator.
///
/// Uses the [`Alloc`] allocator if the `alloc` feature is enabled, otherwise
/// [`NoStd`].
///
/// Requires that [`buffer()`] is used to construct the provided buffer.
pub fn new(buf: &mut DefaultBuffer) -> DefaultAllocator<'_> {
    self::default_alloc::alloc(buf)
}

/// The default allocator.
///
/// The exact implementation depends on which features are enabled (first one
/// takes preference):
/// * If `alloc` is enabled, this is the [`Alloc`] allocator.
/// * Otherwise this is the [`NoStd`] allocator.
#[doc(inline)]
pub use self::default_alloc::{DefaultAllocator, DefaultBuffer};

use musli::context::Buf;

/// An allocator that can be used in combination with a context.
///
/// # Examples
///
/// ```
/// use musli_common::allocator::{self, Allocator};
/// use musli::context::Buf;
///
/// let mut buf = allocator::buffer();
/// let alloc = allocator::new(&mut buf);
///
/// let mut a = alloc.alloc().expect("allocation a failed");
/// let mut b = alloc.alloc().expect("allocation b failed");
///
/// b.write(b"He11o");
/// a.write(b.as_slice());
///
/// assert_eq!(a.as_slice(), b"He11o");
/// assert_eq!(a.len(), 5);
///
/// a.write(b" W0rld");
///
/// assert_eq!(a.as_slice(), b"He11o W0rld");
/// assert_eq!(a.len(), 11);
///
/// let mut c = alloc.alloc().expect("allocation c failed");
/// c.write(b"!");
/// a.write(c.as_slice());
///
/// assert_eq!(a.as_slice(), b"He11o W0rld!");
/// assert_eq!(a.len(), 12);
/// ```
pub trait Allocator {
    /// The type of an allocated buffer.
    type Buf<'this>: Buf
    where
        Self: 'this;

    /// Allocate an empty, uninitialized buffer.
    ///
    /// Calling this method returns `None` if the allocation failed.
    fn alloc(&self) -> Option<Self::Buf<'_>>;
}

impl<A> Allocator for &A
where
    A: ?Sized + Allocator,
{
    type Buf<'this> = A::Buf<'this> where Self: 'this;

    #[inline(always)]
    fn alloc(&self) -> Option<Self::Buf<'_>> {
        (*self).alloc()
    }
}
