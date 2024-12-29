use crate::Context;

/// Trait used to decode a slice into a type.
pub trait DecodeSliceBuilder<T>: Sized {
    /// Construct a new empty container.
    fn new<C>(cx: &C) -> Result<Self, C::Error>
    where
        C: ?Sized + Context;

    /// Construct a new container with the given capacity hint.
    fn with_capacity<C>(cx: &C, hint: usize) -> Result<Self, C::Error>
    where
        C: ?Sized + Context;

    /// Push a value into the container.
    fn push<C>(&mut self, cx: &C, value: T) -> Result<(), C::Error>
    where
        C: ?Sized + Context;

    /// Reserve additional space for `capacity` elements in the collection.
    fn reserve<C>(&mut self, cx: &C, capacity: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context;

    /// Mark the given length as initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that elements up from `old_len..len` have been
    /// initialized.
    unsafe fn set_len(&mut self, len: usize);

    /// Get a mutable pointer to the first element in the collection.
    fn as_mut_ptr(&mut self) -> *mut T;
}
