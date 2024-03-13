//! Trait used to govern allocations.
//! Trait used to handle individual buffer allocations.

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
mod system;
#[cfg(feature = "alloc")]
pub use self::system::{System, SystemBuffer};

mod disabled;
pub use self::disabled::Disabled;

mod stack;
pub use self::stack::{Stack, StackBuffer};

#[cfg(feature = "alloc")]
mod default_alloc {
    #![allow(missing_docs)]

    use super::Allocator;

    pub struct DefaultBuffer(super::SystemBuffer);
    pub struct Default<'a>(super::System<'a>);

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer(super::SystemBuffer::new())
    }

    pub(super) fn new(DefaultBuffer(buf): &mut DefaultBuffer) -> Default<'_> {
        Default(super::System::new(buf))
    }

    impl<'a> Allocator for Default<'a> {
        type Buf<'this> = <super::System<'a> as super::Allocator>::Buf<'this>
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

    type InnerAllocator<'a> = super::Stack<'a>;

    pub struct DefaultBuffer(super::StackBuffer<4096>);
    pub struct Default<'a>(InnerAllocator<'a>);

    pub(super) fn buffer() -> DefaultBuffer {
        DefaultBuffer(super::StackBuffer::new())
    }

    pub(super) fn new(DefaultBuffer(buf): &mut DefaultBuffer) -> Default<'_> {
        Default(super::Stack::new(buf))
    }

    impl<'a> Allocator for Default<'a> {
        type Buf<'this> = <super::Stack<'a> as super::Allocator>::Buf<'this>
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
/// [`Stack`].
///
/// Requires that [`buffer()`] is used to construct the provided buffer.
pub fn new(buf: &mut DefaultBuffer) -> Default<'_> {
    self::default_alloc::new(buf)
}

/// The default allocator.
///
/// The exact implementation depends on which features are enabled (first one
/// takes preference):
/// * If `alloc` is enabled, this is the [`Alloc`] allocator.
/// * Otherwise this is the [`Stack`] allocator.
#[doc(inline)]
pub use self::default_alloc::{Default, DefaultBuffer};

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
