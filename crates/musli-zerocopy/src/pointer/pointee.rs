use crate::ZeroCopy;

/// The trait for a value that can be pointed to by a [`Ref<T>`].
///
/// This ultimately determines the layout of [`Ref<T>`] as for unsized types it
/// needs to accommodate the size of the pointed-to type as well.
///
/// ```
/// use std::mem::size_of;
///
/// use musli_zerocopy::Ref;
///
/// assert_eq!(size_of::<Ref::<u32>>(), 4);
/// assert_eq!(size_of::<Ref::<[u32]>>(), 8);
/// ```
///
/// [`Ref<T>`]: crate::Ref
pub trait Pointee {
    /// Metadata associated with the pointee.
    type Metadata: Copy;

    /// The packed representation of the pointer metadata.
    type Packed<O>: Copy
    where
        O: Copy;
}

impl<T> Pointee for T
where
    T: ZeroCopy,
{
    type Metadata = ();
    type Packed<O> = () where O: Copy;
}

impl<T> Pointee for [T]
where
    T: ZeroCopy,
{
    type Metadata = usize;
    type Packed<O> = O where O: Copy;
}

impl Pointee for str {
    type Metadata = usize;
    type Packed<O> = O where O: Copy;
}
