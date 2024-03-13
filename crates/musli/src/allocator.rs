use crate::Buf;

/// An allocator that can be used in combination with a context.
///
/// # Examples
///
/// ```
/// use musli::{Allocator, Buf};
///
/// let mut buf = musli_allocator::buffer();
/// let alloc = musli_allocator::new(&mut buf);
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
