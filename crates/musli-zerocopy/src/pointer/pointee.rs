use crate::ZeroCopy;

/// The trait for a value that can be pointed to by a [`Ref<T>`].
///
/// [`Ref<T>`]: crate::Ref
pub trait Pointee {
    /// The metadata of the pointee.
    type Metadata<O>: Copy
    where
        O: Copy;
}

impl<T> Pointee for T
where
    T: ZeroCopy,
{
    type Metadata<O> = () where O: Copy;
}

impl<T> Pointee for [T]
where
    T: ZeroCopy,
{
    type Metadata<O> = O where O: Copy;
}

impl Pointee for str {
    type Metadata<O> = O where O: Copy;
}
