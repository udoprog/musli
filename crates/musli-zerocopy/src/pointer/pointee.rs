use crate::ZeroCopy;

/// The trait for a value that can be pointed to by a [`Ref<T>`].
///
/// [`Ref<T>`]: crate::Ref
pub trait Pointee {
    /// The metadata of the pointee.
    type Metadata: ZeroCopy + Copy;
}

impl<T> Pointee for T
where
    T: ZeroCopy,
{
    type Metadata = ();
}
