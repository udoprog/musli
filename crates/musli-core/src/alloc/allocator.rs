use core::ptr::NonNull;

use super::{Alloc, AllocError};

/// The trait governing a global allocator.
///
/// # Safety
///
/// Implementing this trait promises that the implementing allocator ultimately
/// is backed by the global allocator [`Global`] and subsequently is what will
/// be used to allocate and free data.
///
/// [`Global`]: super::Global
pub unsafe trait GlobalAllocator
where
    Self: Allocator,
{
    /// Do not implement this trait. It is internal to Musli and will change.
    fn __do_not_implement();

    /// Construct a new instance of the global allocator.
    #[doc(hidden)]
    fn new() -> Self;

    /// Clone an allocation.
    fn clone_alloc<T>(alloc: &Self::Alloc<T>) -> Self::Alloc<T>;

    /// Construct an allocation from the given raw parts assuming they are
    /// allocated using the [`Global`] allocator.
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated by the [`Global`] allocator with the
    ///   layout of an array of `len` elements of `T`, or be dangling and
    ///   suitably aligned if that layout has a size of zero.
    /// - The returned allocation takes ownership of that memory, so it must not
    ///   be freed or used through any other means afterwards.
    ///
    /// Since the pointer is trusted, calling this requires `unsafe`:
    ///
    /// ```compile_fail,E0133
    /// use core::ptr::NonNull;
    ///
    /// use musli::alloc::{Global, GlobalAllocator};
    ///
    /// let alloc = Global::slice_from_raw_parts(NonNull::<u32>::dangling(), 4);
    /// ```
    ///
    /// [`Global`]: super::Global
    #[doc(hidden)]
    unsafe fn slice_from_raw_parts<T>(ptr: NonNull<T>, len: usize) -> Self::Alloc<T>;
}

/// An allocator which can be obtained from the allocator `A`.
///
/// Every [`Allocator`] can be obtained from itself, and the [`Global`]
/// allocator can be obtained from any allocator since it carries no state.
///
/// This is used when decoding a container which is tied to a particular
/// allocator. A decoder hands out its own allocator `A`, which the container
/// converts into the allocator it stores its data in. This is what allows a
/// field of type `Value<Global>` to be decoded by a decoder using any
/// allocator.
///
/// This is implemented for every allocator provided by Müsli, and only Müsli
/// can implement [`Allocator`], so it does not need to be implemented
/// manually.
///
/// # Examples
///
/// ```
/// use musli::alloc::{Allocator, FromAllocator, Global, Vec};
///
/// fn copy_to<A, B>(alloc: A, bytes: &[u8]) -> Vec<u8, B>
/// where
///     A: Allocator,
///     B: Allocator + FromAllocator<A>,
/// {
///     let mut out = Vec::new_in(B::from_allocator(alloc));
///     out.extend_from_slice(bytes).unwrap();
///     out
/// }
///
/// musli::alloc::default(|alloc| {
///     let bytes: Vec<u8, Global> = copy_to(alloc, b"hello");
///     assert_eq!(bytes.as_slice(), b"hello");
/// });
/// ```
///
/// [`Global`]: super::Global
pub trait FromAllocator<A> {
    /// Obtain this allocator from the allocator `alloc`.
    fn from_allocator(alloc: A) -> Self;
}

/// An allocator that can be used in combination with a context.
///
/// # Safety
///
/// Setting `IS_GLOBAL` to `true` has safety implications, since it determines
/// whether the allocation can be safely converted into a standard container or
/// not.
pub unsafe trait Allocator
where
    Self: Copy + FromAllocator<Self>,
{
    /// Do not implement this trait. It is internal to Musli and will change.
    fn __do_not_implement();

    /// Whether the allocations returned by this allocatore is backed by the
    /// global allocator or not.
    ///
    /// # Safety
    ///
    /// Setting this to `true` has safety implications, since it determines
    /// whether the allocation can be safely converted into a standard container
    /// or not.
    #[doc(hidden)]
    const IS_GLOBAL: bool;

    /// A raw allocation from the allocator.
    #[doc(hidden)]
    type Alloc<T>: Alloc<T>;

    /// Construct an empty uninitialized raw vector with an alignment matching
    /// that of `T` that is associated with this allocator.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::slice;
    /// use musli::alloc::{Alloc, AllocError, Allocator};
    ///
    /// let values: [u32; 4] = [1, 2, 3, 4];
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut buf = alloc.alloc_empty::<u32>();
    ///     let mut len = 0;
    ///
    ///     for value in values {
    ///         buf.resize(len, 1)?;
    ///
    ///         // SAFETY: We've just resized the above buffer.
    ///         unsafe {
    ///             buf.as_mut_ptr().add(len).write(value);
    ///         }
    ///
    ///         len += 1;
    ///     }
    ///
    ///     // SAFETY: Slice does not outlive the buffer it references.
    ///     let bytes = unsafe { slice::from_raw_parts(buf.as_ptr(), len) };
    ///     assert_eq!(bytes, values);
    ///     Ok::<_, AllocError>(())
    /// });
    /// # Ok::<_, AllocError>(())
    /// ```
    #[doc(hidden)]
    fn alloc_empty<T>(self) -> Self::Alloc<T>;

    /// Construct an empty uninitialized raw allocation with an alignment
    /// matching that of `T` that is associated with this allocator.
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
    #[doc(hidden)]
    fn alloc<T>(self, value: T) -> Result<Self::Alloc<T>, AllocError>;
}
