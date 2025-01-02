/// A value allocated through [`Allocator::alloc`].
///
/// [`Allocator::alloc`]: super::Allocator::alloc
///
/// ## Examples
///
/// ```
/// use musli::alloc::{AllocError, Allocator, Alloc};
///
/// musli::alloc::default(|alloc| {
///     let mut buf = alloc.alloc(10u32)?;
///
///     unsafe {
///         buf.as_mut_ptr().write(20u32);
///         assert_eq!(buf.as_ptr().read(), 20u32);
///     }
///
///     Ok::<_, AllocError>(())
/// });
/// # Ok::<_, AllocError>(())
/// ```
pub trait Alloc<T>
where
    T: ?Sized,
{
    /// Get a pointer into the allocation.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Allocator, Alloc};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut buf = alloc.alloc(10u32)?;
    ///
    ///     unsafe {
    ///         buf.as_mut_ptr().write(20u32);
    ///         assert_eq!(buf.as_ptr().read(), 20u32);
    ///     }
    ///
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    fn as_ptr(&self) -> *const T;

    /// Get a mutable pointer into the allocation.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::{AllocError, Allocator, Alloc};
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut buf = alloc.alloc(10u32)?;
    ///
    ///     unsafe {
    ///         buf.as_mut_ptr().write(20u32);
    ///         assert_eq!(buf.as_ptr().read(), 20u32);
    ///     }
    ///
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    fn as_mut_ptr(&mut self) -> *mut T;
}
